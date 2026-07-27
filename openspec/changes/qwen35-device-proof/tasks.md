# Tasks

- [x] Add non-sensitive local runtime diagnostics.
      Evidence: `docs/certification/2026-07-27/android-certification-report.json`
      (`observedBackend`, `modelLoadMs`, `generateMs`, thermal, PSS) and the
      `fallback_reason` tracing field surfaced in the Settings diagnostics panel.
- [x] Add physical integration scenarios.
      Evidence: `docs/certification/2026-07-27/README.md` — exact-match prompt,
      cancellation, four stopped-turn intents, second-send-after-cancel,
      multi-turn history, and a permitted tool call, all on the SM-S936U.
- [x] Build and deploy iOS to Travis's iPhone.
      Evidence: release `Runner.app` (213.8 MB) built 2026-07-27 11:58 in
      1214s, signed, installed and launched on the iPhone 17 Pro Max
      (`3CF53A89-6EDC-544E-8555-1E9BE6F3F452`, iOS 27.0). Media save and share
      then verified there by the operator —
      `docs/certification/2026-07-27/ios-media-report.json`.
- [x] Build and deploy Android to the attached phone.
      Evidence: release APK built 2026-07-27 10:33, installed
      (`lastUpdateTime=10:52`), `libgen_ui_ffi.so` loaded, migrations ready,
      zero crashes on the SM-S936U (`R5GYB4AZD7A`).
- [x] Record model, backend, throughput, fallback, and scenario evidence.
      Evidence: `android-certification-report.json` (backend
      `gemma-4-e2b-it-litertlm`, `modelLoadMs: 228`, `generateMs: 7726`),
      `android-fallback-report.json` (`fallbackReasonObserved`, thermal 0), and
      `android-image-rendering-report.json` (durable media, four gates).

## Known gap

Memory-preflight downgrade is NOT exercised on device and the report says so
(`memoryDowngradeExercised: false`). It is covered by 6 unit tests behind the
`local-llama` feature. The budget lever is a process env var and Android cannot
inject one into a `monkey`/`am start` launch; `pm clear` would wipe the 2 GB
model. Stated rather than quietly omitted.
