# PAGS-SPEC-KM-UIX-001 — KnowMe Flutter UI/UX for Unified Inference Recovery and Context Fabric

**Flutter mobile UI/UX contract for conversation recovery, lifecycle surfaces, branching, model management, attachments, and the context inspector**

| | |
|---|---|
| Document ID | PAGS-SPEC-KM-UIX-001 |
| Status | Draft for review — no code authorized until phase gates approved |
| Date | 2026-07-19 |
| Author | Kimi (session work product for Travis James, Prometheus AGS / KnowMe LLC) |
| Applies to | `know-me-system/mobile` (Flutter iOS/Android), `flutter_packages/gen_ui_widgets`, `flutter_packages/gen_ui_flutter` |
| Supersedes | Nothing. This is the first UI/UX feature spec; where it conflicts with older mockups, this document wins for the surfaces it covers |
| Governing | Unified Inference Recovery and Context Fabric Plan (2026-07), PAGS-SPEC-KM-CTX-001, `docs/knowme-ui-ux-standard.md` (binding design standard), TJ-ARCH-MOB-001, AGENT_BASE_RULES.md |
| Platform notes | Android = Qwen3-4B-Instruct-2507 Q5_K_M, text-only, non-thinking, llama.cpp CPU with gated OpenCL/Adreno. iOS = Qwen3.5 9B, vision, Metal, reasoning-capable |

---

## 1. Purpose and scope

This spec defines every Flutter UI/UX surface required by the Unified Inference
Recovery and Context Fabric Plan ("the Plan"). It covers what the user sees and
touches; it does not restate the Rust contracts except where the UI consumes
them. It is written to be implemented delta-first: several surfaces already
exist in partial form (§3) and must be formalized, not rebuilt.

In scope:

- Chat turn anatomy: ordered regions, thinking, lifecycle, tool/skill/memory
  panels, answer bubble, citations, error/cancelled bubbles.
- Message actions and branch behavior: Copy, Resubmit, Delete, Regenerate,
  Stop/Cancel, branch navigation, disabled-state rules.
- Run lifecycle UI: named phases, model download/load progress, cancellation,
  accessible announcements, crash-restoration presentation.
- Composer and attachments: Xberg/OCR states, conversation-scoped documents,
  promotion to durable knowledge.
- Models screen: library, download/resume/cancel, verification, backend
  diagnostics, the OpenCL ≥15% certification gate outcome, active-backend
  honesty.
- Context inspector: profile, estimator, slice spending (30/70), dropped
  items, degradation, escalation advice, unified with the activity/event
  inspector required by the design standard.
- Startup and lazy-initialization status surfaces.
- Settings and configuration schema: context strategy, developer flags,
  per-assistant fact-extraction opt-in, lane/model selection, cloud presets.
- Accessibility, motion, golden/behavior test matrices, and acceptance gates.

Out of scope: React/Tauri implementation (the standard governs parity; this
spec cites parity expectations where the Flutter surface leads), Rust
internals, and any code change authorized by this document alone.

Non-negotiable invariants inherited by every section:

- **Flat 2.0** — no borders, dividers, decorative outlines, or layout shadows;
  regions differ only by background color (`docs/knowme-ui-ux-standard.md` §3).
- **Layering** — `Widget → @riverpod provider → Repository → FFI (Rust core)`.
  Widgets are pure projections; state mutates only through providers; the Dart
  A2UI/FFI mirrors are regenerated, never hand-edited.
- **Tokens** — all color/spacing/type/radius values come from the shared token
  source projected into `KnowMeTheme`/`KnowMePalette`
  (`mobile/lib/core/theme/tokens.dart`); no raw `Color(0xFF…)` or magic
  `EdgeInsets` at call sites.
- **ContentBlock exhaustiveness** — one widget per variant, `switch` with no
  `default`, in `gen_ui_widgets` (see `.kimi-code/skills/content-block-ui`).

## 2. Governing constraints distilled

From the Plan, PAGS-SPEC-KM-CTX-001, the design standard, and project skills —
the rules every surface below must satisfy:

1. **Named phases replace spinners.** Runs emit `RunStatus` (queued, preparing,
   thinking, generating, complete, cancelled, error) and named phases (context
   assembly, document extraction, OCR, embedding preparation, memory retrieval,
   knowledge retrieval, tool selection, summarization, model download,
   verification, loading, fallback selection, tool execution, generation,
   persistence). The Plan forbids temporary generic spinners at any later
   stage; the final lifecycle components land with the state machines.
2. **The 11-variant ContentBlock union is closed.** Lifecycle and telemetry
   travel in the run-event envelope, never as new block variants. Unknown
   provider events surface as `ArtifactBlock` records (provider, event name,
   original payload) and are rendered by the artifact widget's raw-event path.
3. **Backend honesty.** The active backend (`cpu` / `opencl` / `metal` /
   cloud), device, offloaded layers, and any fallback reason are visible
   product information, not debug data. OpenCL is active only when the
   physical-device certification (correct, thermally safe, ≥15% median
   improvement) passes; otherwise CPU runs with a visible reason.
4. **Android is text-only and non-thinking.** Android attachments always pass
   Xberg/OCR before the text-only model; no image tensor is submitted and no
   thinking UI is promised on the Android local lane. The thinking surface is
   still specified (iOS, cloud, and future models require it) but must degrade
   to a documented empty state on Android, not a broken one.
5. **Reasoning is displayed, persisted, and quarantined.** Thinking blocks
   render and persist, but prior reasoning is excluded from subsequent prompts,
   summarization, fact extraction, and tool context. The UI never offers "send
   thinking back to the model" affordances.
