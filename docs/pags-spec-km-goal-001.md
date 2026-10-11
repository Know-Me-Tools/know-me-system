# PAGS-SPEC-KM-GOAL-001 — Agentic Goal Management

**Status:** Draft (rev 0.1)
**Date:** 2026-08-03
**Owner:** Travis James
**Depends on:** PAGS-SPEC-KM-CRM-001 (`prometheus-container` substrate)
**Consumers:** Morning Brief agent, KnowMe plugin marketplace

---

## 1. Position

Task managers fail for the same reason CRMs fail: they store items, not state. An item has a checkbox. A commitment has an owner, a source, a deadline, a blocking edge, a staleness clock, and a provenance record of whether anyone actually verified it moved.

This spec instantiates `ContainerKind::Objective` on the same substrate defined in PAGS-SPEC-KM-CRM-001. If D-CRM-02 resolves against the merge, this becomes a parallel crate and the gate/clock/provenance code duplicates. That duplication is the cost of answering D-CRM-02 late.

**Design constraint that separates this from every AI todo app:** the agent never marks anything complete. It proposes, cites evidence, and labels confidence. Completion is a human act at the trusted host layer. An agent that can close its own commitments is an agent that reports success.

---

## 2. Entity model

Four levels, each a container or a child of one.

```
Objective       (quarter horizon, ContainerKind::Objective)
  └── KeyResult (measurable, has target + current + source)
        └── Commitment (owed by someone, to someone, by a date)
              └── Step   (atomic, agent-executable or human-only)
```

```rust
pub struct ObjectivePayload {
    pub horizon: Horizon,            // Quarter | Month | Week
    pub entity: BusinessEntity,      // travisjames.ai | KnowMe | PrometheusAgs | TribeHealth
    pub thesis: String,              // why this matters — one sentence, required
    pub key_results: Vec<KeyResult>,
    pub blocked_by: Vec<BlockRef>,   // containers OR decisions
}

pub struct Commitment {
    pub id: CommitmentId,
    pub statement: String,
    pub owed_by: Party,
    pub owed_to: Party,
    pub due: Option<Date>,
    pub source: CommitmentSource,    // Fireflies | Email | Manual | Brief
    pub source_ref: String,          // meeting id, message id — always resolvable
    pub evidence: Vec<EvidenceRef>,  // commits, files, sent messages
    pub slip_count: u8,              // increments each brief it rolls unactioned
}
```

`slip_count` is the field that does the work. Three consecutive slips is not a scheduling problem; it is a signal that the commitment is either misscoped or not actually wanted. The brief says so plainly at slip 3 and stops re-listing it at slip 5, moving it to a `Deferred` set with a note. Nagging past that point trains you to ignore the section.

---

## 3. Decision items

A first-class type, because your standing failure mode is decisions that rot silently rather than tasks that go undone.

```rust
pub struct Decision {
    pub id: DecisionId,
    pub question: String,
    pub options: Vec<DecisionOption>,   // each with named consequence
    pub blocks: Vec<ContainerId>,       // what cannot proceed
    pub opened: Date,
    pub cost_of_delay: CostOfDelay,     // Compounding | Fixed | Optionality
    pub decider: Did,
}
```

Current instances that seed the file on day one:

| Decision | Blocks | Cost of delay |
|---|---|---|
| SmsPort / flint-notify-ports crate placement | FRF, KnowMe web search D-01 | Compounding — both settle under one rule or diverge |
| Shell-vs-critical-path sequencing (SSR) | Phase 3 M1 | Compounding — conflated with agent-scope risk, never cleanly resolved |
| SurrealDB 3.2 on-device benchmark gate | KnowMe mobile store decision | Fixed |
| librefang open-core boundary | W2 dispatch work | Compounding — deadline already accelerated |
| D-CRM-02 (container merge) | Both this spec and CRM | Compounding |

`cost_of_delay: Compounding` items surface in every brief with days-open. Fixed items surface weekly. The distinction prevents the section becoming noise.

