# Nativ Evaluation & the Multi-Engine Local Inference Question

**Date:** 2026-07-22
**Author:** KnowMe research agent
**Reference codebase:** `/Users/gqadonis/Projects/references/nativ` (`github.com/Blaizzy/nativ`, MIT, v0.0.1)
**Question posed:** Should KnowMe introduce multiple optional local-inference engines per platform
(MLX, llama.cpp, candle/mistral.rs, …) on desktop/Tauri and mobile — and does Nativ have features
worth adopting regardless?

> **Method note.** Firecrawl tooling is not installed in this environment; web research was performed
> with the available search/fetch services against benchmarks, vendor engineering blogs, engine
> repositories, and ADRs. All externally sourced claims carry citations inline.

---

## 1. Verdict (read this first)

**Do NOT build a general multi-engine inference layer.** The evidence does not support "multiple
options everywhere" — every engine comparison found shows the winner is **platform- and
model-specific**, not user-preference-specific, which means a "let the user pick an engine" feature
would add a permanent second codepath to test, certify, and support while changing nothing for most
users. The maintenance-cost concern in the brief is real and confirmed by teams that did try dual
backends (RuVector's ADR-008 lists "two backend codepaths require testing both paths" and
mistral.rs API-coupling as top negative consequences
[source](https://github.com/ruvnet/ruvector/blob/main/docs/adr/ADR-008-mistral-rs-integration.md)).

**Do make ONE bounded, evidence-backed engine addition: MLX on Apple Silicon desktop — integrated
in-process in Rust via Apple's official `mlx-c` C API, never via an embedded Python runtime.**
KnowMe's hard constraint is that Python is not a production path, which rules out Nativ's packaging
(embedded CPython + `mlx-vlm` server). It does not rule out MLX: `mlx-c` is an official,
Apple-maintained C API over the same C++ core, and a Rust binding layer already exists — the `mlex`
crate (v0.1.x, statically linked vendored `mlx-c`, "no Python required", MLX-format checkpoints,
tool calling, multimodal) and the OminiX-MLX ecosystem (pure-Rust model crates for Qwen/Mistral/
GLM + an OpenAI-compatible server, "zero Python dependencies at inference time")
([mlex on docs.rs](https://docs.rs/mlex/latest/mlex/?search),
[OminiX-MLX](https://github.com/OminiX-ai/OminiX-MLX),
[oxideai/mlx-rs](https://github.com/oxideai/mlx-rs)). Here the upside *is* significant and quantified
(§3.1): 1.4–1.8× decode over llama.cpp, 5–10% lower memory footprint, 70B-class models that
llama.cpp OOMs on at 64 GB, and industry momentum (Ollama switched its Mac backend to MLX in March
2026; LM Studio ships MLX). Nativ remains the existence proof that "one engine, wrapped deeply"
beats "many engines, wrapped thinly" — but only its *ideas* transfer, not its runtime.

**Maturity gate:** the Rust→`mlx-c` ecosystem is young (mlex 0.1.x; OminiX pre-1.0). The MLX lane
must pass a spike + benchmark + certification gate on KnowMe's exact model set (Qwen3.5 class)
before it becomes the default for any user; llama.cpp stays the macOS default until then (§5).

**Per-platform ruling:**

| Platform | Ruling | Rationale |
|---|---|---|
| Desktop macOS | **Add MLX** as an optional second engine, in-process via Rust→`mlx-c`, behind the existing `InferenceProvider` trait | Large measured gains; Rust-native path exists via `mlx-c` (mlex/OminiX-MLX); gated on a maturity spike |
| Desktop Windows/Linux | **llama.cpp only** | No credible alternative (CUDA/Vulkan/CPU coverage); candle/mistral.rs offers nothing here worth a second codepath |
| iOS | **Fix the llama.cpp lane; do not add engines now** | The iOS failure is a *model-size/jetsam* failure, not an engine failure (§3.3). Re-evaluate MLX-Swift only after the lane is stable |
| Android | **LiteRT-LM only** (current) | Already the right call; LiteRT-LM wins its native Gemma format on iPhone-class hardware benchmarks |
| Web | **WebLLM only** (current) | No alternative worth discussing |
| candle/mistral.rs anywhere | **Do not revive** | KnowMe already corrected this bet (assessment 2026-07-16); ecosystem churn and test-matrix cost confirmed externally |

**Nativ features worth adopting even though we reject its single-platform scope** (§4): the
**OpenAI/Anthropic-compatible localhost API for coding agents** (highest value), **model capability +
memory-fit discovery heuristics**, and **bucketed SQLite inference analytics**.

---

## 2. What Nativ Is

Nativ ("Local AI, native to your Mac") is a 3-day-old (initial commit 2026-07-20), ~23k-line SwiftUI
app by the author of `mlx-vlm`: macOS 26+, Apple-silicon-only, **single-engine (MLX)** chat app +
model manager + performance dashboard + **OpenAI- and Anthropic-compatible localhost inference
server** for external coding agents (Codex, Claude Code, Hermes, Pi, OpenCode)
([GitHub](https://github.com/Blaizzy/nativ), [site](https://blaizzy.github.io/nativ/),
[coverage](https://www.neura.market/news/nativ-local-ai-mac-open-source)).

Architecturally it is a **sidecar product**: SwiftUI app → C launcher → embedded relocatable Python
(python-build-standalone + pinned requirements) → `mlx-vlm` server (FastAPI/uvicorn on
`127.0.0.1:8080`). The app never links inference in-process; everything is localhost HTTP. It has
**no multi-engine abstraction** — a repo-wide search finds zero llama.cpp/GGUF/CoreML/ONNX/LiteRT
integration. It has no memory/RAG/agents/sync — it is an inference workspace, not a personal-AI
product.

So Nativ answers our question by counter-example: the most polished new entrant in local Mac
inference chose *one* engine and spent the effort on packaging, telemetry, and developer-surface
instead.

---

## 3. The Engine Evidence

### 3.1 Desktop macOS: MLX's advantage is real and large

- **Decode throughput:** on M4 Max, mlx-swift wins every measured cell at **1.4–1.8× over llama.cpp**
  after the early-2026 Qwen/Gemma kernel updates — e.g., Qwen 3.5 2B: 291.9 vs 149.7 tok/s; Gemma 4
  E2B: 185.4 vs 119.2 tok/s ([apple-silicon-llm-bench](https://github.com/john-rocky/apple-silicon-llm-bench)).
  An earlier systematic study (M2 Ultra) found ~230 vs ~150 tok/s on Qwen-2.5 7B 4-bit
  ([summary](https://stridenote.net/mlx-llamacpp-metalrt-apple-silicon/)).
- **Memory:** MLX is 5–10% leaner per model; the gap widens with size — Llama 3.1 70B Q4 on 64 GB:
  llama.cpp OOMs, MLX runs at ~7 tok/s ([local-llm.net comparison](https://www.local-llm.net/compare/llama-cpp-vs-mlx/)).
- **Ecosystem signal:** Ollama v0.19 (2026-03-30) switched its Apple Silicon backend from llama.cpp
  to MLX, reporting +57% prefill / +93% decode on M5 (vendor-stated)
  ([writeup](https://www.digitalapplied.com/blog/apple-mlx-framework-local-ai-developers-2026-guide));
  LM Studio shipped an MLX engine earlier; vllm-mlx reports 21–87% higher serving throughput.
- **llama.cpp's counterweights** are real but shrinking: universal GGUF format, Windows/Linux/CUDA
  coverage, and its own Metal 4 Tensor API gains in 2026 (+229% prompt processing on M5 Max)
  ([stridenote](https://stridenote.net/mlx-llamacpp-metalrt-apple-silicon/)).

**Implication:** on a Mac, llama.cpp leaves 30–80% of available decode performance on the table. For
KnowMe — whose desktop personas include exactly the Apple-silicon power users who notice tokens/sec —
one additional engine here has *significant* upside. The integration path must be **in-process Rust**,
not Nativ's embedded-Python sidecar (Python is not a KnowMe production path): Apple's official
`mlx-c` C API wraps the same C++ core that mlx-lm/mlx-swift sit on, and Rust bindings over it are
real and shipping as crates — `mlex` (safe API, vendored statically-linked `mlx-c`, loads MLX-format
checkpoints from the HF layout, tool calling, multimodal
[docs.rs](https://docs.rs/mlex/latest/mlex/?search)) and OminiX-MLX (per-model-family crates —
Qwen, Mistral, GLM, VLM, ASR — plus shared KV-cache/RoPE/attention infrastructure and an
OpenAI-compatible server, explicitly "zero Python at inference time"
[GitHub](https://github.com/OminiX-ai/OminiX-MLX)); `oxideai/mlx-rs` provides the underlying
array-level safe bindings ([GitHub](https://github.com/oxideai/mlx-rs)). An MLX engine would slot
behind the existing `InferenceProvider` trait (`gen_ui_inference/src/lib.rs`) as a feature-gated
crate, exactly like the current `local-llama` / `local-litert-lm` lanes — no process boundary, no
new supervision surface, no second runtime in the bundle. The risk is ecosystem youth (mlex 0.1.x,
docs.rs build red; OminiX pre-1.0; kernel parity with the early-2026 mlx-lm Qwen/Gemma gains must be
verified), which is why §5 gates adoption on a spike rather than committing outright.

### 3.2 candle / mistral.rs: the evidence says stay out

- KnowMe's own assessment (2026-07-16) already flagged mistral.rs-on-mobile as a corrected weak bet,
  replaced by LiteRT-LM.
- Externally: mistral.rs is competitive on CUDA (~95% of llama.cpp on A10
  [discussion](https://github.com/EricLBuehler/mistral.rs/discussions/46)) but carries documented
  costs — dual-codepath testing, API coupling, no WASM
  ([RuVector ADR-008](https://github.com/ruvnet/ruvector/blob/main/docs/adr/ADR-008-mistral-rs-integration.md)) —
  and the Rust-LLM ecosystem has a churn history (rustformers/llm, llama-rs both archived
  [survey](https://zylos.ai/research/2026-03-31-rust-ai-agent-frameworks-infrastructure/)).
- On Apple Silicon specifically, mistral.rs has no advantage story over either MLX or llama.cpp.
  There is no platform where it would be KnowMe's best option.

### 3.3 iOS: the constraint is memory, not the engine

KnowMe's iOS lane is currently dead (5.68 GB Qwen 9B vs the jetsam budget, no RAM preflight —
`docs/diagnostics/mobile-uar-local-inference-2026-07-21.md`). The benchmark data reframes the fix:

- On iPhone 17 Pro: **Qwen 3.5 2B** — MLX-Swift 61.2 tok/s (peak 1,279 MB), llama.cpp 39.1 tok/s
  (1,479 MB), CoreML/ANE 27.9 tok/s (**241 MB**). **Gemma 4 E2B** — LiteRT-LM 55.4 tok/s at just
  **641 MB** peak, 4.5× leaner than MLX ([apple-silicon-llm-bench](https://github.com/john-rocky/apple-silicon-llm-bench)).
- Every viable iOS configuration runs **2–4B-class models at ~0.6–1.5 GB peak**. No engine makes a
  9B model survivable under jetsam; the model choice and the RAM preflight *are* the fix.
- MLX-Swift does run on iOS and would buy ~1.5× decode over llama.cpp on Qwen — but integrating it
  means Swift interop across the Rust/Flutter FFI boundary, a new artifact pipeline (MLX-format
  models), and a second iOS certification surface. That is a poor trade while the existing lane has
  never worked. **Sequence: ship a working 2–4B llama.cpp (or LiteRT-LM-on-iOS) lane first; put
  MLX-Swift on the re-evaluation list.**

### 3.4 Android: already settled

LiteRT-LM is the right single engine for KnowMe's Android lane: it wins its native Gemma format
outright (55.4 tok/s, smallest memory footprint in the iPhone benchmark above), and KnowMe has
already invested in the Kotlin bridge (`KnowMeLiteRtLmBridge.kt`). llama.cpp remains the sensible
*fallback* for non-Gemma models (Qwen3-4B is already cataloged as exactly that). No third engine.

### 3.5 What a "multi-engine option" would actually cost

Each additional engine is not a config flag — it is: a new artifact format + catalog pipeline
(GGUF vs MLX vs .litertlm), a new download/verify path, a new lifecycle/certification surface per
OS, new failure modes in the FFI/sidecar boundary, and a doubling of the inference test matrix on
every release. RuVector's ADR-008 and KnowMe's own mistral.rs experience are the receipts. This is
why the ruling is "right engine per platform, **one** optional second engine only where the measured
gap justifies it (macOS desktop)" — and never user-facing "pick your engine" roulette. Engine
selection should be automatic per platform + model, exactly as Ollama does ("picks MLX when better"
[local-llm.net](https://www.local-llm.net/compare/llama-cpp-vs-mlx/)).

---

## 4. Nativ Features Worth Adopting

Ranked by value to KnowMe's mission:

1. **OpenAI/Anthropic-compatible localhost API for external coding agents.**
   Nativ's most strategic idea: expose local inference as `http://127.0.0.1:PORT/v1/chat/completions`
   + Anthropic `/v1/messages`, with one-click config generation for Codex, Claude Code, Hermes, Pi,
   OpenCode. KnowMe already owns the pieces — `gen_ui_server_axum`, the BossFang "remote model
   server" role, the shared `InferenceProvider` engine — but has no developer-facing endpoint story.
   This turns KnowMe from "an app with local models" into "the private local backend for the user's
   whole agent toolchain," reinforcing sovereignty (agents never need a cloud key).
   *Effort: M. Recommendation: adopt — candidate for a `local-api-server` OpenSpec change.*

2. **Model capability & memory-fit discovery heuristics.**
   Nativ mines `config.json`/chat templates for capabilities (vision/audio/tools/reasoning), parses
   quant + parameter counts, applies a fixed **20%-of-RAM headroom** rule, and cross-validates
   packed-weight metadata before labeling "Likely fits / May not fit"
   (`LocalModelDiscovery.swift:109-151, 865-977`). KnowMe's catalog has artifacts but no fit
   estimation — precisely the gap behind the iOS jetsam incident. Port the *estimation math* into
   `gen_ui_inference` (Rust) and surface fit badges in the Models tile (alongside the privacy-tier
   badges recommended in the os-june report).
   *Effort: S–M. Recommendation: adopt — and it doubles as the iOS RAM-preflight mechanism.*

3. **Bucketed SQLite inference analytics.**
   Nativ's overlay middleware re-parses its own SSE stream to compute TTFT/prefill/decode/peak-memory
   per request and writes raw events + pre-aggregated hourly/daily buckets to SQLite (WAL), read
   directly by the dashboard (`nativ_server.py:770-902`, `NativAnalyticsStore.swift`). KnowMe has no
   inference telemetry at all; this is a cheap, local-first design that fits the SurrealDB/SQLite
   substrate and the Models tile's planned benchmark feature.
   *Effort: S. Recommendation: adopt.*

4. **Engine-facing UX details, not packaging.** Nativ's embedded-Python packaging is **rejected**
   (Python is not a KnowMe production path; the MLX path is in-process Rust via `mlx-c`, §3.1). What
   still transfers: the **overlay-instead-of-fork discipline** (instrument the upstream engine rather
   than fork it — in KnowMe's case, wrap `mlx-c`/model crates behind `InferenceProvider` and upstream
   fixes flow in), **orphan reaping** for any child process (its C launcher kills the child when the
   parent dies — already required for the BossFang sidecar, FR-3), and **stripping debugger env**
   (Xcode Metal validation vars) from inference processes — a production-learned gotcha that applies
   to any Metal-backed engine KnowMe ships, including in-process ones during development.
   *Recommendation: adopt the discipline, not the runtime.*

5. **Minor but real:** pure-function restart decisions (`hasSameLaunchConfiguration` — "does this
   settings delta require an engine restart?"), stdout sentinel progress lines with SIGSTOP/SIGCONT
   pause for downloads, live searchable server logs in a Developer page.
   *Recommendation: adopt opportunistically when touching those modules.*

**Nothing else transfers:** Nativ's chat, model library UI, and menu bar are inferior to or
orthogonal to KnowMe's existing surfaces; it has no memory, agents, sync, or mobile story.

---

## 5. Recommended Action List

| # | Action | Type | Maps to |
|---|---|---|---|
| 1 | **MLX engine for desktop macOS, in-process Rust** — Phase A (spike): integrate `mlx-c` via `mlex` or OminiX-MLX model crates behind `InferenceProvider` as a feature-gated crate; benchmark KnowMe's exact Qwen3.5-class model set vs the llama.cpp lane; verify kernel parity, memory behavior, tool-calling, and streaming. Phase B (only if the spike clears a defined bar, e.g. ≥1.3× decode at equal-or-better memory): MLX-format artifacts added to the catalog with the same digest-pinning discipline; engine auto-selected for Apple Silicon; llama.cpp remains the fallback and the default until certification | Spike first, then OpenSpec change (`inference-mlx-rust`) | M-series desktop lane; versions.toml `[inference]` |
| 2 | **Memory-fit estimation + RAM preflight** in `gen_ui_inference` (port Nativ's math; block or warn on load when weights+headroom exceed budget) | Small change, high urgency | Fixes the iOS failure class; Models tile |
| 3 | **Local OpenAI/Anthropic-compatible API server** exposing KnowMe's engines to external agents, with per-tool config generation | New OpenSpec change (`local-api-server`) | `gen_ui_server_axum`; BossFang remote-model role |
| 4 | **Bucketed inference analytics** (event + hourly/daily aggregates, TTFT/tok-s/peak-mem per request) feeding the Models/benchmark UI | Small change | Models tile; assessment's benchmark gap |
| 5 | **iOS lane: fix before extending** — 2–4B default model, entitlements, preflight (#2), fallback path; keep llama.cpp; revisit MLX-Swift only after certification | Existing recovery branch | `codex/android-litert-lm-recovery` follow-on |
| 6 | **Explicitly reject**: candle/mistral.rs revival; user-facing engine pickers; multi-engine on Android/Windows/Linux | Record as ADR | Prevents roadmap drift |

Items 1–4 are the kind of work that fits the kbd orchestration flow (one phase, four ordered
OpenSpec changes, medium/high model classes) if the owner approves.

---

## 6. Sources

**Codebases (read 2026-07-22):** Nativ — `README.md`, `project.yml`,
`Sources/Nativ/Features/Models/LocalModelDiscovery.swift`, `Sources/Nativ/NativModel.swift`,
`Sources/Nativ/NativSettings.swift`, `Sources/NativServerKit/`,
`PythonDistribution/Overlay/nativ_server.py`, `PythonDistribution/Scripts/build_mlx_vlm_server.py`,
`PythonDistribution/Launcher/mlx_vlm_server_launcher.c`, `Sources/Nativ/Features/Dashboard/`.
KnowMe — `rust/crates/gen_ui_inference/`, `docs/diagnostics/mobile-uar-local-inference-2026-07-21.md`,
`docs/assessment-2026-07-16.md`, `versions.toml`.

**Web (retrieved 2026-07-22):**
- Nativ repo & site — https://github.com/Blaizzy/nativ · https://blaizzy.github.io/nativ/
- Nativ launch coverage — https://www.neura.market/news/nativ-local-ai-mac-open-source
- Apple Silicon cross-runtime benchmark (M4 Max + iPhone 17 Pro) — https://github.com/john-rocky/apple-silicon-llm-bench
- MLX vs llama.cpp vs MetalRT survey — https://stridenote.net/mlx-llamacpp-metalrt-apple-silicon/
- llama.cpp vs MLX comparison — https://www.local-llm.net/compare/llama-cpp-vs-mlx/
- Ollama MLX backend switch (vendor figures via) — https://www.digitalapplied.com/blog/apple-mlx-framework-local-ai-developers-2026-guide
- mistral.rs dual-backend cost ADR — https://github.com/ruvnet/ruvector/blob/main/docs/adr/ADR-008-mistral-rs-integration.md
- mistral.rs performance status — https://github.com/EricLBuehler/mistral.rs/discussions/46
- Rust LLM ecosystem churn — https://zylos.ai/research/2026-03-31-rust-ai-agent-frameworks-infrastructure/
- MLX cross-platform limits (mobile SDK gap framing) — https://cactuscompute.com/compare/best-mlx-alternative
- Rust-native MLX path (added 2026-07-22 after the no-Python constraint was stated):
  `mlex` crate — https://docs.rs/mlex/latest/mlex/?search ·
  OminiX-MLX — https://github.com/OminiX-ai/OminiX-MLX ·
  oxideai/mlx-rs — https://github.com/oxideai/mlx-rs

---

*Revision 1 (2026-07-22): owner constraint "no Python for serious production work — Rust" applied.
The MLX recommendation was changed from a Nativ-pattern embedded-Python sidecar to in-process Rust
via Apple's official `mlx-c` C API (`mlex`/OminiX-MLX), gated on a spike; Nativ's packaging blueprint
was downgraded to "adopt the discipline, not the runtime" (§4 item 4).*
