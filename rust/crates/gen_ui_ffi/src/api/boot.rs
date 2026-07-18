// TJ-ARCH-MOB-001 compliant
//! Mobile boot-order intent surface, mirroring tauri-plugin-gen-ui's
//! run_migrations/load_seeds/attach_sync_shapes commands: migrations -> seeds
//! -> shapes (shapes fail on unknown columns, so this order is load-bearing).
//! Dart resolves the platform data directory (path_provider) and passes it in
//! — Rust has no portable way to ask the OS for it on mobile.
use std::sync::Arc;

use gen_ui_agent::ConfigBackend;
use gen_ui_db_graph::{FastEmbedder, GraphStore, GraphStoreConfig};
// frb's Result<T,E> detection needs a literal `Result<...>` — see chat.rs's
// note; CoreResult<T>'s generic alias is invisible to frb's codegen.
use gen_ui_types::CoreError;

/// Boot-order invariant, step 1: open the embedded SurrealDB store (config +
/// memory/graph-RAG both live in it on mobile — see gen_ui_agent::state::init's
/// doc comment on why memory is SurrealDB-only regardless of platform) and
/// initialise gen_ui_agent's process-lifetime state. Idempotent per process.
pub async fn run_migrations(data_dir: String) -> Result<(), CoreError> {
    fn boot_fail(stage: &str, message: impl std::fmt::Display) -> CoreError {
        let rendered = format!("{stage}: {message}");
        eprintln!("[knowme boot] {rendered}");
        tracing::error!(stage, error = %message, "mobile boot phase failed");
        CoreError::Transient(rendered)
    }

    tracing::info!(%data_dir, "mobile migrations starting");
    crate::api::entity::init_store(
        std::path::PathBuf::from(&data_dir).join("entity-records.sqlite3"),
    )
    .await?;
    let db_path = format!("{data_dir}/knowme-poc.db");
    let embed_cache = std::path::PathBuf::from(&data_dir).join("model-cache/fastembed");
    std::fs::create_dir_all(&embed_cache).map_err(|e| boot_fail("model cache create", e))?;
    let embedder = gen_ui_runtime::spawn_blocking(move || FastEmbedder::new(embed_cache))
        .await
        .map_err(|e| boot_fail("embedder task join", e))?
        .map_err(|e| boot_fail("embedder load", e))?;

    let store = GraphStore::open(GraphStoreConfig {
        endpoint: format!("rocksdb://{db_path}"),
        namespace: "knowme".to_string(),
        database: "poc".to_string(),
        embedder: Arc::new(embedder),
    })
    .await
    .map_err(|e| boot_fail("graph store open", e))?;
    let store = Arc::new(store);

    // The mobile engine ships the smaller Qwen catalog entry. Persist an
    // explicit preference so the agent never guesses the desktop model id.
    if store
        .get_model_pref("chat", "local")
        .await
        .map_err(|e| boot_fail("local model preference read", e))?
        .is_none()
    {
        store
            .upsert_model_pref(&gen_ui_db_graph::ModelPref {
                surface: "chat".to_string(),
                lane: "local".to_string(),
                provider_id: None,
                model_id: "qwen2.5-0.5b-instruct-q4".to_string(),
                params: serde_json::json!({
                    "context_len": 2048,
                    "max_tokens": 128,
                    "temperature": 0.7,
                    "top_p": 0.95
                }),
            })
            .await
            .map_err(|e| boot_fail("local model preference write", e))?;
    }

    let inference: Arc<dyn gen_ui_types::inference::InferenceProvider> =
        Arc::new(gen_ui_inference::LlamaCppEngine::new(
            std::path::PathBuf::from(&data_dir).join("model-cache/llama"),
        ));
    gen_ui_agent::state::init_with_inference(
        ConfigBackend::Surreal(store.clone()),
        store,
        Some(inference),
    );
    tracing::info!("mobile migrations ready");
    Ok(())
}

/// Boot-order invariant, step 2: seed data.
///
/// Ingests the C-111 demo corpus through the SAME `gen_ui_agent::memory` the desktop
/// plugin's `load_seeds` calls — no duplicated seeding logic, and both surfaces get an
/// identical corpus. Idempotent (stable seed ids), so running on every start upserts
/// rather than duplicates. MUST follow `run_migrations`.
///
/// Returns the number of notes seeded, so Dart can report progress rather than stare at
/// a silent pause: every note is embedded here, which is not instant on a cold start.
pub async fn load_seeds() -> Result<u32, CoreError> {
    let n = gen_ui_agent::memory::seed_demo_corpus().await?;
    Ok(n as u32)
}

/// Boot-order invariant, step 3: attach sync (C-106).
///
/// **Still a no-op on mobile, and deliberately so — see below.** Desktop's equivalent
/// (`tauri-plugin-gen-ui::commands::attach_sync_shapes`) is live: it starts an
/// `FrfSyncTransport` whose read lane materialises row changes into `PgLocalStore`.
///
/// Mobile cannot reuse that, because mobile has no Postgres. `run_migrations` above
/// opens embedded SurrealDB and registers it as BOTH config and memory backend
/// (`ConfigBackend::Surreal`) — pglite-oxide is structurally unsupported on iOS/Android
/// (no child processes, no JIT), which is why the split exists at all. So the read lane
/// needs a `LocalStore` over SurrealDB, and `gen_ui_db_graph`'s public surface is
/// intent-level by design ("INTENT-LEVEL, never raw SurrealQL" — lib.rs): it exposes
/// `memory_ingest`/`memory_search`/`graph_expand`, not row upserts. Writing a
/// `SurrealLocalStore` means adding a row-persistence surface to that crate, which is a
/// real design decision about its contract — not something to smuggle in under a sync
/// task. Tracked as C-106 T5; the honest state is "desktop syncs, mobile does not yet".
///
/// The write path is NOT blocked by this: `gen_ui_agent::sync_sink::forge_write_sink`
/// is platform-agnostic, so once a mobile `LocalStore` exists the wiring here mirrors
/// desktop's exactly.
pub async fn attach_sync_shapes() -> Result<(), CoreError> {
    Ok(())
}
