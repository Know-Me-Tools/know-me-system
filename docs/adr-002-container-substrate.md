# ADR-002 — Container Substrate: Merge Engagements and Objectives

**Status:** Accepted
**Date:** 2026-08-03
**Decider:** Travis James (delegated: "do what is most architecturally sound")
**Resolves:** D-CRM-02
**Related:** PAGS-SPEC-KM-CRM-001, PAGS-SPEC-KM-GOAL-001, ADR-001

---

## Decision

**Merge.** One host-side container substrate with typed payload variants — `ContainerKind::Engagement` and `ContainerKind::Objective` — not two crates. One WIT world, one registered module family, one Cedar policy family, one store.

## Reasoning

### 1. The approved inversion already collapsed most of the difference

Under ADR-001 and the approved stateless-module architecture, the substrate is host-side. What actually lives in the substrate is: the gate engine, the staleness clock, the provenance model, and the delegation-chain binding. **All four are identical for engagements and objectives.** The only thing that differs is the payload record.

Two crates means two copies of a gate engine that must stay behaviorally identical, because the Morning Brief renders both through the same provenance rules. Divergence between them would be silent and would surface as inconsistent labeling — the exact failure the provenance model exists to prevent.

### 2. Registration burden under §07 scales with module count

KNOWME-SEC-AGT-001 §07 requires, per skill: a manifest, SHA-256 content hash, import-surface validation, a Cedar `policy_binding`, and immutable versioning with a pinned rollback path. That is fixed overhead per registered artifact.

Split: `crm-gate`, `crm-health`, `goal-gate`, `goal-rollup` — four registrations, four policy sets, four hash-pinned artifacts, four rollback paths to audit.

Merged: `container-gate` and `container-health` handle both kinds via a payload variant; `goal-rollup` remains distinct because horizon rollup is genuinely goal-specific. Three registrations. The gate module is the same code either way — splitting it duplicates the artifact without duplicating the logic.

### 3. Cross-kind edges are the actual payoff

This is the argument that carries the decision. A commitment blocked by an engagement gate is only expressible if both live in one graph.

Concretely: *"M1 acceptance is blocked on Hal's decision"* is simultaneously a pipeline fact and a goal fact. Under a split, that relationship is a synchronization problem between two stores — and synchronization between two stores of the same shape is the classic single-writer violation. Under a merge it is one edge in one graph, and the Blocked Decisions section of the brief reads it directly.

The San Saba deal container — ranked #1 of 44 candidate features — is the same primitive. A merged substrate means the SSR workspace shell and the KnowMe plugins share one store without the shell needing to load WASM at all.

### 4. One WIT world is a stronger validation surface

Per ADR-001, capability validation is a typed structural comparison against a declared world. One world describing `container-eval` is one thing to audit and one thing to keep correct. Two near-identical worlds invite drift in exactly the place where drift is a security finding rather than a bug.

## Cost of the merge, named honestly

- **Payload union growth.** A change to `EngagementPayload` recompiles the objective path. Mitigation: payloads behind a trait in separate modules; the substrate depends on the trait, not the variants.
- **Cedar scoping gets more careful, not less.** With one store, resource scoping must be by namespace rather than falling out of crate separation for free. Policies must scope on `resource.kind` and `resource.namespace` explicitly.
- **Classification asymmetry is the real risk.** Client engagement data carries `classification: internal` under the agent descriptor model. Personal objectives may warrant a higher sensitivity tier. A single store means that distinction is enforced by policy rather than by physical separation.

## When to revisit — named split trigger

**If personal objectives ever require a different classification or retention tier than client engagement data, and Cedar namespace scoping proves insufficient to enforce it, split the store — not the substrate.** The gate engine, clock, and provenance model stay merged regardless; only the persistence boundary moves.

That is the honest failure mode: a merged store makes a governance error easier to make. The mitigation is that it also makes the error *visible in one place*, whereas a split store makes it invisible until an audit.

## Consequences

1. `PAGS-SPEC-KM-CRM-001` §2 becomes the shared substrate definition for both specs; `PAGS-SPEC-KM-GOAL-001` §2 references it rather than restating it.
2. Module set becomes: `container-gate`, `container-health`, `goal-rollup`, `goal-extract`. Four, not five — `crm-gate` and `goal-gate` collapse.
3. Cedar policies must scope on `resource.kind` and `resource.namespace` explicitly. This is a spec addition, not an inference.
4. Both specs need rewriting against ADR-001 and ADR-002 together.
