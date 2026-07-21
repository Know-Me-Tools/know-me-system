# EXECUTION: embedded-uar-offline-agents

Project: KnowMe
Date: 2026-07-20
Selected backend: openspec
Dispatched to: Codex self-execution through `kbd-apply`
Backend rationale: The change crosses the UAR submodule, shared Rust core, two mobile targets, Flutter, persistence, and certification; OpenSpec/KBD task tracking preserves the required architecture and evidence gates.
Backend entrypoint: `/kbd-apply embed-uar-mobile-offline`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/model-runtime-per-device/children/embedded-uar-offline-agents/plan.md`

## Execution scope

- `embed-uar-mobile-offline`: Build and certify one offline embedded UAR kernel with Android LiteRT-LM and iOS Metal host drivers.

## Dispatch contract

- `embed-uar-mobile-offline` → Codex through KBD-owned OpenSpec apply
  - Entry: apply one unchecked task at a time from `openspec/changes/embed-uar-mobile-offline/tasks.md`
  - Model class: frontier
  - Concrete model: current Codex frontier agent
  - Model rationale: new public runtime abstraction, dependency feature gating, three client/runtime boundaries, native cross-compilation, and device certification
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/children/embedded-uar-offline-agents/progress.json`
  - Handoff: mark each OpenSpec task at its verified boundary; mark implementation complete only after the complete code/integration contract is delivered

## Approval gates

- None for repository implementation and connected-device deployment already requested by the user.
- Mutating agent tools remain runtime user-confirmation gated; that product policy is not bypassed by implementation.

## Fallback conditions

- If UAR transport-only dependencies cannot be made mobile-safe without destabilizing the server, split them behind crate/feature boundaries while preserving the same embedded API.
- If the server persistence backend cannot cross-compile, use a host `PersistenceLayer` adapter over KnowMe's durable local store; never silently fall back to ephemeral production data.

## Verification requirements

- UAR core/SDK unit and integration tests.
- KnowMe Rust workspace tests and strict linting for affected crates.
- `aarch64-linux-android`, `aarch64-apple-ios`, iOS simulator, and desktop builds.
- Network-disabled desktop, physical Android, and iOS device/simulator functional runs covering agent prompt, tool loop, A2UI, cancellation, and restart restoration.

## Progress ledger

- [IN_PROGRESS] `embed-uar-mobile-offline` — Codex/KBD apply

## Outputs

- UAR core + SDK embedded library APIs.
- KnowMe local host drivers and direct FFI facade.
- Flutter/desktop routing and diagnostic updates.
- Tests, logs, screenshots, OpenSpec verification, KBD reflection, and Karpathy records.

## Blockers

- Known current UAR dependency mismatch: `rmcp 2.2.0` expects `sse-stream` API newer than lockfile `0.2.3`; repair is task 5.1 prerequisite evidence.

## Reflection handoff

- Compare the prior service-only assumption with actual embedded build/device evidence; record dependency, persistence, event-parity, and model-driver lessons in KBD, Prometheus wiki, and private Karpathy memory.

EXECUTION READY
