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
- Add a second physical model-certification wrapper that runs the local model
  smoke prompt and fails unless the expected LiteRT-LM load/generation/tool
  runtime log lines and UI-visible content blocks appear.
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
  office/personal task following, Mermaid, code, constrained JSON, multi-turn
  memory, cancellation, recovery after restart, and multiple safe tool calls.
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

## Explicit non-goal

Do not revive Vulkan/OpenCL as the Android production baseline. Those lanes may
exist later only as quarantined optimization experiments behind the same
six-gate physical proof standard. LiteRT-LM CPU remains the universal Android
correctness floor.
