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
- Add a physical-device wrapper that installs the APK, launches KnowMe, captures
  logcat, runs the local model smoke prompt, and fails unless the expected
  runtime log lines appear.
- Store device identity, Android API level, ABI, SoC, backend, model artifact,
  load time, first-token latency, throughput, memory pressure, and thermal state
  in a fleet capability record.
- Do not allow “built successfully” to satisfy Android native-inference
  completion. Completion requires a device-side gate report.
- Keep the current static gate strict about vendor dependencies: startup-critical
  Android libraries may depend only on NDK/public platform libraries, not
  device-pulled OpenCL/Vulkan/GPU driver binaries.

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
  tools on a physical device. The code path must prove that approval executes
  only a host-stored pending call by approval ID, never arbitrary model-visible
  interaction prose.
- Persist tool-use, tool-result, confirmation, cancellation, and error evidence
  with the same AG-UI/A2UI content-block contract as other providers.

## Explicit non-goal

Do not revive Vulkan/OpenCL as the Android production baseline. Those lanes may
exist later only as quarantined optimization experiments behind the same
six-gate physical proof standard. LiteRT-LM CPU remains the universal Android
correctness floor.
