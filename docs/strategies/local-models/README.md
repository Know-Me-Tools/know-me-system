# Local Model Acquisition & Management Strategy

**Scope:** how KnowMe selects, downloads, verifies, and activates on-device models across
web, desktop (macOS/Windows/Linux), and mobile (iOS/Android) — including first-run
onboarding, per-environment hardware detection, distribution (Hugging Face vs. a
KnowMe-controlled origin), and post-download validation.

**Status:** v1.1 strategy, 2026-07-18 (v1.1 adds §10 per-machine reference picks and §11 embeddings). Companion to `docs/knowme-functional-spec.html`
(§04 Models tile, §10 Tiers) and `docs/knowme-builder-extensibility-spec.md` (§7
packaging). Implements against `rust/crates/gen_ui_inference` (llama-cpp-2 +
reqwest/sha2, mistral.rs optional) and the `versions.toml` `[inference]` matrix
(desktop/mobile = llama-cpp-2, web = WebLLM).

---

## 1. Principles

1. **The catalog is signed data, not code** (§4). Models, quants, sizes, hashes,
   licenses, and hardware requirements ship as a versioned manifest the app fetches and
   verifies — never as constants in the binary.
2. **Recommend, never force.** Detection produces a *recommendation* with a visible
   reason ("8 GB unified memory → Gemma-3n-2B-Q4"); the library stays open (sideload,
   override). This is the "library, not a menu" stance from the moodboard.
3. **The first token is the onboarding goal.** Everything in §2 is ordered so the user
   reaches a working first generation as fast as their connection allows, with honest
   progress and a working app while they wait.
