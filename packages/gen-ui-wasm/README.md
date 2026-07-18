# @prometheus-ags/gen-ui-wasm

The web/WASM surface of `gen_ui_core`. It exposes the shared `gen_ui_protocol`
adapters — so a browser app folds an A2UI event stream into `ContentBlock`s with
the exact same Rust logic the native surfaces use (no re-implementing protocol
logic in TypeScript).

## Build

```bash
npm run build:wasm   # wasm-pack (bundler) + wasm-opt -Oz -> ./pkg
```

## Use (with @prometheus-ags/gen-ui-react)

```ts
import init, { WasmA2uiAdapter } from "@prometheus-ags/gen-ui-wasm";
await init();
const adapter = new WasmA2uiAdapter("run-123");
const a2uiEvents = adapter.ingest({ type: "text_delta", index: 0, delta: "hi" });
```
