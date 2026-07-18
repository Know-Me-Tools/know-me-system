// TJ-ARCH-MOB-001 compliant
//! Tauri command handlers. Thin wrappers over the shared intent surface. Desktop
//! runs gen_ui_core in-process, so these call the same logic gen_ui_ffi exposes to
//! Flutter — no duplicated business logic (constraint: logic lives in Rust core).
//! run_migrations/load_seeds/attach_sync_shapes replace the ad-hoc stubs that used
//! to live directly in src-tauri/src/commands.rs (removed — this plugin is now the
//! single desktop command surface, matching the mobile FFI's single intent surface).
use std::sync::{Arc, Mutex};

use crate::error::Result;
use gen_ui_db_graph::{MemoryHit, RelatedEntity, SearchMode};
use gen_ui_types::transport::{EntityRecord, ListResult};
use gen_ui_types::view::ViewDescriptor;
use once_cell::sync::OnceCell;
use sqlx::Row;
use tauri::{Manager, Runtime};

fn app_data_dir<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("GEN_UI_APP_DATA_DIR") {
        return Ok(path.into());
    }
    Ok(app.path().app_data_dir()?)
}

// Named to match the frontend's existing invoke('stream_agent_a2ui', ...) call
// site (src/features/chat/stores/chatStore.ts) rather than renaming the
// frontend — mobile's frb bridge calls the equivalent Rust fn `chat_send`
// directly by name (no separate invoke-key layer), so the two ecosystems
// have different but internally consistent naming.
#[tauri::command]
pub async fn stream_agent_a2ui(user_message: String, messages: Vec<String>) -> Result<String> {
    tracing::info!(history_items = messages.len(), "desktop chat turn starting");
    let run_id = gen_ui_agent::chat::send(user_message, messages)
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(crate::Error::from)?;
    tracing::info!(run_id = %run_id, "desktop chat run started");
    Ok(run_id)
}

