# PAGS-SPEC-KM-CRM-001 — Engagement & Relationship Model

**Status:** Draft (rev 0.1)
**Date:** 2026-08-03
**Owner:** Travis James
**Consumers:** Prometheus AGS (enterprise motion), KnowMe plugin marketplace, Morning Brief agent
**Companion:** PAGS-SPEC-KM-GOAL-001

---

## 1. Position

This is not a CRM. A CRM is a database of contacts with a status field bolted on. This is a **state machine over engagements**, where relationship records are edges rather than the primary entity.

The abstraction already exists in your estate: the San Saba **deal container** — ranked #1 of 44 candidate features — a digital replacement for the physical file folder and checklist, modeled as an active state machine rather than a passive record. That container is not San-Saba-specific. It is the general primitive.

**Core claim:** the CRM and the goal management system are the same state machine with different entity types. Building them as two systems duplicates the gate engine, the provenance model, the staleness clock, and the approval path. Build one substrate, instantiate twice.

If that claim is wrong, the place it breaks is entity lifecycle: engagements have external counterparties and legal artifacts; objectives do not. That difference is handled by typed payloads on a shared container, not by a second engine. Named here so it can be argued rather than assumed.

---

## 2. Shared substrate: `prometheus-container`

A new Rust crate. Single responsibility: typed, gated, provenance-carrying state machines.

```
prometheus-container/
├── src/
│   ├── container.rs      // Container<T> — the primitive
│   ├── gate.rs           // Gate definitions, entry/exit predicates
│   ├── checklist.rs      // Required artifacts + completion state
│   ├── provenance.rs     // Source attribution, verified vs. self-reported
│   ├── clock.rs          // Staleness, dwell time, SLA breach
│   └── ports/
│       ├── store.rs      // ContainerStorePort
│       ├── notify.rs     // ContainerEventPort
│       └── policy.rs     // ContainerPolicyPort (Cedar PDP)
```

### 2.1 Container primitive

```rust
pub struct Container<T: ContainerPayload> {
    pub id: ContainerId,
    pub kind: ContainerKind,        // Engagement | Objective | ...
    pub payload: T,
    pub stage: StageId,
    pub stage_entered_at: DateTime<Utc>,
    pub checklist: Vec<ChecklistItem>,
    pub gates: Vec<Gate>,
    pub edges: Vec<Edge>,           // typed relations to other containers/parties
    pub provenance: Provenance,
    pub owner: Did,                 // did:prometheus: | did:uar:
}
```

Invariants, enforced at compile time where possible:

- **Single-writer discipline.** One writer per container. Agents propose `ContainerProposal`; only the trusted host layer applies. The agent kernel crate does not depend on the store actuator — capability inversion at the Cargo level, not runtime policy.
- **Stage transitions are gated, never assigned.** No `set_stage()`. Only `attempt_transition(to, evidence) -> Result<Applied, GateFailure>`.
- **Provenance is mandatory.** Every field carries whether it was `Verified`, `SelfReported`, or `Inferred`, with source DID. This is what lets the Morning Brief label claims honestly instead of laundering an agent's guess into a fact.

### 2.2 Gates

A gate is an entry predicate plus a required-artifact set plus an approver.

```rust
pub struct Gate {
    pub id: GateId,
    pub from: StageId,
    pub to: StageId,
    pub requires: Vec<ChecklistItemId>,
    pub approver: ApproverRule,     // Human(Did) | Policy(CedarPolicyId) | Auto
    pub reversible: bool,
}
```

Cedar owns the threshold for `Auto`. Users may only tighten. Auto applies to reversible transitions only — same rule already standing in the governance model.

---

## 3. Engagement instantiation

`ContainerKind::Engagement` with payload:

```rust
pub struct EngagementPayload {
    pub account: AccountRef,
    pub entity: BusinessEntity,      // PrometheusAgs | KnowMe | TribeHealth
    pub motion: Motion,              // Insight | FusionPoc | Subscription | Retainer
    pub value: Option<Money>,
    pub contract_ref: Option<DocumentRef>,
    pub milestones: Vec<Milestone>,
    pub last_touch: Option<Touch>,
    pub next_gate: Option<GateId>,
}
```

### 3.1 Default stage machine (Prometheus AGS motion)

```
Identified → Qualified → Insight Assessment → Fusion POC → Subscription → Renewal
                  ↓            ↓                   ↓
               Dormant      Dormant             Dormant
```

Stages map to the existing motion rather than generic Salesforce stages. Each stage carries a **dwell SLA**; breach surfaces in the brief's Engagements & Gates section without anyone remembering to check.

### 3.2 Relationship edges

