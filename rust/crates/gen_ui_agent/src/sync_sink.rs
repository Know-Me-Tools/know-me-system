// TJ-ARCH-MOB-001 compliant
//! [`WriteSink`] over the forge/Quarry data plane (C-106 T3c) — the write half of
//! local-first sync.
//!
//! **Why this lives in gen_ui_agent (L3) and not in gen_ui_db (L2):** the `WriteSink`
//! trait is declared in `gen_ui_db::sync`, and the client that satisfies it is
//! `gen_ui_client::flint::forge`. Those two are L2 siblings — neither depends on the
//! other, and making one do so would put a cycle in the layer graph. L3 is the first
//! layer that legitimately sees both, so the wiring belongs here. That is exactly what
//! the seam's own doc comment anticipated ("C-006's forge client implements this").
//!
//! **Why forge and NOT the FRF spine** (verified against FRF @ 9ba04ae, 2026-07-16):
//! `SpineService::Publish` writes to the Iggy broker only — there is no spine→Postgres
//! writer anywhere in the FRF workspace — and `EntityService` is read-only
//! (`GetEntity`/`WatchEntity`). CDC is therefore strictly **Postgres → spine**. A write
//! published to the spine would fan out to live subscribers and then vanish: it never
//! lands in Postgres, so it is absent from the next re-materialisation and never reaches
//! a device that was offline. That failure mode looks like a working demo and silently
//! loses data. Writes go to Postgres through Quarry's PostgREST surface; CDC picks them
//! up from the WAL and fans them back out. That is what closes the loop.
use std::sync::Arc;

use gen_ui_db::sync::{PendingWrite, WriteOutcome, WriteSink};
use gen_ui_types::error::CoreError;
use gen_ui_types::transport::{EntityRecord, EntityTransport};

/// The mutation carried in a [`PendingWrite::change_json`].
///
/// The queue treats `change_json` as opaque; this is the shape the *sink* agrees to.
/// `op` mirrors the CDC vocabulary so a local write and a replayed server change speak
/// the same language.
#[derive(Debug, serde::Deserialize)]
struct WriteBody {
    /// `insert` | `update` | `delete` — matches FRF's `ChangeOp` wire spelling.
    op: String,
    /// Row primary key. Client-generated, so an offline insert already knows its id.
    id: String,
    /// Full row for insert/update; ignored for delete.
    #[serde(default)]
    data: serde_json::Value,
}

/// Replays queued local writes through forge/Quarry.
pub struct ForgeWriteSink<T: EntityTransport> {
    forge: Arc<T>,
}

impl<T: EntityTransport> ForgeWriteSink<T> {
    #[must_use]
    pub fn new(forge: Arc<T>) -> Self {
        Self { forge }
    }
}

/// Classify a failed write.
///
/// The queue's whole contract rests on this being right: [`WriteOutcome::Retry`] keeps a
/// write forever until it succeeds, and [`WriteOutcome::Poison`] drops it from the happy
/// path. Getting it backwards either wedges the queue on a permanently-invalid row or
/// silently discards a good write on a transient blip.
fn classify(err: &CoreError) -> WriteOutcome {
    match err {
        // Network / 5xx / 429 — the write is fine, the world is busy.
        CoreError::Transient(e) => {
            tracing::debug!(error = %e, "write replay: transient, will retry");
            WriteOutcome::Retry
        }
        // 4xx validation, auth, schema mismatch. Retrying replays the same bytes and
        // gets the same rejection, forever.
        CoreError::Terminal(e) => WriteOutcome::Poison { reason: e.clone() },
        // A malformed body cannot fix itself either.
        CoreError::Serde(e) => WriteOutcome::Poison {
            reason: format!("serde: {e}"),
        },
        // NotFound on update/delete: the row is already gone server-side. Treat as
        // poison, not retry — but see `send`, which converts delete-NotFound to Applied
        // before we ever get here, because a delete whose target is absent has already
        // achieved what it wanted.
        CoreError::NotFound(e) => WriteOutcome::Poison {
            reason: format!("not found: {e}"),
        },
        CoreError::Io(e) => {
            tracing::debug!(error = %e, "write replay: io, will retry");
            WriteOutcome::Retry
        }
    }
}

