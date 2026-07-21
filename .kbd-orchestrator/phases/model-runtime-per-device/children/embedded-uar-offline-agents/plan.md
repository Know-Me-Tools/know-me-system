# PLAN: embedded-uar-offline-agents

Project: KnowMe
Date: 2026-07-20
OpenSpec available: YES
Changes to implement: 1

## Change list

1. `embed-uar-mobile-offline`: Replace UAR's placeholder embedded SDK with a complete in-process agent runtime and inject KnowMe's local Android/iOS providers.
   - Scope: UAR core + SDK | KnowMe Rust core/FFI | Flutter mobile | desktop parity | persistence | build/certification
   - Depends on: NONE
   - Library: `cand-001` adapt, `cand-002` adopt, `cand-003` adopt, `cand-004` adopt
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Implement one transport-independent UAR kernel, direct SDK operations, host LLM provider registration, and durable offline state. Connect Android LiteRT-LM and iOS Metal through one shared UAR driver contract, then prove desktop and both mobile targets without connectivity.

## Execution milestone order

1. UAR kernel: failing tests first, then real builder/direct SDK and dependency repair.
2. Host provider: KnowMe `InferenceProvider` ↔ UAR `LlmDriver` plus local provider/model registration.
3. Persistence/control plane: durable embedded records and direct administration/A2UI APIs.
4. Client routing: mobile embedded-first Flutter repository/FFI path, desktop embedded parity, optional service/BYOK preserved.
5. Certification: UAR/KnowMe tests, Android+iOS cross-compilation, desktop, physical Android, and iOS offline functional evidence.

## Ordering rationale

The embedded runtime must be executable and testable before client routing changes; otherwise the UI would only replace one placeholder with another. Provider injection precedes persistence/client wiring because it proves the central offline agent loop. Platform certification comes last but every preceding milestone includes target checks to expose non-mobile dependencies early.

## Trade-offs and explicit scope cuts

- Mobile offline mode supports in-process/native tools; process-spawn and remote MCP resources remain optional and visibly unavailable without their capabilities.
- The first durable backend is selected by cross-compilation evidence, not by assuming the server's SurrealDB profile is mobile-safe.
- This change does not replace either platform's chosen model engine; it makes both engines first-class UAR providers.

## Command

`/opsx:apply embed-uar-mobile-offline`

PLAN COMPLETE
