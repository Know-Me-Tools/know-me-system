# KnowMe — know-me-system

**Personal intelligence, on your device.**

Monorepo for the full KnowMe application — a sovereign personal AI platform
shipping on macOS, Windows, Linux, iOS, and Android from one Rust + React 19 +
Flutter codebase. Organized around **FUNC-SPEC v1.0**
(`docs/knowme-functional-spec.html`); seeded from the `knowme-poc` reference
application built on the Prometheus AGS Hybrid Mobile Architecture
(TJ-ARCH-MOB-001).

The runtime authority is the vendored **Universal Agent Runtime (UAR)**
submodule. KnowMe owns the product UI, the Flutter/Tauri shells, the
local-device model provider bridge, persistence projection, and platform
certification evidence. Everything that touches networking, LLM, or
persistence goes through `gen_ui_core` (Rust); the React and Flutter surfaces
are deliberately thin clients of that core.

## Structure

```
know-me-system/
├── docs/                ← Canonical specs, standards, strategies, diagnostics, research, reports
│                          ─ start at docs/README.md (FUNC-SPEC, UX-MOOD, M1–M7, strategies index)
├── rust/                ← Shared layered Rust workspace
│   ├── crates/
│   │   ├── gen_ui_*              ← working core: types, runtime, protocol, mcp, db(+graph/SurrealDB),
│   │   │                            inference, audio, agent (UAR), document, context, host, ffi,
│   │   │                            tauri plugin, wasm, axum server
│   │   ├── knowme-web-server     ← embeddable web host of the React bundle (axum + rust-embed)
│   │   ├── knowme_hands          ← Hands system skeleton (FUNC-SPEC §06)
│   │   ├── knowme_ofp            ← OFP peer-sync skeleton (FUNC-SPEC §07)
│   │   └── knowme_plugin_host    ← WASM plugin host skeleton + knowme:plugin@0.1.0 WIT (FUNC-SPEC §08)
│   └── vendor/
│       ├── librefang/                  ← submodule: BossFang Agent OS (sidecar base — hands, channels,
│       │                                  skills, uar-spec, ACP, extensions, memory-wiki)
│       ├── universal-agent-runtime/    ← submodule: UAR — the runtime authority (AG-UI/A2UI,
│       │                                  skills, provider routing, tool governance, embedded
│       │                                  kernel for mobile); see docs/strategies/uar-*
│       └── llama-cpp-2/                ← vendored patch (iOS httplib symbol fix; see PATCH.md)
├── desktop/             ← Tauri 2 · React 19 · Zustand 5 · TanStack · Tailwind 4
│   └── src/features/    ← home · startup · chat · hands · entities · memory · models · settings
│                           + blueprints: ask-image · audio-scribe · prompt-lab · skills
│                           (chat owns LaneSwitcher · ChatPolicyToolbar · ChatWelcome ·
│                            A2uiArtifactRenderer · RichDataRenderers; A2UI/AG-UI surfaces
│                            rendered through Zustand stores, which are the only invoke() layer)
├── mobile/              ← Flutter (Riverpod 3.3, flutter_rust_bridge 2.12)
│   └── lib/features/    ← startup · chat · memory · notes · overview · settings
│                           + blueprints: hands · ask_image · audio_scribe · prompt_lab ·
│                                       skills · models
│                           (chat uses data/domain/presentation split; embedded UAR kernel
│                            is the runtime authority, KnowMe's local model is one UAR
│                            provider behind it)
├── packages/            ← npm: @prometheus-ags/gen-ui-react, gen-ui-wasm
├── flutter_packages/    ← pub: gen_ui_flutter (FFI plugin), gen_ui_widgets, prometheus_entity_management
├── plugins/             ← Plugin contract docs + examples/hello-knowme (wasm32-wasip2)
├── providers/
│   └── catalog.json     ← LLM provider registry (models.dev schema + provenance SHA-256;
│                          revisioned 2026-07-19, library_version 1.9.3)
├── openspec/            ← Change-proposal workflow (proposal / design / tasks / specs /
│                          verification-notes per change; archive/ holds completed)
├── .kbd-orchestrator/   ← KBD phase tracker — current waypoint, constraints, position
├── infra/ · deploy/     ← docker-compose, k8s, flint-gate, keto, postgres
├── scripts/             ← ios/build-xcframework.sh, android/build.sh,
│                          android/certify-local-*.sh, android/verify-*-gates.sh,
│                          android/write-local-certification-report.sh, generate-mobile-app-icons.sh
└── versions.toml        ← single source of truth for tool/dependency versions and engine lanes
```

Directory names `rust/`, `mobile/`, `desktop/`, `packages/`, `flutter_packages/`
intentionally match the POC layout so every relative path (FRB config, Tauri path
deps, pubspec path deps) remains valid.

## Quick start

```bash
# One-time submodule init (UAR + librefang are required)
git submodule update --init --recursive

# Rust workspace (includes the knowme_* skeleton crates and the UAR link)
(cd rust/vendor/universal-agent-runtime && cargo check)
(cd rust && cargo check)

# Desktop (Tauri 2 + React 19)
cd desktop && pnpm install && pnpm tauri dev

# Mobile (Flutter) — build the shared core first
bash scripts/ios/build-xcframework.sh release   # or scripts/android/build.sh
(cd rust && flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml)
cd mobile && flutter run
```

