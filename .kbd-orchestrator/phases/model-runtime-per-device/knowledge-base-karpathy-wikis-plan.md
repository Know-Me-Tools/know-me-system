# PLAN: knowledge-base-karpathy-wikis document

Project: KnowMe
Parent phase: `model-runtime-per-device`
Date: 2026-07-19
OpenSpec available: YES
Changes to implement: 1

The assessment phase already produced a working draft of the target README and
its research package. This plan therefore governs deliberate review,
completion, and publication of that document. It does not misclassify the
existing draft as absent, and it does not authorize runtime implementation.

## CHANGE LIST (ordered)

1. `document-uar-karpathy-knowledge-strategy`: Create and publish the evidence-backed UAR RAG and Karpathy wiki strategy.
   - Scope: `docs/strategies/knowledge-base-karpathy-wikis/`, KBD document metadata, OpenSpec document artifacts, and only necessary cross-document supersession notes
   - Depends on: NONE
   - Recommended agent: Roo Code (Architect mode)
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Review and finalize the assessment draft as one coherent architecture strategy. It must define artifact authority, the evaluated Karpathy loop, mobile embedded-kernel boundaries, one-writer desktop supervision, web-through-Axum placement, synchronization, privacy/licensing gates, evaluation, rejected alternatives, open decisions, and staged delivery with traceable evidence.

## ACCEPTANCE CONTRACT

- The target README is self-contained and distinguishes current capability,
  inference, recommendation, blocker, and future work.
- Evidence corpus, wiki, and episodic trace authority are not collapsed.
- Mobile, desktop, and web each identify the runtime owner, process boundary,
  persistence authority, UI boundary, degraded mode, and blockers.
- All implementation claims trace to current KnowMe/UAR source. External
  technical claims cite original papers or official documentation.
- The research package contains valid JSON, a source registry, citation map,
  evidence graph, contradiction log, plan, report, and manifest.
- Existing browser-WASM and stale UAR future-work contradictions are resolved
  explicitly rather than silently overwritten.
- No Rust, Dart, TypeScript, database schema, dependency, deployment, or other
  application-code change is introduced by this OpenSpec change.
- Markdown local links, JSON syntax, KBD progress invariants, the OpenSpec
  document checklist, and the sycophancy gate pass.

## EXECUTION ROUND ORDER

Round 1: `document-uar-karpathy-knowledge-strategy`

This document change is independent of `litertlm-local-tools` and may be
executed without changing the implementation status of the runtime work.

## COMMANDS TO RUN

`/opsx:new document-uar-karpathy-knowledge-strategy`

## EXPLICIT SCOPE CUTS

- No UAR integration code or mobile-kernel extraction.
- No live service, device, retrieval, or performance certification.
- No resolution of the AGPL/product decision; record it as a gate.
- No automatic wiki promotion or skill optimization.
- No advancement of already completed runtime implementation changes.

## PLAN COMPLETE

Sycophancy self-check: passed at 0.0 with standard strictness for the
`pmpo_plan_phase` evaluation domain.
