//! `bindgen!` invocations for the `knowme:plugin@1.0.0` worlds (design D3),
//! type unification via whole-interface `with:` remaps (design D1), the
//! host-trait implementations (task 4.2), and the capability-subset linker
//! wiring (task 6.2).
//!
//! Unification strategy (the wasmtime-wasi-http pattern): the `hook` world
//! is bindgen'd canonically and OWNS every shared host interface
//! (`knowme::plugin::*` modules). The `contributor` and `app-shell` worlds
//! remap those interfaces onto hook's modules via their `with:` tables, so
//! all three stable worlds share one Rust type per WIT type AND one `Host`
//! trait per interface — error classification, host-trait impls, and linker
//! wiring each live in exactly one place.
//!
//! Host-trait impls are one-line forwards to inherent methods on
//! [`PluginHostState`]; every inherent method runs the boundary capability
//! check before touching the dispatch backend (design D2's single choke
//! point).

use super::error::HostError;
use super::state::{GuestContext, HostCapability, PluginHostState};
use crate::types::{
    A2uiMessage, AgUiEvent, HttpRequest, HttpResponse, InferChunk, InferRequest, InferResponse,
    LogLevel, MemoryHit, RecallQuery,
};

/// Stable world `hook` (task 4.1) — canonical owner of the shared
/// `knowme::plugin::*` host-interface bindings.
pub mod hook {
    wasmtime::component::bindgen!({
        path: "wit/v1",
        world: "hook",
        imports: { default: async | trappable },
        exports: { default: async },
        additional_derives: [PartialEq],
    });
}

/// Stable world `contributor` (task 4.1) — shared interfaces remapped to
/// hook's modules; only `contribution-types` is contributor-local.
pub mod contributor {
    wasmtime::component::bindgen!({
        path: "wit/v1",
        world: "contributor",
        imports: { default: async | trappable },
        exports: { default: async },
        additional_derives: [PartialEq],
        with: {
            "knowme:plugin/host-types": crate::sandbox::bindings::hook::knowme::plugin::host_types,
            "knowme:plugin/host-log": crate::sandbox::bindings::hook::knowme::plugin::host_log,
            "knowme:plugin/host-kv": crate::sandbox::bindings::hook::knowme::plugin::host_kv,
            "knowme:plugin/host-settings": crate::sandbox::bindings::hook::knowme::plugin::host_settings,
            "knowme:plugin/host-net": crate::sandbox::bindings::hook::knowme::plugin::host_net,
            "knowme:plugin/host-uar": crate::sandbox::bindings::hook::knowme::plugin::host_uar,
            "knowme:plugin/host-events": crate::sandbox::bindings::hook::knowme::plugin::host_events,
            "knowme:plugin/host-infer": crate::sandbox::bindings::hook::knowme::plugin::host_infer,
            "knowme:plugin/host-memory": crate::sandbox::bindings::hook::knowme::plugin::host_memory,
            "knowme:plugin/fs": crate::sandbox::bindings::hook::knowme::plugin::fs,
            "knowme:plugin/env": crate::sandbox::bindings::hook::knowme::plugin::env,
            "knowme:plugin/time": crate::sandbox::bindings::hook::knowme::plugin::time,
        },
    });
}

/// Stable world `app-shell` (task 4.1) — shared interfaces remapped to
/// hook's modules; only `host-a2ui` (app-shell-only) is local.
pub mod app_shell {
    wasmtime::component::bindgen!({
        path: "wit/v1",
        world: "app-shell",
        imports: { default: async | trappable },
        exports: { default: async },
        additional_derives: [PartialEq],
        with: {
            "knowme:plugin/host-types": crate::sandbox::bindings::hook::knowme::plugin::host_types,
            "knowme:plugin/host-log": crate::sandbox::bindings::hook::knowme::plugin::host_log,
            "knowme:plugin/host-kv": crate::sandbox::bindings::hook::knowme::plugin::host_kv,
            "knowme:plugin/host-settings": crate::sandbox::bindings::hook::knowme::plugin::host_settings,
            "knowme:plugin/host-net": crate::sandbox::bindings::hook::knowme::plugin::host_net,
            "knowme:plugin/host-uar": crate::sandbox::bindings::hook::knowme::plugin::host_uar,
            "knowme:plugin/host-events": crate::sandbox::bindings::hook::knowme::plugin::host_events,
            "knowme:plugin/host-infer": crate::sandbox::bindings::hook::knowme::plugin::host_infer,
            "knowme:plugin/host-memory": crate::sandbox::bindings::hook::knowme::plugin::host_memory,
            "knowme:plugin/fs": crate::sandbox::bindings::hook::knowme::plugin::fs,
            "knowme:plugin/env": crate::sandbox::bindings::hook::knowme::plugin::env,
            "knowme:plugin/time": crate::sandbox::bindings::hook::knowme::plugin::time,
        },
    });
}

