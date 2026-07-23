// TJ-ARCH-MOB-001 compliant
//! On-device (embedded UAR) provider administration.
//!
//! Serves the same eight provider-admin operations the service backend does, but
//! from KnowMe's on-device config store + platform secrets rather than an HTTP
//! UAR. This is the branch the mobile "Model providers" Settings tab uses, where
//! no `uar_service_url` exists. Return types are byte-identical to the service
//! branch so the Flutter/FFI surface is unchanged.
//!
//! ## `provider_meta` scheme
//!
//! The config store's `Provider` row is intentionally minimal (`id`, `kind`,
//! `base_url`, `api_key_ref`, `enabled`) — it has no room for display name,
//! protocol, default model, or the rich per-model capability list the UI shows.
//! Those are persisted as a JSON blob under the setting key
//! `provider_meta:{id}` (see [`provider_meta_key`]) whose shape is
//! [`ProviderMeta`]. `save` writes it; `list`/`catalog` enrich from it; `delete`
//! clears it. Losing the blob degrades gracefully (the provider still lists with
//! catalog/id-derived defaults), so it is never load-bearing for routing.

use serde::{Deserialize, Serialize};
use universal_agent_runtime::config::{build_client_config, LlmConfig};
use universal_agent_runtime::llm::catalog::{ModelCatalog, ModelInfo, ProviderInfo};
use universal_agent_runtime::llm::{
    registry::ModelConfig as UarModelConfig, registry::ProtocolSetting, LiterLlmDriver, LlmDriver,
    LlmRequest, ProviderConfig as UarProviderConfig,
};
use universal_agent_runtime::normalized::NormalizedEvent as UarNormalizedEvent;

use crate::config::{ConfigBackend, ModelPref, Provider};
use crate::error::AgentError;
use crate::uar::{LOCAL_PROVIDER_ID, SETTING_UAR_ACTIVE_PROVIDER};

use super::{
    ConfiguredCloudModel, ConfiguredProvider, ProviderCatalogEntry, ProviderModelEntry,
    ProviderTestResult, SaveCloudModelRequest, SaveProviderRequest,
};

/// Config-store surface/lane the cloud model preference is stored under. Kept in
/// sync with [`crate::uar`]'s `ConfigActiveSelector`, which reads the same pair.
const SURFACE_CHAT: &str = "chat";
const LANE_CLOUD: &str = "cloud";

/// Secret reference prefix for cloud-provider API keys stored on-device.
const CLOUD_SECRET_PREFIX: &str = "cloud:";

/// Rich provider metadata that the minimal config `Provider` row cannot hold,
/// persisted as JSON under `provider_meta:{id}`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProviderMeta {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    protocol: Option<serde_json::Value>,
    #[serde(default)]
    default_model: Option<String>,
    #[serde(default)]
    models: Vec<ProviderModelEntry>,
}

fn provider_meta_key(id: &str) -> String {
    format!("provider_meta:{id}")
}

fn cloud_secret_ref(id: &str) -> String {
    format!("{CLOUD_SECRET_PREFIX}{id}")
}

// ============================================================================
// Public entry points — reach the process-global config store, then delegate to
// the `*_impl` layer that takes an explicit `&ConfigBackend` so the core logic
// is unit-testable without touching the global.
// ============================================================================

pub(super) async fn catalog() -> Result<Vec<ProviderCatalogEntry>, AgentError> {
    catalog_impl(crate::state::config()?).await
}

pub(super) async fn list() -> Result<Vec<ConfiguredProvider>, AgentError> {
    list_impl(crate::state::config()?).await
}

pub(super) async fn save(request: SaveProviderRequest) -> Result<(), AgentError> {
    let id = request.id.trim().to_owned();
    let config = crate::state::config()?;
    save_impl(config, request).await?;
    // Best-effort runtime sync: register the provider into the embedded runtime's
    // registry and drop any cached cloud driver so the next run is rebuilt from
    // the updated credentials. A missing runtime (e.g. desktop) is not an error.
    if let Ok(runtime) = crate::state::embedded_uar() {
        if let Ok(Some(provider_config)) = saved_provider_config(config, &id).await {
            let _ = runtime.register_cloud_provider(provider_config).await;
        }
        runtime.invalidate_routing_cache().await;
    }
    Ok(())
}

pub(super) async fn delete(id: &str) -> Result<(), AgentError> {
    delete_impl(crate::state::config()?, id).await?;
    if let Ok(runtime) = crate::state::embedded_uar() {
        runtime.invalidate_routing_cache().await;
    }
    Ok(())
}

pub(super) async fn test(id: &str, model: Option<&str>) -> Result<ProviderTestResult, AgentError> {
    test_impl(crate::state::config()?, id, model).await
}

pub(super) async fn get_cloud_model() -> Result<Option<ConfiguredCloudModel>, AgentError> {
    get_cloud_model_impl(crate::state::config()?).await
}

