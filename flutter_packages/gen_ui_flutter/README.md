# gen_ui_flutter

Flutter FFI plugin exposing `gen_ui_core` (Rust) via `flutter_rust_bridge` 2.12.

All networking, inference, MCP, persistence and agent logic lives in Rust
(`gen_ui_core`) — this plugin is the thin FFI surface. Never re-implement core
logic in Dart.

Generate bindings from the hybrid project root:

```bash
flutter_rust_bridge_codegen generate --config-file rust/flutter_rust_bridge.yaml
```

Wrap the async intent calls in Riverpod providers, and **opt FFI providers out of
Riverpod 3 automatic retry** (`retry: (_, __) => null`).
