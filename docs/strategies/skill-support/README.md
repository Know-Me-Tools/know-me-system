# Skill Support Strategy — Tools, Skills, Scopes, and the Agent Convergence

**Scope:** how skills work in the capability-centric plugin model — the tool/skill
distinction, the three skill scopes (global, agent-private, agent-exposed), MCP
interop in both directions, governance and settings for skills, and the long-run
convergence between skills and agents.

**Status:** v1.0 strategy, 2026-07-18. Child of
`docs/strategies/plugins/README.md` (capabilities, governance/identity §9, settings
substrate §11) and `docs/strategies/agents/README.md` (pipeline, placements).
Grounded in what exists: UAR's skill registry (`gen_ui_agent::tools`, `gen_ui_mcp`),
the prometheus-skill-pack (95+ instruction-package skills), and Flint Kiln's
`flint-skill` authoring SDK.

---

## 1. Tools vs. skills — the distinction that keeps the system legible

Both are capabilities; both therefore arrive as plugins. The split:

| | `knowme:tool` | `knowme:skill` |
|---|---|---|
| What it is | A single callable: name, description, JSON-Schema input/output, annotations — **MCP-tool-shaped by construction** | A **packaged practice**: one or more tools + instructions (prompt material) + policy defaults + settings + optional eval fixtures |
| Analogy | An MCP server tool | A prometheus-skill-pack skill / Claude skill folder |
| State & autonomy | None — a stateless contract | None — still no autonomy; the *instructions* shape how agents use the tools |
| Where it registers | UAR tool registry | UAR skill registry; its tools register beneath it |
| Spec authority | Its schemas | Its card + instruction docs (embeddable → routable) |

The distinction that stays permanent even as skills and agents converge (§5):
**skills never have identity, lifecycle, or schedule.** An agent *is someone*
(a DID principal, per plugins doc §9); a skill is *something known how to do*.

Why both exist: tools are what the runtime can verify (schemas, Cedar checks per
invocation); skills are what makes tools *usable well* (instructions, sequencing,
guardrails). The skill-pack repos prove the second layer carries most of the value;
the tool layer is what makes it enforceable.

## 2. The three scopes

Declared on the skill's capability card; enforced by UAR at registration and by Cedar
at invocation:

- **`global`** — registered into the container's UAR registry; any agent may request
  it, subject to the standard per-agent, per-skill grants (FUNC-SPEC §06 first-use
  consent). This is today's model, unchanged.
- **`agent-private`** — bound to one agent instance (or bundled inside the same plugin
  as its agent capability); invisible to the registry browsing UI of other agents;
  Cedar denies any other principal structurally, not just by default. Use: an agent's
  internal helper verbs that would be meaningless or dangerous as shared surface.
- **`agent-exposed`** — the interesting one: an **agent publishes a skill interface
  that other agents (and users) invoke as if it were a tool**. The call is
  synchronous-looking at the caller; behind it, the exposing agent runs an A2A task
  (agents doc §4.1) under **its own identity** — the caller's audit entry records
  `(user, calling-agent → exposed-skill → serving-agent)`. This is agent-as-tool,
  and it composes: pipelines of agents without a new orchestration primitive.

Scope is part of governance review: installing a plugin whose skill is
`agent-exposed` shows *who may call it* (which principals, from which placements)
exactly like any other permission.

## 3. MCP interop — both directions, at the edge

- **Inbound (MCP server → skill):** an MCP server registers as a `knowme:tool` set
  under a thin generated skill card (this is `gen_ui_mcp` today, re-described). The
  wrapper card supplies what MCP lacks for our system: scope, governance block,
  settings schema, embeddable description for routing.
- **Outbound (skill → MCP):** any `global` or `agent-exposed` skill can be served to
  external harnesses (Claude Code, other agent systems) as MCP tools by the
  sidecar/BossFang — the container becomes an MCP server whose tool list is the
  user's governed skill registry, with Cedar and the dual-identity audit (plugins doc
  §9) still in the path. KnowMe's skills become usable *from* the user's other AI
  tools without any of them bypassing governance.