pub(super) async fn save_cloud_model(request: SaveCloudModelRequest) -> Result<(), AgentError> {
    save_cloud_model_impl(crate::state::config()?, request).await?;
    if let Ok(runtime) = crate::state::embedded_uar() {
        runtime.invalidate_routing_cache().await;
    }
    Ok(())
}

// ============================================================================
// Testable core (explicit config backend, no global state, no keyring writes in
// the read paths).
// ============================================================================

async fn catalog_impl(config: &ConfigBackend) -> Result<Vec<ProviderCatalogEntry>, AgentError> {
    let configured = config.list_providers().await?;
    let configured_ids: std::collections::HashMap<&str, &Provider> = configured
        .iter()
        .map(|provider| (provider.id.as_str(), provider))
        .collect();

    let mut entries: Vec<ProviderCatalogEntry> = ModelCatalog::global()
        .all_providers()
        .iter()
        .map(|provider| {
            let configured_provider = configured_ids.get(provider.id.as_str());
            catalog_entry_from_provider_info(
                provider,
                configured_provider.is_some(),
                configured_provider.is_some_and(|provider| provider.enabled),
            )
        })
        .collect();

    // Prepend the synthetic on-device provider so the picker always offers the
    // offline lane, first.
    entries.insert(0, local_catalog_entry());

    entries.sort_by(|left, right| {
        // Local first, then configured providers, then alphabetical.
        let local_rank = |entry: &ProviderCatalogEntry| (entry.id != LOCAL_PROVIDER_ID) as u8;
        local_rank(left)
            .cmp(&local_rank(right))
            .then_with(|| right.configured.cmp(&left.configured))
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    Ok(entries)
}

async fn list_impl(config: &ConfigBackend) -> Result<Vec<ConfiguredProvider>, AgentError> {
    let providers = config.list_providers().await?;
    let mut result = Vec::with_capacity(providers.len() + 1);
    // Synthetic local provider is always present (credential-free).
    result.push(local_configured_provider());
    for provider in providers {
        let meta = read_provider_meta(config, &provider.id).await?;
        result.push(configured_provider_from_row(provider, meta));
    }
    result.sort_by(|left, right| {
        let local_rank = |entry: &ConfiguredProvider| (entry.id != LOCAL_PROVIDER_ID) as u8;
        local_rank(left)
            .cmp(&local_rank(right))
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    Ok(result)
}

async fn save_impl(config: &ConfigBackend, request: SaveProviderRequest) -> Result<(), AgentError> {
    let id = request.id.trim().to_owned();
    if id == LOCAL_PROVIDER_ID {
        return Err(AgentError::Config(
            "the on-device provider cannot be reconfigured".into(),
        ));
    }
    let existing = config.get_provider(&id).await?;
    let existing_meta = read_provider_meta(config, &id).await?;

    // Store the API key only when a fresh one is supplied; otherwise preserve the
    // existing reference (the UI never re-sends an unchanged key).
    let api_key_ref = match request
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
    {
        Some(key) => {
            let secret_ref = cloud_secret_ref(&id);
            crate::secrets::store_api_key(&secret_ref, key)?;
            Some(secret_ref)
        }
        None => existing
            .as_ref()
            .and_then(|provider| provider.api_key_ref.clone()),
    };

    config
        .upsert_provider(&Provider {
            id: id.clone(),
            kind: request.kind.trim().to_owned(),
            base_url: request
                .base_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .or_else(|| existing.and_then(|provider| provider.base_url)),
            api_key_ref,
            enabled: request.enabled,
        })
        .await?;

    // Persist the rich metadata the config row cannot hold.
    let display_name = request
        .display_name
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            existing_meta
                .as_ref()
                .and_then(|meta| meta.display_name.clone())
        });
    let models = if request.models.is_empty() {
        existing_meta
            .as_ref()
            .map(|meta| meta.models.clone())
            .unwrap_or_default()
    } else {
        request.models.clone()
    };
    let default_model = request
        .default_model
        .or_else(|| {
            existing_meta
                .as_ref()
                .and_then(|meta| meta.default_model.clone())
        })
        .or_else(|| {
            models
                .iter()
                .find(|model| model.enabled)
                .map(|model| model.id.clone())
        });
    let meta = ProviderMeta {
        display_name,
        protocol: request
            .protocol
            .or_else(|| existing_meta.and_then(|meta| meta.protocol)),
        default_model,
        models,
    };
    write_provider_meta(config, &id, &meta).await
}

async fn delete_impl(config: &ConfigBackend, id: &str) -> Result<(), AgentError> {
    config.delete_provider(id).await?;
    // Drop the secret (a missing entry is already the requested state).
    let _ = crate::secrets::delete_api_key(&cloud_secret_ref(id));
    // Remove the metadata blob.
    config
        .set_setting(&provider_meta_key(id), serde_json::Value::Null)
        .await?;
    // If the deleted provider was the active route, fall back to on-device.
    let active = config
        .get_setting(SETTING_UAR_ACTIVE_PROVIDER)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned));
    if active.as_deref() == Some(id) {
        config
            .set_setting(
                SETTING_UAR_ACTIVE_PROVIDER,
                serde_json::Value::String(LOCAL_PROVIDER_ID.to_owned()),
            )
            .await?;
    }
    Ok(())
}

