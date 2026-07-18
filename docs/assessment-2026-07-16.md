# TJ-ARCH-MOB-001 / Hybrid Mobile Architecture Skill Pack — Independent Deep Assessment

**Prepared for:** Travis James, Prometheus AGS / KnowMe, LLC
**Date:** 2026-07-16
**Method:** Full read of `docs/` (TJ-ARCH-MOB-001, gen_ui spec, pglite-oxide notes, KnowMe reference docs, PoC plan) plus repo-level SKILL.md / CLAUDE.md / AGENT_BASE_RULES.md / README.md, followed by a fan-out deep-research pass (104 agents, 22 primary sources fetched, 107 claims extracted, 25 adversarially verified by 3-voter panels, 1 refuted). Evaluative sections were passed through the sycophancy-correction skill before delivery. Confidence labels below come from the verification votes.

---

## 1. Executive verdict

**The pattern is sound. Several of the component bets are not — and the biggest risk in the whole program is not any single technology, it's maintenance surface divided by team size.**

Broken into the four questions you asked:

**Is what you're trying to do a good idea?** Yes, conditionally. One shared Rust core under a Flutter mobile shell and a Tauri desktop shell is a real, production-proven pattern (1Password, AppFlowy — §3.1), and packaging your architecture as an agent-executable skill pack is well-timed: the scaffold→audit→rules-propagation loop is exactly where agentic development is heading, and almost nobody has a pack of this depth yet. The condition: the precedents show this pattern consumes a *permanent* engineering function, not a one-time scaffold. 1Password runs a dedicated squad just to own the bridge layer between native shells and the Rust core, for one product. You are proposing the same pattern across three products plus the skill pack itself, presumably with a much smaller team. That ratio, not any framework choice, is the thing most likely to sink this.

**Is the architecture good?** The skeleton is genuinely good: the Flutter-mobile/Tauri-desktop split is well-argued and matches independent evidence; the ContentBlock sealed-union contract, single-Tokio-runtime threading model, and intent-level FFI seams are solid engineering. But three specific component bets are weaker than the skeleton: the mistral.rs/candle mobile inference lane (no documented mobile path anywhere, ecosystem runs on llama.cpp — high confidence), ElectricSQL as the sync layer (its own rewrite explicitly dropped write-path sync — high confidence), and the four-engine database matrix on top of a pre-1.0 pglite-oxide (§3.3–3.5).

**Is the execution plan as stated correct?** Mostly yes in structure, too broad in scope. The wave ordering is right and the C-103 iOS-simulator milestone is correctly identified as the highest-information next step. But the PoC's MUST set alone (142-provider gateway + graph-RAG + local-first sync + local inference on three platforms + a Postgres-semantics config DB) is four or five products' worth of integration risk, and your own C-102 run — ~25 real defects in the first codegen pass — is direct evidence of how much friction each additional subsystem buys.

**What impact could the skill package have?** Potentially significant as an internal velocity multiplier and consulting asset, with one hard prerequisite: the pack's authority documents currently contradict each other (§5), including one actively harmful stale claim in CLAUDE.md. A skill pack whose entire value proposition is deterministic guidance to agents cannot ship with divergent authorities — agents will faithfully implement whichever wrong version they read first.

---

## 2. What you're building (steelman)

To assess it honestly, it's worth stating the strongest version of the thesis, because it is strong:

Write all infrastructure — networking, LLM gateway, inference, MCP, agent loop, persistence — once, in Rust, where it is fast, safe, and portable. Let each platform get the rendering technology that structurally wins there: Flutter where bundled-engine rendering and gesture ownership matter (consumer mobile, regulated mobile), the web ecosystem where component richness and artifact rendering matter (desktop, web). Enforce the boundary with compiler-checked sealed unions on every surface so the protocol cannot drift. Then encode the whole thing — decision authority, scaffolds, codegen, audits, and 40 rules of agent conduct — as a skill pack, so that both humans and coding agents produce compliant systems by default, and every defect found in generated output flows back into the generator. Use it to ship three products and arm a consulting practice.

That is a coherent, differentiated thesis. Nothing in the research contradicts the thesis itself. What the research does is grade the individual bets underneath it.

