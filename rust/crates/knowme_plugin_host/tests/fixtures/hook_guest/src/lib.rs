//! Hook-world fixture guest (task 7.1).
//!
//! Topic-switched behaviors so one component covers several host tests:
//! * `ok`    — logs a line and writes a KV entry in its own namespace.
//! * `spin`  — infinite loop (fuel-exhaustion test).
//! * `mem`   — one allocation past the 64 MiB ceiling (memory-limit test).
//! * `sleep` — wall-clock spin past the host timeout (timeout test).

wit_bindgen::generate!({
    path: "../../../wit/v1",
    world: "hook",
});

use knowme::plugin::host_kv;
use knowme::plugin::host_log::{self, LogLevel};
use knowme::plugin::time;

/// Must match the `plugin_id` the host tests instantiate with (own-namespace
/// KV rule).
const NAMESPACE: &str = "knowme.fixture.hook";

struct HookGuest;

impl Guest for HookGuest {
    fn handle_event(topic: String, event: AgUiEvent) -> Result<(), String> {
        let _ = event;
        match topic.as_str() {
            "ok" => {
                host_log::log(LogLevel::Info, "hook fixture received event");
                host_kv::set(NAMESPACE, "last-topic", topic.as_bytes())
                    .map_err(|e| format!("kv set failed: {e:?}"))?;
                Ok(())
            }
            "spin" => loop {
                std::hint::spin_loop();
            },
            "mem" => {
                // One growth past the default 64 MiB ceiling. The host
                // limiter rejects it; `try_reserve` returning Err instead of
                // aborting keeps the guest alive to prove the host maps the
                // denial by limiter signal, not by trap.
                let mut buf: Vec<u8> = Vec::new();
                let _ = buf.try_reserve(128 * 1024 * 1024);
                std::hint::black_box(&buf);
                Ok(())
            }
            "sleep" => {
                // ~6 wall-clock seconds via the (ungated) host clock.
                let start = time::now();
                while time::now().saturating_sub(start) < 6 {
                    std::hint::spin_loop();
                }
                Ok(())
            }
            other => Err(format!("unknown topic: {other}")),
        }
    }
}

impl exports::knowme::plugin::lifecycle::Guest for HookGuest {
    fn init() -> Result<(), String> {
        Ok(())
    }

    fn shutdown() {}
}

export!(HookGuest);
