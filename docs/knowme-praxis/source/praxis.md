## 00 | premise | KnowMe Praxis

**Intent should survive the conversation.** A workspace earns its place when the user's question, the work it produces, and the authority to act remain connected. KnowMe Praxis is the proposed product and architecture name for that workspace. The name refers to knowledge put into practice. It is an editorial name, not a claim of trademark clearance.

The original paper identified three durable responsibilities: selecting work, directing it, and keeping its result. Praxis retains that distinction and removes the assumption that one fixed arrangement of three panels can serve every task. Home, a focused conversation, a document, a small application, and a live call are states of one workspace. Their data and authorization contracts remain stable as the layout changes.

The product has a native Flutter mobile application, a Tauri/React desktop shell, and a browser shell. Rust owns domain behavior, context construction, persistence access, policy decisions, and agent coordination behind platform adapters. Optional private dots add either personal assistance or project coaching. Plugins contribute capabilities and declarative surfaces. LiveKit carries realtime human and agent audio/video. Durable actions and receipts remain in the application control plane.

**This is an engineering specification and proposed integration design.** It is not a release announcement. “Code observed” means files and implementations were inspected in the checkout. “Source requirement” means an existing specification requires or proposes it. “Proposed” means this paper adds or reconciles a design. “Certified” requires a named execution receipt against a specific build and device; this document does not manufacture one.

The complete technical source ledger is part of the specification. Each retained source section is assigned to an owning chapter, given a stable clause identifier, and linked from the coverage matrix. Existing source text remains available at full depth. Explicit amendments in §02 govern contradictions. Historical vendor observations and commercial assumptions retain their source date rather than becoming current product facts.

### Revision history

- **1.1 · 20 October 2026:** Adds WEB-01–14 and screenshot/social metadata publication contract; retains 17 chapters, 45 source records and 811 mappings.
- **1.0 · 20 October 2026:** Initial complete published specification.

## 01 | workspace | The workspace has three responsibilities

The navigator identifies the current work container. The conversation expresses intent, records progress and decisions, and exposes the actions taken. The canvas holds durable artifacts and applications. A tool rail provides local operations on the active artifact. These responsibilities survive on phones even when only one surface is visible at a time. Short answers remain in the thread; a canvas opens when there is something worth keeping or operating.

**WS-01 — Container before form.** A project, engagement, objective, conversation, or personal workspace exists from the first meaningful act of work. Creating a draft container must not require every field of a downstream business form. A title can be provisional; identity and provenance cannot. Selecting another container changes the visible context, thread list, dashboard query, artifact scope, and suggested actions together.

**WS-02 — Durable record.** Every domain mutation and governed external effect links to a thread or activity record with actor, subject, time, command identity, affected entity versions, and outcome. A stream of model text is not the evidence that a file was saved or an effect completed. Preserve raw event envelopes and materialized blocks separately so the timeline can be rebuilt without another model call.

**WS-03 — Layout as a mode.** Desktop supports conversation-forward and application-forward arrangements. In the former, navigator/thread/canvas reproduces the original paper's pattern. In the latter, navigator/application/assistant reproduces the San Saba shell. The active work determines emphasis; switching modes does not move records to a different store. Canvas states are closed, docked, expanded, and full-width. Resizing is keyboard accessible and persisted per device, with safe minimum widths.

**WS-04 — Artifact lifecycle.** Preserve requested, generating, published, previewing, iterating, and exported states from the original paper. Add explicit draft version, selected version, current version, signed version where relevant, failure, and uncertain publication outcome. “Published” means a publication receipt exists. “Signed” means the authoritative signature service confirms the exact artifact digest. Neither is inferred from model language.

**WS-05 — Human decisions.** Sending, signing, paying, publishing, and representing a person cross action boundaries. The thread shows the prepared object, destination, scope, costs where relevant, and the named authority. Existing valid authorization may satisfy the gate within its scope. The agent cannot approve itself, manufacture consent, or treat a generic “continue” as permission for a different destination or object. The approval is bound to the effect digest and expires or invalidates when material fields change.

**WS-06 — Visible context.** Show selected container, attached files, chosen artifact version, model placement, memory scopes, plugin contributions, and delegated agent identity. Each removable context item has a provenance link. Removing it changes subsequent context construction and does not claim to erase plaintext previously sent to a recipient.

**WS-07 — Mobile continuity.** On a phone, the navigator becomes a drawer or route and the canvas becomes a full-screen destination. Back returns to the exact thread position and composer draft. A persistent call strip survives internal navigation. No three-column miniature is permitted. Tablet split views are an enhancement over the same navigation state.

The original mineral-rights example remains useful as a technical pattern: one deal container, a running thread, a checklist, versioned documents, units and valuation inputs, and an existing app embedded in the canvas. A confirmed workspace value may populate a legacy form without saving it. Preserve that explicit save boundary. The generalized KnowMe counterpart is an engagement linked to objectives, decisions, documents, and a plugin surface, all reading the same durable records.


# First-class WebMCP for Praxis: browser-backed site capabilities

Primary-source research, 20 October 2026, integrated into edition 1.1. This is an author-ready design, not implementation or deployment evidence. MUST/SHOULD clauses below are proposed Praxis requirements, not claims that upstream already supplies these guarantees. The existing source corpus remains authoritative under EV-01; this report adds a distinct site capability contract.

## Verified upstream baseline and research pins

