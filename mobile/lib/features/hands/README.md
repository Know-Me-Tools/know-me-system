# feature: hands (Tile 02 · FUNC-SPEC §06)

Flutter counterpart of `desktop/src/features/hands`: active Hands list,
6-template strip, 3-step creation wizard (Identity → Configuration →
Deployment). Riverpod providers subscribe to the FFI bridge
(`lib/bridge`, generated from `rust/crates/gen_ui_ffi`); domain types mirror
`rust/crates/knowme_hands`. Serialize `HandDefinition` across the bridge —
regenerate with flutter_rust_bridge once `gen_ui_ffi` exposes the hands API.
Roadmap M1/M6.
