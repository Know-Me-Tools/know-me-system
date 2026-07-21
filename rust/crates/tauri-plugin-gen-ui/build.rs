// TJ-ARCH-MOB-001 compliant
//! Tauri 2 plugin build script: generates permission schemas + the guest-side
//! command allowlist from COMMANDS. Keep COMMANDS in sync with the invoke_handler
//! in src/lib.rs.
const COMMANDS: &[&str] = &[
    "stream_agent_a2ui",
    "send_agent_interaction_a2ui",
    "promote_a2ui_component",
    "get_active_lane",
    "set_active_lane",
    "has_local_engine",
    "cancel_agent_run",
    "prepare_local_model",
    "local_model_status",
    "cancel_model_operation",
    "local_runtime_diagnostics",
    "run_migrations",
    "load_seeds",
    "attach_sync_shapes",
    "memory_ingest",
    "entity_runtime_start",
    "entity_runtime_stop",
    "entity_list",
    "entity_get",
    "entity_create",
    "entity_update",
    "entity_delete",
    "memory_search",
    "graph_expand",
    "scribe_start",
    "scribe_stop",
    "uar_service_config",
    "uar_service_save",
    "uar_service_probe",
    "uar_runtime_status",
    "provider_catalog",
    "provider_list",
    "provider_save",
    "provider_delete",
    "provider_test",
    "cloud_model_get",
    "cloud_model_save",
    "chat_control_catalog",
    "conversation_policy_get",
    "conversation_policy_save",
    "conversation_policy_delete",
    "conversation_effective_config",
    "uar_admin_snapshot",
    "uar_setting_save",
    "uar_agent_save",
    "uar_agent_delete",
    "uar_mcp_server_save",
    "uar_mcp_server_delete",
    "uar_memory_delete",
    "uar_knowledge_base_create",
    "uar_knowledge_base_delete",
    "memory_delete_conversation_context",
    "memory_delete_message_context",
    "memory_promote_context",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