People are not rows in a contacts table. They are `Party` nodes joined to engagements by typed edges carrying **authority**, which is the field that actually matters and that every commercial CRM gets wrong:

```rust
pub enum PartyRole {
    Principal,        // final decision-maker  (Hal @ San Saba)
    TechnicalLead,
    Advisory,         // raises risk, does not decide  (Randy)
    Operator,
    Gatekeeper,
    Influencer,
}
```

Encoding authority explicitly prevents the failure mode you already named: inverting the hierarchy, treating advisory input as a decision. The container refuses a gate transition whose `approver` resolves to a non-`Principal` party.

---

## 4. v0 wire format — flat files

Ships **before** any plugin work. The Morning Brief reads this on day one; the plugin later emits the identical schema so the agent's read path never changes.

`~/prometheus-brief/pipeline.yaml`

```yaml
version: 0
generated_by: manual          # later: knowme-crm-plugin
engagements:
  - id: eng-ssr-p3
    account: San Saba Royalty
    entity: prometheus-ags
    motion: fusion-poc
    stage: fusion-poc
    stage_entered: 2026-07-14
    dwell_sla_days: 45
    value_usd: 40000
    contract_ref: PAGS-SOW-SSR-003
    parties:
      - name: Hal
        role: principal
      - name: Randy Jesberg
        role: advisory
    milestones:
      - id: m1
        title: Fabric install
        status: in_progress
        due: 2026-08-15
        provenance: self_reported
    next_gate: m1-acceptance
    last_touch: 2026-07-28
    open_risks:
      - buyer-conveyance generation defect
      - hard validation failure on legacy records lacking required dates
```

Rules the brief enforces against this file:

- `stage_entered` older than `dwell_sla_days` → surfaces as stalled.
- `last_touch` older than 30 days on any `principal` party → surfaces as a relationship gap.
- Any milestone marked `self_reported` and past `due` → surfaces with the provenance label attached. Never promoted to done.

---

## 5. KnowMe plugin surface

Plugin id: `knowme-crm`. Depends on `prometheus-container`, not on the KnowMe app shell.

- **Storage:** SurrealDB 3.x (RocksDB local, hybrid-cloud sync per the standing constraint).
- **UI:** React 19 / TSX in the Tauri desktop shell; Flutter view on mobile through `gen_ui_core`.
- **Auth/entitlement:** blocked on F-04 (Kratos + Flint Gate + entitlements + Stripe + license spine). **This is a hard dependency.** The plugin cannot be sold before F-04 exists.
- **Agent access:** read-only through `ContainerStorePort::read`. Write proposals route through the Tauri host approval gate.

### 5.1 Tier placement

| Tier | CRM capability |
|---|---|
| Free | Flat-file import/export only, no sync |
| $20 | Local container store, single device, manual gates |
| $200 | Sync, policy-driven auto-gates, agent proposals, multi-entity |

Risk to name: the $200 sovereignty premium is still unproven against incumbents selling frontier capability at the same price. A CRM plugin does not by itself validate that premium — it broadens the surface a buyer is evaluating, which cuts both ways.

---

## 6. Estimate

Per standing estimation standard: PMPO/agent velocity at 11.3x compression, implementation only, human gates uncompressed, agent-build and calendar reported separately.

| Item | Agent-build | Human-gated | Notes |
|---|---|---|---|
| `prometheus-container` crate | ~1.5 agent-days | — | Gate engine, provenance, clock |
| Engagement payload + stage machine | ~0.75 agent-days | — | |
| v0 YAML reader (brief-side) | ~0.25 agent-days | — | Ships first, unblocks the brief |
| `knowme-crm` plugin UI | ~2 agent-days | — | Blocked on F-04 |
| F-04 dependency | not estimated here | — | First revenue-blocking item estate-wide |
| Schema review / your approval | — | 1–2 sessions | |

**Critical path is F-04, not the container work.** The container crate can land in under a week of agent-build; the plugin cannot ship commercially at all until entitlements exist. Building the plugin UI before F-04 produces an asset that cannot be sold.

---

## 7. Open decisions

| ID | Decision | Blocking |
|---|---|---|
| D-CRM-01 | Does `prometheus-container` live in the Prometheus Fabric repo or KnowMe? Consequence: whether Prometheus AGS client deployments can use it without a KnowMe dependency. | Crate scaffolding |
| D-CRM-02 | Merge or separate — is the goal system a second `ContainerKind` or a separate crate? Spec assumes merge. | Both specs |
| D-CRM-03 | Does the SSR deal container implementation become the reference instantiation, or stay bespoke? | SSR Phase 3 scope |

D-CRM-02 is the one that gets expensive if deferred. Answer it before either crate is scaffolded.
