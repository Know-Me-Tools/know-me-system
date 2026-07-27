# Certification run — 2026-07-27

Desktop (macOS release bundle) and Android (SM-S936U) run against the seven
chat-parity commits landed this session, plus the Tauri command-registration
fix required to build them.

- Desktop bundle: `tools.knowme.knowme`, built 2026-07-27 04:32
- Android: Samsung SM-S936U, SM8750, Android 16 / API 36, `R5GYB4AZD7A`
- APK sha256 recorded in `android-certification-report.json`

## Desktop — PASS

Driven through the real release app, not a dev server.

| Check | Result |
|---|---|
| One toolbar, no UAR/on-device lane switch | PASS — the toggle is gone |
| Trigger shows the agent NAME | PASS — reads `Orchestrator`, not `orchestrator-agent` |
| Picker lists the live UAR catalog | PASS — `Default Assistant`, `Orchestrator`, then `Local` |
| Flutter's copy, verbatim | PASS — header, subtitle, and the Local subtitle |
| Selection state not by colour alone | PASS — ✓ tracks the selected row |
| Keyboard: arrows + Enter select a row | PASS — Enter selected Local |
| `setMode` ordering contract | PASS — see below |
| Model-operation banner + Cancel | PASS — "Downloading model · 0 of 2173 MB", Cancel worked |
| Composer gating while the model loads | PASS — toolbar and composer disabled, re-enabled on cancel |
| Local lane generation | PASS — replied `CERTOK` exactly, ~13.4 tok/s |

**The ordering contract is the important one.** Switching to Orchestrator on a
machine with no UAR configured was REFUSED: the error surfaced
("Configure Universal Agent Runtime in Settings before using the UAR lane") and
the trigger stayed on `Local`. The UI never claimed a mode the runtime declined.
Under the previous code it would have shown Orchestrator while routing local.

## Android — PASS, with one gate not reliably reproducible

| Gate | Result |
|---|---|
| G3 install | PASS |
| G4 process alive, `libgen_ui_ffi` loaded, migrations-ready | PASS |
| Exact-match prompt | PASS — generated run token `KMEXACT045...` verbatim |
| Cancellation | PASS — a real Stop tap interrupted a live generation |
| Four stopped-turn intents | PASS — kept 938 chars; Steer/Resume/Retry/Discard all present with the kept-work line; Resume continued to 10,729 chars rather than restarting |
| Second send after a cancel | PASS — cycle 1 stopped at 934 chars, cycle 2 generated (the engine-rebuild fix holds) |
| Multi-turn history | PASS — second turn recovered the prior marker |
| Permitted tool call | See below |

Runtime evidence from `android-certification-report.json`:

- `observedBackend: gemma-4-e2b-it-litertlm` — the LiteRT-LM engine served the run
- `modelLoadMs: 228`, `generateMs: 7726`
- thermal status 0, PSS 3.08 GB under load
- corroborated in logcat: `LiteRT-LM generation timing generate_ms=12331 chars=23`

### Tool call — PASSING, root cause was context not compliance

Re-run after the fixes below: **PASS on attempt 1 of 3**, with `history_count=0`.

The gate was never failing on model reluctance. logcat showed
`history_count=108` — every certification run this session accumulated into one
conversation — with the engine emitting 2-, 4- and 6-character fragments at
`tool_calls=0`. The instruction was drowned in context. Starting a fresh
conversation (`CERT_NEW_CONVERSATION=1`) reduced history to zero and the model
called the tool immediately.

Three things had to be fixed to get there, two of them real app defects:

- The chat header's thread-list button and the sheet's new-conversation button
  were both icon-only with NO `Semantics` label. A screen reader announced
  nothing for either, and automation could not find them. Fixed in
  `chat_screen.dart`; the label is verified present on device.
- The fresh-conversation step originally WARNED and continued with dirty
  history, so three runs looked like model failures when the setup had never
  executed. It now exits 70.
- A retry budget (`TOOL_CALL_ATTEMPTS`, default 4) distinguishes a skipped call
  (retry — the run says nothing about the tool path) from a call that was
  invoked and still failed (fail immediately — retrying would hide a real
  defect).

Evidence preserved: `toolcall-success-ui-dump.xml`, containing
`Skill knowme_diagnostics, complete`.

### Fallback — PASSING

`android-fallback-report.json`, captured on device:

- `backendObserved: gemma-4-e2b-it-litertlm`
- `fallbackReasonObserved: CPU is the certification baseline; GPU/NPU require device proof`
- `thermalStatus: 0`

That reason line did not exist before this run. Rust computed `fallback_reason`
on every model load but never logged or rendered it, so no certification run
could record which backend was chosen or what it fell back from. It is now a
tracing field and a line in the Settings diagnostics panel.

**Memory-preflight downgrade is NOT exercised on device** and the report says so
(`memoryDowngradeExercised: false`). The budget lever is a process env var and
Android cannot inject one into a `monkey`/`am start` launch; `pm clear` would
have wiped the 2 GB model. It is covered by 6 unit tests behind the
`local-llama` feature (downgrade selection, budget override, preflight
rejection) — invisible to a bare `cargo test`, which is why an earlier note in
this session wrongly called it unevidenced.

### Superseded: the earlier tool-call finding

One run showed the complete local tool path in the UI dump:

```
Skill knowme_diagnostics, running
  -> Tool completed / Call litert-t
  -> Skill knowme_diagnostics, complete
  -> TOOLSTATUSOK
```

shortlist -> gate -> execution -> streamed tool events -> continuation, all of it.
The script reported FAIL on that run because its assertion was too narrow. NOTE
the correction: I described `Tool use knowme_diagnostics__local_runtime_status`
as a string the app never renders, but a later device dump shows it DOES appear
in some states. The change in `72b61ab` widened the assertion; it did not
correct a fabricated literal.

On **three** subsequent runs the model answered `TOOLSTATUSOK` DIRECTLY without
calling the tool, so the tool block was genuinely absent and the gate correctly
failed. That is model non-determinism on this prompt, not a script defect.

That conclusion — "model non-determinism" — was WRONG, and the section above
supersedes it. The real cause was accumulated conversation history, which the
retry budget alone would never have fixed: every retry inherited the same 108
messages. Preserved here because the wrong diagnosis is instructive.

## Still open

- Attachment flow has no Android certification script (the React side is covered
  by unit tests only).
- iOS airplane-mode certification (task 5.5).
- `CERT_CLEAN_CHAT_STATE=1` requires a debuggable build; every run here used
  `=0`. `CERT_NEW_CONVERSATION=1` is the workaround for gates that need a clean
  thread, but it is deliberately OFF for the fallback gate: a new conversation
  resets the mode to the default agent, which routes through the UAR and never
  loads a local model, so the diagnostics that gate asserts on would never be
  emitted.
- The fallback gate needs the conversation already set to Local and uses
  `CERT_SKIP_INSTALL=1`, because reinstalling resets the mode.
