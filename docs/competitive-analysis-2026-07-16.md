# Competitive Analysis — TJ-ARCH-MOB-001 vs. the Agentic Software Landscape

**Prepared for:** Travis James, Prometheus AGS / KnowMe, LLC
**Date:** 2026-07-16
**Scope:** Production-certified approaches across the agentic software world — open and closed source — compared against the Prometheus AGS hybrid architecture (shared Rust core + Flutter mobile + Tauri/React desktop), the KnowMe product thesis, and the skill pack itself.
**Method:** Four parallel research passes (agent frameworks/SDKs; agent↔UI protocols; local/on-device AI products; cross-platform architectures + scaffold ecosystems), all facts sourced from official repos, vendor announcements, and press with URLs; figures dated; unverifiable claims marked. Evaluative sections passed through the sycophancy-correction skill.

---

## 1. Comparison framework

Twelve points of comparison, chosen because they are where production outcomes in this space have actually been decided:

1. **Primary surface** — server, web, desktop, mobile, on-device
2. **Agent runtime placement** — cloud-hosted, your-server, in-process/on-device
3. **UI/protocol layer** — how agent output reaches humans (MCP Apps, AG-UI, A2UI, proprietary)
4. **Language & core stack**
5. **Local/on-device inference** — first-class, adapter-only, or none
6. **Memory & persistence** — built-in, pluggable, or DIY
7. **Tool integration** — MCP support and direction (client/server)
8. **Orchestration** — single agent loop vs. graphs/multi-agent
9. **Traction** — verified stars/downloads/adopters, July 2026
10. **Governance & license** — foundation, vendor, community
11. **Monetization** — what actually earns revenue in this category
12. **Agent-assisted development** — scaffolds, skills, rules, spec-driven workflows

---

## 2. Cluster A — Agent frameworks & SDKs (the runtime competition)

### The certified-successful set (July 2026)

| Framework | Owner | Lang | Runtime placement | MCP | Local inference | Memory | Traction (verified) | License |
|---|---|---|---|---|---|---|---|---|
| **LangGraph/LangChain** | LangChain Inc ($125M B, $1.25B val) | Py/TS | Your server + managed platform | ✅ client | Ollama adapter | ★ Durable checkpointers (headline feature) | 137k★; LangGraph ~66M PyPI/mo; Klarna, Uber, LinkedIn, Replit | MIT |
| **OpenAI Agents SDK** | OpenAI | Py/TS | Your server (Agent Builder killed Jun 2026) | ✅ | Adapter via LiteLLM | Sessions (SQLite/Redis/…) | 26.3k★; ~28M PyPI/mo; Ramp, Canva, Carlyle | MIT |
| **Claude Agent SDK / Claude Code** | Anthropic ($30B G, $380B val) | Py/TS | Your process + Managed Agents + Cowork (desktop/web/iOS/Android) | ✅ | ❌ (Claude only) | Sessions, CLAUDE.md, hooks | 123k★ (claude-code); ~26M npm/mo (SDK); Skills spec adopted by ~40 products | SDK MIT/ToS; app proprietary |
| **Google ADK 2.0** | Google | Py/Go/Java/TS/Kotlin | Your infra or Gemini Enterprise Agent Platform | ✅ | Ollama/LiteLLM | Managed Sessions + Memory Bank | 19.1k★; ~28M PyPI/mo | Apache 2.0 |
| **Microsoft Agent Framework 1.0** | Microsoft | Py/.NET | Your process + Foundry hosted (framework-agnostic) | ✅ + AG-UI | Ollama connector, Foundry Local | Threads + vector memory + checkpoints | 11.8k★; ~1.3M PyPI/mo | MIT |
| **Mastra** | Mastra (YC W25, $35M) | TS | Node/serverless + managed | ✅ both directions | Ollama/LM Studio providers | 4-layer memory (working/observational/semantic) | ~24–26k★; ~4M npm/mo; Brex, Sanity, Replit | Apache 2.0 + EE |
| **Vercel AI SDK 6** | Vercel | TS | Any JS runtime; browser hooks | ✅ | Community providers | ❌ explicitly DIY | 24.5k★; ~58M npm/mo; Thomson Reuters | Apache 2.0 |
| **CrewAI** | CrewAI Inc ($18M) | Py | Your server + AMP | ✅ | Ollama/NIM | ★ Unified scored memory (LanceDB) | 55.5k★; ~11M PyPI/mo; "100k certified devs" | MIT |
| **Pydantic AI** | Pydantic | Py | Your server; durable via Temporal/DBOS/Prefect/Restate | ✅ both | Ollama/OpenAI-compat | Typed history + durable execution | 17.2k★; ~21M PyPI/mo | MIT |
| **Rig (rig-core)** | Playgrounds | **Rust** | Your server; WASM feature | ✅ via rmcp | Via Ollama/OpenAI-compat | 13 vector-store crates incl. **SurrealDB** | 7.3k★; 810k downloads/90d (accelerating) | MIT |
| **Kalosm/Floneum** | Community | **Rust** | **In-process, local-first** | ❌ | ★ Built-in candle GGUF | SurrealDB embeddings | 2.2k★; 17-mo release gap | MIT/Apache |

