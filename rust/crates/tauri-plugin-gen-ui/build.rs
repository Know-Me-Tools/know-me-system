// TJ-ARCH-MOB-001 compliant
//! Tauri 2 plugin build script: generates permission schemas + the guest-side
//! command allowlist from COMMANDS. Keep COMMANDS in sync with the invoke_handler
//! in src/lib.rs.
const COMMANDS: &[&str] = &[
    "stream_agent_a2ui",
    "get_active_lane",
    "set_active_lane",
    "has_local_engine",
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
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
