# gen_ui_widgets

ContentBlock widget set for the TJ-ARCH-MOB-001 hybrid architecture. `ContentBlockView`
renders one `ContentBlock` — the cross-platform UI contract from the shared Rust
core. The render `switch` is exhaustive over all 11 variants; because `ContentBlock`
is a sealed class, a missing case is a Dart compile error (no default branch).

Presentational only — no FFI calls, no providers. Feed it blocks folded from the
`chatEvents` stream by a `ChatNotifier`.
