# Agent Fabric Convergence: KnowMe system

## Current position

This planning note is grounded in
`c561d89e783264ef1215764db69e57493c4b815b` on
`codex/agent-fabric-convergence`. KnowMe already has a substantial UAR integration
through `rust/crates/gen_ui_agent`: embedded and service deployments, local
inference routing, run projection, A2UI interaction, cancellation and durable
UAR persistence are represented in current source. The active
`embedded-uar-offline-agents` phase is still completing desktop persistence
certification; this initiative must not change that phase or its files.

`knowme_hands` and the agent strategy already describe declarative Hands,
placements and Cedar-governed skills. `knowme_ofp` and some Hands scheduling are
still explicitly partial or stubbed in `docs/implementation-roadmap.md`. The
convergence plan should extend those owned concepts rather than describe the
planned peer/team behavior as shipped.

## Dependency on accepted contracts

KnowMe must consume the accepted `D-UAR-P1` contract before adapting its facade.
The receipt needs to cover principal/session identity, approval and cancellation,
history ownership, embedded versus service lifecycle and stable capability
discovery. The existing `UarRuntimeFacade` remains the only KnowMe-to-UAR code
boundary; a UAR revision is an explicit migration, especially for embedded mobile
builds where the Rust API is not a compatibility promise.

Subsequent work also waits for C03 definitions, C04 deployment bindings, C06
logical instances, C09 teams, C12 personal peer consent and C17 representation
grants. C11 business replication is required only for enterprise-sync profiles;
basic standalone embedded execution should remain useful without it.

## Repository responsibility

KnowMe owns the sovereign personal-agent experience across mobile, desktop and
web. Its product vocabulary can remain “Hand,” but the layers must be explicit:

- a Hand is the approachable authoring view of an AgentDefinition or
  TeamDefinition plus workflow intent;
- a DeploymentBinding chooses embedded UAR, sidecar/service, home peer or other
  supported placement and pins capabilities and policy/grant revisions;
- an instance is the durable personal activation; a run is one bounded attempt;
- an effect is a separately governed action with an intent and outcome receipt.

The UI should show an honest per-profile contract. Mobile foreground,
opportunistic background and delegated execution have different guarantees.
Users should see selected placement, local/cloud context movement, current model
and skills, schedule limits, pending/offline authority, run history, approvals
and peer status. The existing Flutter/React layering remains binding and the UI
continues to project Rust-owned durable state.

Admin surfaces should cover the local catalog, runtime/provider status, Hands and
team instances, paired personal peers, grants, revocation, storage/retention and
audit export. Enterprise or BossFang-specific administration remains in its
owning service; KnowMe consumes scoped capabilities and status.

## Personal context and digital twins

KnowMe is the likely owner of a human’s private preference/context profile and
consent history. That profile supplies selected context; it is not itself an
agent definition and does not grant action authority. Personal peer sync must
replicate only eligible data classes under device membership, key, recipient,
purpose and offline-validity rules. Organization policy, runtime authority and
approvals must not be accepted as ordinary CRDT writes.

Human representation needs a separate private RepresentationGrant. It binds a
named human or office, trusted issuer, organization assignment, approved context,
communication audiences, allowed actions, disclosure, expiry, revocation and
evidence revision. CEO/CFO/CIO/security/marketing/product role templates remain
generic assistants until such a grant is installed. A personal grant cannot
authorize organizational spending, and simulated approval never satisfies a
human check.

Revocation prevents new access and effects. Previously disclosed plaintext may
remain with recipients, so the product must present that limit honestly.

## Portable definitions, skills and Codex

KnowMe should consume the collaboration document profile accepted in C03:
AgentDefinition, TeamDefinition, WorkflowDefinition and DeploymentBinding, with
RepresentationGrant stored privately. Its existing `HandDefinition` remains the
product authoring/governance projection and compiles or binds to UAR execution
artifacts. Skills are reviewed, revision-pinned dependencies whose declared use
is checked again at runtime; they do not carry grants or credentials.

Codex CLI should be used as the first reference harness for developing this
story. The inspected `codex-cli 0.154.0` is a native executable with stable
multi-agent/goals support, shared session browsing, worktree isolation and
explicit sandbox/approval controls; the [official build instructions](https://github.com/openai/codex/blob/main/docs/install.md)
build it from the `codex-rs` Cargo workspace. Its Rust harness is a useful model for fresh
subagent contexts, typed task ownership, steering, cancellation and durable
session inspection. KnowMe should implement the same portable semantics through
its Rust core and UAR facade, without embedding Codex or claiming its native
session/permission state transfers into a Hand.

## Non-goals

- Replacing `gen_ui_agent`, introducing a second chat/team loop, or bypassing the
  active embedded-UAR phase.
- Making OFP transport, CRDT writes or a role title an authorization source.
- Pretending mobile has reliable cron or silently falling back to cloud/peer
  execution.
- Uploading private context without an explicit grant or packaging a
  RepresentationGrant with a portable agent/team.
- Claiming cross-host takeover, exactly-once effects or complete Hands/OFP
  execution while the current roadmap still marks those paths partial.

## Next repository-scoped KBD child

After the active persistence certification completes and `D-UAR-P1`, C03 and C04
are accepted, create `agent-fabric-knowme-definition-binding`. Give it exact
ownership in `gen_ui_agent`, `knowme_hands` and the corresponding Flutter/desktop
projection files. Its first slice should round-trip one portable declarative Hand
into an accepted UAR artifact, bind it to embedded or selected service placement,
and show the effective binding without adding teams or peer sync. A later
`agent-fabric-knowme-personal-consent` child should consume C12 before any digital
representation work; C17 remains the gate for named-human representation.