---

## 3. Component-by-component findings

### 3.1 Shared Rust core + multiple shells — **sound, but budget for it** (high confidence)

- **1Password** ships a single Rust core consumed by Swift, Kotlin, and TypeScript/web shells at millions-of-users scale — and maintains a dedicated Core Platform squad whose job is the async bridge layer, plus purpose-built tooling ([Typeshare](https://github.com/1Password/typeshare)) just to keep types synchronized across language boundaries ([corrode.dev interview](https://corrode.dev/podcast/s04e06-1password/)).
- **AppFlowy** is the closest analog to your Flutter half: Flutter over a Rust core via Dart FFI, with explicit portability intent later validated by a Tauri/React reuse. Their own engineering blog flags that FFI serialization cost "will worsen along with the business growth" ([AppFlowy tech design](https://appflowy.com/blog/tech-design-flutter-rust)) — that caveat applies to their protobuf FFI, not flutter_rust_bridge specifically, but the structural warning stands.
- **flutter_rust_bridge** is past experimental: stable 2.x line (2.12.0 current), Flutter Favorite since the program's 3.16 reboot ([Flutter 3.16 announcement](https://medium.com/flutter/whats-new-in-flutter-3-16-dba6cb1015d1)). **However**, the verifier panel *refuted* (1–2 vote) the claim that frb demonstrably supports the full six-target matrix plus the async-Rust/Rust-calls-Dart patterns gen_ui_core's streaming pipeline depends on. Your plan already treats the first on-device iOS run as the next milestone — the evidence says that instinct is exactly right, and nothing else should be trusted until it happens.

**Implication:** the pattern is viable but is a payroll line, not a scaffold. Before committing three products to two shells, name the person (or fraction of a person, or agent-run process) who owns the bridge layer indefinitely.

### 3.2 Flutter mobile / Tauri desktop split — **your strongest call** (high confidence)

The research independently corroborates TJ-ARCH-MOB-001's central claims. Tauri 2.0 does officially support iOS/Android, but the Tauri team itself named mobile DX a top post-release improvement priority ([Tauri 2.0 release post](https://v2.tauri.app/blog/tauri-20/)), and WebView fragmentation is inherent by design — four different system engines, macOS/iOS engines frozen to OS versions, and Tauri's own docs conceding the Linux WebKitGTK spread is "very hard to compile accurate information" about ([webview versions reference](https://v2.tauri.app/reference/webview-versions/)). Your standard's refusal to put consumer mobile on Tauri, and its acceptance of the QA cost on desktop where the fragmentation is manageable, is exactly what the evidence supports. Note the corollary: this evidence also cuts against any future fallback plan of consolidating mobile onto Tauri to save the Flutter surface. The split is load-bearing; plan as if it's permanent.

### 3.3 Mobile inference lane (mistral.rs / candle) — **weakest bet in the stack** (high confidence)

- mistral.rs is pre-1.0 (0.9.0, July 2026, after rapid 0.8.x churn), documents installation and acceleration only for Linux/macOS/Windows, and its README contains **no iOS/Android or mobile deployment claims at all**. Its published benchmarks are vendor-run on datacenter GPUs ([mistral.rs repo](https://github.com/EricLBuehler/mistral.rs)).
- A comprehensive curated catalog of the mobile-LLM ecosystem (updated June 2026) lists llama.cpp as the foundational engine with multiple shipped mobile apps on it (LLMFarm, LLM.swift, Sherpa), plus MLC-LLM, ExecuTorch, MediaPipe, MNN — and contains **zero mentions of mistral.rs, candle, Flutter, or Rust** ([awesome-mobile-llm](https://github.com/stevelaskaridis/awesome-mobile-llm)). Medium confidence (single curated source, 2–1 vote), but it converges with the README silence.
- Your own documents already know part of this: the PoC plan excludes vision because "candle Metal is broken on iOS (candle#1841)," and the KnowMe functional spec (May 2026) specified **llama.cpp on mobile** before the July revision switched to mistral.rs.

Choosing candle/mistral.rs for mobile means volunteering to be the first known project to ship that path, inside a PoC whose purpose is to de-risk everything else. An undocumented cross-compile path is a research project, not a dependency. The functional spec's own `InferenceProvider` trait abstraction is the right seam: put `llama-cpp-2` behind it on mobile (S4 "sovereign mode"), keep mistral.rs on desktop Metal where it is documented, and revisit when mistral.rs publishes a mobile story.

### 3.4 Sync layer (ElectricSQL) — **usable only with eyes open** (high confidence)

ElectricSQL discarded its original vertically-integrated local-first stack in July 2024 after early adopters hit performance and reliability problems ("the complexity of the stack has provided a wide surface for bugs"), and rebuilt as a deliberately narrow, **read-path-only** sync engine. Local writes through the database, active-active replication, and the DDLX permissions system were all dropped; current docs still state Electric does not do write-path sync ([electric-next announcement](https://electric-sql.com/blog/2024/07/17/electric-next), [writes guide](https://electric-sql.com/docs/guides/writes)).

Your PoC plan already says "Electric read-path + DIY write queue," which is honest. The assessment point is sharper than the plan acknowledges: **the sync layer of your architecture is custom software you are writing**, with Electric as a read cache — plan and staff it as such, don't present it (even to yourselves) as an integrated dependency. And there's a second, uncomfortable lesson in that source: the one directly comparable vertically-integrated local-first platform to TJ-ARCH-MOB-001's full vision *collapsed under its own complexity* and survived only by rebuilding narrow. That is the strongest single piece of evidence in this entire report for trimming the matrix.

### 3.5 Database matrix — **overbuilt for the stage** (high confidence on components, medium on the judgment)

Count the engines a PoC-stage team is committing to operate, migrate, and test: pglite-oxide (desktop), PGlite WASM (web), SQLite+sqlite-vec (mobile), SurrealDB embedded (all tiers), and cloud Postgres/Supabase — five engines, two SQL dialects plus SurrealQL.

- **PGlite** grew from ~500k to 13M+ weekly downloads in a year (figure inflated by Prisma CLI bundling), but it fundamentally runs Postgres in single-user, single-connection mode — v0.4's multiplexing simulates concurrency over one connection — and v0.4 (March 2026) shipped a major internal refactor, i.e., core internals were churning four months ago ([PGlite v0.4 announcement](https://electric.ax/blog/2026/03/25/announcing-pglite-v04)). For an agent runtime doing concurrent local writes (agent loop + UI + sync queue), single-connection semantics are a real constraint, not a footnote.
- **pglite-oxide** (0.5.1, June 2026) adds an unverified Rust host layer *on top of* that moving target. Your corrected doc scopes it accurately — desktop only, WASM guest, not native. Fine as a desktop config-DB bet; wrong as anything more, and (§5) your CLAUDE.md still says otherwise.
- **SurrealDB embedded** produced *no verified production-readiness evidence at all* in the research pass — silence, not clearance. Meanwhile its own tracker documents severe 3.0 performance regressions in embedded/RocksDB configurations — e.g., a simple indexed WHERE going from 1ms (2.4) to ~2000ms (3.0), alongside big wins on other query shapes ([surrealdb#6800](https://github.com/surrealdb/surrealdb/issues/6800)) — plus open abnormal-memory reports ([surrealdb#5541](https://github.com/surrealdb/surrealdb/issues/5541)). You've pinned 3.2 as the graph-RAG spine on *mobile RAM budgets* — your architecture's "biggest differentiator" (your words) is resting on the least-verified engine in the stack. That demands a representative on-device benchmark gate before C-104, and a named fallback (sqlite-vec + FTS5 + recursive CTEs covers a useful subset).

The repository-trait seam is the right abstraction. But an abstraction doesn't pay the per-engine costs: migrations, CI matrices, corruption/upgrade paths, and dialect drift each remain per-engine. For the PoC, one relational engine per tier plus *one* graph/vector engine — gated by benchmarks — is defensible; five is not.

### 3.6 KnowMe plugin system (Wasmtime + WIT + IPFS) — **precedented core, narrow it** (corroborated)

There is now a strong shipped precedent: **Zed** runs Rust→WASM extensions under Wasmtime with WIT contracts in production since 2024 ([Zed: Life of an Extension](https://zed.dev/blog/zed-decoded-extensions)). The instructive part is what Zed did *not* do: the API is deliberately restricted (languages, themes, snippets, slash commands — "no support for modifying the UI... or touching the file system how you want"). KnowMe's proposed v1 surface (UAR skill invocation, storage, network-by-proxy, navigation injection, iframe UI, IPFS distribution, Ed25519 signing, Cedar governance) is dramatically wider than what the only successful consumer-app precedent shipped. The architecture is right; the v0.1 WIT surface should be a third of the spec'd one.

### 3.7 The sovereign-AI market thesis — **unvalidated** (no surviving evidence)

The research pass found **no verified evidence** of paying demand at premium price points for sovereign personal AI — no traction, pricing, or revenue benchmarks for KnowMe-comparable products survived verification. That is absence of evidence, not evidence of absence: the crowded local-LLM tool landscape (Jan, LM Studio, Ollama — all free) shows strong *usage* demand for local inference, but those tools' economics also mean the $0 tier of this market is extremely well-served. The $200/mo tier's load-bearing assumption — that a licensed local Qwen-class model plus packaging justifies 10× the standard consumer AI price — is a business hypothesis with no market comps in evidence. Also flag: Qwen weights are permissively licensed, so be precise internally about what the customer is actually paying for (TurboQuant packaging? support? signing?) before building tier machinery. This is the kind of assumption to validate with a landing page and 50 conversations, not with 13 crates.

---

## 4. The execution plan, graded

**Right:** wave ordering; scaffold-first discipline (defects fixed at source — the ~25-defect C-102 ledger is the single most convincing artifact in the repo); the honest WON'T list (vision, mobile cron); the airplane-mode demo narrative (a story, not a tile grid — correct read of why Wonderous beat Flutter Gallery); harness scorecarding (claude 8/8, codex 2/2, opencode 0/2 → no opencode lanes) — that's evidence-driven agent ops, which almost nobody does.

**Wrong-sized:** the MUST set. M1 (142-provider gateway) + M2 (hybrid graph-RAG with HNSW+BM25+graph traversal) + M3 (local-first sync with DIY write path) + M4 (local inference on desktop Metal *and* WebGPU web) + M5 (Postgres-semantics config DB on every target) is a portfolio, not a PoC. Two concrete trims that don't damage the demo narrative:

1. **M5:** the config DB does not need Postgres semantics to store provider rows and model prefs. SQLite (mobile/desktop) + keychain refs covers the demo; pglite-oxide can land later as a desktop upgrade. This removes the newest, least-verified dependency from the critical path.
2. **M4's web lane (WebLLM)** is already an honest documented exception to the Rust-core invariant — but it's a *third* inference integration inside one PoC. Feature-gate it to "if time remains after C-106," not a MUST-adjacent commitment.

And one addition: **benchmark gates as explicit plan items.** iOS frb streaming throughput (C-103), SurrealDB 3.2 on-device graph-RAG latency/memory (before C-104), mobile llama.cpp tok/s (C-105/S4). Each gate has a pre-named fallback. The plan currently verifies *that things build*; these verify *that the architecture's claims are true on hardware*.

---

## 5. Internal consistency audit — the pack contradicts itself

This matters more for you than for a normal repo, because this repo's *product is authoritative instruction to agents*. Found in the current tree:

| Contradiction | Where | Severity |
|---|---|---|
| CLAUDE.md still describes pglite-oxide as "a real PostgreSQL 17.5 binary... Rust-native" and routes **Mobile** → pglite-oxide | CLAUDE.md §Embedded PostgreSQL vs. `docs/pglite-oxide-tauri-hybrid.md` (corrected 2026-07-15, explicitly debunks both claims) | **High — actively harmful.** Agents read CLAUDE.md first and will architect mobile against a dependency with no iOS/Android runtime. |
| Inference engine churn: candle (gen_ui_spec §9, SKILL.md step 4) → candle-vllm/llama.cpp (KnowMe functional spec §5) → mistral.rs + WebLLM (PoC plan §3.5) | three docs, all currently in-tree | High — three authorities give three different answers to "what runs inference." |
| Version skew: Riverpod 2.x (SKILL.md) vs 2.6+ (CLAUDE.md) vs 3.3 (README/PoC); Node 22+ vs 24+; Flutter 3.29+ vs beta channel; frb 2.3+ vs 2.12+; Vite 7 (TJ-ARCH) vs Vite 8 (PoC); SurrealDB 2.x → 3.0.5 → 3.2 | SKILL.md / CLAUDE.md / README / docs | Medium — violates your own Rules 22/23 inside the artifact that enforces them. |
| gen_ui_spec describes a monolithic `gen_ui_core` crate with `api.rs`; README/PoC describe the 13-crate layered workspace | docs/gen_ui_spec.html vs README | Medium — the flagship spec predates the workspace refactor. |

The fix is structural, not editorial: designate one generated source of truth for stack versions and engine choices (a `versions.toml` or the arch standard), generate the CLAUDE.md/SKILL.md sections from it, and add a **doc-consistency check to `audit.sh`** so drift fails CI. A skill pack that can audit scaffolded projects but not itself is missing its most important audit target.

---

## 6. Impact of the pack + Prometheus Skill System, and the full usage surface

### Impact assessment

Combined with the Prometheus Skill System (self-improving loops, `pk` knowledge base, activation hooks raising skill-trigger rates), the pack is three things at once:

1. **An internal velocity multiplier** — the defect-flows-back-to-scaffold discipline means every project makes the next one cheaper. This is the highest-probability payoff and is already demonstrably working (C-102).
2. **A consulting asset** — for Prometheus AGS engagements, "we scaffold a compliant, audited, agent-maintainable hybrid AI app in days" is a differentiated pitch, and `audit.sh` doubles as a paid assessment tool for client codebases.
3. **A distributable product** — via agentskills.io / Claude Code marketplaces. Honest caveat: the research found no verified evidence yet that third-party scaffold-generator skill packs achieve meaningful adoption — you'd be early, which cuts both ways.

The main threat to all three is the same: **currency decay**. The pack hard-pins a fast-moving frontier (TS 7, Flutter beta, Riverpod 3.3, Tauri 2.10, pre-1.0 inference and DB crates). Without the doc-lint + a scheduled re-verification loop (the skill system's self-improving machinery is the natural home for this), the pack's guidance rots in months — §5 shows the rot has already started.

### All the ways the toolset can be used

Greenfield: full hybrid scaffold (three surfaces + 13-crate workspace); single-surface scaffolds (Flutter-only, Tauri-only, Rust-core-only); publishable-package scaffolds (crates.io / pub.dev / npm SDK family that third parties can consume without ever seeing the pack). Brownfield: add gen_ui_core to an existing Flutter app; add A2UI/AG-UI streaming to an existing React/Tauri app; retrofit feature-based clean architecture; add Kratos/Supabase auth. Codegen: new feature modules; new ContentBlock variants full-stack (Rust enum → Dart/TS union → widget/component); MCP server integrations; DB stores. Governance: AGENT_BASE_RULES as an org-wide agent-conduct layer propagated into every generated project, across four harnesses (Claude Code, Codex, OpenCode, Kimi); `audit.sh` as CI gate and as a consulting deliverable; harness scorecarding as an ongoing eval harness for choosing coding agents. Operations: `check-env.sh` four-pillar bootstrap for onboarding and CI images; project-local UI/UX skills (design tokens, golden tests, a11y-gate) reusable outside the hybrid architecture entirely. Products: KnowMe (Flutter mobile + Tauri desktop), Prometheus AGS platform (Tauri primary + field mobile), TribeHealth (Flutter, regulated), each seeding scaffold improvements back into the pack; the KnowMe PoC as a sales/demo asset; UAR external mode turning scaffolded apps into clients of shared enterprise agent infrastructure.

That is a genuinely large surface — which is the point, and the risk. Every one of those uses draws on the same maintenance budget.

---

## 7. Prioritized recommendations

1. **Fix CLAUDE.md's pglite-oxide section today.** It instructs agents to do something physically impossible on mobile. (Minutes of work, removes the single most dangerous artifact in the repo.)
2. **Unify stack authority** — one generated source for versions/engines; add doc-consistency to `audit.sh`; make the pack audit itself in CI.
3. **Run C-103's iOS on-device frb streaming test before any further breadth.** The refuted frb claim (§3.1) makes this the program's highest-information action. Already your plan; the research just underlines it.
4. **Swap the mobile inference lane to llama.cpp (`llama-cpp-2`) behind the existing `InferenceProvider` trait.** Keep mistral.rs on desktop Metal. Revisit when mistral.rs documents mobile.
5. **Benchmark-gate SurrealDB 3.2 embedded on-device** (graph-RAG workload, mobile RAM) before C-104; name the fallback in the plan.
6. **Collapse the PoC database matrix**: SQLite everywhere the demo allows; pglite-oxide as a post-PoC desktop upgrade; treat the five-engine matrix as the v2 target, not the v1 floor.
7. **Reclassify sync as first-class custom code** (Electric = read cache) in the plan's language and estimates, not just its implementation notes.
8. **Validate the $200/mo sovereign-AI tier with buyers before building tier machinery.** No market evidence currently exists in either direction; that's a cheap experiment and an expensive assumption.
9. **Narrow KnowMe's v0.1 WIT plugin surface** toward Zed-scale (few, typed, boring capabilities), expanding only with ecosystem pull.
10. **Budget the bridge layer as a permanent function** — the 1Password lesson. If no one owns it, the two-shell strategy silently becomes two codebases.

---

## 8. Sources

**Repo documents (primary inputs):** `docs/tj-arch-mob-001.html`, `docs/gen_ui_spec.html`, `docs/pglite-oxide-tauri-hybrid.md`, `docs/reference-app/knowme-functional-specification-architecture.html`, `docs/reference-app/knowme-moodboard-user-journeys.html`, `docs/reference-app/knowme-poc-architecture-and-implementation-plan.md`, `SKILL.md`, `CLAUDE.md`, `AGENT_BASE_RULES.md`, `README.md`.

**Verified web sources:** [corrode.dev — 1Password's Rust core (S04E06)](https://corrode.dev/podcast/s04e06-1password/) · [1Password Typeshare](https://github.com/1Password/typeshare) · [AppFlowy tech design: Flutter + Rust](https://appflowy.com/blog/tech-design-flutter-rust) · [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) · [Flutter 3.16 / Flutter Favorites reboot](https://medium.com/flutter/whats-new-in-flutter-3-16-dba6cb1015d1) · [Flutter Favorites program](https://docs.flutter.dev/packages-and-plugins/favorites) · [Tauri 2.0 release](https://v2.tauri.app/blog/tauri-20/) · [Tauri webview versions reference](https://v2.tauri.app/reference/webview-versions/) · [Tauri × Verso](https://v2.tauri.app/blog/tauri-verso-integration/) · [mistral.rs](https://github.com/EricLBuehler/mistral.rs) · [awesome-mobile-llm (June 2026)](https://github.com/stevelaskaridis/awesome-mobile-llm) · [ElectricSQL: electric-next](https://electric-sql.com/blog/2024/07/17/electric-next) · [ElectricSQL writes guide](https://electric-sql.com/docs/guides/writes) · [PGlite v0.4 announcement](https://electric.ax/blog/2026/03/25/announcing-pglite-v04) · [SurrealDB #6800 — 3.0 performance regressions](https://github.com/surrealdb/surrealdb/issues/6800) · [SurrealDB #5541 — abnormal memory](https://github.com/surrealdb/surrealdb/issues/5541) · [Zed: Life of an Extension (Rust, WIT, Wasm)](https://zed.dev/blog/zed-decoded-extensions).

**Method note:** one claim was refuted in adversarial verification (frb full six-target + async matrix support) and is treated above as *unproven*, not false. Coverage gaps that produced no verified claims — SurrealDB embedded production readiness (partially covered by issue-tracker evidence found separately), sovereign-AI market demand, and scaffold-skill-pack adoption — are labeled as open questions in the text rather than silently assumed safe. Time-sensitivity is high across the stack (mistral.rs 0.8→0.9 in one month; PGlite v0.4 four months old); component conclusions should be re-checked before commitments later than Q4 2026.