- MCP remains the interop boundary, not the internal model: internally everything is
  capability cards, because MCP tool specs alone carry no scope, governance, or
  settings semantics.

## 4. Skills get the full plugin treatment

Because skills are capabilities (plugins doc §2–§3), they inherit without exception:

- **Settings** — schema on the card, stored in the generic substrate (plugins doc
  §11), scopes device/user/tenant/agent; settings UI generated as A2UI forms; a
  skill's per-agent configuration ("which mailbox may Inbox Triage's `email-imap`
  read for *this* Hand") lives at the `agent` scope key.
- **Governance** — Cedar templates + budgets in the card's governance block; every
  invocation carries the `(user, agent)` pair; grants are per-agent, per-skill,
  revocable from the Skills tab (unchanged from FUNC-SPEC §06, now generalized).
- **Discovery/routing** — instructions and descriptions are embedded into the local
  capability index; the router can propose skills mid-conversation ("this looks like
  a `pdf-extract` task") with the same provenance/budget discipline as hooks.
- **Sharing** — skill definitions travel over OFP/export/`.pxt`; grants never do
  (plugins doc §10). Team instances publish org skill sets with org policy layered
  restrictively over personal.
- **Versioning/rollback** — card semver + the Skills tab's version history
  (FUNC-SPEC §04 tile 6), now backed by content-addressed plugin versions.

## 5. The convergence (and its limit)

The prediction in the requirements is right: skills and agents drift toward each
other. The model makes the convergence *mechanical* rather than confusing:

- an **agent exposed as a skill** (§2, `agent-exposed`) is callable like a tool;
- a **skill "promoted" to an agent** is just a new agent whose instructions are the
  skill's instructions and whose toolset is the skill's tools — a wizard affordance
  ("Create a Hand from this skill"), not a new concept;
- a declarative Hand is, squinting, *a skill plus identity, schedule, and policy* —
  which is exactly the residue that never converges. **Identity, lifecycle, schedule,
  autonomy: agent. Stateless know-how: skill.** The system supports every hybrid via
  composition of the two, so neither concept needs to blur.

## 6. Open questions

1. **Skill eval fixtures** — cards may carry eval/golden fixtures (the skill-pack's
   BDD habit); should the container run them at install (slow, honest) or on first
   grant (lazy)? Default: first grant, in `utility.fast`-priced background.
2. **`agent-exposed` reentrancy** — agent A calls B's exposed skill which calls A?
   Default: cycle detection with depth cap (3) in UAR; audit flags cycles.
3. **Instruction size vs. context budget** — large skill instruction sets compete for
   context; default: instructions are RAG'd (embedded, retrieved per relevance) above
   a size threshold rather than inlined. Confirm threshold policy.
4. **Naming** — "skill" is used by both this system and the AI-harness world
   (prometheus-skill-pack authors *development* skills for coding agents). The
   KnowMe Builder skills that *author* `.pxt` skills are themselves harness skills —
   document the two senses once, prominently, and never mix them in UI copy.

## 7. Implementation map (this repo)

- `gen_ui_agent::tools` + `gen_ui_mcp`: scope field + Cedar structural checks for
  `agent-private`; `agent-exposed` bridge (skill call → A2A task on the serving
  agent, identity-switched, audited as a pair-chain).
- Sidecar/BossFang: outbound MCP server surface over the governed registry (§3).
- `knowme_plugin_host`: `knowme:tool` / `knowme:skill` card parsing; instruction
  embedding into the capability index on install.
- Skills tab (desktop `features/skills` blueprint, mobile counterpart): scope
  badges, per-agent grant matrix (already specced), exposed-skill caller lists,
  version history.
- KnowMe Builder harness skills: `pxt-new-skill`, `pxt-skill-evals`,
  `pxt-expose-agent-as-skill` join the W7 list in the extensibility spec.