6. **Mutating tools always require confirmation** (`AlwaysConfirm`,
   non-disableable by non-privileged layers, CTX §7.1/G4). The confirmation
   surface is a designed, accessible flow — never a raw JSON dump.
7. **Budget UX is a differentiator.** The context inspector is a product
   surface: profile, estimator, slice spending, dropped items, degradation,
   and escalation advice are all renderable from `ContextTelemetryEvent` and
   `PromptPlan` mirrors (CTX §11.2, §12).
8. **Tightening-only user overrides** (CTX §6.6): user-level budget controls
   may only reduce envelopes, never raise them.
9. **Destruction is confirmed and branch-scoped.** Delete removes the response
   plus its branch tail, invalidates branch summaries, and cleans fact
   provenance; destructive and resubmit actions are disabled while the
   affected branch has an active run.
10. **WCAG 2.2 AA everywhere:** 4.5:1 body / 3:1 large-text and component
    contrast in both themes, accessible names on every actionable element,
    keyboard/focus order, `liveRegion` announcements for streaming and status,
    ≥24 px targets (44 px primary touch), reduced-motion path, layouts that
    survive 200% text scaling.

## 3. Current-state inventory (delta baseline)

Verified against the working tree on 2026-07-19. "Formalize" means keep the
implementation and bring it to this spec; "extend" means add missing states or
data; "new" means the surface does not exist.

| Surface | Location | State today | Disposition |
|---|---|---|---|
| App shell, 6-destination bottom bar | `mobile/lib/app/router.dart` | Working (Home, Chat, Hands, Memory, Models, Settings) | Keep; resolve >5-destination policy (§4.2) |
| Chat screen, composer, scroll behavior | `features/chat/presentation/screens/chat_screen.dart` | Working | Formalize + extend (§6) |
| Turn segmentation (panels vs answer bubble, 80% cap) | `_AssistantTurn` in chat_screen.dart | Working | Formalize (§6.1) |
| Thinking panel (streaming-expand, manual override, copy) | `ContentBlockThinking` in `gen_ui_widgets` | Working | Extend: cyan tint, duration metadata, empty-state rule (§6.2) |
| Error/cancelled bubbles | `_RunErrorBubble`, `_RunCancelledBubble` | Working | Extend: retryability, phase/code chips, crash-restore variant (§6.5) |
| Run status bubble | `_RunStatusBubble` (spinner + statusMessage) | Partial | Replace with named-phase lifecycle strip (§6.4) |
| Model operation banner (progress, cancel, failed) | `_ModelOperationBanner` | Partial | Extend: resume affordance, verification state, backend context (§8.2) |
| Actions: Resubmit/Copy/Delete/Regenerate, Stop send button | chat_screen.dart | Working | Extend: branch semantics, active-run disable rule (§7) |
| Tool confirmation card + AG-UI confirmation | `_ToolConfirmationRequiredBlock`, `_ConfirmationRequest` | Working | Formalize as the AlwaysConfirm flow (§6.6) |
| Attachment strip (chips, 25 MB / 6-file guard) | `_AttachmentStrip` | Partial | Extend: extraction states; add document panel + promotion (§9) |
| Conversation sheet | `_ConversationSheet` | Partial | Extend: branch/running/failed badges, drafts (§11) |
| Startup gate (phase progress, boot error) | `features/startup/.../startup_gate.dart` | Partial | Extend: decoupled lazy-init status (§12) |
| Theme/palette | `core/theme/tokens.dart` | Working | No change; consume only |
| Models screen | placeholder in `overview_screens.dart` | Missing | New (§8) |
| Settings screen | placeholder (theme toggle only) | Missing | New (§13) |
| Context inspector | — | Missing | New (§10) |
| Branch navigation | — | Missing | New (§7.2) |
| ContentBlock dispatcher + variant widgets | `gen_ui_widgets/lib/src/content_block_view.dart` | Working (all 11 variants, mermaid/SVG/video/OG) | Keep; per-variant extensions only |

## 4. Information architecture

### 4.1 Destinations

The six-tab shell stays. The work in this spec concentrates in **Chat**
(primary), **Models** (new), **Settings** (new), and **Startup** (gate).
Memory and Hands are untouched except where memory panels appear inside chat.

### 4.2 Navigation policy note (decision required)

`mobile-navigation` caps bottom-bar destinations at 5; the shell currently has
6. This spec does not silently violate the skill. Options, in preference
order: (a) fold **Models** into **Settings → Model & Inference** and keep 5
destinations; (b) keep 6 and record an accepted deviation in this spec's
release notes. **Recommendation: (a)** — the Models surface is opened most
often from chat's model banner and settings, and 5 destinations preserve M3
rhythm. Decision owner: product. Until decided, the route table is unchanged.

### 4.3 Chat sub-surfaces

| Surface | Entry | Dismissal |
|---|---|---|
| Conversation library sheet | Header icon (exists) | Swipe/back (exists) |
| Conversation documents panel | Header "Documents" action when the conversation has ingested files | Sheet close |
| Run/context inspector | Telemetry chip on any assistant turn; inspector icon on the model banner | Sheet close |
| Model detail | Models screen row, or chat model banner | Route pop |
| Settings sections | Settings destination rows | Route pop |

Sheets use `shadcn_flutter` overlay primitives restyled Flat 2.0 (scrim, no
shadow, no border). All sheets are reachable by screen reader and keyboard.

