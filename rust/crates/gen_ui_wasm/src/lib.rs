// TJ-ARCH-MOB-001 compliant
//! gen_ui_wasm (LEAF) — wasm-bindgen/web surface for browser embedding. The web
//! app (@prometheus-ags/gen-ui-wasm) drives the SAME gen_ui_protocol adapters the
//! native surfaces use, so ContentBlock rendering is identical across platforms.
//!
//! Build: ./build-wasm.sh   (wasm-pack --profile wasm-release + wasm-opt -Oz)
//! Types cross the boundary as JS objects via serde-wasm-bindgen (camelCase,
//! matching the ContentBlock `rename_all` contract).
use gen_ui_protocol::A2uiAdapter;
use gen_ui_types::events::{A2uiEvent, StreamEvent};
use wasm_bindgen::prelude::*;

/// Install a readable panic hook (panics -> console.error with a Rust backtrace).
/// Call once from JS at module init.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Crate version — smoke test that the module loaded.
#[wasm_bindgen]
pub fn gen_ui_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Wraps the shared A2UI adapter so the web app folds a raw StreamEvent feed into
/// A2uiEvents (ContentBlock-bearing) with the same logic as native. This is the
/// point of the shared core: no re-implementing protocol logic in TypeScript.
#[wasm_bindgen]
pub struct WasmA2uiAdapter {
    inner: A2uiAdapter,
}

#[wasm_bindgen]
impl WasmA2uiAdapter {
    #[wasm_bindgen(constructor)]
    pub fn new(run_id: String) -> Self {
        Self {
            inner: A2uiAdapter::new(run_id),
        }
    }

    /// Feed one StreamEvent (as a JS object); get back the A2uiEvents it produced
    /// (as a JS array). Errors surface as thrown JsValue.
    #[wasm_bindgen]
    pub fn ingest(&mut self, event: JsValue) -> std::result::Result<JsValue, JsValue> {
        let ev: StreamEvent =
            serde_wasm_bindgen::from_value(event).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let out: Vec<A2uiEvent> = self.inner.ingest(&ev);
        serde_wasm_bindgen::to_value(&out).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
