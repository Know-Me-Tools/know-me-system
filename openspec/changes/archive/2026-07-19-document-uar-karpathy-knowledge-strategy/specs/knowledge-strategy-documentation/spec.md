## ADDED Requirements

### Requirement: Normative cross-platform knowledge strategy

The project SHALL maintain
`docs/strategies/knowledge-base-karpathy-wikis/README.md` as the normative
architecture strategy for UAR-backed RAG, Karpathy wikis, and evaluated
learning loops across mobile, desktop, and web.

#### Scenario: A reader determines runtime placement

- **WHEN** a reader consults the strategy for a supported platform
- **THEN** the document identifies the runtime/process owner, persistence
  authority, UI boundary, degraded mode, and known blockers for that platform
- **AND** mobile uses an embedded Rust-library boundary through FFI, desktop
  uses one supervised UAR service, and web uses UAR behind KnowMe Axum

### Requirement: Separate knowledge artifact authorities

The strategy SHALL distinguish the evidence corpus, human-readable Karpathy
wiki, and episodic execution traces, including their provenance, retention,
synchronization, promotion, and derived-index behavior.

#### Scenario: Knowledge synchronizes across different embedding models

- **WHEN** two authorized peers use different embedding model bindings
- **THEN** the documented architecture synchronizes canonical text, revisions,
  provenance, evaluation records, and tombstones
- **AND** it requires each peer to rebuild embeddings and indexes locally

### Requirement: Evidence-backed and auditable documentation

The strategy SHALL retain a persistent research package containing its plan,
source registry, citation map, evidence graph, contradiction log, synthesis
report, and manifest.

#### Scenario: A reviewer audits an implementation-status claim

- **WHEN** the strategy claims that a KnowMe or UAR capability exists, is
  partial, or is missing
- **THEN** the reviewer can trace the claim to current repository source
- **AND** stale roadmap prose is recorded as a contradiction when it disagrees
  with current source

### Requirement: Document-only change boundary

This OpenSpec change SHALL modify only architecture, research, specification,
and KBD documentation artifacts.

#### Scenario: The document change is verified

- **WHEN** the OpenSpec change is ready for verification
- **THEN** its affected-file set contains no Rust, Dart, TypeScript, database
  schema, dependency, build, deployment, or runtime configuration changes
- **AND** JSON syntax, local Markdown links, OpenSpec structure, KBD progress
  invariants, and sycophancy screening pass

### Requirement: Human-gated learning promotion

The strategy SHALL document wiki promotion and executable skill optimization
as separate, reversible approvals backed by explicit evaluation evidence.

#### Scenario: A successful execution trace produces a candidate lesson

- **WHEN** an execution trace passes initial deterministic evaluation
- **THEN** the architecture permits creation of a project-private wiki candidate
- **BUT** it does not treat that trace as sufficient evidence for shared-wiki
  promotion or automatic production-skill modification
