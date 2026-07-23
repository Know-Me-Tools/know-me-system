//! The typed `HostDispatch` seam (design D2) and the in-memory reference
//! backend (design D7).
//!
//! Every host-trait implementation in `sandbox::bindings` performs its
//! capability check at the sandbox boundary and then delegates the side
//! effect to one of these methods — one method per WIT host-interface
//! function, each taking the caller's [`GuestContext`]. Capability denials
//! therefore happen at a single auditable choke point and never reach the
//! backend (the counting test double in `sandbox::bindings::tests` proves
//! this).
//!
//! Production backends (real KV store, SSRF-guarded HTTP client, Cedar-gated
//! `uar-invoke`, AG-UI fabric, inference, memory, A2UI surface push) arrive
//! with their owning changes and plug into this same trait without touching
//! the sandbox.

use async_trait::async_trait;

use crate::sandbox::error::HostError;
use crate::sandbox::state::GuestContext;
use crate::types::{
    A2uiMessage, AgUiEvent, HttpRequest, HttpResponse, InferChunk, InferRequest, InferResponse,
    LogLevel, MemoryHit, RecallQuery,
};

/// One method per WIT host-interface function (design D2). Object-safe so
/// backends can be installed as `Arc<dyn HostDispatch>` per guest.
///
/// Method ↔ WIT mapping:
/// `kv_*` → `host-kv`, `log` → `host-log`, `settings_*` → `host-settings`,
/// `net_fetch` → `host-net.fetch`, `uar_invoke` → `host-uar.invoke`,
/// `events_*` → `host-events`, `infer*` → `host-infer`,
/// `memory_recall` → `host-memory.recall`, `a2ui_push` →
/// `host-a2ui.push-surface`, `fs_*` → `fs`, `env_read` → `env.read`,
/// `time_now` → `time.now`.
#[async_trait]
pub trait HostDispatch: Send + Sync {
    // host-kv
    async fn kv_get(
        &self,
        cx: &GuestContext,
        namespace: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, HostError>;
    async fn kv_set(
        &self,
        cx: &GuestContext,
        namespace: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), HostError>;
    async fn kv_delete(&self, cx: &GuestContext, namespace: &str, key: &str)
    -> Result<(), HostError>;

    // host-log — infallible per the WIT signature.
    async fn log(&self, cx: &GuestContext, level: LogLevel, message: &str);

    // host-settings
    async fn settings_get(&self, cx: &GuestContext, key: &str)
    -> Result<Option<String>, HostError>;
    async fn settings_set_own(
        &self,
        cx: &GuestContext,
        key: &str,
        value: &str,
    ) -> Result<(), HostError>;

    // host-net
    async fn net_fetch(&self, cx: &GuestContext, req: HttpRequest)
    -> Result<HttpResponse, HostError>;

    // host-uar
    async fn uar_invoke(
        &self,
        cx: &GuestContext,
        skill: &str,
        payload_json: &str,
    ) -> Result<String, HostError>;

    // host-events
    async fn events_emit(
        &self,
        cx: &GuestContext,
        topic: &str,
        event: AgUiEvent,
    ) -> Result<(), HostError>;
    async fn events_subscribe(&self, cx: &GuestContext, topic: &str)
    -> Result<String, HostError>;
    async fn events_unsubscribe(&self, cx: &GuestContext, subscription: &str)
    -> Result<(), HostError>;

    // host-infer
    async fn infer(&self, cx: &GuestContext, req: InferRequest)
    -> Result<InferResponse, HostError>;
    async fn infer_stream(
        &self,
        cx: &GuestContext,
        req: InferRequest,
    ) -> Result<Vec<InferChunk>, HostError>;

    // host-memory
    async fn memory_recall(
        &self,
        cx: &GuestContext,
        query: RecallQuery,
    ) -> Result<Vec<MemoryHit>, HostError>;

    // host-a2ui
    async fn a2ui_push(&self, cx: &GuestContext, message: A2uiMessage) -> Result<(), HostError>;

    // fs
    async fn fs_read(&self, cx: &GuestContext, path: &str) -> Result<Vec<u8>, HostError>;
    async fn fs_write(&self, cx: &GuestContext, path: &str, body: &[u8])
    -> Result<(), HostError>;
    async fn fs_list(&self, cx: &GuestContext, path: &str) -> Result<Vec<String>, HostError>;

    // env
    async fn env_read(&self, cx: &GuestContext, name: &str) -> Result<Option<String>, HostError>;

    // time — infallible per the WIT signature.
    async fn time_now(&self, cx: &GuestContext) -> u64;
}

/// Error returned for interfaces whose production backend is not installed
/// in this build. The detail names the missing backend per the spec
/// ("capability-denied whose detail names the missing backend").
#[cfg(any(test, feature = "in-memory-dispatch"))]
pub fn backend_not_installed(what: &str) -> HostError {
    HostError::CapabilityDenied(format!(
        "backend not installed: {what} (only the in-memory reference backend is available)"
    ))
}

// ---------------------------------------------------------------------------
// InMemoryDispatch — the reference backend (design D7).
// ---------------------------------------------------------------------------

/// In-memory reference backend, available in tests and under the
/// `in-memory-dispatch` feature. Real behavior for KV (plugin-namespaced
/// map), log (capture sink), settings (map), env (empty), time (wall clock),
/// and the event bus (capture sink + stub subscription ids). `net_fetch` is
/// guarded by the SSRF predicate but never performs I/O — passing targets
/// get [`backend_not_installed`]. `uar_invoke`, `infer`, `memory_recall`,
/// `a2ui_push`, and the `fs` surface deny with [`backend_not_installed`].
#[cfg(any(test, feature = "in-memory-dispatch"))]
#[derive(Debug, Default)]
pub struct InMemoryDispatch {
    /// (plugin_id, namespace, key) → value. Own-namespace rule is enforced
    /// against `cx.plugin_id` on every call.
    kv: std::sync::Mutex<std::collections::BTreeMap<(String, String, String), Vec<u8>>>,
    /// Captured log lines: (plugin_id, level, message).
    logs: std::sync::Mutex<Vec<(String, LogLevel, String)>>,
    /// (plugin_id, key) → value.
    settings: std::sync::Mutex<std::collections::BTreeMap<(String, String), String>>,
    /// Captured emitted events: (topic, event).
    events: std::sync::Mutex<Vec<(String, AgUiEvent)>>,
    /// Monotonic subscription id counter for the stub event bus.
    next_subscription: std::sync::Mutex<u64>,
}

#[cfg(any(test, feature = "in-memory-dispatch"))]
impl InMemoryDispatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test/inspection accessors for the capture sinks.
    pub fn captured_logs(&self) -> Vec<(String, LogLevel, String)> {
        self.logs.lock().expect("logs poisoned").clone()
    }

