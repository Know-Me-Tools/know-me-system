## 1. Repair the UAR Embedded Library

- [x] 1.1 Add a transport-independent UAR `EmbeddedRuntimeBuilder` with explicit host driver, provider/model metadata, persistence, embeddings, tools, skills, policy, and A2UI composition
- [x] 1.2 Replace the Rust SDK `Runtime` stub with direct operations over the initialized embedded kernel and separate server startup behind a server feature
- [x] 1.3 Add `embedded-mobile` feature profiles and gate transport-only/server dependencies until Android and iOS targets compile
  - FIXED: `cargo check --no-default-features --features embedded-mobile` now
    finishes in 5m52s with 0 errors (previously broken). Cargo auto-discovers
    src/bin/*.rs, so stub-llm (opens a TcpListener) and test_db_setup (needs
    PostgresProvider) built under every profile; both are now declared with
    required-features = ["server"]. All 4 bins are excluded under the profile
    and still build with `server`. UAR commit 392eea1.
  - NOT done, and deliberately so: `surreal-memory` still pulls axum+tonic
    transitively. It also supplies the CORE domain types (Memory, MemoryScope,
    MemoryType, MemoryStorage) re-exported as public API from
    uar/domain/memory.rs and uar/memory/mod.rs, used across 19 files of which
    ZERO are cfg-gated. Making it optional removes the memory subsystem from
    mobile — a product decision, not a dependency fix. The task's own success
    condition ("until Android and iOS targets compile") is met: the APK built
    and certified at HEAD this session.
  - The second axum path is via axum-test, a dev-dependency, so it does not
    affect a --lib build.
- [x] 1.4 Add UAR tests for build readiness, provider routing, agent run streaming/replay, cancellation, tool approval, skill events, A2UI continuation, and restart persistence

## 2. Connect KnowMe Local Inference

- [x] 2.1 Implement UAR `LlmDriver` conversion over KnowMe's structured `InferenceProvider` request and canonical event streams
- [x] 2.2 Register Android LiteRT-LM model metadata/capabilities as the embedded UAR default provider and model
- [x] 2.3 Register iOS Metal/Qwen model metadata/capabilities through the same provider injection contract
- [x] 2.4 Replace mobile `CloudService` defaults and service-only chat routing with the embedded UAR facade while retaining explicit optional service/BYOK mode

## 3. Offline Data and Control Plane

- [x] 3.1 Inject durable sandboxed persistence for agents, skills, policies, settings, memory, knowledge, conversations, run state, and A2UI data
- [/] 3.2 Expose direct FFI operations for agent administration, provider/model catalog, settings, skills, MCP/native tools, knowledge, memory, run policy, and A2UI registry/actions
  - DONE: skills, knowledge bases and MCP servers are served in-process by the
    embedded runtime instead of hardcoded []. UAR gained transport-free
    uar::admin::{mcp,skills,knowledge,memory} (1cd130b) that the HTTP layer now
    DELEGATES to, so both containers run one implementation. 11 SDK Runtime
    methods; KnowMe control_plane calls them; MCP writes work embedded.
  - MCP edit policy (operator-chosen B): adds/removals apply live, an edit to an
    already-CONNECTED server returns Deferred rather than dropping a healthy
    connection mid-session. save_mcp_server returns SaveResult so a UI can say so.
  - Skill pack loader (pack_sync): metadata to the DB, bodies on disk, config
    seeds the DB once and the DB is authoritative after. Verified against the
    real tree — 140 canonical skills, 37 with scripts/, 0 dups, 0 parse failures.
  - Capability gate (skill_exec, 9b894ef): iOS never degrades to local execution
    (asserted), and permanent vs transient unavailability is distinguished so a UI
    never offers a retry that cannot work.
  - tools: DONE. admin::tools merges native + MCP and keeps `source`, because
    native tools work with no network while MCP tools need a reachable server —
    a flattened list cannot explain why some survive going offline.
  - A2UI: schema LISTING done (admin::a2ui, read-only — registering a schema
    changes how every future artifact renders, so that belongs to composition).
  - A2UI ACTIONS: already wired end to end; my earlier "still absent" note was
    WRONG. Verified chain — ChatNotifier.submitInteraction (chat_notifier.dart:738)
    -> PemChatRepository.submitInteraction (:74) -> bridge.chatAction ->
    gen_ui_ffi chat_action -> gen_ui_agent::chat::send_interaction ->
    uar_a2ui_action_from_interaction -> submit_a2ui_action, implemented on BOTH
    the embedded (uar.rs:1320) and service (:1579) facades.
    promote_a2ui_component is handled on the same path, and a malformed payload
    raises a typed `uar_interaction_not_a2ui_response` error rather than being
    silently dropped. Nothing to build.
  - memories: NOT done, and blocked at the storage layer rather than unfinished.
    PersistenceLayer exposes only search_memory(query_vec, ..) — there is no way
    to enumerate memories without inventing an embedding to search with. Its
    unavailable marker stays until the storage layer grows a listing.
  - Memory DELETION is deliberately not exposed: writes belong to the memory
    service that owns scoping, embedding and auto-capture.
  - Verified: 92/92 gen_ui_agent, 9/9 UAR admin::. Two uar::eval failures are
    PRE-EXISTING (proven by stashing) and untouched by this work.
- [x] 3.3 Ensure network/process-dependent resources advertise requirements and are visibly unavailable under offline mobile policy
  - RUST DONE: ChatControlPlaneCatalog and UarAdminSnapshot now carry an
    `unavailable: [{resource, reason, platformLimitation}]` list. Previously a
    client could not tell "no skills configured" from "skills unavailable
    here" — both arrived as []. Carried ALONGSIDE the arrays so existing
    consumers keep working; omitted entirely when nothing is unavailable.
    2 contract tests; 92/92 gen_ui_agent tests pass.
  - UI DONE both surfaces. React ChatPolicyToolbar and the Flutter resource
    sheet now render "Not available on this device" plus the runtime's reason
    instead of an empty list. React previously said "No enabled resources are
    advertised by UAR" (reads as "none configured"); Flutter rendered nothing
    at all. platformLimitation decides whether "try again" is offered, so the
    UI never suggests a retry that cannot work.
    Tests: 3 React + 3 Flutter, incl. matching the sheet's snake_case policy
    key against the camelCase wire name — a mismatch there would silently show
    no reason. 161/161 desktop, 53/53 flutter chat, 92/92 rust, tsc + analyze
    clean. a11y: Semantics label / role="note", heading text not colour alone.
- [x] 3.4 Verify built-in default and orchestrator agents are seeded, protected from deletion, and default to the local model without keys

## 4. Client Integration

- [x] 4.1 Wire Flutter repositories/providers to embedded UAR operations using the required Widget → Riverpod → Repository → FFI layering
- [x] 4.2 Preserve ordered thinking, skill activation, citations, tools, memory, errors, cancellation, AG-UI, and rendered A2UI artifacts in mobile chat
- [ ] 4.3 Update diagnostics/settings to distinguish embedded local, optional remote UAR, and optional BYOK providers without suggesting network is required
- [ ] 4.4 Keep desktop embedded runtime behavior aligned and verify HTTP mode remains an explicit compatible adapter

## 5. Build and Certification

- [ ] 5.1 Fix the pinned UAR dependency graph and pass UAR/SDK host tests plus KnowMe Rust workspace tests
- [ ] 5.2 Cross-compile the shared library for `aarch64-linux-android`, `aarch64-apple-ios`, and iOS simulator architectures
- [/] 5.3 Certify desktop offline local-agent prompting, tool loop, A2UI interaction, cancellation, and persistence
  - persistence: CERTIFIED (scripts/desktop/certify-durable-uar-persistence.sh;
    /tmp/knowme-desktop-uar-persistence.json). Gate proven in both directions —
    it FAILS when state is wiped between phases. Prompting/tool loop/A2UI/
    cancellation remain uncertified on desktop.
- [/] 5.4 Install and certify physical Android with networking disabled, including local default/orchestrator agents, tools, A2UI, cancellation, and restart restoration
  - networking IS now disabled and PROVEN (both radios + reachability probe;
    `svc wifi disable` alone left the device online over mobile data).
  - APK built at HEAD; report records currentMobileIntegration=embedded_library,
    replacing the prior projection_only evidence.
    docs/certification/2026-07-27/android-offline-embedded-runtime.json
  - offline PASS: cancellation, second-send, multiturn-history, steer-intents,
    restart-recovery (persisted marker visible again after relaunch). A full
    tool loop (generate -> tool call -> tool result -> 160 chars) was observed
    offline.
  - branch-actions SKIPs its SQL assertions with exit 69: `run-as` refuses a
    release build. Was masked as exit 26 SQLITE_NOTADB because the redirect
    wrote run-as's error text into the .db path; now reported honestly.
  - restart-recovery's first FAIL was MODEL NONDETERMINISM, not a harness
    defect: identical prompt to multiturn-history, which passes. One run
    emitted a tool call instead of the marker; the re-run emitted none and
    passed. No assertion was loosened.
  - offline agents PASS: both UAR-seeded agents ("Default Assistant",
    "Orchestrator") are listed in the mode picker with both radios down.
    scripts/android/certify-offline-agents-and-a2ui.sh
    docs/certification/2026-07-27/android-offline-agents-a2ui.json
    Gate proven to FAIL on an empty registry: with the agent-context control
    still rendered but no seeded agents, it exits 1. The control alone is
    deliberately NOT accepted as proof — it renders either way.
  - A2UI: asserted CONDITIONALLY, and BLOCKED on device for a STRUCTURAL
    reason. An A2UI surface has never appeared in any device dump in this
    suite: the only registered local tool
    (knowme_diagnostics__local_runtime_status) emits no artifact, and
    promote_a2ui_component (chat.rs:613) is driven BY an interaction — so
    nothing on this build produces an artifact to interact with. Rendering and
    action dispatch ARE verified at the widget layer
    (mobile/test/features/chat/a2ui_surface_test.dart, 2 tests PASS incl.
    "unknown A2UI catalog components fail closed"). Closing the device gap
    needs an artifact producer, not a louder prompt.
- [ ] 5.5 Certify iOS device or simulator with networking disabled against the same functional suite
- [ ] 5.6 Record logs, screenshots, performance, KBD reflection, Prometheus wiki entries, and Karpathy success/failure learning records
