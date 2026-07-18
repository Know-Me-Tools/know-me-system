# KnowMe — know-me-system

**Personal intelligence, on your device.**

Monorepo for the full KnowMe application — a sovereign personal AI platform
shipping on macOS, Windows, Linux, iOS, and Android from one Rust + React 19 +
Flutter codebase. Organized around **FUNC-SPEC v1.0**
(`docs/knowme-functional-spec.html`); seeded from the `knowme-poc` reference
application built on the Prometheus AGS Hybrid Mobile Architecture
(TJ-ARCH-MOB-001).

## Structure

```
know-me-system/
├── docs/               ← Canonical specs (FUNC-SPEC, UX-MOOD), standards, roadmap — start at docs/README.md
├── rust/               ← Shared layered Rust workspace
│   └── crates/
│       ├── gen_ui_*            ← working core: types, runtime, protocol, mcp, db(+graph/SurrealDB),
│       │                          inference, audio (whisper), agent (UAR), ffi, tauri plugin, wasm, servers
│   └── vendor/librefang/       ← submodule: BossFang Agent OS (sidecar base — hands, channels,
│                                  skills, uar-spec, ACP, extensions, memory-wiki; see docs/strategies/bossfang)
│       ├── knowme_hands        ← Hands system skeleton (FUNC-SPEC §06)
│       ├── knowme_ofp          ← OFP peer-sync skeleton (FUNC-SPEC §07)
│       └── knowme_plugin_host  ← WASM plugin host skeleton + knowme:plugin@0.1.0 WIT (FUNC-SPEC §08)
├── desktop/            ← Tauri 2 · React 19 · Zustand 5 · TanStack · Tailwind 4
│   └── src/features/   ← home · chat · hands · entities · memory · models · settings
│                          + blueprints: ask-image · audio-scribe · prompt-lab · skills
├── mobile/             ← Flutter (Riverpod, flutter_rust_bridge 2.12)
│   └── lib/features/   ← chat · memory · notes · overview + blueprints for the remaining tiles
├── packages/           ← npm: @prometheus-ags/gen-ui-react, gen-ui-wasm
├── flutter_packages/   ← pub: gen_ui_flutter (FFI plugin), gen_ui_widgets, prometheus_entity_management
├── plugins/            ← Plugin contract docs + examples/hello-knowme (wasm32-wasip2)
├── infra/ · deploy/    ← docker-compose, k8s, flint-gate, keto, postgres
├── scripts/            ← env check, android/ios core builds
└── versions.toml       ← single source of truth for tool/dependency versions
```

Directory names `rust/`, `mobile/`, `desktop/`, `packages/`,
`flutter_packages/` intentionally match the POC layout so every relative path
(FRB config, Tauri path deps, pubspec path deps) remains valid.

## Quick start

```bash
# Check environment
bash scripts/check-env.sh --install

# Rust workspace (includes the knowme_* skeleton crates)
cd rust && cargo check

# Desktop (Tauri 2 + React 19)
cd desktop && pnpm install && pnpm tauri dev

# Mobile (Flutter) — build the shared core first
bash scripts/ios/build-xcframework.sh release   # or scripts/android/build.sh
(cd rust && flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml)
cd mobile && flutter run
```

## Where the spec lives vs. where the code lives

The eight tiles, the Hands system, OFP/BossFang sync, the plugin system, and
the privacy guarantees each map to concrete modules — the full table and the
M1–M7 build order are in **`docs/implementation-roadmap.md`**. Working POC
subsystems were migrated as-is; every not-yet-built spec subsystem has either
a compiling skeleton crate (with the domain model already typed) or a
blueprint README in its feature directory, none of which disturb existing
builds.

## Standards

- Architecture: `docs/tj-arch-mob-001.html` (TJ-ARCH-MOB-001)
- UI/UX: `docs/knowme-ui-ux-standard.md` + `docs/knowme-moodboard-user-journeys.html`
- Versions: `versions.toml` (Rust 1.96 floor · Tauri 2.10 · React 19 · SurrealDB 3.x · FRB 2.12)
- Agent rules for working in this repo: `CLAUDE.md`, `AGENTS.md`, `AGENT_BASE_RULES.md`

---
KnowMe, LLC · know-me.tools
