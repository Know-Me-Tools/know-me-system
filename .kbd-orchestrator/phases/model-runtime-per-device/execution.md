# EXECUTION: model-runtime-per-device

Project: KnowMe
Date: 2026-07-18
Selected backend: `openspec` via `/kbd-apply`
Dispatched to: SELF (Codex)
Backend rationale: OpenSpec is present and the phase changes already have traceable proposals/tasks. `/kbd-apply` is required so each task updates KBD progress and waypoint state.
Backend entrypoint: `/kbd-apply` task-by-task driver
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/model-runtime-per-device/plan.md`

## EXECUTION SCOPE

- `qwen35-model-runtime`: device-aware Qwen3.5 catalog, verified downloads, acceleration, fallback, generation defaults
- `qwen35-mobile-vision`: typed image input and mtmd projector path
- `qwen35-local-tools`: bounded structured tool-call continuation
- `qwen35-device-proof`: physical iPhone and Android build/deployment evidence

## DISPATCH CONTRACTS

- `qwen35-model-runtime` → `/kbd-apply qwen35-model-runtime`
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6` (per project model policy)
  - Model rationale: high-complexity cross-layer native runtime change
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`
  - Handoff: complete each OpenSpec task through `kbd-apply`; run focused Rust checks before marking implementation complete

- `qwen35-mobile-vision` → `/kbd-apply qwen35-mobile-vision`
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6`
  - Model rationale: high-complexity FFI and native multimodal change
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`

- `qwen35-local-tools` → `/kbd-apply qwen35-local-tools`
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6`
  - Model rationale: high-complexity security-sensitive agent/tool loop
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`

- `qwen35-device-proof` → manual physical-device verification
  - Model class: frontier
  - Concrete model: `claude-sonnet-4-6`
  - Model rationale: external devices and deployment state require direct verification
  - Progress file: `.kbd-orchestrator/phases/model-runtime-per-device/progress.json`

## APPROVAL GATES

- Never activate an artifact before revision, byte length, and SHA-256 verification succeeds.
- Never execute a local tool call without registry and permission validation.
- Never accept Vulkan output without the runtime correctness self-test; CPU fallback is required.

## FALLBACK CONDITIONS

- If OpenSpec task state cannot be synchronized, stop and restore the KBD apply path before editing further.
- If a model/projector checksum or architecture check fails, discard the temporary artifact.
- If Metal/Vulkan initialization or smoke output fails, use the documented CPU/4B fallback and record diagnostics.

## VERIFICATION REQUIREMENTS

- `cd rust && cargo check --workspace`
- Focused Rust tests for catalog, resume/checksum, backend selection, and inference contracts
- `cd mobile && flutter analyze` and affected Flutter tests
- Physical-device proof for iPhone 17 Pro Max and Galaxy S25 Ultra in the final change

## PROGRESS LEDGER

- [IN_PROGRESS] `qwen35-model-runtime` — Codex via `/kbd-apply`
- [PENDING] `qwen35-mobile-vision` — Codex via `/kbd-apply`
- [PENDING] `qwen35-local-tools` — Codex via `/kbd-apply`
- [PENDING] `qwen35-device-proof` — Manual physical-device verification

## OUTPUTS

- OpenSpec task state and verification records
- Runtime catalog/diagnostic implementation
- Physical-device evidence attached to the final change

## BLOCKERS

- NONE at dispatch time

## REFLECTION HANDOFF

Reflect must consume per-change implementation, evidence, certification, and publication status, including any Android Vulkan fallback or iOS memory fallback observed on physical hardware.

## EXECUTION READY
