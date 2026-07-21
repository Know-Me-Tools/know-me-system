# Mobile inference diagnosis: iOS broken, Android UAR breaks after lane switch-back

**Date:** 2026-07-21
**Scope:** diagnosis only — no code changed.
**Reported symptoms:**
1. iOS: neither the UAR lane nor the local-model lane produces inference at all.
2. Android: the local model works. Switching a conversation from local back to UAR causes UAR to stop working (for the remainder of that conversation).

Both platforms share one architectural fact that shapes everything below: since
[`ac1122d`](../../rust/crates/gen_ui_ffi/src/api/boot.rs) / [`55f56ba`](../../rust/crates/gen_ui_agent/src/uar.rs),
mobile boot builds a single `Arc<dyn InferenceProvider>` engine instance and hands
the **same instance** to both the "local" lane (`state.inference`) and the embedded
UAR kernel's driver bridge (`UarEmbeddedRuntime::build_local`). There are not two
engines — there is one engine, wrapped twice.

```
rust/crates/gen_ui_ffi/src/api/boot.rs:275-320
    let inference: Arc<dyn InferenceProvider> = LiteRtLmEngine::new(...)   // Android
                                               | LlamaCppEngine::new(...)  // iOS
    let embedded_uar = UarEmbeddedRuntime::build_local(inference.clone(), spec, provider_config)
    gen_ui_agent::state::init_with_inference_and_uar(config, store, Some(inference), embedded_uar)
```

---

## Part 1 — iOS: both lanes are broken

### Root cause: the iOS default model is oversized for the process's memory budget, and there is no entitlement or preflight to prevent iOS from killing the app on load

- iOS's pinned default is `IPHONE_QWEN35_9B`: a **5.68 GB** GGUF plus a **918 MB**
  vision projector
  ([`gen_ui_inference/src/catalog.rs:39-57`](../../rust/crates/gen_ui_inference/src/catalog.rs)).
- `mobile/ios` has **no `.entitlements` file at all** — grep for
  `CODE_SIGN_ENTITLEMENTS` in `mobile/ios/Runner.xcodeproj/project.pbxproj`
  returns nothing. In particular `com.apple.developer.kernel.increased-memory-limit`
  is absent, so the process is capped at the standard (much lower) iOS jetsam
  budget for a ~5.7 GB working set plus KV cache.
- `LlamaCppEngine::load` attempts full GPU (Metal) offload first —
  `LlamaModelParams::default().with_n_gpu_layers(u32::MAX)`
  ([`gen_ui_inference/src/llama.rs:322`](../../rust/crates/gen_ui_inference/src/llama.rs)) —
  which pulls the whole model into resident memory.
- `preflight_artifact` only checks the artifact's **declared file size** against a
  static 8 GB cap
  ([`llama.rs:273-285`](../../rust/crates/gen_ui_inference/src/llama.rs)); there is
  no check against the device's actual available RAM or the jetsam budget.
- The code does have a fallback path (9B → `ANDROID_QWEN3_4B`,
  [`llama.rs:499-525`](../../rust/crates/gen_ui_inference/src/llama.rs)), but it
  only triggers on `Err(primary_error)` returned from
  `LlamaModel::load_from_file`. **A jetsam OOM kill is a process termination, not
  a catchable Rust `Err`** — the process is gone before the fallback branch can
  run. This is why inference looks like a hard, silent failure rather than a
  graceful degrade to the smaller model.
- Because `boot.rs` shares one `LlamaCppEngine` instance between the local lane
  and the UAR embedded driver bridge (see the shared-engine note above), **the
  jetsam kill on model load takes down both lanes identically** — there's no
  independent UAR-only or local-only failure mode on iOS; there's one process
  death that both lanes route through.
