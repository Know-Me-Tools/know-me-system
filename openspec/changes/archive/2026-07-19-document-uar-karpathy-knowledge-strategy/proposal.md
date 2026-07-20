# Change proposal: document UAR knowledge and Karpathy wiki strategy

## Why

KnowMe needs one evidence-backed architecture document describing how UAR owns
RAG, Karpathy wikis, and evaluated learning loops across mobile, desktop, and
web. Existing source has useful but unevenly mature components, and existing
strategy prose contains at least one deployment contradiction. Without a
normative document, later implementation could duplicate persistence, run two
desktop writers, assume unsupported browser WASM, or enable unsafe automatic
learning.

The KBD assessment already produced a working draft. This change treats that
draft as input to verify and finalize, not as evidence that runtime behavior is
implemented.

## What changes

- Finalize `docs/strategies/knowledge-base-karpathy-wikis/README.md` as the
  normative subject strategy.
- Finalize its persistent `.research` package with source, citation, evidence,
  contradiction, planning, and synthesis artifacts.
- Preserve a traceable distinction among current capability, recommendation,
  blocker, open decision, and future delivery stage.
- Record platform placement, artifact authority, synchronization, privacy,
  licensing, evaluation, and human-promotion gates.
- Add only the minimum supersession note needed where an older strategy
  conflicts with the new document.

## Non-goals

- No UAR, BossFang, Axum, Flutter, React, Rust, Dart, or TypeScript runtime work.
- No database, schema, dependency, build, deployment, or configuration change.
- No physical-device, sidecar, service, retrieval, or performance certification.
- No legal conclusion about AGPL distribution.
- No automatic wiki promotion or skill optimization.

## Success

The document is coherent, source-grounded, cross-platform, locally link-valid,
machine-artifact-valid, explicit about contradictions and uncertainty, and its
change set contains no application code.
