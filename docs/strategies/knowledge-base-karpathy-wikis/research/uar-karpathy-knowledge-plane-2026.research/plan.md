# Research plan

## Question

How can the UAR submodule provide RAG, Karpathy wikis, and evaluated learning loops across embedded Flutter mobile, supervised/BossFang desktop, and Axum-served React web without duplicating KnowMe persistence or violating the Rust-core boundary?

## Subquestions

1. Which UAR knowledge, RAG, trace, wiki, and SDK capabilities exist in current source, and which are stubs?
2. Which KnowMe stores, FFI seams, event projections, and service boundaries can be reused?
3. What belongs in the canonical evidence corpus, wiki, and episodic store?
4. Which deployment topology is supportable per platform?
5. How should canonical artifacts synchronize when embedding models and device capabilities differ?
6. Which evaluation, privacy, licensing, and promotion gates prevent unsafe self-improvement?

## Method

- Inspect current KnowMe and pinned UAR source as primary implementation evidence.
- Prefer original papers and official platform/protocol/database documentation for external claims.
- Record stale or conflicting local documentation as contradictions rather than averaging it with current source.
- Derive architecture from common semantic contracts and platform-specific placement constraints.
- Run a strict sycophancy check on the core recommendation.
- Publish an auditable strategy and retain the research registry/graph.

## Completion criteria

- Each platform has an explicit runtime owner, process boundary, persistence authority, and UI boundary.
- Current capabilities and blockers are distinguished.
- Synchronization excludes derived indexes and records model provenance.
- The learning loop includes evidence, reversibility, evaluation, and human promotion gates.
- No application code is added or modified.