- This matches the project's own outstanding-risk notes: `.kbd-orchestrator/phases/on-device-qwen35/plan.md:67-68`
  explicitly calls for "an app-memory preflight" and "automatic 4B fallback...
  rather than allowing iOS jetsam termination" as **not yet implemented**, and
  `docs/strategies/local-models/README.md:92,373` documents the iOS jetsam
  budget math and recommends a model in the ~2–4 GB class, which the current
  9B default violates.

### Contributing factor: iOS local/UAR inference has never been certified on a physical device

- Recent commit history (`ac1122d`, `55f56ba`, `12622cd`, `e8d9c48`) only
  **registers and routes** iOS through the embedded UAR kernel — it never adds a
  device-certification commit. By contrast, Android has a long run of
  certification commits (e.g. "Certify Android branch actions", "Certify Android
  tool fallback path", "Harden Android clean-session certification").
- `.kbd-orchestrator/phases/model-runtime-per-device/execution.md:64,1213` and
  `.kbd-orchestrator/phases/on-device-qwen35/assessment.md:10,39` both mark iOS
  Metal inference as a deferred/`[STUB]` item awaiting a separate iOS
  certification pass. In other words: "iOS doesn't run yet" is the documented,
  expected state of this in-flight change, not a regression from working code.

### Secondary, lower-confidence risk: Metal/MetalKit linking

- The iOS FFI target enables `local-llama-metal` + `local-llama-mtmd`
  (`gen_ui_ffi/Cargo.toml:56-58`). The iOS podspec
  (`mobile/rust_builder/ios/gen_ui_ffi.podspec:54`) force-loads the static
  library and links `CoreAudio`, `AudioToolbox`, `Accelerate`, `AVFoundation`,
  but does not explicitly list `Metal`/`MetalKit`/`Foundation`. In a
  force-loaded static library, Cargo's own `rustc-link-lib` directives don't
  reach Xcode's linker (the podspec's existing comment about `-lc++` describes
  the same class of problem). A `libgen_ui_ffi.a` and `.framework` were present
  in the last build output, so this is not confirmed as the active failure —
  flagged as a latent risk to check if a clean build is attempted.

### Ranked fix order for iOS
1. Add `mobile/ios/Runner.entitlements` with
   `com.apple.developer.kernel.increased-memory-limit`, wire
   `CODE_SIGN_ENTITLEMENTS` into the Xcode project, **and/or** change the iOS
   default model to a jetsam-safe size (Qwen3.5 2B / Gemma-class, ~2–4 GB) per
   `docs/strategies/local-models/README.md:373`.
2. Implement a real device-RAM/jetsam preflight before `load()` (not just the
   static 8 GB file-size cap), so the 4B fallback is chosen proactively instead
   of relying on an `Err` path a jetsam kill never produces.
3. Run an actual iOS device certification pass — this is already called out as
   outstanding in the phase plan, not assumed-working.
4. Verify the Metal/MetalKit link on a clean iOS build; add the missing
   frameworks to the podspec `OTHER_LDFLAGS` if undefined-symbol errors appear.

---

## Part 2 — Android: UAR stops working after switching back from local model, mid-conversation

This is a distinct bug from the iOS one, and it is **not** a jetsam/memory issue —
Android's Gemma 4 E2B model is ~2.6 GB and has a working certification history.
The cause lives in three separate, compounding defects across the UAR embedded
kernel and the KnowMe protocol bridge:

### 2.1 The seeded orchestrator agent's provider route is empty, and gracefully falls back to whichever model string the *global* LLM config was built with

`EmbeddedRuntimeBuilder::build()` does two independent things with the model
identity:
- It registers the platform's `UarProviderConfig` (e.g.
  `knowme-local-litert-lm/gemma-4-e2b-it-litertlm`) in the `ProviderRegistry` and
  sets it as the registry default.
- It separately constructs a **global `LlmConfig`** string
  `"{provider.id}/{default_model}"` and hands that to both the `Orchestrator`
  and `RunManager` (`universal-agent-runtime/src/embedded.rs:271-274`).