/// Current chat lane: "cloud" or "local".
#[tauri::command]
pub async fn get_active_lane() -> Result<String> {
    gen_ui_agent::chat::active_lane()
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

/// Switch chat lanes. Errors on an unknown lane, or on "local" where this build
/// has no engine — the toggle fails loudly here rather than silently answering
/// from the cloud at the next turn.
#[tauri::command]
pub async fn set_active_lane(lane: String) -> Result<()> {
    gen_ui_agent::chat::set_active_lane(&lane)
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

/// Whether a local-inference engine exists on this build — the UI shows the lane
/// toggle only when it does.
#[tauri::command]
pub async fn has_local_engine() -> Result<bool> {
    Ok(gen_ui_agent::chat::has_local_engine())
}

#[tauri::command]
pub fn provider_catalog() -> Result<Vec<gen_ui_agent::provider_admin::ProviderCatalogEntry>> {
    gen_ui_agent::provider_admin::catalog()
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn provider_list() -> Result<Vec<gen_ui_agent::provider_admin::ConfiguredProvider>> {
    gen_ui_agent::provider_admin::list()
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn provider_save(
    request: gen_ui_agent::provider_admin::SaveProviderRequest,
) -> Result<()> {
    gen_ui_agent::provider_admin::save(request)
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn provider_delete(id: String) -> Result<()> {
    gen_ui_agent::provider_admin::delete(&id)
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn cloud_model_get() -> Result<Option<gen_ui_agent::provider_admin::ConfiguredCloudModel>>
{
    gen_ui_agent::provider_admin::get_cloud_model()
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn cloud_model_save(
    request: gen_ui_agent::provider_admin::SaveCloudModelRequest,
) -> Result<()> {
    gen_ui_agent::provider_admin::save_cloud_model(request)
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

/// Boot-order invariant, step 1: open the config store (pglite-oxide) AND the
/// memory/graph-RAG store (embedded SurrealDB — a separate concern from config
/// storage, see gen_ui_agent::state::init's doc comment: memory is
/// SurrealDB-only on every platform, desktop included, regardless of which
/// config-DB engine is in use), then initialise gen_ui_agent's process-lifetime
/// state (config backend + memory store + A2uiEvent broadcast). Idempotent per
/// process — the frontend calls this once at startup, before
/// load_seeds/attach_sync_shapes.
///
/// Concurrent duplicate invocations (React StrictMode double-invokes the startup
/// effect in dev) are safe: `PgliteStore::open` coalesces concurrent callers
/// onto one in-flight initialization (see its doc comment). Cross-process
/// contention is prevented by `tauri-plugin-single-instance`, registered ahead
/// of this plugin. Any remaining open failure surfaces pglite-oxide's own error
/// text verbatim — no re-labelling, so the real cause is never masked.
#[tauri::command]
pub async fn run_migrations<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    let data_dir = app_data_dir(&app)?;
    tracing::info!(path = %data_dir.display(), "desktop migrations starting");
    std::fs::create_dir_all(&data_dir).map_err(|e| {
        tauri::Error::from(std::io::Error::new(e.kind(), format!("app data dir: {e}")))
    })?;
    // Desktop's zero-configuration local-inference lane. It shares the same
    // revision-pinned, checksum-gated Qwen GGUF engine as mobile; the first local
    // turn downloads into app data, and later turns reuse that durable cache.
    let inference: Arc<dyn gen_ui_types::inference::InferenceProvider> = Arc::new(
        gen_ui_inference::LlamaCppEngine::new(data_dir.join("model-cache")),
    );

    gen_ui_host::AppServices::bootstrap(
        gen_ui_host::HostConfig::knowme(data_dir.clone()),
        Some(inference),
    )
    .await
    .map_err(|error| gen_ui_types::CoreError::Transient(error.to_string()))?;
    tracing::info!(path = %data_dir.display(), "desktop migrations ready");
    Ok(())
}

/// Boot-order invariant, step 2: seed data.
///
/// Ingests the C-111 demo corpus so a fresh install's memory search returns real
/// results instead of nothing. Idempotent (stable seed ids), so running on every start
/// upserts rather than duplicates. MUST follow `run_migrations` — there is no store to
/// seed into before it.
///
/// Seeding embeds every note, so this is not instant on a cold start; it is a boot step
/// rather than a lazy path precisely so the cost lands once, before the user searches.
#[tauri::command]
pub async fn load_seeds() -> Result<()> {
    tracing::info!("seed load starting");
    let n = gen_ui_agent::memory::seed_demo_corpus()
        .await
        .map_err(gen_ui_types::CoreError::from)?;
    log::info!("seeded {n} demo memories");
    tracing::info!(count = n, "seed load ready");
    Ok(())
}

/// Boot-order invariant, step 3: attach sync (C-106).
///
/// Starts the FRF-backed [`FrfSyncTransport`]: the read lane materialises server row
/// changes into the local pglite store, and the write queue replays local mutations
/// through forge/Quarry. MUST run after `run_migrations` — a change cannot be applied to
/// a table that does not exist yet, which is the whole reason boot order is an invariant
/// here and not a suggestion.
///
/// **Sync is opt-in and absent-by-default.** With no `sync.frf` settings in the config
/// DB this returns `Ok(())` and the app runs purely local. That is the correct default
/// for a PoC whose fabric is a private cross-org service most developers cannot reach —
/// failing startup because an optional backend is unconfigured would make the app
/// unrunnable for everyone who is not doing sync work today.
#[tauri::command]
pub async fn attach_sync_shapes<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    use gen_ui_db::sync::{FrfSyncTransport, PgLocalStore, SyncTransport};

    let data_dir = app_data_dir(&app)?;
    let Some(cfg) = read_sync_config(&data_dir).await? else {
        log::info!("sync: not configured (no `sync.frf` setting) — running local-only");
        tracing::info!("sync ready in local-only mode");
        return Ok(());
    };

    // The pool the read lane writes into is the SAME pglite store `run_migrations`
    // opened — `PgliteStore::open` is a per-process singleton, so this hands back the
    // live server rather than racing a second one against the same directory.
    let store = gen_ui_db::relational::PgliteStore::open(data_dir.join("config-db"))
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    let local = Arc::new(PgLocalStore::new(store.store().pool().clone()));

    let sink = gen_ui_agent::sync_sink::forge_write_sink(
        cfg.forge_base.clone(),
        "public",
        cfg.token.clone(),
    )?;

    let transport = FrfSyncTransport::new(cfg.into_transport_config(), local, sink);
    transport.start().await?;

    // Hold the transport for the process lifetime: dropping it stops the drain loop and
    // the read lane (they only live as long as the Arcs they captured).
    SYNC.set(Arc::new(transport))
        .map_err(|_| gen_ui_types::CoreError::Terminal("sync already attached".into()))?;
    Ok(())
}

/// The live sync transport. `OnceCell` mirrors `gen_ui_agent::state`'s process-lifetime
/// pattern and makes a second `attach_sync_shapes` a loud error rather than a silent
/// second engine competing for the same channel.
static SYNC: OnceCell<Arc<gen_ui_db::sync::FrfSyncTransport>> = OnceCell::new();

/// Sync settings as stored in the config DB under `sync.frf`.
#[derive(serde::Deserialize)]
struct SyncSettings {
    endpoint: String,
    tenant_id: String,
    channel_path: String,
    consumer_id: String,
    forge_base: String,
    #[serde(default)]
    token: Option<String>,
}

impl SyncSettings {
    fn into_transport_config(self) -> gen_ui_db::sync::FrfSyncConfig {
        gen_ui_db::sync::FrfSyncConfig {
            endpoint: self.endpoint,
            token: self.token,
            tenant_id: self.tenant_id,
            channel_path: self.channel_path,
            consumer_id: self.consumer_id,
            write_batch: 32,
            max_write_attempts: 5,
        }
    }
}

/// Read `sync.frf` from the config DB. `Ok(None)` = not configured (the default).
///
/// A malformed setting IS an error: someone tried to configure sync and got it wrong,
/// and silently falling back to local-only would hide that from them.
async fn read_sync_config(data_dir: &std::path::Path) -> Result<Option<SyncSettings>> {
    use gen_ui_db::relational::ConfigStore;

    let store = gen_ui_db::relational::PgliteStore::open(data_dir.join("config-db"))
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    let Some(raw) = store
        .store()
        .get_setting("sync.frf")
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?
    else {
        return Ok(None);
    };
    let parsed = serde_json::from_value(raw)
        .map_err(|e| gen_ui_types::CoreError::Serde(format!("sync.frf setting: {e}")))?;
    Ok(Some(parsed))
}

/// Ingest through the shared agent boundary, matching mobile's FFI command.
#[tauri::command]
pub async fn memory_ingest(text: String) -> Result<String> {
    gen_ui_agent::memory::ingest(text)
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

/// C-104/C-003 wire the real entity runtime; stub keeps the app booting.
#[tauri::command]
pub async fn entity_runtime_start(tenant_id: String) -> Result<()> {
    let _ = tenant_id;
    Ok(())
}

#[tauri::command]
pub async fn entity_runtime_stop() -> Result<()> {
    Ok(())
}

#[tauri::command]
pub async fn entity_list<R: Runtime>(
    app: tauri::AppHandle<R>,
    view: ViewDescriptor,
) -> Result<ListResult> {
    let store = gen_ui_db::relational::PgliteStore::open(app_data_dir(&app)?.join("config-db"))
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    let limit = i64::from(view.limit.unwrap_or(200).min(1_000));
    let rows = sqlx::query(
        "SELECT id, entity_type, data_json::text AS data_json FROM entity_records \
         WHERE entity_type = $1 ORDER BY updated_at DESC LIMIT $2",
    )
    .bind(&view.entity_type)
    .bind(limit)
    .fetch_all(store.store().pool())
    .await
    .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    Ok(ListResult {
        items: rows
            .into_iter()
            .map(|row| EntityRecord {
                id: row.get("id"),
                entity_type: row.get("entity_type"),
                data_json: row.get("data_json"),
            })
            .collect(),
        next_cursor: None,
    })
}

#[tauri::command]
pub async fn entity_get<R: Runtime>(
    app: tauri::AppHandle<R>,
    entity_type: String,
    id: String,
) -> Result<Option<EntityRecord>> {
    let store = gen_ui_db::relational::PgliteStore::open(app_data_dir(&app)?.join("config-db"))
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    let row = sqlx::query(
        "SELECT id, entity_type, data_json::text AS data_json FROM entity_records \
         WHERE entity_type = $1 AND id = $2",
    )
    .bind(&entity_type)
    .bind(&id)
    .fetch_optional(store.store().pool())
    .await
    .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    Ok(row.map(|row| EntityRecord {
        id: row.get("id"),
        entity_type: row.get("entity_type"),
        data_json: row.get("data_json"),
    }))
}

#[tauri::command]
pub async fn entity_create<R: Runtime>(
    app: tauri::AppHandle<R>,
    record: EntityRecord,
) -> Result<EntityRecord> {
    let store = gen_ui_db::relational::PgliteStore::open(app_data_dir(&app)?.join("config-db"))
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    sqlx::query(
        "INSERT INTO entity_records (entity_type, id, data_json, updated_at) \
         VALUES ($1, $2, $3::jsonb, CURRENT_TIMESTAMP)",
    )
    .bind(&record.entity_type)
    .bind(&record.id)
    .bind(&record.data_json)
    .execute(store.store().pool())
    .await
    .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    Ok(record)
}

#[tauri::command]
pub async fn entity_update<R: Runtime>(
    app: tauri::AppHandle<R>,
    record: EntityRecord,
) -> Result<EntityRecord> {
    let store = gen_ui_db::relational::PgliteStore::open(app_data_dir(&app)?.join("config-db"))
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    sqlx::query(
        "INSERT INTO entity_records (entity_type, id, data_json, updated_at) \
         VALUES ($1, $2, $3::jsonb, CURRENT_TIMESTAMP) \
         ON CONFLICT (entity_type, id) DO UPDATE SET data_json = EXCLUDED.data_json, \
         updated_at = CURRENT_TIMESTAMP",
    )
    .bind(&record.entity_type)
    .bind(&record.id)
    .bind(&record.data_json)
    .execute(store.store().pool())
    .await
    .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    Ok(record)
}

#[tauri::command]
pub async fn entity_delete<R: Runtime>(
    app: tauri::AppHandle<R>,
    entity_type: String,
    id: String,
) -> Result<()> {
    let store = gen_ui_db::relational::PgliteStore::open(app_data_dir(&app)?.join("config-db"))
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    sqlx::query("DELETE FROM entity_records WHERE entity_type = $1 AND id = $2")
        .bind(entity_type)
        .bind(id)
        .execute(store.store().pool())
        .await
        .map_err(|e| gen_ui_types::CoreError::Transient(e.to_string()))?;
    Ok(())
}

/// Hybrid graph-RAG search. Delegates to the SAME `gen_ui_agent::memory` the mobile
/// FFI calls — no duplicated business logic.
///
/// Returns the real `MemoryHit` (id/text/kind/score), not the `Vec<String>` this
/// used to stub out: C-104 wired the mobile FFI to the agent but left this command
/// returning an empty vec, so desktop memory search silently produced nothing.
/// Found while surfacing the Memory screen in C-113.
/// `mode` is optional: absent → hybrid, the product path. `"vector"` runs the
/// diagnostic lane (no BM25, no RRF) so the UI can show what fusion buys on the same
/// query. Scores compare only WITHIN a mode — never merge results from both.
#[tauri::command]
pub async fn memory_search(
    query: String,
    k: u32,
    mode: Option<SearchMode>,
) -> Result<Vec<MemoryHit>> {
    let hits = gen_ui_agent::memory::search_with(query, k, mode.unwrap_or_default())
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(crate::Error::from)?;
    tracing::info!(count = hits.len(), "desktop memory search ready");
    Ok(hits)
}

/// Graph expansion from an entity. Same delegation, same C-104 gap as above.
#[tauri::command]
pub async fn graph_expand(entity_id: String, depth: u32) -> Result<Vec<RelatedEntity>> {
    gen_ui_agent::memory::graph_expand(entity_id, depth)
        .await
        .map_err(gen_ui_types::CoreError::from)
        .map_err(Into::into)
}

// Scribe (voice-to-memory): delegates entirely to gen_ui_audio, the SAME crate
// gen_ui_ffi's mobile scribe_start/scribe_stop call — no duplicated business
// logic. One recording in flight per process, mirroring gen_ui_agent's
// single-chat-turn precedent.
static RECORDING: OnceCell<Mutex<Option<gen_ui_audio::Recorder>>> = OnceCell::new();

fn recording_slot() -> &'static Mutex<Option<gen_ui_audio::Recorder>> {
    RECORDING.get_or_init(|| Mutex::new(None))
}

#[tauri::command]
pub async fn scribe_start() -> Result<()> {
    let mut guard = recording_slot()
        .lock()
        .expect("scribe recording mutex poisoned");
    if guard.is_some() {
        return Err(gen_ui_types::CoreError::Terminal(
            "a recording is already in progress".to_string(),
        )
        .into());
    }
    let recorder = gen_ui_audio::Scribe::new()
        .start_recording()
        .map_err(Into::<gen_ui_types::CoreError>::into)?;
    *guard = Some(recorder);
    Ok(())
}

#[tauri::command]
pub async fn scribe_stop() -> Result<String> {
    let recorder = recording_slot()
        .lock()
        .expect("scribe recording mutex poisoned")
        .take()
        .ok_or_else(|| gen_ui_types::CoreError::Terminal("no recording in progress".to_string()))?;
    gen_ui_audio::Scribe::new()
        .stop_and_transcribe(recorder)
        .await
        .map_err(Into::<gen_ui_types::CoreError>::into)
        .map_err(Into::into)
}
