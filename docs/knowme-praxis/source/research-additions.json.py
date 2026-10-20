# Executed by build_praxis.py; proposed clauses grounded in the linked primary documentation.
ADDITIONS = {
'home': r'''
### HOME-08 — Activity is the durable unit of attention

An Activity survives individual model runs and media sessions. Store `activity_id`, container, objective, owner, policy revision, current checkpoint, linked run IDs, pending decisions, host receipts and terminal outcome. Use explicit states: draft, queued, running, waiting for input, waiting for approval, waiting for dependency, waiting for host, pause requested, paused, cancellation requested, cancelled, failed, completed and unknown/reconciling. Waiting is not failure. A cancellation request is not acknowledgement. A completed model turn is not a completed Activity.

Cards summarize Activities from authoritative records. The user can inspect the exact checkpoint and outstanding decision, resume with preserved context, or reconcile an uncertain host outcome. Do not invent progress percentages from token generation. A result card links to the artifact revision and receipt that establish its status.

### HOME-09 — Suggestions and drafts have their own contracts

A suggestion stores its provenance, freshness, prerequisites, permitted context, intended placement and draft text. Stable ordering preserves keyboard focus while background queries refresh. “Dismiss this suggestion” differs from “do not suggest this kind of work.” Never infer sensitive facts on a shared display. A disabled prerequisite explains the missing capability without executing installation.

One composer owns the active draft. Empty work centers it; populated work can dock it at the bottom, as requested for this design. A compact top composer was considered in the upstream home research; it remains an optional future layout variant rather than changing the selected center/bottom convention. Preserve drafts separately per container/thread. Failed sends retain the draft and attachment handles. IME composition does not trigger send; double submission uses an idempotency key. Changing destination shows the resulting scope before sending. Expose placement, attachment access and billable model selection when they affect the decision.

### HOME-10 — A complete screen inventory

Acceptance must exercise these distinct layouts, not merely recolor one screenshot: (1) empty home with no dot; (2) returning home with a decision, activity and artifact; (3) calm home with no urgent work; (4) partial dashboard failure; (5) offline home; (6) empty container with centered composer; (7) active thread with bottom composer; (8) docked canvas; (9) expanded canvas; (10) phone thread, navigation and canvas routes; (11) dirty form with remote conflict; (12) unsupported catalog; (13) blocked plugin; (14) activity running, interrupted, offline and uncertain; (15) approval expired or resolved elsewhere; (16) pause requested versus acknowledged; (17) all private-dot lifecycle states; (18) personal versus project memory; (19) explicit cloud choice and unavailable local model; (20) every terminal outcome; (21) export, notification and offboarding settings; and (22) keyboard navigation, 200% text and compact screens.

The specimens in this paper illustrate the central layout only. This publication's render checks do not claim implementation of the entire screen inventory.
''',
'api': r'''
### API-06 — Durable activities and interrupted runs

AG-UI is an adapter for observable agent events. Under the researched AG-UI 1.0 HTTP+SSE transport, `Last-Event-ID` is not a general durable resume contract. Praxis therefore owns event cursors, snapshots and reconciliation above the transport. An interrupt ends a run; continuing starts a new run attached to the same thread and Activity. Resolve or explicitly abandon all outstanding interrupts before continuation. Reject expired approvals and validate that the approved candidate still matches the current effect digest. Activity messages describe rendered activity and are not silently injected into the next conversation input.

Apply state deltas against a declared base revision. A missing predecessor or failed JSON Patch pauses application of subsequent patches and requests a fresh authoritative snapshot. Preserve local dirty fields during that recovery. Store server cursors separately from producer-local sequence numbers. Persist state before acknowledging acceptance, and distinguish an accepted command from a committed effect.

A mutation envelope includes command ID, Activity ID, actor, scope, expected revision, schema version, payload, policy revision, correlation ID and causation ID. A receipt includes the accepted command identity, execution attempt, authoritative outcome, resulting revision, effect digest and uncertainty. A timeout requires lookup by command ID before retry; it never grants permission to issue a new effect.
''',
'plugins': r'''
### PL-09 — A small portable event profile

The existing typed hook, contributor and app-shell worlds remain supported dialects. A proposed minimal event profile can simplify new portable plugins without claiming ABI compatibility with those worlds. Its adapter is versioned and tested. One entry point consumes an event and returns bounded declarative commands. Long work runs through host-managed jobs and later completion events; it does not hold a reentrant callback open across an unbounded host operation.

```wit
package knowme:portable-event@0.1.0;
interface runtime {
  enum error-code { invalid-input, unsupported, failed }
  record plugin-error { code: error-code, message: string }
  record event {
    id: string,
    correlation-id: option<string>,
    kind: string,
    schema: string,
    payload: list<u8>,
  }
  enum command-kind { result, ui, host-request, checkpoint, log }
  record command {
    id: string,
    kind: command-kind,
    schema: string,
    payload: list<u8>,
  }
  handle: func(input: event) -> result<list<command>, plugin-error>;
}
world plugin { export runtime; }
```

This WIT is illustrative and has not been compiled as part of this document. The separate package name makes its proposal status explicit. Payloads use UTF-8 JSON under pinned schemas with size and nesting bounds; large assets use scoped handles. Validate before deserialization into domain types. Reject unknown privilege-bearing fields. An intermediary may preserve unknown extension fields only where the schema explicitly permits them and cannot interpret them as grants.

A request refers to a declared capability alias such as `own-store`. The host resolves that alias to the current caller's admitted capability and grant. The guest cannot supply an actor or tenant identifier to acquire different authority. Common host services are compare-and-swap storage, scoped assets, connector operations, jobs, model requests, network requests, surface updates and host confirmations. Do not expose a generic shell or raw credential getter through a convenience service.

### PL-10 — Seven independent versions

Track package version, manifest schema, WIT ABI/world, payload schema, UI catalog profile, persisted state schema and runtime/compiler fingerprint independently. A single semver string cannot represent all seven compatibility decisions. Installation verifies each required axis before migration, records the admission result and leaves an exportable prior state on failure. A package signature covers the exact manifest and resource hashes; generated browser output is a separately verified build artifact derived from the component.

A plugin declares locality as device-required, device-preferred or remote-allowed. Remote execution is a separate placement decision with user-visible data consequences. Missing device execution support does not silently become a cloud upload. Headless and full-UI profiles are separate; a headless host rejects UI-only assumptions with a structured unsupported result. No installation scripts or arbitrary native dynamic libraries are part of the portable package.

### PL-11 — Reference plugins and conformance

Certify three small reference implementations before a marketplace: a pure transform with no host imports, a container CRUD application with declarative UI, and an asynchronous connector/agent workflow with cancellation and receipts. Run each supported profile against the same schema fixtures, denied capability attempts, restart checkpoints, duplicate events, unknown catalog components, state migration and tampered artifacts. Record the exact WIT, runtime, catalog and platform build; “supports Wasm” is not a conformance result.
''',
'runtimes': r'''
### RT-09 — Runtime support is an admission decision

Upstream Wasmtime platform documentation distinguishes execution support and support tiers. The researched iOS/Android entries are not a promise of production parity with desktop tier-one targets. AOT compilation does not by itself solve an iOS executable-memory or downloaded-code constraint: deserializing a native compiled module still depends on runtime and platform behavior. Never accept an untrusted `.cwasm` blob; native compiled artifacts require authenticated provenance and an exact compatible engine fingerprint. Wasmtime's Pulley is an internal portable interpreter bytecode, not a universal substitute for component packaging or store approval.

WASI 0.3 ratification, a runtime's experimental implementation and a released production feature set are different facts. The initial Praxis profile should pin a tested WASI Preview 2 contract and host-managed asynchronous jobs. Evaluate newer async/component features through explicit compatibility fixtures. The inspected KnowMe dependency is Wasmtime 46; an upstream release observed during research does not authorize silently upgrading the live project. Freeze an exact supported patch and security review at implementation time.

The browser profile may use `jco`-produced JavaScript and core Wasm in a worker, with an allowlisted import adapter and no ambient DOM authority. A worker improves responsiveness; it does not independently confine every imported function or shared origin capability. Treat generated glue, host imports, asset origins and CSP as part of the trusted computing base. A browser-only component must still pass schema, grant and resource limits.

### RT-10 — Native FFI has ownership, not magic portability

Flutter Rust Bridge transports typed domain calls and subscriptions. It is neither a Wasm sandbox nor permission to run downloaded native code. Pin bridge generation/runtime versions together, generate reproducible bindings, translate errors without leaking secrets, specify buffer ownership and dispose subscriptions at the correct lifecycle boundary. Rust's default ABI is not a stable dynamic-plugin ABI; arbitrary Rust dylibs are outside the portable plugin contract. Native extensions require a reviewed compiled application update and stable explicit boundaries.

Host-enforced budgets cover component linear memory, tables, stack, compute/fuel, deadline, event input/output, pending jobs, network bytes and destinations, model tokens/cost and process RSS. A linear-memory limiter does not bound all host allocations. Fuel and epoch interruption do not cancel a blocking host call: each host operation owns its timeout, cancellation, concurrency and cleanup. A separate process adds containment for untrusted workloads and crash recovery. Test quota cleanup after timeout rather than only successful calls.

### RT-11 — Store policy and distribution gates

Apple App Review 2.5.2 and the applicable 4.7 rules require a distribution review for downloaded software, mini applications, access to native APIs, per-miniapp consent, indexing/deep links, content/age restrictions and commerce. A declarative catalog or remote execution does not automatically exempt an application. Google Play restrictions on downloaded dex/JAR/native libraries and its interpreter/VM conditions still require policy compliance, including indirectly invoked platform APIs. These are engineering admission gates based on primary policy text, not a guarantee of store approval.

The default mobile release ships audited native catalogs and host services. A downloaded component profile remains unavailable until device behavior, resource limits, security review and the relevant distribution policy are documented. The installation UI explains unsupported profiles and offers only already-authorized alternatives.
''',
'surfaces': r'''
### UI-09 — Catalog and action semantics

Pin the A2UI protocol version separately from renderer and catalog versions. Research identifies A2UI 0.9.1 support in multiple renderer families; a Flutter GenUI SDK alpha status does not mean the same thing as the protocol's maturity. Catalog identifiers are identifiers, not URLs that a client fetches and executes. The allowlisted catalog includes trusted functions as well as components. Universal web components based on Lit are not automatically native Flutter widgets.

Use shared semantic families for layout, content, input, work status, decisions and navigation. Define native/web mappings and loading, empty, error, disabled, stale, offline, large-text and accessibility behavior for every component. Streamed unresolved children remain inert until valid; malformed or unsupported nodes show recoverable errors. Action payloads identify the surface revision, component, semantic action and schema. The host confirms privileged actions using its own labels, candidate details and authority checks.

Returning an entire data model with an action is optional and risky: it can include hidden fields or stale private context. Prefer explicit field allowlists and purpose-scoped action context. Dirty local values survive server patches; base-revision conflicts request resolution rather than overwriting silently. Closing a surface changes layout only; it neither cancels an activity nor deletes business data.

### UI-10 — Embedded applications and static document safety

An embedded iframe/webview is a separate capability tier with a narrow, origin-checked bridge. Declare clipboard, popups, downloads, camera, microphone and navigation independently. Do not copy another protocol's sandbox flags without evaluating the threat model. A hidden iframe is not a trusted service and a visually hidden participant is not a private call.

HTMX fragments are host-authored, escaped templates. Model output does not become HTML or `hx-*` instructions. Keep canonical links and form semantics so reading/navigation survives JavaScript failure. Coordinate concurrent requests so a stale refresh cannot overwrite a newer surface or dirty form. Sensitive pages use appropriate history/cache/logout rules, including `hx-history="false"` where required; that attribute alone does not secure all browser caches. Modal dialogs trap focus, make background controls inert and restore focus to the invoker. Announce bounded state changes, not every generated token. Preserve keyboard access, visible focus, reflow and reduced-motion preferences.

### UI-11 — Native continuity and notifications

Persist selected IDs and semantic navigation state, not fragile pixel widths. Recompute layouts for safe areas, folding displays, virtual keyboards and text scale. Back navigates; cancellation is an explicit operation. Offline storage preserves drafts but does not preserve stale authorization for high-impact effects. Revalidate upon reconnect before execution.

Notifications carry opaque references and minimal privacy-safe hints. Opening one fetches current authorized state; it is not evidence that an action remains pending or a delivery guarantee. Respect quiet periods, category controls and lock-screen privacy. Mobile push delivery, force-quit behavior and background processing are bounded platform capabilities. Do not promise continuous local inference or indefinite background microphone access.
''',
'realtime': r'''
### RTC-12 — Control plane services and room identities

Use a Rust control plane with IdentityService, ConversationService, RtcGateway, AgentOrchestrator, PolicyGateway, ConsentService, EventStore/Outbox, ArtifactService and UsageService. ConversationService owns a room epoch so a recreated room cannot accidentally inherit old authorization or event cursors. AgentOrchestrator stores dispatch intent, a lease and a fenced generation. Python LiveKit Agents workers are the initial integration target; Node workers are supported by upstream, while a custom Rust participant is not automatically feature-equivalent to the Agents framework.

Flutter owns LiveKit capture, playback, rendering and platform AudioManager integration. Rust FFI carries domain state and control, not a custom per-frame audio/video loop. If later DSP requires shared buffers, specify sample format, time base, ownership, bounded buffering, cancellation and disposal before adding that path.

Separate user ID, device ID, login session ID, room ID, room epoch and participant identity. Reusing a participant identity can evict another connection. Mint unique opaque participant identities for device/session membership. A join token permits only the required room, publish/subscribe/data rights and track sources; it never grants room administration to a normal client. Token expiry is not an active-session kill switch. Removal, refusing reissue and provider-specific revocation are distinct controls, and self-hosted deployments must test whether an old token can rejoin.

Three mechanisms must agree: token grants, publisher subscription permissions and local subscriber choice. Default room subscription behavior can expose tracks. `autoSubscribe=false` or hiding a track in the UI is not confidentiality. Test existing, newly published and republished tracks after permissions change. Dispatching an agent into an existing room requires explicit dispatch; token-based dispatch on room creation is not the same operation.

### RTC-13 — Topologies with explicit audiences

Human-only calls have no hidden agent. A one-to-one personal-dot call binds the owner to the agent. An invited shared agent uses meeting-authorized context and a visible roster identity. Multiple agents require a speech-floor lease with human preemption. An internal agent handoff does not imply multiple independently audible participants.

A private companion starts in text or a separate room. Enforce one active audio destination. Confirm shared publishing has stopped before sending private speech; headphones or text reduce physical speaker/microphone leakage. A personal agent hearing a group call requires the group's applicable processing consent. Personal memory is never copied into the shared room merely because its owner is present. Late joiners receive only authorized retained history.

### RTC-14 — Dispatch, turns and interruption

The agent lifecycle is requested → authorized → dispatching → joining → initializing → ready → draining → ended. Track degraded, reconnecting, failed and cancellation-requested conditions separately. Persist dispatch through an outbox, use compare-and-swap generation fencing and acknowledge readiness only when model/provider and subscribed/published tracks are ready. Reconcile a timed-out dispatch before creating another worker. Cleanup must stop orphan jobs and their billing.

Default single-participant RoomIO assumptions do not solve multiparty coordination. Bind a one-to-one session explicitly to its owner. For multiparty, maintain participant-track identity, timestamps, overlap and turn policy in a coordinator. Select one primary turn authority: provider server detection, LiveKit client turn detection, or manual push-to-talk. Do not run competing authorities that commit duplicate turns.

Handle short backchannels, overlapping speech, barge-in, delayed audio and echo. Distinguish generated text from text actually played to listeners. Stopping speech does not undo a tool effect already committed. Speculative generation must not produce speculative external effects. STT diarization labels are not authenticated participant identities. Preserve both the room identity source and any diarization uncertainty.

### RTC-15 — Video and consented perception

Publishing a camera track does not authorize sending frames to a model. Store the purpose, recipient/model route, audience and retention decision for vision separately. Select sampled frames under explicit bandwidth/cost limits and display when vision is enabled. Upstream frame sampling defaults and provider session limits are version-specific observations; freeze measured values with the chosen SDK/provider build. Passive camera publication alone does not trigger an inference. A provider session can roll over while the LiveKit room persists; preserve conversation state and consent without duplicating effects.

### RTC-16 — Proposed API and event contract

```text
POST   /v1/conversations
POST   /v1/conversations/{id}/join
POST   /v1/conversations/{id}/agents
DELETE /v1/conversations/{id}/agents/{binding}
POST   /v1/conversations/{id}/consents
POST   /v1/tool-runs/{id}/approvals
POST   /v1/tool-runs/{id}/cancel
GET    /v1/conversations/{id}/events?after={cursor}
GET    /v1/conversations/{id}/snapshot
```

Join returns room epoch, LiveKit URL, scoped short-lived token, participant identity, expiry, policy revision, effective permissions, agent bindings, recording/processing status, durable cursor and negotiated capabilities. Deliver encryption keys separately through an authenticated authorized channel. Never put signing secrets in a client response.

An event envelope carries event ID, room epoch, Activity ID, producer identity and sequence, durable server cursor, schema version, timestamp, audience scope, policy/consent revisions, correlation and causation. Families cover call, participant, track, agent, utterance, transcript, response, playout, floor, tool, privacy and usage. Partial transcripts are ephemeral unless explicitly retained; final transcript storage requires its own policy. Missing intervals are shown, not reconstructed as if observed.

LiveKit text streams have no durable history and late joiners miss prior/open streams. Reliable data packets do not create offline durable delivery. RPC has payload/time bounds (the researched default is 15 KiB and 10 seconds); long-running work returns an operation ID and completes through the control plane. Media data channels can carry low-latency hints, while application receipts remain authoritative.

### RTC-17 — Encryption, recording and redaction

Human-only E2EE, agent-readable encryption and recording are distinct modes. A participating AI endpoint receives plaintext by design; describe that boundary honestly. Rotate key epochs after membership changes and test removal. Modern SDK `encryption` settings and deprecated `e2ee` options have different data coverage; pin and test the complete media/data configuration. Signaling and service APIs remain TLS-protected service-visible channels. Do not assume an Egress recording path can record an encrypted room without a separately designed key/consent path.

For Agents session recording, explicitly configure recording off for the private baseline. The researched API fixes recording options at session start; omitting the option can inherit project settings, and omitted granular recording keys may remain enabled. Verify configuration with a receipt rather than trusting an empty options object. LiveKit Cloud PII redaction is best effort and does not automatically cover application history or Egress. Realtime audio redaction has timing limitations even where transcript redaction is available. Keep consent and retention controls independent of this optional processing aid.

Consent is per participant, purpose and audience. Rejection remains usable where possible. A mode change requiring more access pauses that processing until the relevant decision arrives; it does not silently downgrade privacy. Recording, transcription, agent processing, vision and memory persistence each have visible states and audit records.

### RTC-18 — Mobile audio, recovery and economics

Request microphone/camera permission at the relevant action and offer receive-only or text alternatives. iOS audio-session interruptions and Android foreground-service/background-start restrictions govern behavior. A foreground user call can have permitted background continuation; this does not justify an always-listening private dot. Test wired/Bluetooth routes, speaker switching, phone calls, camera interruption, process death and foreground/background transitions on devices.

Recover transport health separately from application membership and agent health. Verify signed webhook payloads against their original bytes, deduplicate by event identity and durably ingest before acknowledging. Webhooks have finite delivery and need reconciliation; they do not replace periodic state repair. Test network switching, TURN-only networks, revoked membership, provider outage, worker crash and model maximum-session rollover.

Proposed performance targets are warm agent ready p95 under 3 seconds, first audio under 1.5 seconds and interruption playback stop under 300 ms. They are targets, not measured results. Budget room media, agent minutes, inference, STT/TTS, vision, Egress, storage and application work together. Enforce tenant/room/binding/model budgets, idle timeout, maximum duration, frame rate and tool limits. Published plan prices, included minutes and concurrency change; a dated commercial snapshot is not a contractual entitlement or a provider-model quota.

### RTC-19 — Delivery phases and tests

Phase zero pins versions and proves compatibility/security on actual devices. Phase one delivers human calls and one personal dot. Phase two adds an explicitly addressed shared agent. Phase three adds private sidecars and then optional voice/multiagent arbitration. Phase four adds vision, governed recording, retention and regional controls.

The gate includes duplicate identities, unauthorized track subscription, newly published tracks after removal, private-memory leakage, consent refusal/revocation, E2EE membership rotation and vision-off behavior; speech overlap, backchannels, stop requests, echo and delivered-versus-generated transcript; native permission/audio-route/network transitions; duplicate/timed-out dispatch, fenced workers, outbox replay, uncertain tool effects, model limits and orphan billing. Unit simulations supplement device and multiparty tests; they do not substitute for them.
''',
'acceptance': r'''
### Detailed acceptance families

Home tests assert honest freshness and failure states; permission-safe counts; one focused composer; draft-preserving failed sends; suggestions that only prepare drafts; stable focus during refresh; restore behavior; and all 22 screen states. Workspace tests assert context switching, artifact revision identity, canvas navigation without unintended cancellation, legacy prefill without saving, and layout persistence.

A2UI tests cover protocol/catalog negotiation; ordered updates; unresolved-child handling; unknown components; schema-invalid actions; host-owned approvals; malicious text/URLs; explicit action context; dirty-field conflicts; snapshot recovery; native/web parity; and disabled/revoked-plugin recovery. Run tests cover interrupt completion and continuation, multiple outstanding decisions, expiry, activity/run separation, unknown outcomes, cursor gaps and failed patches. Dot tests cover absent/setup/active/paused/offline/revoked, child cancellation acknowledgement, scoped memory, export/deletion and healthy-coach silence.

Plugin tests cover all seven version axes, locality refusal, no silent cloud fallback, hash/signature/namespace rejection, undeclared imports, grant expiry/revocation, guest identity spoofing, quota exhaustion, blocking host I/O, cancellation cleanup, state migration, browser glue confinement and mobile distribution admission. Mobile tests cover safe area/keyboard/reflow, process restoration, audio routing, notification freshness and offline authorization. Browser tests cover escaped HTMX fragments, race control, sensitive history/cache handling and progressive navigation. Accessibility tests cover keyboard, screen reader, focus visibility/return, large text and reduced motion. Every pass records build/device/profile and every unsupported case remains explicit.
'''
}