The seeded `orchestrator-agent` (created by `seed_defaults(true)`) has an
**empty** provider policy —
`policy.provider.default.provider == ""`, `.model == ""`
(`universal-agent-runtime/src/uar/defaults.rs:36-39`) — by design, so it defers
to "whatever the registry/global config currently is" rather than pinning a
model that could go stale. `resolve_run_policy` turns this into
`effective_policy.model == None`
(`universal-agent-runtime/src/uar/domain/policy.rs:240-246`), and
`start_run_with_policy`'s per-run resolution falls back to the process-global
`self.llm_config` when no route resolves
(`universal-agent-runtime/src/uar/runtime/manager.rs:1299-1324`).

Critically: **the run always executes against the same registered
`primary_driver` object** regardless of which provider/model string won the
resolution (`manager.rs:1371-1378`) — the model string only affects
prompt-dialect parameters and telemetry labels, never which driver instance is
invoked. So a route mismatch by itself is not what breaks UAR after a lane
switch; it's a red herring for this specific bug, but it does mean the "model"
identity UAR believes it's talking to can silently drift from what's actually
loaded in the shared engine (see 2.2), which matters once you're debugging with
logs that report the wrong model string.

### 2.2 The local lane and the UAR lane share one `LiteRtLmEngine`/JNI bridge instance, and the JNI bridge holds a single mutable native engine handle

This is the actual mechanism of the reported bug. Walk the sequence for
"send on UAR → switch to local → send on local → switch back to UAR → send
again":

1. **First UAR turn.** `send_uar` (`gen_ui_agent/src/chat.rs:1297-1408`) resolves
   the embedded runtime and calls `UarEmbeddedRuntime::start_run`, which
   ultimately drives the shared engine through
   `UarLocalInferenceDriverBridge::stream` →
   `provider.load(&model_spec)` → `provider.generate_request(&request)`
   (`gen_ui_agent/src/uar.rs:242-258`). On Android this reaches
   `LiteRtLmEngine::generate_request`, which — because `self.native` is already
   populated — calls `NativeLiteRtLmAdapter.generate_text`, i.e.
   `KnowMeLiteRtLmBridge.sendMessage(requestJson)`
   (`gen_ui_inference/src/litert_lm.rs:663-699`,
   `gen_ui_inference/src/android_litert_jni.rs:73-89`).
2. `sendMessage` on the Kotlin side **creates a brand-new `Conversation`** from
   the full message array in the request JSON every single call
   (`KnowMeLiteRtLmBridge.kt:67-81`) and overwrites the single `conversation`
   field (`kt:73`). This part is actually fine in isolation — each call is
   self-contained and stateless from Rust's perspective, because Rust always
   sends full history.
3. **Switch to local lane and send.** `send_local`
   (`gen_ui_agent/src/chat.rs:1722-2015`) calls `engine.load(&spec)` on the
   *same* `LiteRtLmEngine` instance. `load()` short-circuits
   (`litert_lm.rs:499-513`) if the spec matches what's already loaded — it does
   **not** call the JNI `load` again, it just republishes a "Ready" diagnostics
   event. Generation proceeds through the same `native.generate_text` path,
   again creating a fresh `Conversation` per call. Still fine.
4. **Switch back to UAR and send again.** Nothing in the Rust local↔UAR lane
   switch (`set_active_lane`, `chat.rs:1190-1211`) touches the shared engine at
   all — it only flips a config setting. The next UAR turn calls
   `UarLocalInferenceDriverBridge::stream` again, which again calls
   `provider.load(&self.model_spec)` and `provider.generate_request(...)`.

So far, nothing in the direct Rust↔JNI call chain explains a hard break — each
call is stateless per-message at the JNI boundary. The actual defect is one
layer up, in how UAR's own session/history and the protocol bridge interact
with this shared, stateless-per-call engine:

