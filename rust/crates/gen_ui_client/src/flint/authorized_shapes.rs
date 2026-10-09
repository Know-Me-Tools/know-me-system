//! Reads selected server-catalog shapes through Gate's authorized Fabric facade.
//! No caller SQL predicate, table name or tenant header is sent to the authority.
use frf_sdk_rust::{ShapeClient, ShapeCursor, ShapeRequestOptions, SdkError};
use gen_ui_types::{CoreError, CoreResult};
use gen_ui_types::sync::{AuthorizedReadSource, ReadCursor, ReadPoll, ReplicaChange};

pub struct FabricShapeSource {
    client: ShapeClient,
    shape: String,
    table: String,
}

impl FabricShapeSource {
    pub fn new(gate_url: String, token: String, shape: String, table: String) -> CoreResult<Self> {
        if token.is_empty() || shape.is_empty() || !matches!(table.as_str(), "notes" | "memories") {
            return Err(CoreError::Terminal("authorized sync requires a token and a selected business shape".into()));
        }
        let client = ShapeClient::new(gate_url, Some(token))
            .map_err(|_| CoreError::Terminal("invalid Fabric shape endpoint".into()))?;
        Ok(Self { client, shape, table })
    }
}

#[async_trait::async_trait]
impl AuthorizedReadSource for FabricShapeSource {
    fn shape_id(&self) -> &str { &self.shape }
    fn table(&self) -> &str { &self.table }

    async fn poll(&self, cursor: Option<&ReadCursor>) -> CoreResult<ReadPoll> {
        let cursor = cursor.map(|c| ShapeCursor { handle: c.handle.clone(), offset: c.offset.clone() });
        let options = ShapeRequestOptions { live: cursor.is_some(), ..Default::default() };
        let frame = match self.client.fetch_shape(&self.shape, cursor.as_ref(), &options).await {
            Ok(frame) => frame,
            Err(SdkError::ShapeForbidden) => return Ok(ReadPoll::Revoked { reason: "Selected read grant was revoked or refused".into() }),
            Err(SdkError::ShapeGrantExpired) => return Ok(ReadPoll::Revoked { reason: "Read grant expired; authenticate again".into() }),
            Err(SdkError::UnknownShape(_)) | Err(SdkError::InvalidShapeRequest(_)) => {
                return Err(CoreError::Terminal("selected Fabric shape is unavailable or invalid".into()));
            }
            Err(_) => return Err(CoreError::Transient("Fabric read connection unavailable".into())),
        };
        if frame.must_refetch { return Ok(ReadPoll::Resnapshot); }
        let mut changes = Vec::with_capacity(frame.messages.len());
        for message in frame.messages {
            let deleted = message.is_delete();
            let data = message.value.unwrap_or_else(|| serde_json::json!({}));
            // Catalog business projections must include their row identity. Electric's
            // protocol key can be a composite encoded key and is never treated as SQL.
            let id = data.get("id").and_then(serde_json::Value::as_str)
                .ok_or_else(|| CoreError::Terminal("business shape omitted its row identity".into()))?.to_owned();
            changes.push(ReplicaChange { id, deleted, data });
        }
        Ok(ReadPoll::Batch {
            changes,
            cursor: frame.cursor.map(|c| ReadCursor { handle: c.handle, offset: c.offset }),
            caught_up: frame.up_to_date,
        })
    }
}
