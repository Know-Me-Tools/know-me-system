// TJ-ARCH-MOB-001 compliant
//! Mobile boot-order intent surface, mirroring tauri-plugin-gen-ui's
//! run_migrations/load_seeds/attach_sync_shapes commands: migrations -> seeds
//! -> shapes (shapes fail on unknown columns, so this order is load-bearing).
//! Dart resolves the platform data directory (path_provider) and passes it in
//! — Rust has no portable way to ask the OS for it on mobile.
use std::sync::Arc;

use gen_ui_agent::ConfigBackend;
use gen_ui_db_graph::{GraphStore, GraphStoreConfig, LazyFastEmbedder};
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
    // Android does not define HOME for app processes. hf-hub constructs its
    // default cache before fastembed applies our explicit cache path, so give
    // that constructor a valid app-private home directory.
    if std::env::var_os("HOME").is_none() {
        std::env::set_var("HOME", &data_dir);
    }
    crate::api::entity::init_store(
        std::path::PathBuf::from(&data_dir).join("entity-records.sqlite3"),
    )
    .await?;
    let db_path = format!("{data_dir}/knowme-poc.db");
    let embed_cache = std::path::PathBuf::from(&data_dir).join("model-cache/fastembed");
    std::fs::create_dir_all(&embed_cache).map_err(|e| boot_fail("model cache create", e))?;
    let embedder = LazyFastEmbedder::new(embed_cache);

    let store = GraphStore::open(GraphStoreConfig {
        endpoint: format!("rocksdb://{db_path}"),
        namespace: "knowme".to_string(),
        database: "poc".to_string(),
        embedder: Arc::new(embedder),
    })
    .await
    .map_err(|e| boot_fail("graph store open", e))?;
    let store = Arc::new(store);

    // Resolve the default entirely in Rust so Flutter never contains device
    // or model-selection logic. Existing user-selected preferences are kept;
    // only superseded bundled Android Qwen defaults are migrated.
    let default_model = gen_ui_inference::catalog::current_mobile_platform()
        .map(|platform| platform.default_model().model_id)
        .unwrap_or(gen_ui_inference::catalog::ANDROID_GEMMA4_E2B_LITERTLM_ID)
        .to_string();
    let existing_pref = store
        .get_model_pref("chat", "local")
        .await
        .map_err(|e| boot_fail("local model preference read", e))?;
    let migrate_legacy = existing_pref.as_ref().is_none_or(|pref| {
        matches!(
            pref.model_id.as_str(),
            "qwen2.5-0.5b-instruct-q4" | "qwen3.5-4b-q4-k-m" | "qwen3-4b-instruct-2507-q5-k-m"
        )
    });
    if migrate_legacy {
        store
            .upsert_model_pref(&gen_ui_db_graph::ModelPref {
                surface: "chat".to_string(),
                lane: "local".to_string(),
                provider_id: None,
                model_id: default_model,
                params: serde_json::json!({
                    "context_len": 8192,
                    "max_tokens": 1024,
                    "temperature": 0.7,
                    "top_p": 0.8,
                    "top_k": 20,
                    "min_p": 0.0,
                    "presence_penalty": 0.0
                }),
            })
            .await
            .map_err(|e| boot_fail("local model preference write", e))?;
    }

    #[cfg(target_os = "android")]
    let inference: Arc<dyn gen_ui_types::inference::InferenceProvider> =
        Arc::new(gen_ui_inference::LiteRtLmEngine::new(
            std::path::PathBuf::from(&data_dir).join("model-cache/litert-lm"),
        ));
    #[cfg(not(target_os = "android"))]
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