- **Assistant `tool_calls` are silently dropped when UAR's own session history
  is converted into KnowMe's local-inference request format.**
  `uar_message_to_inference_message`
  (`gen_ui_protocol/src/lib.rs:161-178`) only reads `role`, `content`, `name`,
  and the singular `tool_call_id` off each UAR session message — it never reads
  the `tool_calls` array UAR's own `Message` struct serializes for an
  assistant turn that made a tool call
  (`universal-agent-runtime/src/session/thread.rs:167-179`,
  `.../src/llm/mod.rs:64-90`). Once a conversation has gone through even one
  tool call on the UAR lane (which the local lane's tool loop also produces,
  since KnowMe's own `LocalToolCallGate` and UAR's orchestrator both support
  local tool calling), UAR's persisted session history contains an assistant
  message whose `tool_calls` are silently dropped the next time that history is
  re-sent to the local driver bridge. The model then sees a **truncated,
  self-contradictory** conversation (an assistant turn with empty content, no
  visible reasoning for the following tool-result message, because the tool
  call that "explains" that result vanished on the KnowMe side).
- **Empty-content assistant turns are dropped from UAR's session entirely on
  the RunManager side** if a run ends with no text and no tool calls
  (`universal-agent-runtime/src/uar/runtime/manager.rs:2039-2041`,
  `if !accumulated_content.is_empty()`). Combined with the point above, a
  UAR→local→UAR round trip can leave UAR's session history internally
  inconsistent in more than one way, and the *local* engine (LiteRT-LM via the
  Kotlin bridge) receiving that inconsistent history is exactly what's
  observed as "UAR stops working" — the shared engine isn't broken, the
  **conversation payload UAR is replaying into it is corrupted upstream of the
  JNI boundary.**
- **Context-window trimming can orphan a tool-call/tool-result pair.** UAR's
  `ContextManager::apply_sliding_window`
  (`universal-agent-runtime/src/uar/runtime/context/manager.rs:86-141`) trims
  purely by estimated token count against the model's `context_window` (8192
  for both mobile provider registrations —
  `gen_ui_agent/src/uar.rs:114,158`) and always keeps the system message, but
  has **no awareness of tool-call/tool-result pairing**. A trim that drops an
  assistant tool-call message while keeping its paired tool-result message (or
  vice versa) produces a message sequence most model chat templates and/or
  KnowMe's local conversion layer will reject or mishandle. This is a second,
  independent way the same "lost tool call" failure mode can be triggered
  purely by conversation length, without ever switching lanes.
- **`RUN_FINISHED` can be misreported as `RUN_ERROR`.** If a UAR embedded stream
  ends while a `<think>...</think>` block was left open,
  `A2uiAdapter`'s reasoning-flush path emits `A2uiEvent::RunError` instead of
  `A2uiEvent::RunFinished`
  (`gen_ui_protocol/src/lib.rs:1430-1461`, referenced by
  `gen_ui_agent/src/uar.rs:850-856`). This doesn't corrupt state by itself, but
  it means a lane-switch sequence that happens to end a UAR turn mid-`<think>`
  will surface as a hard error to the Flutter UI even though generation
  technically completed — worth ruling out when reproducing "UAR stops
  working," since it can look identical to a real failure from the UI's
  perspective.
- **`RUN_ERROR`'s `code` field from the producer is discarded** — the projector
  hardcodes `RunError.code = "inference_error"` regardless of what the embedded
  stream actually reported (`gen_ui_protocol/src/lib.rs:1448-1461` vs.
  `gen_ui_agent/src/uar.rs:972-979`). This doesn't cause the bug but makes it
  harder to diagnose from client-visible logs/telemetry alone, since the
  original error code/message detail from the JNI/engine layer never reaches
  the UI as anything other than a generic string.

### 2.3 Contributing (non-causal but relevant) defect: `active_runs` is never cleaned up

