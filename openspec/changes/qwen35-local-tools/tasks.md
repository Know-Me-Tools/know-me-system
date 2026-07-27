# Tasks

- [x] Reuse the existing MCP tool schema and execution registry.
      Evidence: `knowme_diagnostics` MCP server registered at mobile boot —
      `gen_ui_ffi::api::boot: mobile diagnostics MCP tools registered
      server=knowme_diagnostics` in device logcat.
- [x] Add Qwen local tool prompt formatting and strict parsing.
      Evidence: the multi-tag segmenter (`think` + `tool_call`) wired into
      `mlx.rs`, `llama.rs` and `litert_lm.rs`, with `ReasoningFormat` per preset.
- [x] Implement a bounded permission-aware continuation loop.
      Evidence: the tool-approval gate — a proposed call KnowMe refuses to run
      surfaces an approval card and resumes via `local_tool_approve`; certified
      on device (`toolcall-success-ui-dump.xml`).
- [x] Publish tool events through the existing protocol adapter.
      Evidence: device UI dump shows the full streamed path — `Skill
      knowme_diagnostics, running` → `Tool completed` → `Skill
      knowme_diagnostics, complete` → continuation.
- [x] Add parser, policy, iteration, and event tests.
      Evidence: segmenter tests plus the `gen_ui_agent` suite (88 tests green
      as of 2026-07-27).
- [x] Verify one safe local tool flow on a physical phone.
      Evidence: `docs/certification/2026-07-27/README.md` — PASS on attempt 1
      of 3 with `history_count=0`, on the SM-S936U. That run also found the
      real cause of three earlier failures: accumulated conversation history
      (`history_count=108`) drowning the instruction, not model reluctance.

## Note on this file

These tasks were implemented and device-certified on 2026-07-27 but the
checkboxes were never updated. Ticked here against named evidence rather than
from memory; anything without evidence stayed unticked.