    pub fn captured_events(&self) -> Vec<(String, AgUiEvent)> {
        self.events.lock().expect("events poisoned").clone()
    }

    pub fn kv_snapshot(&self) -> std::collections::BTreeMap<(String, String, String), Vec<u8>> {
        self.kv.lock().expect("kv poisoned").clone()
    }

    /// The own-namespace rule from `wit/v1/capabilities.wit` host-kv: naming
    /// a namespace other than the calling plugin's own is `policy-denied`.
    fn check_own_namespace(cx: &GuestContext, namespace: &str) -> Result<(), HostError> {
        if namespace == cx.plugin_id {
            Ok(())
        } else {
            Err(HostError::PolicyDenied(format!(
                "namespace `{namespace}` is not the calling plugin's own (`{}`)",
                cx.plugin_id
            )))
        }
    }
}

#[cfg(any(test, feature = "in-memory-dispatch"))]
#[async_trait]
impl HostDispatch for InMemoryDispatch {
    async fn kv_get(
        &self,
        cx: &GuestContext,
        namespace: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, HostError> {
        Self::check_own_namespace(cx, namespace)?;
        Ok(self
            .kv
            .lock()
            .expect("kv poisoned")
            .get(&(cx.plugin_id.clone(), namespace.to_string(), key.to_string()))
            .cloned())
    }

    async fn kv_set(
        &self,
        cx: &GuestContext,
        namespace: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), HostError> {
        Self::check_own_namespace(cx, namespace)?;
        self.kv.lock().expect("kv poisoned").insert(
            (cx.plugin_id.clone(), namespace.to_string(), key.to_string()),
            value.to_vec(),
        );
        Ok(())
    }

    async fn kv_delete(
        &self,
        cx: &GuestContext,
        namespace: &str,
        key: &str,
    ) -> Result<(), HostError> {
        Self::check_own_namespace(cx, namespace)?;
        self.kv.lock().expect("kv poisoned").remove(&(
            cx.plugin_id.clone(),
            namespace.to_string(),
            key.to_string(),
        ));
        Ok(())
    }

    async fn log(&self, cx: &GuestContext, level: LogLevel, message: &str) {
        self.logs
            .lock()
            .expect("logs poisoned")
            .push((cx.plugin_id.clone(), level, message.to_string()));
    }

    async fn settings_get(
        &self,
        cx: &GuestContext,
        key: &str,
    ) -> Result<Option<String>, HostError> {
        Ok(self
            .settings
            .lock()
            .expect("settings poisoned")
            .get(&(cx.plugin_id.clone(), key.to_string()))
            .cloned())
    }

    async fn settings_set_own(
        &self,
        cx: &GuestContext,
        key: &str,
        value: &str,
    ) -> Result<(), HostError> {
        self.settings
            .lock()
            .expect("settings poisoned")
            .insert((cx.plugin_id.clone(), key.to_string()), value.to_string());
        Ok(())
    }

    async fn net_fetch(
        &self,
        _cx: &GuestContext,
        req: HttpRequest,
    ) -> Result<HttpResponse, HostError> {
        if ssrf_blocked(&req.url) {
            return Err(HostError::SsrfDenied(format!(
                "url `{}` resolves to a blocked target (loopback, link-local, RFC1918, or metadata service)",
                req.url
            )));
        }
        Err(backend_not_installed("host-net HTTP client"))
    }

    async fn uar_invoke(
        &self,
        _cx: &GuestContext,
        _skill: &str,
        _payload_json: &str,
    ) -> Result<String, HostError> {
        Err(backend_not_installed("host-uar (Cedar-gated skill invocation)"))
    }

    async fn events_emit(
        &self,
        _cx: &GuestContext,
        topic: &str,
        event: AgUiEvent,
    ) -> Result<(), HostError> {
        self.events
            .lock()
            .expect("events poisoned")
            .push((topic.to_string(), event));
        Ok(())
    }

    async fn events_subscribe(
        &self,
        _cx: &GuestContext,
        topic: &str,
    ) -> Result<String, HostError> {
        let mut n = self.next_subscription.lock().expect("subs poisoned");
        let id = *n;
        *n += 1;
        Ok(format!("inmem-sub-{id}:{topic}"))
    }

    async fn events_unsubscribe(
        &self,
        _cx: &GuestContext,
        _subscription: &str,
    ) -> Result<(), HostError> {
        Ok(())
    }

    async fn infer(
        &self,
        _cx: &GuestContext,
        _req: InferRequest,
    ) -> Result<InferResponse, HostError> {
        Err(backend_not_installed("host-infer"))
    }

    async fn infer_stream(
        &self,
        _cx: &GuestContext,
        _req: InferRequest,
    ) -> Result<Vec<InferChunk>, HostError> {
        Err(backend_not_installed("host-infer streaming"))
    }

    async fn memory_recall(
        &self,
        _cx: &GuestContext,
        _query: RecallQuery,
    ) -> Result<Vec<MemoryHit>, HostError> {
        Err(backend_not_installed("host-memory"))
    }

    async fn a2ui_push(&self, _cx: &GuestContext, _message: A2uiMessage) -> Result<(), HostError> {
        Err(backend_not_installed("host-a2ui surface push"))
    }

    async fn fs_read(&self, _cx: &GuestContext, _path: &str) -> Result<Vec<u8>, HostError> {
        Err(backend_not_installed("fs (sandboxed filesystem)"))
    }

    async fn fs_write(&self, _cx: &GuestContext, _path: &str, _body: &[u8]) -> Result<(), HostError> {
        Err(backend_not_installed("fs (sandboxed filesystem)"))
    }

    async fn fs_list(&self, _cx: &GuestContext, _path: &str) -> Result<Vec<String>, HostError> {
        Err(backend_not_installed("fs (sandboxed filesystem)"))
    }

    async fn env_read(&self, _cx: &GuestContext, _name: &str) -> Result<Option<String>, HostError> {
        // Deny-by-default environment: nothing is granted, so reads return
        // `none` rather than an error (per the WIT contract for unset vars).
        Ok(None)
    }

    async fn time_now(&self, _cx: &GuestContext) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// SSRF predicate (spec: loopback, link-local, RFC1918, cloud metadata).
// ---------------------------------------------------------------------------

/// Returns true when `url`'s host is a blocked SSRF target: loopback,
/// link-local (which covers the 169.254.169.254 cloud metadata service),
/// RFC1918 private, unspecified, or the `localhost` name. Hostnames that are
/// not literal IPs are not resolved here — the in-memory backend performs no
/// DNS or I/O, and production backends resolve-and-pin with the same
/// predicate applied to every resolved address.
pub fn ssrf_blocked(url: &str) -> bool {
    let Some(host) = extract_host(url) else {
        // Unparseable URLs are not our call to block here; the HTTP backend
        // rejects them as invalid-argument downstream.
        return false;
    };
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    // Strip IPv6 brackets before parsing.
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    match bare.parse::<std::net::IpAddr>() {
        Ok(ip) => is_blocked_ip(&ip),
        Err(_) => false,
    }
}

/// Extract the host portion of `scheme://host[:port][/path…]`.
fn extract_host(url: &str) -> Option<&str> {
    let after_scheme = url.split_once("://")?.1;
    let authority = after_scheme.split(['/', '?', '#']).next()?;
    // Strip userinfo if present.
    let authority = authority.rsplit('@').next()?;
    // Split host:port (IPv6 literals keep their brackets).
    if authority.starts_with('[') {
        let end = authority.find(']')?;
        Some(&authority[..=end])
    } else {
        Some(authority.split(':').next()?)
    }
}

fn is_blocked_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback()              // 127.0.0.0/8
                || v4.is_private()        // 10/8, 172.16/12, 192.168/16 (RFC1918)
                || v4.is_link_local()     // 169.254.0.0/16 — includes 169.254.169.254 metadata
                || v4.is_unspecified()    // 0.0.0.0
                || v4.is_broadcast()
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()              // ::1
                || v6.is_unspecified()    // ::
                || v6.is_unique_local()   // fc00::/7
                || v6.is_unicast_link_local() // fe80::/10
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::state::{GuestContext, HostCapability};
    use crate::SandboxLimits;
    use std::collections::HashSet;
    use std::sync::Arc;

    fn cx() -> GuestContext {
        GuestContext {
            plugin_id: "knowme.test".to_string(),
            capabilities: HashSet::<HostCapability>::new(),
            dispatch: Arc::new(InMemoryDispatch::new()),
            limits: SandboxLimits::default(),
        }
    }

    #[test]
    fn ssrf_predicate_blocks_spec_targets() {
        for url in [
            "http://169.254.169.254/",
            "http://169.254.169.254/latest/meta-data",
            "http://127.0.0.1/",
            "http://127.0.0.1:8080/admin",
            "http://localhost/",
            "http://LOCALHOST:3000/",
            "http://10.0.0.5/internal",
            "http://172.16.0.1/",
            "http://172.31.255.255/",
            "http://192.168.1.1/",
            "http://0.0.0.0/",
            "http://[::1]/",
            "http://[fe80::1]/",
            "http://[fc00::1]/",
        ] {
            assert!(ssrf_blocked(url), "expected blocked: {url}");
        }
        for url in [
            "https://example.com/",
            "https://example.com:8443/path?q=1",
            "https://user@example.com/",
            "http://8.8.8.8/",
            "http://172.15.0.1/",
            "http://172.32.0.1/",
            "http://11.0.0.1/",
        ] {
            assert!(!ssrf_blocked(url), "expected allowed: {url}");
        }
    }

    #[tokio::test]
    async fn net_fetch_denies_ssrf_targets_with_ssrf_denied() {
        let d = InMemoryDispatch::new();
        let req = HttpRequest {
            method: "GET".into(),
            url: "http://169.254.169.254/latest/meta-data".into(),
            headers: vec![],
            body: None,
        };
        let err = d.net_fetch(&cx(), req).await.unwrap_err();
        assert!(
            matches!(err, HostError::SsrfDenied(ref s) if s.contains("169.254.169.254")),
            "got: {err:?}"
        );
    }

    #[tokio::test]
    async fn net_fetch_denies_loopback_with_ssrf_denied() {
        let d = InMemoryDispatch::new();
        let req = HttpRequest {
            method: "GET".into(),
            url: "http://127.0.0.1/".into(),
            headers: vec![],
            body: None,
        };
        let err = d.net_fetch(&cx(), req).await.unwrap_err();
        assert!(matches!(err, HostError::SsrfDenied(_)), "got: {err:?}");
    }

    #[tokio::test]
    async fn net_fetch_to_public_url_denies_with_backend_not_installed() {
        let d = InMemoryDispatch::new();
        let req = HttpRequest {
            method: "GET".into(),
            url: "https://example.com/".into(),
            headers: vec![],
            body: None,
        };
        let err = d.net_fetch(&cx(), req).await.unwrap_err();
        assert!(
            matches!(err, HostError::CapabilityDenied(ref s) if s.contains("backend not installed")),
            "got: {err:?}"
        );
    }

    #[tokio::test]
    async fn uninstalled_backends_deny_cleanly_naming_the_backend() {
        let d = InMemoryDispatch::new();
        let cx = cx();

        let err = d
            .uar_invoke(&cx, "web-fetch", "{}")
            .await
            .unwrap_err();
        assert!(
            matches!(err, HostError::CapabilityDenied(ref s) if s.contains("backend not installed") && s.contains("host-uar")),
            "got: {err:?}"
        );

        let req = InferRequest {
            modality: crate::types::Modality::Text,
            model_hint: None,
            prompt: "hi".into(),
            params_json: None,
        };
        let err = d.infer(&cx, req.clone()).await.unwrap_err();
        assert!(matches!(err, HostError::CapabilityDenied(ref s) if s.contains("host-infer")));
        let err = d.infer_stream(&cx, req).await.unwrap_err();
        assert!(matches!(err, HostError::CapabilityDenied(ref s) if s.contains("host-infer")));

        let err = d
            .memory_recall(
                &cx,
                RecallQuery {
                    text: "x".into(),
                    limit: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, HostError::CapabilityDenied(ref s) if s.contains("host-memory")));

        let err = d
            .a2ui_push(&cx, A2uiMessage::DeleteSurface("s1".into()))
            .await
            .unwrap_err();
        assert!(matches!(err, HostError::CapabilityDenied(ref s) if s.contains("host-a2ui")));

        let err = d.fs_read(&cx, "/etc/passwd").await.unwrap_err();
        assert!(matches!(err, HostError::CapabilityDenied(ref s) if s.contains("backend not installed")));
    }

    #[tokio::test]
    async fn kv_roundtrip_is_namespaced_per_plugin() {
        let d = InMemoryDispatch::new();
        let cx = cx();
        d.kv_set(&cx, "knowme.test", "k", b"v")
            .await
            .expect("set");
        assert_eq!(
            d.kv_get(&cx, "knowme.test", "k").await.expect("get"),
            Some(b"v".to_vec())
        );
        d.kv_delete(&cx, "knowme.test", "k").await.expect("delete");
        assert_eq!(d.kv_get(&cx, "knowme.test", "k").await.expect("get"), None);
    }

    #[tokio::test]
    async fn kv_foreign_namespace_is_policy_denied() {
        let d = InMemoryDispatch::new();
        let cx = cx();
        let err = d
            .kv_get(&cx, "some.other.plugin", "k")
            .await
            .unwrap_err();
        assert!(
            matches!(err, HostError::PolicyDenied(ref s) if s.contains("some.other.plugin")),
            "got: {err:?}"
        );
    }

    #[tokio::test]
    async fn log_and_event_sinks_capture() {
        let d = InMemoryDispatch::new();
        let cx = cx();
        d.log(&cx, LogLevel::Info, "hello").await;
        d.events_emit(&cx, "topic", AgUiEvent::RunFinished("r1".into()))
            .await
            .expect("emit");
        assert_eq!(
            d.captured_logs(),
            vec![("knowme.test".to_string(), LogLevel::Info, "hello".to_string())]
        );
        assert_eq!(d.captured_events().len(), 1);
    }

    #[tokio::test]
    async fn env_is_empty_and_time_is_real() {
        let d = InMemoryDispatch::new();
        let cx = cx();
        assert_eq!(d.env_read(&cx, "HOME").await.expect("env"), None);
        let now = d.time_now(&cx).await;
        assert!(now > 1_700_000_000, "wall clock should be sane, got {now}");
    }

    #[tokio::test]
    async fn settings_roundtrip() {
        let d = InMemoryDispatch::new();
        let cx = cx();
        assert_eq!(d.settings_get(&cx, "theme").await.expect("get"), None);
        d.settings_set_own(&cx, "theme", "dark").await.expect("set");
        assert_eq!(
            d.settings_get(&cx, "theme").await.expect("get"),
            Some("dark".to_string())
        );
    }
}
