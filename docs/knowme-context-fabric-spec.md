# PAGS-SPEC-KM-CTX-001 — KnowMe Context Fabric

**Cross-platform context builder, capability model, and budget abstraction for the KnowApp harness**

| | |
|---|---|
| Document ID | PAGS-SPEC-KM-CTX-001 |
| Status | Draft for review — no code authorized until phase gates approved |
| Date | 2026-07-18 |
| Author | Claude (session work product for Travis James, Prometheus AGS / KnowMe LLC) |
| Applies to | `know-me-system` (rust workspace, Flutter mobile, Tauri desktop, web, Axum cloud) |
| Supersedes | Ad-hoc per-call-site context assembly in `gen_ui_agent::chat` |
| Related | TJ-ARCH-MOB-001, FUNC-SPEC v1.0, cherry-studio fork `contextStrategy.ts` |

---

## 1. Executive summary

KnowMe's promise — "it knows the user" — is delivered through three inputs
(memories, dropped-note RAG, tools) fed to models whose real capacity varies by
two orders of magnitude across deployment targets: a Q4_K_M Qwen3.5-4B on a
Galaxy S25 Ultra is not a cloud frontier model, and pretending otherwise
produces an app that works in demos and fails in pockets. Today, context
assembly is implicit and scattered: `gen_ui_agent::chat` builds message lists
ad hoc, `gen_ui_db_graph` retrieves without budget awareness, tool schemas are
injected wholesale, and nothing enforces the token arithmetic that on-device
models require.

This specification defines **one context fabric**: a `gen_ui_context` crate in
the shared Rust core that owns (a) a three-layer **capabilities model**
(device / model / runtime) resolved into an **EffectiveBudgetPlan**, (b) a
deterministic **ContextBuilder pipeline** that fills that plan in a fixed
order, (c) the four **context-management strategies** from the Cherry Studio
fork, ported natively (sliding window, truncate-middle, progressive
summarization, hierarchical memory) plus the agent pre-compact trigger, (d) a
**chunking subsystem** for dropped documents, and (e) a **retrieval/re-ranking
pipeline** (existing RRF → MMR diversity → recency decay → placement policy).
Every surface — Flutter mobile, Tauri desktop, web/WASM, Axum cloud — consumes
the same crate through the same trait seams already used for
`InferenceProvider`; platforms differ only in the capability profile they
load, never in pipeline logic.

The design thesis is the same as the rest of the Fabric: **architectural
correctness over feature count**. The model plans nothing about its own
context; the harness owns budgets, ordering, truncation, and escalation as
deterministic Rust. This is not a stylistic preference — the research base
(§3) shows small on-device models measurably cannot self-manage tool memory,
long contexts, or multi-step state, and that structured subsystems recover
most of the gap.

## 2. Scope, goals, non-goals

**Goals**

1. A single capability + budget abstraction controlling: system prompt size,
   history window, memory injection count/tokens, knowledge-base retrieval
   count/tokens, tool shortlist size, tool-schema tokens, output reserve,
   thinking reserve, re-ranking configuration — on every platform.
2. Port the four fork strategies as first-class Rust implementations with the
   fork's layered override model (global → assistant/agent → topic/session).
3. Chunking for dropped notes/documents with provenance, integrated with the
   existing `gen_ui_db_graph` ingest path without breaking atomic short notes.
4. Deterministic, testable assembly: same inputs + same profile ⇒ same prompt,
   byte-for-byte, on every platform.
5. Telemetry parity with the fork (`tokensBefore/After/Saved`, alteration
   summary) surfaced on the AG-UI stream.

**Non-goals (v1)**

- Cross-encoder re-ranking (deferred behind a measurement gate, §10.4).
- Model fine-tuning / LoRA for tool calling (separate initiative; §3.4 notes
  when it becomes the higher-leverage move).
- Changing the embedding model or `EMBED_DIM` (stays 384 / all-MiniLM-L6-v2).
- Prompt-content authoring (skills/personas). This spec governs *assembly and
  budgets*, not what the system prompt says.

## 3. Research foundation (condensed, with sources)

Findings gathered across this design session; each shaped a specific mechanism
below. Full citations in §16.

**3.1 Effective context ≪ nominal context.** Qwen3.5-4B and -9B share an
identical 32-layer hybrid layout — 8 × (3 × Gated DeltaNet → 1 × Gated
Attention), `full_attention_interval: 4` — so both have exactly 8
full-attention "precision checkpoints"; the difference is width (2560 vs 4096
hidden), not checkpoint density. GDN layers compress history into fixed-size
recurrent state; precise recall depends on the 8 full-attention layers.
Nominal 262K is a training capability, not a usable retrieval window.
Consequence: **8K operating ceiling, 16K only behind an explicit degraded-mode
flag; reliable RAG budget is low thousands of tokens** (§7 tables).

**3.2 Quantization compounds with context length.** 8-bit ≈ −0.8% on
long-context suites; 4-bit methods lose up to 59% worst-case, damage growing
past 8K; K-quants are gentler than GPTQ/BNB-nf4 but Q4_K_M is still 4-bit.
The GDN-recurrent-state × 4-bit interaction is uncharacterized in the
literature — an explicit empirical risk. Consequence: **needle-in-haystack
smoke test against our actual GGUFs is a Phase 0 gate, not an afterthought.**

