// TJ-ARCH-MOB-001 compliant
//! Mobile boot-order intent surface, mirroring tauri-plugin-gen-ui's
//! run_migrations/load_seeds/attach_sync_shapes commands: migrations -> seeds
//! -> shapes (shapes fail on unknown columns, so this order is load-bearing).
//! Dart resolves the platform data directory (path_provider) and passes it in
//! — Rust has no portable way to ask the OS for it on mobile.
use std::sync::Arc;

use gen_ui_agent::ConfigBackend;
use gen_ui_db_graph::{GraphStore, GraphStoreConfig, LazyFastEmbedder};
use gen_ui_mcp::{McpServerHandle, McpTransport};
// frb's Result<T,E> detection needs a literal `Result<...>` — see chat.rs's
// note; CoreResult<T>'s generic alias is invisible to frb's codegen.
use gen_ui_types::{CoreError, CoreResult};

struct MobileDiagnosticsTransport;

#[async_trait::async_trait]
impl McpTransport for MobileDiagnosticsTransport {
    async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> CoreResult<serde_json::Value> {
        match method {
            "tools/list" => Ok(serde_json::json!({
                "tools": [
                    {
                        "name": "local_runtime_status",
                        "title": "Local Runtime Status",
                        "description": "Report the current KnowMe on-device runtime, platform, default local model, and safe local-tool capability status. Use this when asked to inspect the local model, runtime, device backend, or available local tools.",
                        "inputSchema": {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {}
                        },
                        "annotations": {
                            "readOnlyHint": true,
                            "destructiveHint": false,
                            "idempotentHint": true,
                            "openWorldHint": false
                        },
                        "outputSchema": {
                            "type": "object",
                            "required": ["ok", "runtimeKind", "platform", "defaultModel", "safeLocalTools"],
                            "properties": {
                                "ok": {"type": "boolean"},
                                "runtimeKind": {"type": "string"},
                                "platform": {"type": "string"},
                                "defaultModel": {"type": "string"},
                                "safeLocalTools": {"type": "boolean"}
                            }
                        }
                    },
                    {
                        "name": "approval_required_echo",
                        "title": "Approval Required Echo",
                        "description": "Echo a short certification phrase only after explicit user approval. Use this only when the user asks to test or certify the local tool approval flow.",
                        "inputSchema": {
                            "type": "object",
                            "required": ["phrase"],
                            "additionalProperties": false,
                            "properties": {
                                "phrase": {
                                    "type": "string",
                                    "minLength": 1,
                                    "maxLength": 80
                                }
                            }
                        },
                        "annotations": {
                            "readOnlyHint": true,
                            "destructiveHint": false,
                            "idempotentHint": true,
                            "openWorldHint": true
                        },
                        "outputSchema": {
                            "type": "object",
                            "required": ["ok", "phrase", "approvalFlow"],
                            "properties": {
                                "ok": {"type": "boolean"},
                                "phrase": {"type": "string"},
                                "approvalFlow": {"type": "string"}
                            }
                        }
                    }
                ]
            })),
            "tools/call" => {
                let name = params
                    .as_ref()
                    .and_then(|value| value.get("name"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if name == "approval_required_echo" {
                    let phrase = params
                        .as_ref()
                        .and_then(|value| value.pointer("/arguments/phrase"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default();
                    return Ok(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Approved diagnostic echo: {phrase}")
                        }],
                        "structuredContent": {
                            "ok": true,
                            "phrase": phrase,
                            "approvalFlow": "approved_pending_tool_call"
                        },
                        "isError": false
                    }));
                }
                if name != "local_runtime_status" {
                    return Err(CoreError::Terminal(format!(
                        "unknown diagnostics tool '{name}'"
                    )));
                }
                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": "KnowMe local runtime is available with a safe read-only diagnostics tool."
                    }],
                    "structuredContent": {
                        "ok": true,
                        "runtimeKind": if cfg!(target_os = "android") {
                            "litert-lm"
                        } else {
                            "llama.cpp"
                        },
                        "platform": std::env::consts::OS,
                        "defaultModel": gen_ui_inference::catalog::current_mobile_platform()
                            .map(|platform| platform.default_model().model_id)
                            .unwrap_or(gen_ui_inference::catalog::ANDROID_GEMMA4_E2B_LITERTLM_ID),
                        "safeLocalTools": true
                    },
                    "isError": false
                }))
            }
            other => Err(CoreError::Terminal(format!(
                "unsupported diagnostics MCP method '{other}'"
            ))),
        }
    }
}