#[async_trait::async_trait]
impl<T: EntityTransport + Send + Sync> WriteSink for ForgeWriteSink<T> {
    async fn send(&self, write: &PendingWrite) -> WriteOutcome {
        let body: WriteBody = match serde_json::from_str(&write.change_json) {
            Ok(b) => b,
            // Unparseable payload: quarantine rather than retry a body we cannot read.
            Err(e) => {
                return WriteOutcome::Poison {
                    reason: format!("bad write body: {e}"),
                };
            }
        };

        let record = EntityRecord {
            id: body.id.clone(),
            entity_type: write.table.clone(),
            data_json: body.data.to_string(),
        };

        let result = match body.op.as_str() {
            // Insert and update are both upserts server-side for the same reason they are
            // locally (see PgLocalStore): a replayed insert must not fail just because a
            // previous attempt already landed. `create` carries the client-generated id,
            // so Quarry's ON CONFLICT does the dedup — which is also why replay is safe
            // even before the idempotency key is consulted.
            "insert" | "upsert" => self.forge.create(&record).await.map(|_| ()),
            "update" => self.forge.update(&record).await.map(|_| ()),
            "delete" => match self.forge.delete(&write.table, &body.id).await {
                // Already deleted server-side: the write's intent is satisfied. Reporting
                // this as a failure would quarantine a write that actually succeeded.
                Err(CoreError::NotFound(_)) => Ok(()),
                other => other,
            },
            other => {
                return WriteOutcome::Poison {
                    reason: format!("unknown write op: {other}"),
                };
            }
        };

        match result {
            Ok(()) => WriteOutcome::Applied,
            Err(e) => classify(&e),
        }
    }
}

