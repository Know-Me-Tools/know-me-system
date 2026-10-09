//! Shared C13 placement and durable-observation contract, independent of storage engines.
use serde::{Deserialize, Serialize};
use crate::CoreResult;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileKind { StandaloneEmbedded, Home, PersonalCloud, Enterprise }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecutionProfile {
    pub schema_version: u32,
    pub kind: ProfileKind,
    pub instance_id: String,
    pub service_url: Option<String>,
    /// Only newly entered input may leave the device; no implicit history or memory upload.
    pub remote_input_approved: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JobState { AdmissionPending, Running, ObservationDetached, Completed, Cancelled, Uncertain }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionJob {
    pub host_run_id: String,
    pub conversation_id: String,
    pub runtime_run_id: Option<String>,
    pub runtime_mode: String,
    pub profile: Option<ExecutionProfile>,
    pub cursor: Option<u64>,
    pub state: JobState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionFrame {
    pub host_run_id: String,
    pub source_id: Option<u64>,
    pub event: String,
    pub data: serde_json::Value,
}

/// Host persistence only. This port never starts or schedules an execution.
#[async_trait::async_trait]
pub trait ExecutionJournalStore: Send + Sync {
    async fn profile(&self) -> CoreResult<Option<ExecutionProfile>>;
    async fn set_profile(&self, profile: &ExecutionProfile) -> CoreResult<()>;
    async fn jobs(&self) -> CoreResult<Vec<ExecutionJob>>;
    /// Compare-and-set; None expects a new identity. Existing admission is never overwritten.
    async fn put_job(&self, previous: Option<&ExecutionJob>, next: &ExecutionJob) -> CoreResult<bool>;
    /// Persist the raw frame and advanced job cursor together before UI projection.
    async fn append_frame(&self, previous: &ExecutionJob, next: &ExecutionJob, frame: &ExecutionFrame) -> CoreResult<bool>;
    async fn frames(&self, host_run_id: &str) -> CoreResult<Vec<ExecutionFrame>>;
}
