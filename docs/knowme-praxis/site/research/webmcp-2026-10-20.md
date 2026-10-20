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

## Adapter placement and Flutter

| Placement | Strategy | Limits |
|---|---|---|
| Cooperative embedded web site | JS/TS or Dart interop to actual document API | Browser-permitted frame tree and negotiated exposure only. |
| Browser host/extension | Authorized integration with tested execution world | Explicit page host permissions and installation/access. |
| Controlled local/remote Chromium | Experimental CDP domain if target advertises it | Pin protocol, monitor target/frame; remote endpoint never page-accessible. |
| Native Flutter | Typed platform/FFI broker to an admitted browser | WebView only after actual engine fixtures pass. |
| Unsupported engine/blocked embed | Connector, permitted UI automation, external/manual completion | Same Activity; disclose changed session and evidence. |

[Flutter WebView](https://pub.dev/packages/webview_flutter) uses system engines including WKWebView and Android WebView. [Platform channels](https://docs.flutter.dev/platform-integration/platform-channels) carry asynchronous native messages; web uses JS interop. Neither creates missing browser support. A2UI may render a host-owned origin/action/authority/status review and open/takeover control; the website remains a separate browser surface. Never import website scripts into the native catalog.

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

## Phased implementation and remaining decisions

Phase 0 pins source/browser/protocol fixtures and threat-models actual boundaries without production credentials. Phase 1 supplies explicit same-origin read-only sessions, receipts and lifecycle invalidation. Phase 2 adds consequential tools with decisions, budgets, cancellation and authoritative reconciliation. Phase 3 separately certifies cross-origin policy, declarative collaboration and extension/CDP placements, then mobile/native handoff. Phase 4 proves interoperability and accessible parity across supported profiles. Each phase requires its G17 evidence and a disable-admission rollback preserving receipts.

Release decisions remain: adapter support matrix, legacy cutoff, trial dependence, normative draft pin, elicitation evolution, output-schema ownership and real mobile engine behavior. No rollout or UAR edits are authorized by publishing this proposed contract.

Suggested figure: Container + Activity → candidate → policy/data/approval → capability broker → browser-session adapter → origin/frame-bound tool → untrusted observed result → evidence verification → durable Activity. Show A2UI review at the host gate, visible browser at the adapter with takeover/resume, distinct MCP/UI-automation branches, website/session/host boundaries and verified-effect-or-uncertainty outcomes.

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
