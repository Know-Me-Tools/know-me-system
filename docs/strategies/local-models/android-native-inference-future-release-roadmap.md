# Android Native Inference Future Release Roadmap

Date: 2026-07-19

This roadmap captures the durable process changes from the Android six-gate
postmortem without reopening the superseded Vulkan/OpenCL plan. The production
Android baseline remains LiteRT-LM first: CPU correctness, optional
device-specific acceleration only after physical proof, and no startup-critical
vendor library dependency.

## Release R+1 — Make six-gate proof automatic

- Keep `scripts/android/verify-native-inference-gates.sh` as the minimum static
  APK gate.
- Use `scripts/android/verify-device-runtime-gates.sh` as the minimum physical
  launch gate: install the APK, launch KnowMe, capture logcat, prove the app
  process remains alive, prove `libgen_ui_ffi.so` loaded through
  `System.loadLibrary`, and prove Rust reached the mobile migrations-ready
  state.
- Use `scripts/android/certify-local-model-prompt.sh` as the minimum physical
  model prompt gate: run the launch gate, drive the Flutter chat UI with a
  fresh expected token prompt, require LiteRT-LM generation logs, keep the
  transcript pinned to `Latest`, and require the expected token in the visible
  UIAutomator hierarchy.
- Use `CERT_CLEAN_CHAT_STATE=1` when a prompt wrapper must reject stale
  transcript contamination. The shared `scripts/android/lib-chat-state.sh`
  harness backs up only app-private conversation databases with `run-as`,
  preserves downloaded model caches, and restores the original files on exit.
- Use `scripts/android/certify-local-prompt-suite.sh` to run named physical
  prompt cases (`exact`, `office`, `mermaid`, `code`, `json`) through the same
  UI-driven gate. The suite is intentionally case-selectable so long-running
  model checks can be certified incrementally while still using one automation
  path.
- Use `scripts/android/certify-local-restart-recovery.sh` to prove a generated
  local-model response survives force-stop/relaunch and is restored as visible
  UI state.
- Use `scripts/android/certify-local-multiturn-history.sh` to prove the local
  context path includes previous turns: the second prompt intentionally omits
  the full marker and must recover it from the immediately previous assistant
  message.
- Use `scripts/android/certify-local-cancellation.sh` to prove the visible Stop
  control reaches Rust cancellation and renders a cancelled run state.
- Use `scripts/android/certify-local-tool-call.sh` to prove explicit safe local
  tool calling: the model must emit `knowme_diagnostics__local_runtime_status`,
  the host must execute it, the UI must render tool-use/tool-result blocks, and
  the continuation must produce final visible answer text. The wrapper passed
  through the shared clean-session harness on the attached Samsung with visible
  `Tool use`, `Tool completed`, and `TOOLSTATUSOK` evidence.
- Use `scripts/android/certify-local-tool-result-fallback.sh` as the
  deterministic debug-build harness for the empty-continuation fallback branch.
  It must use fresh markers, force the Android bridge fallback sentinel, require
  current logcat tool-call evidence, and reject stale UI content. The wrapper
  now uses reversible clean chat-state isolation for this gate and has passed on
  the attached Samsung.
- Use `scripts/android/certify-local-skill-activation.sh` to prove skill
  activation chunks render as first-class `ContentBlock::Skill` regions outside
  answer content. The Samsung baseline passed with visible running and complete
  `knowme_diagnostics` skill blocks around the diagnostic local tool call.
- Add richer physical model-certification wrappers for branch-tail isolation.
  General clean-session isolation now exists for prompt gates, restart
  recovery, multi-turn history, cancellation, and explicit safe tool calling;
  delete/regenerate/resubmit branch behavior still needs its own physical
  automation.
- Store device identity, Android API level, ABI, SoC, backend, model artifact,
  load time, first-token latency, throughput, memory pressure, and thermal state
  in a fleet capability record.
