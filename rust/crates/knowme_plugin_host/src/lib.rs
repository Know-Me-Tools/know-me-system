//! # knowme_plugin_host — WASM plugin host (FUNC-SPEC §08)
//!
//! Third parties extend KnowMe through WASM components built against the
//! `knowme:plugin@0.1.0` WIT contract (`wit/knowme-plugin.wit` in this crate).
//! Plugins are content-addressed (IPFS CIDs), SHA-256-verified after fetch,
//! optionally Ed25519-signed by a publisher `did:key:`, and sandboxed by
//! Wasmtime with the Component Model: 64 MiB linear memory, 30 s per
//! `handle-event` via epoch interruption, no direct sockets, no threads.
//! Network access exists only through `uar-invoke("web-fetch", …)` under
//! Cedar policy and the manifest allow-list.
//!
//! ## What is real in this skeleton
//! The manifest and permission domain model — the exact shape the installer
//! dialog (UX-MOOD Screen 07) renders before any code runs — plus signature
//! status and the install-time verification pipeline's types.
//!
//! ## What is stubbed
//! The Wasmtime host itself. `wasmtime::component::bindgen!()` against the
//! WIT file, the IPFS fetcher with gateway fallback, and hash/signature
//! verification are tracked in `docs/implementation-roadmap.md` (M3).
//! The commented dependencies in Cargo.toml mark the intended crates.
//!
//! Spec anchors: FUNC-SPEC §08 (Plugin System), §09 (sandbox guarantee),
//! UX-MOOD Screen 07.

use serde::{Deserialize, Serialize};

/// v0.1 permission set. `network` is intentionally absent — the network is
/// reached only via `uar-invoke("web-fetch", …)`, governed by Cedar.
/// v0.2 candidates (librefang) are parsed but rejected by v0.1 hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Scoped key-value storage, namespaced per plugin.
    Storage,
    /// May call `uar-invoke` for skills listed in `uar_skill_allowlist`.
    UarSkills,
}

/// Content address + integrity hash for the WASM blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmRef {
    /// IPFS CID of `plugin.wasm`.
    pub cid: String,
    /// SHA-256 of the bytes, verified after every fetch. Even a malicious
    /// gateway cannot serve a tampered binary.
    pub sha256: String,
}

/// Ed25519 signature over the canonical manifest JSON, bound to the
/// publisher's `did:key:`. Optional — unsigned plugins install with a warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSignature {
    pub publisher_did: String,
    pub signature_b64: String,
}

/// `manifest.json` — everything the host needs to install, sandbox, and
/// govern the plugin (FUNC-SPEC §08 manifest table).
///
/// `ui` and `navigation` are v0.2 candidates: reserved in the schema, ignored
/// by v0.1 hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Globally unique. Reverse-DNS recommended.
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub wasm: WasmRef,
    pub permissions: Vec<PluginPermission>,
    /// Skills this plugin may pass to `uar-invoke`. Empty means none.
    #[serde(default)]
    pub uar_skill_allowlist: Vec<String>,
    #[serde(default)]
    pub signature: Option<ManifestSignature>,
    /// v0.2 candidate — reserved; ignored by v0.1 hosts.
    #[serde(default)]
    pub ui: Option<serde_json::Value>,
    /// v0.2 candidate — reserved; ignored by v0.1 hosts.
    #[serde(default)]
    pub navigation: Option<serde_json::Value>,
}

/// What the installer dialog shows: green "Ed25519 verified", amber
/// "unsigned" (requires the explicit trust-unverified toggle), or red.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignatureStatus {
    Verified,
    Unsigned,
    Invalid,
}

/// Sandbox limits applied to every instantiation (FUNC-SPEC §08, "The sandbox").
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SandboxLimits {
    pub max_linear_memory_bytes: u64,
    pub max_event_duration_secs: u64,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_linear_memory_bytes: 64 * 1024 * 1024,
            max_event_duration_secs: 30,
        }
    }
}

/// An installed plugin record. The permission grant is immutable from install;
/// updating a plugin requires re-consent if the permission set changed
/// (FUNC-SPEC §09, "What KnowMe does not do").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub signature_status: SignatureStatus,
    pub granted_permissions: Vec<PluginPermission>,
    pub installed_at_ms: u64,
    pub enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginHostError {
    #[error("manifest parse error: {0}")]
    Manifest(String),
    #[error("sha256 mismatch for CID {cid}: expected {expected}, got {actual}")]
    HashMismatch {
        cid: String,
        expected: String,
        actual: String,
    },
    #[error("signature invalid for publisher {0}")]
    SignatureInvalid(String),
    #[error("permission '{0:?}' not granted")]
    PermissionDenied(PluginPermission),
    #[error("wasmtime host not wired yet — see docs/implementation-roadmap.md (M3)")]
    NotImplemented,
}

/// Parse and structurally validate a manifest. Fetching, hash verification,
/// and signature verification are M3 work.
pub fn parse_manifest(json: &str) -> Result<PluginManifest, PluginHostError> {
    let manifest: PluginManifest =
        serde_json::from_str(json).map_err(|e| PluginHostError::Manifest(e.to_string()))?;
    if manifest.id.is_empty() {
        return Err(PluginHostError::Manifest("id must not be empty".into()));
    }
    if manifest.wasm.cid.is_empty() || manifest.wasm.sha256.is_empty() {
        return Err(PluginHostError::Manifest(
            "wasm.cid and wasm.sha256 are required".into(),
        ));
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_manifest() {
        let json = r#"{
            "id": "tools.know-me.examples.hello",
            "name": "Hello KnowMe",
            "version": "0.1.0",
            "description": "Example plugin",
            "wasm": { "cid": "bafkreigexample", "sha256": "deadbeef" },
            "permissions": ["storage"]
        }"#;
        let m = parse_manifest(json).expect("valid manifest");
        assert_eq!(m.permissions, vec![PluginPermission::Storage]);
        assert!(m.signature.is_none());
    }

    #[test]
    fn default_sandbox_limits_match_spec() {
        let l = SandboxLimits::default();
        assert_eq!(l.max_linear_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(l.max_event_duration_secs, 30);
    }
}
