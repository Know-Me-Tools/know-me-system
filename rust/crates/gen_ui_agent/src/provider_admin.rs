// TJ-ARCH-MOB-001 compliant
//! UAR-owned LLM provider administration.
//!
//! KnowMe never persists cloud-provider credentials or duplicates Liter-LLM's
//! catalog. Every operation is delegated to the configured Universal Agent
//! Runtime; responses are secret-free and suitable for React or Flutter.

use serde::{Deserialize, Serialize};

use crate::{chat, error::AgentError, uar::UarDeploymentMode};

#[path = "provider_admin_embedded.rs"]
mod embedded;

/// Which provider-administration backend serves this platform. The service
/// backend delegates every call to the HTTP UAR; the embedded backend serves
/// the same operations from the on-device config store + secrets, so the mobile
/// "Model providers" tab works with no `uar_service_url`.
enum Backend {
    Embedded,
    Service,
}

/// Select the provider-admin backend from the configured deployment mode. Only
/// [`UarDeploymentMode::EmbeddedLibrary`] uses the on-device path; the HTTP and
/// cloud service modes both go through the service client.
async fn backend() -> Result<Backend, AgentError> {
    match chat::selected_uar_deployment_mode().await? {
        UarDeploymentMode::EmbeddedLibrary => Ok(Backend::Embedded),
        UarDeploymentMode::HttpService | UarDeploymentMode::CloudService => Ok(Backend::Service),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub default_base_url: Option<String>,
    pub requires_api_key: bool,
    pub configured: bool,
    pub enabled: bool,
    pub models: Vec<ProviderModelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelEntry {
    pub id: String,
    pub display_name: Option<String>,
    pub context_window: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_reasoning: bool,
    pub supports_structured_output: bool,
    pub supports_streaming: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguredProvider {
    pub id: String,
    pub display_name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub has_api_key: bool,
    pub protocol: serde_json::Value,
    pub default_model: Option<String>,
    pub models: Vec<ProviderModelEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguredCloudModel {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderRequest {
    pub id: String,
    /// Catalog provider id retained for compatibility with the first settings UI.
    pub kind: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub protocol: Option<serde_json::Value>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ProviderModelEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCloudModelRequest {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTestResult {
    pub ok: bool,
    pub provider_id: String,
    pub model_id: String,
    pub latency_ms: u128,
    pub received_text: bool,
}

pub async fn catalog() -> Result<Vec<ProviderCatalogEntry>, AgentError> {
    match backend().await? {
        Backend::Embedded => return embedded::catalog().await,
        Backend::Service => {}
    }
    let client = chat::uar_service_client().await?;
    let providers_api = client.providers();
    let (catalog, configured) =
        futures::try_join!(providers_api.catalog(), providers_api.list()).map_err(uar_error)?;
    let configured = configured
        .providers
        .into_iter()
        .map(|provider| (provider.id.clone(), provider))
        .collect::<std::collections::HashMap<_, _>>();
    let root = catalog.as_object().ok_or_else(|| {
        AgentError::Client("UAR provider catalog response must be an object".into())
    })?;
    let mut entries = root
        .iter()
        .map(|(id, value)| {
            let configured_provider = configured.get(id);
            let models = value
                .get("models")
                .and_then(serde_json::Value::as_object)
                .map(|models| {
                    models
                        .iter()
                        .map(|(model_id, model)| model_from_catalog(model_id, model))
                        .collect()
                })
                .unwrap_or_default();
            ProviderCatalogEntry {
                id: id.clone(),
                display_name: value
                    .get("display_name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(id)
                    .to_string(),
                default_base_url: value
                    .get("base_url")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string),
                requires_api_key: true,
                configured: configured_provider.is_some(),
                enabled: configured_provider.is_some_and(|provider| provider.enabled),
                models,
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .configured
            .cmp(&left.configured)
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    Ok(entries)
}

pub async fn list() -> Result<Vec<ConfiguredProvider>, AgentError> {
    match backend().await? {
        Backend::Embedded => return embedded::list().await,
        Backend::Service => {}
    }
    let client = chat::uar_service_client().await?;
    let response = client.providers().list().await.map_err(uar_error)?;
    let mut providers = response
        .providers
        .into_iter()
        .map(configured_provider_from_sdk)
        .collect::<Vec<_>>();
    providers.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    Ok(providers)
}

pub async fn save(request: SaveProviderRequest) -> Result<(), AgentError> {
    let id = request.id.trim();
    let catalog_id = request.kind.trim();
    if id.is_empty() || catalog_id.is_empty() {
        return Err(AgentError::Config(
            "provider id and catalog provider id are required".into(),
        ));
    }
    match backend().await? {
        Backend::Embedded => return embedded::save(request).await,
        Backend::Service => {}
    }
    let client = chat::uar_service_client().await?;
    let existing = client
        .providers()
        .list()
        .await
        .map_err(uar_error)?
        .providers
        .into_iter()
        .find(|provider| provider.id == id);
    let catalog = client.providers().catalog().await.map_err(uar_error)?;
    let catalog_provider = catalog.get(catalog_id);
    let display_name = request
        .display_name
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            existing
                .as_ref()
                .map(|provider| provider.display_name.clone())
        })
        .or_else(|| {
            catalog_provider
                .and_then(|provider| provider.get("display_name"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| id.to_string());
    let base_url = request
        .base_url
        .filter(|value| !value.trim().is_empty())
        .or_else(|| existing.as_ref().map(|provider| provider.base_url.clone()))
        .or_else(|| {
            catalog_provider
                .and_then(|provider| provider.get("base_url"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    let models = if request.models.is_empty() {
        existing
            .as_ref()
            .map(|provider| provider.models.clone())
            .unwrap_or_default()
    } else {
        request.models.into_iter().map(model_to_sdk).collect()
    };
    let default_model = request
        .default_model
        .or_else(|| {
            existing
                .as_ref()
                .and_then(|provider| provider.default_model.clone())
        })
        .or_else(|| {
            models
                .iter()
                .find(|model| model.enabled)
                .map(|model| model.id.clone())
        });
    client
        .providers()
        .save(universal_agent_runtime_sdk::SaveProviderConfig {
            id: id.to_string(),
            display_name,
            base_url,
            api_key: request.api_key.filter(|value| !value.trim().is_empty()),
            protocol: request
                .protocol
                .or_else(|| existing.as_ref().map(|provider| provider.protocol.clone()))
                .unwrap_or_else(|| serde_json::Value::String("auto".into())),
            default_model,
            models,
            enabled: request.enabled,
        })
        .await
        .map_err(uar_error)?;
    Ok(())
}

pub async fn delete(id: &str) -> Result<(), AgentError> {
    match backend().await? {
        Backend::Embedded => return embedded::delete(id.trim()).await,
        Backend::Service => {}
    }
    chat::uar_service_client()
        .await?
        .providers()
        .delete(id.trim())
        .await
        .map_err(uar_error)
}

pub async fn test(id: &str, model: Option<&str>) -> Result<ProviderTestResult, AgentError> {
    match backend().await? {
        Backend::Embedded => return embedded::test(id.trim(), model).await,
        Backend::Service => {}
    }
    let result = chat::uar_service_client()
        .await?
        .providers()
        .test(id.trim(), model)
        .await
        .map_err(uar_error)?;
    Ok(ProviderTestResult {
        ok: result.ok,
        provider_id: result.provider_id,
        model_id: result.model_id,
        latency_ms: result.latency_ms,
        received_text: result.received_text,
    })
}

pub async fn get_cloud_model() -> Result<Option<ConfiguredCloudModel>, AgentError> {
    match backend().await? {
        Backend::Embedded => return embedded::get_cloud_model().await,
        Backend::Service => {}
    }
    let response = chat::uar_service_client()
        .await?
        .providers()
        .list()
        .await
        .map_err(uar_error)?;
    let Some(provider_id) = response.default_id else {
        return Ok(None);
    };
    Ok(response
        .providers
        .into_iter()
        .find(|provider| provider.id == provider_id)
        .and_then(|provider| {
            provider.default_model.map(|model_id| ConfiguredCloudModel {
                provider_id,
                model_id,
            })
        }))
}

pub async fn save_cloud_model(request: SaveCloudModelRequest) -> Result<(), AgentError> {
    match backend().await? {
        Backend::Embedded => return embedded::save_cloud_model(request).await,
        Backend::Service => {}
    }
    let client = chat::uar_service_client().await?;
    let response = client.providers().list().await.map_err(uar_error)?;
    let provider = response
        .providers
        .into_iter()
        .find(|provider| provider.id == request.provider_id)
        .ok_or_else(|| {
            AgentError::Config(format!(
                "provider '{}' is not configured",
                request.provider_id
            ))
        })?;
    if !provider.enabled {
        return Err(AgentError::Config(format!(
            "provider '{}' is disabled",
            request.provider_id
        )));
    }
    if provider
        .models
        .iter()
        .find(|model| model.id == request.model_id)
        .is_some_and(|model| !model.enabled)
    {
        return Err(AgentError::Config(format!(
            "model '{}' is disabled",
            request.model_id
        )));
    }
    client
        .providers()
        .save(universal_agent_runtime_sdk::SaveProviderConfig {
            id: provider.id.clone(),
            display_name: provider.display_name,
            base_url: provider.base_url,
            api_key: None,
            protocol: provider.protocol,
            default_model: Some(request.model_id),
            models: provider.models,
            enabled: provider.enabled,
        })
        .await
        .map_err(uar_error)?;
    client
        .providers()
        .set_default(&provider.id)
        .await
        .map_err(uar_error)
}

fn configured_provider_from_sdk(
    provider: universal_agent_runtime_sdk::ProviderView,
) -> ConfiguredProvider {
    ConfiguredProvider {
        id: provider.id,
        display_name: provider.display_name,
        base_url: (!provider.base_url.is_empty()).then_some(provider.base_url),
        enabled: provider.enabled,
        has_api_key: provider.credential_configured,
        protocol: provider.protocol,
        default_model: provider.default_model,
        models: provider.models.into_iter().map(model_from_sdk).collect(),
    }
}

fn model_from_sdk(model: universal_agent_runtime_sdk::ProviderModelConfig) -> ProviderModelEntry {
    ProviderModelEntry {
        id: model.id,
        display_name: model.display_name,
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

fn model_to_sdk(model: ProviderModelEntry) -> universal_agent_runtime_sdk::ProviderModelConfig {
    universal_agent_runtime_sdk::ProviderModelConfig {
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

fn model_from_catalog(id: &str, value: &serde_json::Value) -> ProviderModelEntry {
    let inputs = value
        .pointer("/modalities/input")
        .and_then(serde_json::Value::as_array);
    ProviderModelEntry {
        id: id.to_string(),
        display_name: value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        context_window: value
            .pointer("/limit/context")
            .and_then(serde_json::Value::as_u64),
        max_output_tokens: value
            .pointer("/limit/output")
            .and_then(serde_json::Value::as_u64),
        supports_vision: inputs
            .is_some_and(|values| values.iter().any(|value| value.as_str() == Some("image"))),
        supports_tools: value
            .get("tool_call")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        supports_reasoning: value
            .get("reasoning")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        supports_structured_output: value
            .get("structured_output")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        supports_streaming: value
            .get("streaming")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        enabled: value
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    }
}

fn uar_error(error: universal_agent_runtime_sdk::Error) -> AgentError {
    AgentError::Client(format!("UAR provider administration: {error}"))
}
