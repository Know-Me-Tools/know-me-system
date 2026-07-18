# Provider & Model Configuration Strategy

**Scope:** how KnowMe configures LLM/embedding providers and models across three
resolution levels — **global defaults**, **agent/assistant** (client- or server-side),
and **conversation** — plus embedding-model management for RAG indexes (knowledge
bases, karpathy wikis, logs, loops) and usage-driven suggestions.

**Status:** v1.0 strategy, 2026-07-18. Companion to
`docs/strategies/local-models/README.md` (acquisition, library UX) and
`docs/knowme-builder-extensibility-spec.md` (settings scopes/sync). Studied against
Cherry Studio, which handles this unusually well; we adopt its load-bearing ideas and
diverge where KnowMe's multi-device sovereignty changes the problem.

---

## 1. What Cherry Studio gets right (and what we adopt)

From the [Cherry Studio default-models docs](https://docs.cherryai.com.cn/docs/en-us/pre-basic/settings/default-models.md),
[chat docs](https://docs.cherryai.com.cn/docs/en-us/cherry-studio/preview/chat), and the
[model-configuration internals](https://deepwiki.com/CherryHQ/cherry-studio/5.3-model-configuration-and-capabilities):

1. **Role-slot defaults, not one global model.** Separate defaults for the *default
   assistant model*, a *fast model* for background chores (topic naming, keyword
   extraction), and a *translation model* — "the models used by the behind-the-scenes
   helpers," explicitly distinct from the chat model. **Adopt**, with more roles (§3).
2. **Assistant = role, Topic = conversation with that role.** The assistant pins the
   system prompt, model, and sampling params; topics inherit; any topic can override;
   `@`-mention temporarily borrows another model *for one reply* while keeping context;
   an **auto-reset** policy decides whether new topics revert to the assistant's model
   or inherit the last override. **Adopt all four behaviors verbatim** (§4).
3. **Deterministic model identity: `providerId::modelId`.** One key format everywhere
   (their `user_model` table), with per-model `capabilities`, `userOverrides`, and
   `orderKey`. **Adopt** — it is exactly what our entity graph and OFP sync want (§2).
4. **Capability tags with a resolution precedence.** vision / function-calling /
   reasoning / embedding / rerank per model; resolved as *registry-force → user
   overrides → registry add/remove*, so user removals survive registry updates.
   **Adopt precedence unchanged** (§2.3).
5. **Provider-specific settings are schema, not special-cased UI** (Azure api-version,
   Vertex project, custom OpenAI-compatible base URLs). **Adopt** via our
   settings-schema renderer (§2.2) — one generic form engine, per-provider schemas.

Where we must diverge: Cherry Studio is a single-machine app. KnowMe's config must
survive **multiple devices with different local capabilities plus shared peers** —
that changes identity, resolution, and sync (§5).

## 2. The registry

### 2.1 Providers

```jsonc
{
  "id": "prov-anthropic",             // stable, user-scoped
  "kind": "anthropic",                 // local-engine | bossfang | anthropic | openai |
                                       // openai-compatible | ollama | azure-openai | vertex | …
  "label": "Anthropic",
  "base_url": null,                    // overridable where kind allows
  "auth_ref": "keychain://prov-anthropic",  // secrets NEVER in the settings doc —
                                       // OS keychain via Tauri secure storage / Flint Vault server-side
  "settings": { /* validated by the provider-kind's JSON Schema */ },
  "health": { "state": "ok", "checked_at": "…", "latency_ms": 240 }
}
```

Two provider kinds are KnowMe-specific and **auto-registered, never hand-configured**:

- **`local-engine`** — the in-process llama.cpp/mistral.rs runtime; its model list *is*
  the local model library (local-models §12). Exists on every device; contents differ
  per device.
- **`bossfang`** — one provider per connected instance from Settings → Instances,
  created/removed with the instance itself. Its model list is served by the peer
  (the cyan `remote` rows in the moodboard). Health = OFP reachability.

Cloud kinds route through liter-llm in UAR (FUNC-SPEC §10's 142+ providers) but are
*presented* as first-class providers with their own rows, keys, and health checks.
Every provider row: status dot · label · base host · kind pill · "Check" button
(connectivity + auth + a 1-token echo), matching the moodboard Providers tab.

### 2.2 Provider-kind schemas

Each `kind` ships a JSON-Schema for its settings (api-version for Azure, project/region
for Vertex, org id for OpenAI, TLS options for custom). The generic schema→form
renderer from the extensibility spec draws the UI; adding a provider kind (including
via a `.pxt` extension exporting a `provider` capability) never requires new settings
UI code. This is how we stay "Cherry-class" on provider coverage without their
hand-built per-provider screens.

### 2.3 Models

Key: **`provider_id::model_id`** (adopted). Record: capability set (`chat`, `vision`,
`tools`, `reasoning`, `embedding`, `rerank`), context length, pricing (cloud) or
size/quant + measured tok/s (local, from the benchmark gate), `enabled`, `order_key`,
user overrides. Capability resolution: **registry force → user overrides → registry
add/remove** (adopted). The registry layer is fed by: the signed KnowMe catalog (local
models), the provider's `/models` listing (cloud), and the peer's advertisement
(bossfang). Users can hand-add models a provider doesn't list (custom deployments).

## 3. Global level — role slots

Cherry Studio's insight, extended. Global settings bind **roles** to model refs:

| Role slot | Used by | Default guidance |
|---|---|---|
| `chat.default` | New assistants/conversations without a pin | The §10 device recommendation (local) |
| `utility.fast` | Topic naming, summarization, memory extraction, digest drafting, karpathy log/loop summarization | Small + cheap + local: Qwen3 1.7–4B class |
| `agent.default` | New Hands without an explicit model | Tool-calling floor for the tier (14B+/MoE, or a peer) |
| `vision.default` | Ask Image, image drops in chat | Gemma E-class / 3 4B+ |
| `translation` | One-click translate | Any chat model; user preference |
| `embedding.default` | **Seed for new indexes only** (§6 — never applied retroactively) | EmbeddingGemma@768 (device-appropriate) |
| `rerank.default` | Optional second retrieval stage | Qwen3-Reranker-0.6B, desktop/server only |

Rules: every slot shows *provenance and fallback* ("local Qwen3 8B; falls back to
BossFang `home-mac` when unloaded"). Role slots are **user-scoped settings** — they sync
via the CRDT settings layer — but a slot bound to a `local-engine` model resolves
per-device (§5). The `utility.fast` slot is what keeps background chores (Cherry's
"behind-the-scenes helpers") off the expensive model — same separation, more consumers:
Hands digests, memory writes, wiki/log/loop maintenance all draw from it.

## 4. Agent and conversation levels

### 4.1 Agent/assistant level

An assistant (chat persona) or Hand pins: `model_ref` (or "inherit global"), sampling
params (temperature, top-p, max tokens, ctx), and — for Hands — the **deployment
target**, which scopes resolution: a Hand deployed to `home-mac` resolves its model
against *that instance's* providers at run time, not the authoring device's. This is
the client-side/server-side split the FUNC-SPEC's Hands wizard (Step 2 Model,
Step 3 Deployment) already sketches; the registry makes it precise: the wizard's model
picker filters to `capabilities ⊇ tools` and *the chosen target's* available models,
and warns when a pick is below the tool-calling floor for that target's hardware.

### 4.2 Conversation level

Adopted from Cherry Studio wholesale, in moodboard terms:

- The **model chip** at the top of Chat (already in Screen 02) is the per-conversation
  override: tap → selector grouped by provider, capability-filtered, with fit/trust
  pills carried over from the library. An override is **sticky for that conversation**
  and labeled with provenance: "Qwen3-14B — set for this conversation."
- **`@`-mention** in the composer borrows a model for a single reply with full context
  ("Mention model" behavior) — including *multiple* mentions for side-by-side answers;
  the transcript records which model produced which message (per-message model
  attribution is already implied by the moodboard's per-message thinking blocks).
- **Auto-reset policy** (per assistant, default ON): new conversations revert to the
  assistant's model; OFF inherits the last override. Exposed as one toggle, phrased in
  outcomes: "New conversations start with this assistant's model."

### 4.3 Resolution order (the whole system in six lines)

```
message-level  @mention            (one reply)
  → conversation override          (sticky, this topic)
    → assistant/Hand pin           (+ params; Hands: scoped to deployment target)
      → role slot                  (chat.default / agent.default / …)
        → device recommendation    (local-models §10, if the slot is unset)
          → ask the user           (never silently pick a cloud provider)
```

Every level may be unset; the effective choice always renders its source. One hard
rule from the sovereignty posture: **resolution never escalates locality on its own** —
a chain that resolves to a local model may fall back to a *peer*, but falling back to a
*cloud* provider requires an existing Cedar grant for that provider, or a prompt.

## 5. Multi-device semantics (where we go beyond Cherry Studio)

A pinned `local-engine` model may not exist on the next device. Resolution therefore
has a **placement walk** when the bound ref is unavailable *here*:

1. Same `model_id` on a reachable `bossfang` provider → use it (chip shows cyan
   `remote`);
2. Same catalog **equivalence class** locally (the signed catalog groups e.g.
   Qwen3 {4B, 8B, 14B} as tier-variants of one family; pick the device-appropriate
   tier and label it "device equivalent");
3. Offer: download it here (jumps into the local-models §2 flow) / pick another / use
   peer.

Sync scopes (per the extensibility-spec settings model): role slots and assistant pins
are **user-scoped** (CRDT-synced); the *placement* of `local-engine` refs and anything
bound to device hardware is **device-scoped**; provider auth is **secret** (keychain,
never CRDT — each device holds its own copy; BossFang instances may hold shared keys in
Flint Vault). Conversation overrides travel *with the conversation entity* through
normal OFP sync, and the placement walk re-runs on whatever device opens it.

## 6. Embedding models — per-index bindings, not a global knob

The `embedding.default` slot only *seeds creation flows*. The binding that matters
lives on the index, because vectors are only comparable within one model+dim
(local-models §11):

```jsonc
// stamped on every KB / karpathy wiki / log index / loop memory at creation
{
  "index_id": "kb-client-acme",
  "embedding": { "model_ref": "prov-local::embeddinggemma-300m", "dim": 256 },
  "rerank":    { "model_ref": null },
  "chunking":  { "strategy": "semantic", "target_tokens": 480, "overlap": 64 },
  "created_at": "…"
}
```

- **Immutable after creation** except through an explicit **re-embed migration**: a
  visible job (progress, cancellable, old index kept until the new one completes) —
  the honest version of what most tools hide. Cherry Studio binds the embedding model
  at KB creation for the same reason; we make the migration path first-class instead
  of impossible.
- **Creation wizard**: defaults from `embedding.default`, shows dim choice with the
  tradeoff stated ("256-dim: ~3× smaller/faster, slight recall cost — recommended on
  this device"), warns when the chosen model can't run on devices that sync this index
  ("your phone can embed queries with EmbeddingGemma but not BGE-M3 — queries will
  route to a peer").
- **Karpathy wikis / logs / loops** (the `karpathy-progress-memory` artifacts): each
  gets the same binding block. Their *maintenance* passes (summarize, distill, link)
  draw from `utility.fast`; their *retrieval* uses the index's own embedding binding;
  loops that grade/reflect may pin a reasoning-capable model explicitly — that pin is
  an agent-level setting on the loop's Hand, resolved like any other (§4.1).
- **Mismatch on sync = re-embed job, never merge** (restated from local-models §11);
  the index's `{model_ref, dim}` stamp is the guard.
- **Server-side indexes** (flint-forge/BossFang corpus): may bind heavier models
  (BGE-M3 hybrid) and optionally embed *in-database* via Flint Ember; devices keep a
  device-appropriate binding over their hot subset. Two indexes, two bindings, one
  query plan — the read path merges results, it does not pretend the vectors are
  comparable.

## 7. Usage-based suggestions

All signals are **local-only** (SurrealDB; no analytics pipeline — FUNC-SPEC §09), the
engine is rules over counters, and every suggestion is a dismissible card that
*explains itself and applies with one tap* — never an automatic switch.

Tracked per `(context, model_ref)`: use counts and recency; measured latency/tok/s;
error and **malformed-tool-call rates**; regenerate-with-a-different-model events (the
strongest preference signal: the user tried A, redid it with B); manual override
frequency per assistant/topic-class; token spend per cloud provider (priced from the
registry) vs. local; provider health history.

The cards these rules produce:

- "In coding conversations you switch to **Qwen3-Coder** 8 of the last 10 times —
  make it the Coding assistant's model?"
- "**Inbox Triage** has an 18 % malformed-tool-call rate on Qwen3-4B. Its target
  `home-mac` can run 30B-A3B at ~40 t/s — redeploy?" (agent-level, ties §4.1 to the
  tool-calling floor)
- "This month: $14.20 to Anthropic for summarization tasks your local
  `utility.fast` model handles within quality thresholds — route utility tasks local?"
- "Provider **Groq** failed 40 % of requests this week — health-check or disable?"
- "**Phi-4-mini** hasn't been used in 90 days — reclaim 2.3 GB?"
- After a benchmark: "Measured 47 t/s on the new model — 2.1× faster than your current
  chat default with equal capability tags. Swap?"

Suggestion cards land in the Models tile and (for agent-level ones) on the Hand's
detail card, in the moodboard's cyan-annotation style: claim, evidence, one action.

## 8. UI surface map (moodboard-anchored)

| Surface | Level | Contents |
|---|---|---|
| Settings → **Providers** (tab exists in Screen 05's rail) | registry | Provider rows (dot · label · host · kind pill · Check); detail sheet = schema-driven settings + key entry (keychain) + model list with enable toggles, capability pills, order |
| Settings → **Default Models** (new pane under Providers) | global | The §3 role-slot list, each row: slot name · bound model chip with provenance · fallback line |
| Assistant editor / Hands wizard Step 2 | agent | Model picker (capability- and target-filtered) · params · inherit toggle · floor warnings |
| Chat header model chip (Screen 02) | conversation | Sticky override selector, provenance label, auto-reset per assistant |
| Composer `@` | message | One-reply borrow, multi-model compare |
| KB / wiki / loop creation wizards | index | Embedding binding block (§6), re-embed migration entry point on the index detail sheet |
| Models tile | cross-cutting | Suggestion cards (§7), measured numbers, library (local-models §12) |

## 9. Implementation notes (this repo)

- Registry + resolver in Rust (`gen_ui_client` / a small `knowme_model_registry`
  module): `ModelRef(provider_id, model_id)`, capability resolution, the §4.3 resolver
  with placement walk — one implementation behind FFI/IPC/WASM so React, Flutter, and
  Hands runtime resolve identically; resolution results carry `{ref, source_level,
  placement, reason}` for provenance rendering.
- Settings storage: role slots + assistant pins as user-scoped CRDT settings docs;
  device-scoped placement cache; keys in keychain (`gen_ui_agent::secrets` already
  exists) / Flint Vault server-side.
- Usage counters: local SurrealDB tables written by the runtime (chat + Hands + UAR
  tool results already flow through the core); the suggestion rules run in
  `utility.fast`-priced background passes.
- Provider-kind schemas ship in the app + are extensible via `.pxt` `provider`
  capability (extensibility spec §5).

## 10. References

- Cherry Studio: [default model settings](https://docs.cherryai.com.cn/docs/en-us/pre-basic/settings/default-models.md) ·
  [chat interface (topics, @mention, auto-reset)](https://docs.cherryai.com.cn/docs/en-us/cherry-studio/preview/chat) ·
  [model configuration & capabilities internals](https://deepwiki.com/CherryHQ/cherry-studio/5.3-model-configuration-and-capabilities) ·
  [provider settings](https://docs.cherry-ai.com/docs/en-us/cherry-studio/preview/settings/providers) ·
  [custom providers](https://docs.cherry-ai.com/docs/en-us/pre-basic/providers/zi-ding-yi-fu-wu-shang) ·
  [knowledge base](https://docs.cherryai.com.cn/docs/en-us/cherry-studio/preview/knowledge-base.md)
- In-repo: `docs/strategies/local-models/README.md` (§10 picks, §11 embeddings, §12
  library UX) · `docs/knowme-builder-extensibility-spec.md` (§6 settings scopes/sync) ·
  `docs/knowme-functional-spec.html` (§04, §09, §10)