**3.3 Position matters ("lost in the middle").** Small models retrieve worst
at 20–70% depth. Consequence: the **placement policy** (§10.3): best hit last
(adjacent to the query), second-best first, middle treated as distractor
space.

**3.4 Agentic limits.** Success ∝ (1−p)^m — 99% per-step accuracy is 36.6% at
100 steps; per-step accuracy itself degrades with step count
(self-conditioning). Deviations on *mutating* actions reduce success odds by
up to 92–96%; read-only deviations barely matter (SABER). Tool-selection
accuracy degrades past ~20–25 exposed tools; Anthropic measured 58 tools ≈
55K tokens. Two-stage retrieve-then-load gives ~6× system-prompt reduction and
10–25× context-growth reduction (TinyAgent/ToolRAG lineage). Small models
cannot self-manage tool memory (MemTool); structured subsystems close the gap
(fastWorkflow). Consequence: **max 3–4 autonomous steps before checkpoint,
individual gates on all mutating actions, 5–8 tool shortlist ceiling, harness
owns planning.** A purpose-LoRA over our real schemas becomes the better
investment only after a tool cluster stabilizes — not before.

**3.5 Thermal envelope is a budget dimension.** Sustained inference: iPhone
−41.5% throughput plateau (Hot state by ~iteration 17); S25-class GPU cliffs
680→231 MHz at 78.3 °C; shell >42 °C in 100 s at peak; dense on-device RAG
prefill has exceeded 14 s TTFT in published deployments. Consequence: the
**SessionEnvelope** (§6.4) tracks cumulative inference and degrades or hands
off to cloud; RAG budgets also bound prefill latency, not just recall.

**3.6 Structured output.** Grammar-constrained decoding (llguidance, already a
declared-but-unwired feature of our vendored `llama-cpp-2`) makes malformed
JSON structurally impossible; it fixes shape, not semantics. Consequence:
wire `llama-cpp-2/llguidance` into both mobile composite features; keep
in-prompt tool descriptions to name + one line; compile grammars once per
session; temp 0.0–0.2 on structured turns; post-parse semantic validation;
1–2 capped repair retries then fail closed.

## 4. Current-state review

### 4.1 know-me-system (Rust workspace)

What exists and is kept:

- **`InferenceProvider` trait seam** (`gen_ui_inference`): engines behind a
  trait; mobile = `llama-cpp-2` 0.1.151 vendored (Metal/Vulkan + mtmd feature
  gates in `gen_ui_ffi`); desktop lane `local-mistral` target-gated off mobile.
  The context fabric slots in as a sibling seam, not inside any engine.
- **Hybrid retrieval** (`gen_ui_db_graph`): SurrealDB `search::rrf` fuses
  vector + BM25 lanes; `rrf.rs` fuses the RELATE graph-hop lane (k=60,
  limit=20) with an explicitly documented constraint — RRF only for lanes that
  can agree; disjoint-ordered lanes score by hop distance. Kept as-is; the new
  pipeline wraps it (§10).
- **Embedder trait** (`embed.rs`): 384-dim, model-provenance check, batch API,
  `spawn_blocking` discipline. Chunking reuses it unchanged.
- **Atomic note ingest** (`corpus.rs`/`store.rs`): `memory_ingest` embeds each
  note whole; idempotent stable ids. Correct for short notes; there is **no
  document chunking anywhere** — the gap §9 fills.
- **Streaming chat loop** (`gen_ui_agent::chat`): OpenAI-shaped streaming with
  tool-call accumulation across chunks. It currently *assembles* context
  inline; after this spec it *requests* an assembled context from the builder.

Gaps this spec closes: no capability profiles; no budget arithmetic anywhere;
no chunking; no MMR/recency re-ranking; no history strategies; `llguidance`
declared in the vendored crate but absent from `local-llama-vulkan` /
`local-llama-metal` composites; no context telemetry.

### 4.2 cherry-studio fork (`src/renderer/.../contextStrategy.ts` + agent lane)

The fork defines exactly four managed strategies plus `none`, a layered
resolution model, and telemetry. Honest implementation status, verified in
source:

| Strategy | Config | Chat apply path | Persistence |
|---|---|---|---|
| `sliding_window` | `maxMessages` (else dynamic by token budget) | **Fully wired** — `takeLastMessages` | — |
| `truncate_middle` | `keepFirstMessages=2`, `keepLastMessages=4`, `showOmissionMarker` | **Fully wired** — first+last slices | — |
| `summarize` | `summarizationModelId`, `summaryMaxTokens=500`, `summarizeThreshold=6` | Falls back to sliding window in the synchronous path | `TopicContextMetadata.conversationSummary`, `summarizedMessageIds` |
| `hierarchical` | `shortTermTurns=5`, `midTermSummaryTokens=2000`, `longTermFactsTokens=500` | Falls back likewise | `TopicContextMetadata.longTermFacts`, `factsUpdatedAt` |

Additional fork mechanics ported into the fabric:

- **Layered resolution**: `resolveEffectiveChatContextStrategy` merges
  global → assistant → topic overrides key-by-key; agents default to
  `{ type: 'none' }` and opt in separately. Ported as `StrategyResolver` (§8.1).
