<!-- source=primary; branch=main-pre-consolidation; original_sha256=01af3ff068e8865a90dcead2ae3ba6c6e699d1882bbc2839cdb1390903de49d0 -->
---
type: Reference
id: knowme-poc-chat-routing-restored-after-aborted-stale-pr-merge
title: KnowMe PoC chat routing restored after aborted stale PR merge
tags:
- hybrid-mobile
- knowme-poc
- tauri
- chat-routing
- ci-verification
- codegen
- pem
- surrealdb
links:
- hybrid-mobile-poc-phase-goals-and-verification-scope
- t8-resume-status-for-hybrid-mobile-poc-codegen-verification
- knowme-poc-scribe-feature-and-ci-workflow-verification-status
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T12:07:38.528987+00:00
created_at: 2026-07-16T12:07:38.528987+00:00
updated_at: 2026-07-16T12:07:38.528987+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `~/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-16T11:56:54Z`
- **Recorded status:** `executing`

This update continues the KnowMe proof-of-concept effort tracked in [Hybrid Mobile PoC phase goals and verification scope](/hybrid-mobile-poc-phase-goals-and-verification-scope.md), [T8 Resume Status for Hybrid Mobile PoC Codegen Verification](/t8-resume-status-for-hybrid-mobile-poc-codegen-verification.md), and [KnowMe PoC Scribe feature and CI workflow verification status](/knowme-poc-scribe-feature-and-ci-workflow-verification-status.md).

## Phase objective

As revised on `2026-07-15`, the phase deliverable is a working proof-of-concept application, not only pipeline verification. Code generation and CI remain supporting objectives proven through the PoC.

The PoC must be built under:

```text
apps/<name>/
```

It must use repository scaffolds and skills, guided by KnowMe reference documentation in:

```text
docs/reference-app/
```

Required showcase coverage includes:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web from one Rust core
- Feature subset selected via web research on showcase-app best practices and 2026 on-device AI feasibility

## Supporting verification goals

The PoC should prove the original codegen/CI scope in passing:

- Run the real codegen pipeline:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - full `flutter pub get`
  - full `pnpm install`
- Confirm pre-codegen warnings clear after generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:

```text
@prometheus-ags/entity-graph-core@workspace:*
```

- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## Session outcome

An attempted merge of PR #1 was aborted after discovering that `main` already contained an equivalent and more complete implementation.

### Correction to earlier assessment

Initial assessment incorrectly treated PR #1 as more advanced than `main`. During the merge, `main` was found to already include the real chat backend and additional capabilities in commit:

```text
60cb949
```

`main` includes:

- Real `ChatAgent`-equivalent implementation
- Keychain-backed secrets
- Scribe/voice feature
- CI pipeline

PR #1 had forked from an older point and did not include those later changes.

### Merge handling

- The merge was aborted cleanly.
- No data loss occurred.
- Repository state was verified using `git status` and `MERGE_HEAD` before reset.
- PR #1 was closed on GitHub with a comment explaining that `main` already had the more complete implementation.

## Fix applied

The only genuine gap found in the session was that no router route mounted the chat UI.

Restored/wired components:

- `ChatScreen`
- `ChatInput`
- `router.tsx` route wiring

The restored UI wiring type-checks cleanly against `main`'s real `chatStore.ts`.

## Current blocker

A stale `config-db` directory still needs to be cleared before relaunch. This is tied to the earlier runtime error:

```text
model_prefs does not exist
```

## Next steps

1. Clear the stale `config-db` directory.
2. Relaunch:

```bash
tauri:dev
```

3. Confirm the chat UI renders and streams correctly.
4. Move to task 20: moodboard-matching visual design.

# Citations

1. [1] stdin
2. [2] manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification