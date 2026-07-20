# Supplemental assessment: UAR knowledge base, RAG, and Karpathy wikis

Project: KnowMe
Phase: `model-runtime-per-device`
Date: 2026-07-19
Artifact scope: architecture and documentation only

## Assessment outcome

Use one logical UAR knowledge control plane with platform-specific placement:

- mobile embeds a future mobile-safe UAR knowledge kernel in the shared Rust core and exposes it to Flutter through typed FFI;
- desktop runs one supervised UAR service, supervised by BossFang when available and by KnowMe otherwise;
- web runs UAR as a private service behind the KnowMe Axum application.

Keep the evidence/RAG corpus, human-readable Karpathy wiki, and episodic execution traces as separate artifact classes. Synchronize canonical text, revisions, provenance, evaluations, and tombstones. Rebuild embeddings and indexes per device/model binding.

The normative strategy is `docs/strategies/knowledge-base-karpathy-wikis/README.md`. The persistent research package is under that strategy's `research/` directory.

## IMPLEMENTATION STATUS

- UAR knowledge service: **SUBSTANTIAL / SERVICE-READY CANDIDATE** — current source has knowledge CRUD, ingestion, hybrid retrieval, embedding backends, verification, citations, and evaluation assets. This assessment did not run a live service certification.
- UAR embedded mobile runtime: **NOT READY** — the Rust SDK's embedded builder starts the full server path and direct runtime accessors are not populated. No narrow mobile-safe kernel/profile exists.
- KnowMe mobile retrieval substrate: **SUBSTANTIAL** — the Rust graph store already provides embedded SurrealDB, 384-dimensional embeddings, BM25, vector search, graph expansion, and RRF. It is the preferred mobile retrieval backend.
- KnowMe UAR integration: **PARTIAL** — non-mobile service calls and event projection exist. Mobile explicitly reports that embedded UAR is not wired.
- Karpathy wiki: **PARTIAL** — Markdown storage, TF-IDF focus, linting, MCP/UAR bridges, and trace-to-wiki compiler components exist in the UAR submodule.
- Cross-platform trace capture: **PARTIAL** — a real `TraceCapture` trait and Claude Code adapter exist. Generic Codex/OpenCode adapters return no data; output/error/tool-result correlation and durations remain incomplete.
- Learning CLI: **PARTIAL** — trace capture/storage/evaluation are wired, but the compile command does not yet load traces and call the available batch compiler.
- Autonomous optimization: **NOT SAFE TO ENABLE** — current evaluation is primarily structural/heuristic and cannot prove semantic task success. Wiki promotion and skill release need separate evidence and human gates.
- Web knowledge facade: **MISSING** — KnowMe Axum exposes chat/run surfaces but no tenant-scoped UAR knowledge facade.
- Desktop supervision: **DESIGNED, NOT CERTIFIED** — UAR documents service-first BossFang integration, but KnowMe has no verified single-writer discovery/lease evidence for this subject.

## CROSS-TOOL PROGRESS

- The active phase remains `litertlm-local-tools`, implementation 3 of 4.
- This assessment does not complete, add, or reorder an implementation change.
- Concurrent working-tree changes outside this strategy and KBD metadata were preserved.

## SPEC GAP SUMMARY

- The requested mobile placement assumes a complete embedded library, while current UAR source exposes only the full server-oriented runtime. A crate/feature extraction is required before FFI integration.
- Current UAR and KnowMe retrieval implementations overlap. Without a retrieval-backend adapter, mobile would duplicate SurrealDB state, embeddings, and indexes.
- Existing agent documentation contains a browser-WASM UAR statement that conflicts with the current UAR support matrix and this requested Axum service topology. The new knowledge strategy supersedes it for this subject.
- UAR future-work documents contain stale claims about missing trace/event/compression components. Current source must be treated as authoritative; those features remain partial or uncertified rather than uniformly missing.
- A portable identity/provenance/scope contract does not yet bind UAR knowledge records, Prometheus wiki entries, KnowMe graph records, and sync revisions.
- No explicit rule currently prevents BossFang and KnowMe from launching separate UAR writers against the same scope.
- Current trace evaluation can reward output presence without proving correctness; it is not a sufficient learning gate.
- Embedded-runtime licensing is materially different from using the MIT SDK as a client and requires a product/legal decision.

## BUILD HEALTH

- No build, test, runtime, or physical-device command was run because the requested output is documentation-only.
- Source and documentation were inspected at the currently initialized UAR submodule revision.
- JSON research artifacts and Markdown links require validation before handoff.

## CONSTRAINT CHECK

- Architecture keeps networking, persistence, retrieval, LLM calls, and learning-loop execution in Rust.
- Flutter remains a typed FFI presentation client.
- React remains behind hooks/stores and the Axum/Tauri Rust boundary.
- No application code was created or modified for this assessment.
- The KBD implementation and change counters remain unchanged.

## GOAL PROGRESS

- Cross-platform placement decision: **MET AT ARCHITECTURE LEVEL**.
- UAR/KnowMe capability and gap inventory: **MET**.
- RAG/wiki/episodic authority and sync model: **MET**.
- Karpathy loop with evaluation and promotion gates: **MET AT ARCHITECTURE LEVEL**.
- Runtime implementation and certification: **NOT STARTED BY THIS ASSESSMENT**.

## Required next evidence

1. Service-first desktop/web vertical slice: scoped ingest, retrieve, cite, project, trace, evaluate, and compile into a project-private review queue.
2. UAR-native trace adapter with complete tool outcome, duration, error, cancellation, and citation correlation.
3. Single-writer BossFang/KnowMe supervisor contract and restart evidence.
4. Mobile kernel dependency/feature audit and AGPL distribution decision.
5. Mobile retrieval adapter design against `gen_ui_db_graph`, followed later by physical-device certification.

## ASSESSMENT COMPLETE

This supplemental assessment is complete. It does not assert that the KBD phase or active implementation change is complete.
