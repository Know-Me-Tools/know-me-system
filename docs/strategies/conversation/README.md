# Conversation UX Strategy — Decision Record and Completion Scope

**Scope:** the conversation surface (chat thread, composer, models surfaces,
settings, inspector) — the resolved product decisions for the Flutter UI/UX
spec and the remaining implementation areas. The full component-level contract
lives in **PAGS-SPEC-KM-UIX-001** (`docs/knowme-flutter-uiux-spec.md`); this
document records the decisions that govern it and tracks the spec's own
reconciliation needs.

**Status:** v1.0, 2026-07-19. Companions: PAGS-SPEC-KM-CTX-001 (context
fabric), `docs/strategies/ag-ui/README.md` (event stream), `docs/strategies/a2ui/README.md`
(declarative surfaces), `docs/strategies/model-configuration/README.md`
(resolution levels), `docs/strategies/local-models/` (Android runtime direction).

---

## 1. Resolved decisions (2026-07-19, product-approved)

These resolve the open questions raised in UIX-001 §17. They are binding on
the next revision of that spec and on implementation.

1. **Navigation: fold Models into Settings → Model & Inference.** The shell
   keeps five destinations (Home, Chat, Hands, Memory, Settings) — inside the
   `mobile-navigation` cap — and the Models surfaces (UIX-001 §8) become a
   Settings section, still deep-linked from the chat model banner. There is no
   sixth bottom-bar destination.
2. **16K extended context: Developer-flag-only in v1.** The trusted ceiling
   ships at 8K everywhere; the 16K toggle appears only with Developer mode on
   (UIX-001 §13.4/§13.6). Inherits CTX-001 §15.5.
3. **Fact extraction: per-assistant opt-in, default off.** No global default.
   Each assistant's enrollment toggle, the review-and-delete surface, and the
   provenance requirements stand as specced (UIX-001 §13.3; CTX-001 §8.5).
4. **Citation adjacency: presentation-only re-flow.** `citation` blocks keep
   their recorded stream order; at finalization the presentation layer moves
   their rendering to chips adjacent to the supported answer run (UIX-001
   §6.3). Persistence order never changes.
5. **Mermaid: keep the WebView-class renderer.** `flutter_mermaid` stays,
   with the source view and accessible-description fallback (UIX-001 §17.5);
   pure-Dart rendering is revisited only when a mature renderer exists.
6. **Benchmark runner: in the Models surface, with a cooldown.** Self-serve
   re-certification (the ≥15% acceleration gate) ships in the Models section
   with an explanation and a cooldown, mirrored read-only in Developer
   (UIX-001 §8.1, §13.6).

## 2. Flag: Android runtime stance needs one reconciliation pass

UIX-001 was written against the Unified Inference Recovery and Context Fabric
Plan's Android stance (Qwen3-4B Q5_K_M, llama.cpp, OpenCL/Adreno behind the
≥15% gate). The in-repo Android direction — `docs/strategies/local-models/android-litert-lm-six-gates-roadmap.md`
and `android-native-inference-future-release-roadmap.md` (both 2026-07-19) —
records the **supersession of the Vulkan/OpenCL plan**: the production Android
lane is **LiteRT-LM with Gemma 4 E2B, CPU as the universal correctness floor,
optional device-specific acceleration only after physical proof**, with
llama.cpp retained as an optional CPU/GGUF fallback and desktop/server lane.

Consequence: UIX-001's backend-honesty copy (§6.7 backend chip, §8.1 backend
row, §8.4 capability figures, §13.2 sampling display) names the llama.cpp
stack and must be re-worded to the delegate/provider language of the LiteRT-LM
lane (CPU baseline; GPU/NPU delegates as gated variants; Gemma 4 E2B as the
default model) in the spec's next revision. The *structure* of those surfaces
(active-backend honesty, certification gate visibility, fallback reasons,
nominal-vs-trusted context) is runtime-neutral and stands. **Action: one
targeted revision pass on `docs/knowme-flutter-uiux-spec.md` sections 6.7,
8.1, 8.4, 13.2, 15–16** when the spec is next edited; not a rewrite.

## 3. Completion scope for this area

Per the verified current-state inventory (UIX-001 §3), the conversation area
is complete **except** the following, which constitute its implementation
backlog (no code authorized here — scope tracking only):

| Surface | State | Spec section |
|---|---|---|
| Named-phase lifecycle strip (replaces generic spinner) | Missing | UIX-001 §6.4 |
| Context inspector (profile/estimator/slices/drops/degradation/escalation + prompt plan + events) | Missing | §10 |
| Branch navigation (`‹ n/m ›`, branch badges) | Missing | §7.2 |
| Conversation documents panel (scoped docs, promote-to-knowledge) | Missing | §9.3 |
| Settings sections (Model & Inference, Context & Memory, Budget, Developer) | Missing | §13 |
| Models surfaces (library, download states, certification gate) — now under Settings per §1.1 | Missing | §8 |
| Attachment extraction states (Xberg/OCR per-chip) | Partial | §9.1 |
| Startup lazy-init status surface | Partial | §12 |
| Thinking cyan tint + duration metadata | Delta | §6.2 |
| Turn anatomy, actions, error/cancelled bubbles, tool confirmation | **Exists — formalize only** | §6–§7 |

Dependencies: the lifecycle strip and inspector consume the AG-UI envelope
work (`docs/strategies/ag-ui/README.md` §5.2–5.3); settings forms ride the
A2UI settings generator (`docs/strategies/a2ui/README.md` §4); models
surfaces follow the Android LiteRT-LM lane's certification scripts
(`docs/strategies/local-models/`).

## 4. Invariants carried into every change here

- Flat 2.0, token-sourced styling, both themes (design standard; skills).
- The 11-variant ContentBlock union stays closed; lifecycle and telemetry
  travel in the run-event envelope.
- Layering: Widget → @riverpod provider → Repository → FFI (Rust core);
  blocks mutate only via `ChatNotifier.streamBlock`.
- Reasoning is displayed, persisted, and excluded from future prompts.
- Mutating tools always confirm; destructive actions are branch-scoped and
  confirmed; nothing destructive is enabled while its branch has an active
  run.