The `versions.toml` `[inference]` table picks the engine per lane; see "Local
inference engines" below for the current matrix and the `docs/strategies/local-models`
walkthrough of first-run onboarding.

## Where the spec lives vs. where the code lives

The eight tiles, the Hands system, OFP/BossFang sync, the plugin system, the
local model lane, and the privacy guarantees each map to concrete modules —
the full table and the M1–M7 build order are in
**`docs/implementation-roadmap.md`**. Functional spec → module mapping is the
authoritative bridge. The `openspec/changes/` directory holds the per-change
proposal/design/tasks/verification-notes that move the roadmap forward; the
`.kbd-orchestrator/current-waypoint.md` records the current KBD phase.

Working POC subsystems were migrated as-is; every not-yet-built spec
subsystem has either a compiling skeleton crate (with the domain model already
typed) or a blueprint README in its feature directory. None of them disturb
existing builds.

## Local inference engines (per-lane)

Pinned in `versions.toml` `[inference]`. Rust owns catalog, multi-file
download/verify, memory preflight, and event normalization on every lane; only
the native generation binding differs.

| Lane | Engine | Notes |
|---|---|---|
| macOS | llama-cpp-2 (default) or MlxcEngine via `mlex` (mlx-c, in-process) | MLX is the Apple-platform lane; `local-mlxc` feature, macOS-only |
| Windows / Linux | llama-cpp-2 | Pinned/checksummed Qwen GGUF; zero-config reference default |
| iOS | **MLX-Swift** (`local-mlx` feature) | Pinned Qwen3-4B/8B MLX; iOS deployment target raised to 16.4 (MLX-Swift floor); Swift bridge (`mobile/ios/Runner/KnowMeMlxBridge.swift`) is the only iOS engine, llama-cpp-2 is off the iOS build |
| Android | LiteRT-LM (Gemma 4 E2B `.litertlm`) | The universal mobile recovery baseline; llama.cpp/llama-metal is off the Android build |
| Web | WebLLM | MLC WebGPU; zero-key browser default |
| Desktop (experimental) | mistral.rs | Behind the `local-mistral` feature; mistral.rs/candle are desktop-only |

## Runtime authority (UAR)

The vendored `rust/vendor/universal-agent-runtime` (UAR) owns agent execution,
skill definitions and storage, A2UI component catalog and execution loop,
tool governance and approvals, and general LLM/provider routing. KnowMe's
product surfaces are thin clients that project UAR's typed event stream into
the `ContentBlock` view model (`SkillBlock`, `ToolUseBlock`, `ToolResultBlock`,
`ThinkingBlock`, `CitationBlock`, `MemoryBlock`, `ArtifactBlock`, etc.).

- **Strategy:** `docs/strategies/uar-knowme-runtime-integration.md` — the
  authority, deployment modes, provider bridge, and phased implementation
- **Future roadmap:** `docs/strategies/uar-future-release-roadmap.md` — DEMO-0,
  UAR-1, UAR-2 (mobile embedded kernel), UAR-3 (real skills + A2UI), UAR-4
  (web/desktop convergence), UAR-5 (cloud fallback)
- **Status:** desktop service facade and UAR chat lane are wired through
  `gen_ui_agent::uar::UarServiceRuntime`; the React `LaneSwitcher` exposes the
  `uar` lane (guarded until the service is runnable); the mobile embedded
  kernel is in UAR-2 and is **compile-verified end-to-end** but not yet
  physical-device certified (see `docs/reports/embedded-uar-flutter-assessment.md`)

## Standards

- Architecture: `docs/tj-arch-mob-001.html` (TJ-ARCH-MOB-001)
- Functional spec: `docs/knowme-functional-spec.html` (FUNC-SPEC v1.0); also
  pinned on IPFS: `QmXFPBMvNw6jgbTmtt6TZszouJWxERbnDF7XpDfkkTUcFf`
- UI/UX: `docs/knowme-ui-ux-standard.md` + `docs/knowme-moodboard-user-journeys.html`
- Platform support: `docs/platform-support.md` (current baseline + physical-device log)
- Agent rules for working in this repo: `CLAUDE.md`, `AGENTS.md`, `AGENT_BASE_RULES.md`
- Strategies index: `docs/strategies/` (bossfang, agents, plugins, skills,
  local-models, model-configuration, a2ui, ag-ui, conversation,
  knowledge-base-karpathy-wikis, business-model, uar-*)
- Versions: `versions.toml` — the **only** place tool/stack versions and engine
  pins live. Authority docs (`CLAUDE.md`, `AGENTS.md`, this README, the
  strategies) state these same values; the `hybrid-mobile-architecture` skill's
  `audit.sh` doc-consistency gate fails on drift.
  Current pin set: Rust 1.97.1 · Tauri CLI 2.11.4 · React 19.2.7 ·
  Flutter 3.47.0-0.1.pre · TypeScript 7.0.2 · Bun 1.3.14 · pnpm 11.15.0 ·
  SurrealDB 3.2.1 · LiteRT-LM Android 0.14.0 · FRB 2.13.0-beta.5

---
KnowMe, LLC · know-me.tools