The [current WebMCP draft](https://webmachinelearning.github.io/webmcp/) is a Draft Community Group Report dated 20 October 2026, not a W3C Standard or Standards Track document. The reference secure-context surface is `document.modelContext`, with asynchronous `registerTool`, `getTools` and `executeTool`. Registration and execution have distinct abort signals; removing a tool does not terminate an already running callback. The current events include `toolchange`, `toolactivated` and `toolcancel`. Discovery carries originating window/origin; the in-page API is confined to one top-level traversable rather than arbitrary tabs. Browser-agent observation is a separate implementation-defined facility. The `tools` Permissions Policy defaults to self. The draft explicitly identifies a same-name remove/re-register race. Results are JSON-serializable callback values, returned serialized; named failures include `NotFoundError`, `DataError`, `OperationError` and `AbortError`, with additional inactive/security failures.

Research pin: official WebMCP `main` resolved by the parent through the GitHub API to [`cb3d16964ab3e92cfd5f09901ed0504373a906e8`](https://github.com/webmachinelearning/webmcp/tree/cb3d16964ab3e92cfd5f09901ed0504373a906e8), committed 20 October 2026 09:53:18 UTC. This records the research snapshot; release fixtures must independently verify and pin the implemented revision.

[Chrome's overview](https://developer.chrome.com/docs/ai/webmcp) describes an origin trial beginning with Chrome 149 and `chrome://flags/#enable-webmcp-testing` for development. Visiting the site and visible human collaboration are central; cross-origin iframe registration uses `allow="tools"` subject to ancestors' policy. Trial and flag state are not authorization. [The imperative guide](https://developer.chrome.com/docs/ai/webmcp/imperative-api) separates provider `exposedTo`, consumer `fromOrigins` and Permissions Policy. It documents object inputs and JSON-string argument deprecation from Chrome 155, continued execution after removal from 153, and `debugging` annotation from 156. Its null-on-navigation description differs from the newer draft's unload-abort model: certify each adapter's observed behavior instead of interpreting either as effect success.

The [official implementation inventory](https://github.com/webmachinelearning/webmcp/blob/main/implementation-status.md) lists Chrome/Edge trials and Brave experimental work; it does not certify Praxis. [Mozilla's position](https://github.com/mozilla/standards-positions/issues/1412) is neutral; [WebKit's issue](https://github.com/WebKit/standards-positions/issues/670) records opposition. Do not promise Safari/WKWebView or universal parity. The [Chrome Status record](https://chromestatus.com/feature/5117755740913664) fetched by the parent has trial-stage milestones 149–156 and an extension-stage desktop end at 162. Stage history does not prove stable shipping, a token's validity, or support in a specific mobile engine.

### Declarative profile

[Chrome's declarative API](https://developer.chrome.com/docs/ai/webmcp/declarative-api) describes forms annotated by `toolname` and `tooldescription`; removing either unregisters the tool. User submission is the default; `toolautosubmit` permits invocation to submit. `SubmitEvent.agentInvoked`, `preventDefault()` and `respondWith()` support explicit asynchronous form results. Auto-submit remains site behavior, not user authority. Navigation-result extraction in the [declarative explainer](https://github.com/webmachinelearning/webmcp/blob/main/declarative-api-explainer.md) is a separate proposal/profile. The repository's normative specification currently focuses on imperative tools; certify declarative behavior independently.

### Experimental remote browser profile

The [tip-of-tree CDP WebMCP schema](https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/master/pdl/domains/WebMCP.pdl) defines an experimental domain: `enable`, initial/existing `toolsAdded`, frame-scoped `invokeTool`, invocation IDs, `cancelInvocation`, catalog/removal/invocation/response events and Completed/Canceled/Error responses. Descriptors carry frame IDs and optional declarative DOM node references; returned content is untrusted. This is a debugging-protocol adapter, not the standard JavaScript API or an MCP transport. Probe the actual endpoint rather than assuming tip-of-tree exists.

Research pin: official devtools-protocol `master` resolved by the parent to [`95b1af69d6a20161dc3558f8e38366b7fa7a27d1`](https://github.com/ChromeDevTools/devtools-protocol/tree/95b1af69d6a20161dc3558f8e38366b7fa7a27d1), committed 19 October 2026 04:39:30 UTC. [CDP documentation](https://chromedevtools.github.io/devtools-protocol/index.html) warns that tip-of-tree evolves; the target browser protocol is the deployment evidence.

### Capabilities excluded from the baseline

Older `navigator.modelContext`, `provideContext`, explicit `unregisterTool` and `ModelContextClient.requestUserInteraction` examples are not the current reference API. The [elicitation proposal](https://github.com/webmachinelearning/webmcp/pull/204) replacing earlier interaction with `requestUserInput` is open; the baseline must not depend on it. [Schema-validation ownership](https://github.com/webmachinelearning/webmcp/issues/92) remains open: neither full JSON Schema enforcement nor a universal output schema can be assumed. MCP resources, prompts, sampling, subscriptions and well-known server discovery are not implicit WebMCP capabilities. Third-party shims and proposal issues do not establish native support.

## One broker, distinct adapters

WebMCP exposes functions/forms owned by a live document. Ordinary [MCP transports](https://modelcontextprotocol.io/specification/latest/basic/transports) connect a separately identified and authorized server through stdio or Streamable HTTP. A server bridge may relay browser tools only while retaining their session provenance. [MCP Apps](https://modelcontextprotocol.io/extensions/apps/overview) provides server-supplied UI resources, a host sandbox and postMessage protocol; it does not discover arbitrary visited-site tools. A participating MCP App can use WebMCP only if both boundaries are explicitly supported.

Browser automation through DOM, accessibility, pixels and input remains a separate capability and policy surface. Extensions are an adapter placement, not a universal protocol: [Chrome's agent-security guidance](https://developer.chrome.com/docs/agents/security) requires page host permission. A2UI is native declarative presentation, not permission to execute website output as HTML. The common abstraction is a governed action candidate; the adapters retain different identity, access and lifecycle semantics.

## Proposed normative Praxis requirements

### WEB-01 — First-class capability source

The host MUST represent `source_kind=webmcp` separately from installed packages, server MCP and synthesized UI tools. A site registration MUST NOT install a plugin or inherit its grant. A reviewed plugin MAY provide an adapter or site policy; live tools retain website provenance. Prefer an admitted site tool over brittle reconstruction when it meets the same authorized intent and evidence requirements. The historical S42 `webmcp.relay` enum does not implement this contract.

### WEB-02 — Browser-session binding

Every admitted session MUST bind to container, Activity, placement, authenticated host actor and browser-profile partition. Opaque session, target, frame and document-generation handles combine with provider origin, tool name and descriptor revision/digest. Names are not global IDs. Page title, favicon, page-supplied URL or claimed user ID MUST NOT determine authority.

### WEB-03 — Actual capabilities

Proposed `GET /api/v1/host` MUST advertise tested adapter profiles and bounds. Per-session probing MUST distinguish absent API, policy-disabled API, unsupported version/method, zero tools, unapproved origin, unready document and disconnected placement. Record engine/build, adapter, fixture revision, declarative support, cancellation, takeover and trial dependency. Unsupported is a normal explained state; a WebView widget, iframe or Chromium brand is not proof.

### WEB-04 — Bounded discovery

Discovery MUST stay within authorized targets and origins, never inventory all tabs into model context. Maintain bounded initial catalogs, `toolchange` or adapter notifications, revisioned snapshots and resynchronization after event gaps. Bound descriptions/schema depth and size; treat annotations as untrusted. Do not fetch arbitrary schema `$ref` destinations. The in-page profile cannot discover unrelated tabs.

### WEB-05 — Navigation and lifecycle

Cross-document navigation, redirect/origin change, frame removal, target close, profile switch, suspension and disconnect MUST invalidate affected handles and stop new calls. Same-document routes require application-context/catalog refresh without inventing a new Document. Revalidate BFCache restoration. Durable Activities survive session loss; stored handles do not survive as executable authority. Availability and invocation lifetime are separate: removal does not prove callback termination. Abort/cancellation is best effort and never rollback.

### WEB-06 — Origin and frame policy

Distinguish top-level, provider-frame and calling-agent origins. In-page adapters MUST obey the frame tree, `exposedTo`, `fromOrigins` and applicable `tools` Permissions Policy. Privileged extension/CDP adapters MUST still enforce a host allowlist. Do not replace a denied call with injection or a relay, weaken TLS/CSP/framing/sandbox controls, or upgrade an opaque-origin frame. Displayability, site exposure and owner execution authority are separate facts. IPFS path-gateway CIDs share an origin; prefer a separately verified CID subdomain for privileged active integration.

### WEB-07 — Existing authority path

Every invocation MUST pass through the existing container command/decision/effect service. Bind tool identity, descriptor revision, canonical input digest, actor, container, site audience, policy revision and required decision; validate task scope and egress before dispatch. Material target/argument/account/origin/destination changes invalidate dependent approval. Honor valid prior authorization within scope. Read-only/consequential hints, absent annotations and names such as `get` never decide policy.

### WEB-08 — Meaning and registration races

A descriptor digest proves observed metadata freshness, not JavaScript behavior. Disclose inability to bind atomically to an immutable registration. Recheck immediately before dispatch; reject observed mismatches. The draft's same-name remove/re-register race remains material even when metadata matches. Use conservative admission for consequential tools on unreviewed sites. Verify high-impact effects by reviewed provider contracts, authoritative postconditions or explicit human completion, not success prose.

### WEB-09 — Typed inputs and honest outputs

Validate a declared accepted schema profile before admission and invocation. Unsupported dialects/constructs produce typed unsupported-schema results, not silent relaxation. Preserve bounded inert raw output and normalize using the negotiated adapter encoding. Distinguish serialized JSON, primitive strings, decoded objects, MCP content arrays and markup; do not double decode or execute output. Transport, platform, site error, validation, approval denial, cancellation and uncertain effect are distinct outcomes. Current draft errors include NotFoundError, DataError, OperationError and AbortError; preserve original exception plus host category. Callback return proves a return only.

### WEB-10 — Human interaction and takeover

The canvas MUST display the receiving browser, placement, origin/frame, observable account/auth status, tool availability/freshness, pending review, cancellation and uncertainty. Preserve site manual-submit boundaries: never add `toolautosubmit`, manufacture consent or report submission after field population. Host approvals and site confirmations are distinct. Praxis `requestTakeover`/`resumeAfterTakeover` are proposed host semantics, not upstream methods. Takeover pauses competing input; resume revalidates document, account, tool and policy. Offer accessible stop, distinguish acknowledgement from final outcome, and preserve thread/draft/focus. An unreachable remote human surface is blocked interaction.

### WEB-11 — Credentials and egress

Keep credentials in the selected browser partition or approved authentication component. UAR, agents, models, plugin manifests and events MUST NOT receive exported cookies, bearer tokens or hidden secrets. Local/remote profiles are distinct identities; login rather than copying sessions. Even read-only inputs cross a destination boundary. Apply classification/minimization to each argument. Origin binding does not prove downstream JavaScript network confinement; such a claim needs separately enforced and tested network policy.

### WEB-12 — Injection containment

Names, descriptions, schemas, annotations and results MUST NOT grant privileges, select host endpoints, create approvals, authorize cross-site transfer or redefine acceptance. Namespace candidates so a hostile `send_email` cannot shadow a trusted connector. Policy stays outside the model; descriptors cannot become executable routes. Delimiters, screening and [attack evaluations](https://developer.chrome.com/docs/agents/security) add layers, not proof. [Site tool security](https://developer.chrome.com/docs/ai/webmcp/secure-tools) also requires website authorization/business invariants and bounded inputs; models alone cannot guarantee safety.

### WEB-13 — Durable evidence and safe reconciliation

Persist command before dispatch and correlate attempt/adapter invocation IDs. Record not-dispatched, dispatched or unknown separately from effect verification. Timeout, unload, disconnect and process loss can leave uncertainty. Host idempotency does not make arbitrary website code idempotent. Retry mutations only with provider deduplication, authoritative no-effect readback, or specifically authorized uncertain retry under policy. Keep necessary redacted evidence rather than perpetual page copies; private partitions stay separate from general retention/memory. Revocation stops future execution/model exposure while retaining permitted audit.

### WEB-14 — Explicit fallbacks

Offer an admitted server connector, separately authorized UI automation, a supported browser or manual completion. Disclose changed placement/session/evidence; never bypass browser/site denial or silently move private context to cloud. Label shims/relays as compatibility implementations and certify separately. An external browser without an authorized bridge is human handoff, not proven agentic integration.



## 02 | evidence | Evidence, precedence, and deliberate corrections

**EV-01 — Evidence hierarchy.** User decisions and accepted amendments govern product intent. Inspected code establishes implementation presence. Build and device receipts establish only the behavior actually exercised. Source specifications establish requirements and historical rationale. External primary documentation establishes upstream contracts at the access date. None of these substitutes for another.

| Source statement or conflict | Praxis resolution and rationale |
|---|---|
| The original product name and San Saba document branding | Superseded by KnowMe Praxis. Historical names remain only in source attribution and the migration record. |
| React/Tauri “one codebase” on all devices | Native Flutter is the mobile presentation layer; shared Rust behavior crosses FFI. Retain the original seven-layer separation, not the obsolete mobile UI implementation. |
| The plugin host is only a skeleton | Checkout contains `WasmtimePluginHost`, typed bindings, fuel/epoch/memory enforcement and host dispatch. Mark code observed, not certified. Installer/distribution completion must be proved separately. |
| `knowme:plugin@0.1.0`, `knowme:plugin@1.0.0`, `prometheus:component/*`, `uar:plugin@0.2.0` | These are distinct source dialects. Current local WIT uses KnowMe 1.0.0; the strategy assigns future generic ownership to UAR. Pin each ABI and require a tested adapter; do not rename a package string and call it migration. |
| One signed Wasm binary runs everywhere | Superseded by capability portability plus explicit execution profiles (§08). Native mobile admission, browser adaptation, and server execution each need their own conformance evidence. |
| Plugin UI can ship arbitrary HTML, iframe UI, or platform source | A2UI data is the portable baseline. Audited embedded/webview surfaces are separately declared exceptions. No downloaded React or Dart code enters the native component catalog. |
| `kind: core/agentic`, `/embed/*`, caller-built AG-UI subpaths | Retired by the San Saba amendments: independent navigation/protection/surface fields, registry-resolved embed origins, registered agent IDs. Retained as migration history only. |
| San Saba authentication is its app authorization boundary | Preserved for that application. It does not remove KnowMe personal-context grants, representation grants, plugin capability enforcement, or tenant isolation. |
| Unknown capability actions need no schema validation | Superseded: all action inputs and outputs are schema validated, recognized namespace or not. Semantic discovery never grants authority. |
| `did:key` proves a reverse-DNS namespace | Superseded: it proves key control. Namespace ownership needs independent proof or a trusted registry decision. |
| Policy checked only at activation/compile time | Activation may precompute policy structures; every action re-evaluates effective authority, expiry, revocation, scope and budget at execution time. |
| A2UI attributed to Anthropic or confused with internal `A2uiEvent` | Google A2UI is the external declarative format. AG-UI is the event vocabulary. KnowMe ContentBlock and internal envelope types are adapters, not alternate names for the standard. |
| One shared memory across assistant and coach | Superseded: personal dot, general assistant, each project coach, and team contexts are separately scoped; no shared coach/general-assistant embedding index. |
| “Zero capability” imports clocks and randomness | Corrected to minimum capability: time and randomness are capabilities and must be declared. A no-import pure evaluator is a stricter profile. |
| Fixed context presets always fit | The budget invariant governs. Some source presets and thinking reserves cannot simply be added. Recompute available slices and validate the sum before dispatch. |
| Browser UAR, OFP transport, Hands scheduling, or local inference described as universal | Retain as source intentions with placement-specific proof gates. The native UAR library and browser facade are not evidence of a complete browser agent kernel. |
| Private-dot infrastructure and protocol versions presented as deployed | Preserve as design requirements unless a current implementation receipt establishes them. Model names, runtime versions, pricing, latency and policy assertions in historical sources are dated observations or targets. |
| Vendor examples imply industry-wide convergence or unique invention | Retain the field-guide observations and sources. The synthesis is this design's architectural choice; novelty and all-vendor claims are not established by the evidence. |

**Code observed on 20 October 2026:** the Rust workspace declares 22 member crates. `gen_ui_ffi` exposes chat send/cancel/actions, tool confirmation, model operations, entity CRUD, sync/status streams, provider configuration, memory ingest/search/deletion, and graph expansion. The plugin host depends on Wasmtime 46 and supports hook, contributor, and app-shell typed worlds. Its older top-level comments and `NotImplemented` variants coexist with implementation files. Hands and OFP domain crates still contain explicit incomplete-path markers. These findings justify a mixed status table, not a blanket “implemented” label.

The snapshot inspection did not run Rust builds, device installations, app releases, or the unrelated UAR workstream. Public publication of this document proves document delivery only.

## 03 | home | A home screen that earns its space

**HOME-01 — Four regions, one context.** Home combines a prompt composer, dashboard cards, prompt suggestions, and A2UI mini applications. The visible container selector and placement indicator govern all four. Empty home centers the composer with a short invitation and three to six suggestions. An established workspace places the composer at the bottom of the primary work region, leaving the dashboard readable above it. Avoid duplicate composers competing for focus.

**HOME-02 — Dashboard cards.** Initial card types are Continue working, Needs your decision, Recent artifacts, Scheduled Hands, Active calls, and optional Private Dot. Each card shows its data freshness, owner or actor, relevant container, and an explicit action. Permission-filter the query before aggregation; a hidden private container must not leak through a badge count. Loading, empty, offline, stale, denied, and error are distinct states. A card never reports zero when a request failed.

**HOME-03 — Suggestions.** Suggestions are editable drafts, not executable commands. Derive them from the selected container's permitted state, installed capabilities, and explicit user preferences. Display why each suggestion appears and the data scopes it would use. Clicking inserts text into the composer or opens a preview with parameter fields. It does not send mail, start recording, invite participants, or activate a background agent. Support dismiss, pin, refresh, and disabling personalized suggestions. No private-dot memory contributes unless explicitly allowed for this surface.

**HOME-04 — Mini applications.** Home surfaces persist by `surface_id` and `container_id`. An A2UI surface can be a form, checklist, brief, table, tracker, settings panel, or call roster. A user can pin, resize within allowed spans, reorder, open full-screen, close, and restore it. Persist layout separately from business state: device-specific geometry, user-specific preferred cards, container-specific data. Live run endings do not destroy durable surfaces.

**HOME-05 — Adaptive layout.** At compact widths, order is context/title, critical decision card, suggestions, dashboard, mini applications, and a safe-area-aware composer. Above a measured layout breakpoint, cards form a two-column grid; wide desktop can show the navigator and conversation simultaneously. Text scale and content fit, not device names, decide breakpoints. The document's 860 px reading-layout breakpoint is not an application platform mandate.

**HOME-06 — Restore safely.** Restore the selected container, thread scroll position, draft, card order, and surfaces after process death. Revalidate action grants, surface revision, and current user scope before enabling controls. Unsaved local forms are labeled drafts. A revoked or removed plugin leaves an explanatory placeholder with export/recovery options; it does not leave an invisible data loss.

**HOME-07 — Attention budget.** Personal-dot proactivity is opt-in. A project coach is silent during healthy execution. Group low-priority notices; expose a single decision inbox with counts by urgency and source. Cards are not a place for simulated activity. Every “working” state links to an actual run or bounded pending command.

A static home specimen in this document demonstrates the proposed arrangement. It is a specification illustration, with synthetic data and no live agent connection.

## 04 | containers | One container substrate and one durable state plane

**CT-01 — Domain model.** Keep one host-side container substrate with typed payloads. `Engagement` and `Objective` share identity, namespace, provenance, staleness clock, gates, delegation links and audit rules. Goal horizon rollups remain objective-specific. Add workspace/project presentation types without duplicating the gate engine. Cross-kind edges represent real dependencies: an objective milestone can depend on an engagement decision in the same governed graph.

```text
Container(id, kind, namespace, classification, title, revision,
          parent_id?, payload_schema, payload, created_at, updated_at)
ContainerEdge(id, from_id, relation, to_id, provenance, revision)
Thread(id, container_id, participants, context_policy, revision)
Artifact(id, container_id, kind, versions[], current_version, provenance)
Surface(id, container_id, plugin_id?, catalog_profile, revision, bindings)
Decision(id, container_id, effect_digest, authority, status, expires_at)
Command(id, scope, actor, subject, expected_revision, state, receipt?)
```

**CT-02 — Identity and isolation.** Authorization derives from authenticated claims or verified local identity. Never accept tenant or user identity from an untrusted body as authority. Scope storage, subscriptions, cache entries, embeddings and command replay by user/tenant/namespace. Policy evaluates both container kind and namespace. If classification or retention demands physical separation, split the store while retaining the shared substrate and gate logic.

**CT-03 — Projection.** Normalize entities by `(type,id)` inside the active authorized scope. Lists hold IDs. Human edits, agent results, embedded-app bridge changes and sync events feed one projection; components do not maintain competing copies. React uses PEM for entity/server state and Zustand only for transient interaction. Flutter uses typed repositories and Riverpod projections over Rust-owned records. The server or local transaction boundary owns authoritative writes.

**CT-04 — Three persistence lanes.** Keep authoritative read replication, durable mutation intent, and collaborative CRDT state separate. Read shapes cannot widen beyond the server-selected scope. Mutations enter an outbox in the same transaction as optimistic projection, with stable command IDs. Loro/CRDT merges may handle explicitly eligible collaborative content and preferences; grants, releases, approvals, policy changes and financial authority are never ordinary CRDT values.

**CT-05 — Offline writes and uncertain outcomes.** New submission gets a new identity; an IPC/network retry reuses the original identity. Persist an attempt before network I/O. A lost reply is uncertain, not failed. Recover by querying or replaying the identical request against a provider receipt. Acknowledgment and local settlement commit together; later edits reapply in order. Conflicts and terminal rejection remain inspectable, while transient failures use bounded backoff. “Exactly once” is not a network guarantee; the target is deduplicated effects under a documented provider receipt contract.

**CT-06 — Revocation and resnapshot.** Scope changes suspend old command replay and clear previous readable projection without deleting command history. Revoked read grants remove disclosed rows and prevent overlay from repopulating them. Opaque cursor compare-and-set and row application are atomic. Resnapshot preserves tombstones and eligible pending intent. Each shape reports its own status so one healthy shape cannot hide another revoked grant.

**CT-07 — CRM and goals.** Retain engagement stage gates, health/staleness evaluation, source attribution, blocked decisions, goal extraction, horizon rollup and explicit human commitments. Stateless `container-gate`, `container-health`, `goal-rollup` and `goal-extract` modules operate on host-supplied typed payloads. They return evaluations and proposals; the host applies approved state changes. A shared gate evaluator does not merge personal objectives with client data without classification checks.

## 05 | api | Container and capability APIs

The following is a proposed versioned contract. Existing San Saba routes remain adapters until parity is certified. Do not advertise these routes as deployed merely because they appear here.

| Method and route | Semantics |
|---|---|
| `GET /api/v1/host` | Supported profiles, action schemas, catalogs, execution placements, limits and compatibility versions; no secret values. |
| `GET /api/v1/containers?cursor=…` | Authorized containers with stable cursor pagination and freshness. |
| `POST /api/v1/containers` | Create typed container from schema-validated input; idempotency key required. |
| `GET /api/v1/containers/{id}` | Snapshot with revision and ETag. |
| `PATCH /api/v1/containers/{id}` | Conditional typed mutation with `If-Match`; reject stale revisions with conflict details. |
| `POST /api/v1/containers/{id}/actions/{action}` | Invoke a registered action through authorization and approval gates. |
| `GET /api/v1/commands/{command_id}` | Durable outcome: queued, running, awaiting decision, committed, rejected, cancelled or uncertain. |
| `GET /api/v1/containers/{id}/events` | SSE events with resume cursor, scoped replay and explicit resnapshot instruction. |
| `POST /api/v1/surfaces/assemble` | Resolve installed catalog, producer and bindings; assemble durable surface state. |
| `POST /api/v1/surfaces/{id}/actions` | Semantic A2UI action with surface revision, component ID and command identity. |
| `POST /api/v1/runs` / `POST /api/v1/runs/{id}/cancel` | Start/cancel a bounded agent run under a pinned deployment binding. |
| `POST /api/v1/decisions/{id}/resolve` | Authenticated human decision bound to exact proposal/effect digest. |
| `POST /api/v1/calls` / `POST /api/v1/calls/{id}/join` | Create room policy, then issue short-lived participant credentials after current access checks. |

**API-01 — Envelope.** Commands contain `command_id`, `schema_version`, target reference, `expected_revision`, payload, and a client correlation ID. The host supplies authenticated subject, actor, effective grants, policy revision and timestamps. An idempotency key reused with a different canonical payload is a conflict. Paginated reads set a maximum page size; uploads have declared byte limits, media types and hashes.

```json
{
  "command_id": "<new-submission-uuid>",
  "schema_version": "knowme.command/1",
  "target": {"kind": "container", "id": "<container-id>"},
  "action": "commitment.reschedule",
  "expected_revision": 17,
  "payload": {"commitment_id": "<id>", "date": "<date-only>"}
}
```

**API-02 — Errors.** Use typed problems: invalid schema, unauthenticated, forbidden, approval required, stale revision, unsupported profile, quota exhausted, rate limited, unavailable placement, cancelled, and uncertain effect. Include correlation ID and safe remediation. Never return provider credentials, full prompts, or raw private memory in generic errors. A timeout after submission does not authorize a fresh command identity.

**API-03 — Event contract.** Persist `profile,event_id,sequence,thread_id,run_id,command_id?,message_id?,tool_call_id?`. Deduplicate event IDs, detect gaps, preserve unknown payloads in an inert artifact/inspector channel, and resume from a durable cursor. Keep raw envelopes separate from UI projection. Sequence scope is explicitly defined per stream; do not compare unrelated room/media sequences to domain revisions.

**API-04 — Protocol adapters.** AG-UI transports run lifecycle and state events; A2UI carries validated component/data surfaces; A2A covers delegated task lifecycle; MCP exposes tools and resources at integration edges. REST/FFI/IPC adapt to the same service methods. The WIT host interface is an in-process capability boundary, not an HTTP URL. A plugin cannot construct arbitrary agent routes or backend destinations.

**API-05 — Backpressure.** Bound events by count and byte size. Preserve semantic checkpoints and terminal outcomes; coalesce replaceable token deltas under pressure, with recorded coalescing metadata. Cancellation is idempotent and acknowledged. Subscription disposal releases native resources. A disconnected consumer must not leave an unbounded Rust channel growing behind a suspended Flutter view.


### API-07 — Browser site actions and evidence

## Proposed host contract

These are schema additions to the existing action and command paths, not deployed routes or upstream WebMCP interfaces.

```typescript
type WebAdapterProfile = 'in-page-webmcp' | 'extension-webmcp'
  | 'chromium-cdp-webmcp-experimental' | 'site-relay-compat';
type WebSessionRef = {
  sessionId: string; containerId: string; activityId: string;
  placementBindingId: string; browserProfilePartition: string;
  targetId: string; topLevelOrigin: string; adapterProfile: WebAdapterProfile;
  adapterVersion: string; capabilitySnapshotRevision: number;
};
type WebToolRef = {
  sessionId: string; targetId: string; frameId: string;
  documentGeneration: string; providerOrigin: string; name: string;
  descriptorRevision: number; descriptorDigest: string;
};
type WebToolCandidate = {
  commandId: string; tool: WebToolRef; input: Record<string, unknown>;
  inputDigest: string; expectedSessionRevision: number;
  decisionRef?: string; deadline: string;
  // Host resolves actor, grants, policy, scope and egress.
};
type WebInvocationEvidence = {
  commandId: string; attemptId: string; adapterInvocationId?: string;
  tool: WebToolRef; dispatchState: 'not_dispatched' | 'dispatched' | 'unknown';
  outcome: 'returned' | 'tool_error' | 'platform_error' | 'cancelled' | 'uncertain';
  effectVerification: 'not_required' | 'pending' | 'verified' | 'not_verified';
  resultEncoding?: 'json-text' | 'json-value' | 'legacy-profile-value';
  resultArtifactRef?: string; platformErrorName?: string;
  safeErrorCode?: string; receiptRef?: string;
};
```

| Surface | Proposed operation | Required behavior |
|---|---|---|
| Existing discovery | `GET /api/v1/host` | Tested profiles and nonsecret limits. |
| Container action | `web.session.attach` | Authorized session binding; no arbitrary debugging endpoint. |
| Authorized read | `GET /api/v1/containers/{id}/web-sessions` | Only the selected container's admitted sessions. |
| Authorized read | `GET /api/v1/web-sessions/{id}/tools?revision=…` | Bounded catalog or refresh requirement; no source callbacks/secrets. |
| Container action | `web.tool.invoke` | Validate, decide, persist, dispatch. |
| Container action | `web.invocation.cancel` | Idempotent request with separately observable outcome. |
| Container action | `web.session.takeover` / `web.session.resume` | Pause conflicts and revalidate after human action/login. |
| Container action | `web.session.detach` | Revoke handles/subscriptions without reversing effects. |
| Existing command read | `GET /api/v1/commands/{id}` | Durable command and effect evidence. |

Use existing container event envelopes with Activity/run/actor/command/correlation/causation and monotonic sequence scope. Add `web.session.attached/capabilities_changed/suspended/detached`, `web.document.changed`, `web.frame.removed`, `web.tools.snapshot/changed/resync_required`, `web.invocation.proposed/awaiting_decision/dispatched/started/result_observed/failed/cancel_requested/cancel_acknowledged/uncertain`, `web.interaction.required/taken_over/resumed` and `web.effect.verified/verification_failed`. A page cannot emit host-owned effect verification. Browser terminal events and business effect settlement remain separate. Gaps require snapshots/reconciliation; generation/correlation govern races rather than arrival order.



## 06 | dots | Optional private dots, with separate authority

**DOT-01 — Optional means absent is complete.** KnowMe works without any dot. Home has no fake empty companion, background memory indexing, microphone listener, or blocked workflow when dots are disabled. A user can enable a personal dot, attach a coach to a project, or do both independently. Disabling a dot stops new triggers, revokes relevant runtime grants, cancels or explicitly reports delegated work, and preserves inspectable history and export controls.

**DOT-02 — Personal dot.** It supports the user's own intentions and commitments, with on-device, editable, exportable memory. Outcomes are **Kept, Missed, Rescheduled, Dropped, Unknown**, tagged with evidence class: self-report, calendar or artifact. It never calls a personal commitment “verified.” Local inference is the default; an explicit per-conversation route choice governs off-device processing. Contacting a counselor, partner or other person is a typed effect requiring the user's recorded consent. There is no automatic third-party escalation.

**DOT-03 — Project coach.** This is a separate BossFang Hand observing a particular project under a signed `CoachingContract`. It has a per-project provenance store and no cross-project reads or embedding index shared with the general assistant. It observes read-only KBD predicates, current candidate receipts, repository differences and CI state. Executor transcripts are statements to inspect, not authoritative evidence of completion.

**DOT-04 — Authority classes.** The contract enumerates `pre_authorized`, `decision_required`, and `never`. Unlisted steering is decision-required. Preauthorization remains bounded by action, scope, candidate/contract revision, expiry, counters and stop condition. Repair attempts, no-progress count, coach interventions, executor credit budget, and coach credit budget are independent counters. No summing or transfer between them is allowed. An exhausted contract is amended by its owner or closed; reconciliation does not replenish it.

**DOT-05 — Triggers and proposals.** Preserve RepeatedIdenticalFailure, ConflictingTerminalClaims, MissingRequiredScenario, ScopeGrowth, BudgetInconsistency, StaleEvidence, MissingPhaseReceipt and OverclaimDetected. Healthy execution yields no coaching commentary. Only the semantic portion of scope growth/overclaim classification can consult a decision model. Each proposal contains one discrepancy, its receipts, `objective_unchanged=true`, exactly one next action, expected evidence, authority class and allowance consumption. Excluded successor work is rejected at schema validation.

**DOT-06 — Steering boundary.** A coach cannot run a shell, edit code, execute tests, spawn a coding harness or alter a ledger. It submits one bounded A2A task through UAR under the owner's subject and coach actor identity. UAR checks policy each time. Denial produces a named blocker; retrying through another channel, tool or worker is forbidden. Decision-required tasks resume only after an authenticated owner response bound to the proposal.

**DOT-07 — Evidence and closure.** KBD owns deterministic verdicts. Required receipts bind candidate digest, contract revision, check, signer and previous hash. A stale receipt cannot prove the current candidate. Coaching memory is derivative and reconstructable; the signed receipt chain and authoritative ledger remain the record. Reconcile an intervention once as resolved, diagnosis changed, or no progress. Unchanged evidence cannot generate the same proposal again. Terminal dispositions are VerifiedSuccess, Blocked, FailedAcceptance, ExhaustedAllowance, Cancelled and MissingEvidence. After disposition, stop; useful successor work is retained backlog.

**DOT-08 — Memory and metering.** Beliefs carry source, receipt ID, contract revision, observation time and freshness. Owner corrections create attributed revisions. Export includes readable wiki data plus structured database export. Every coach call/retrieval/intervention has an Assignment-bound usage event; coach cost is separate from executor cost and includes review/rework in intervention-yield accounting. No valid Assignment means no work-lane spend. Credential revocation stops future coach authority.

**DOT-09 — UX.** A dot panel shows identity, personal/project scope, deployment placement, current contract, next allowed trigger, budget, memory access and Stop. Personal-dot and coach colors/icons may differ, but labels—not color—carry the distinction. Switching projects closes coach context before opening another. Inviting a dot into a call shares only the selected room context; it does not silently share its private memory or all project artifacts.

**DOT-10 — Targets and gates.** Retain source targets of trigger-to-proposal under 30 s, interrupt delivery under 5 s, and restart reconstruction under 60 s as proposed SLOs. Certify historical-receipt overclaim, exhausted budget, scope creep, denied steering, healthy silence, and crash recovery before autonomous pilot activation. Project coach rollout is independent of personal-dot availability.

## 07 | plugins | A smaller plugin concept with full expressiveness

**PL-01 — One package, many capabilities.** A plugin is a versioned, integrity-checked capability package. It may contain declarative data only, a Wasm component, a trusted bundled implementation reference, or a registered remote implementation reference. The package says what it provides and requires; a deployment binding says where it can execute; a separately stored grant says what this installation may access. Avoid distinct “agent plugin,” “tool plugin” and “UI plugin” lifecycles.

**PL-02 — Keep the contract small.** Five required groups are identity, capabilities, artifacts/implementations, compatibility, and integrity. Settings, UI, events and distribution are optional contributions. A capability card names an ID/version, description/tags, JSON Schema 2020-12 action inputs/outputs, emits/consumes declarations, dependencies and required host profile. Headless is normal. Unknown capability namespaces remain discoverable and schema checked but receive no special pipeline authority.

```json
{
  "schema": "knowme.package/1-proposed",
  "id": "example.commitments",
  "version": "1.0.0",
  "publisher": "<publisher-identity>",
  "capabilities": [{
    "id": "example.commitment.review", "version": "1.0.0",
    "profile_required": "full-ui",
    "actions": [{"id": "review", "input_schema": "schemas/input.json",
                 "output_schema": "schemas/output.json"}],
    "requires": ["host:container.read"],
    "emits": ["example.commitment.reviewed"]
  }],
  "implementations": [{"kind": "wasm-component", "path": "review.wasm",
    "world": "<exact-supported-world>", "profiles": ["desktop-component"]}],
  "ui": {"catalog": "<pinned-catalog>", "surfaces": ["ui/review.json"]},
  "integrity": {"files": "sha256.json", "signature": "manifest.sig"}
}
```

This example is a proposed interchange view, not a claim that the current parser accepts it. Its schema migration must preserve every field in the existing capability-card model: settings scopes, actions, events, chunks, `requires`, `enhances`, namespace proof, combination risks, license, entitlement, SBOM and governance proposals. Move them into orthogonal groups; do not discard them to make a shorter manifest.

**PL-03 — Installation transaction.** Download from an explicitly chosen source, bound total bytes and archive depth, reject path traversal/symlinks escaping staging, verify each hash and signature, validate schema and namespace proof, resolve dependency graph and pinned versions, check supported world/imports, compute effective capabilities and risky combinations, show permission diff, obtain required consent, then atomically activate one generation. Invalid signatures reject. Unsigned local developer packages require an explicit developer trust policy; they never display verified publisher status.

**PL-04 — Lifecycle.** States are discovered, downloaded, verified, awaiting consent, installed, active, suspended, quarantined, disabled, and removed. Update is a new generation with rollback pointer and additive migration plan. New privileges require renewed consent. Revocation/advisory refresh can quarantine a compromised generation. A crash-loop circuit breaker disables repeated failures and preserves recoverable data. Uninstall revokes execution and subscriptions first, then offers retention/export or data deletion according to declared policy.

**PL-05 — Capability enforcement.** Effective authority is the intersection of declared capability, installation grant, user/organization policy, current task scope, platform profile and live budget. Capability discovery is never a grant. The host stamps event provenance, checks emit allowlists and rate caps, validates all action I/O, and prevents contributor reentrancy. Events cannot impersonate another publisher or fabricate a trusted tool outcome.

**PL-06 — Hooks.** Preserve ingress → intent-classify → route → context-assemble → prompt-build → inference → tool-gate → tool-execute → post-process → egress. Hooks declare stages and data classes at install time. They return typed contributions with source and budget; they do not overwrite prompts or weaken policy. Deterministic order is priority then stable plugin ID. The context allocator decides admitted tokens. A proposed 200 ms hook limit must be measured per profile; timeout skips a contribution with a trace, never silently grants more time.

**PL-07 — Identity, sharing and marketplace.** Definitions travel; identities, secrets and grants do not. Every deployed agent instance gets its own principal. Audit records user subject and agent actor. Signed exports, IPFS/OCI distribution, and peer delivery use the same verification pipeline. Keep publisher identity, key rotation, license, SBOM and entitlement metadata now; storefront business policy remains deferred. Offline entitlement verification is not action authorization and does not promise tamper-proof DRM. A lapsed update entitlement must not silently delete local work.

**PL-08 — Settings.** One schema-driven substrate stores definitions and values with owner kind/ID, device/user/tenant/agent scope, schema version, sensitivity, sync class and UI hints. Secret settings store keychain/Vault references only. Validate on write and migration. Settings precedence and defaults are inspectable. Generate native A2UI forms from the same schemas; custom settings surfaces bind the same definitions rather than introducing a second store.


### PL-09 — Website capability provenance

PL-01 and PL-05 also govern reviewed browser adapter packages. A live website tool remains a separate `source_kind=webmcp` capability, never an automatically installed package or inherited grant. Credentials and browser-profile partitions do not travel in packages. See [WEB-01–14](#workspace), [proposed API-07](#api) and [adapter placements](#runtimes).

## 08 | runtimes | Rust, FFI, and component execution boundaries

**RT-01 — Mobile is native.** Flutter widgets call Riverpod providers, repositories, then generated Rust FFI. Rust owns networking for domain services, model/provider routing, persistence, policy, context and agent coordination. Flutter owns rendering, gestures, accessibility and necessary OS/RTC SDK adapters. The LiveKit Flutter media engine captures/renders tracks through native SDK plumbing; this bounded media exception must not become a second domain service layer.

**RT-02 — Desktop and web.** Tauri components call hooks, stores and typed IPC; stores own API calls. Browser components use the same service semantics through server-backed HTTP/SSE or a deliberately smaller browser/Wasm facade. A browser core module compiled for `wasm32-unknown-unknown` is not a WASI Preview 2 component host. `wasm32-wasip2` is a guest component target with explicit WIT imports. Treat these as separate compilation and integration products.

| Execution profile | Baseline | Limits and fallback |
|---|---|---|
| Declarative portable | A2UI, settings, capability metadata, workflows without downloaded native code | Host vocabulary and policy must support every required primitive/action. Unsupported surfaces show a labeled fallback. |
| Trusted bundled native | Reviewed Rust code included in the signed application; FFI/IPC adapters | New native capabilities ship through application release and platform review. |
| Desktop/server component | Wasmtime Component Model, pinned WIT and host ABI | OS/runtime resource isolation, import grants, fuel/epoch/memory budgets; no ambient credentials. |
| Mobile interpreted component | Optional separately certified profile, such as an interpreter path | Not the default promise. Store-policy review, interpreter build, battery/memory measurements and physical-device tests required. No untrusted precompiled native cache on iOS. |
| Browser adapted component | Optional jco/component adaptation in a worker with explicit host shims | Browser permissions, CSP, network origin constraints, unsupported WASI APIs and termination semantics differ from native Wasmtime. |
| Remote execution | Registered peer/server worker reached through authenticated API | Explicit off-device data/placement consent; online requirement and latency visible. Never silently used when local execution is unsupported. |

**RT-03 — FFI contract.** Expose typed command requests and results; where current bridges carry JSON strings, validate schema at the Rust boundary and keep migration versioned. Use stable IDs, enums and owned buffers. Never expose raw pointers to plugins. Document ownership/lifetimes, cancellation tokens, stream subscription disposal, panic containment, error taxonomy and engine shutdown. Expensive CPU/blocking work runs off the UI isolate and async executor core. Repeated initialization must not create duplicate runtimes.

**RT-04 — WIT.** The local 1.0.0 source includes granular host logging, KV, settings, network, UAR invocation, events, inference, memory, filesystem, environment and time interfaces, with `host-a2ui` only for app-shell. Hook, contributor and app-shell exports are implemented bindings; other proposed worlds remain separately tracked. Linking a world is not a blanket grant to every declared interface: import compatibility and per-call capability checks both apply. Narrow worlds for pure evaluation should omit unneeded imports.

**RT-05 — Sandbox.** Enforce linear memory limits, CPU/fuel limits, wall-clock deadlines, host-call byte/time quotas, bounded event queues and maximum recursion/concurrency. The source default is 64 MiB and 30 s per event; hooks and mobile need tighter measured profiles. A zero fuel setting in current types is effectively unbounded and is not an acceptable untrusted production default. Termination must cancel host I/O and mark uncertain external effects for reconciliation.

**RT-06 — Host calls.** Inference is a host callback under a filtered model registry and budget, never a provider key handed to the guest. Network is destination-constrained with DNS/IP/redirect validation against SSRF, including loopback, link-local and private-network policy. Files use scoped handles or validated roots, not unrestricted path strings. KV namespaces include plugin instance and user scope. Logs are bounded and redacted. Credentials stay in the host secret store.

**RT-07 — Caches and versions.** A Wasmtime `.cwasm` cache is a host-produced optimization, never a portable trusted artifact. Bind cache keys to component digest, engine/compiler version, target, feature configuration and signing provenance; invalidate on mismatch. Never deserialize attacker-provided precompiled native output. Pin the current host major and plan security updates; historical ADR LTS recommendations and current dependency pins must be reconciled by a release decision, not guessed from a calendar.

**RT-08 — Honest mobile scheduling.** Foreground, OS-opportunistic and delegated execution are distinct modes. iOS/Android background opportunities do not provide reliable arbitrary cron. A browser tab or service worker is not an always-on scheduler. The UI shows actual placement, next eligible attempt, missed opportunity, and delegated status. A call's permitted media background mode does not authorize unrelated perpetual agent compute.


## Adapter placement and Flutter

| Placement | Strategy | Limits |
|---|---|---|
| Cooperative embedded web site | JS/TS or Dart interop to actual document API | Browser-permitted frame tree and negotiated exposure only. |
| Browser host/extension | Authorized integration with tested execution world | Explicit page host permissions and installation/access. |
| Controlled local/remote Chromium | Experimental CDP domain if target advertises it | Pin protocol, monitor target/frame; remote endpoint never page-accessible. |
| Native Flutter | Typed platform/FFI broker to an admitted browser | WebView only after actual engine fixtures pass. |
| Unsupported engine/blocked embed | Connector, permitted UI automation, external/manual completion | Same Activity; disclose changed session and evidence. |

[Flutter WebView](https://pub.dev/packages/webview_flutter) uses system engines including WKWebView and Android WebView. [Platform channels](https://docs.flutter.dev/platform-integration/platform-channels) carry asynchronous native messages; web uses JS interop. Neither creates missing browser support. A2UI may render a host-owned origin/action/authority/status review and open/takeover control; the website remains a separate browser surface. Never import website scripts into the native catalog.



## 09 | surfaces | A2UI and the native interaction contract

**UI-01 — One approved catalog.** App, agent and plugin producers emit declarative components resolved against the installed catalog/profile. UAR owns the generic catalog contract; KnowMe owns renderer conformance and its cache/contributions. Catalog ID, wire version, component version and renderer capability must be negotiated independently. Historical `uar.a2ui/1`/Google v0.9.1 declarations remain pinned source profiles, not claims of the latest upstream version.

**UI-02 — Fail closed with a useful explanation.** Unknown components or actions never become HTML/JavaScript. Render an inert labeled fallback with source, required profile and a supported alternative. A partially supported surface cannot leave a destructive action enabled when its required confirmation or fields are missing. A2UI is data validation, not protection against misleading text; provenance and action policy remain necessary.

**UI-03 — Interaction.** Text inputs echo and validate locally. Send semantic actions—field committed, form submitted, item selected—not raw keystrokes. Actions include surface/component IDs, version and command identity, and use the same audited service path as chat tools. Debounce only where it preserves semantics. Persist drafts independently from authoritative entity state and show optimistic updates with their pending status.

**UI-04 — Surface lifecycle.** Create, update components, patch data, delete and restore have explicit revisions. Bind durable surfaces to watched graph state, not one run's transient stream. A run can propose a surface; the host persists the approved definition and bindings. On reopen, check producer installation, catalog, grants and entity freshness before enabling controls.

**UI-05 — Content blocks.** Preserve the closed ContentBlock vocabulary and its typed tool, citation, memory, image/audio, artifact and status paths. Provider-reported reasoning, where present, stays in a separate provenance-tagged block and is excluded from future prompts, fact extraction and summaries by default. Do not imply access to hidden model reasoning. Raw protocol frames belong in an inspector. Tool calls/results remain paired across truncation and replay.

**UI-06 — Flutter UX.** Use native scrolling, keyboard insets, safe areas, platform back behavior, focus restoration, screen-reader semantics, large-text reflow, reduced motion and minimum touch targets of 44 points on iOS/48 dp on Android as design targets. A composer remains above the keyboard without obscuring the selected message. Attachments have progress, cancel and failure states. Selection/copy, streaming stop, and unsent text survive navigation. Dynamic UI must announce meaningful status changes without reading every token aloud.

**UI-07 — Visual system.** Retain KnowMe Flat 2.0: surfaces separated by tone, restrained accent, no decorative layout borders/shadows. Renderers inherit shared typography, spacing, status and theme tokens. Tables permit local horizontal scrolling on phones; the entire page must not overflow. Charts require text equivalents. Dark mode is a first-class validation target, not a color inversion.

**UI-08 — Legacy embed exception.** Resolve `embed.ref` to a trusted registered origin and sandbox policy. Verify exact origin, `event.source`, protocol version, mini-app identity and correlation ID for every bridge message. Send context/action payloads, never bearer tokens. Do not revive the rejected path-prefix reverse proxy for root-relative application assets. Embedded desktop/web apps need a native or external-browser fallback on mobile; the UI discloses that limitation before opening.


### UI-12 — Visible website sessions and shared control

Keep the actual site in a visible embedded or external browser canvas. Show actual execution placement, site/frame origin, observable account/authentication status, tool availability and catalog freshness, pending review, cancellation and uncertain outcomes. A2UI supplies a host-owned status/review/takeover surface; website scripts never become catalog components. Preserve manual form submission; `toolautosubmit` is not consent. Host `requestTakeover` and `resumeAfterTakeover` are proposed semantics, not upstream WebMCP methods: pause competing agent input and revalidate before resume. Keep keyboard/screen-reader access, focus and thread drafts across handoff. UI-08 and existing sandbox/origin rules still apply. Unsupported engines offer an explicit fallback under WEB-14.

## 10 | context | Context, models, memory and knowledge

**CX-01 — One context fabric.** `gen_ui_context` resolves device, model and runtime profiles plus measured session envelope into an effective budget. The builder owns ordering, truncation, tool shortlisting, retrieval and escalation. UI adapters load profiles and display decisions; they do not duplicate budget arithmetic. A model's advertised context length is not its measured reliable retrieval capacity.

**CX-02 — Budget invariant.** System, tool schemas, history, memories, knowledge, output reserve, reasoning reserve and margin must fit the effective ceiling. Exact tokenizer counts are preferred; heuristic estimates carry an explicit wider margin and telemetry. Context presets in the retained source are initial targets. Validate their arithmetic on every turn, especially when reasoning is enabled, rather than assuming source tables are internally additive. Thermal pressure or sustained latency reduces the local budget and offers an explicit handoff choice.

**CX-03 — Strategies.** Preserve sliding window, truncate-middle with counted omission marker, progressive summary, hierarchical short/mid/long-term memory, and pre-compact triggers. Never split a tool call from its result. Summaries store covered turn IDs and provenance. If summarization fails or is thermally disallowed, fall back visibly to sliding window for that turn. Fact extraction is opt-in and writes through the governed memory store rather than a hidden second memory.

**CX-04 — Retrieval.** Preserve atomic short notes, semantic boundary chunking, recursive overlap, and parent-child retrieval. Chunks have stable document/ordinal IDs, source spans, token estimates and ingestion provenance; document replacement is transactional. Separate memory and knowledge budgets prevent a large document from starving explicit personal memory. Fuse eligible vector/BM25/graph lanes, apply bounded diversity/recency ranking and deterministic placement. A cross-encoder is a measured follow-up, not a default mobile burden.

**CX-05 — Model resolution.** Global role slots, agent/Hand pins, deployment-target configuration and conversation overrides resolve to a permitted provider/model with required capabilities. Preserve `provider::model` identity, embedding model/dimension stamps, re-embedding migrations, explicit cloud consent and local-only usage-based suggestions. Remote fallback is a proposal, never a silent privacy downgrade. A model unavailable on the current device produces an honest state with alternative placement choices.

**CX-06 — Acquisition.** Retain curated catalog provenance, license gating, user-chosen model imports, resumable downloads, content integrity, available storage checks, hardware/memory probes, and post-download validation. Source engine/model combinations are lane-specific and historically dated. Certify model load, tokenizer/template correctness, bounded generation, tool/structured output semantics and physical-device resource behavior against actual artifacts. A successful download is not a successful local inference deployment.

**CX-07 — Knowledge base.** Preserve source-linked knowledge/wikis, incremental ingestion, link/provenance graphs, stale-source detection, index rebuilds and export. Derived summaries do not overwrite originals or acquire higher authority than their sources. Deletion propagates through eligible indexes and derived artifacts with an inspectable receipt; it cannot recall previously disclosed text from another participant or provider.

**CX-08 — Capture and tools.** Ask Image, Audio Scribe, Prompt Lab and Skills remain first-class workflows. Vision/OCR/document extraction preserve attachments and source spans; Audio Scribe keeps recording, transcription, speaker labels and summaries distinct; Prompt Lab supports ephemeral single-turn templates and model comparison; Skills distinguish atomic tools from packaged practices with instructions and policy proposals. MCP is an adapter at the edge, not a bypass around grants or budgets.

## 11 | realtime | LiveKit rooms for people and dots

**RTC-01 — One media standard.** Use LiveKit for one-to-one and multiparty audio/video. Supported compositions are human↔human, human↔personal dot, human↔project coach, multiple humans, and multiple humans with one or more explicitly invited dots. A dot is a visibly labeled agent participant, with separate identity and permissions. A call can exist without a dot; enabling a dot does not automatically join calls.

**RTC-02 — Split responsibilities.** LiveKit provides room/media transport and SDK primitives. The Rust application control plane authenticates participants, creates call records, applies container access, issues short-lived scoped tokens, selects agent workers, records consent and persists durable outcomes. Use LiveKit Agents Python/Node workers for supported agent orchestration; Rust RTC/server/auth SDKs do not establish a Rust equivalent of the complete Agents framework. A worker calls UAR tools through a bounded adapter with subject/actor/grant context.

**RTC-03 — Join flow.** Select a container and intended participants; create a call policy; review microphone/camera, recording/transcription and agent-processing choices; request OS permissions; obtain a short-lived room-scoped credential; connect and publish only allowed tracks. Human invitees authenticate under the room access policy. Tokens are not embedded in public pages or plugin manifests. A join URL locates a call; it is not by itself unlimited room authority.

**RTC-04 — Multiparty agent turn-taking.** A LiveKit room can contain many participants, while a default AgentSession RoomIO connection is associated with one participant. Multiparty conversation therefore needs an explicit coordinator. Preserve participant-scoped audio input/transcription, speaker IDs and timing. The coordinator tracks addressed agent, explicit mention/push-to-talk, current floor, interruptions, pending responses and moderation. It routes a bounded utterance with the permitted shared context to the selected dot. Only one agent owns the response floor unless a deliberate multi-agent mode is enabled.

**RTC-05 — Interruption and echo.** Human speech may interrupt agent output according to room policy. Cancel or truncate the corresponding response and persist the actually delivered transcript extent. Do not feed agent playback into human transcription as a new human utterance. Separate microphone tracks, use SDK echo cancellation where available, and retain participant attribution. A reconnect must not restart an already delivered response or duplicate a tool effect.

**RTC-06 — Shared context.** Room context is its own scoped object containing explicitly shared artifacts, participant consent, transcript policy and selected container references. A dot's private memory is not room memory. A project coach may explain its packet using authorized evidence; it cannot read another project to answer a casual question. Whisper/private-to-dot channels need explicit isolated delivery and UI state; never implement them as hidden messages in the shared transcript.

**RTC-07 — Tools and decisions.** Spoken intent can request an action, but a high-impact effect still goes through a typed proposal and the applicable authenticated decision surface. Transcription confidence is not identity proof. A statement attributed to the wrong speaker must not authorize an effect. Call summaries include decision IDs and receipts; they do not create approvals retroactively.

**RTC-08 — Recording, transcription and encryption.** Recording, transcription, summaries and agent inference are separate consent/data flows. Before enabling each, show who receives media/text, storage location, retention and available deletion. Transport encryption protects a connection; end-to-end media encryption requires a separate key/distribution design and compatible participants. An agent that hears encrypted media necessarily receives plaintext at its endpoint. Server recording/transcription compatibility cannot be assumed. A change in processing mode reopens the relevant consent gate.

**RTC-09 — Mobile behavior.** Native Flutter controls handle device switching, camera facing, Bluetooth/headset routes, mute, speakerphone, background/foreground transition and interruptions by system calls. Request permissions just in time. Provide audio-only fallback for constrained bandwidth/thermal state. Screen sharing and picture-in-picture use platform-specific support and explicit availability states. Losing camera permission must not disconnect a permitted audio call.

**RTC-10 — Failure recovery.** Distinguish connecting, connected, reconnecting, degraded, permission denied, removed and ended. Short media reconnection preserves the call record and participant identity while refreshing authorization as needed. Long gaps show missing transcript intervals. TURN/network restrictions, worker crash, revoked membership and provider outage have separate remediation. If agent inference fails, human conversation continues where possible and the roster marks the dot unavailable.

**RTC-11 — Multiparty controls.** Provide participant list, invite/remove, mute controls within host authority, raise hand, active speaker, speaker labels, connection quality, agent listening/responding state and clear leave/end distinction. Late joiners receive only the history permitted by policy. A host ending the room closes agent jobs and durable subscriptions with terminal receipts; leaving personally does not necessarily end a multiparty room.

## 12 | security | Identity, authority and trustworthy outcomes

**SEC-01 — Four separate checks.** Package signature establishes integrity/publisher identity. Capability admission establishes a supported contract. A user grant establishes scoped authority. Runtime policy determines whether this action may execute now. No single green “trusted” badge should collapse all four. Surface unsigned, unsupported, ungranted and revoked independently.

**SEC-02 — Representation.** A private context profile does not grant the right to impersonate its owner. Named-human representation requires a separately stored RepresentationGrant with issuer, subject, audience, allowed actions, context classes, disclosure, expiry, revocation and evidence revision. A generic role assistant remains generic until that grant exists. A personal grant cannot authorize organizational spending. Voice surfaces identify AI participants plainly.

**SEC-03 — Secret custody.** Provider keys, room signing secrets, peer credentials and publisher private keys remain in keychain/keystore/Vault or equivalent host storage. Guest components receive capability handles and scoped outputs. Public signed manifests contain public verification material only. Logs, exports, screenshots and crash diagnostics pass redaction controls before leaving the device.

**SEC-04 — Prompt injection.** Treat retrieved documents, plugin descriptions, messages and transcripts as untrusted content. They may inform a response but cannot redefine grants, executable routes, identity or acceptance predicates. Tool arguments are schema checked and policy evaluated independently of model claims. A2UI text can be deceptive even when no script runs; consequential actions show host-owned labels, source and destination.

**SEC-05 — Receipt limits.** A signed receipt proves an issuer signed a statement about a specific candidate/check; it does not prove the check was correct or the signer uncompromised. Record tool version, environment, input digest, outcome and uncertainty. Separate observation from interpretation and deterministic check from semantic classification. Export supports independent verification of signatures, chains, versions and missing segments.

**SEC-06 — Public artifacts.** Publishing a paper is a separate effect from deploying KnowMe. The artifact may contain technical schemas, synthetic examples and source hashes. It must exclude credentials, private personality/writing substrate, customer records and unnecessary local paths. A public IPFS CID is immutable distribution, not access control or a deletion guarantee. Publish only the vetted artifact directory.


### SEC-07 — Browser site trust boundary

Apply WEB-06–13 at the site/browser/host boundaries: exact origin/frame/session admission, independent grants and approvals, bounded schemas and results, private-argument egress policy, no credential export, and host-owned policy against descriptor/result poisoning. A descriptor digest cannot prove callback semantics or eliminate a same-name re-registration race. Preserve SEC-04 injection defenses and the existing decision/effect digest authority. Null, abort, timeout and disconnect cannot certify a real-world effect; reconcile before unsafe replay. Privileged CDP/extension handles never cross into page code or downloaded guests.

## 13 | implementation | Implementation ownership and migration

| Workstream | Owning seam | Deliverable and proof |
|---|---|---|
| Workspace/home | Desktop feature shell and Flutter feature shell | Container-aware home, composer modes, dashboard freshness, suggestions-as-drafts, pinned A2UI surfaces; responsive and accessibility proof. |
| Container substrate | Rust domain/repository/service layer | Typed variants, edges, gates, optimistic revision control, intent receipts and scope-safe replay. |
| Plugin package | Generic UAR contract plus KnowMe product adapter | Schema, capability cards, registry proof, install/update/quarantine generations, current-dialect adapters. |
| Component host | Existing plugin-host facade and sandbox | Import/per-call enforcement, budgets, cancellation, cache integrity and denied-call fixtures. |
| Native bridge | `gen_ui_ffi`, Tauri IPC, browser/server facade | Versioned DTOs, stream lifecycle, cancellation and identical service outcomes. |
| A2UI | Catalog/profile owner and platform renderers | Closed vocabulary, durable surface store, semantic actions and paired native/web conformance fixtures. |
| Private dots | Personal Hand and project coach as separate packages | Explicit scope, contract, budgets, receipts, stop behavior and export. |
| Realtime | Rust call service + LiveKit SDKs + Python/Node workers | Human-only rooms first, then personal dot, then coordinator-backed multiparty dots. |
| Context/model/knowledge | Rust context/inference/database abstractions | Budget arithmetic, acquisition gates, provenance, no silent cloud fallback. |

**IM-01 — Sequence.** First freeze vocabulary and generate contracts/fixtures. Then prove a minimal vertical: one container, one dashboard, one declarative plugin, one typed action, one durable receipt, and the same surface on desktop and Flutter. Next add component execution on desktop/server with denial tests. Add private dots independently. Add human-only LiveKit calling, one-to-one agent calling, then multiparty arbitration. Roll out optional browser/mobile component profiles only after their distinct certification gates.

**IM-02 — Compatibility.** Maintain a migration registry mapping source ABI/world/schema to destination and adapter version. Preserve legacy manifests as immutable inputs with content hashes. Convert navigation placement independently from protection and surface kind. Do not infer permission grants from old UI visibility. Run old and new fixtures against the same host service contract before retiring an adapter.

**IM-03 — Data migration.** Use additive schemas, explicit version stamps, transactionally checkpointed backfills and reversible feature activation. Never delete old personal data merely because a plugin profile is unsupported. Rollback may disable new execution while keeping command/receipt history. Reconcile uncertain effects before replaying them under a different runtime generation.

**IM-04 — Workstream isolation.** This specification does not authorize changes to the active UAR implementation, infrastructure, signing policy or release workflow. Its proposed migration is a reviewable future implementation contract. Source ownership splits remain explicit so a product document does not silently create a competing agent runtime.


## Phased implementation and remaining decisions

Phase 0 pins source/browser/protocol fixtures and threat-models actual boundaries without production credentials. Phase 1 supplies explicit same-origin read-only sessions, receipts and lifecycle invalidation. Phase 2 adds consequential tools with decisions, budgets, cancellation and authoritative reconciliation. Phase 3 separately certifies cross-origin policy, declarative collaboration and extension/CDP placements, then mobile/native handoff. Phase 4 proves interoperability and accessible parity across supported profiles. Each phase requires its G17 evidence and a disable-admission rollback preserving receipts.

Release decisions remain: adapter support matrix, legacy cutoff, trial dependence, normative draft pin, elicitation evolution, output-schema ownership and real mobile engine behavior. No rollout or UAR edits are authorized by publishing this proposed contract.

Suggested figure: Container + Activity → candidate → policy/data/approval → capability broker → browser-session adapter → origin/frame-bound tool → untrusted observed result → evidence verification → durable Activity. Show A2UI review at the host gate, visible browser at the adapter with takeover/resume, distinct MCP/UI-automation branches, website/session/host boundaries and verified-effect-or-uncertainty outcomes.



## 14 | acceptance | Acceptance gates and measurable failure cases

| Gate | Required scenario and pass condition |
|---|---|
| G01 Source coverage | Every inventoried technical section/requirement has a stable source hash, retained detail location, owning revised clause and disposition; zero unexplained omissions. |
| G02 Home continuity | Empty, populated, stale, offline, denied and failed dashboards at compact/tablet/desktop widths; suggestions never execute without submission; restore draft/card state after restart. |
| G03 Container isolation | Cross-user/tenant/project read and mutation attempts fail at the service boundary; badge counts and semantic search do not leak hidden containers. |
| G04 Intent reconciliation | Duplicate submission, lost reply, process kill, conflict, revocation and scope change preserve identities and never replay an old command under new authority. |
| G05 Plugin admission | Tampered hash, invalid signature, namespace squatting, undeclared import, new privilege, missing profile and malicious archive all fail safely. |
| G06 Runtime limits | Infinite compute, memory growth, slow host I/O, recursion and flood events hit distinct enforced limits; a failed plugin cannot wedge the UI or leave an unbounded queue. |
| G07 A2UI parity | Shared fixtures on React and Flutter in light/dark, compact/wide and large text; unknown components inert; local echo and semantic actions preserve schema/version identity. |
| G08 FFI lifecycle | Repeated boot, stream disposal, cancelled run, panic/error propagation and background/foreground transitions do not duplicate engines, leak subscriptions or block the UI. |
| G09 Personal dot | Local default, explicit conversation cloud route, separated memory, editable/exportable beliefs, five honest commitment outcomes, no unsolicited third-party contact. |
| G10 Project coach | Stale evidence, exhausted allowance, scope creep, denied task, healthy silence and restart cases satisfy every PD requirement; KBD remains verdict owner. |
| G11 Human media | One-to-one and multiparty audio/video, permission denial, headset switch, reconnect, late join, participant removal and end-room tested with distinct accounts/devices. |
| G12 Agent media | One-to-one and multiparty room arbitration, overlapping speech, barge-in, echoed audio, worker crash, revoked dot grant and repeated tool request; no duplicate effect or private-memory leak. |
| G13 Data consent | Recording/transcription/agent processing independently consented; decline remains usable; encrypted mode compatibility and plaintext endpoints disclosed accurately. |
| G14 Context | Exact budget invariant, paired tool messages, retrieval provenance, summarizer degradation, thermal limits and cloud-refusal behavior measured on real model artifacts. |
| G15 Accessibility | Keyboard-only controls, screen-reader labels, focus return, contrast, reduced motion, dynamic type and no page-level horizontal overflow. |
| G16 Publication | Hash final files, verify public HTML and every required asset, render desktop/mobile, inspect diagrams, keep receipt with actual CID and observed URLs. |

Performance values are proposed targets pending measurements: local navigation should respond within one frame budget where possible; permission/action validation and first useful feedback should be bounded and visibly pending; media tests must record join time, RTT, packet loss, jitter, first response, interruption latency, CPU, memory, battery and thermal state. Do not turn an unmeasured target into a marketing number.

Release evidence records build/commit digest, host/runtime/catalog versions, device/OS, test inputs, timestamp, result and known gaps. A desktop screenshot cannot certify an iOS plugin runtime. A room with two tracks cannot certify multiparty agent turn-taking. A source file named `sandbox` cannot certify a secure installer.


## G17 — WebMCP site integration fixtures

Pass only named adapter/browser profiles; unsupported profiles must fail safely with an explained fallback.

1. Distinguish absent API, policy disabled, zero tools and disconnect.
2. Test current document-based imperative registration/discovery/execution/removal separately from legacy navigator fixtures.
3. Normalize object, primitive string, nested JSON, throw/rejection, unserializable and oversize results without execution/double decoding.
4. Reject invalid instances, unsupported dialects, costly regex/depth and remote `$ref` before dispatch; disclose absent output schemas.
5. Manual declarative invocation fills/focuses fields, preserves validation and waits for human submit; cancel/reset stays visible.
6. Automatic declarative invocation still passes host approval/egress; denied candidates never invoke forms.
7. Test same-origin, denied cross-origin policy, delegation without exposure, exposure without consumer selection, all grants, ancestor deny and opaque sandbox; no denial bypass.
8. Duplicate names across frames/tabs execute only the selected target and record actual provider identity.
9. Test full navigation, redirects, SPA route, iframe replacement/removal, BFCache and close; no speculative mutation replay.
10. Remove/re-register during approval invalidates observed descriptor changes; disclose remaining malicious same-name atomicity limitation.
11. Cancel before dispatch, during execution, after removal and concurrently with completion; canonical outcome and no fabricated rollback.
12. Disconnect after dispatch; reconnect same target versus new document; no duplicate purchase/message/delete.
13. Poisoned names/descriptions/schema/results/fake approvals cannot grant authority or exfiltrate another container.
14. False read-only/consequential annotations and success prose cannot bypass policy or prove effects.
15. Separate profiles/accounts/login transitions/revocation/private browsing; no credentials in logs or model/bridge input.
16. Takeover pauses competing input, preserves page, revalidates on resume; failed remote handoff cannot auto-submit.
17. Run equivalent fixtures on each claimed actual web/native/remote engine/protocol; unsupported WKWebView is not a skipped pass.
18. Bound tool floods, churn, schema/result sizes, callback duration and event queues without wedging Flutter.
19. Canvas close/reopen and process restart preserve Activity/audit, then recreate live handles without replay.
20. Keyboard and screen-reader review identifies origin and supports reject/takeover/cancel/return with restored focus.

Record build digest, device/OS/engine, adapter/protocol/fixture revision, trial/flag state, test origin topology, inputs, times, observations, cancellation behavior and known gaps. Use controlled fixtures, never production purchases, sends or destructive data.



## 15 | decisions | Decisions that remain open

The recommended default is native Flutter plus shared Rust services, declarative A2UI portability, desktop/server component execution, explicit remote fallback, and LiveKit with a separate multiparty coordinator. That is sufficient to implement the first complete vertical without betting on mobile downloaded-code acceptance.

Decisions still needing implementation evidence are: the generic UAR plugin namespace/version migration; minimum production runtime/toolchain pins; mobile interpreted component admission and measured budgets; browser component adaptation; room encryption/key management and agent participation policy; recording/transcript retention; reliable delegated scheduling; and exact model/hardware presets. Marketplace revenue policy and LoRa transport remain deferred source proposals. They are not prerequisites for a working workspace.

The cost of this design is real. Shared semantics require conformance work across native and web renderers. Explicit grants add state to installation and collaboration. A multiparty voice agent needs coordination beyond a room connection. Those costs buy something concrete: the user can tell what is running, what it can see, who it represents, and whether its work actually happened.


### Browser integration release decisions

The supported adapter/engine matrix, legacy API cutoff, origin-trial dependence, exact normative and CDP fixture pins, declarative parity, output-schema ownership and future elicitation remain explicit release decisions. Browser API presence, trial enrollment, adapter installation, user consent and verified effect are separately reported. No universal Safari/WKWebView or cross-browser parity is asserted.

## 16 | provenance | Sources, authorship and coverage

Prepared for KnowMe from the user-provided published paper, selected KnowMe and San Saba technical specifications, current local code inspection, and primary documentation. The original paper is preserved locally byte-for-byte. Technical source sections are retained in full in the integrated requirements records, with privacy edits documented in the transformation ledger and contradictions governed by §02.

**Authorship:** new prose, diagrams, examples, integration contracts and editorial decisions are AI-generated with Codex (OpenAI; model identifier not independently exposed in this execution). No claim of human editing or approval of every sentence is made. The original article is attributed as supplied; its prior AI/human provenance was not independently established. The authentic digital twin writing substrate guided naming and voice and is not included in the public artifact.

Each source-derived block has a source ID and revision hash. Each new chapter has a stable clause prefix. The coverage matrix maps source sections and explicit requirement identifiers to their retained text and revised owning clause. An original-source reference is not an assertion that the described feature ships. Historical pricing, market observations and dated dependency claims remain historical context. Public technical references use repository-relative paths; private raw snapshots and audit paths remain in the local evidence package.

## Primary-source byte verification supplied by the research task

Fetched 20 October 2026 UTC. These hashes identify research inputs, not runtime support:

- [Pinned normative source](https://raw.githubusercontent.com/webmachinelearning/webmcp/cb3d16964ab3e92cfd5f09901ed0504373a906e8/index.bs): 88,074 bytes; SHA256 `dabc073a28a21131d6c175a0738f7554c8c6ab431c4997ce5f1450a4da834a39`.
- [Pinned CDP schema](https://raw.githubusercontent.com/ChromeDevTools/devtools-protocol/95b1af69d6a20161dc3558f8e38366b7fa7a27d1/pdl/domains/WebMCP.pdl): 4,126 bytes; SHA256 `f386bcfd036328f1fb1572b5404bfad09a7f6ff078093241fbc3409a1036f99f`.
- [Machine-readable Chrome Status evidence](https://chromestatus.com/api/v0/features/5117755740913664): stage-history evidence, not a shipping guarantee.

### Implementation cautions retained verbatim from the research handoff

- Do not implement a guessed `navigator.webmcp` or a generic browser `tools/list` method.
- Do not treat `getTools()` as a cross-tab registry. Use host-owned target routing with separately admitted sessions.
- Do not infer permission to call a cross-origin tool solely from an iframe's `allow` attribute.
- Do not depend on `requestUserInteraction` or `requestUserInput` in the current reference profile.
- Do not send raw CDP/extension privileges to an untrusted webpage or downloaded plugin.
- Do not normalize a navigation-related null/abort/disconnect into business success.
- Do not promise that descriptor hashing fixes a malicious same-name registration race.
- Do not label browser API availability, trial enrollment, package installation, user consent and effect completion with a single “supported/trusted” status.
