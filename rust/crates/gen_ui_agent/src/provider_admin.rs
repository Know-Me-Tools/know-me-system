// TJ-ARCH-MOB-001 compliant
//! Intent-level cloud-provider administration shared by desktop and mobile.
//! Provider metadata lives in the config store; plaintext credentials live only
//! in platform-secure storage and are never serialized back to a caller.

use serde::{Deserialize, Serialize};

use crate::{
    config::{ModelPref, Provider},
    error::AgentError,
    secrets, state,
};

const CLOUD_SURFACE: &str = "chat";
const CLOUD_LANE: &str = "cloud";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub default_base_url: Option<String>,
    pub requires_api_key: bool,
    pub supports_chat: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguredProvider {
    pub id: String,
    pub kind: String,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub has_api_key: bool,
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
    pub kind: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCloudModelRequest {
    pub provider_id: String,
    pub model_id: String,
}

pub fn catalog() -> Result<Vec<ProviderCatalogEntry>, AgentError> {
    let complex = liter_llm::provider::complex_provider_names()
        .map_err(|error| AgentError::Config(format!("Liter-LLM provider registry: {error}")))?;
    let mut entries = liter_llm::provider::all_providers()
        .map_err(|error| AgentError::Config(format!("Liter-LLM provider registry: {error}")))?
        .iter()
        .filter(|provider| {
            let supports_chat = provider
                .endpoints
                .as_ref()
                .is_some_and(|endpoints| endpoints.iter().any(|endpoint| endpoint == "chat"));
            let accepts_api_key = provider
                .auth
                .as_ref()
                .is_some_and(|auth| auth.auth_type != liter_llm::provider::AuthType::None);
            supports_chat && accepts_api_key && !complex.contains(&provider.name)
        })
        .map(|provider| ProviderCatalogEntry {
            id: provider.name.clone(),
            display_name: provider
                .display_name
                .clone()
                .unwrap_or_else(|| provider.name.clone()),
            default_base_url: provider.base_url.clone(),
            requires_api_key: provider
                .auth
                .as_ref()
                .is_some_and(|auth| auth.auth_type != liter_llm::provider::AuthType::None),
            supports_chat: true,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    Ok(entries)
}

pub async fn list() -> Result<Vec<ConfiguredProvider>, AgentError> {
    Ok(state::config()?
        .list_providers()
        .await?
        .into_iter()
        .map(|provider| ConfiguredProvider {
            has_api_key: provider.api_key_ref.is_some(),
            id: provider.id,
            kind: provider.kind,
            base_url: provider.base_url,
            enabled: provider.enabled,
        })
        .collect())
}

pub async fn save(request: SaveProviderRequest) -> Result<(), AgentError> {
    let id = request.id.trim();
    let kind = request.kind.trim();
    if id.is_empty() || kind.is_empty() {
        return Err(AgentError::Config(
            "provider id and provider kind are required".into(),
        ));
    }
    let known = catalog()?.into_iter().any(|entry| entry.id == kind);
    if !known {
        return Err(AgentError::Config(format!(
            "provider kind '{kind}' is not in this Liter-LLM build"
        )));
    }

    let existing = state::config()?.get_provider(id).await?;
    let api_key_ref = match request.api_key.as_deref().map(str::trim) {
        Some(api_key) if !api_key.is_empty() => {
            let reference = format!("provider:{id}");
            secrets::store_api_key(&reference, api_key)?;
            Some(reference)
        }
        _ => existing.and_then(|provider| provider.api_key_ref),
    };
    let base_url = request
        .base_url
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    state::config()?
        .upsert_provider(&Provider {
            id: id.to_owned(),
            kind: kind.to_owned(),
            base_url,
            api_key_ref,
            enabled: request.enabled,
        })
        .await
}

pub async fn delete(id: &str) -> Result<(), AgentError> {
    let id = id.trim();
    if let Some(provider) = state::config()?.get_provider(id).await? {
        if let Some(reference) = provider.api_key_ref {
            secrets::delete_api_key(&reference)?;
        }
    }
    state::config()?.delete_provider(id).await
}

pub async fn get_cloud_model() -> Result<Option<ConfiguredCloudModel>, AgentError> {
    Ok(state::config()?
        .get_model_pref(CLOUD_SURFACE, CLOUD_LANE)
        .await?
        .and_then(|pref| {
            pref.provider_id.map(|provider_id| ConfiguredCloudModel {
                provider_id,
                model_id: pref.model_id,
            })
        }))
}

pub async fn save_cloud_model(request: SaveCloudModelRequest) -> Result<(), AgentError> {
    let provider_id = request.provider_id.trim();
    let model_id = request.model_id.trim();
    if provider_id.is_empty() || model_id.is_empty() {
        return Err(AgentError::Config(
            "provider and model identifiers are required".into(),
        ));
    }
    let provider = state::config()?.get_provider(provider_id).await?;
    if !provider.is_some_and(|provider| provider.enabled) {
        return Err(AgentError::Config(format!(
            "provider '{provider_id}' is missing or disabled"
        )));
    }
    state::config()?
        .upsert_model_pref(
            CLOUD_SURFACE,
            CLOUD_LANE,
            &ModelPref {
                provider_id: Some(provider_id.to_owned()),
                model_id: model_id.to_owned(),
                params: serde_json::json!({}),
            },
        )
        .await
}