## 5. Data contracts the UI consumes

The UI renders only generated Dart mirrors (FRB) of these Rust types. Field
names below are logical; exact Dart names come from codegen. This section is
the widget-to-contract index, not a redefinition of the contracts.

| Contract | Carries | Consumed by |
|---|---|---|
| `ContentBlock` (11 variants) | Ordered semantic blocks per message | `ContentBlockView` dispatcher (exists) |
| `RunStatus` + named phase + `statusMessage` | Lifecycle of one run | Lifecycle strip (§6.4), message status, library badges |
| `RunError` | Stable code, phase, safe message, retryable, diagnostics | Error bubble (§6.5), announcements |
| `ModelOperationState` | not installed / downloading(bytes, rate, resumable) / verifying / loading / ready / cancelling / cancelled / failed(RunError) | Model banner (§8.2), Models screen (§8.3), startup (§12) |
| `RuntimeDiagnostics` | modelId, backend, deviceName, offloadedLayers, fallbackReason | Backend chip (§6.7), Models screen, inspector |
| `ContextTelemetryEvent` | strategyKind, tokensBefore/After/Saved, messagesRemoved, slices(kind, budget, spent, items), estimator, degraded, escalation, trigger | Telemetry chip, context inspector (§10) |
| `PromptPlan` (mirror) | Exactly what will be sent | Inspector "Prompt plan" view (§10.4) |
| `EscalationAdvice` | Typed cloud-handoff recommendation | Escalation card (§10.5) |
| `ChatMessage` mirror | id, role, blocks, runId, parentMessageId, branchId, status, error, attachments, generationSettings, runtimeDiagnostics, contextPlanVersion, timestamps | Thread, branches, actions |
| Conversation-scoped document record | name, mime, chunk count, extraction state, provenance, promotion state | Documents panel (§9.3) |
| Capability profile (baked) | device/model/runtime profile, trusted ceiling, nominal context | Models screen, inspector, capability explainer (§8.4) |

Persistence expectations the UI relies on (owned by PEM/Rust, listed here
because UI behavior depends on them): per-message entities with stable ordered
block wrappers; stream checkpoints at semantic boundaries plus a short
debounce; atomic finalization; crash restore reproducing exact block order,
status, errors, settings, diagnostics, and branch identity.

## 6. Chat thread — turn anatomy

### 6.1 Ordered regions (formalized)

An assistant turn renders, in order:

1. **Thinking region** (0..n `thinking` blocks) — outside the answer bubble.
2. **Lifecycle region** — the named-phase strip while the run is active;
   collapses to a summary line on completion (§6.4).
3. **Activity panels** — `toolUse`/`toolResult`/`skill`/`memory` panels in
   stream order, outside the answer bubble.
4. **Answer bubble** — `text`/`code`/`artifact`/`image`/`divider` runs, each
   maximal contiguous run wrapped in one surface, capped at 80% of viewport
   width (max 720 px), left-aligned.
5. **Citation attachments** — adjacent to the answer content they support
   (§6.3).
6. **Terminal bubbles** — persisted error or cancelled bubble after the last
   semantic block (§6.5).
7. **Action row** — right-aligned, below the turn (§7.1).

The existing `_AssistantTurn` segmentation implements regions 1–5; this spec
formalizes that behavior and adds the lifecycle strip and citation adjacency
rules. Region backgrounds come from the surface ladder: panels on `surface`,
the answer bubble on `card`/raised, thinking on the cyan-tinted surface
(§6.2). No region may introduce a border, divider, or shadow.

### 6.2 Thinking region

Behavior (formalizing `ContentBlockThinking`):

- **Expanded while streaming** its own block; **auto-collapses when answer
  content begins**, unless the user manually expanded or collapsed it. A
  manual choice always wins over automation for that block.
- Collapsed header shows: brain/psychology icon, "Thinking" label, and —
  once terminal — **duration metadata** ("Thought for 42 s"); while streaming,
  a live elapsed time replaces any spinner glyph.
- Body is **selectable plain text with whitespace preserved**; no Markdown
  rendering inside thinking; copy button copies the raw trace.
- **Tint:** cyan-tinted flat surface per design standard §7.5 (cyan =
  reasoning/citations/streaming). Current implementation uses the generic
  `secondary` surface — migrating to the cyan annotation tint is a formal
  delta of this spec, applied via tokens in both themes.
- **Android empty state:** on the Android local lane (non-thinking Qwen), the
  region simply never appears. If a thinking toggle exists in settings
  (§13.3), it is shown disabled with the explanation "The on-device Android
  model does not produce reasoning traces."
- Reasoning quarantine is invisible in day-to-day UX except one honesty touch:
  the inspector's prompt-plan view marks prior thinking blocks as excluded
  from context (§10.4).
- Accessibility: header exposes expanded/collapsed state programmatically;
  body announces via polite live region while streaming; expanded thinking is
  reachable in keyboard focus order after the header.

### 6.3 Citation attachments

The Plan moves citations from "panel among panels" to **adjacent to supported
answer content**:

- `citation` blocks render as compact source chips **inline at the end of the
  answer run they cite** (trailing the bubble), not interleaved with tool
  panels. Each chip: source icon, short label, count badge when several
  chunks ground the same span.
- Tapping a chip opens the citation card (existing `_BlockSurface` pattern):
  source title, excerpt, provenance (conversation / message / chunk / run),
  and — for memory citations — a "view memory" jump that opens the Memory
  destination at that record without losing thread position.
