use std::{collections::BTreeMap, sync::Arc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::{error::AgentError, state, uar::{UarRuntimeFacade, UarServiceRuntime, UarEventProjector}};
use super::{ExecutionJob, JobState, ExecutionProfile, ProfileKind, ExecutionFrame};

const JOBS: &str = "afc.execution-jobs.v1";
static WRITER: Mutex<()> = Mutex::const_new(());
static OBSERVERS: std::sync::Mutex<std::collections::BTreeSet<String>> = std::sync::Mutex::new(std::collections::BTreeSet::new());
pub(crate) struct ObservationLease(String);
impl Drop for ObservationLease { fn drop(&mut self) { if let Ok(mut owners) = OBSERVERS.lock() { owners.remove(&self.0); } } }
pub(crate) fn claim_observation(host: &str) -> Result<Option<ObservationLease>, AgentError> {
    let mut owners = OBSERVERS.lock().map_err(|_| AgentError::Interaction("EXECUTION_OBSERVER_UNAVAILABLE".into()))?;
    Ok(owners.insert(host.into()).then(|| ObservationLease(host.into())))
}
#[derive(Default, Serialize, Deserialize)]
struct Journal { jobs: BTreeMap<String, ExecutionJob>, frames: BTreeMap<String, Vec<ExecutionFrame>> }
async fn load() -> Result<Journal, AgentError> {
    match state::config()?.get_setting(JOBS).await? {
        Some(value) => serde_json::from_value(value).map_err(|_| AgentError::Config("EXECUTION_JOURNAL_INVALID".into())),
        None => Ok(Journal::default()),
    }
}
async fn save(value: &Journal) -> Result<(), AgentError> {
    state::config()?.set_setting(JOBS, serde_json::to_value(value)
        .map_err(|_| AgentError::Config("EXECUTION_JOURNAL_INVALID".into()))?).await
}
pub async fn jobs() -> Result<Vec<ExecutionJob>, AgentError> { Ok(load().await?.jobs.into_values().collect()) }
pub(super) async fn begin(id: &str, conversation: &str, mode: &str, profile: Option<ExecutionProfile>) -> Result<(), AgentError> {
    let _writer = WRITER.lock().await;
    let mut values = load().await?;
    if values.jobs.contains_key(id) || values.jobs.values().any(|job| job.conversation_id == conversation &&
        matches!(job.state, JobState::AdmissionPending | JobState::Running | JobState::ObservationDetached)) {
        return Err(AgentError::Interaction("EXECUTION_ADMISSION_ALREADY_RECORDED".into()));
    }
    values.jobs.insert(id.into(), ExecutionJob { host_run_id: id.into(), conversation_id: conversation.into(),
        runtime_run_id: None, runtime_mode: mode.into(), profile, cursor: None, state: JobState::AdmissionPending });
    save(&values).await
}
pub(super) async fn admitted(id: &str, runtime: &str) -> Result<(), AgentError> {
    let _writer = WRITER.lock().await;
    let mut values = load().await?;
    let job = values.jobs.get_mut(id).ok_or_else(|| AgentError::Interaction("EXECUTION_JOB_UNAVAILABLE".into()))?;
    if job.runtime_run_id.as_deref().is_some_and(|previous| previous != runtime) {
        return Err(AgentError::Interaction("EXECUTION_IDENTITY_CONFLICT".into()));
    }
    job.runtime_run_id = Some(runtime.into()); job.state = JobState::Running;
    save(&values).await
}
pub(super) async fn admitted_session(session: Option<&str>, runtime: &str) -> Result<(), AgentError> {
    if state::get().is_err() { return Ok(()); }
    let Some(session) = session else { return Ok(()); };
    let job = load().await?.jobs.into_values().find(|job|
        job.conversation_id == session && job.state == JobState::AdmissionPending);
    if let Some(job) = job { admitted(&job.host_run_id, runtime).await?; }
    Ok(())
}
pub(super) async fn record_frame(runtime: &str, source_id: Option<u64>, event: &str, data: &serde_json::Value) -> Result<(), AgentError> {
    if state::get().is_err() { return Ok(()); }
    let _writer = WRITER.lock().await;
    let mut values = load().await?;
    let Some(job) = values.jobs.values_mut().find(|job| job.runtime_run_id.as_deref() == Some(runtime)) else { return Ok(()); };
    if source_id.is_some_and(|id| job.cursor.is_some_and(|previous| id <= previous)) { return Ok(()); }
    let host = job.host_run_id.clone();
    if let Some(id) = source_id { job.cursor = Some(id); }
    values.frames.entry(host.clone()).or_default().push(ExecutionFrame {
        host_run_id: host, source_id, event: event.into(), data: data.clone() });
    save(&values).await
}
pub(super) async fn terminal(id: &str, success: bool) -> Result<(), AgentError> {
    let _writer = WRITER.lock().await;
    let mut values = load().await?;
    if let Some(job) = values.jobs.get_mut(id) {
        if job.state != JobState::Cancelled { job.state = if success { JobState::Completed } else { JobState::Uncertain }; }
        save(&values).await?;
    }
    Ok(())
}
pub(super) async fn observation_detached(host: &str) -> bool {
    load().await.ok().and_then(|values| values.jobs.get(host).cloned())
        .is_some_and(|job| job.state == JobState::ObservationDetached)
}
/// Detach presentation only; the native collector remains connected and journals.
pub async fn suspend(id: &str) -> Result<ExecutionJob, AgentError> {
    let _writer = WRITER.lock().await;
    let mut values = load().await?;
    let job = values.jobs.get_mut(id).ok_or_else(|| AgentError::Interaction("EXECUTION_JOB_UNAVAILABLE".into()))?;
    if !matches!(job.state, JobState::Running | JobState::ObservationDetached) {
        return Err(AgentError::Interaction("EXECUTION_JOB_NOT_OBSERVABLE".into()));
    }
    job.state = JobState::ObservationDetached;
    let result = job.clone(); save(&values).await?; Ok(result)
}
async fn runtime(job: &ExecutionJob) -> Result<Arc<dyn UarRuntimeFacade>, AgentError> {
    match job.profile.as_ref() {
        Some(profile) if profile.kind == ProfileKind::StandaloneEmbedded => Ok(state::embedded_uar()?.clone()),
        Some(profile) => {
            let current = super::current().await?.ok_or_else(|| AgentError::Interaction("EXECUTION_BINDING_UNAVAILABLE".into()))?;
            if &current != profile { return Err(AgentError::Interaction("EXECUTION_BINDING_CHANGED_RESTORE_ORIGINAL".into())); }
            let config = crate::chat::load_uar_service_chat_config().await?;
            super::guard_remote(&config.service_url).await?;
            Ok(Arc::new(UarServiceRuntime::new(&config.service_url, config.api_key)?))
        }
        None if job.runtime_mode == "embedded_library" => Ok(state::embedded_uar()?.clone()),
        None => Err(AgentError::Interaction("EXECUTION_UNPINNED_REMOTE_OBSERVATION_UNSUPPORTED".into())),
    }
}
/// Reattach to the admitted ID and replay journaled output; never create another run.
pub async fn resume(id: &str) -> Result<ExecutionJob, AgentError> {
    let journal = load().await?;
    let job = journal.jobs.get(id).cloned().ok_or_else(|| AgentError::Interaction("EXECUTION_JOB_UNAVAILABLE".into()))?;
    if !matches!(job.state, JobState::Running | JobState::ObservationDetached | JobState::Uncertain | JobState::Completed) {
        return Err(AgentError::Interaction("EXECUTION_JOB_NOT_OBSERVABLE".into()));
    }
    let run_id = job.runtime_run_id.clone().ok_or_else(|| AgentError::Interaction("EXECUTION_ADMISSION_UNKNOWN_DO_NOT_RETRY".into()))?;
    let mut projector = UarEventProjector::new(run_id.clone());
    for frame in journal.frames.get(id).into_iter().flatten() {
        for event in projector.ingest_agui_payload(&frame.data) {
            state::publish(crate::chat::event_with_run_id(event, &job.host_run_id));
        }
    }
    if job.state == JobState::Completed { return Ok(job); }
    let Some(lease) = claim_observation(id)? else {
        admitted(id, &run_id).await?;
        return Ok(job);
    };
    let projection = runtime(&job).await?.observe_run(&run_id, job.cursor).await?;
    let host = job.host_run_id.clone();
    admitted(&host, &run_id).await?;
    gen_ui_runtime::spawn(async move {
        let _lease = lease;
        let mut events = projection.events;
        let mut completed = false;
        while let Some(event) = events.next().await {
            match event {
                Ok(event) => {
                    completed |= matches!(&event, gen_ui_types::events::A2uiEvent::RunFinished { .. });
                    if !observation_detached(&host).await { state::publish(crate::chat::event_with_run_id(event, &host)); }
                    if completed { break; }
                },
                Err(_) => { let _ = terminal(&host, false).await; return; }
            }
        }
        let _ = terminal(&host, completed).await;
    });
    Ok(job)
}
pub async fn cancel(id: &str) -> Result<ExecutionJob, AgentError> {
    let job = load().await?.jobs.remove(id).ok_or_else(|| AgentError::Interaction("EXECUTION_JOB_UNAVAILABLE".into()))?;
    let run = job.runtime_run_id.as_deref().ok_or_else(|| AgentError::Interaction("EXECUTION_ADMISSION_UNKNOWN_DO_NOT_RETRY".into()))?;
    if !runtime(&job).await?.cancel_run(run).await? { return Err(AgentError::Interaction("EXECUTION_CANCEL_NOT_CONFIRMED".into())); }
    let _writer = WRITER.lock().await;
    let mut values = load().await?;
    let selected = values.jobs.get_mut(id).ok_or_else(|| AgentError::Interaction("EXECUTION_JOB_UNAVAILABLE".into()))?;
    selected.state = JobState::Cancelled;
    let result = selected.clone(); save(&values).await?; Ok(result)
}
