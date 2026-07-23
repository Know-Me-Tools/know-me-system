//! Deny-by-default-HTTP fixture (task 7.3, spec scenario "JS-style guest
//! links without using HTTP"): a hook-world guest whose component ALSO
//! imports `wasi:http/types@0.2.12` — the StarlingMonkey preview2-shim
//! pattern. It instantiates successfully because the host always binds the
//! (deny-by-default) HTTP context; the guest only touches inert `Fields`
//! state and has no `wasi:http/outgoing-handler` import, so no outbound
//! request is even expressible.

mod knowme_world {
    wit_bindgen::generate!({
        path: "../../../wit/v1",
        world: "hook",
    });
}

mod http_world {
    wit_bindgen::generate!({
        path: "wit",
        world: "http-types-guest",
        generate_all,
    });
}

use knowme_world::knowme::plugin::host_events::AgUiEvent;
use knowme_world::knowme::plugin::host_log::{self, LogLevel};

struct HttpTypesGuest;

impl knowme_world::Guest for HttpTypesGuest {
    fn handle_event(topic: String, event: AgUiEvent) -> Result<(), String> {
        let _ = event;
        if topic != "ok" {
            return Err(format!("unknown topic: {topic}"));
        }
        // Touch wasi:http/types for real so the import survives linker GC:
        // construct Fields, append one header, read entries back.
        let fields = http_world::wasi::http::types::Fields::new();
        fields
            .append("x-fixture", b"1")
            .map_err(|_| "fields append failed".to_string())?;
        let entries = fields.entries();
        host_log::log(
            LogLevel::Info,
            &format!("wasi:http Fields roundtrip ok: {} entries", entries.len()),
        );
        Ok(())
    }
}

impl knowme_world::exports::knowme::plugin::lifecycle::Guest for HttpTypesGuest {
    fn init() -> Result<(), String> {
        Ok(())
    }

    fn shutdown() {}
}

knowme_world::export!(HttpTypesGuest with_types_in knowme_world);
