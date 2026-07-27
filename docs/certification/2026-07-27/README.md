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

### Tool call — proven once, not reproducible

One run showed the complete local tool path in the UI dump:

```
Skill knowme_diagnostics, running
  -> Tool completed / Call litert-t
  -> Skill knowme_diagnostics, complete
  -> TOOLSTATUSOK
```

shortlist -> gate -> execution -> streamed tool events -> continuation, all of it.
The script reported FAIL on that run because it asserted a literal the app has
never rendered; that assertion is fixed (commit `72b61ab`).

On **three** subsequent runs the model answered `TOOLSTATUSOK` DIRECTLY without
calling the tool, so the tool block was genuinely absent and the gate correctly
failed. That is model non-determinism on this prompt, not a script defect.

**This gate is not yet trustworthy as a release gate.** It needs either a prompt
that compels the call or an explicit retry budget. The successful UI dump was
overwritten by the later runs and could not be preserved — the quoted content
above is from the transcript of that run, not from a file in this directory.

## Still open

- Attachment flow has no Android certification script (the React side is covered
  by unit tests only).
- iOS airplane-mode certification (task 5.5).
- `CERT_CLEAN_CHAT_STATE=1` requires a debuggable build; every run here used
  `=0`, so chat state carried across runs.