- Fallback: when a citation arrives before its answer run exists (streaming
  edge), it renders in the activity-panel region and migrates adjacency on
  finalization. The block order on record never changes; this is a
  presentation rule only.
- Citations use the cyan annotation tint; status is never by color alone
  (icon + "Citation · {source}" label).

### 6.4 Lifecycle region (replaces `_RunStatusBubble`)

While a run is active, a single strip above the answer area shows the run's
named phase — this is the component the Plan requires to exist before any
later stage may ship (no generic spinners downstream):

- Content: phase icon + human phase label ("Retrieving memories", "Reading
  document", "Selecting tools", "Generating") + elapsed time + **Stop**
  affordance (44 px, accessible name "Stop generating").
- Phase label mapping is a fixed table owned by the presentation layer; an
  unrecognized phase renders its snake_case name humanized — never a raw
  enum string, never blank.
- The strip is a **polite live region**: it announces on phase *change*, not
  on every tick; elapsed time is visual only.
- On completion the strip collapses to a one-line summary ("Generated in
  8.2 s · on-device · CPU") which remains persisted with the message; on
  error/cancel it is superseded by the terminal bubble.
- Queued/preparing states before the first phase event show "Preparing
  response" — this is a named state, not a spinner.
- Model-bound phases (download, verification, loading, fallback selection)
  are normally surfaced by the model banner (§8.2); if a run hits them
  mid-thread (e.g. first send after install), the strip mirrors the same
  label so the user never sees a dead turn.

### 6.5 Error and cancelled bubbles (extended)

Existing `_RunErrorBubble` / `_RunCancelledBubble` are extended, not replaced:

- Error bubble content, in order: error icon, **safe message** (plain
  language, from `RunError.message` — never a transport exception string),
  phase chip (e.g. "while loading model"), stable code in mono metadata,
  and a **retry affordance when `retryable`** ("Try again" re-runs the
  originating user message through the same path as Regenerate).
- Non-retryable errors state the recovery path in one sentence ("This model
  file failed verification. Re-download it from the Models screen.") with a
  deep-link action where one exists.
- Cancelled bubble: stop icon + "Response cancelled" + elapsed-before-cancel;
  no destructive styling (secondary surface).
- **Crash-restoration variant:** when a restored message has a non-terminal
  run status that can never resume (process death), the terminal bubble reads
  "Interrupted when the app closed" with a "Regenerate" action. Restoration
  must reproduce exact block order, status, and diagnostics; partial blocks
  keep their content with a "restored partial response" metadata line.
- Both bubbles persist with the message and appear in goldens (§15).

### 6.6 Tool, skill, and memory panels (formalized)

- Panels render status, not raw JSON: `pending → running → complete/failed`
  with disclosure for input/output (existing `_DisclosureBlock` pattern).
- **AlwaysConfirm flow:** a mutating tool renders the confirmation card in
  the panel region: tool name, human description of the mutation, parameters
  summary, consequence text, and **Confirm / Deny** actions (44 px, safe
  default = Deny, keyboard-focusable, announced as an alert). Confirmation
  resumes the exact stored call (existing `local_tool_approve` interaction);
  denial records a failed/skip result so the model can continue.
- Restored confirmations that can no longer resume (process-local approvals)
  show the existing "resubmit to create a fresh approval request" guidance —
  keep this copy.
- Repair attempts are visible: "Retrying tool call (2 of 2)" as panel status
  metadata, from the bounded-repair contract.
- Skill panels show skill identity + status + output summary behind
  disclosure; memory panels distinguish recalled / proposed / written /
  updated / rejected (existing mapping) and link to the Memory destination.

### 6.7 Backend chip

Every assistant turn carries a small metadata chip (mono, faint text,
non-interactive-by-default): model short name + backend + lane, e.g.
`Qwen3-4B · CPU · on-device` or `Qwen3.5-9B · Metal · on-device` or
`Gemma-4 · Cloud`. When `fallbackReason` exists, the chip gains a warning
icon and opens the inspector's diagnostics section on tap (e.g. "CPU
selected: OpenCL requires a physical-device certification…"). Lane wording
follows the standard's "On-device / My server / Cloud provider" trichotomy.

## 7. Message actions and branch behavior

### 7.1 Action inventory and rules

| Action | On | Behavior | Disabled when |
|---|---|---|---|
| Copy block | Every block header (exists) | Copies that block's canonical text | Never |
| Copy answer | Assistant action row (exists) | Copies answer-region blocks only (not thinking/panels) | Empty answer |
| Copy message | User bubble (exists) | Copies full user content | Never |
| Stop/Cancel | Active run (send button morphs, exists; lifecycle strip, §6.4) | Emits cancellation; terminal cancelled bubble follows | Not disabled while active |
| Resubmit | User message | **Creates a new branch**: saved user content + attachments + branch prefix + current settings + newly assembled context plan | Affected branch has an active run |
| Regenerate | Assistant turn | Branch-tail cleanup, then reruns the preceding saved user message | Affected branch has an active run |
| Delete | Assistant turn | Confirms scope, removes response + branch tail, invalidates branch summaries, cleans solely-originating fact provenance | Affected branch has an active run |

- The active-run disable rule is **branch-scoped**, not global: actions on
  other branches/conversations remain available. Disabled actions show a
  tooltip/explanation ("Wait for the current response to finish").
- Delete confirmation copy states the blast radius: "Delete this response and
  the N messages after it on this branch? Memories saved from them are
  removed too." Confirm is destructive-styled; recovery is not promised.
- All actions keep ≥48 px hit constraints (existing), tooltips, and
  accessible names; the action row is reachable after the turn content in
  focus order.

### 7.2 Branch navigation (new)

Branches are invisible today; the Plan makes them first-class:

- Any message with siblings (created by Resubmit/Regenerate) shows a compact
  **branch navigator** trailing the turn: `‹ 2/3 ›` monochrome, 44 px
  targets, accessible label "Response 2 of 3, branch {branchId}".
- Switching swaps the branch tail in place with the persisted block order of
  the selected sibling; scroll anchor stays on the navigator's message.
- The active branch id is conversation state, restored on relaunch; the
  conversation header shows a "Branch" badge when viewing a non-main branch,
  with a "Jump to main" action.
- The conversation library marks branched conversations (existing
  `branched` view expectation in the standard §7.8).
- Branch creation is never silent: a one-line divider-less metadata row in
  the thread marks "Branched from an earlier message" at the fork point.

## 8. Models surfaces

### 8.1 Models screen (new — `features/models`)

Design standard §9: "a user-owned library, not a storefront." Sections:

1. **Active model card** — model name, quant (`Q5_K_M`), size on disk,
   lane ("On-device"), **acceleration backend** (`CPU` / `OpenCL (Adreno)` /
   `Metal`), device name, offloaded layers when known, and status
   (`Ready`, `Not installed`, operation-in-progress). Android copy names the
   model honestly: "Qwen3-4B-Instruct-2507, Q5_K_M, ~2.9 GB".
2. **Backend row** — active backend with certification summary: "OpenCL
   locked: needs an on-device benchmark ≥15% faster than CPU" or "OpenCL
   certified: +23% median, thermally safe" or "CPU active: certification
   failed (correctness)". A "Re-run benchmark" action lives here (developer
   section mirrors it), producing the certification artifact the Rust gate
   consumes. Benchmark UX shows: running state with per-config progress
   (CPU run, OpenCL run), thermal readings, and the pass/fail verdict with
   the three criteria itemized.
3. **Available models** — the pinned catalog (Android: Qwen3-4B; iOS:
   Qwen3.5 9B) with capability tags (text-only / vision / reasoning),
   context figures (§8.4), and per-model state + primary action
   (Download / Resume / Cancel / Delete / Activate).
4. **Remote models** — BossFang-linked remote entries marked "Remote —
   runs on your server", visually and verbally distinct from local (standard
   §7.3 trichotomy); no remote row may look installed when it is not.
5. **Cloud presets** — baked, versioned presets listed read-only with
   provider, model, and capability tags; enabling one requires the lane/BYOK
   configuration in Settings (§13.2) and is marked "Cloud".

Empty state (no model installed): explains local-first value, download size
and time expectation, and one primary "Download Qwen3-4B" action — matching
the standard's "one-action recovery" rule.

### 8.2 Model operation banner (extended)

The existing chat `_ModelOperationBanner` extends to the full
`ModelOperationState` machine:

- States and copy: Not installed → slim info bar "No local model yet — chat
  uses cloud until one is installed" + action; Downloading → percent, data
  rate, **resumable indicator** ("resumes after interruption"), Cancel;
  Verifying → indeterminate-with-label; Loading → "Loading model into
  memory"; Ready → auto-dismiss with a transient confirmation; Cancelling →
  label only; Cancelled → "Download cancelled — resume anytime" + Resume;
  Failed → `RunError` safe message + retry/deep-link.
- The banner is the **single** place model operations appear in chat; the
  Models screen shows the same state machine on the active-model card (one
  source of truth via the shared provider).
- Polite live region announcing state transitions only; progress percent is
  visual.

### 8.3 Download integrity presentation

- Verification state is named ("Verifying download"), never folded into the
  progress bar.
- A failed checksum deletes the artifact and shows "Download failed
  integrity check and was removed" with a retry — this is the honesty
  standard applied to model files.
- Byte counts use the pinned catalog values; resumed downloads show
  "resumed at N%" once.

### 8.4 Capability honesty (nominal vs trusted context)

Model rows and the inspector show two figures with an explainer
(CTX §6.2): **nominal context** ("256K tokens — the model's advertised
window") and **trusted ceiling** ("8K tokens — verified reliable on this
device"). The explainer sheet (one tap on the figures) states: ceilings come
from on-device measurements, 16K is available in Developer settings, and
raising the ceiling trades reliability for history. iOS rows show the same
pattern with the 9B profile.

## 9. Composer and attachments

### 9.1 Attachment strip (extended)

Existing chip strip (25 MiB / 6-file guards, error line) gains **per-chip
extraction state** once sent is pressed and extraction begins:

- `pending → extracting → OCR → ready → failed`, shown as a state dot +
  label on the chip ("Reading", "OCR", "Ready", "Couldn't read").
- Android rule surfaced honestly: images and scans show "Text will be
  extracted — the on-device model can't see images directly" as a one-time
  hint under the strip (dismissible, per-conversation memory).
- Extraction failure never blocks silently: the chip turns destructive-tinted
  with the safe message, and send proceeds only after the user removes the
  failed file or explicitly sends without it (a per-chip "Send anyway as
  file reference" is **not** offered on Android's text-only lane; cloud lane
  keeps original parts per the Xberg pipeline doc).
- Limits remain: 25 MiB/file, 6 attachments, 512 KiB extracted text into
  context; hitting the text cap shows "Only the first part of this document
  fits the context" on the chip.

### 9.2 Attachment blocks in the thread

Sent attachments render as `artifact` blocks (existing
`ContentBlockDocumentPreview` path) with extraction metadata on disclosure:
extracted length, OCR used (yes/no), and warning when extraction partially
failed. The block header action copies the extracted text.

### 9.3 Conversation documents panel (new)

Plan §5: uploaded documents are conversation-scoped by default. A
header-level "Documents" sheet per conversation lists:

- Each ingested document: name, mime, size, chunk count, extraction state,
  and ingestion time.
- Scope marker: "Stored with this conversation — deleted when it is
  deleted."
- **Promote to Knowledge** action per document: moves chunks to durable
  knowledge with provenance intact; confirmation explains the scope change
  ("Available to all future conversations; governed by Memory settings").
- Delete action per document (removes its chunks from this conversation's
  retrieval, with confirmation).
- Empty state: "Files you send in this conversation can be searched in
  follow-up questions."

### 9.4 Retrieval visibility

When retrieval ran for a turn, a one-line metadata row under the lifecycle
summary names it: "3 memories · 2 document chunks · 1 knowledge note".
Tapping opens the inspector's slice section (§10.3). This keeps RAG
transparent without flooding the thread (standard: "inspectable without
flooding the default view").

## 10. Context inspector (new)

One inspector surface unifies the Plan's context inspector and the design
standard's activity/event inspector (§7.5) — a single progressive-depth
sheet, not two debug models. Entry: telemetry chip on a turn, retrieval row,
backend chip, or the model banner.

### 10.1 Overview section

Run identity (run id, message, branch), final status, timings per phase,
backend/lane/diagnostics, model + profile name. This is the default landing
section.

### 10.2 Diagnostics section

`RuntimeDiagnostics` rendered readably: backend, device, offloaded layers,
fallback reason (full text), thermal state during the run when recorded, and
the certification reference when the backend is OpenCL.

### 10.3 Context section (the Plan's inspector)

From `ContextTelemetryEvent` + profile:

- **Profile row:** baked capability profile name (device/model/runtime),
  estimator in use ("exact tokenizer" / "chars÷4 estimate"), and drift
  readout when comparison data exists ("estimate ran 12% hot this turn").
- **Envelope:** trusted ceiling vs nominal context (§8.4), session envelope,
  output and safety reserves as fixed rows.
- **Slice spending:** one row per slice — history, memory, knowledge, tools —
  with budget / spent / item count and a background-fill meter (Flat 2.0: a
  fill, not a bordered bar). The memory:knowledge split is labeled "30/70
  fixed" and is not user-editable (§13.4).
- **Dropped items:** list of dropped messages/chunks/tools with the reason
  (over budget, diversity dedup, recency decay, shortlist cap) — each row
  restorable for inspection ("what was dropped") but never re-injected from
  the UI.
- **Degradation badge:** present when the run degraded (e.g. "Summarization
  unavailable → sliding window", "Thermal denial → reduced envelope"); badge
  text explains the visible consequence.
- **Thinking-cost hint:** when thinking is enabled on a lane that supports
  it, the slice view shows the thinking reserve visibly reducing knowledge
  chunks (CTX §7.3), e.g. "Thinking reserve: 1,024 tokens (≈2 knowledge
  chunks)".

### 10.4 Prompt plan view

A read-only, syntax-grouped rendering of the `PromptPlan` mirror: system,
history window, memory/knowledge slices, tool schemas (shortlisted count
shown: "4 of 11 tools offered"), and the exact exclusion markers — prior
thinking shown struck-through-with-label "excluded from context". This is
the sovereignty surface (CTX §11.2): the user can see precisely what will
be sent. Copy action included; no editing here.

### 10.5 Escalation card

When `EscalationAdvice` is present: a distinct card ("This device is
struggling with this request") with the advice reason (thermal, envelope
downgrade), the suggested action ("Continue on My server" / "Use a cloud
preset"), and a one-tap handoff that resubmits the turn on the advised lane.
No automatic lane switching ever happens without this explicit action.

### 10.6 Events section

The standard's progressive-depth timeline: filter chips by family
(lifecycle, content, context, tools, memory, telemetry), rows with sequence
numbers, timestamps, and sanitized payload previews; unknown payloads render
as the `ArtifactBlock` raw records they are. Replay action rebuilds the
materialized thread from the append-only record without contacting the
model. Payloads are sanitized — secrets and raw base64 blobs are elided.

## 11. Conversation library (extended)

The existing sheet gains the standard's required states:

- Row metadata: title, relative time, message count, and **state badges**:
  running (phase label), failed (error icon + tint), branched (branch icon),
  local-only vs synced (privacy marker), per standard §7.8.
- Running conversations keep running when the sheet or another conversation
  is opened; selecting a conversation cancels only view subscriptions.
- Draft preservation: unsent composer text (and staged attachments) restore
  per conversation.
- Search across titles and message text; recency groups Today / Previous 7
  days / Older; row actions (rename, pin, archive, export, delete) in a
  context menu with destructive confirmation on delete ("Deletes this
  conversation, its branches, and its conversation-scoped documents.
  Promoted knowledge is kept.").

## 12. Startup and lazy initialization (extended)

The Plan decouples startup: only FFI, the entity database, the SurrealDB
schema, and compatibility metadata open eagerly; FastEmbed, seeds,
Xberg/OCR, downloads, and inference initialize lazily with named status.

- `StartupGate` keeps its current eager phase progress (migrations, schema)
  but the eager path must reach the usable shell within the 5-second cold-
  start gate; everything lazy moves out of the blocking sequence.
- A **lazy-init status surface** replaces blocking spinners for deferred
  subsystems: a subtle header chip (or Home instrument row) listing
  deferred components with named states — "Document reader: starting" /
  "Embeddings: ready" / "Model: not installed". Each resolves independently;
  failures surface per-component with a retry, never as an app-level boot
  error.
- Feature-level gating: invoking a feature whose dependency is still lazy
  (e.g. sending an attachment before Xberg is ready) shows the named
  preparation state inline ("Preparing document reader…") and completes
  without user intervention; it never dead-ends.
- Boot errors remain blocking and visible only for the eager set
  (migrations/schema/FFI), per the existing `_BootError`.

## 13. Settings and configuration schema

Settings becomes a real destination (`features/settings`). Every row: label,
current value, and — where the value came from a layered override — a
"source" hint (default / global / assistant / conversation). Tightening-only
applies to all budget controls (§2.8).

### 13.1 Appearance (existing)

Theme mode (system/light/dark). No change.

### 13.2 Model & Inference

- Active model (deep-links to Models screen; read-only summary row).
- **Execution lane:** On-device / My server (BossFang URL + status) / Cloud
  (preset picker from baked, versioned presets + BYOK credential entry via
  secure storage). Lane choice is per-conversation overridable from the chat
  header; the settings row shows the global default.
- **Sampling (read-only display):** temperature 0.7, top-p 0.8, top-k 20,
  min-p 0, presence penalty 0 for the pinned Qwen lane, labeled "Tuned for
  the on-device model". Cloud presets carry their own baked values.
- **Thinking/reasoning toggle:** visible only on lanes that support it
  (iOS local, reasoning-capable cloud presets); Android local shows the
  disabled row with the non-thinking explanation (§6.2). Enabling shows the
  context cost hint ("reserves ≈1,024 tokens of your context").

### 13.3 Context & Memory

- **History strategy:** None / Sliding window / Truncate middle / Summarize /
  Hierarchical (CTX §8.1), with the per-strategy fields from CTX §4.2
  (maxMessages; keepFirst 2 / keepLast 4 + omission marker; summarization
  model + 500-token cap + threshold 6; short-term 5 turns / mid 2,000 /
  long-term facts 500). Fields appear only for the active strategy.
- **Layered overrides:** global default here; per-assistant override in the
  assistant's configuration; per-conversation override in the conversation's
  detail sheet. Each level edits only its own layer; effective value is
  shown computed with its source.
- **Truncate-middle transcript marker:** when active, the thread renders the
  omission marker as a metadata row "[… 12 earlier messages omitted …]"
  (Divider-adjacent presentation: spacing + faint text, never a line).
- **Fact extraction (opt-in per assistant):** an assistant list with a
  toggle per assistant, default off. Enrolling shows what it does ("Saves
  durable facts from this assistant's answers; visible in Memory; deletion
  respected"). A "Review extracted facts" link opens Memory filtered to
  facts with provenance (conversation / message / run), each deletable
  (CTX §8.5 sovereignty requirement).
- **Memory behavior:** read-only summary of what enters future context
  (user content, assistant answers, tool outcomes, governed memory — never
  prior reasoning).

### 13.4 Budget & performance

- **Trusted ceiling:** default 8K; the 16K extended ceiling is visible only
  with Developer mode on (decision inherited from CTX §15.5 — §17). The
  control only tightens below the profile ceiling (§2.8); copy explains the
  trade.
- **Memory/knowledge split:** displayed as fixed "30 / 70" — not a user
  control (Plan fixes it).
- **Estimator:** read-only current estimator + drift summary; comparison
  mode is a Developer tool (§13.6).
- **Thermal policy display:** "Heavy work pauses when the device is hot"
  informational row, linked to escalation behavior (§10.5).

### 13.5 Attachments & Documents

- Default scope explainer ("Files stay with their conversation unless you
  promote them"), promotion review link, and per-file/OCR limits listed
  (25 MiB, 6 files, 512 KiB extracted).

### 13.6 Developer (behind a Developer-mode toggle)

- Extended 16K ceiling switch (§13.4).
- Benchmark runner + last certification artifact viewer (correctness,
  thermal, median improvement, verdict) — the same data the Models screen
  summarizes.
- Estimator comparison readouts; telemetry ring viewer (bounded local ring,
  newest-first).
- Protocol/event console: the inspector's Events section without a run
  context.
- Copy of build pins: Flutter 3.45.0-0.1.pre, Dart 3.13.0, Rust 1.96,
  FRB 2.12, Android arm64 / API 29.

### 13.7 Privacy & Sync

Unchanged scope (existing settings intent): what crosses the wire, instance
roles, per-entity sync defaults. This spec adds only the lane/privacy marker
consistency requirement (§6.7, §11).

## 14. Accessibility and motion requirements (per surface)

Global floor: WCAG 2.2 AA as itemized in §2.10. Per-surface specifics:

- **Streaming text/thinking:** polite `liveRegion` semantics; no per-token
  announcements — announce phase changes and completion, not deltas.
- **Thinking/panels:** expanded state programmatic (`Semantics(expanded:)`);
  disclosure controls labeled; focus order: panels → answer → actions.
- **Lifecycle strip:** phase transitions announced; Stop reachable by
  keyboard at all times during a run; Escape cancels when focus is in the
  thread.
- **Confirmations (tool/delete/promote):** alert semantics, safe default
  focused first, destructive action last in order, Escape = safe choice.
- **Branch navigator:** `Semantics(button, label: 'Response 2 of 3')`;
  switch announced.
- **Charts/meters in the inspector:** meter fills are decorative; values
  always paired with text (never color-only), slices readable top-to-bottom
  by screen reader.
- **Targets:** 44×44 for all touch actions (existing 48 px constraints meet
  this); inspector chips ≥44 px.
- **Text scaling:** all layouts verified at 130% and 200% scaler without
  overflow; bubbles use max-width + flexible wrapping; the inspector tables
  collapse to stacked rows at ≥130%.
- **Reduced motion:** auto-scroll becomes instant (exists); strip
  transitions, panel expansions, and sheet presentations become
  opacity-only or instant; no parallax or physics animations anywhere in
  these surfaces.
- **Themes:** every surface specified here renders in both palettes from
  `KnowMeTheme`; cyan/ember/status tints verified against resolved
  foreground pairs, not token names.

## 15. Golden and behavior test matrix

Per `flutter-golden-ui`: real `KnowMeTheme` wrapper, deterministic pumps
(fonts loaded, animations off, fixed sizes, settled), one golden per
meaningful state × light/dark, `goldens/<widget>_<light|dark>.png`. Alchemist
is the configured harness.

**Sizes:** 320 px and 768 px widths; **text scale:** 1.0 and 2.0 on selected
rows; **states** as listed per widget.

| Widget/surface | Golden states (×2 themes) |
|---|---|
| Thinking region | streaming-expanded, auto-collapsed, manual-expanded, terminal-with-duration |
| Lifecycle strip | each representative phase (retrieval, OCR, tool selection, generation), stopped-cancelled |
| Error bubble | retryable with phase chip, non-retryable with deep-link, crash-restored |
| Cancelled bubble | standard, restored-partial |
| Tool confirmation card | pending confirm, confirmed-running, failed, restored-unresumable |
| Citation chip + card | single, multi-source, memory-citation |
| Answer bubble | markdown+code, mermaid, narrow-320 at 2.0× text |
| Action row | full, disabled-during-active-run |
| Branch navigator | 1/2, 2/3, with fork metadata row |
| Model banner | download 40% resumable, verifying, loading, failed-checksum, cancelled-resume |
| Models screen | empty, installed-CPU, installed-OpenCL-certified, benchmark-running, remote-marked |
| Documents panel | empty, scoped list, promoted item, extraction-failed |
| Inspector | overview, slices with drops+degradation, prompt plan with exclusions, escalation card |
| Conversation library | badges (running/failed/branched), groups, search results |
| Startup | eager phases, boot error, lazy-status chip states |
| Settings sections | each section both themes; developer section |

**Behavior tests (widget-level, 3–5 per feature at the public surface):**
thinking auto-collapse on answer start (and manual-override precedence);
active-run disabling of Resubmit/Delete/Regenerate; branch navigator
switching swaps tails; banner cancel → cancelled state; confirmation
approve/deny interaction payloads; attachment chip failure removes-or-blocks
send; extraction-state transitions; tightening-only budget control rejects
increases; live-region announcements fire on phase change only.

## 16. Acceptance mapping to the Plan's gates

| Plan gate | Spec evidence |
|---|---|
| Lifecycle UI final at stage 2 (no later generic spinners) | §6.4 strip + §8.2 banner defined as stage-2 deliverables; §15 goldens |
| Turn anatomy: panels outside, 80% bubble, adjacent citations, persisted terminal bubbles | §6.1–§6.5 |
| Thinking: selectable, whitespace-preserved, streaming-expanded, auto-collapse, manual precedence | §6.2 + behavior tests §15 |
| Actions incl. branch-scoped disable; Resubmit/Delete/Regenerate semantics | §7 |
| Xberg before text-only model; extraction visibility; conversation-scoped docs + promotion | §9 |
| Context inspector (profile, estimator, slices, drops, degradation, escalation) | §10 |
| Settings: strategy layers, fact-extraction opt-in, dev flag, cloud presets | §13 |
| Backend honesty: OpenCL ≥15% gate visible; CPU fallback reason; diagnostics | §6.7, §8.1–§8.2, §10.2 |
| Startup decoupling with named lazy status; 5 s cold start | §12 |
| Android non-thinking honesty | §6.2, §13.2 |
| Goldens: both themes, text scaling, narrow, a11y semantics, focus, reduced motion, 44 px | §14, §15 |
| Crash restoration presentation (exact order/status/diagnostics/branch) | §5, §6.5, §7.2 |

## 17. Open questions (decisions required before implementation)

1. **Destination count** (§4.2): fold Models into Settings (recommended) or
   accept a 6-destination deviation.
2. **16K extended context in v1 UI** (inherited CTX §15.5): this spec
   assumes Developer-flag-only; confirm.
3. **Fact-extraction default** (inherited CTX §15.2): this spec assumes
   per-assistant opt-in, off by default; confirm against a global opt-in.
4. **Citation adjacency migration** (§6.3): confirm the presentation-only
   re-flow (panel → adjacent chip at finalization) versus keeping citations
   in the panel region permanently.
5. **Mermaid renderer:** current `flutter_mermaid` (WebView-class) is kept;
   pure-Dart rendering remains unviable upstream. Confirm continued WebView
   dependency with the existing source/accessible fallback.
6. **Benchmark runner placement:** Models screen action (this spec) vs
   Developer-only; exposing it to all users makes the OpenCL gate
   self-serve but invites thermal test abuse. Leaning: keep in Models with
   a cooldown + explanation.

— End of PAGS-SPEC-KM-UIX-001 —