- **Agent pre-compact**: `shouldRunSdkCompactBeforeTurn` compares the *prior
  turn's* total-token usage (usage cache) against `compactTriggerTokens` and
  runs a compaction step before the next user message. Ported as the
  `PreCompactPolicy` on cloud/agent sessions (§8.6).
- **Telemetry**: `ContextManagementStreamPayload` (tokensBefore/After/Saved,
  messagesRemoved, alterationSummary, trigger). Ported as
  `ContextTelemetryEvent` on the AG-UI stream (§12).
- **Hygiene filters**: the fork runs a fixed filter chain (context-clear,
  useful-message, error-pair, adjacent-user, user-role-start) *before* the
  strategy. The fabric keeps the same two-phase shape: normalize → strategize.
- **Token estimation**: the fork uses chars/4. Acceptable for Electron; not
  acceptable as the only estimator on-device (§6.5).

The port obligation: **all four strategies become real in Rust** — the fork's
fallback behavior for `summarize`/`hierarchical` is a TypeScript-renderer
limitation, not a design decision to preserve.

## 5. Architecture

### 5.1 Crate placement

New workspace crate **`gen_ui_context`** (Rust, no engine deps, `no_std`-clean
core not required but wasm32-clean **is**):

```
gen_ui_types ──► gen_ui_context ──► consumed by: gen_ui_agent, gen_ui_host,
                     │                gen_ui_server_axum, gen_ui_ffi,
                     │                gen_ui_wasm, tauri-plugin-gen-ui
                     ├── depends on: gen_ui_db_graph (retrieval traits only)
                     └── never depends on: gen_ui_inference engines
```

Capability inversion holds: `gen_ui_context` knows *about* budgets and
retrieval capabilities through traits; it never links an inference engine, and
no engine crate links it. `gen_ui_agent` asks the builder for an
`AssembledContext`; the builder asks injected providers (history store, memory
store, KB store, tool registry, token estimator) for slices. Compile-time
dependency direction is the guarantee, per the standing Fabric invariant.

### 5.2 Module layout

```
gen_ui_context/
├── src/
│   ├── lib.rs
│   ├── capability/        # §6 — Device/Model/Runtime profiles, resolver
│   │   ├── device.rs
│   │   ├── model.rs
│   │   ├── runtime.rs
│   │   ├── resolve.rs     # → EffectiveCapabilities
│   │   └── presets.rs     # named tiers (§7)
│   ├── budget/            # §7 — BudgetPlan, slices, enforcement
│   │   ├── plan.rs
│   │   ├── ledger.rs      # running spend during assembly
│   │   └── policy.rs      # overflow policies
│   ├── strategy/          # §8 — the four fork strategies + resolver
│   │   ├── mod.rs         # ContextStrategy trait
│   │   ├── sliding_window.rs
│   │   ├── truncate_middle.rs
│   │   ├── summarize.rs
│   │   ├── hierarchical.rs
│   │   └── resolver.rs    # global→assistant→session override merge
│   ├── chunking/          # §9
│   │   ├── mod.rs         # ChunkingStrategy trait + router
│   │   ├── semantic.rs    # paragraph/boundary splitter
│   │   ├── recursive.rs   # size-targeted with overlap
│   │   └── parent_child.rs
│   ├── retrieval/         # §10 — pipeline around gen_ui_db_graph
│   │   ├── pipeline.rs    # lanes → RRF → MMR → recency → placement
│   │   ├── mmr.rs
│   │   └── recency.rs
│   ├── assemble/          # §11 — the ContextBuilder itself
│   │   ├── builder.rs
│   │   ├── placement.rs
│   │   └── render.rs      # → engine-neutral PromptPlan
│   ├── tokens/            # §6.5 — TokenEstimator trait + impls
│   └── telemetry.rs       # §12
└── tests/                 # golden assembly tests, property tests
```

### 5.3 Platform adapters (thin, config-only)

| Surface | Adapter | What differs |
|---|---|---|
| Mobile (Flutter/FFI) | `gen_ui_ffi` exposes `context_resolve_profile` + `context_assemble` via flutter_rust_bridge | Loads `mobile-4b-android` / `mobile-9b-ios` presets; runtime probe fills DeviceProfile |
| Desktop (Tauri 2) | `tauri-plugin-gen-ui` command | `desktop-local` preset; Metal/large-RAM numbers |
| Web | `gen_ui_wasm` | `web-wasm` preset; estimator = tokenizer-in-wasm or chars-heuristic fallback |
| Cloud (Axum) | `gen_ui_server_axum` middleware | `cloud-server` preset; provider-declared context sizes via liter-llm metadata |

No adapter contains budget logic. An adapter that needs an `if platform`
inside the pipeline is a spec violation.

## 6. Capabilities model

Three declared layers plus one measured layer, resolved into one effective
struct. Declared layers are data (TOML/JSON, shippable, overridable);
measurement adjusts at runtime.

