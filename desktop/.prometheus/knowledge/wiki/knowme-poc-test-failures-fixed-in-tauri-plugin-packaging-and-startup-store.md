---
type: Reference
id: knowme-poc-test-failures-fixed-in-tauri-plugin-packaging-and-startup-store
title: KnowMe PoC Test Failures Fixed in Tauri Plugin Packaging and Startup Store
tags:
- hybrid-mobile
- knowme-poc
- tauri
- vitest
- pnpm-packaging
- startup-store
- ci-verification
links:
- phase-codegen-and-ci-verification-session-status
- knowme-poc-codegen-and-tauri-verification-c-102
- knowme-poc-phase-pr-1-opened-for-t6-t12
- knowme-poc-android-assembledebug-running-on-sm-s936u
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T20:45:08.737784+00:00
created_at: 2026-07-16T20:45:08.737784+00:00
updated_at: 2026-07-16T20:45:08.737784+00:00
revision: 0
---

## Context

- **Project:** Hybrid Mobile Architecture
- **Phase:** `phase-codegen-and-ci-verification`
- **KBD root:** `$REPO_ROOT`
- **Captured:** `2026-07-16T20:43:57Z`
- **Phase source record:** `manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification`

This session continues the revised KnowMe PoC phase tracked in [Phase Codegen and CI Verification Session Status](/phase-codegen-and-ci-verification-session-status.md), [KnowMe PoC Codegen and Tauri Verification C-102](/knowme-poc-codegen-and-tauri-verification-c-102.md), and [KnowMe PoC Phase PR #1 Opened for T6-T12](/knowme-poc-phase-pr-1-opened-for-t6-t12.md).

## Phase Goal

As revised on `2026-07-15`, the phase deliverable is a **working proof-of-concept application**, not only codegen and CI verification.

The PoC must be built under `apps/<name>/`, use repository scaffolds and skills, and be based on KnowMe reference documentation in `docs/reference-app/`. It should prove the skill package end-to-end and showcase the broadest practical range of supported capabilities:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter, Tauri, and web from one Rust core
- Feature subset selected through web research on showcase-app best practices and 2026 on-device AI feasibility

Supporting objectives remain:

- Run the real codegen pipeline on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - `flutter pub get`
  - `pnpm install`
- Confirm pre-codegen warnings clear once generated code and sibling packages exist.
- Resolve or work around the PEM install blocker: `@prometheus-ags/entity-graph-core@workspace:*` unresolvable outside the PEM monorepo.
- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter, continuing the mobile verification line from [KnowMe PoC Android assembleDebug Running on SM S936U](/knowme-poc-android-assembledebug-running-on-sm-s936u.md)
- Wire CI to run:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - Boundary test suites against the PoC on every push

## Session Result

- `pnpm test` now passes: **10/10 tests passing from a clean install**.
- Nothing was skipped or deleted.
- Four reported failures were caused by three stacked issues. The initial symptom masked deeper failures because both test files failed to load before the four tests could run.

## Root Causes and Fixes

### 1. Tauri plugin package exported missing `dist/`

Package: `@prometheus-ags/tauri-plugin-gen-ui`

Problem:

- `package.json` declared:
  - `"files": ["dist"]`
  - entrypoints under `./dist/index.js`
- `dist/` is gitignored and was not built before local install.
- `pnpm` snapshots a `file:` dependency at install time and honors `files`.
- The linked package therefore contained only `package.json` and `README`; imports failed before tests could execute.
- A `postinstall` build cannot fix this because `pnpm` snapshots the package **before** running `postinstall`.

Fix:

- Point local package entrypoints at `src`.
- Rely on the consuming app’s Vite setup and TypeScript `moduleResolution: "bundler"` to consume TypeScript directly.
- Add `publishConfig` so published package consumers still resolve to built `dist` outputs.
- Preserve package build/pack behavior so published packages still contain `dist` correctly.

### 2. Vitest did not mock linked dependency because plugin was externalized

Problem after imports resolved:

```text
Cannot read properties of undefined (reading 'invoke')
```

Cause:

- As a linked dependency, the plugin was externalized and loaded through Node instead of Vite’s module registry.
- Existing tests mocked `@tauri-apps/api/core` with `vi.mock(...)`.
- Because the plugin loaded outside Vite’s registry, the mock did not apply inside the plugin.
- The plugin hit the real Tauri internals path and attempted to access `window.__TAURI_INTERNALS__`, which is undefined under `jsdom`.

Fix:

- Inline `@prometheus-ags/*` in the test/Vite configuration so Vitest transforms the linked packages and applies mocks consistently.

### 3. Startup store `inflight` guard latched forever after success

File:

```text
apps/knowme-poc/desktop/src/features/startup/stores/startupStore.ts:31
```

Problem:

- The `inflight` promise guard was cleared only on failure.
- After one successful startup run, `inflight` remained set forever.
- Subsequent `run()` calls became silent no-ops that returned the stale resolved promise.
- This preserved neither retry behavior nor expected repeated startup semantics.

Fix:

- Move `inflight` reset into a `finally` block.
- Preserve the guard’s intended React StrictMode de-duplication behavior.

Verified behavior with a throwaway regression test:

- Failed boot now retries.
- Concurrent calls still de-dupe.
- Expected invokes observed: `3`, not `6`.

## Test Assertion Updates

Only command names were changed in the tests, based on Rust-side plugin registration.

Rust source establishes the Tauri plugin namespace:

```rust
Builder::new("gen-ui")
```

Location:

```text
src/lib.rs:28
```

Therefore Tauri routes commands as:

```text
plugin:gen-ui|run_migrations
```

The tests previously asserted bare command names from a pre-plugin design. Those assertions were incorrect.

The `not.toContain('attach_sync_shapes')` assertion was also updated. Left unchanged, it would have passed vacuously because no element could equal the bare name, which would silently remove the boot-order invariant the assertion was intended to protect.

## Related Packaging Bug Fixed Proactively

Package: `gen-ui-react`

- It had the same latent packaging bug as `@prometheus-ags/tauri-plugin-gen-ui`.
- It did not fail in the current test run because the only test touching it used `import type`, which is erased at compile time.
- A fresh clone running CI `tsc --noEmit` could fail once runtime/value imports are introduced.
- The same local-entrypoint and `publishConfig` fix was applied.

## Remaining Known Issue

`pnpm exec tsc --noEmit` still reports three pre-existing type errors in `@flint/react`, a git-URL dependency.

- Confirmed against a stashed pristine tree.
- Unrelated to the plugin packaging, Vitest, or startup-store fixes.
- Left unresolved in this session.

## Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