/// Compile-only bindings for the `unstable`-annotated worlds (task 4.3).
/// These prove the WIT keeps compiling for all seven worlds; instantiation
/// is refused by `WorldKind::ensure_stable` with
/// `SandboxError::UnsupportedWorld` until each world's WIT stabilizes.
/// Nothing here is wired into a linker, so each module is self-contained.
#[allow(dead_code)]
mod unstable {
    pub mod agent {
        wasmtime::component::bindgen!({
            path: "wit/v1",
            world: "agent",
            imports: { default: async | trappable },
            exports: { default: async },
        });
    }
    pub mod provider {
        wasmtime::component::bindgen!({
            path: "wit/v1",
            world: "provider",
            imports: { default: async | trappable },
            exports: { default: async },
        });
    }
    pub mod service {
        wasmtime::component::bindgen!({
            path: "wit/v1",
            world: "service",
            imports: { default: async | trappable },
            exports: { default: async },
        });
    }
    pub mod workflow {
        wasmtime::component::bindgen!({
            path: "wit/v1",
            world: "workflow",
            imports: { default: async | trappable },
            exports: { default: async },
        });
    }
}

// ---------------------------------------------------------------------------
// Boundary layer (task 4.2): inherent methods on PluginHostState, one per
// WIT host-interface function. Each performs the capability check at the
// sandbox boundary, then delegates to the typed HostDispatch backend.
// ---------------------------------------------------------------------------

