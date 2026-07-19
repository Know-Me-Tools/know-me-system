# Android AgentSkills / Process Activation Roadmap

Date: 2026-07-19

This roadmap exists because the current Android LiteRT-LM recovery path can
certify `SkillActivation` rendering and persistence for trusted-local MCP tool
namespaces, but real agent/skill definition and execution must come from
Universal Agent Runtime (UAR). UAR has now been added as a pinned submodule so
KnowMe can integrate and modify it as needed. Until the mobile app actually
routes through UAR's embedded runtime/kernel, claiming physical Android proof
for real AgentSkills/process activations would be false.

## Current certified baseline

- `StreamEvent::SkillActivation` is typed and maps to
  `ContentBlock::Skill`.
- Flutter renders `SkillBlock` outside the answer bubble alongside thinking,
  citation, tool, memory, lifecycle, and artifact regions.
- Physical Android certification exists for the embedded trusted-local
  `knowme_diagnostics` MCP namespace:
  - `Skill knowme_diagnostics, running`
  - `Skill knowme_diagnostics, complete`
- This proves the cross-platform event/block/UI contract, not the future UAR
  skill registry path.
- The UAR submodule provides the real runtime authority for agent definitions,
  skill storage/routing, A2UI catalog management, and LLM provider access.
  KnowMe's existing Android LiteRT-LM work should become a UAR local-provider
  backend, not a parallel agent runtime.

## Release A — UAR runtime behind the existing Rust boundary

- Add a mobile-safe UAR runtime facade behind `gen_ui_agent`, not in Flutter.
- Use embedded-library mode on Flutter mobile; BossFang/LibreFang cannot be a
  local sidecar on phones.
- Use HTTP/service mode for web and desktop when UAR runs as a sidecar or cloud
  service.
- Register skill cards with:
  - stable skill id and display name;
  - instruction source/version;
  - tool list;
  - scope (`global`, `agent-private`, `agent-exposed`);
  - governance metadata;
  - eval fixture references.
- Route real skill selection/execution through UAR and emit UAR stream events
  into KnowMe's existing content-block projection.
- Keep the Flutter skills feature as UI only:
  `Widget → Riverpod → Repository → FFI → Rust/UAR`.
- Keep skill activation events provider-neutral:
  UAR process activations and MCP-derived skill wrappers both emit the same
  `SkillActivation` stream event.

## Release A.1 — Local model provider bridge

- Expose Android LiteRT-LM/Gemma 4 E2B as a UAR provider/backend.
- Translate UAR structured messages, tool declarations, tool results, sampling
  settings, cancellation, and lifecycle into the existing
  `gen_ui_inference::LiteRtLmProvider`.
- Translate LiteRT-LM deltas/tool calls/errors back into UAR-compatible events.
- Preserve the certified KnowMe lifecycle UI:
  download, verify, loading, ready, cancelling, cancelled, failed.
- Keep CPU as the universal Android correctness floor; GPU/NPU become optional
  UAR provider variants only after physical device proof.

## Release B — Process lifecycle normalization

- Define a process activation envelope:
  - `skillId`
  - `processId`
  - `runId`
  - `phase`
  - `status`
  - user-safe message
  - optional diagnostics
  - provenance for selected instructions/tools
- Map lifecycle transitions to visible chunks:
  - queued / selected → `SkillActivation(status: "queued")`
  - loading instructions → lifecycle phase, not answer text
  - running → `SkillActivation(status: "running")`
  - waiting for approval → confirmation/tool policy card
  - complete → `SkillActivation(status: "complete")`
  - failed/cancelled → terminal error/cancelled state plus skill status
- Persist process activations as first-class ordered message blocks.

## Release C — Physical Android certification

Add a physical gate equivalent to the diagnostics skill gate, but sourced from a
real installed AgentSkill:

1. Start from clean chat state while preserving model cache.
2. Install/register a minimal read-only local AgentSkill fixture.
3. Submit a prompt that routes to the skill.
4. Require visible ordered blocks:
   - skill selected/queued;
   - skill running;
   - any tool use/tool result emitted by the skill;
   - skill complete;
   - answer block.
5. Pull `entity-records.sqlite3` and verify the skill blocks are persisted in
   order on the assistant message.
6. Force-stop/relaunch and verify the same skill blocks restore.
7. Cancel mid-process and verify partial skill blocks remain with terminal
   cancellation state.

## Release D — Governance and process safety

- Mutating/open-world AgentSkills require confirmation exactly like mutating
  tools.
- Approval executes only a host-stored pending process/tool call by opaque id.
- Stale approvals become typed errors; they never deserialize as opaque FFI
  object strings.
- Skill instructions are excluded from user-visible answer Markdown unless the
  skill explicitly emits an answer block.
- Skill reasoning/process notes are excluded from future prompt context by
  default, matching thinking-block policy.

## Non-goal for the current Android LiteRT-LM recovery

Do not fake AgentSkills certification by adding another embedded diagnostics
tool and naming it “AgentSkills.” The current release can certify the semantic
block contract and Android local inference/tool path. Real AgentSkills/process
certification starts only after KnowMe routes mobile agent/skill execution
through UAR's embedded runtime/kernel behind the Rust core boundary.