async fn test_impl(
    config: &ConfigBackend,
    id: &str,
    model: Option<&str>,
) -> Result<ProviderTestResult, AgentError> {
    let model_id = model.unwrap_or_default().to_owned();
    if id == LOCAL_PROVIDER_ID {
        // The on-device lane is always available; a network round-trip would be
        // meaningless, so report success without one.
        return Ok(ProviderTestResult {
            ok: true,
            provider_id: id.to_owned(),
            model_id,
            latency_ms: 0,
            received_text: true,
        });
    }

    let provider = match config.get_provider(id).await? {
        Some(provider) => provider,
        None => return Ok(test_failure(id, &model_id)),
    };
    let api_key = match provider
        .api_key_ref
        .as_deref()
        .map(crate::secrets::resolve_api_key)
        .transpose()
    {
        Ok(Some(key)) => key,
        _ => return Ok(test_failure(id, &model_id)),
    };
    if model_id.is_empty() {
        return Ok(test_failure(id, &model_id));
    }

    let started = std::time::Instant::now();
    match probe_cloud(id, &model_id, provider.base_url, api_key).await {
        Ok(received_text) => Ok(ProviderTestResult {
            ok: true,
            provider_id: id.to_owned(),
            model_id,
            latency_ms: started.elapsed().as_millis(),
            received_text,
        }),
        Err(_) => Ok(ProviderTestResult {
            latency_ms: started.elapsed().as_millis(),
            ..test_failure(id, &model_id)
        }),
    }
}

async fn get_cloud_model_impl(
    config: &ConfigBackend,
) -> Result<Option<ConfiguredCloudModel>, AgentError> {
    let active = config
        .get_setting(SETTING_UAR_ACTIVE_PROVIDER)
        .await?
        .and_then(|value| value.as_str().map(str::to_owned));
    let provider_id = match active {
        Some(id) if id != LOCAL_PROVIDER_ID => id,
        // Local or unset ⇒ no cloud model selected.
        _ => return Ok(None),
    };
    let model_id = config
        .get_model_pref(SURFACE_CHAT, LANE_CLOUD)
        .await?
        .map(|pref| pref.model_id)
        .filter(|model_id| !model_id.is_empty())
        .unwrap_or_default();
    Ok(Some(ConfiguredCloudModel {
        provider_id,
        model_id,
    }))
}

async fn save_cloud_model_impl(
    config: &ConfigBackend,
    request: SaveCloudModelRequest,
) -> Result<(), AgentError> {
    let provider_id = request.provider_id.trim().to_owned();
    // Selecting a provider (local or cloud) sets the active route.
    config
        .set_setting(
            SETTING_UAR_ACTIVE_PROVIDER,
            serde_json::Value::String(provider_id.clone()),
        )
        .await?;
    // For a cloud provider, also persist the chosen model under the chat/cloud
    // pref so `ConfigActiveSelector` resolves it. Selecting the on-device lane
    // needs no model write (it carries no cloud model).
    if provider_id != LOCAL_PROVIDER_ID {
        config
            .upsert_model_pref(
                SURFACE_CHAT,
                LANE_CLOUD,
                &ModelPref {
                    provider_id: Some(provider_id),
                    model_id: request.model_id.trim().to_owned(),
                    params: serde_json::json!({}),
                },
            )
            .await?;
    }
    Ok(())
}

// ============================================================================
// Pure mapping helpers (unit-tested directly).
// ============================================================================

/// Map a UAR catalog [`ModelInfo`] to KnowMe's UI-facing [`ProviderModelEntry`].
fn model_entry_from_catalog(model: &ModelInfo) -> ProviderModelEntry {
    ProviderModelEntry {
        id: model.id.clone(),
        display_name: (!model.name.is_empty()).then(|| model.name.clone()),
        context_window: (model.limits.context_window > 0).then_some(model.limits.context_window),
        max_output_tokens: (model.limits.max_output > 0).then_some(model.limits.max_output),
        supports_vision: model
            .modalities
            .input
            .iter()
            .any(|modality| modality == "image"),
        supports_tools: model.capabilities.tool_call,
        supports_reasoning: model.capabilities.reasoning,
        supports_structured_output: model.capabilities.structured_output,
        supports_streaming: model.capabilities.streaming,
        // Catalog models are all selectable; the config store's own row governs
        // whether a *provider* is enabled.
        enabled: true,
    }
}

