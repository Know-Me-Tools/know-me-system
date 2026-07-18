// TJ-ARCH-MOB-001 compliant
//! **LEGACY (Electric lane).** Superseded by `frf_transport.rs` — the substrate moved
//! to flint-realtime-fabric on 2026-07-16 (C-106) and nothing on the FRF path uses this.
//! Kept until its removal is scheduled; do not build on it.
//!
//! Electric HTTP shape consumer. Long-polls the Electric `/v1/shape` endpoint,
//! tracks `(handle, offset)`, and applies decoded rows to a [`LocalStore`].
//!
//! Wire protocol (Electric v1.x): the initial request uses `offset=-1`; responses
//! carry `electric-handle` + `electric-offset` headers and a JSON array of change
//! messages (`{ headers.operation, key, value }`) interleaved with control
//! messages (`{ headers.control: "up-to-date" | "must-refetch" }`). Subsequent
//! requests add `handle=<h>&offset=<o>&live=true` to long-poll. A `must-refetch`
//! control message (or HTTP 409) means the shape rotated: truncate locally and
//! restart from `offset=-1` with the fresh handle.
use super::config::ShapeSpec;
use super::seam::{LocalStore, RowChange, RowOp};
use super::status::SyncStatusHandle;
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::sync::SyncStatus;
use serde::Deserialize;
use std::sync::Arc;

/// A shape message: either a row operation or a control frame.
#[derive(Debug, Deserialize)]
struct ShapeMessage {
    #[serde(default)]
    headers: MsgHeaders,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    value: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
struct MsgHeaders {
    #[serde(default)]
    operation: Option<String>,
    #[serde(default)]
    control: Option<String>,
}

/// Tracks position within one shape's log.
struct ShapeCursor {
    handle: Option<String>,
    offset: String,
}

impl ShapeCursor {
    /// Fresh cursor — Electric's initial-sync sentinel offset is `-1`.
    fn initial() -> Self {
        Self {
            handle: None,
            offset: "-1".to_string(),
        }
    }
}

pub(crate) struct ShapeConsumer {
    client: reqwest::Client,
    electric_url: String,
    shape: ShapeSpec,
    store: Arc<dyn LocalStore>,
    status: SyncStatusHandle,
}

impl ShapeConsumer {
    pub(crate) fn new(
        client: reqwest::Client,
        electric_url: String,
        shape: ShapeSpec,
        store: Arc<dyn LocalStore>,
        status: SyncStatusHandle,
    ) -> Self {
        Self {
            client,
            electric_url,
            shape,
            store,
            status,
        }
    }

    /// Consume the shape until the task is cancelled (drop of the join handle) or a
    /// terminal error. Loops: catch up → long-poll live → apply → repeat, handling
    /// `must-refetch` by truncating and resetting the cursor.
    pub(crate) async fn run(&self) -> CoreResult<()> {
        let mut cursor = ShapeCursor::initial();
        loop {
            let live = cursor.handle.is_some(); // only long-poll once we have a handle
            let (messages, next) = self.poll(&cursor, live).await?;

            let mut changes = Vec::new();
            let mut must_refetch = false;
            for msg in &messages {
                if let Some(control) = &msg.headers.control {
                    match control.as_str() {
                        "up-to-date" => self.status.set(SyncStatus::Live),
                        "must-refetch" => {
                            must_refetch = true;
                            break;
                        }
                        _ => {} // unknown control frame — ignore forward-compatibly
                    }
                    continue;
                }
                if let Some(change) = decode_row(&self.shape.table, msg) {
                    changes.push(change);
                }
            }

            if must_refetch {
                tracing::warn!(table = %self.shape.table, "shape rotated; refetching");
                self.store.truncate_shape(&self.shape.table).await?;
                cursor = ShapeCursor::initial();
                continue;
            }

            if !changes.is_empty() {
                self.store.apply_batch(&changes).await?;
            }
            cursor = next;
        }
    }

    /// One HTTP request against the shape endpoint. Returns decoded messages and the
    /// advanced cursor. HTTP 409 is Electric's shape-rotation signal → surface it as
    /// an empty batch with a reset cursor so `run` re-materialises.
    async fn poll(
        &self,
        cursor: &ShapeCursor,
        live: bool,
    ) -> CoreResult<(Vec<ShapeMessage>, ShapeCursor)> {
        let mut req = self
            .client
            .get(format!("{}/v1/shape", self.electric_url))
            .query(&[
                ("table", self.shape.table.as_str()),
                ("offset", cursor.offset.as_str()),
            ]);
        if let Some(handle) = &cursor.handle {
            req = req.query(&[("handle", handle.as_str())]);
        }
        if let Some(where_clause) = &self.shape.where_clause {
            req = req.query(&[("where", where_clause.as_str())]);
        }
        if live {
            req = req.query(&[("live", "true")]);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;

        // 409 = shape handle rotated: reset to initial and let run() refetch.
        if resp.status().as_u16() == 409 {
            return Ok((Vec::new(), ShapeCursor::initial()));
        }
        if !resp.status().is_success() {
            return Err(CoreError::Transient(format!(
                "shape http {}",
                resp.status()
            )));
        }

        let handle = header(&resp, "electric-handle").or_else(|| cursor.handle.clone());
        let offset = header(&resp, "electric-offset").unwrap_or_else(|| cursor.offset.clone());

        let body = resp
            .text()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        // Empty body on a live long-poll timeout = no new data; keep the cursor.
        let messages: Vec<ShapeMessage> = if body.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&body).map_err(|e| CoreError::Serde(e.to_string()))?
        };

        Ok((messages, ShapeCursor { handle, offset }))
    }
}

fn header(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// Decode one shape row message into a [`RowChange`]. Returns `None` for messages
/// without a usable operation (already filtered control frames upstream).
fn decode_row(table: &str, msg: &ShapeMessage) -> Option<RowChange> {
    let op = match msg.headers.operation.as_deref()? {
        "insert" => RowOp::Insert,
        "update" => RowOp::Update,
        "delete" => RowOp::Delete,
        _ => return None,
    };
    let key = msg.key.clone().unwrap_or_default();
    let value_json = msg
        .value
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());
    Some(RowChange {
        table: table.to_string(),
        op,
        key,
        value_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Boundary behavior: an Electric change message decodes to the right RowOp;
    // a control frame (no operation) decodes to None so run() skips it as a row.
    #[test]
    fn decodes_insert_and_skips_control_frames() {
        let insert: ShapeMessage = serde_json::from_str(
            r#"{"headers":{"operation":"insert"},"key":"\"e1\"","value":{"id":"e1"}}"#,
        )
        .expect("insert message parses");
        let row = decode_row("entities", &insert).expect("insert decodes to a row");
        assert_eq!(row.op, RowOp::Insert);
        assert_eq!(row.table, "entities");

        let up_to_date: ShapeMessage =
            serde_json::from_str(r#"{"headers":{"control":"up-to-date"}}"#)
                .expect("control message parses");
        assert!(decode_row("entities", &up_to_date).is_none());
    }
}