- Do not allow “built successfully” to satisfy Android native-inference
  completion. Completion requires a device-side gate report.
- Keep the current static gate strict about vendor dependencies: startup-critical
  Android libraries may depend only on NDK/public platform libraries, not
  device-pulled OpenCL/Vulkan/GPU driver binaries.
- Add an error-message-to-gate decoder to the verification output so failures
  like `undefined symbol`, namespace access denial, `SIGTRAP` in
  `__loader_dlopen`, missing `uses-native-library`, and zero backend platforms
  point to the gate that actually failed.
- Add an APK packaging check for current Play/Android native-library alignment
  expectations, including 16 KB page-alignment compatibility where applicable.
- Add a device probe that records public vendor-library availability from the
  phone itself. The probe is evidence for optional accelerator experiments only;
  it must never make a vendor library startup-critical.

## Release R+2 — AgentSkills/process hardening

- Author an `android-native-inference` skill that forces agents to apply the
  six-gate model before editing Android native runtime code.
- Author a `device-verified-completion` skill whose script owns install,
  launch, logcat scraping, and pass/fail criteria.
- Amend the hybrid mobile architecture skill so native Android inference work
  routes through both skills automatically.
- Keep the skills concise and put heavy references/scripts under progressive
  disclosure so agents execute verification instead of reconstructing the
  rules from memory.
- The `android-native-inference` skill must include hard prohibitions from the
  postmortem: do not add startup-critical `DT_NEEDED` entries for non-NDK vendor
  libraries, do not bundle device-pulled GPU/OpenCL/Vulkan libraries, do not
  hardcode `/vendor` or `/system/vendor` library paths, do not claim success
  from a build, and always resolve the SoC/device variant before making
  accelerator claims.
- Put the six-gate taxonomy, Android loading-regime table, public-library
  namespace notes, SONAME examples, manifest stanza, and error decoder under
  `references/`; put install/launch/logcat/probe automation under `scripts/`.

## Release R+3 — Fleet certification

- Certify the current Samsung device as the first baseline device.
- Add at least one non-Qualcomm modern Android phone before claiming broad
  Android support.
- Keep the generic LiteRT-LM artifact as the universal path until device
  measurements prove a better accelerator/model profile.
- Promote the SM8750-specific LiteRT-LM artifact only for matching Qualcomm
  devices after correctness, cancellation, thermal, and latency gates pass.

## Release R+4 — Richer local-agent certification

- Expand prompt certification beyond the embedded diagnostics MCP tool:
  branch-tail isolation and additional real-world tool and skill sources.
  Office/personal task following, Mermaid, code, constrained JSON, restart
  recovery, multi-turn history, cancellation, explicit safe tool calling,
  empty-continuation fallback, diagnostic skill activation, and shared
  clean-session prompt gating now have physical Android wrapper evidence.
- Certify the confirmation UX/policy for mutating, destructive, and open-world
  tools beyond the embedded diagnostic tool. The baseline physical proof now
  exists for a read-only/open-world diagnostic: approval executes only a
  host-stored pending call by approval ID, never arbitrary model-visible
  interaction prose.
- Persist tool-use, tool-result, skill activation, confirmation, cancellation,
  and error evidence with the same AG-UI/A2UI content-block contract as other
  providers.
- Treat skill activation chunks as first-class ordered semantic blocks, rendered
  outside answer Markdown in the same turn region model as thinking, citation,
  tool, memory, lifecycle, and artifact chunks.
- Extend skill activation certification beyond the embedded diagnostics source:
  real AgentSkills/process activations must use the same `skill` block ordering,
  persistence, copy, and recovery semantics as the diagnostic baseline.

## Explicit non-goal

Do not revive Vulkan/OpenCL as the Android production baseline. Those lanes may
exist later only as quarantined optimization experiments behind the same
six-gate physical proof standard. LiteRT-LM CPU remains the universal Android
correctness floor.