Sources: per-framework digests — [LangChain 1.0 + Series B](https://www.langchain.com/blog/series-b), [OpenAI deprecations](https://developers.openai.com/api/docs/deprecations), [Claude Agent SDK](https://code.claude.com/docs/en/agent-sdk/overview), [ADK 2.0](https://adk.dev/2.0/), [MAF 1.0](https://devblogs.microsoft.com/agent-framework/microsoft-agent-framework-version-1-0/), [Mastra Series A](https://mastra.ai/blog/series-a), [AI SDK 6](https://vercel.com/blog/ai-sdk-6), [CrewAI](https://github.com/crewAIInc/crewAI), [Pydantic AI](https://pydantic.dev/articles/pydantic-ai-v1), [Rig](https://github.com/0xPlaygrounds/rig), [Kalosm](https://floneum.com/kalosm/).

### What this cluster tells you

**The empty quadrant is real — and it's yours.** Every framework above except Kalosm is strictly server/local-process. None ships an on-device agent runtime for mobile. The only vendor with any mobile agent *product* story is Anthropic (Cowork on iOS/Android since July 2026 — cloud execution, not on-device). Kalosm is the only framework with built-in local inference, and it's a community project with a 17-month release gap and no MCP. **An embedded Rust agent runtime (PMPO + MCP + memory) that runs inside a phone is a genuinely unoccupied position in this table.** That is the strongest validation the research produced for gen_ui_core's existence.

**But three consolidation lessons cut against parts of your build:**

1. **Shakeout is underway.** OpenAI killed Agent Builder and hosted Evals eight months after launch (June 2026); Microsoft merged AutoGen + Semantic Kernel; hosted runtimes now host competitors' frameworks. Surviving differentiation is harness quality, memory, and ecosystem standards — not orchestration loops, which are commoditized. Your PMPO loop is not a moat; your *placement* of it (on-device) is.
2. **MCP is table stakes and has an official Rust SDK.** All ten frameworks support MCP; Rig builds its MCP layer on **rmcp** (the official `modelcontextprotocol/rust-sdk`, ~3.5k★, v1.7.0 May 2026). Your hand-rolled MCP client in `gen_ui_mcp` (SSE transport implemented, stdio a stub) is re-implementing a maintained official SDK. That is bespoke surface with no differentiation value.
3. **Memory is where leaders invest.** LangGraph's durable checkpointers, CrewAI's scored unified memory, Mastra's observational memory — the market has decided memory is a first-class pillar. Your SurrealDB graph-RAG memory store is directionally right and more ambitious than most; the risk (per the prior assessment) is that it's resting on the least-verified engine in your stack.

---

## 3. Cluster B — The agent↔UI protocol layer (your protocol bets, audited)

### ⚠ Finding: your specs misattribute A2UI — and the real A2UI is a direct overlap

Your gen_ui spec describes A2UI as "Anthropic Agent-to-UI." **That attribution is wrong. A2UI is Google's protocol** — announced on the Google Developers Blog in December 2025, spec at [a2ui.org](https://a2ui.org/), Apache 2.0, ~15.8k★, v0.9.1 stable July 3 2026, v1.0 in RC ([Google announcement](https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/), [v0.9 post](https://developers.googleblog.com/a2ui-v0-9-generative-ui/)). Anthropic has no protocol named A2UI; its plays are MCP itself plus the **MCP Apps extension** (SEP-1865, Nov 2025, co-developed with OpenAI).

This matters beyond naming hygiene:

- Google's A2UI is a **declarative, native-first generative-UI standard** — the agent streams a JSON component tree, client renderers map it to native widgets — **with a Flutter GenUI SDK and an official React renderer already shipped**. That is, almost feature-for-feature, what your custom 27-event A2UI pipeline + 11-variant ContentBlock system does by hand across Dart and TypeScript.
- Your protocol therefore now collides with a Google-backed open standard *that shares its name*. Any external developer reading your docs will assume you implement Google's spec. You don't.
- The convergence is already happening around you: A2UI runs over AG-UI/MCP/A2A as transports; Vercel shipped a json-renderer with A2UI support; CopilotKit ships an A2UI runtime; Oracle integrated it.

**Options, in order of strategic value:** (a) adopt Google A2UI as the wire format for your ContentBlock layer — keep your sealed unions as the internal representation, emit/consume the standard — instantly gaining the Flutter GenUI SDK and React renderer as leverage instead of competition; (b) keep your protocol but rename it (e.g., "GenUI Events") and document explicit non-equivalence; (c) do nothing — this guarantees ecosystem confusion and forfeits interop. (a) is the recommendation; at minimum do (b) immediately.

### The rest of the protocol stack — your bets mostly validated

| Protocol | Owner/governance | Status July 2026 | Verdict for you |
|---|---|---|---|
| **MCP** | Linux Foundation **Agentic AI Foundation** (donated by Anthropic Dec 2025) | Universal: OpenAI, Google, Microsoft, AWS, Apple (Xcode 27); registry in preview; ~9,400+ public servers | ✅ Correct bet. Switch to rmcp SDK. |
| **AG-UI** | CopilotKit ($27M Series A May 2026) | 14.7k★; SDKs incl. **Dart and Kotlin**; integrations: LangGraph, MAF, ADK, Mastra, Pydantic AI; Deutsche Telekom, Docusign, Cisco named | ✅ Correct bet — and the Dart SDK means you may not need to hand-write the Flutter side. |
| **A2UI** | **Google** (a2ui.org) | v0.9.1; v1.0 RC; Flutter GenUI SDK, React renderer | ⚠ Fix attribution; adopt or rename (above). |
| **A2A** | Linux Foundation | v1.0 spec; 150+ orgs; shipping in Azure, Bedrock, Google Cloud | Neutral — relevant later for UAR external mode / BossFang federation. |
| **MCP Apps** (SEP-1865) | Anthropic + OpenAI | Official MCP UI extension (sandboxed iframe); ChatGPT-compatible | Watch — the web-iframe pole vs. A2UI's native pole. Your Tauri surface could support both cheaply. |

Sources: [AAIF/LF announcement](https://www.linuxfoundation.org/press/linux-foundation-announces-the-formation-of-the-agentic-ai-foundation), [AG-UI](https://github.com/ag-ui-protocol/ag-ui/), [CopilotKit raise](https://techcrunch.com/2026/05/05/copilotkit-raises-27m-to-help-devs-deploy-app-native-ai-agents/), [A2A 1.0](https://www.linuxfoundation.org/press/a2a-protocol-surpasses-150-organizations-lands-in-major-cloud-platforms-and-sees-enterprise-production-use-in-first-year), [MCP Apps coverage](https://thenewstack.io/agent-ui-standards-multiply-mcp-apps-and-googles-a2ui/), [assistant-ui](https://github.com/assistant-ui/assistant-ui).

---

## 4. Cluster C — Local/on-device AI products (KnowMe's actual competitors)

### Certified traction, July 2026

| Product | Stack | Inference | Memory | Agents | Sync | Traction | Revenue motion |
|---|---|---|---|---|---|---|---|
| **Ollama** | Go + llama.cpp | ★ local | ❌ | ❌ | ❌ | **~9M MAU**, 176k★, $65M Series B (Jul 2026), 85% of F500 | Cloud add-on $20–100/mo |
| **LM Studio** | Electron (closed) | llama.cpp + MLX | ❌ | MCP/plugins | ❌ | "Millions of downloads," F500 | **Enterprise licensing** |
| **Open WebUI** | Python/Svelte | via backends | RAG | pipelines | server multi-user | 144k★ | Enterprise licensing (license controversy) |
| **AnythingLLM** | Node/React | embedded + BYO | workspaces | ★ no-code agents, scheduled | ❌ | 60k★ | Hosted ~$50/mo |
| **Jan** (closest analog) | **Tauri (TS+Rust)** + llama.cpp | ★ local | "Coming soon" | MCP | ❌ | 43.6k★, ~4–6M downloads | None announced |
| **Cherry Studio** | Electron | via Ollama/LMS | ❌ | assistants+MCP | mobile WIP | 46.9k★ | AGPL dual-license |
| **Khoj** | Py/TS AGPL | local+cloud | ★ second brain | ★ scheduled automations | cloud (dead) | 35k★; **cloud shut down Apr 2026** | ☠ Subscription failed |
| **Limitless** (ex-Rewind) | — | cloud | ★ | — | cloud | $33M raised → **acquired by Meta Dec 2025**, products killed | ☠ Exit, not scale |
| **Msty** | closed desktop | llama.cpp | workspace | agents | — | undisclosed | **$149/yr; $349 lifetime** |
| **PocketPal AI** (mobile leader) | React Native + llama.rn | ★ on-device | ❌ | ❌ | ❌ | 7.2k★, **1M+ Android installs, 3.5★ rating** | Free |

Sources: [Ollama raise/TechCrunch](https://techcrunch.com/2026/07/09/popular-open-source-ai-developer-tool-ollama-raises-65m-grows-to-nearly-9m-users/), [LM Studio free-for-work](https://lmstudio.ai/blog/free-for-work), [Jan](https://github.com/menloresearch/jan), [Khoj cloud shutdown](https://app.khoj.dev/), [Meta/Limitless](https://techcrunch.com/2025/12/05/meta-acquires-ai-device-startup-limitless/), [Msty pricing](https://msty.ai/studio/pricing), [PocketPal Play listing](https://play.google.com/store/apps/details?id=com.pocketpalai&hl=en_US), [Open WebUI license](https://docs.openwebui.com/license/), [AnythingLLM](https://github.com/Mintplex-Labs/anything-llm).

### What this cluster tells you

**1. Nobody ships KnowMe's full bundle.** On-device inference + persistent local memory graph + autonomous scheduled agents + cross-device sync: no product in the table has all four. Jan (your nearest architectural cousin — Tauri, Rust, llama.cpp, Apache 2.0, 43.6k stars) has "Memory: coming soon" and no mobile app despite an open roadmap issue. PocketPal — the mobile local-AI traction leader at 1M+ installs — has *no* memory, agents, or sync. The composite position is unoccupied. That's the good news, and it's real.

**2. The pricing thesis is contradicted by every comp.** The verified ceiling for consumer local-AI willingness-to-pay is **Msty at $149/year**. Ollama's premium tier is $100/mo — for *cloud* GPU time, the opposite of sovereignty. Khoj — the closest product to KnowMe's memory+agents vision — shut down its cloud subscription in April 2026 because the model "proved difficult to scale." Limitless exited to Meta rather than scaling subscriptions. **No product charging $100–200/mo for local personal AI exists.** The two revenue motions that verifiably work in this category are enterprise licensing (LM Studio, Open WebUI, Cherry Studio) and modest cloud add-ons ($20/mo). KnowMe's $200/mo "load-bearing tier" is priced into an empty band with two adjacent corpses. It needs either enterprise repositioning (TribeHealth/regulated is actually the natural buyer of "provably air-gapped") or a 5–10× price cut with a licensing story that survives the fact that Qwen weights are free.

**3. The OS platforms just commoditized your base layer.** Apple's Foundation Models framework (iOS 26, Sept 2025; image input + custom skills at WWDC 2026) gives every third-party app **zero-cost, offline, ~3B-param on-device inference with typed structured output and tool calling** ([Apple newsroom](https://www.apple.com/newsroom/2025/09/apples-foundation-models-framework-unlocks-new-intelligent-app-experiences/)). Google's ML Kit GenAI APIs expose Gemini Nano on premium Android ([docs](https://developers.google.com/ml-kit/genai)). Consequence: "small local model on a phone" is no longer a feature — it's an OS primitive. Your differentiation must live in the layers above (memory graph, agents, sync, sovereignty guarantees) — and your `InferenceProvider` seam should treat Apple FM / Gemini Nano as first-class lanes (free, zero-download, battery-optimized) with bundled llama.cpp for custom/larger models and mid-range Android. Shipping a 1GB Qwen download as the *only* mobile path when the OS offers a free 3B model is a strategic error the current PoC plan (S4 "sovereign mode" via mistral.rs CPU) would commit.

---

## 5. Cluster D — Cross-platform shared-core architectures (your pattern, certified)

The one-Rust-core/multiple-shells pattern now has **seven verified production precedents**, materially strengthening the case beyond the 1Password/AppFlowy pair from the earlier assessment:

| Precedent | Core | Shells | Scale |
|---|---|---|---|
| **Signal (libsignal)** | Rust protocol+logic | Swift / Kotlin / TS-Electron | Tens of millions of users |
| **Bitwarden SDK** | Rust crates | Swift/Kotlin mobile rewrite, web via WASM, CLI | Mainstream password manager |
| **Element X (matrix-rust-sdk)** | Rust (sync, crypto, timeline) | SwiftUI / Jetpack Compose | Flagship Matrix clients |
| **Mozilla App Services** | Rust + **UniFFI** bindings | Firefox Android / iOS | Firefox mobile scale |
| **Amazon Prime Video** | Rust + WASM UI core | 8,000+ device types | ([Amazon Science](https://www.amazon.science/blog/how-prime-video-updates-its-app-for-more-than-8-000-device-types)) |
| **1Password** | Rust core + Typeshare | Swift / Kotlin / TS | Millions of users |
| **AppFlowy** | Rust (FlowySDK) | Flutter, later Tauri/React | Leading OSS Notion alt |

Two design notes fall out of the precedent set. First, **none of the mainstream precedents share a single UI across mobile and desktop** — they all pair one core with per-platform shells, which is exactly your two-shell choice; the pattern-level decision is validated. Second, **UniFFI** (Mozilla's binding generator, also used by Bitwarden and Matrix) is the production-proven alternative binding layer to flutter_rust_bridge — if the frb on-device validation (the refuted claim from the prior assessment) goes badly, UniFFI + thin platform channels is the precedented fallback, not a rewrite.

On the UI-shell side: Flutter's 2026 position is strong (~2.8M monthly active devs, ~30% of new free iOS apps, Google's own **NotebookLM mobile app is Flutter** — [VGV](https://verygood.ventures/blog/top-companies-using-flutter/)); Kotlin Multiplatform is the rising alternative for shared *logic* (Google Docs ships it — [Google endorsement](https://android-developers.googleblog.com/2024/05/android-support-for-kotlin-multiplatform-to-share-business-logic-across-mobile-web-server-desktop.html)) but its shared-logic role is what your Rust core already does; and Rust-native UI (Dioxus 0.7) remains production-thin — correctly excluded from your standard.

---

## 6. Cluster E — Scaffold/skill/rules ecosystems (the pack's competition)

| Approach | Owner | Traction (verified) | Model |
|---|---|---|---|
| **Agent Skills spec (SKILL.md)** | Anthropic → open spec (agentskills.io, Dec 2025) | Adopted by ~40 products: OpenAI Codex, GitHub Copilot (Dec 2025), VS Code, Cursor, Gemini CLI, JetBrains | Open standard — **your format won** |
| **AGENTS.md** | OpenAI et al. → **Linux Foundation AAIF** | 30+ tools, **60k+ repos** | Open standard — you already ship it |
| **Claude Code plugins/marketplaces** | Anthropic (decentralized git marketplaces) | Community hubs: 425 plugins / 2,810 skills (one hub); skillsmp ~1.9M scraped | Your distribution channel |
| **GitHub spec-kit** | GitHub/Microsoft | **122k★**, v0.12.16 (Jul 15, 2026), 30+ agents | Spec-driven pipeline (heavier) |
| **OpenSpec** | Fission-AI | **52.6k★** | Your chosen SDD tool — validated |
| **awesome-cursorrules** | Community | 39.5k★, 200+ rule files | Static rules — being superseded by skills |
| **shadcn/ui registry** | shadcn/Vercel | **119k★**; registry protocol, namespaces, private registries | ★ The winning scaffold-distribution pattern |
| **Devin Playbooks / Copilot awesome-copilot** | Cognition / GitHub | Vendor-scoped | Walled ecosystems |

Sources: [agentskills.io ecosystem report](https://agentman.ai/blog/agent-skills-ecosystem-report-2026), [Copilot adopts Agent Skills](https://github.blog/changelog/2025-12-18-github-copilot-now-supports-agent-skills/), [agents.md](https://agents.md/), [spec-kit](https://github.com/github/spec-kit), [OpenSpec](https://github.com/Fission-AI/OpenSpec), [shadcn registry](https://ui.shadcn.com/docs/registry), [Claude Code plugins](https://claude.com/blog/claude-code-plugins).

### What this cluster tells you

**Your format bets won the standards war without you having to do anything** — SKILL.md is now the cross-vendor skill standard (even GitHub Copilot and OpenAI Codex consume it), AGENTS.md is Linux-Foundation-governed with 60k repos, and OpenSpec (your choice) is a top-two SDD tool at 52.6k stars. Your pack is standing on the right substrates.

**Two documented failure modes to design against:**

1. **Quality/security is the ecosystem's proven weakness, not supply.** The 2026 SkillsBench/Snyk "ToxicSkills" research found average public skill quality of 6.2/12, only top-quartile skills actually improving outcomes, **36% of tested skills vulnerable to prompt injection, and 140,963 issues across 22,511 audited skills**. A pack that ships *audited, versioned, security-reviewed* skills — and says so — differentiates on exactly the axis the ecosystem is failing. Your 40 Base Rules + audit.sh are already most of the story; you haven't productized the security half (prompt-injection review of skill content, provenance/signing — which KnowMe's Ed25519 plugin-signing design already knows how to do).
2. **One-shot scaffolds don't get updates; registries do.** The create-t3-app weakness (generate once, drift forever) is documented, and the shadcn registry model (copy-source + namespaced registry + `diff`/update path + private registries) is the demonstrated winner at 119k stars. Your scaffold scripts are currently one-shot. The upgrade path — a registry-style distribution where scaffolded projects can pull updated templates/skills and `audit.sh` can diff against the current standard — is the difference between a snapshot and a platform.

---

## 7. Head-to-head: where TJ-ARCH-MOB-001 stands

| Comparison point | Your position | Landscape verdict |
|---|---|---|
| On-device mobile agent runtime | gen_ui_core embedded (PMPO+MCP+memory in-process) | **Unoccupied quadrant — genuine differentiation.** No major framework ships this. |
| Two-shell shared Rust core | Flutter + Tauri over one core | **Validated 7× in production** (Signal, Bitwarden, Element X, Mozilla, Prime Video, 1Password, AppFlowy). |
| Protocol alignment | MCP ✅, AG-UI ✅, "A2UI" ⚠ misattributed & colliding with Google's spec | Fix required (§3). |
| MCP implementation | Hand-rolled client (stdio stub) | Official rmcp SDK exists; bespoke re-implementation is negative-value work. |
| Memory | SurrealDB graph-RAG (ambitious, unverified engine) | Direction right — market leaders all invest in memory; engine risk stands (prior assessment). |
| Mobile inference | mistral.rs/candle (undocumented mobile path) | Contradicted by ecosystem (llama.cpp) **and** by OS-native free inference (Apple FM / Gemini Nano). |
| UI components | Custom ContentBlock widgets (Flutter) / assistant-ui (React) | assistant-ui validated (11k★, 2.8k projects); Flutter side could lever Google's A2UI Flutter GenUI SDK. |
| Sync | ElectricSQL read-path + DIY writes | Confirmed custom-software territory (prior assessment stands). |
| Skill pack substrate | SKILL.md + AGENTS.md + OpenSpec | **All three won their standards races.** |
| Pack distribution | One-shot scaffold scripts | Behind the shadcn-registry pattern; no update path. |
| Pack security posture | 40 Rules + audit.sh (functional audits only) | Ecosystem's documented failure is skill security — unclaimed differentiation. |
| Product pricing (KnowMe) | $0 / $20 / **$200/mo** | $200 consumer band empty; nearest comps died or exited; enterprise licensing is the proven motion. |

---

## 8. What you need to change

Ordered by (impact × urgency), with the certified evidence behind each:

1. **Resolve the A2UI collision now.** Rename your internal protocol or (better) adopt Google's A2UI as your wire format and keep ContentBlock as the internal model — gaining their Flutter GenUI SDK and React renderer as free leverage. Every month of delay deepens doc debt and ecosystem confusion with a Google-backed, v1.0-RC standard that shares your protocol's name. *(Evidence: a2ui.org v0.9.1, Flutter/React renderers shipped, AG-UI/Vercel/Oracle/CopilotKit converging on it.)*
2. **Replace the hand-rolled MCP client with rmcp.** Official SDK, v1.7.0, maintained by the protocol org, already proven inside Rig. Your stdio transport is a stub anyway — delete, don't finish, bespoke protocol plumbing. Redirect that effort at memory and sync, where you're differentiated.
3. **Make platform-native inference a first-class mobile lane.** Add Apple Foundation Models (iOS 26+) and ML Kit GenAI/Gemini Nano (premium Android) behind `InferenceProvider`, with llama.cpp (`llama-cpp-2`) as the bundled fallback for custom models and mid-range Android — and demote mistral.rs to desktop. This converts your weakest bet (prior assessment §3.3) into a strength: zero-cost, zero-download inference on exactly the devices consumers hold, with sovereignty preserved (Apple FM is on-device by design).
4. **Reprice/reposition KnowMe's top tier before building any tier machinery.** The $100–200/mo consumer local-AI band is empty; Khoj's subscription death and Limitless's acquihire are the two nearest data points; enterprise licensing is the only verified premium motion (LM Studio, Open WebUI). Either aim the premium tier at regulated/enterprise (TribeHealth overlap: air-gap guarantees, audit bundles, compliance export) or cap consumer pricing near the verified ceiling (~$149/yr) until you have contrary evidence from actual buyers.
5. **Move the pack from one-shot scaffold to registry-with-updates.** Adopt the shadcn registry pattern for your templates and project-local skills: versioned, namespaced, diff-able, private-registry-capable. Your audit.sh becomes the `diff` engine against the living standard. This is also the mechanism that fixes your documented doc-drift problem at the root.
6. **Productize skill security.** Add a prompt-injection/security review gate for every skill in the pack (the ToxicSkills failure numbers are your marketing copy), sign pack releases, and publish conformance to the agentskills.io spec. You already designed Ed25519 signing for KnowMe plugins — apply it to your own skill distribution first.
7. **Ship AG-UI via its Dart/Kotlin SDKs rather than hand-writing the Flutter client layer** where feasible — CopilotKit maintains them; your effort belongs in the blocks, not the transport.
8. **Name UniFFI as the contingency binding layer** in the arch standard. If C-103's on-device frb validation fails or drags, the Mozilla/Bitwarden/Matrix path is the precedented fallback — writing it down now converts a potential crisis into a decision tree.
9. **Keep and sharpen the genuinely differentiated claims.** In order of defensibility: (a) the only embedded on-device agent runtime in Rust with MCP + graph memory; (b) the only "full bundle" personal AI (inference + memory + agents + sync) if you ship even a modest version of all four — no competitor has; (c) the only skill pack that scaffolds *and audits* agentic-app architecture with a security posture. Say these three things everywhere; drop breadth claims (142 providers, 4 databases, 3 inference lanes) from positioning — the landscape evidence says breadth is what killed the comparable efforts (original ElectricSQL, Khoj).

### What not to change

The two-shell/one-core pattern (validated seven times over); the Flutter-mobile/Tauri-desktop split (independently corroborated); MCP and AG-UI as protocol bets (both won); SKILL.md/AGENTS.md/OpenSpec as pack substrates (all won); the ContentBlock sealed-union discipline as *internal* representation (sound regardless of wire format); the 40 Base Rules (they encode exactly the failure modes the ToxicSkills research documents); the defects-flow-back-to-scaffold loop (your best process asset).

---

## 9. Source index

**Agent frameworks:** [LangChain Series B](https://www.langchain.com/blog/series-b) · [LangGraph](https://github.com/langchain-ai/langgraph) · [OpenAI Agents SDK](https://github.com/openai/openai-agents-python) · [OpenAI deprecations (Agent Builder sunset)](https://developers.openai.com/api/docs/deprecations) · [Claude Agent SDK](https://code.claude.com/docs/en/agent-sdk/overview) · [Cowork web/mobile](https://claude.com/blog/cowork-web-mobile) · [Google ADK 2.0](https://adk.dev/2.0/) · [ADK Go 2.0](https://developers.googleblog.com/announcing-adk-go-20/) · [Microsoft Agent Framework 1.0](https://devblogs.microsoft.com/agent-framework/microsoft-agent-framework-version-1-0/) · [MAF at Build 2026](https://devblogs.microsoft.com/agent-framework/microsoft-agent-framework-at-build-2026-announce/) · [Mastra Series A](https://mastra.ai/blog/series-a) · [Vercel AI SDK 6](https://vercel.com/blog/ai-sdk-6) · [CrewAI](https://github.com/crewAIInc/crewAI) · [Pydantic AI v1](https://pydantic.dev/articles/pydantic-ai-v1) · [Rig](https://github.com/0xPlaygrounds/rig) · [rmcp](https://github.com/modelcontextprotocol/rust-sdk) · [Kalosm](https://floneum.com/kalosm/) · [Swiftide](https://github.com/bosun-ai/swiftide)

**Protocols/UI:** [AG-UI](https://github.com/ag-ui-protocol/ag-ui/) · [CopilotKit $27M](https://techcrunch.com/2026/05/05/copilotkit-raises-27m-to-help-devs-deploy-app-native-ai-agents/) · [Google A2UI announcement](https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/) · [A2UI v0.9](https://developers.googleblog.com/a2ui-v0-9-generative-ui/) · [a2ui.org](https://a2ui.org/) · [AAIF / MCP donation](https://www.anthropic.com/news/donating-the-model-context-protocol-and-establishing-of-the-agentic-ai-foundation) · [MCP Registry](https://modelcontextprotocol.io/registry/about) · [A2A 1.0 / LF](https://www.linuxfoundation.org/press/a2a-protocol-surpasses-150-organizations-lands-in-major-cloud-platforms-and-sees-enterprise-production-use-in-first-year) · [MCP Apps / agent-UI standards](https://thenewstack.io/agent-ui-standards-multiply-mcp-apps-and-googles-a2ui/) · [assistant-ui](https://github.com/assistant-ui/assistant-ui) · [AI Elements](https://vercel.com/changelog/introducing-ai-elements)

**Local AI products:** [Ollama $65M / 9M users](https://techcrunch.com/2026/07/09/popular-open-source-ai-developer-tool-ollama-raises-65m-grows-to-nearly-9m-users/) · [Ollama Cloud pricing](https://ollama.com/cloud) · [LM Studio free-for-work](https://lmstudio.ai/blog/free-for-work) · [LM Studio Enterprise](https://lmstudio.ai/enterprise) · [Jan](https://github.com/menloresearch/jan) · [Open WebUI license](https://docs.openwebui.com/license/) · [Cherry Studio](https://github.com/CherryHQ/cherry-studio) · [AnythingLLM](https://github.com/Mintplex-Labs/anything-llm) · [MLC-LLM](https://github.com/mlc-ai/mlc-llm) · [WebLLM](https://github.com/mlc-ai/web-llm) · [PocketPal](https://github.com/a-ghorbani/pocketpal-ai) · [Khoj](https://github.com/khoj-ai/khoj) · [Meta acquires Limitless](https://techcrunch.com/2025/12/05/meta-acquires-ai-device-startup-limitless/) · [Msty pricing](https://msty.ai/studio/pricing) · [Apple Foundation Models](https://www.apple.com/newsroom/2025/09/apples-foundation-models-framework-unlocks-new-intelligent-app-experiences/) · [ML Kit GenAI](https://developers.google.com/ml-kit/genai)

**Architectures & scaffolds:** [libsignal](https://github.com/signalapp/libsignal) · [Mozilla App Services / UniFFI](https://mozilla.github.io/application-services/) · [Element X iOS](https://github.com/element-hq/element-x-ios) · [Bitwarden SDK](https://sdk-api-docs.bitwarden.com/bitwarden_core/) · [Prime Video Rust/WASM](https://www.amazon.science/blog/how-prime-video-updates-its-app-for-more-than-8-000-device-types) · [KMP Google endorsement](https://android-developers.googleblog.com/2024/05/android-support-for-kotlin-multiplatform-to-share-business-logic-across-mobile-web-server-desktop.html) · [Compose Multiplatform iOS stable](https://blog.jetbrains.com/kotlin/2025/05/compose-multiplatform-1-8-0-released-compose-multiplatform-for-ios-is-stable-and-production-ready/) · [Expo apps dataset](https://evanbacon.dev/blog/expo-apps) · [Flutter adoption (VGV)](https://verygood.ventures/blog/top-companies-using-flutter/) · [Dioxus 0.7](https://dioxuslabs.com/blog/release-070/) · [Claude Code plugins](https://claude.com/blog/claude-code-plugins) · [Agent Skills ecosystem report](https://agentman.ai/blog/agent-skills-ecosystem-report-2026) · [Copilot adopts Agent Skills](https://github.blog/changelog/2025-12-18-github-copilot-now-supports-agent-skills/) · [agents.md](https://agents.md/) · [spec-kit](https://github.com/github/spec-kit) · [OpenSpec](https://github.com/Fission-AI/OpenSpec) · [shadcn registry](https://ui.shadcn.com/docs/registry) · [awesome-cursorrules](https://github.com/PatrickJS/awesome-cursorrules)

**Method note:** Figures are as-of dates shown; npm/PyPI windows lag ~6 weeks; company-sourced claims (CopilotKit "Fortune 500," CrewAI "half of F500") are marked as vendor claims in the underlying digests. Claims that could not be verified were either excluded or labeled. The "ChatGPT mobile app uses React Native" meme specifically failed verification and is not used.