`RunManager.active_runs` is inserted into once per run
(`universal-agent-runtime/src/uar/runtime/manager.rs:888-897`) and is **never
removed** — grep confirms the only write is the initial insert; the terminal
cleanup path only removes the run's cancellation token
(`manager.rs:2200-2205`), not the `active_runs` entry itself. This means every
run's `broadcast::Sender` (buffer 256) and full event-history buffer are
retained for the life of the process. This is a real, unbounded memory leak,
and worth fixing, but it does **not** block or hang subsequent runs in the same
session — `start_run_with_policy` never waits on or reads a prior run's
channel/sender state, and there is no per-session lock. Flagging it here
because it's the kind of thing that eventually produces its own separate
memory-pressure symptom (which could itself look like "stops working" after
enough turns/conversations), but it is not the mechanism for the specific
switch-back bug reported.

### Most likely explanation for the reported Android symptom, ranked

1. **(Primary)** A tool call happened at some point in the conversation (very
   likely, since both lanes support local tool calling and it's a natural thing
   to trigger during ordinary use). UAR's session now holds an assistant
   message with `tool_calls` that KnowMe's `uar_message_to_inference_message`
   silently drops when that history is replayed into the local `LiteRtLmEngine`
   after switching back to the UAR lane — producing a malformed/self-
   contradictory prompt that the model (or the Kotlin bridge's response
   parsing) cannot recover from, which surfaces to the user as "UAR just isn't
   responding anymore" in that conversation.
2. **(Secondary, same family)** The conversation's `<think>` reasoning
   accounting left a block open at the exact moment of the lane switch,
   causing the next UAR turn's completion to be misreported as `RunError`
   rather than `RunFinished` — this looks identical to "UAR stopped working"
   from the Flutter UI but is a projector bug, not an inference failure.
3. **(Tertiary)** If the conversation was long enough to trigger UAR's sliding-
   window context trim, a tool-call/tool-result pair could have been split,
   independently producing the same malformed-history symptom described in #1.

None of these three require a lane switch to reproduce in principle (#1 and #3
only require a prior tool call plus enough conversation for UAR to replay that
history through the local driver bridge again — switching lanes is simply the
easiest way to force a second `stream()` call against the already-corrupted
session history). The "switch back to UAR" step is likely the trigger that
happens to be the *first point* the corrupted history gets replayed through the
local engine a second time, not a defect in the switching mechanism itself.

---

## Summary table

| Platform | Lane | Status | Root cause |
|---|---|---|---|
| iOS | local | broken | 5.68 GB model + no memory entitlement → jetsam kill on load; fallback can't catch a process kill |
| iOS | UAR | broken | shares the exact same engine instance/model as local lane — same jetsam kill |
| Android | local | works | 2.6 GB model, certified, no shared-history replay issue in this lane alone |
| Android | UAR (first turn / fresh conversation) | works | empty seeded route falls back cleanly to global config; correct driver still invoked |
| Android | UAR (after switching back mid-conversation) | breaks | UAR's own session history — corrupted by dropped `tool_calls` and/or orphaned tool-result pairing from context trimming — gets replayed through the shared local engine and produces a malformed/unrecoverable prompt |

## What this diagnosis does *not* cover

- No physical-device reproduction was performed as part of this diagnosis; all
  conclusions are from static code tracing across `gen_ui_ffi`, `gen_ui_agent`,
  `gen_ui_inference`, the vendored `universal-agent-runtime` (embedded runtime,
  `RunManager`, `Orchestrator`, session store), `gen_ui_protocol`, and the
  Android Kotlin JNI bridge.
- The exact trigger conditions in #2 (how many turns, whether a tool call is
  strictly required, whether context trimming alone can reproduce it without
  any tool call) would benefit from an instrumented on-device repro before
  landing a fix, since three independent defects in the same area (dropped
  `tool_calls`, empty-turn history loss, un-paired context trimming) could each
  independently produce the same user-visible symptom.