/// Map a UAR catalog [`ProviderInfo`] to a KnowMe [`ProviderCatalogEntry`],
/// stamping the on-device configured/enabled state from the config store.
fn catalog_entry_from_provider_info(
    provider: &ProviderInfo,
    configured: bool,
    enabled: bool,
) -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: provider.id.clone(),
        display_name: if provider.display_name.is_empty() {
            provider.id.clone()
        } else {
            provider.display_name.clone()
        },
        default_base_url: provider.base_url.clone(),
        requires_api_key: true,
        configured,
        enabled,
        models: provider
            .models
            .iter()
            .map(model_entry_from_catalog)
            .collect(),
    }
}

/// The one on-device model advertised in the catalog/list, derived from the
/// platform local-provider config so the model id stays in lockstep with the
/// pinned artifact. On host builds either platform config compiles; we pick a
/// representative one.
fn local_model_entry() -> ProviderModelEntry {
    let config = local_provider_config();
    let model = config
        .models
        .first()
        .expect("local provider config always has one model");
    ProviderModelEntry {
        id: model.id.clone(),
        display_name: model.display_name.clone(),
        context_window: model.context_window.map(u64::from),
        max_output_tokens: model.max_output_tokens.map(u64::from),
        supports_vision: model.supports_vision,
        supports_tools: model.supports_tools,
        supports_reasoning: model.supports_reasoning,
        supports_structured_output: model.supports_structured_output,
        supports_streaming: model.supports_streaming,
        enabled: model.enabled,
    }
}

/// Synthetic catalog entry for the always-present on-device provider.
fn local_catalog_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: LOCAL_PROVIDER_ID.to_owned(),
        display_name: "On-device (MLX / LiteRT-LM)".to_owned(),
        default_base_url: None,
        requires_api_key: false,
        configured: true,
        enabled: true,
        models: vec![local_model_entry()],
    }
}

/// Synthetic configured-provider entry for the on-device provider.
fn local_configured_provider() -> ConfiguredProvider {
    ConfiguredProvider {
        id: LOCAL_PROVIDER_ID.to_owned(),
        display_name: "On-device (MLX / LiteRT-LM)".to_owned(),
        base_url: None,
        enabled: true,
        has_api_key: false,
        protocol: serde_json::Value::String("auto".to_owned()),
        default_model: local_provider_config().default_model,
        models: vec![local_model_entry()],
    }
}

/// Build a [`ConfiguredProvider`] from a config row plus its optional metadata.
fn configured_provider_from_row(
    provider: Provider,
    meta: Option<ProviderMeta>,
) -> ConfiguredProvider {
    let meta = meta.unwrap_or_default();
    ConfiguredProvider {
        display_name: meta.display_name.unwrap_or_else(|| provider.id.clone()),
        base_url: provider.base_url,
        enabled: provider.enabled,
        has_api_key: provider.api_key_ref.is_some(),
        protocol: meta
            .protocol
            .unwrap_or_else(|| serde_json::Value::String("auto".to_owned())),
        default_model: meta.default_model,
        models: meta.models,
        id: provider.id,
    }
}

/// The platform local-provider config, used to derive the on-device model entry.
/// Gated per target so each mobile build advertises its own engine; host builds
/// (tests) use the LiteRT-LM config as a representative stand-in.
fn local_provider_config() -> UarProviderConfig {
    #[cfg(target_os = "ios")]
    {
        crate::uar::ios_qwen35_mlx_provider_config(crate::uar::IOS_MLX_PROVIDER_ID)
    }
    #[cfg(target_os = "android")]
    {
        crate::uar::android_litert_lm_provider_config("gemma-4-e2b-it-litertlm")
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        crate::uar::android_litert_lm_provider_config("gemma-4-e2b-it-litertlm")
    }
}

// ============================================================================
// Metadata persistence + cloud probe.
// ============================================================================

async fn read_provider_meta(
    config: &ConfigBackend,
    id: &str,
) -> Result<Option<ProviderMeta>, AgentError> {
    let Some(value) = config.get_setting(&provider_meta_key(id)).await? else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(|error| AgentError::Config(format!("provider metadata for '{id}': {error}")))
}

async fn write_provider_meta(
    config: &ConfigBackend,
    id: &str,
    meta: &ProviderMeta,
) -> Result<(), AgentError> {
    let value = serde_json::to_value(meta)
        .map_err(|error| AgentError::Config(format!("serialize provider metadata: {error}")))?;
    config.set_setting(&provider_meta_key(id), value).await
}

fn test_failure(id: &str, model_id: &str) -> ProviderTestResult {
    ProviderTestResult {
        ok: false,
        provider_id: id.to_owned(),
        model_id: model_id.to_owned(),
        latency_ms: 0,
        received_text: false,
    }
}

