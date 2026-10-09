//! Typed host controls for placement and existing-run observation.
use gen_ui_types::execution_profiles::{ExecutionProfile, ExecutionJob};
#[tauri::command] pub async fn profile() -> Result<Option<ExecutionProfile>, String> {
    gen_ui_agent::execution_profiles::current().await.map_err(|error| error.to_string())
}
#[tauri::command] pub async fn configure(profile: ExecutionProfile) -> Result<ExecutionProfile, String> {
    gen_ui_agent::execution_profiles::configure(profile).await.map_err(|error| error.to_string())
}
#[tauri::command] pub async fn jobs() -> Result<Vec<ExecutionJob>, String> {
    gen_ui_agent::execution_profiles::jobs().await.map_err(|error| error.to_string())
}
#[tauri::command] pub async fn suspend(host_run_id: String) -> Result<ExecutionJob, String> {
    gen_ui_agent::execution_profiles::suspend(&host_run_id).await.map_err(|error| error.to_string())
}
#[tauri::command] pub async fn resume(host_run_id: String) -> Result<ExecutionJob, String> {
    gen_ui_agent::execution_profiles::resume(&host_run_id).await.map_err(|error| error.to_string())
}
#[tauri::command] pub async fn cancel(host_run_id: String) -> Result<ExecutionJob, String> {
    gen_ui_agent::execution_profiles::cancel(&host_run_id).await.map_err(|error| error.to_string())
}