### 6.1 DeviceProfile — what the hardware can sustain

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub id: String,                     // "iphone-17-pro-max", "gcp-l4", "browser"
    pub class: DeviceClass,             // MobileHigh | MobileMid | Desktop | Web | Server
    pub ram_mb: u32,
    pub sustained_infer_class: SustainedClass, // Bursty | Sustained | Unlimited
    pub thermal: ThermalPolicy,         // §6.4
    pub battery_aware: bool,
}
```

### 6.2 ModelProfile — what the weights can do

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub id: String,                     // "qwen3.5-9b-q4km"
    pub family: String,
    pub params_b: f32,
    pub quant: QuantTier,               // F16 | Q8 | Q5 | Q4 | Cloud
    pub nominal_context: u32,           // 262_144
    pub trusted_context: u32,           // 8_192 — empirically defended ceiling
    pub extended_context: Option<u32>,  // 16_384 behind degraded-mode flag
    pub attention: AttentionKind,       // Dense | HybridGdn { full_attn_layers: u8 }
    pub supports_vision: bool,          // mtmd projector present
    pub tool_call_format: ToolCallFormat, // Qwen35 | Hermes | OpenAI | ...
    pub thinking_modes: ThinkingModes,  // None | Toggleable | Always
}
```

`trusted_context` is the load-bearing field: it is set by the Phase 0 needle
test (§14, P0-3), not copied from a model card. `nominal_context` exists only
so the UI can explain the difference to a curious user.

### 6.3 RuntimeProfile — what the engine build offers

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeProfile {
    pub engine: EngineKind,             // LlamaCpp | MistralRs | WebLlm | CloudApi
    pub grammar_constrained: bool,      // llguidance wired?
    pub kv_cache_quant: Option<KvQuant>,
    pub streaming: bool,
    pub mtmd: bool,
    pub tokenizer: TokenizerRef,        // exact tokenizer for the estimator
}
```

### 6.4 SessionEnvelope — the measured layer

Mutable per-session state the resolver consults on every turn:

```rust
pub struct SessionEnvelope {
    pub consecutive_inferences: u32,
    pub session_infer_ms: u64,
    pub thermal_state: ThermalState,    // Nominal | Warm | Hot (OS-reported where available)
    pub last_ttft_ms: Option<u64>,
    pub last_total_tokens: Option<u32>, // feeds PreCompactPolicy (fork parity)
}
```

Envelope rules (deterministic, tunable per DeviceProfile): after
`max_consecutive_inferences` (default 12 mobile) or `thermal_state == Hot`,
the resolver downgrades — shorter output reserve, fewer RAG chunks, then
recommends cloud handoff via a typed `EscalationAdvice` the host surfaces.
This encodes §3.5 as mechanism instead of hope.

### 6.5 TokenEstimator

`trait TokenEstimator { fn estimate(&self, text: &str) -> u32; fn kind(&self) -> EstimatorKind; }`
Implementations: `ExactTokenizer` (llama.cpp vocab via FFI on-device; HF
tokenizer server-side), `CharsHeuristic` (chars/4, fork parity, web fallback).
The plan records which estimator produced its numbers; safety margin widens
from 10% → 15% when the heuristic is in play. Cross-platform consistency
requires estimating with the *model's own* tokenizer wherever possible —
Qwen3.5's 248K vocab tokenizes materially differently than chars/4 assumes.

### 6.6 Resolution

```rust
pub fn resolve(
    device: &DeviceProfile,
    model: &ModelProfile,
    runtime: &RuntimeProfile,
    envelope: &SessionEnvelope,
    overrides: &BudgetOverrides,        // user/agent/session layered, fork-style
) -> EffectiveCapabilities
```

Order: preset lookup (device×model) → derivation rules (§7.2) → envelope
degradation → override merge (only *tightening* overrides accepted from
non-privileged layers, mirroring the Cedar posture: product supplies
defaults, users may tighten). Result is cached per (profiles, envelope-bucket)
and logged once per session for reproducibility.

## 7. Budget abstraction

### 7.1 BudgetPlan — the contract every platform honors

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetPlan {
    pub context_ceiling: u32,           // = trusted_context unless extended flag

    // Token slices (hard caps)
    pub system_prompt_max: u32,
    pub tool_schema_max: u32,
    pub history_max: u32,
    pub memory_max: u32,                // user memories slice
    pub knowledge_max: u32,             // dropped-notes / KB RAG slice
    pub output_reserve: u32,
    pub thinking_reserve: u32,          // 0 when thinking disabled this turn
    pub safety_margin_pct: u8,          // 10 exact-tokenizer / 15 heuristic

    // Item-count caps (independent of tokens)
    pub max_tool_shortlist: u8,         // tools exposed to one call
    pub max_memory_items: u8,
    pub max_kb_retrievals: u8,          // chunks after re-ranking
    pub max_history_messages: Option<u16>,

    // Retrieval tuning
    pub chunk_target_tokens: u16,       // 250–400 band
    pub rerank: RerankPlan,             // §10
    pub placement: PlacementPolicy,     // §10.3

    // Agentic caps (§3.4)
    pub max_autonomous_steps: u8,       // before checkpoint
    pub mutating_action_gate: GatePolicy, // AlwaysConfirm (non-negotiable default)
}
```

### 7.2 Derivation rules

Given `C = context_ceiling` after safety margin:

1. `output_reserve` and `thinking_reserve` come off the top (never squeezed).
2. `system_prompt_max` + `tool_schema_max` are fixed by tier.
3. `history_max` fixed by tier; strategy (§8) keeps history inside it.
4. `memory_max + knowledge_max = remainder`, split by tier ratio
   (default 30/70 — dropped notes are the larger corpus; memories are short).
