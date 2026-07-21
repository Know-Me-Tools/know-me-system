# Research package index

Question: How should KnowMe use the UAR submodule for RAG, Karpathy wikis, and evaluated learning loops across mobile, desktop, and web?

## Result

The recommended architecture is one logical UAR knowledge control plane with three deployment profiles: an extracted mobile-safe kernel behind Rust FFI, one supervised desktop UAR service, and a private UAR service behind the KnowMe Axum BFF for web. Evidence corpus, Markdown wiki, and episodic traces remain separate. Canonical text/provenance sync; derived indexes rebuild locally.

The current source is capable enough for a service-first pilot but not for mobile embedding or autonomous skill improvement. In particular, UAR has concrete trace and compiler components, while generic trace adapters, CLI compile wiring, and a narrow embedded runtime remain incomplete.

## Package map

- [Research plan](plan.md)
- [Machine-readable plan](plan.json)
- [Source registry](sources/registry.json)
- [Evidence graph](graph.json)
- [Citation map](citations.json)
- [Contradictions](contradictions.json)
- [Synthesis report](report.md)
- [Published strategy](../../README.md)

The package records research state. The published strategy is the normative architecture artifact.