/// Stream a one-token "ping" against a cloud provider, reusing the exact cloud
/// path the routing driver builds (`build_client_config` + `LiterLlmDriver`), and
/// report whether any assistant text arrived.
async fn probe_cloud(
    provider_id: &str,
    model_id: &str,
    base_url: Option<String>,
    api_key: String,
) -> anyhow::Result<bool> {
    use futures::StreamExt;

    let model = format!("{provider_id}/{model_id}");
    let mut llm_config: LlmConfig = serde_json::from_value(serde_json::json!({}))?;
    llm_config.model = model.clone();
    llm_config.api_key = Some(api_key);
    llm_config.base_url = base_url;
    let client_config = build_client_config(&llm_config);
    let driver = LiterLlmDriver::new(client_config, model, None)?;

    let mut stream = driver
        .stream(LlmRequest {
            messages: vec![serde_json::json!({ "role": "user", "content": "ping" })],
            tools: Vec::new(),
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
            extra_params: Some(serde_json::json!({ "max_tokens": 1 })),
        })
        .await?;

    let mut received_text = false;
    while let Some(event) = stream.next().await {
        if let UarNormalizedEvent::MessageDelta { text } = event? {
            if !text.is_empty() {
                received_text = true;
            }
        }
    }
    Ok(received_text)
}

/// Rebuild the [`UarProviderConfig`] for a just-saved provider so it can be
/// registered into the embedded runtime's registry. Returns `None` when the
/// provider row (or its key) is missing.
async fn saved_provider_config(
    config: &ConfigBackend,
    id: &str,
) -> Result<Option<UarProviderConfig>, AgentError> {
    let Some(provider) = config.get_provider(id).await? else {
        return Ok(None);
    };
    let api_key = provider
        .api_key_ref
        .as_deref()
        .map(crate::secrets::resolve_api_key)
        .transpose()
        .ok()
        .flatten();
    let meta = read_provider_meta(config, id).await?.unwrap_or_default();
    Ok(Some(UarProviderConfig {
        id: provider.id.clone(),
        display_name: meta.display_name.unwrap_or_else(|| provider.id.clone()),
        base_url: provider.base_url.unwrap_or_default(),
        api_key,
        protocol: protocol_from_value(meta.protocol.as_ref()),
        default_model: meta.default_model,
        models: meta
            .models
            .into_iter()
            .map(model_config_from_entry)
            .collect(),
        enabled: provider.enabled,
    }))
}

fn protocol_from_value(value: Option<&serde_json::Value>) -> ProtocolSetting {
    match value.and_then(serde_json::Value::as_str) {
        Some("chat") => ProtocolSetting::Chat,
        Some("responses") => ProtocolSetting::Responses,
        _ => ProtocolSetting::Auto,
    }
}

