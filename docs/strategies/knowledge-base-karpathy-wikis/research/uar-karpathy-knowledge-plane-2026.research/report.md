# Deep-research synthesis

## Executive finding

KnowMe should adopt UAR as the semantic owner of knowledge operations and learning-loop events, but not as one identical binary topology across every platform. The supportable path is service-first on desktop/web and a later extracted kernel on mobile. The core architectural rule is to keep canonical knowledge artifacts portable while allowing indexes and execution placement to differ.

## What is real today

UAR source provides a credible service-oriented RAG foundation: knowledge APIs, ingestion, multiple embedding backends, retrieval traits, vector/graph RRF, lexical verification, citations, and evaluation assets. Its Prometheus skill-system subtree provides a Markdown/TF-IDF wiki, MCP/UAR bridges, project scope resolution, trace types and storage, evaluation, a concrete trace-to-wiki compiler, and policy/optimization components.

KnowMe provides complementary mobile primitives: an embedded SurrealDB store, vector and BM25 retrieval, graph expansion, RRF, a typed embedder seam, Rust-owned FFI, an Axum server, and UAR event projection.

## What is not ready

The UAR SDK's embedded mode is not the narrow in-process runtime implied by the mobile goal. It starts the server path and does not populate the direct handles exposed by the SDK. The root runtime's feature profile is too broad for an assumed mobile embedding.

The self-learning path is partial rather than absent. Current source contradicts older missing-feature notes: trace abstractions and compilers exist. However, Codex/OpenCode adapters return no data, the Claude adapter does not fully correlate outputs/errors/durations, and the CLI compile path stops before calling the compiler. Current heuristic evaluation also cannot establish semantic task correctness.

## Architecture inference

The common unit across platforms should be a semantic contract—scope, artifact identity, provenance, retrieval events, citations, loop states—not a common process image. This yields:

- mobile: an extracted UAR knowledge kernel inside KnowMe's Rust library, using the existing graph store and surfaced by typed FFI;
- desktop: one supervised UAR service, with BossFang preferred as supervisor when present;
- web: a private UAR service behind KnowMe Axum, with no direct browser/runtime relationship.

Evidence documents, wiki lessons, and execution traces are separate because they have different trust, retention, and promotion rules. Canonical text, revisions, provenance, evaluations, and tombstones synchronize. Embeddings and indexes do not: their model and platform binding make them rebuildable projections.

## Learning-loop inference

The loop should be capture → redact → evaluate → compile candidate → lint/link/deduplicate → project-private wiki → observe outcomes. Shared-wiki promotion and skill optimization branch into distinct review gates. A successful trace alone is weak evidence; promotion needs independent examples, explicit task success, negative cases, security and privacy policy, sycophancy checks, regression tests, and reversible release.

## Evidence quality

External technical claims rely on original papers or official platform/protocol/database documentation. Local implementation claims rely on current pinned source rather than roadmap statements. The principal uncertainty is operational: no live service, mobile build, physical device, or performance benchmark was run because this KBD step is document-only.

The strict sycophancy detector found no classified pattern in the architecture thesis and returned 0.0. A Feynman/learn-grade tool was not available, so no numeric pedagogy grade is claimed. The published strategy instead exposes assumptions, rejected alternatives, blockers, and falsifiable certification gates.

## Recommended first evidence-producing slice

Begin with desktop/web service mode. Demonstrate scoped ingest, hybrid retrieval, citations, typed run events, complete native trace capture, evaluation, and project-private wiki candidate compilation. Keep promotion manual and skill optimization off. This slice exercises the supported UAR topology and produces the evidence needed to define the later mobile kernel rather than designing that kernel from hypothetical APIs.

## Sources

See [the source registry](sources/registry.json), [claim citations](citations.json), [evidence graph](graph.json), and [contradiction log](contradictions.json). The normative result is the [published strategy](../../README.md).
