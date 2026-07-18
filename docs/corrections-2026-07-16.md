# Corrections & Open Decisions — 2026-07-16

Source analyses: [`assessment-2026-07-16.md`](assessment-2026-07-16.md) (deep assessment, 25 claims
adversarially verified) and [`competitive-analysis-2026-07-16.md`](competitive-analysis-2026-07-16.md)
(landscape across agent frameworks, UI protocols, local-AI products, cross-platform
architectures, scaffold ecosystems). This file records (A) corrections applied to the
repo, and (B) decisions the analyses recommend that are **deliberately NOT applied**,
so in-flight work streams (phase: codegen-and-ci-verification) are not disturbed.

---

## A. Corrections applied

| # | Correction | Where | Status |
|---|---|---|---|
| 1 | pglite-oxide accurately described (PGlite WASI guest in WASM runtime; desktop-only; mobile = SQLite + sqlite-vec) | `CLAUDE.md` §Embedded PostgreSQL | ✅ applied (prior stream, 2026-07-16) |
| 2 | Version skew unified — Rust 1.96+, Node 24+, Flutter beta, Riverpod 3.3, frb 2.12+ — with `versions.toml` as single source of truth and `audit.sh doc-consistency` as the CI gate | `versions.toml`, `SKILL.md`, `CLAUDE.md`, `scripts/audit.sh` | ✅ applied (prior stream, 2026-07-16) |
| 3 | Inference engines made per-lane: mistral.rs desktop (Metal), **llama-cpp-2 mobile**, WebLLM web (post-C-106 stretch), behind `InferenceProvider` | `versions.toml` §[inference] | ✅ applied (prior stream, 2026-07-16) |
| 4 | **A2UI misattribution fixed** — spec previously said "A2UI (Anthropic Agent-to-UI)". Anthropic has no such protocol; A2UI is Google's standard (a2ui.org). Spec now labels ours as an internal protocol distinct from Google's | `docs/gen_ui_spec.html` §06 | ✅ applied (this change) |
| 5 | Protocol-naming note added so agents never present internal A2UI as an external standard in generated code/docs | `CLAUDE.md` §ContentBlock | ✅ applied (this change) |

## B. Open decisions (recommended, NOT applied — do not act without an OpenSpec change)

| # | Decision | Recommendation (evidence) | Suggested timing |
|---|---|---|---|
| 1 | **A2UI: adopt or rename** | Adopt Google A2UI (a2ui.org, v1.0 RC) as the wire format, keeping ContentBlock sealed unions as the internal model — gains the shipped Flutter GenUI SDK + React renderer; else rename internal protocol (e.g. "GenUI Events") | Decide before C-103 finalizes the TS event layer |
| 2 | **MCP client → rmcp** | Replace hand-rolled `gen_ui_mcp` client with the official Rust MCP SDK (`modelcontextprotocol/rust-sdk` rmcp v1.7+, proven in Rig). stdio transport is currently a stub anyway | Candidate for C-108 (MCP change) |
| 3 | **Platform-native inference lanes** | Add Apple Foundation Models (iOS 26+) and ML Kit GenAI / Gemini Nano (premium Android) as first-class `InferenceProvider` lanes; bundled llama.cpp stays the fallback for custom models / mid-range Android. OS-free 3B models beat a 1 GB Qwen download for the default path | Design in C-105/C-109 window |
| 4 | **SurrealDB 3.2 benchmark gate** | On-device graph-RAG latency/memory benchmark before C-104 (surrealdb#6800-class regressions documented); named fallback: sqlite-vec + FTS5 + recursive CTEs | Before C-104 starts |
| 5 | **AG-UI via official SDKs** | CopilotKit maintains Dart and Kotlin AG-UI SDKs — evaluate before hand-writing the Flutter transport layer | Before Flutter-surface AG-UI work |
| 6 | **UniFFI as contingency binding layer** | If C-103 on-device frb validation fails/drags, UniFFI is the production-proven fallback (Mozilla, Bitwarden, Matrix). Record in arch standard as the decision tree, not a switch | Document at C-103 close |
| 7 | **Pack distribution: registry + updates** | Move from one-shot scaffolds toward a shadcn-style versioned registry with a diff/update path; `audit.sh` becomes the diff engine | Post-PoC |
| 8 | **Skill security gate** | Prompt-injection review + signing for pack skills (ToxicSkills: 36% of public skills vulnerable; 141k issues/22.5k skills). Reuse the Ed25519 signing design from the KnowMe plugin spec | Post-PoC; cheap to spec now |
| 9 | **KnowMe premium pricing** | $100–200/mo consumer local-AI band is empty (ceiling comp: Msty $149/yr; Khoj subscription shut down 4/2026; Limitless exited to Meta). Reposition premium tier to enterprise/regulated or cap near verified ceiling until buyer evidence exists | Before any tier machinery is built |
| 10 | **PoC scope trims** | M5 config DB → SQLite + keychain for the demo (pglite-oxide as desktop upgrade later); WebLLM already demoted to stretch in versions.toml | C-103 planning |

## Notes

- gen_ui_spec.html and tj-arch-mob-001.html otherwise remain v1.0 as-published; the
  13-crate workspace (README/PoC plan) supersedes the spec's monolithic `gen_ui_core`
  description — a spec v1.1 refresh is deferred until after the PoC phase.
- Nothing in `apps/`, `openspec/changes/`, `scripts/` (beyond the prior stream's own
  edits), or `references/` was modified by this change.
