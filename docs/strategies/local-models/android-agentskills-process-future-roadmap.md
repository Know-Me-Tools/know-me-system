# Android AgentSkills / Process Activation Roadmap

Date: 2026-07-19

This roadmap exists because the current Android LiteRT-LM recovery path can
certify `SkillActivation` rendering and persistence for trusted-local MCP tool
namespaces, but the mobile product does not yet contain a UAR-backed
AgentSkills/process runtime. The mobile skills feature README places that
registry integration in M5/M6. Until that runtime exists, claiming physical
Android proof for real AgentSkills/process activations would be false.

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
  skill registry.

## Release A — UAR registry behind the existing Rust boundary

- Add a mobile-safe UAR skill registry adapter behind `gen_ui_agent`, not in
  Flutter.
- Register skill cards with:
  - stable skill id and display name;
  - instruction source/version;
  - tool list;
  - scope (`global`, `agent-private`, `agent-exposed`);
  - governance metadata;
  - eval fixture references.
- Keep the Flutter skills feature as UI only:
  `Widget → Riverpod → Repository → FFI → Rust/UAR`.
- Keep skill activation events provider-neutral:
  UAR process activations and MCP-derived skill wrappers both emit the same
  `SkillActivation` stream event.

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
certification starts only after the UAR mobile registry exists behind the Rust
core boundary.