fn model_config_from_entry(model: ProviderModelEntry) -> UarModelConfig {
    UarModelConfig {
        id: model.id,
        display_name: model.display_name,
        context_window: model
            .context_window
            .and_then(|value| u32::try_from(value).ok()),
        supports_vision: model.supports_vision,
        supports_tools: model.supports_tools,
        supports_reasoning: model.supports_reasoning,
        supports_structured_output: model.supports_structured_output,
        supports_streaming: model.supports_streaming,
        max_output_tokens: model
            .max_output_tokens
            .and_then(|value| u32::try_from(value).ok()),
        enabled: model.enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gen_ui_db::relational::{
        config::{ModelPref as PgModelPref, Provider as PgProvider},
        ConfigStore, RelationalError, RelationalResult,
    };
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Hermetic in-memory [`ConfigStore`] so the embedded `*_impl` functions can
    /// be exercised end-to-end without a real DB, the process-global state, or
    /// the platform keychain.
    #[derive(Default)]
    struct MemConfigStore {
        providers: Mutex<Vec<PgProvider>>,
        model_prefs: Mutex<HashMap<(String, String), PgModelPref>>,
        settings: Mutex<HashMap<String, serde_json::Value>>,
    }

    #[async_trait::async_trait]
    impl ConfigStore for MemConfigStore {
        async fn list_providers(&self) -> RelationalResult<Vec<PgProvider>> {
            Ok(self.providers.lock().clone())
        }

        async fn upsert_provider(&self, provider: &PgProvider) -> RelationalResult<()> {
            let mut providers = self.providers.lock();
            if let Some(existing) = providers.iter_mut().find(|row| row.id == provider.id) {
                *existing = provider.clone();
            } else {
                providers.push(provider.clone());
            }
            Ok(())
        }

        async fn delete_provider(&self, id: &str) -> RelationalResult<()> {
            self.providers.lock().retain(|row| row.id != id);
            Ok(())
        }

        async fn get_model_pref(
            &self,
            surface: &str,
            lane: &str,
        ) -> RelationalResult<Option<PgModelPref>> {
            Ok(self
                .model_prefs
                .lock()
                .get(&(surface.to_owned(), lane.to_owned()))
                .cloned())
        }

        async fn upsert_model_pref(&self, pref: &PgModelPref) -> RelationalResult<()> {
            self.model_prefs
                .lock()
                .insert((pref.surface.clone(), pref.lane.clone()), pref.clone());
            Ok(())
        }

        async fn get_setting(&self, key: &str) -> RelationalResult<Option<serde_json::Value>> {
            Ok(self.settings.lock().get(key).cloned())
        }

        async fn set_setting(&self, key: &str, value: serde_json::Value) -> RelationalResult<()> {
            if value.is_null() {
                self.settings.lock().remove(key);
            } else {
                self.settings.lock().insert(key.to_owned(), value);
            }
            let _ = RelationalError::Sync; // keep the import referenced on all feature sets
            Ok(())
        }
    }

    fn mem_backend() -> ConfigBackend {
        ConfigBackend::Postgres(Arc::new(MemConfigStore::default()))
    }

    /// A `save` request that carries no API key, so `save_impl` never touches the
    /// platform keychain (the key-ref preservation path is exercised instead).
    fn keyless_save_request(id: &str) -> SaveProviderRequest {
        SaveProviderRequest {
            id: id.to_owned(),
            kind: id.to_owned(),
            display_name: Some(format!("{id} display")),
            base_url: Some("https://api.example.test".to_owned()),
            api_key: None,
            enabled: true,
            protocol: Some(serde_json::Value::String("chat".to_owned())),
            default_model: Some("model-a".to_owned()),
            models: vec![sample_model_entry("model-a"), sample_model_entry("model-b")],
        }
    }

    #[tokio::test]
    async fn embedded_save_then_list_round_trips_rich_metadata() {
        let config = mem_backend();
        save_impl(&config, keyless_save_request("openai"))
            .await
            .expect("save persists the provider");

        let listed = list_impl(&config).await.expect("list reads back");
        // Local synthetic entry is always present, plus the saved provider.
        assert!(listed
            .iter()
            .any(|provider| provider.id == LOCAL_PROVIDER_ID));
        let saved = listed
            .iter()
            .find(|provider| provider.id == "openai")
            .expect("saved provider lists");
        assert_eq!(saved.display_name, "openai display");
        assert_eq!(saved.base_url.as_deref(), Some("https://api.example.test"));
        assert_eq!(saved.protocol, serde_json::Value::String("chat".to_owned()));
        assert_eq!(saved.default_model.as_deref(), Some("model-a"));
        assert_eq!(saved.models.len(), 2);
        assert_eq!(saved.models[0].id, "model-a");
        assert!(saved.enabled);
        // No key was supplied, so no ref was stored.
        assert!(!saved.has_api_key);
    }

    #[tokio::test]
    async fn embedded_catalog_includes_local_and_real_catalog_providers() {
        let config = mem_backend();
        let entries = catalog_impl(&config).await.expect("catalog builds");

        // The synthetic on-device entry is present and sorts first.
        assert_eq!(entries[0].id, LOCAL_PROVIDER_ID);
        assert!(!entries[0].requires_api_key);
        // The compile-time UAR catalog is embedded, so real providers appear too.
        assert!(
            entries.len() > 1,
            "expected real catalog providers alongside the local entry"
        );
        assert!(entries
            .iter()
            .skip(1)
            .all(|entry| entry.id != LOCAL_PROVIDER_ID));
    }

    #[tokio::test]
    async fn embedded_delete_removes_provider_meta_and_resets_active() {
        let config = mem_backend();
        save_impl(&config, keyless_save_request("openai"))
            .await
            .expect("save persists");
        // Make the saved provider the active route.
        save_cloud_model_impl(
            &config,
            SaveCloudModelRequest {
                provider_id: "openai".to_owned(),
                model_id: "model-a".to_owned(),
            },
        )
        .await
        .expect("select cloud model");

        // Sanity: meta exists before delete.
        assert!(read_provider_meta(&config, "openai")
            .await
            .expect("read meta")
            .is_some());

        delete_impl(&config, "openai")
            .await
            .expect("delete removes");

        // Provider row gone.
        assert!(config
            .get_provider("openai")
            .await
            .expect("get provider")
            .is_none());
        // Metadata blob cleared.
        assert!(read_provider_meta(&config, "openai")
            .await
            .expect("read meta after delete")
            .is_none());
        // Active route reset to the on-device provider.
        let active = config
            .get_setting(SETTING_UAR_ACTIVE_PROVIDER)
            .await
            .expect("read active")
            .and_then(|value| value.as_str().map(str::to_owned));
        assert_eq!(active.as_deref(), Some(LOCAL_PROVIDER_ID));
    }

    #[tokio::test]
    async fn embedded_get_and_save_cloud_model_round_trip() {
        let config = mem_backend();
        // Unset ⇒ no cloud model.
        assert!(get_cloud_model_impl(&config)
            .await
            .expect("get with no selection")
            .is_none());

        save_cloud_model_impl(
            &config,
            SaveCloudModelRequest {
                provider_id: "anthropic".to_owned(),
                model_id: "claude-sonnet-4".to_owned(),
            },
        )
        .await
        .expect("select cloud model");

        // Active provider setting + chat/cloud pref both written.
        let active = config
            .get_setting(SETTING_UAR_ACTIVE_PROVIDER)
            .await
            .expect("read active")
            .and_then(|value| value.as_str().map(str::to_owned));
        assert_eq!(active.as_deref(), Some("anthropic"));

        let got = get_cloud_model_impl(&config)
            .await
            .expect("get after select")
            .expect("a cloud model is now configured");
        assert_eq!(got.provider_id, "anthropic");
        assert_eq!(got.model_id, "claude-sonnet-4");
    }

    #[tokio::test]
    async fn embedded_save_cloud_model_local_id_selects_local_without_model_pref() {
        let config = mem_backend();
        save_cloud_model_impl(
            &config,
            SaveCloudModelRequest {
                provider_id: LOCAL_PROVIDER_ID.to_owned(),
                model_id: String::new(),
            },
        )
        .await
        .expect("select local");

        let active = config
            .get_setting(SETTING_UAR_ACTIVE_PROVIDER)
            .await
            .expect("read active")
            .and_then(|value| value.as_str().map(str::to_owned));
        assert_eq!(active.as_deref(), Some(LOCAL_PROVIDER_ID));
        // Local selection carries no cloud model.
        assert!(get_cloud_model_impl(&config)
            .await
            .expect("get after local select")
            .is_none());
        // No chat/cloud model pref was written for the local lane.
        assert!(config
            .get_model_pref(SURFACE_CHAT, LANE_CLOUD)
            .await
            .expect("read pref")
            .is_none());
    }

    #[tokio::test]
    async fn embedded_test_local_provider_reports_ok_without_network() {
        let config = mem_backend();
        let result = test_impl(&config, LOCAL_PROVIDER_ID, Some("qwen3.5-4b-mlx-4bit"))
            .await
            .expect("local test never fails");
        assert!(result.ok);
        assert_eq!(result.provider_id, LOCAL_PROVIDER_ID);
        assert_eq!(result.model_id, "qwen3.5-4b-mlx-4bit");
        assert_eq!(result.latency_ms, 0);
        assert!(result.received_text);
    }

    #[tokio::test]
    async fn embedded_test_unconfigured_cloud_provider_reports_failure() {
        let config = mem_backend();
        let result = test_impl(&config, "openai", Some("gpt-4o"))
            .await
            .expect("test maps errors to ok:false rather than erroring");
        assert!(!result.ok);
        assert_eq!(result.provider_id, "openai");
        assert!(!result.received_text);
    }

    fn sample_model_entry(id: &str) -> ProviderModelEntry {
        ProviderModelEntry {
            id: id.to_owned(),
            display_name: Some(format!("Model {id}")),
            context_window: Some(128_000),
            max_output_tokens: Some(4_096),
            supports_vision: true,
            supports_tools: true,
            supports_reasoning: false,
            supports_structured_output: true,
            supports_streaming: true,
            enabled: true,
        }
    }

    #[test]
    fn model_entry_from_catalog_maps_capabilities_and_limits() {
        let mut model = ModelInfo {
            id: "gpt-4o".to_owned(),
            name: "GPT-4o".to_owned(),
            ..ModelInfo::default()
        };
        model.limits.context_window = 128_000;
        model.limits.max_output = 16_384;
        model.modalities.input = vec!["text".to_owned(), "image".to_owned()];
        model.capabilities.tool_call = true;
        model.capabilities.reasoning = true;
        model.capabilities.structured_output = true;
        model.capabilities.streaming = true;

        let entry = model_entry_from_catalog(&model);
        assert_eq!(entry.id, "gpt-4o");
        assert_eq!(entry.display_name.as_deref(), Some("GPT-4o"));
        assert_eq!(entry.context_window, Some(128_000));
        assert_eq!(entry.max_output_tokens, Some(16_384));
        assert!(entry.supports_vision);
        assert!(entry.supports_tools);
        assert!(entry.supports_reasoning);
        assert!(entry.supports_structured_output);
        assert!(entry.supports_streaming);
        assert!(entry.enabled);
    }

    #[test]
    fn model_entry_from_catalog_treats_zero_limits_as_absent() {
        let model = ModelInfo {
            id: "text-only".to_owned(),
            ..ModelInfo::default()
        };
        let entry = model_entry_from_catalog(&model);
        assert_eq!(entry.context_window, None);
        assert_eq!(entry.max_output_tokens, None);
        assert!(!entry.supports_vision);
        assert!(entry.display_name.is_none());
    }

    #[test]
    fn catalog_entry_from_provider_info_uses_id_when_display_name_missing() {
        let provider = ProviderInfo {
            id: "acme".to_owned(),
            display_name: String::new(),
            base_url: Some("https://api.acme.test".to_owned()),
            ..ProviderInfo::default()
        };
        let entry = catalog_entry_from_provider_info(&provider, true, false);
        assert_eq!(entry.id, "acme");
        assert_eq!(entry.display_name, "acme");
        assert_eq!(
            entry.default_base_url.as_deref(),
            Some("https://api.acme.test")
        );
        assert!(entry.requires_api_key);
        assert!(entry.configured);
        assert!(!entry.enabled);
    }

    #[test]
    fn local_catalog_entry_is_credential_free_and_configured() {
        let entry = local_catalog_entry();
        assert_eq!(entry.id, LOCAL_PROVIDER_ID);
        assert_eq!(entry.display_name, "On-device (MLX / LiteRT-LM)");
        assert!(!entry.requires_api_key);
        assert!(entry.configured);
        assert!(entry.enabled);
        assert_eq!(entry.models.len(), 1);
        assert!(entry.default_base_url.is_none());
    }

    #[test]
    fn local_configured_provider_has_no_api_key() {
        let provider = local_configured_provider();
        assert_eq!(provider.id, LOCAL_PROVIDER_ID);
        assert!(!provider.has_api_key);
        assert!(provider.enabled);
        assert_eq!(provider.models.len(), 1);
        assert!(provider.default_model.is_some());
    }

    #[test]
    fn configured_provider_from_row_enriches_from_meta() {
        let provider = Provider {
            id: "openai".to_owned(),
            kind: "openai".to_owned(),
            base_url: Some("https://api.openai.com".to_owned()),
            api_key_ref: Some("cloud:openai".to_owned()),
            enabled: true,
        };
        let meta = ProviderMeta {
            display_name: Some("OpenAI".to_owned()),
            protocol: Some(serde_json::Value::String("chat".to_owned())),
            default_model: Some("gpt-4o".to_owned()),
            models: vec![sample_model_entry("gpt-4o")],
        };
        let configured = configured_provider_from_row(provider, Some(meta));
        assert_eq!(configured.id, "openai");
        assert_eq!(configured.display_name, "OpenAI");
        assert!(configured.has_api_key);
        assert_eq!(
            configured.protocol,
            serde_json::Value::String("chat".to_owned())
        );
        assert_eq!(configured.default_model.as_deref(), Some("gpt-4o"));
        assert_eq!(configured.models.len(), 1);
        assert_eq!(configured.models[0].id, "gpt-4o");
    }

    #[test]
    fn configured_provider_from_row_defaults_when_meta_absent() {
        let provider = Provider {
            id: "groq".to_owned(),
            kind: "groq".to_owned(),
            base_url: None,
            api_key_ref: None,
            enabled: false,
        };
        let configured = configured_provider_from_row(provider, None);
        assert_eq!(configured.display_name, "groq");
        assert!(!configured.has_api_key);
        assert!(!configured.enabled);
        assert_eq!(
            configured.protocol,
            serde_json::Value::String("auto".to_owned())
        );
        assert!(configured.default_model.is_none());
        assert!(configured.models.is_empty());
    }

    #[test]
    fn provider_meta_round_trips_through_json() {
        let meta = ProviderMeta {
            display_name: Some("OpenAI".to_owned()),
            protocol: Some(serde_json::Value::String("responses".to_owned())),
            default_model: Some("gpt-4o".to_owned()),
            models: vec![
                sample_model_entry("gpt-4o"),
                sample_model_entry("gpt-4o-mini"),
            ],
        };
        let json = serde_json::to_value(&meta).unwrap();
        let restored: ProviderMeta = serde_json::from_value(json).unwrap();
        assert_eq!(restored.display_name.as_deref(), Some("OpenAI"));
        assert_eq!(restored.default_model.as_deref(), Some("gpt-4o"));
        assert_eq!(restored.models.len(), 2);
        assert_eq!(restored.models[1].id, "gpt-4o-mini");
    }

    #[test]
    fn provider_meta_key_and_secret_ref_are_id_scoped() {
        assert_eq!(provider_meta_key("openai"), "provider_meta:openai");
        assert_eq!(cloud_secret_ref("openai"), "cloud:openai");
    }

    #[test]
    fn protocol_from_value_maps_known_variants() {
        assert!(matches!(
            protocol_from_value(Some(&serde_json::Value::String("chat".into()))),
            ProtocolSetting::Chat
        ));
        assert!(matches!(
            protocol_from_value(Some(&serde_json::Value::String("responses".into()))),
            ProtocolSetting::Responses
        ));
        assert!(matches!(protocol_from_value(None), ProtocolSetting::Auto));
        assert!(matches!(
            protocol_from_value(Some(&serde_json::Value::String("weird".into()))),
            ProtocolSetting::Auto
        ));
    }

    #[test]
    fn model_config_from_entry_narrows_token_widths() {
        let entry = sample_model_entry("gpt-4o");
        let config = model_config_from_entry(entry);
        assert_eq!(config.id, "gpt-4o");
        assert_eq!(config.context_window, Some(128_000));
        assert_eq!(config.max_output_tokens, Some(4_096));
        assert!(config.supports_vision);
        assert!(config.enabled);
    }
}
