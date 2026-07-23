//! Per-guest host state (design D1, D4).
//!
//! [`PluginHostState`] is the data wasmtime hands to every host-trait impl:
//! the guest's [`GuestContext`] (identity, capability set, dispatch backend,
//! limits), the Component Model [`ResourceTable`], and the two WASI
//! contexts. Both WASI contexts are deny-by-default and always constructed:
//!
//! * `WasiCtx` — no preopens, no env, no stdio inheritance, no sockets.
//!   Guest language runtimes (CPython, StarlingMonkey, Go) call
//!   `wasi:cli/environment` during init and must get empty-but-valid
//!   responses rather than link failures.
//! * `WasiHttpCtx` — always bound (StarlingMonkey's preview2-shim
//!   auto-imports `wasi:http/types` even when unused) with no outbound
//!   request handler, so no HTTP flows without an explicit backend.
//!
//! This is the single most expensive librefang lesson, adopted verbatim per
//! design D4 — the *pattern*, not the code.

use std::collections::HashSet;
use std::sync::Arc;

use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::p2::{WasiHttpCtxView, WasiHttpView};
use wasmtime_wasi_http::WasiHttpCtx;

use crate::dispatch::HostDispatch;
use crate::sandbox::error::HostError;
use crate::sandbox::limits::MemoryLimiter;
use crate::SandboxLimits;

/// A host capability, mirroring the WIT host-interface names (design D1).
///
/// `plugin-lifecycle-registry` will map manifest capability cards onto this
/// enum; the sandbox itself only ever sees this vocabulary. The interface
/// name is the canonical serialization (used in `MissingCapability` errors
/// and audit logs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HostCapability {
    /// `fs` — sandboxed filesystem.
    Fs,
    /// `env` — granted environment variables.
    Env,
    /// `time` — wall clock (no gating per the WIT, tracked for audit).
    Time,
    /// `host-net` — SSRF-guarded HTTP.
    Net,
    /// `host-uar` — Cedar-gated UAR skill invocation.
    Uar,
    /// `host-kv` — plugin-namespaced key-value storage.
    Storage,
    /// `host-log` — structured host logging (no gating, tracked for audit).
    Log,
    /// `host-settings` — plugin settings namespace.
    Settings,
    /// `host-events` — AG-UI event bus.
    Events,
    /// `host-infer` — host-mediated inference.
    Infer,
    /// `host-memory` — provenance-tagged memory recall.
    Memory,
    /// `host-a2ui` — A2UI surface push (app-shell world only).
    A2ui,
}

impl HostCapability {
    /// The WIT interface name this capability gates.
    pub fn interface_name(self) -> &'static str {
        match self {
            HostCapability::Fs => "fs",
            HostCapability::Env => "env",
            HostCapability::Time => "time",
            HostCapability::Net => "host-net",
            HostCapability::Uar => "host-uar",
            HostCapability::Storage => "host-kv",
            HostCapability::Log => "host-log",
            HostCapability::Settings => "host-settings",
            HostCapability::Events => "host-events",
            HostCapability::Infer => "host-infer",
            HostCapability::Memory => "host-memory",
            HostCapability::A2ui => "host-a2ui",
        }
    }

    /// Inverse of [`HostCapability::interface_name`]. `host-types` has no
    /// runtime surface (types only) and returns `None`.
    pub fn from_interface_name(name: &str) -> Option<Self> {
        match name {
            "fs" => Some(HostCapability::Fs),
            "env" => Some(HostCapability::Env),
            "time" => Some(HostCapability::Time),
            "host-net" => Some(HostCapability::Net),
            "host-uar" => Some(HostCapability::Uar),
            "host-kv" => Some(HostCapability::Storage),
            "host-log" => Some(HostCapability::Log),
            "host-settings" => Some(HostCapability::Settings),
            "host-events" => Some(HostCapability::Events),
            "host-infer" => Some(HostCapability::Infer),
            "host-memory" => Some(HostCapability::Memory),
            "host-a2ui" => Some(HostCapability::A2ui),
            _ => None,
        }
    }

    /// Capabilities that are always wired and never denied at the boundary,
    /// regardless of the grant set: every plugin may log and read the clock
    /// (the WIT documents both as ungated). `host-types` needs no wiring —
    /// it carries types only, so components have no runtime import for it.
    pub const UNGATED: &'static [HostCapability] = &[HostCapability::Log, HostCapability::Time];

    /// Whether calls through this capability's interface are allowed without
    /// an explicit grant.
    pub fn is_ungated(self) -> bool {
        Self::UNGATED.contains(&self)
    }
}

/// Everything a host call needs to know about the calling guest. Shared by
/// reference with the dispatch backend (each `HostDispatch` method takes
/// `&GuestContext`).
#[derive(Clone)]
pub struct GuestContext {
    /// The installed plugin's manifest id. Own-namespace rules (KV,
    /// settings) are evaluated against this.
    pub plugin_id: String,
    /// The granted capability set — checked at the sandbox boundary before
    /// any dispatch method runs.
    pub capabilities: HashSet<HostCapability>,
    /// The installed dispatch backend for this guest.
    pub dispatch: Arc<dyn HostDispatch>,
    /// Resource limits applied to this guest's store.
    pub limits: SandboxLimits,
}

