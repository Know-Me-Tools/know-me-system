//! Hello KnowMe — minimal guest for the knowme:plugin@0.1.0 contract.
//!
//! Build: `cargo build --release --target wasm32-wasip2`
//! The WIT contract is vendored from
//! `rust/crates/knowme_plugin_host/wit/knowme-plugin.wit` — copy it into
//! `wit/` here (or use a path reference) before generating bindings.

wit_bindgen::generate!({
    world: "knowme-plugin",
    path: "../../../rust/crates/knowme_plugin_host/wit",
});

use exports::knowme::plugin::plugin::Guest;
use knowme::plugin::host;

struct HelloKnowMe;

impl Guest for HelloKnowMe {
    fn init() -> Result<(), String> {
        host::log(host::LogLevel::Info, "hello-knowme initialized");
        Ok(())
    }

    fn handle_event(topic: String, payload_json: String) -> Result<(), String> {
        // Echo every event into scoped storage under its topic. Storage is
        // namespaced per plugin — this can never collide with another plugin.
        host::storage_set(&format!("last:{topic}"), payload_json.as_bytes())?;
        host::log(
            host::LogLevel::Debug,
            &format!("stored payload for topic '{topic}'"),
        );
        Ok(())
    }

    fn shutdown() {
        host::log(host::LogLevel::Info, "hello-knowme shutting down");
    }
}

export!(HelloKnowMe);