/// Build the production sink: a forge/Quarry client wrapped as a [`WriteSink`].
///
/// Lives here rather than in the platform leaves so neither the Tauri plugin nor the FFI
/// crate needs to depend on `gen_ui_client`/`reqwest`/`parking_lot` just to assemble one
/// — the leaves stay thin command surfaces (Rule 2/3), which is the whole point of L3.
///
/// `bearer` is the gate-minted JWT. `None` is legitimate for a local demo stack with no
/// auth proxy in front of Quarry; a deployed configuration must supply one. A malformed
/// token is an error rather than a silent downgrade to unauthenticated — a sync engine
/// that quietly writes as anon when it was told to authenticate is a data-leak shape.
pub fn forge_write_sink(
    base: impl Into<String>,
    schema: impl Into<String>,
    bearer: Option<String>,
) -> Result<Arc<dyn WriteSink>, CoreError> {
    use gen_ui_client::flint::forge::{ForgeClient, ForgeConfig};
    use gen_ui_client::flint::{FlintAuthState, Token};

    let auth = match bearer {
        Some(raw) => FlintAuthState::Authenticated {
            token: Token::parse(raw)?,
        },
        None => {
            tracing::warn!("forge sink: no bearer — writes go to Quarry unauthenticated");
            FlintAuthState::Unauthenticated
        }
    };
    let client = ForgeClient::new(
        reqwest::Client::new(),
        ForgeConfig {
            base: base.into(),
            schema: schema.into(),
        },
        Arc::new(parking_lot::RwLock::new(auth)),
    );
    Ok(Arc::new(ForgeWriteSink::new(Arc::new(client))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gen_ui_types::error::CoreResult;
    use gen_ui_types::transport::ListResult;
    use gen_ui_types::view::ViewDescriptor;
    use std::sync::Mutex;

    /// Fake at the real IO boundary (the HTTP transport), not a mock of internal code.
    #[derive(Default)]
    struct FakeForge {
        calls: Mutex<Vec<String>>,
        fail_with: Option<CoreError>,
    }

    impl FakeForge {
        fn failing(e: CoreError) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fail_with: Some(e),
            }
        }
        fn err(&self) -> CoreResult<()> {
            match &self.fail_with {
                Some(e) => Err(clone_err(e)),
                None => Ok(()),
            }
        }
    }

    fn clone_err(e: &CoreError) -> CoreError {
        match e {
            CoreError::Transient(s) => CoreError::Transient(s.clone()),
            CoreError::Terminal(s) => CoreError::Terminal(s.clone()),
            CoreError::NotFound(s) => CoreError::NotFound(s.clone()),
            CoreError::Serde(s) => CoreError::Serde(s.clone()),
            CoreError::Io(s) => CoreError::Io(s.clone()),
        }
    }

    #[async_trait::async_trait]
    impl EntityTransport for FakeForge {
        async fn list(&self, _v: &ViewDescriptor) -> CoreResult<ListResult> {
            Ok(ListResult {
                items: vec![],
                next_cursor: None,
            })
        }
        async fn get(&self, _t: &str, _id: &str) -> CoreResult<Option<EntityRecord>> {
            Ok(None)
        }
        async fn create(&self, r: &EntityRecord) -> CoreResult<EntityRecord> {
            self.calls.lock().unwrap().push(format!("create:{}", r.id));
            self.err()?;
            Ok(r.clone())
        }
        async fn update(&self, r: &EntityRecord) -> CoreResult<EntityRecord> {
            self.calls.lock().unwrap().push(format!("update:{}", r.id));
            self.err()?;
            Ok(r.clone())
        }
        async fn delete(&self, _t: &str, id: &str) -> CoreResult<()> {
            self.calls.lock().unwrap().push(format!("delete:{id}"));
            self.err()
        }
    }

    fn write(op: &str) -> PendingWrite {
        PendingWrite {
            idempotency_key: "k1".into(),
            table: "notes".into(),
            change_json: format!(r#"{{"op":"{op}","id":"n1","data":{{"id":"n1","title":"x"}}}}"#),
            attempts: 0,
        }
    }

    #[tokio::test]
    async fn insert_creates_and_reports_applied() {
        let forge = Arc::new(FakeForge::default());
        let sink = ForgeWriteSink::new(Arc::clone(&forge));
        assert_eq!(sink.send(&write("insert")).await, WriteOutcome::Applied);
        assert_eq!(forge.calls.lock().unwrap().as_slice(), ["create:n1"]);
    }

    #[tokio::test]
    async fn transient_failure_retries_rather_than_poisons() {
        // The write is good; the network is not. Poisoning here would drop real data.
        let forge = Arc::new(FakeForge::failing(CoreError::Transient("503".into())));
        let sink = ForgeWriteSink::new(forge);
        assert_eq!(sink.send(&write("insert")).await, WriteOutcome::Retry);
    }

    #[tokio::test]
    async fn terminal_failure_poisons_rather_than_retrying_forever() {
        // Replaying the same invalid bytes gets the same 4xx until the heat death of
        // the universe — quarantine instead of wedging the queue.
        let forge = Arc::new(FakeForge::failing(CoreError::Terminal(
            "400 invalid".into(),
        )));
        let sink = ForgeWriteSink::new(forge);
        assert!(matches!(
            sink.send(&write("insert")).await,
            WriteOutcome::Poison { .. }
        ));
    }

    #[tokio::test]
    async fn deleting_an_already_deleted_row_is_success_not_failure() {
        // The intent ("this row should be gone") is satisfied. Quarantining would flag a
        // write that actually achieved its goal.
        let forge = Arc::new(FakeForge::failing(CoreError::NotFound("gone".into())));
        let sink = ForgeWriteSink::new(forge);
        assert_eq!(sink.send(&write("delete")).await, WriteOutcome::Applied);
    }

    #[tokio::test]
    async fn unparseable_body_is_quarantined_not_retried() {
        let forge = Arc::new(FakeForge::default());
        let sink = ForgeWriteSink::new(forge);
        let bad = PendingWrite {
            idempotency_key: "k".into(),
            table: "notes".into(),
            change_json: "not json".into(),
            attempts: 0,
        };
        assert!(matches!(sink.send(&bad).await, WriteOutcome::Poison { .. }));
    }
}