impl PluginHostState {
    pub(crate) async fn boundary_kv_get(
        cx: &GuestContext,
        namespace: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, HostError> {
        cx.require(HostCapability::Storage)?;
        cx.dispatch.kv_get(cx, namespace, key).await
    }

    pub(crate) async fn boundary_kv_set(
        cx: &GuestContext,
        namespace: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), HostError> {
        cx.require(HostCapability::Storage)?;
        cx
            .dispatch
            .kv_set(cx, namespace, key, value)
            .await
    }

    pub(crate) async fn boundary_kv_delete(
        cx: &GuestContext,
        namespace: &str,
        key: &str,
    ) -> Result<(), HostError> {
        cx.require(HostCapability::Storage)?;
        cx.dispatch.kv_delete(cx, namespace, key).await
    }

    pub(crate) async fn boundary_log(cx: &GuestContext, level: LogLevel, message: &str) {
        // Ungated (design: every plugin may log); audited at the sink.
        cx.dispatch.log(cx, level, message).await;
    }

    pub(crate) async fn boundary_settings_get(
        cx: &GuestContext,
        key: &str,
    ) -> Result<Option<String>, HostError> {
        cx.require(HostCapability::Settings)?;
        cx.dispatch.settings_get(cx, key).await
    }

    pub(crate) async fn boundary_settings_set_own(
        cx: &GuestContext,
        key: &str,
        value: &str,
    ) -> Result<(), HostError> {
        cx.require(HostCapability::Settings)?;
        cx
            .dispatch
            .settings_set_own(cx, key, value)
            .await
    }

    pub(crate) async fn boundary_net_fetch(
        cx: &GuestContext,
        req: HttpRequest,
    ) -> Result<HttpResponse, HostError> {
        cx.require(HostCapability::Net)?;
        cx.dispatch.net_fetch(cx, req).await
    }

    pub(crate) async fn boundary_uar_invoke(
        cx: &GuestContext,
        skill: &str,
        payload_json: &str,
    ) -> Result<String, HostError> {
        cx.require(HostCapability::Uar)?;
        cx
            .dispatch
            .uar_invoke(cx, skill, payload_json)
            .await
    }

    pub(crate) async fn boundary_events_emit(
        cx: &GuestContext,
        topic: &str,
        event: AgUiEvent,
    ) -> Result<(), HostError> {
        cx.require(HostCapability::Events)?;
        cx.dispatch.events_emit(cx, topic, event).await
    }

    pub(crate) async fn boundary_events_subscribe(
        cx: &GuestContext,
        topic: &str,
    ) -> Result<String, HostError> {
        cx.require(HostCapability::Events)?;
        cx.dispatch.events_subscribe(cx, topic).await
    }

    pub(crate) async fn boundary_events_unsubscribe(
        cx: &GuestContext,
        subscription: &str,
    ) -> Result<(), HostError> {
        cx.require(HostCapability::Events)?;
        cx
            .dispatch
            .events_unsubscribe(cx, subscription)
            .await
    }

    pub(crate) async fn boundary_infer(
        cx: &GuestContext,
        req: InferRequest,
    ) -> Result<InferResponse, HostError> {
        cx.require(HostCapability::Infer)?;
        cx.dispatch.infer(cx, req).await
    }

    pub(crate) async fn boundary_infer_stream(
        cx: &GuestContext,
        req: InferRequest,
    ) -> Result<Vec<InferChunk>, HostError> {
        cx.require(HostCapability::Infer)?;
        cx.dispatch.infer_stream(cx, req).await
    }

    pub(crate) async fn boundary_memory_recall(
        cx: &GuestContext,
        query: RecallQuery,
    ) -> Result<Vec<MemoryHit>, HostError> {
        cx.require(HostCapability::Memory)?;
        cx.dispatch.memory_recall(cx, query).await
    }

    pub(crate) async fn boundary_a2ui_push(cx: &GuestContext, message: A2uiMessage) -> Result<(), HostError> {
        cx.require(HostCapability::A2ui)?;
        cx.dispatch.a2ui_push(cx, message).await
    }

    pub(crate) async fn boundary_fs_read(cx: &GuestContext, path: &str) -> Result<Vec<u8>, HostError> {
        cx.require(HostCapability::Fs)?;
        cx.dispatch.fs_read(cx, path).await
    }

    pub(crate) async fn boundary_fs_write(cx: &GuestContext, path: &str, body: &[u8]) -> Result<(), HostError> {
        cx.require(HostCapability::Fs)?;
        cx.dispatch.fs_write(cx, path, body).await
    }

    pub(crate) async fn boundary_fs_list(cx: &GuestContext, path: &str) -> Result<Vec<String>, HostError> {
        cx.require(HostCapability::Fs)?;
        cx.dispatch.fs_list(cx, path).await
    }

    pub(crate) async fn boundary_env_read(cx: &GuestContext, name: &str) -> Result<Option<String>, HostError> {
        cx.require(HostCapability::Env)?;
        cx.dispatch.env_read(cx, name).await
    }

    pub(crate) async fn boundary_time_now(cx: &GuestContext) -> u64 {
        // Ungated (the WIT documents the clock as free for every plugin).
        cx.dispatch.time_now(cx).await
    }
}

// ---------------------------------------------------------------------------
// Host-trait impls (task 4.2). Interface remapping unified the traits, so
// ONE impl per interface covers all three stable worlds. Every body is a
// one-line forward to the boundary layer.
// ---------------------------------------------------------------------------

use hook::knowme::plugin as host;

impl host::host_types::Host for PluginHostState {}

impl host::host_log::Host for PluginHostState {
    async fn log(&mut self, level: LogLevel, message: String) -> wasmtime::Result<()> {
        Self::boundary_log(&self.guest, level, &message).await;
        Ok(())
    }
}

impl host::host_kv::Host for PluginHostState {
    async fn get(
        &mut self,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, HostError>> {
        Ok(Self::boundary_kv_get(&self.guest, &namespace, &key).await)
    }

    async fn set(
        &mut self,
        namespace: String,
        key: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), HostError>> {
        Ok(Self::boundary_kv_set(&self.guest, &namespace, &key, &value).await)
    }

    async fn delete(
        &mut self,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Result<(), HostError>> {
        Ok(Self::boundary_kv_delete(&self.guest, &namespace, &key).await)
    }
}

impl host::host_settings::Host for PluginHostState {
    async fn get(&mut self, key: String) -> wasmtime::Result<Result<Option<String>, HostError>> {
        Ok(Self::boundary_settings_get(&self.guest, &key).await)
    }

    async fn set_own(
        &mut self,
        key: String,
        value: String,
    ) -> wasmtime::Result<Result<(), HostError>> {
        Ok(Self::boundary_settings_set_own(&self.guest, &key, &value).await)
    }
}

impl host::host_net::Host for PluginHostState {
    async fn fetch(
        &mut self,
        req: HttpRequest,
    ) -> wasmtime::Result<Result<HttpResponse, HostError>> {
        Ok(Self::boundary_net_fetch(&self.guest, req).await)
    }
}

impl host::host_uar::Host for PluginHostState {
    async fn invoke(
        &mut self,
        skill: String,
        payload_json: String,
    ) -> wasmtime::Result<Result<String, HostError>> {
        Ok(Self::boundary_uar_invoke(&self.guest, &skill, &payload_json).await)
    }
}

impl host::host_events::Host for PluginHostState {
    async fn emit(
        &mut self,
        topic: String,
        event: AgUiEvent,
    ) -> wasmtime::Result<Result<(), HostError>> {
        Ok(Self::boundary_events_emit(&self.guest, &topic, event).await)
    }

    async fn subscribe(&mut self, topic: String) -> wasmtime::Result<Result<String, HostError>> {
        Ok(Self::boundary_events_subscribe(&self.guest, &topic).await)
    }

    async fn unsubscribe(
        &mut self,
        subscription: String,
    ) -> wasmtime::Result<Result<(), HostError>> {
        Ok(Self::boundary_events_unsubscribe(&self.guest, &subscription).await)
    }
}

impl host::host_infer::Host for PluginHostState {
    async fn infer(
        &mut self,
        request: InferRequest,
    ) -> wasmtime::Result<Result<InferResponse, HostError>> {
        Ok(Self::boundary_infer(&self.guest, request).await)
    }

    async fn infer_stream(
        &mut self,
        request: InferRequest,
    ) -> wasmtime::Result<Result<Vec<InferChunk>, HostError>> {
        Ok(Self::boundary_infer_stream(&self.guest, request).await)
    }
}

impl host::host_memory::Host for PluginHostState {
    async fn recall(
        &mut self,
        query: RecallQuery,
    ) -> wasmtime::Result<Result<Vec<MemoryHit>, HostError>> {
        Ok(Self::boundary_memory_recall(&self.guest, query).await)
    }
}

impl host::fs::Host for PluginHostState {
    async fn read(&mut self, path: String) -> wasmtime::Result<Result<Vec<u8>, HostError>> {
        Ok(Self::boundary_fs_read(&self.guest, &path).await)
    }

    async fn write(
        &mut self,
        path: String,
        body: Vec<u8>,
    ) -> wasmtime::Result<Result<(), HostError>> {
        Ok(Self::boundary_fs_write(&self.guest, &path, &body).await)
    }

    async fn list_entries(
        &mut self,
        path: String,
    ) -> wasmtime::Result<Result<Vec<String>, HostError>> {
        Ok(Self::boundary_fs_list(&self.guest, &path).await)
    }
}

impl host::env::Host for PluginHostState {
    async fn read(
        &mut self,
        name: String,
    ) -> wasmtime::Result<Result<Option<String>, HostError>> {
        Ok(Self::boundary_env_read(&self.guest, &name).await)
    }
}

impl host::time::Host for PluginHostState {
    async fn now(&mut self) -> wasmtime::Result<u64> {
        Ok(Self::boundary_time_now(&self.guest).await)
    }
}

/// The `host-a2ui` interface exists only in the `app-shell` world.
impl app_shell::knowme::plugin::host_a2ui::Host for PluginHostState {
    async fn push_surface(
        &mut self,
        message: A2uiMessage,
    ) -> wasmtime::Result<Result<(), HostError>> {
        Ok(Self::boundary_a2ui_push(&self.guest, message).await)
    }
}

// ---------------------------------------------------------------------------
// Capability-subset linker wiring (task 6.2). Only interfaces implied by the
// granted capability set (plus the ungated host-log/time) are wired; a
// component importing anything else is rejected by the pre-instantiation
// import scan with `SandboxError::MissingCapability` before guest code runs.
// The add_to_linker functions are unified too, so one function serves all
// stable worlds.
// ---------------------------------------------------------------------------

/// Wire the shared host interfaces implied by `caps` (plus ungated
/// log/time) into `linker`.
pub(crate) fn wire_shared_interfaces(
    linker: &mut wasmtime::component::Linker<PluginHostState>,
    caps: &std::collections::HashSet<HostCapability>,
) -> Result<(), wasmtime::Error> {
    type Hs = wasmtime::component::HasSelf<PluginHostState>;
    // Ungated: every plugin may log and read the clock.
    host::host_log::add_to_linker::<_, Hs>(linker, |s| s)?;
    host::time::add_to_linker::<_, Hs>(linker, |s| s)?;
    if caps.contains(&HostCapability::Storage) {
        host::host_kv::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Settings) {
        host::host_settings::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Net) {
        host::host_net::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Uar) {
        host::host_uar::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Events) {
        host::host_events::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Infer) {
        host::host_infer::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Memory) {
        host::host_memory::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Fs) {
        host::fs::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    if caps.contains(&HostCapability::Env) {
        host::env::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    Ok(())
}

/// Wire the app-shell-only `host-a2ui` interface when granted.
pub(crate) fn wire_a2ui_interface(
    linker: &mut wasmtime::component::Linker<PluginHostState>,
    caps: &std::collections::HashSet<HostCapability>,
) -> Result<(), wasmtime::Error> {
    if caps.contains(&HostCapability::A2ui) {
        type Hs = wasmtime::component::HasSelf<PluginHostState>;
        app_shell::knowme::plugin::host_a2ui::add_to_linker::<_, Hs>(linker, |s| s)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::{HostDispatch, InMemoryDispatch};
    use crate::sandbox::state::GuestContext;
    use crate::SandboxLimits;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Counting test double (spec scenario "Denial without backend"):
    /// wraps the in-memory backend and counts every dispatch invocation.
    struct CountingDispatch {
        inner: InMemoryDispatch,
        calls: Arc<AtomicUsize>,
    }

    impl CountingDispatch {
        fn new() -> (Self, Arc<AtomicUsize>) {
            let calls = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    inner: InMemoryDispatch::new(),
                    calls: calls.clone(),
                },
                calls,
            )
        }
        fn bump(&self) {
            self.calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[async_trait::async_trait]
    impl HostDispatch for CountingDispatch {
        async fn kv_get(
            &self,
            cx: &GuestContext,
            namespace: &str,
            key: &str,
        ) -> Result<Option<Vec<u8>>, HostError> {
            self.bump();
            self.inner.kv_get(cx, namespace, key).await
        }
        async fn kv_set(
            &self,
            cx: &GuestContext,
            namespace: &str,
            key: &str,
            value: &[u8],
        ) -> Result<(), HostError> {
            self.bump();
            self.inner.kv_set(cx, namespace, key, value).await
        }
        async fn kv_delete(
            &self,
            cx: &GuestContext,
            namespace: &str,
            key: &str,
        ) -> Result<(), HostError> {
            self.bump();
            self.inner.kv_delete(cx, namespace, key).await
        }
        async fn log(&self, cx: &GuestContext, level: LogLevel, message: &str) {
            self.bump();
            self.inner.log(cx, level, message).await
        }
        async fn settings_get(
            &self,
            cx: &GuestContext,
            key: &str,
        ) -> Result<Option<String>, HostError> {
            self.bump();
            self.inner.settings_get(cx, key).await
        }
        async fn settings_set_own(
            &self,
            cx: &GuestContext,
            key: &str,
            value: &str,
        ) -> Result<(), HostError> {
            self.bump();
            self.inner.settings_set_own(cx, key, value).await
        }
        async fn net_fetch(
            &self,
            cx: &GuestContext,
            req: HttpRequest,
        ) -> Result<HttpResponse, HostError> {
            self.bump();
            self.inner.net_fetch(cx, req).await
        }
        async fn uar_invoke(
            &self,
            cx: &GuestContext,
            skill: &str,
            payload_json: &str,
        ) -> Result<String, HostError> {
            self.bump();
            self.inner.uar_invoke(cx, skill, payload_json).await
        }
        async fn events_emit(
            &self,
            cx: &GuestContext,
            topic: &str,
            event: AgUiEvent,
        ) -> Result<(), HostError> {
            self.bump();
            self.inner.events_emit(cx, topic, event).await
        }
        async fn events_subscribe(
            &self,
            cx: &GuestContext,
            topic: &str,
        ) -> Result<String, HostError> {
            self.bump();
            self.inner.events_subscribe(cx, topic).await
        }
        async fn events_unsubscribe(
            &self,
            cx: &GuestContext,
            subscription: &str,
        ) -> Result<(), HostError> {
            self.bump();
            self.inner.events_unsubscribe(cx, subscription).await
        }
        async fn infer(
            &self,
            cx: &GuestContext,
            req: InferRequest,
        ) -> Result<InferResponse, HostError> {
            self.bump();
            self.inner.infer(cx, req).await
        }
        async fn infer_stream(
            &self,
            cx: &GuestContext,
            req: InferRequest,
        ) -> Result<Vec<InferChunk>, HostError> {
            self.bump();
            self.inner.infer_stream(cx, req).await
        }
        async fn memory_recall(
            &self,
            cx: &GuestContext,
            query: RecallQuery,
        ) -> Result<Vec<MemoryHit>, HostError> {
            self.bump();
            self.inner.memory_recall(cx, query).await
        }
        async fn a2ui_push(&self, cx: &GuestContext, message: A2uiMessage) -> Result<(), HostError> {
            self.bump();
            self.inner.a2ui_push(cx, message).await
        }
        async fn fs_read(&self, cx: &GuestContext, path: &str) -> Result<Vec<u8>, HostError> {
            self.bump();
            self.inner.fs_read(cx, path).await
        }
        async fn fs_write(&self, cx: &GuestContext, path: &str, body: &[u8]) -> Result<(), HostError> {
            self.bump();
            self.inner.fs_write(cx, path, body).await
        }
        async fn fs_list(&self, cx: &GuestContext, path: &str) -> Result<Vec<String>, HostError> {
            self.bump();
            self.inner.fs_list(cx, path).await
        }
        async fn env_read(&self, cx: &GuestContext, name: &str) -> Result<Option<String>, HostError> {
            self.bump();
            self.inner.env_read(cx, name).await
        }
        async fn time_now(&self, cx: &GuestContext) -> u64 {
            self.bump();
            self.inner.time_now(cx).await
        }
    }

    fn state_with(caps: &[HostCapability], dispatch: Arc<dyn HostDispatch>) -> PluginHostState {
        PluginHostState::new(GuestContext {
            plugin_id: "knowme.test".into(),
            capabilities: caps.iter().copied().collect::<HashSet<_>>(),
            dispatch,
            limits: SandboxLimits::default(),
        })
    }

    /// Spec scenario: a guest call to `host-kv.get` without the `storage`
    /// capability receives `host-error.capability-denied` and NO dispatch
    /// method is invoked (counting test double).
    #[tokio::test]
    async fn kv_get_denied_at_boundary_without_backend_invocation() {
        let (dispatch, calls) = CountingDispatch::new();
        let mut state = state_with(&[], Arc::new(dispatch));

        let result = <PluginHostState as host::host_kv::Host>::get(
            &mut state,
            "knowme.test".into(),
            "k".into(),
        )
        .await
        .expect("no trap");

        let err = result.expect_err("must be denied");
        assert!(
            matches!(err, HostError::CapabilityDenied(ref s) if s.contains("host-kv")),
            "got: {err:?}"
        );
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "denial must happen before the backend is invoked"
        );
    }

    /// The granted path reaches the backend exactly once.
    #[tokio::test]
    async fn kv_get_with_storage_reaches_backend() {
        let (dispatch, calls) = CountingDispatch::new();
        let mut state = state_with(&[HostCapability::Storage], Arc::new(dispatch));

        let result = <PluginHostState as host::host_kv::Host>::get(
            &mut state,
            "knowme.test".into(),
            "k".into(),
        )
        .await
        .expect("no trap");

        assert!(matches!(result, Ok(None)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    /// Ungated interfaces (host-log) never hit the boundary check, and the
    /// unified trait is shared across worlds (this is the same trait the
    /// contributor and app-shell worlds use).
    #[tokio::test]
    async fn log_is_ungated() {
        let (dispatch, calls) = CountingDispatch::new();
        let mut state = state_with(&[], Arc::new(dispatch));
        <PluginHostState as host::host_log::Host>::log(
            &mut state,
            LogLevel::Warn,
            "ungated".into(),
        )
        .await
        .expect("log");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
