# apps/knowme-poc — Hybrid Mobile + Desktop Application

Built on the [Prometheus AGS Hybrid Mobile Architecture](docs/tj-arch-mob-001.html).

## Structure

```
apps/knowme-poc/
├── rust/               ← Shared layered Rust workspace (gen_ui_core + leaves: ffi, tauri-plugin, wasm)
├── mobile/             ← Flutter iOS/Android application (Riverpod, gen_ui widgets)
├── desktop/            ← Tauri macOS/Windows/Linux application (React 19, Zustand, Prometheus Entity Management 3.x, TanStack Router/Table)
├── packages/           ← Publishable npm packages (@prometheus-ags/gen-ui-react, gen-ui-wasm)
├── flutter_packages/   ← Publishable pub.dev packages (gen_ui_flutter FFI plugin, gen_ui_widgets)
└── docs/               ← Architecture documentation
```

## Quick start

```bash
# Check environment
bash scripts/check-env.sh --install

# Build shared Rust core (Android)
bash scripts/android/build.sh release

# Build shared Rust core (iOS/macOS)
bash scripts/ios/build-xcframework.sh release

# Generate Flutter FFI bindings from the multi-crate contract
(cd rust && flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml)

# Run Flutter
cd mobile && flutter run

# Run Tauri desktop
cd desktop && pnpm tauri dev
```

## Architecture Standard

See [TJ-ARCH-MOB-001](docs/tj-arch-mob-001.html) for platform selection criteria,
state management standards, and enforcement rules.

The reference UI is governed by the repository-level
[KnowMe UI/UX standard](../../docs/knowme-ui-ux-standard.md), including Shadcn UI,
Assistant UI, strict borderless Flat 2.0 styling, durable conversations, rich AG-UI
events, and matching React/Flutter light and dark themes.