---

## 4. Agent role — proposal only

```rust
pub enum AgentAction {
    ProposeCommitment { extracted_from: SourceRef, confidence: Confidence },
    ProposeEvidence   { commitment: CommitmentId, evidence: EvidenceRef },
    ProposeSlip       { commitment: CommitmentId, reason: String },
    FlagStale         { decision: DecisionId, days_open: u16 },
}
```

No `CompleteCommitment`. No `CloseObjective`. The `knowme-goal-kernel` crate has no dependency on the write actuator — enforced at Cargo level, same inversion as the CRM substrate.

**Extraction sources**, in confidence order:
1. Git commits and PR merges → `Verified` evidence
2. Sent email / calendar completion → `Verified`
3. Fireflies transcript extraction → `SelfReported` (someone said they'd do it; nobody checked)
4. Agent inference from context → `Inferred`, always labeled, never auto-applied

The brief renders provenance inline. "Milestone 1 complete (self-reported, no commit evidence)" is a different sentence from "Milestone 1 complete" and the difference is the entire point.

---

## 5. v0 wire format

`~/prometheus-brief/goals.yaml` — hand-maintained until the plugin ships, identical schema after.

```yaml
version: 0
generated_by: manual
objectives:
  - id: obj-knowme-ship
    horizon: quarter
    entity: knowme
    thesis: KnowMe reaching paid tiers converts brand authority into recurring revenue
    blocked_by: [dec-f04]
    key_results:
      - id: kr-1
        statement: F-04 spine operational across all three entities
        target: 1
        current: 0
        source: manual
    commitments:
      - id: com-1
        statement: Wire llguidance into mobile composite features
        owed_by: travis
        owed_to: self
        due: 2026-08-14
        source: manual
        slip_count: 0
        evidence: []
decisions:
  - id: dec-notify-port
    question: SmsPort and notify port placement — FRF or new flint-notify-ports crate
    blocks: [obj-frf, obj-knowme-ship]
    opened: 2026-07-20
    cost_of_delay: compounding
```

---

## 6. Plugin surface

Plugin id: `knowme-goal`. Same stack, same F-04 dependency, same tier logic as `knowme-crm`.

One capability unique to this plugin: **horizon rollup.** Weekly commitments roll to monthly key results roll to quarterly objectives, and the rollup is computed, not asserted. If no commitment this week advanced any key result, the objective shows zero movement regardless of how busy the week felt. That is the single most useful thing the system can tell you and no commercial tool does it.

---

## 7. Estimate

Standard applied: 11.3x agent velocity, implementation only, human gates uncompressed.

| Item | Agent-build | Human-gated |
|---|---|---|
| `Objective` payload on shared substrate | ~0.75 agent-days | — |
| Decision type + staleness clock | ~0.5 agent-days | — |
| Extraction adapters (git, Fireflies, mail) | ~1.25 agent-days | — |
| v0 YAML reader (brief-side) | ~0.25 agent-days | — |
| `knowme-goal` plugin UI + rollup | ~2 agent-days | Blocked on F-04 |
| Seeding real objectives | — | 1 session, ~2 hours |

**Elapsed calendar time is dominated by F-04 and by your decision latency on D-CRM-02, not by build.** Agent-build for everything except the plugin UI is under a week. Both plugin UIs sit behind the same commercial blocker.

---

## 8. Interim operating cost

Until both plugins ship, `pipeline.yaml` and `goals.yaml` are hand-maintained. Realistic cost: 60–90 minutes per week, front-loaded on Monday. If that cost is not paid, the brief degrades to calendar-and-email summarization, which is worth having but is not a chief of staff.

Name the failure honestly: the most likely outcome is that the YAML files go stale in week three and the brief quietly becomes a news digest. The mitigation is that the brief itself writes back to `memory.md` and flags file staleness — if `pipeline.yaml` has not been modified in 10 days, that becomes a Blocked Decisions item about whether this system is actually wanted.
