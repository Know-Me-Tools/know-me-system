//! Capability-gating fixture (task 7.3): a hook-world guest that actually
//! calls `host-net.fetch`, so the `host-net` import survives linker GC.
//!
//! * Without the `net` capability, instantiation must fail with
//!   `MissingCapability("host-net")`.
//! * With it, instantiation succeeds; the guest logs the fetch outcome so
//!   tests can observe the typed `host-error` values it receives
//!   (`ssrf-denied` for the metadata URL, `backend not installed` for a
//!   public URL via the in-memory backend).

wit_bindgen::generate!({
    path: "../../../wit/v1",
    world: "hook",
});

use knowme::plugin::host_log::{self, LogLevel};
use knowme::plugin::host_net::{self, HttpRequest};

struct NetImporterGuest;

impl Guest for NetImporterGuest {
    fn handle_event(topic: String, event: AgUiEvent) -> Result<(), String> {
        let _ = event;
        let url = match topic.as_str() {
            "fetch:metadata" => "http://169.254.169.254/latest/meta-data",
            "fetch:example" => "https://example.com/",
            other => return Err(format!("unknown topic: {other}")),
        };
        let req = HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
        };
        match host_net::fetch(&req) {
            Ok(resp) => host_log::log(
                LogLevel::Info,
                &format!("fetch ok status {}", resp.status),
            ),
            Err(e) => host_log::log(LogLevel::Warn, &format!("fetch denied: {e:?}")),
        }
        Ok(())
    }
}

impl exports::knowme::plugin::lifecycle::Guest for NetImporterGuest {
    fn init() -> Result<(), String> {
        Ok(())
    }

    fn shutdown() {}
}

export!(NetImporterGuest);
