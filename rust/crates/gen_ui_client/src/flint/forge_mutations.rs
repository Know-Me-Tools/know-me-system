//! C11 receipt-backed mutations. Requires Forge ed72018 (migration 0020).
//! Repeating an identical request reads its committed response from the server
//! receipt, including when the previous HTTP response was lost after commit.
use super::{row_to_record, map_status, ForgeClient};
use gen_ui_types::{CoreError, CoreResult};
use gen_ui_types::transport::EntityRecord;

impl ForgeClient {
    pub(super) async fn replay_mutation(
        &self,
        operation: &str,
        record: &EntityRecord,
        key: &str,
    ) -> CoreResult<Option<EntityRecord>> {
        if key.is_empty() {
            return Err(CoreError::Terminal("durable command identity is required".into()));
        }
        let mut url = reqwest::Url::parse(&self.table_url(&record.entity_type))
            .map_err(|_| CoreError::Terminal("invalid Forge endpoint".into()))?;
        // Path segments encode row identifiers; IDs never change the target route.
        if matches!(operation, "update" | "delete") {
            url.path_segments_mut().map_err(|_| CoreError::Terminal("invalid Forge endpoint".into()))?
                .push(&record.id);
        }
        let request = match operation {
            "insert" | "upsert" | "update" => {
                let mut value: serde_json::Value = serde_json::from_str(&record.data_json)
                    .map_err(|e| CoreError::Serde(e.to_string()))?;
                let object = value.as_object_mut().ok_or_else(|| CoreError::Terminal("mutation data must be an object".into()))?;
                object.insert("id".into(), serde_json::json!(record.id));
                let request = if operation == "update" { self.http.patch(url) } else { self.http.post(url) };
                request.json(&value)
            }
            "delete" => self.http.delete(url),
            _ => return Err(CoreError::Terminal("unknown mutation operation".into())),
        };
        let response = self.apply_bearer(request)
            .header("Idempotency-Key", key)
            .header("Prefer", "return=representation")
            .send().await.map_err(|e| CoreError::Transient(e.without_url().to_string()))?;
        if !response.status().is_success() {
            return Err(map_status(response.status()));
        }
        if operation == "delete" { return Ok(None); }
        let mut rows: Vec<serde_json::Value> = response.json().await
            .map_err(|_| CoreError::Transient("committed mutation response unavailable; replay receipt".into()))?;
        let row = rows.pop().ok_or_else(|| CoreError::Terminal("mutation affected no authorized row".into()))?;
        Ok(Some(row_to_record(&record.entity_type, row)))
    }
}
