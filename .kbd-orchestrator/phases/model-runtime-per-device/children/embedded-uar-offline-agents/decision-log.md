# Decision Log — embedded-uar-offline-agents

## D-001 · UAR core owns the embedded agent kernel          [analyze · 2026-07-20]

**TL;DR:** Repair the UAR library so embedded and HTTP deployments execute the same runtime semantics.

**Why:** KnowMe must not duplicate agent, skill, tool, policy, memory, AG-UI, or A2UI behavior. UAR already owns those contracts, but its SDK embedded builder currently returns no state.

**Alternatives:** Keep HTTP on mobile (rejected: violates offline requirement) · Reimplement agents in KnowMe (rejected: two authorities and behavioral drift)

**Learn more:** `analysis.md` and candidate `cand-001` in `library-candidates.json`.

---

## D-002 · One mobile kernel with injected platform model drivers [analyze · 2026-07-20]

**TL;DR:** Android and iOS share the identical UAR kernel; Android injects LiteRT-LM and iOS injects Metal inference.

**Why:** The agent runtime is platform-neutral Rust. Only model loading/generation is hardware-specific, and UAR already has a host `LlmDriver` seam.

**Alternatives:** Separate Android/iOS UAR implementations (rejected: divergence) · Cloud UAR on iOS (rejected: network dependency)

**Learn more:** candidates `cand-002`, `cand-003`, and `cand-004`.

---

## D-003 · HTTP is an optional transport adapter             [analyze · 2026-07-20]

**TL;DR:** Direct Rust operations are primary for embedded apps; Axum/SSE wraps the same application services for remote clients.

**Why:** Transport choice must not determine runtime capability or data ownership. Direct event streams also remove needless loopback networking and serialization on mobile.

**Alternatives:** Run an in-process loopback server (rejected: needless ports, lifecycle, and security surface) · SDK HTTP client only (rejected: not embedding)

**Learn more:** UAR `RunManager`, normalized events, and A2UI replay backbone.
