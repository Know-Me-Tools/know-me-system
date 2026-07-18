# KnowMe — Documentation Index

## Product specifications (canonical)

| Doc | What it is |
|---|---|
| `knowme-functional-spec.html` | **FUNC-SPEC v1.0** — the full functional specification & architecture: overview, philosophy, personas, the eight tiles, 7-layer architecture, Hands system, BossFang/OFP sync, plugin system, privacy & trust, tiers, roadmap. Also pinned on IPFS: [`QmXFPBMvNw6jgbTmtt6TZszouJWxERbnDF7XpDfkkTUcFf`](https://ipfs.prometheusags.ai/ipfs/QmXFPBMvNw6jgbTmtt6TZszouJWxERbnDF7XpDfkkTUcFf) |
| `knowme-moodboard-user-journeys.html` | **UX-MOOD v1.0** — moodboard & user-journey atlas: 8 screens across desktop/tablet/phone with per-surface narratives, plus three end-to-end persona journeys |
| `implementation-roadmap.md` | Spec → module map for this monorepo and the M1–M7 gap-closure plan |
| `strategies/local-models/README.md` | **Local model acquisition & management** — onboarding flow, per-platform hardware detection (unified memory, VRAM, jetsam, WebGPU), KnowMe-controlled origin (R2/CDN + IPFS + OFP peer) vs. Hugging Face fallback, license/auth model, resumable downloads per tier, five-gate post-download validation |
| `strategies/model-configuration/README.md` | **Provider & model configuration** — Cherry-Studio-informed three-level resolution (global role slots → agent/Hand pin with deployment-target scoping → conversation override + @mention), `provider::model` registry with capability precedence, multi-device placement walk, per-index embedding bindings with re-embed migrations, local-only usage-based suggestions |
| `knowme-builder-extensibility-spec.md` | **DRAFT v0.1** — KnowMe Builder local-first × realtime × WASM extensibility: triad sync topology (Electric/FRF read path · intent log · Loro CRDT), unified `prometheus:component/*` WIT family, settings-as-schema model, `.pxt` packaging (IPFS+OCI, DID+cosign), W1–W8 plan, open questions for the deep-research report |

## Engineering standards & references (carried from the reference repo)

| Doc | What it is |
|---|---|
| `tj-arch-mob-001.html` | TJ-ARCH-MOB-001 — hybrid mobile architecture standard (platform selection, state management, enforcement rules) |
| `knowme-ui-ux-standard.md` | Repository UI/UX standard: Shadcn UI, Assistant UI, borderless Flat 2.0, AG-UI events, matched React/Flutter themes |
| `gen_ui_spec.html` | gen_ui core specification |
| `KnowMe.dc.html` | KnowMe design/concept document from the POC |
| `assessment-2026-07-16.md` | 2026-07-16 architecture assessment (referenced by the plugin v0.2 deferrals, rec #9) |
| `competitive-analysis-2026-07-16.md` | Competitive analysis |
| `corrections-2026-07-16.md` | Corrections log |