4. **Trust is structural**: SHA-256 on every artifact, signed manifests, and a
   load-and-generate smoke test before a model is marked Active (§7). llama.cpp does
   not verify integrity at load time — there is an open upstream request for exactly
   this ([ggml-org/llama.cpp#15250](https://github.com/ggml-org/llama.cpp/issues/15250))
   — so verification is **our** job, in the acquisition layer.
5. **One state machine, three transports.** Discovery → fit-check → consent → fetch →
   verify → activate is identical everywhere; only the fetch adapter differs per tier.

## 2. Onboarding (first run)

The Engine Card is the destination: green dot, model name, live tok/s. First run is the
path to lighting it up.

**Step 0 — instant utility.** The app must be usable before any model lands: UI opens,
settings work, and (web/desktop) a "connect to a BossFang instance or cloud provider"
escape hatch exists so a user on hotel Wi-Fi isn't staring at a progress bar. If a peer
instance is reachable, offer remote inference *now* while the local model downloads in
the background.

**Step 1 — probe (silent, <1 s).** Run the `DeviceProfile` probe (§3). No consent
needed; nothing leaves the device. Cache in settings; re-probe on hardware change
(eGPU, RAM upgrade, browser update enabling WebGPU).

**Step 2 — recommend (one card, one number).** Show a single recommended model with
size, why it was chosen, and what it costs: "Gemma-3n-2B-Q4 · 1.5 GB download ·
runs ~47 tok/s on this Mac." Secondary link: "choose a different model" opens the full
library with fits / tight / won't-fit badges. License display happens here (§5.4):
permissive licenses (Apache-2.0/MIT) get a link; restricted-license models get an
explicit acceptance step *before* download.

**Step 3 — consent to the download, with the right defaults per tier.**
- Desktop: start immediately on click; show resumable progress in the Engine Card slot.
- Mobile: disclose size prominently; default **Wi-Fi only** (user can override to
  cellular); warn on low storage or low battery; the download survives app suspension
  (§6.3).
- Web: check `navigator.storage.estimate()` against the artifact size first; request
  `navigator.storage.persist()`; if the browser lacks WebGPU or refuses persistence,
  say so plainly and offer the smaller/CPU-lane model or a remote instance instead.

**Step 4 — verify + activate (§7), then benchmark.** The first activation runs the
30-second micro-benchmark; its measured tok/s becomes the Engine Card number and is
stored in the device profile. If decode lands under the usability floor (~8–10 tok/s),
offer the next model down *with the measurement as the reason*.

**Step 5 — background upsell, never a blocker.** Larger models the hardware can carry
("this machine could run Gemma-4-3B — 2.1 GB") appear as a dismissible suggestion in
the Models tile, never as onboarding friction.

**Re-onboarding paths** reuse the same flow: new device via OFP (peer already has the
GGUF → LAN fetch, §5.3), restored backup (models are excluded from backups → re-offer),
storage eviction (web/mobile → detect at load, §7, re-offer).

## 3. Environment detection

One Rust implementation in `gen_ui_inference` (`DeviceProfile` prober + fit-checker),
reached over FFI/IPC from every UI; a TS mirror (or `gen_ui_wasm` export) for web.

### 3.1 What we probe, per platform

| Platform | Budget that matters | Probe |
|---|---|---|
| macOS (Apple Silicon) | **Unified memory** — one pool for weights + KV + app | `sysctl hw.memsize`; Metal `recommendedMaxWorkingSetSize` as the honest ceiling; always Metal backend |
| macOS (Intel, legacy) | System RAM, CPU SIMD | AVX2 check; CPU lane only; recommend smallest tier |
| Windows / Linux + dGPU | **VRAM**, not system RAM | wgpu/Vulkan adapter enumeration (name, VRAM, driver); NVML for NVIDIA detail; backend order CUDA → Vulkan → CPU |
| Windows / Linux iGPU/CPU | System RAM + SIMD | total RAM; AVX2/AVX-512 flags; core count → thread config; iGPU shared-memory counts as *system* RAM, don't double-count |
| iOS | **Jetsam limit**, not physical RAM | device model → RAM tier; budget ≈ 50–60 % of physical RAM (less on ≤4 GB devices); Metal always present |
| Android | Heterogeneous everything | `ActivityManager.memoryClass`/total RAM; Vulkan availability + version; thermal headroom API; be pessimistic by default |
| Web | WebGPU adapter limits + storage quota | `navigator.gpu.requestAdapter()` (presence + `maxBufferSize`, `maxStorageBufferBindingSize`); `navigator.deviceMemory` (coarse, capped at 8 — treat as a floor, not a truth); `storage.estimate()` for disk |

### 3.2 The fit test

```
fits :=  weights(quant)
       + kv_cache(ctx_len, n_layers, kv_quant)
       + compute_buffers
       + headroom            <  budget
```

- `budget` = VRAM on discrete GPUs; **60–70 % of unified/system RAM** elsewhere (the
  app, SurrealDB, and the OS live in the same pool — on iOS use the jetsam-derived
  figure).
- Choose quant by budget: Q4_K_M default; Q8_0 when abundant; when constrained, **drop a
  model size tier before dropping below Q4** — harsh quants degrade quality faster than
  smaller models do.
- Context length is part of the equation: the same model "fits" at 4k and doesn't at
  32k. The recommendation pins a default ctx per device tier and says so.

### 3.3 Heuristics propose, benchmarks decide

Static detection selects the candidate; the first-activation micro-benchmark
(short prefill + ~64-token decode, measured separately) is the source of truth for the
Engine Card and for downgrade/upgrade suggestions. Store `{model, quant, backend,
prefill_tps, decode_tps, measured_at}` in the device profile. Never show a user a
guessed tok/s when a measured one exists.

## 4. The catalog

A signed JSON manifest (same Ed25519/`did:key:` discipline as the plugin system,
FUNC-SPEC §08), fetched from the KnowMe origin with an ETag, cached for offline use.

Per model entry:

```jsonc
{
  "id": "gemma-3n-2b",
  "family": "gemma", "params_b": 2.0,
  "license": { "spdx": "Apache-2.0", "acceptance_required": false, "url": "…" },
  "capabilities": ["chat", "vision"],
  "artifacts": [
    {
      "kind": "gguf", "quant": "Q4_K_M", "bytes": 1520000000,
      "sha256": "…",
      "sources": [
        { "type": "knowme",  "url": "https://models.know-me.tools/gemma-3n-2b/q4_k_m.gguf" },
        { "type": "ipfs",    "cid": "bafy…" },
        { "type": "hf",      "repo": "google/gemma-3n-2b-gguf", "file": "…", "gated": false }
      ]
    },
    { "kind": "webllm", "quant": "q4f16_1", "shards": [ { "url": "…", "bytes": 0, "sha256": "…" } ],
      "model_lib": "https://models.know-me.tools/webllm/gemma-3n-2b-q4f16_1-webgpu.wasm" }
  ],
  "requirements": {
    "min_budget_bytes": { "gguf-Q4_K_M": 2400000000 },
    "backends": ["metal", "cuda", "vulkan", "cpu"],
    "default_ctx": 8192
  }
}
```

Notes: WebLLM entries carry the **shard list + model_lib** (WebLLM's `appConfig` takes
custom `model` URLs and caches artifacts via the Cache API, or IndexedDB via
`appConfig.cacheBackend` — see the [WebLLM advanced-usage docs](https://webllm.mlc.ai/docs/user/advanced_usage.html)
and [config.ts](https://github.com/mlc-ai/web-llm/blob/main/src/config.ts)); GGUF
entries list multiple sources in preference order (§5.3).

## 5. Distribution: Hugging Face vs. a KnowMe-controlled origin

### 5.1 The decision

**Primary: a KnowMe-controlled origin. Hugging Face as fallback. HF tokens only for
genuinely gated models the user brings their own account for.**

The licensing landscape makes this clean for the entire curated catalog: Qwen and
Mistral ship Apache-2.0, Phi ships MIT, and Gemma 4 moved to Apache-2.0 in 2026
([VentureBeat](https://venturebeat.com/technology/google-releases-gemma-4-under-apache-2-0-and-that-license-change-may-matter),
[MindStudio](https://www.mindstudio.ai/blog/gemma-4-apache-2-license-commercial-use));
earlier Gemma generations use the [Gemma Terms of Use](https://ai.google.dev/gemma/terms),
which permit redistribution with pass-through of the use restrictions — which is
exactly how Ollama serves Gemma from its own registry today
([ollama.com license blob](https://ollama.com/library/gemma4:26b-a4b-it-q4_K_M/blobs/7339fa418c9a)).
So we can legally mirror everything in the default catalog and control our own
availability, latency, and versioning.

Why not HF as primary: HF's gated-model access is granted **to individual users, never
to organizations** — a shared org token cannot download gated repos on users' behalf
([HF gated-models docs](https://huggingface.co/docs/hub/en/models-gated)) — and
resolve-URL semantics, rate limits, and repo layout changes are all outside our
control. HF remains a *listed fallback source* for permissive models (anonymous
`resolve/main/<file>` URLs work ungated —
[HF downloading docs](https://huggingface.co/docs/hub/models-downloading)) and the
*only* path for gated models, where the flow is: user pastes their own HF token
(stored in OS keychain via Tauri secure storage; per-user, per HF policy —
[HF token docs](https://huggingface.co/docs/hub/security-tokens)).

### 5.2 The KnowMe origin, concretely

- **Storage/CDN:** Cloudflare R2 behind a custom domain (`models.know-me.tools`) —
  zero egress fees make multi-GB artifacts economically sane
  ([R2 overview](https://developers.cloudflare.com/r2/)), and a custom-domain public
  bucket with CDN caching is the recommended pattern for large public artifacts
  ([R2 access patterns](https://stackharbor.com/en/knowledge-base/cf-r2-buckets-access-patterns/),
  [community guidance](https://community.cloudflare.com/t/general-advice-concerning-public-r2-buckets-vs-pre-signed-urls/718993)).
- **Auth model:** *no authentication for free-catalog artifacts.* Integrity comes from
  the signed manifest + SHA-256, not from access control; anonymous download is a
  feature (no account required — FUNC-SPEC §09 "no central account"). Range requests
  work natively, which resumable downloads (§6) depend on.
- **Pro-tier licensed models** (the Qwen-35B-class artifacts) are the one place owned
  auth earns its keep: a license key (issued at purchase, stored in keychain) is
  exchanged at a tiny token service for a **short-lived presigned URL**
  ([R2 presigned URLs](https://developers.cloudflare.com/r2/api/s3/presigned-urls/))
  or passed to a Cloudflare Worker gate in front of the private bucket
  ([Worker-gated R2 pattern](https://blog.dankying.com/en/posts/20250429-how-to-build-an-image-service-using-cloudflare-workers/)).
  Keys authorize *download*, not *inference* — once fetched and verified, the model
  runs offline forever, preserving the sovereignty guarantee.
- **Layout: content-addressed, Ollama-style.** Store blobs under
  `/blobs/sha256/<digest>` with catalog manifests pointing at digests — the OCI-like
  blob/manifest split Ollama uses is the proven shape for model distribution
  ([Ollama storage internals](https://medium.com/@dewasheesh.rana/inside-ollamas-model-storage-understanding-blobs-and-manifests-06f1620dd0b2),
  [DeepWiki: registry & layers](https://deepwiki.com/ollama/ollama/4.2-model-registry-and-layers)).
  This makes the same blobs trivially publishable to IPFS (same digests → CIDs) and to
  an OCI registry later, aligning with the `.pxt` distribution story in the
  extensibility spec.

### 5.3 Source preference order (per artifact, walked by the fetcher)

1. **LAN/OFP peer** that already holds the digest (BossFang instance) — free, fast,
   sovereign;
2. **KnowMe origin** (R2/CDN);
3. **IPFS gateway** (own gateway first, public fallback) — same digest;
4. **Hugging Face** resolve URL (anonymous, or user token if gated).

Every source yields bytes that must hash to the same SHA-256; a source that fails
verification is skipped and reported, never trusted.

### 5.4 License UX

`acceptance_required: false` (Apache/MIT): show the license link, no click-through.
`acceptance_required: true` (Gemma-Terms-class, Pro licensed models): record
`{model, license_version, timestamp}` locally on acceptance, gate the download on it,
and pass the terms through in the UI — that pass-through is what the Gemma terms
require of redistributors.

## 6. Download management per tier

### 6.1 Shared semantics (all tiers)

Persistent `DownloadTask` records in SurrealDB: `{model, artifact, source, bytes_total,
bytes_done, state}` with states `queued → fetching → verifying → smoke-testing → ready
| failed(reason)`. Preflight checks disk space (artifact + 10 % slack). Fetch to
`<digest>.part`, atomic rename into the content-addressed store on verification.
Progress streams to the UI over Tauri channels / FFI streams / JS events. Pause,
resume, and cancel are user-visible verbs.

### 6.2 Desktop (Tauri) — the reference implementation

Rust in `gen_ui_inference` (reqwest is already wired): HTTP Range resumption, 3–4
parallel range segments for large files, streaming SHA-256 (sha2 already a dep),
bandwidth cap setting, retry with exponential backoff + resume from `bytes_done`.
Store under `<app-data>/models/blobs/sha256/<digest>`; a `models/registry.json` maps
human names → digests (mirroring §5.2's layout locally, which is what makes peer-serving
trivial for BossFang).

### 6.3 Mobile — use the OS transfer service, not reqwest

A foreground Rust download dies on suspension. Use
[`URLSession` background sessions](https://developer.apple.com/documentation/Foundation/downloading-files-in-the-background)
on iOS (survives suspension and app termination; OS-managed retry; delegate wakes the
app on completion — mind the gotchas: background sessions are discretionary under Low
Power Mode/poor connectivity, per [Apple's forums guidance](https://developer.apple.com/forums/thread/699462)
and [field notes](https://www.appxiom.com/blogs/urlsession-background-download-not-working-ios/))
and `WorkManager`-scheduled `DownloadManager` on Android (constraint-aware: unmetered
network, storage, battery). The FRB bridge only observes: platform code reports
progress/completion; Rust verifies the hash and finalizes. Defaults: Wi-Fi-only
(overridable), no auto-start on cellular, `isExcludedFromBackup = true` on iOS (models
must not inflate iCloud backups), never bundle weights in the app binary.

### 6.4 Web — delegate to WebLLM, manage the edges

WebLLM fetches shards and caches via the Cache API (or IndexedDB via
`appConfig.cacheBackend`) ([docs](https://webllm.mlc.ai/docs/user/advanced_usage.html)).
KnowMe's `appConfig` points `model` URLs at the KnowMe origin (§5.2) rather than HF.
Around it: `storage.estimate()` preflight; `storage.persist()` request with honest
messaging when denied; eviction detection at engine init (cache miss → re-offer);
WebGPU absence → offer the wllama/WASM-CPU lane for small models or a remote instance
(the dual-lane pattern in
[webgpu-webllm-app](https://github.com/krtarunsingh/webgpu-webllm-app)). Safari today
means "small models or remote" — say it, don't fake it.

## 7. Post-download validation

Marking a model **Ready** requires all five gates, in order:

1. **Hash gate** — streaming SHA-256 equals the manifest digest (web: per-shard).
2. **Format gate** — parse the GGUF header/metadata (magic, version, tensor count,
   arch) *without* full load; catches truncation and wrong-file errors cheaply. GGUF
   corruption and version drift are the top field failure classes
   ([corruption guide](https://ritridata.com/ai-tool-recovery/gguf-corruption-llama-cpp-compatibility-model-versioning),
   [load-failure taxonomy](https://markaicode.com/errors/llamacpp-model-load-failed-fix/)).
3. **Load gate** — actually instantiate on the selected backend with the profile's
   ctx/offload settings; catches OOM-at-load, backend/driver issues, unsupported
   quant-arch combos. On failure, retry once on the CPU lane to distinguish "bad file"
   from "bad backend" and report accordingly.
4. **Smoke gate** — one short deterministic generation (fixed prompt, temp 0, ~16
   tokens): asserts non-empty, valid UTF-8, no NaN/garbage repetition. Not a quality
   eval — a "the tensors are sane" check. (This is the layer llama.cpp itself doesn't
   provide — [#15250](https://github.com/ggml-org/llama.cpp/issues/15250).)
5. **Benchmark gate (first activation)** — the §3.3 micro-benchmark; records measured
   tok/s; triggers the downgrade suggestion below the usability floor.

Re-validation: gates 1–2 re-run whenever the file's mtime/size changed or after storage
eviction events; a `Ready` model that fails gate 3 at any later launch is demoted to
`needs-repair` with a one-click re-download (blobs are content-addressed, so repair
means "fetch the digest again", not "figure out what broke").

## 8. Failure modes and their answers

| Failure | Answer |
|---|---|
| Interrupted download | Range-resume from `bytes_done` (desktop/web); OS-managed resume (mobile) |
| Hash mismatch | Delete `.part`, try next source in §5.3 order, surface which source failed |
| Disk full mid-download | Preflight prevents most; on hit, pause + prompt, never corrupt the store |
| OOM at load | CPU-lane retry to classify; recommend smaller quant/model with the measurement shown |
| Browser evicted cache | Detect at init, re-offer download, remind about `persist()` |
| iOS purged model / restored backup | Backup-excluded by design; gate 2 catches it; one-click re-fetch |
| Catalog origin down | Cached signed catalog still valid; IPFS + HF sources still listed per artifact |
| Gated model, no token | Clear UX: link to HF request-access page, paste-token field, keychain storage |

## 9. Implementation map (this repo)

- `rust/crates/gen_ui_inference`: `DeviceProfile` prober + fit-checker; `ModelAcquisition`
  trait with `DesktopFetcher` (Range/parallel/SHA-256), validation gates 2–5 (gate 4
  behind `InferenceProvider`).
- New thin platform adapters: iOS background-session observer (Swift, via FRB),
  Android WorkManager observer (Kotlin), WebLLM `appConfig` builder + storage guards
  (TS, in `packages/gen-ui-react` or the web shell).
- `desktop/src/features/models` + `mobile/lib/features/models`: onboarding card (§2),
  fits/tight/won't-fit badges from the fit-checker, measured-tok/s display.
- Catalog signing + serving: R2 bucket + Worker gate for Pro artifacts; blob layout per
  §5.2; publish job that pins the same digests to IPFS.
- `knowme_ofp`: "peer-has-digest" query + LAN fetch (§5.3 source 1) — additive to the
  M2 sync milestone.

## 10. Reference picks by machine (July 2026)

Concrete catalog seeds for the situations we care about. Sizes are Q4_K_M GGUF unless
noted; tok/s are decode-phase, *indicative* — the §7 benchmark gate always decides
(figures from [llmcheck.net Apple-Silicon benchmarks](https://llmcheck.net/benchmarks),
[modelfit iPhone guide](https://modelfit.io/guides/best-llm-for-iphone/),
[klymentiev hardware guide](https://klymentiev.com/blog/best-local-llm), and the
[PromptQuorum tool-calling benchmark](https://www.promptquorum.com/power-local-llm/best-local-models-tool-calling-2026)).
RAM figures below are GB. Two rules of thumb frame every row:

- **Tool calling has a size floor.** Multi-step agentic tool use gets reliable at
  ~14B+ dense or ~30B-A3B MoE (≥93 % well-formed calls); 7–8B is fine for single-shot
  structured calls; below 7B, models emit malformed calls on multi-step chains — a 95 %
  per-call rate compounds to ~66 % over 8 steps. Sub-4B models are for
  triage/classification and drafting, with **grammar-constrained JSON**
  (llama.cpp [function-calling support](https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md)
  handles Qwen/Hermes-style templates and can force valid JSON) as the mitigation.
  Consequence for KnowMe: on small-tier devices, Hands should run *fewer, simpler*
  tool steps locally and route complex chains to a BossFang peer.
- **MoE is the CPU/unified-memory cheat code.** Qwen3-30B-A3B (30B total / 3B active)
  gives ~14B-class quality at small-model decode cost wherever *memory* is abundant and
  *compute* is not (Intel CPU boxes, 32 GB+ Macs).

| Machine | Daily driver (why) | Agent/Hands tier | Vision | Indicative speed |
|---|---|---|---|---|
| **MacBook Pro M1 2021, 16 GB** | Qwen3 8B (~5.0 GB) — best quality that leaves headroom for app+DB; Qwen3 4B (~2.5 GB) when multitasking | Single-shot tools OK (8B); multi-step → peer | Gemma 3 4B-IT (~3 GB, 128k ctx, vision) | 4B ≈ 50 t/s · 8B ≈ 35–40 t/s |
| **M1 Pro/Max 2021, 32 GB** | Qwen3 14B (~9 GB) — first tier where local multi-step tool calling is respectable | Qwen3-30B-A3B (~18 GB) fits, memory-bandwidth-friendly (3B active) | Gemma 3 12B (~7.6 GB) | 14B ≈ 25–30 t/s · 30B-A3B ≈ 30–40 t/s (Max) |
| **M1 Max 2021, 64 GB** | Qwen3-30B-A3B comfortable + room for KV/ctx | Gemma 4 27B (~16 GB, ~95 % tool-call rate) or Qwen3 32B (~93 %); Llama 3.3 70B Q4 (~40 GB) possible but ~7–9 t/s — quality ceiling, not a daily driver | Gemma 3 27B | 27–32B dense ≈ 12–18 t/s |
| **Mac mini M4, 16 GB** | Qwen3 4B — M4 makes small models feel instant; Qwen3 8B (~5 GB) for quality | 8B single-shot; multi-step → peer | Gemma 3 4B | 4B ≈ 80–118 t/s · 8B ≈ 60–80 t/s |
| **Mac mini M4, 32 GB** | Qwen3 14B (~9 GB) | **Qwen3-30B-A3B (~18 GB) — the recommended always-on BossFang/Hands node config** | Gemma 3 12B | 14B ≈ 35–40 t/s · 30B-A3B ≈ 40+ t/s |
| **Mac Intel desktop, 96 GB, no GPU** | **Qwen3-30B-A3B** — the MoE showcase: 96 GB swallows the 18 GB file; only 3B active params per token keeps CPU decode usable (~10–15 t/s AVX2); Phi-4-mini 3.8B (~2.3 GB) as the snappy lane (~15–20 t/s) | 30B-A3B (good calls, slow chains — cap steps) | Skip local vision; route to peer | Dense 8B ≈ 8–12 t/s — prefer the MoE |
| **Standard Windows laptop, no GPU (8–16 GB)** | Qwen3 4B or Phi-4-mini 3.8B (~2.3–2.5 GB), ctx capped 4–8k; Gemma 3n E2B where its per-layer-embedding footprint helps | Triage/single-tool only, grammar-constrained; chains → peer/cloud | Gemma 3 4B if 16 GB | ≈ 8–15 t/s CPU (AVX2) |
| **Linux, 32 GB RAM, RTX 4070 Ti 12 GB** | Qwen3 14B (~9 GB) fully offloaded — the 12 GB-VRAM sweet spot | Qwen3-Coder-30B-A3B with MoE expert-offload to system RAM (llama.cpp `--n-cpu-moe`): ~96 % on code-shaped tool work at ~20–30 t/s | Gemma 3 12B (~7.6 GB, fits VRAM) | 14B ≈ 45–60 t/s CUDA · 12B ≈ 40–50 t/s |
| **Chrome (WebLLM lane), any of the above** | Qwen3 1.7B q4f16 (~1.1 GB) as the universal default; Qwen3 4B (~2.5 GB) on machines with a real GPU and granted persistent storage | Structured single calls only (grammar via WebLLM JSON mode); Hands don't run in the browser tier | — | dGPU ≈ 25–40 t/s · iGPU ≈ 8–20 t/s |
| **iPhone 17 (8 GB — the 128/256/512 GB storage tiers don't change RAM, only how many models fit on disk)** | Gemma 4 E2B (~25 t/s) or Qwen3.5 2B — both under ~4 GB resident, inside the jetsam budget (§3.1); 4B-class Q4 works but at 8–15 t/s and thermal cost | Single-tool max; Hands execute on peers, phone renders results | Gemma 4 E2B (multimodal) | 2B ≈ 20–25 t/s · 4B ≈ 8–15 t/s |
| **iPhone 17 Pro (12 GB)** | Gemma 4 E4B (~30 t/s measured); Qwen3.5 4B for text-heavy | Single-shot tools locally; short chains viable but battery/thermal-gated | Gemma 4 E4B | ≈ 30 t/s |
| **Galaxy S24 (8 GB; Ultra 12 GB)** | Qwen3 1.7B / Qwen3.5 2B Q4; Gemma 3n E2B (its PLE design targets exactly this class) | Triage/single-tool, grammar-constrained | Gemma 3n E2B | ≈ 10–18 t/s (NEON; Vulkan on Adreno where stable) |
| **Galaxy S26 (12 GB+)** | Gemma 4 E4B or Qwen3.5 4B | Single-shot tools locally | Gemma 4 E4B | ≈ 15–25 t/s |
| **Standard Android (6–8 GB midrange)** | Qwen3 0.6B–1.7B Q4 (0.4–1.1 GB) — summarize/draft/triage lane; be honest that this tier is an *assistant*, not an agent | None locally — always peer/cloud | — | ≈ 5–12 t/s CPU |

Catalog implications: the matrix reduces to a small artifact set — Qwen3
{0.6B, 1.7B, 4B, 8B, 14B, 30B-A3B, Coder-30B-A3B}, Qwen3.5 {2B, 4B}, Gemma 3
{4B, 12B, 27B} + Gemma 3n/4 {E2B, E4B}, Phi-4-mini, and (Pro tier) Gemma 4 27B /
Qwen3 32B-class — all Apache-2.0/Gemma-terms redistributable through the §5 origin.
Every row is a *default*, surfaced with its reason string in the §2 recommendation card.

## 11. Embedding models for RAG (all tiers)

The memory/RAG layer (SurrealDB graph + sqlite-vec on mobile, PGlite/pgvector on
web/desktop, Postgres+pgvector on flint-forge) needs embeddings everywhere the LLM
runs — and unlike chat models, **the embedding model must be consistent per index**:
vectors from different models (or different truncation dims) don't compare. The catalog
therefore stamps `{embedding_model_id, dim}` into every index/KB it creates, and OFP
sync treats a mismatch as a re-embed job, not a merge.

| Model | Size | Dims | Why / where |
|---|---|---|---|
| **EmbeddingGemma-300M** — the default everywhere | ~300 MB (Q8; <200 MB quantized) | 768, Matryoshka-truncatable to 512/256/128 | 100+ languages, 2k ctx, built for on-device; runs on every tier including midrange Android and the browser (ONNX/transformers.js). Truncate to 256 on mobile (sqlite-vec size/speed), keep 768 on desktop/server — same model, so re-embedding on upgrade is optional, but *dims must match within one index* |
| **Qwen3-Embedding-0.6B** — quality upgrade | ~600 MB | flexible 32–1024 (MRL) | Instruction-aware (task prefixes buy 1–5 %), 100+ languages; the desktop/BossFang default when quality beats footprint |
| **BGE-M3** — server-side heavy hitter | ~1.2 GB (fp16) | dense + sparse + multi-vector | 8k ctx, hybrid retrieval; runs on flint-forge/BossFang (optionally in-DB via Flint Ember), not on devices |
| **Nomic Embed Text v2 (MoE)** | 475M total / 305M active | 768 → 256 | Alternative default if MoE efficiency matters; ~100 languages |
| Reranker: **Qwen3-Reranker-0.6B** (or bge-reranker-v2) | ~600 MB | — | Optional second stage on desktop/server only; skip on mobile |

Tiering that falls out: **phones and web** embed with EmbeddingGemma@256 locally (queries
always embeddable offline); **desktop** embeds with Qwen3-Embedding-0.6B@768 or
EmbeddingGemma@768; **BossFang/flint-forge** can maintain a BGE-M3 hybrid index for the
heavy corpus and serve it to devices via the read path — devices keep a local
EmbeddingGemma index over their own hot subset. Embedding artifacts flow through the
same §5 origin, §6 downloader, and §7 gates (smoke test: embed a fixed sentence, assert
dim + finite norm + cosine-similarity sanity against a stored reference vector).
Sources: [BentoML open-source embedding guide](https://www.bentoml.com/blog/a-guide-to-open-source-embedding-models),
[Milvus 2026 comparison](https://milvus.io/blog/choose-embedding-model-rag-2026.md),
[Ollama embedding benchmark](https://www.morphllm.com/ollama-embedding-models).

## 12. References

- Hugging Face: [gated models](https://huggingface.co/docs/hub/en/models-gated) ·
  [downloading](https://huggingface.co/docs/hub/models-downloading) ·
  [access tokens](https://huggingface.co/docs/hub/security-tokens)
- WebLLM/MLC: [advanced usage & caching](https://webllm.mlc.ai/docs/user/advanced_usage.html) ·
  [JS SDK](https://llm.mlc.ai/docs/deploy/webllm.html) ·
  [config.ts (AppConfig/ModelRecord)](https://github.com/mlc-ai/web-llm/blob/main/src/config.ts) ·
  [WebGPU + WASM fallback pattern](https://github.com/krtarunsingh/webgpu-webllm-app)
- Apple: [Downloading files in the background](https://developer.apple.com/documentation/Foundation/downloading-files-in-the-background) ·
  [background download caveats (forums)](https://developer.apple.com/forums/thread/699462) ·
  [field guide to background URLSession](https://www.appxiom.com/blogs/urlsession-background-download-not-working-ios/)
- Ollama distribution internals: [blobs & manifests](https://medium.com/@dewasheesh.rana/inside-ollamas-model-storage-understanding-blobs-and-manifests-06f1620dd0b2) ·
  [registry & layers](https://deepwiki.com/ollama/ollama/4.2-model-registry-and-layers) ·
  [storage & blob transfer](https://deepwiki.com/ollama/ollama/2.4-storage-and-blob-transfer)
- Cloudflare R2: [overview (zero egress)](https://developers.cloudflare.com/r2/) ·
  [presigned URLs](https://developers.cloudflare.com/r2/api/s3/presigned-urls/) ·
  [public vs private access patterns](https://stackharbor.com/en/knowledge-base/cf-r2-buckets-access-patterns/) ·
  [community: public bucket vs presigned](https://community.cloudflare.com/t/general-advice-concerning-public-r2-buckets-vs-pre-signed-urls/718993) ·
  [Worker-gated R2](https://blog.dankying.com/en/posts/20250429-how-to-build-an-image-service-using-cloudflare-workers/)
- Licensing: [Gemma Terms of Use](https://ai.google.dev/gemma/terms) ·
  [Gemma 4 → Apache 2.0 (VentureBeat)](https://venturebeat.com/technology/google-releases-gemma-4-under-apache-2-0-and-that-license-change-may-matter) ·
  [license analysis (MindStudio)](https://www.mindstudio.ai/blog/gemma-4-apache-2-license-commercial-use) ·
  [Ollama-served Gemma license blob](https://ollama.com/library/gemma4:26b-a4b-it-q4_K_M/blobs/7339fa418c9a)
- Integrity/validation: [llama.cpp #15250 (no built-in verification)](https://github.com/ggml-org/llama.cpp/issues/15250) ·
  [GGUF corruption & compatibility](https://ritridata.com/ai-tool-recovery/gguf-corruption-llama-cpp-compatibility-model-versioning) ·
  [load-failure taxonomy](https://markaicode.com/errors/llamacpp-model-load-failed-fix/)
- Model selection & performance (§10): [Apple-Silicon tok/s benchmarks](https://llmcheck.net/benchmarks) ·
  [hardware-tier model guide](https://klymentiev.com/blog/best-local-llm) ·
  [iPhone on-device LLM guide](https://modelfit.io/guides/best-llm-for-iphone/) ·
  [tool-calling benchmark 2026](https://www.promptquorum.com/power-local-llm/best-local-models-tool-calling-2026) ·
  [llama.cpp function-calling docs](https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md) ·
  [Unsloth tool-calling guide](https://unsloth.ai/docs/basics/tool-calling-guide-for-local-llms) ·
  [SLM rankings 2026](https://localaimaster.com/blog/small-language-models-guide-2026)
- Embeddings (§11): [BentoML open-source embedding models](https://www.bentoml.com/blog/a-guide-to-open-source-embedding-models) ·
  [Milvus embedding comparison 2026](https://milvus.io/blog/choose-embedding-model-rag-2026.md) ·
  [Ollama embedding benchmark](https://www.morphllm.com/ollama-embedding-models)