impl GuestContext {
    /// The boundary capability check (design D2's single choke point).
    /// Ungated capabilities (log, time) always pass.
    pub fn require(&self, capability: HostCapability) -> Result<(), HostError> {
        if capability.is_ungated() || self.capabilities.contains(&capability) {
            Ok(())
        } else {
            Err(HostError::CapabilityDenied(format!(
                "capability `{}` is not granted to plugin `{}`",
                capability.interface_name(),
                self.plugin_id
            )))
        }
    }
}

/// State carried in each component `Store`. Constructed once per instance.
pub struct PluginHostState {
    /// Per-guest context: identity, capabilities, dispatch, limits.
    pub guest: GuestContext,
    /// Component Model resource table — required by the WASI views and by
    /// any interface using `result<T, E>` across the ABI.
    pub table: ResourceTable,
    /// Deny-by-default WASI Preview 2 context (design D4): no preopens, no
    /// env vars, no stdio inheritance, no network.
    pub wasi: WasiCtx,
    /// Always-on deny-by-default WASI HTTP context (design D4): bound so
    /// guests whose runtimes auto-import `wasi:http/types` link
    /// successfully; no outbound request handler is registered, so any
    /// attempted outbound request fails.
    pub wasi_http: WasiHttpCtx,
    /// Linear-memory ceiling for this guest's store (design D5).
    pub limiter: MemoryLimiter,
}

impl PluginHostState {
    pub fn new(guest: GuestContext) -> Self {
        let limiter = MemoryLimiter::new(guest.limits.max_linear_memory_bytes);
        Self {
            guest,
            table: ResourceTable::new(),
            wasi: WasiCtxBuilder::new().build(),
            wasi_http: WasiHttpCtx::new(),
            limiter,
        }
    }
}

/// Bridges the state to `wasmtime_wasi::p2::add_to_linker_async`.
impl WasiView for PluginHostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// Bridges the state to `wasmtime_wasi_http::p2::add_only_http_to_linker_async`.
impl WasiHttpView for PluginHostState {
    fn http(&mut self) -> WasiHttpCtxView<'_> {
        WasiHttpCtxView {
            ctx: &mut self.wasi_http,
            table: &mut self.table,
            hooks: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_lookup_by_interface_name_roundtrips() {
        let all = [
            HostCapability::Fs,
            HostCapability::Env,
            HostCapability::Time,
            HostCapability::Net,
            HostCapability::Uar,
            HostCapability::Storage,
            HostCapability::Log,
            HostCapability::Settings,
            HostCapability::Events,
            HostCapability::Infer,
            HostCapability::Memory,
            HostCapability::A2ui,
        ];
        for cap in all {
            assert_eq!(
                HostCapability::from_interface_name(cap.interface_name()),
                Some(cap),
                "roundtrip failed for {:?}",
                cap
            );
        }
        // Types-only interface has no runtime capability.
        assert_eq!(HostCapability::from_interface_name("host-types"), None);
        assert_eq!(HostCapability::from_interface_name("nope"), None);
    }

    #[test]
    fn contexts_build_deny_by_default() {
        let state = PluginHostState::new(GuestContext {
            plugin_id: "knowme.test".into(),
            capabilities: HashSet::new(),
            dispatch: Arc::new(crate::dispatch::InMemoryDispatch::new()),
            limits: SandboxLimits::default(),
        });
        // The contexts exist and the limiter carries the default ceiling.
        assert_eq!(
            state.limiter.max_bytes(),
            SandboxLimits::default().max_linear_memory_bytes as usize
        );
        // Deny-by-default WasiCtx: no preopened dirs are exposed.
        // (WasiCtx exposes no introspection API; construction with the bare
        // builder is the deny-by-default guarantee — see design D4.)
        let _ = &state.wasi;
        let _ = &state.wasi_http;
        let _ = &state.table;
    }

    #[test]
    fn require_denies_ungranted_and_allows_ungated() {
        let cx = GuestContext {
            plugin_id: "knowme.test".into(),
            capabilities: HashSet::from([HostCapability::Storage]),
            dispatch: Arc::new(crate::dispatch::InMemoryDispatch::new()),
            limits: SandboxLimits::default(),
        };
        assert!(cx.require(HostCapability::Storage).is_ok());
        let err = cx.require(HostCapability::Net).unwrap_err();
        assert!(
            matches!(err, HostError::CapabilityDenied(ref s) if s.contains("host-net") && s.contains("knowme.test")),
            "got: {err:?}"
        );
        // Ungated capabilities pass even with an empty grant set.
        assert!(cx.require(HostCapability::Log).is_ok());
        assert!(cx.require(HostCapability::Time).is_ok());
    }
}