5. Invariant checked at resolve time:
   `Σ slices + margin ≤ C` — violation is a config error, not a runtime clamp.

### 7.3 Named tier presets (initial values)

Numbers for the two mobile tiers carry over from the budget analysis earlier
in this design cycle; cloud/desktop are set generously but still finite —
consistency of *shape* across platforms is the product requirement.

| Field | mobile-4b (S25U) | mobile-9b (iPhone 17 PM) | desktop-local | web-wasm | cloud-server |
|---|---|---|---|---|---|
| context_ceiling | 8 192 | 8 192 | 16 384 | 8 192 | 32 768¹ |
| system_prompt_max | 500 | 600 | 1 200 | 600 | 2 000 |
| tool_schema_max | 200 | 300 | 800 | 300 | 1 500 |
| history_max | 1 200 | 1 800 | 4 000 | 1 800 | 8 000 |
| memory_max | 600 | 850 | 1 800 | 850 | 3 000 |
| knowledge_max | 1 400 | 1 950 | 4 200 | 1 950 | 7 000 |
| output_reserve | 768 | 1 024 | 2 048 | 1 024 | 4 096 |
| thinking_reserve (when on) | 512 | 768 | 1 536 | 512 | 4 096 |
| safety_margin_pct | 10 | 10 | 10 | 15 | 10 |
| max_tool_shortlist | 4 | 6 | 8 | 5 | 8 |
| max_memory_items | 3 | 4 | 6 | 4 | 8 |
| max_kb_retrievals | 3 | 5 | 8 | 5 | 10 |
| chunk_target_tokens | 250 | 320 | 400 | 320 | 400 |
| max_autonomous_steps | 3 | 4 | 6 | 3 | 8 |

¹ Cloud ceiling is per-model via liter-llm metadata; 32K is the floor preset.
Thinking reserve is carved from output+knowledge when toggled on for a turn —
the resolver emits a per-turn variant plan, so "thinking on" visibly costs RAG
chunks rather than silently overflowing.

### 7.4 Enforcement — the BudgetLedger

Assembly runs against a `BudgetLedger` that debits each slice as content is
admitted. Policies per slice: `Hard` (refuse and record), `Shrink` (drop
lowest-ranked items first — retrieval slices), `Summarize` (history slice,
strategy-mediated). Overflow at final render is a **defect**: the ledger makes
it unreachable by construction, and a debug assertion + telemetry counter
back that claim in CI and in the field.

## 8. Context-management strategies (fork port, budget-aware)

### 8.1 Trait and resolver

```rust
#[async_trait]
pub trait ContextStrategy: Send + Sync {
    fn kind(&self) -> StrategyKind; // None|SlidingWindow|TruncateMiddle|Summarize|Hierarchical
    async fn apply(
        &self,
        history: Vec<Turn>,
        cfg: &StrategyConfig,
        ledger: &mut BudgetLedger,   // history slice only
        cx: &StrategyCx<'_>,         // estimator, summarizer handle, persisted metadata
    ) -> Result<StrategyOutcome, ContextError>;
}

pub struct StrategyOutcome {
    pub turns: Vec<Turn>,
    pub summary_updated: Option<SummaryRecord>,
    pub facts_extracted: Vec<FactRecord>,
    pub removed: u32,
    pub tokens_saved: u32,
    pub was_applied: bool,
}
```

`StrategyResolver` reproduces the fork's layered merge exactly: defaults →
global settings → assistant/agent config → topic/session override, key-by-key,
with `hasOverride` semantics preserved so a partial override doesn't clobber
unrelated fields. Agent sessions default to `None` + explicit opt-in, as in
the fork. The hygiene filter chain (context-clear, empty, error-pair,
adjacent-user, user-role-start) runs before any strategy, ported as pure
functions with the fork's tests translated to Rust.

### 8.2 SlidingWindow

