# UAR Future Release Roadmap for KnowMe

Date: 2026-07-19

This roadmap captures the future-release work created by the decision that
Universal Agent Runtime (UAR) owns real agent execution, skills, A2UI registry
management, tool governance, and general LLM/provider access. KnowMe's local
model work stays valuable, but it becomes a local provider/backend that UAR can
route to.

## Current baseline

- UAR is pinned as a source submodule at `rust/vendor/universal-agent-runtime`.
- KnowMe can normalize representative UAR `uar.agui/1` event payloads into its
  provider-neutral `StreamEvent` contract.
- The same projection path then renders UAR reasoning, skill activation, tool
  lifecycle, text, error, completion, and unknown custom payloads as KnowMe
  A2UI/content blocks.
- `gen_ui_agent::uar::UarServiceRuntime` can create a UAR run through the UAR
  HTTP SDK, consume its SSE stream, and project events into KnowMe A2UI blocks.
- Flutter FFI and the Tauri plugin expose read-only UAR runtime status.
- Android local inference is still owned by the current LiteRT-LM path. It is
  not yet routed through UAR.

## Release UAR-1 — Runtime facade and projection certification

Goal: make UAR visible as a first-class runtime boundary without changing
production chat execution yet.

- Keep `gen_ui_agent::uar` as the stable local boundary.
- Expose `UarRuntimeStatus` through FFI/Tauri so every platform can show:
  - runtime authority;
  - target access mode;
  - current integration state;
  - submodule commit;
  - whether mobile sidecars are allowed. **Done.**
- Provide a service-mode facade for desktop/web/cloud UAR:
  - create run;
  - stream events;
  - cancel run;
  - project UAR events into KnowMe A2UI. **Done at Rust seam level.**
- Add golden fixtures from UAR's `uar.agui/1` profile:
  - reasoning;
  - text;
  - tool call lifecycle;
  - tool result;
  - `uar.skill.activated`;
  - A2UI/artifact availability;
  - citation/memory events;
  - unknown future custom events.
- Verify every fixture becomes an ordered KnowMe content-block projection and
  unknown events are preserved as artifacts.

Exit gate: protocol/component tests prove UAR event projection across Flutter,
React/web, and Tauri. Service-mode execution must additionally be proven
against a real UAR service before desktop/web can be called runtime-certified.

## Release UAR-2 — Mobile embedded kernel

Goal: run the UAR agent loop in-process on mobile while preserving the existing
Flutter → Rust FFI boundary.

- Extract or stabilize a mobile-safe UAR embedded kernel profile:
  - no HTTP server requirement;
  - no local sidecar process;
  - no BossFang/LibreFang sidecar dependency on phones;
  - cancellable run manager hooks;
  - mobile persistence/config hooks;
  - provider registration traits;
  - in-process UAR event stream.
- Register KnowMe's Android LiteRT-LM/Gemma local lane as a UAR provider.
- Keep Android CPU LiteRT-LM as the universal correctness floor.
- Keep GPU/NPU delegates optional and selected only after physical-device
  certification.
- Route one Android chat turn through UAR while preserving current lifecycle
  messages, cancellation, persistence, tool validation, and visible error
  semantics.

Exit gate: attached Android device completes a UAR-routed local chat turn with
the same evidence level as the current LiteRT-LM certification.

## Release UAR-3 — Real skills and A2UI registry

Goal: replace diagnostic skill simulation with actual UAR-owned skill and A2UI
behavior.

- Load/register a real UAR skill fixture on mobile.
- Emit `uar.skill.activated` for selected/running/complete/failed lifecycle.
- Render skill activation chunks outside answer Markdown, parallel to thinking
  and citation chunks.
- Use UAR's A2UI registry as the catalog authority.
- Fail closed on unknown or unsafe A2UI component ids.
- Cache approved mobile renderer profiles for offline rendering.
- Persist skill/A2UI blocks in `entity-records.sqlite3` with run/message
  provenance.

Exit gate: physical Android proof shows skill selection/execution, A2UI
artifact rendering, persistence, restart recovery, and cancellation.

## Release UAR-4 — Web and desktop convergence

Goal: make desktop and browser consume UAR through the service path while mobile
uses the embedded path.

- Desktop uses UAR over HTTP/SSE as a supervised service or sidecar.
- Web uses cloud/service UAR over HTTP/SSE.
- Both routes feed the same `UarEventProjector` behavior as mobile.
- React moves remaining agent/tool/prompt ownership out of UI stores and into
  UAR-backed typed service calls.
- Browser/PGlite remains a local projection/cache, not the owner of runtime
  truth.

Exit gate: identical UAR AG-UI fixtures produce identical ordered content
blocks on Flutter, React/web, and Tauri.

## Release UAR-5 — Cloud fallback and provider routing

Goal: allow UAR to choose local or cloud execution without hiding the backend
from the user.

- UAR receives local runtime diagnostics from KnowMe providers.
- UAR can route to:
  - Android LiteRT-LM local provider;
  - iOS local provider;
  - desktop local provider;
  - cloud provider;
  - cloud BossFang/LibreFang-accessible service.
- Mobile uses cloud access for capabilities that require BossFang/LibreFang or
  other sidecar-only services.
- User-visible lifecycle states show local/cloud routing, fallback reason,
  cancellation, and errors.

Exit gate: routing/fallback is visible, persisted, cancellable, and never
silently swaps local execution for cloud execution.

## Non-goals

- Do not reintroduce Android llama.cpp/OpenCL/Vulkan as the default path.
- Do not make Flutter own agent orchestration, tool policy, prompt assembly, or
  A2UI catalog decisions.
- Do not claim UAR mobile runtime certification while only the projection seam
  is implemented.
- Do not run BossFang/LibreFang as a local mobile sidecar.