async fn register_mobile_diagnostics_mcp() -> Result<(), CoreError> {
    let registry = gen_ui_agent::state::mcp()?;
    let handle = registry.register(McpServerHandle::new_trusted_local(
        "knowme_diagnostics",
        Box::new(MobileDiagnosticsTransport),
    ));
    handle.refresh_tools().await?;
    tracing::info!(
        server = %handle.name,
        "mobile diagnostics MCP tools registered"
    );
    Ok(())
}

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
    // Mobile boots independently of gen_ui_host, so the attachment root must be
    // configured HERE too — otherwise every Android/iOS attachment fails with
    // "storage root is not configured" while desktop works fine.
    //
    // It must be the app data dir, never the cache dir: `file_picker` returns a
    // purgeable cache path (it ships clearTemporaryFiles() for exactly that),
    // so rendering from the picked path passes testing and breaks weeks later.
    gen_ui_agent::attachment_store::set_root(
        std::path::PathBuf::from(&data_dir).join("attachments"),
    );
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
        .map(|platform| platform.default_model_id())
        .unwrap_or(gen_ui_inference::catalog::ANDROID_GEMMA4_E2B_LITERTLM_ID)
        .to_string();
    #[cfg(target_os = "android")]
    {
        let provider = crate::local_provider::android_litert_lm_uar_provider_config();
        if provider.default_model.as_deref() != Some(default_model.as_str()) {
            return Err(boot_fail(
                "embedded UAR local provider registration",
                "Android UAR default model does not match the pinned inference artifact",
            ));
        }
        tracing::info!(
            provider_id = %provider.id,
            model_id = %default_model,
            context_tokens = provider.models[0].context_window,
            "Android LiteRT-LM provider metadata ready for embedded UAR"
        );
    }
    #[cfg(target_os = "ios")]
    {
        let provider = crate::local_provider::ios_qwen35_mlx_uar_provider_config();
        if provider.default_model.as_deref() != Some(default_model.as_str()) {
            return Err(boot_fail(
                "embedded UAR local provider registration",
                "iOS UAR default model does not match the pinned inference artifact",
            ));
        }
        // The iOS default is text-only today: MLXVLM's Swift registry has no
        // Qwen3.5 vision processor yet, so images route through the Xberg
        // extraction path (like the Android text-only lane). Vision returns as
        // a catalog swap when the processor lands upstream — no assert here.
        tracing::info!(
            provider_id = %provider.id,
            model_id = %default_model,
            context_tokens = provider.models[0].context_window,
            "iOS Qwen3.5 MLX provider metadata ready for embedded UAR"
        );
    }
    let existing_pref = store
        .get_model_pref("chat", "local")
        .await
        .map_err(|e| boot_fail("local model preference read", e))?;
    let migrate_legacy = existing_pref.as_ref().is_none_or(|pref| {
        matches!(
            pref.model_id.as_str(),
            "qwen2.5-0.5b-instruct-q4"
                | "qwen3.5-4b-q4-k-m"
                | "qwen3-4b-instruct-2507-q5-k-m"
                | "qwen3.5-9b-q4-k-m"
        )
    });
    if migrate_legacy {
        store
            .upsert_model_pref(&gen_ui_db_graph::ModelPref {
                surface: "chat".to_string(),
                lane: "local".to_string(),
                provider_id: None,
                model_id: default_model.clone(),
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
    // iOS runs MLX-Swift (Metal) via the registered Swift bridge — llama.cpp is
    // no longer built for Apple platforms.
    #[cfg(target_os = "ios")]
    let inference: Arc<dyn gen_ui_types::inference::InferenceProvider> =
        Arc::new(gen_ui_inference::MlxEngine::new(
            std::path::PathBuf::from(&data_dir).join("model-cache/mlx"),
        ));
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let inference: Arc<dyn gen_ui_types::inference::InferenceProvider> =
        Arc::new(gen_ui_inference::LlamaCppEngine::new(
            std::path::PathBuf::from(&data_dir).join("model-cache/llama"),
        ));

    #[cfg(target_os = "android")]
    let embedded_uar = Some(Arc::new(
        gen_ui_agent::uar::UarEmbeddedRuntime::build_local(
            inference.clone(),
            gen_ui_types::inference::LocalModelSpec {
                model: default_model.clone(),
                context_len: Some(8_192),
            },
            crate::local_provider::android_litert_lm_uar_provider_config(),
            // Mobile boots HERE and never touches gen_ui_host, so the storage
            // path has to be threaded through both. Wiring only the host would
            // leave Android and iOS discarding every agent on exit while
            // desktop looked fixed.
            Some(&std::path::PathBuf::from(&data_dir).join("uar-db")),
        )
        .await
        .map_err(|error| boot_fail("embedded UAR initialization", error))?,
    ));
    #[cfg(target_os = "ios")]
    let embedded_uar = Some(Arc::new(
        gen_ui_agent::uar::UarEmbeddedRuntime::build_local(
            inference.clone(),
            gen_ui_types::inference::LocalModelSpec {
                model: default_model.clone(),
                context_len: Some(8_192),
            },
            crate::local_provider::ios_qwen35_mlx_uar_provider_config(),
            // Mobile boots HERE and never touches gen_ui_host, so the storage
            // path has to be threaded through both. Wiring only the host would
            // leave Android and iOS discarding every agent on exit while
            // desktop looked fixed.
            Some(&std::path::PathBuf::from(&data_dir).join("uar-db")),
        )
        .await
        .map_err(|error| boot_fail("embedded UAR initialization", error))?,
    ));
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    let embedded_uar = None;

    gen_ui_agent::state::init_with_inference_and_uar(
        ConfigBackend::Surreal(store.clone()),
        store,
        Some(inference),
        embedded_uar,
    );
    register_mobile_diagnostics_mcp()
        .await
        .map_err(|e| boot_fail("mobile diagnostics MCP registration", e))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_diagnostics_exposes_safe_and_approval_required_tools() {
        gen_ui_runtime::init(Some(2));
        gen_ui_runtime::handle().block_on(async {
            let transport = MobileDiagnosticsTransport;
            let listed = transport
                .request("tools/list", None)
                .await
                .expect("tools/list succeeds");
            let tools = listed
                .get("tools")
                .and_then(serde_json::Value::as_array)
                .expect("tools array");

            let safe = tools
                .iter()
                .find(|tool| {
                    tool.get("name").and_then(serde_json::Value::as_str)
                        == Some("local_runtime_status")
                })
                .expect("safe diagnostics tool");
            assert_eq!(
                safe.pointer("/annotations/openWorldHint")
                    .and_then(serde_json::Value::as_bool),
                Some(false)
            );

            let approval = tools
                .iter()
                .find(|tool| {
                    tool.get("name").and_then(serde_json::Value::as_str)
                        == Some("approval_required_echo")
                })
                .expect("approval diagnostics tool");
            assert_eq!(
                approval
                    .pointer("/annotations/readOnlyHint")
                    .and_then(serde_json::Value::as_bool),
                Some(true)
            );
            assert_eq!(
                approval
                    .pointer("/annotations/destructiveHint")
                    .and_then(serde_json::Value::as_bool),
                Some(false)
            );
            assert_eq!(
                approval
                    .pointer("/annotations/openWorldHint")
                    .and_then(serde_json::Value::as_bool),
                Some(true),
                "open-world annotation intentionally forces manual confirmation"
            );

            let called = transport
                .request(
                    "tools/call",
                    Some(serde_json::json!({
                        "name": "approval_required_echo",
                        "arguments": {"phrase": "approval-ok"}
                    })),
                )
                .await
                .expect("approval echo call succeeds after host approval");
            assert_eq!(
                called.pointer("/structuredContent/approvalFlow"),
                Some(&serde_json::json!("approved_pending_tool_call"))
            );
            assert_eq!(
                called.pointer("/structuredContent/phrase"),
                Some(&serde_json::json!("approval-ok"))
            );
        });
    }
}