Token-budget-first (the fork's "dynamic" mode becomes the primary mode):
admit turns newest→oldest until `history_max` or `max_history_messages` is
hit; `maxMessages` remains as an optional cap for fork parity. Never splits a
turn; never separates a tool call from its result (a constraint the fork
handles upstream via filters — here it is a builder invariant).

### 8.3 TruncateMiddle

`keep_first` (default 2) + `keep_last` (default 4) with an explicit omission
marker turn (`[… N earlier messages omitted …]`) when `show_omission_marker`.
Marker text is part of the token math, not free.

### 8.4 Summarize (progressive)

Real implementation, not fallback. Trigger: history over `history_max` AND
turn count ≥ `summarize_threshold` (6). The oldest overflow block is
summarized into ≤ `summary_max_tokens` (500) by a **summarizer route**:
on-device it reuses the loaded model with a short constrained prompt at temp
0.2 (one extra inference — the SessionEnvelope charges it); cloud/desktop may
route to a designated small model (fork's `summarizationModelId`).
`SummaryRecord` persists (SurrealDB, §11.3) with the covered turn-ids so
re-summarization is incremental — fork's `summarizedMessageIds` parity.
Failure mode honored: if the summarizer call fails or the envelope forbids the
extra inference, degrade to SlidingWindow for this turn and record it in
telemetry; never block the user's message on a summary.

### 8.5 Hierarchical (three-tier) — the KnowMe-native strategy

Short-term: last `short_term_turns` (5) verbatim. Mid-term: rolling summary
inside `mid_term_summary_tokens` (2000 desktop/cloud; clamped to the history
slice on mobile). Long-term: extracted durable facts within
`long_term_facts_tokens` (500). The KnowMe twist: **long-term facts are not a
parallel store** — extraction writes them through the existing memory ingest
(`gen_ui_db_graph::memory_ingest`, kind `"fact"`, provenance = session id),
so hierarchical memory and user memories converge on one governed store and
facts become retrievable by the same RRF pipeline in future sessions. This is
the mechanism that makes "it knows the user" compound over time. Extraction is
opt-in per assistant and surfaced in the UI (sovereignty: users see what was
written and can delete it; deletion respects the standing memory-removal path).

### 8.6 PreCompactPolicy (agent/cloud lane)

Fork parity: cache the prior turn's total-token usage; when it exceeds
`compact_trigger_tokens`, run a compaction step (strategy-mediated summarize)
before dispatching the next user message. On-device this maps to
"force-summarize now"; on cloud SDK sessions it maps to the provider compact
primitive where one exists.

## 9. Chunking subsystem

### 9.1 Router

Content entering the knowledge path is routed by size and structure:

| Condition | Strategy | Notes |
|---|---|---|
| ≤ 1.5 × chunk_target_tokens | **Atomic** | Current `memory_ingest` path unchanged — short notes stay whole |
| Prose above threshold | **SemanticBoundary** | Split on paragraph/heading boundaries near target size; never mid-sentence |
| Unstructured / very long | **RecursiveOverlap** | Size-targeted splits with 10–15% overlap |
| Long docs where precision matters | **ParentChild** | Embed child chunks (target size); store parent window reference; inject parent context at assembly |

### 9.2 Data model

```rust
pub struct ChunkRecord {
    pub id: String,                  // stable: {doc_id}#c{ordinal}
    pub doc_id: String,
    pub ordinal: u32,
    pub text: String,
    pub token_len: u32,              // by the ingest-time estimator
    pub parent_span: Option<(u32, u32)>, // char range in parent for window fetch
    pub kind: String,                // extends MemoryRecord.kind: "chunk"
    pub source: ChunkProvenance,     // doc title, ingest ts, origin (dropped|scribe|fact)
}
```

SurrealDB (3.x strict syntax) additions live beside the existing memory table;
chunks embed through the same `Embedder` (batch API, `spawn_blocking`), HNSW
`DIMENSION 384` unchanged. Ingest of an N-chunk doc is one transaction;
re-ingest of the same `doc_id` replaces its chunk set (stable ids ⇒
idempotent, matching `seed_corpus` discipline).

### 9.3 Injection

At assembly, a selected child chunk may be widened to its parent span if and
only if the knowledge ledger can afford it; widening is ranked (best hit
widens first). Provenance flows into `ContentBlock::Citation`/`Memory` so the
UI can show *which note* grounded an answer — a sovereignty feature, not just
debugging.

## 10. Retrieval & re-ranking pipeline

### 10.1 Stages

```
query ──► lanes (vector | BM25 | graph-hop)      [existing gen_ui_db_graph]
      ──► RRF fuse (k=60, limit=20)              [existing, unchanged]
      ──► MMR diversity pass (λ default 0.7)     [new — embedding math only]
      ──► recency boost: score × exp(−λr·age_d)  [new — λr per corpus kind]
      ──► truncate to max_kb_retrievals / max_memory_items
      ──► optional parent widening (§9.3)
      ──► placement (§10.3)
```

MMR uses the vectors already fetched for the RRF candidates — zero extra
model calls, bounded O(k²) at k ≤ 20. Recency defaults: memories λr = 0.01/day
(gentle), facts 0.005, chunks 0 unless the doc kind opts in. Both passes are
pure functions with property tests.

### 10.2 Memory vs knowledge lanes

Memories and knowledge chunks are retrieved as **separate pipeline runs
against separate ledger slices** (`memory_max` / `knowledge_max`). One shared
run would let a large dropped document starve the user's explicit memories —
the exact inversion of the product promise.

### 10.3 PlacementPolicy

Per §3.3: rank-1 item is placed *last* in its section (adjacent to the user
query); rank-2 first; remaining ranks fill the middle. Sections order:
system → tools → long-term facts → history → memories → knowledge → query.
Placement is deterministic and covered by golden tests: same inputs, same
bytes.

### 10.4 Cross-encoder gate (deferred)

Add a reranker model only if, after MMR+recency ship, retrieval precision is
still the measured bottleneck (eval set: ≥50 real query/expected-note pairs;
precision@3 target 0.8). If added: runs over RRF top-15–20 only, desktop/cloud
first; mobile only if the envelope affords it. Until that gate, the extra
forward passes buy less than they cost on-device.

## 11. Assembly pipeline & data models

### 11.1 End-to-end turn sequence

```
1  resolve EffectiveCapabilities (profiles + envelope + overrides)   [cached]
2  derive per-turn BudgetPlan variant (thinking on/off, degraded?)
3  tool stage: registry → relevance shortlist (≤ max_tool_shortlist)
      → minimal one-line descriptions in-prompt
      → full schemas compiled to grammar (llguidance) once per session
4  history stage: hygiene filters → resolved ContextStrategy.apply(ledger)
5  memory stage: retrieval pipeline against memory_max slice
6  knowledge stage: retrieval pipeline against knowledge_max slice (+widening)
7  assemble: PlacementPolicy orders sections; ledger proves Σ ≤ ceiling
8  render: engine-neutral PromptPlan → engine adapter maps to chat template
9  generate (structured turns: grammar-constrained, temp ≤ 0.2)
10 post: semantic validation → ≤2 repair retries → fail closed
11 telemetry event; envelope update; PreCompactPolicy check for next turn
```

Steps 1–8 are synchronous, deterministic, engine-free. Only 3 (grammar
compile) and 4 (summarize path) may incur model work, both envelope-charged.

### 11.2 Core assembly types

```rust
pub struct AssembleRequest {
    pub user_turn: Turn,
    pub session_id: String,
    pub assistant_id: String,
    pub thinking_requested: bool,
    pub overrides: BudgetOverrides,
}

pub struct AssembledContext {
    pub plan: BudgetPlan,               // the variant actually used
    pub prompt: PromptPlan,             // ordered sections, engine-neutral
    pub tool_shortlist: Vec<ToolRef>,
    pub grammar: Option<GrammarHandle>,
    pub telemetry: ContextTelemetryEvent,
    pub escalation: Option<EscalationAdvice>,
}

pub struct PromptPlan {
    pub sections: Vec<Section>,         // (SectionKind, Vec<RenderedItem>)
    pub estimated_tokens: u32,
    pub estimator: EstimatorKind,
}
```

`PromptPlan` is the cross-platform contract: Dart/TS mirrors are generated
(flutter_rust_bridge / ts-rs) so every UI can display *exactly* what will be
sent — the transparency surface the sovereignty story requires.

### 11.3 SurrealDB additions (3.x strict)

```sql
DEFINE TABLE chunk SCHEMAFULL;
DEFINE FIELD doc_id     ON chunk TYPE string;
DEFINE FIELD ordinal    ON chunk TYPE int;
DEFINE FIELD text       ON chunk TYPE string;
DEFINE FIELD token_len  ON chunk TYPE int;
DEFINE FIELD kind       ON chunk TYPE string;
DEFINE FIELD provenance ON chunk TYPE object;
DEFINE FIELD embedding  ON chunk TYPE array<float>;
DEFINE INDEX chunk_hnsw ON chunk FIELDS embedding HNSW DIMENSION 384;

DEFINE TABLE context_summary SCHEMAFULL;   -- SummaryRecord (fork TopicContextMetadata parity)
DEFINE FIELD session_id  ON context_summary TYPE string;
DEFINE FIELD text        ON context_summary TYPE string;
DEFINE FIELD covered_ids ON context_summary TYPE array<string>;
DEFINE FIELD updated_at  ON context_summary TYPE datetime;

DEFINE TABLE context_turn_log SCHEMAFULL;  -- telemetry ring (bounded, local-only)
DEFINE FIELD session_id ON context_turn_log TYPE string;
DEFINE FIELD event      ON context_turn_log TYPE object;
DEFINE FIELD at         ON context_turn_log TYPE datetime;
```

Writes use `type::record($table, $key)` and two-arg UPDATE form per standing
SurrealDB 3.x discipline; no `id` field inside content objects. Facts from
§8.5 go to the existing memory table (kind `"fact"`), not a new table.

## 12. Telemetry & consistency guarantees

`ContextTelemetryEvent` (Rust mirror of the fork payload, extended):
`strategy_kind, tokens_before, tokens_after, tokens_saved, messages_removed,
slices: Vec<(SliceKind, budget, spent, items)>, estimator, degraded: bool,
escalation: Option<…>, trigger`. Emitted on the AG-UI stream (`agui.*`
namespace) so any surface can render a context inspector; persisted to the
bounded local ring only. Guarantees the fabric makes testable: (G1)
byte-deterministic assembly; (G2) ledger-proven no-overflow; (G3) tool
call/result pairing never split; (G4) mutating-action gate cannot be disabled
by non-privileged override layers; (G5) memory slice cannot be starved by
knowledge slice.

## 13. Failure scenarios & trade-offs (what hurts)

Per standing discipline, the scenarios that hurt this design, named:

1. **Token-estimation drift.** chars/4 on Qwen3.5's 248K vocab can misestimate
   by 20–30% on code/multilingual text; even the exact tokenizer path costs an
   FFI round-trip per slice. Mitigation is the widened margin + estimator
   telemetry, but a systematic underestimate still means silent quality loss
   near the ceiling. Residual risk: **medium** until P0-2 measures real drift.
2. **Summarization on-device is an extra inference.** It competes with the
   thermal envelope and adds latency exactly when conversations are longest.
   The degrade-to-sliding-window path caps the damage but means the summarize
   strategy silently behaves like sliding window on hot devices — users may
   perceive memory loss. Telemetry makes it visible; it does not make it free.
3. **GDN × Q4 interaction is uncharacterized.** If the P0-3 needle test shows
   retrieval breaking below 8K on either GGUF, every tier table in §7.3
   shrinks and the product's on-device RAG story narrows. Probability
   subjectively 20–35%; impact contained because `trusted_context` is a data
   field, not a constant.
4. **Two token vocabularies of truth.** Fork-era chats managed by chars/4 and
   fabric-era chats managed by exact counts will window differently; users
   migrating desktop→mobile may see different history depth. Accepted;
   documented in release notes rather than papered over.
5. **Preset ossification.** Hardcoded tier numbers rot as models/devices
   change. Mitigation: presets are data files versioned with the app +
   overridable server-side for cloud tier; the needle test is rerun per new
   GGUF as a release gate.
6. **The scenario that hurts Prometheus:** a competitor ships "just use the
   cloud" with a 200K window and no budget UX, and it *feels* smarter on day
   one. Our counter is compounding personal memory (§8.5) + sovereignty +
   consistency — but that is a bet on retention economics, not an immediate
   demo win, and it should be stated as such to stakeholders.

## 14. Project plan

Phases gate on KDD discipline: each ends in a reviewable artifact + explicit
approval before the next begins. Estimates are ranges at stated confidence;
they assume one primary implementer with review support.

**Phase 0 — Spikes & gates (3–5 days, 80% confidence)**
- P0-1: Wire `llama-cpp-2/llguidance` into `local-llama-metal` /
  `local-llama-vulkan`; smoke a constrained JSON tool call on both devices.
  *Gate: overhead ≤ 10% tok/s.*
- P0-2: Estimator drift measurement — exact tokenizer vs chars/4 over a 50-doc
  corpus (notes, code, mixed-language). *Gate: margin table confirmed.*
- P0-3: Needle-in-haystack on actual Q4_K_M GGUFs (facts at 10/40/70/95%
  depth × 4K/8K/16K, both devices). *Gate: `trusted_context` values set from
  data.*
- P0-4: SessionEnvelope thermal probe feasibility per OS (thermal state APIs
  on iOS/Android; fallback = inference-count-only).

**Phase 1 — `gen_ui_context` core: capabilities + budgets (1.5–2.5 wks, 75%)**
Profiles, presets, resolver, BudgetPlan/Ledger, TokenEstimator impls, golden
tests. Exit: G1/G2 proven in CI; presets reviewed against P0 data.

**Phase 2 — Strategies + assembly (2–3 wks, 70%)**
Hygiene filters, four strategies (summarize/hierarchical real), resolver
layering with fork tests ported, PromptPlan render, telemetry event.
Exit: fork-parity test suite green; `gen_ui_agent::chat` consumes the builder
behind a feature flag.

**Phase 3 — Chunking + retrieval upgrades (1.5–2 wks, 75%)**
Chunk router/strategies, SurrealDB schema, MMR + recency passes, placement,
memory/knowledge slice separation, citation provenance to ContentBlock.
Exit: precision@3 eval harness exists with baseline numbers recorded.

**Phase 4 — Platform adapters (1–2 wks, 70%)**
FFI surface (flutter_rust_bridge), Tauri command, wasm build, Axum middleware;
preset loading + envelope probes per platform. Exit: same golden assembly
bytes on all four targets in CI.

**Phase 5 — Field hardening (1–1.5 wks, 60%)**
Context inspector UI hook-up, degraded-mode UX, PreCompactPolicy on cloud
lane, telemetry ring bounds, release-gate automation for P0-3 reruns.

Critical path: P0-3 → §7.3 numbers → everything. Total: **7–11 weeks**.
Explicitly out: cross-encoder (gated, §10.4), tool-calling LoRA.

## 15. Open decisions (need your call before Phase 1)

1. Memory/knowledge remainder split — 30/70 proposed; confirm or set per-tier.
2. Fact extraction default: opt-in per assistant (proposed) vs global opt-in.
3. Summarizer route on desktop: same loaded model vs designated small model.
4. Preset distribution for cloud tier: baked vs server-fetched with pinning.
5. Whether `extended_context` (16K) ships in v1 UI at all, or stays a dev flag.

## 16. References (session research base)

RULER / effective-context and hybrid-attention analyses (Qwen3.5 model cards
and config.json, HF; Gated DeltaNet deep-dives); "Does quantization affect
models' performance on long-context tasks?" (arXiv 2505.20276); lost-in-the-
middle SLM survey (arXiv 2409.15790); SABER mutating-action analysis (arXiv
2512.07850); long-horizon execution / self-conditioning (arXiv 2509.09677);
error-compounding formalization (arXiv 2601.22290); Tau-Bench mock SLM result
(arXiv 2601.12979); TinyAgent/ToolRAG (arXiv 2409.00608) and adaptive context
management 6×/10–25× reductions (OpenReview); MemTool (arXiv 2507.21428);
fastWorkflow (ACM AIAS 2026); tool-count degradation & Anthropic 58-tool
measurement (Composio 2026 guide); mobile thermal studies (arXiv 2603.23640,
2410.03613, 2606.23001; MNN-AECS); on-device RAG TTFT (arXiv 2602.13229).
Cherry Studio fork sources: `src/renderer/src/types/contextStrategy.ts`,
`services/chatContextStrategy.ts`, `main/.../agentContextStrategy/*`.

— End of PAGS-SPEC-KM-CTX-001 —
