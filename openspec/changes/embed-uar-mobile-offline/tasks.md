## 1. Repair the UAR Embedded Library

- [x] 1.1 Add a transport-independent UAR `EmbeddedRuntimeBuilder` with explicit host driver, provider/model metadata, persistence, embeddings, tools, skills, policy, and A2UI composition
- [x] 1.2 Replace the Rust SDK `Runtime` stub with direct operations over the initialized embedded kernel and separate server startup behind a server feature
- [/] 1.3 Add `embedded-mobile` feature profiles and gate transport-only/server dependencies until Android and iOS targets compile
  - profiles exist, but the GATING half is false: tokio(full)/reqwest/rmcp/liter-llm
    are unconditional and `surreal-memory` drags axum+tonic in
    (`cargo tree --no-default-features --features embedded-mobile -i axum` resolves).
    Two ungated bins break `cargo check` under the profile. Only `--lib` builds.
- [x] 1.4 Add UAR tests for build readiness, provider routing, agent run streaming/replay, cancellation, tool approval, skill events, A2UI continuation, and restart persistence

## 2. Connect KnowMe Local Inference

- [x] 2.1 Implement UAR `LlmDriver` conversion over KnowMe's structured `InferenceProvider` request and canonical event streams
- [x] 2.2 Register Android LiteRT-LM model metadata/capabilities as the embedded UAR default provider and model
- [x] 2.3 Register iOS Metal/Qwen model metadata/capabilities through the same provider injection contract
- [x] 2.4 Replace mobile `CloudService` defaults and service-only chat routing with the embedded UAR facade while retaining explicit optional service/BYOK mode

## 3. Offline Data and Control Plane

- [x] 3.1 Inject durable sandboxed persistence for agents, skills, policies, settings, memory, knowledge, conversations, run state, and A2UI data
- [ ] 3.2 Expose direct FFI operations for agent administration, provider/model catalog, settings, skills, MCP/native tools, knowledge, memory, run policy, and A2UI registry/actions
- [ ] 3.3 Ensure network/process-dependent resources advertise requirements and are visibly unavailable under offline mobile policy
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
  - A2UI is asserted CONDITIONALLY and is currently UNVERIFIED: a 4B local
    model is nondeterministic about emitting an artifact (same prompt gave 1
    tool call on one run, 0 on the next). If a surface renders it must carry
    its accessible name; if none is emitted that is reported as UNVERIFIED
    rather than dressed up as a pass. Settling it needs a deterministic A2UI
    fixture, not a louder prompt.
- [ ] 5.5 Certify iOS device or simulator with networking disabled against the same functional suite
- [ ] 5.6 Record logs, screenshots, performance, KBD reflection, Prometheus wiki entries, and Karpathy success/failure learning records
