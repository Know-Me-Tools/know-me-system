// TJ-ARCH-MOB-001 compliant
//! Rust->Dart event streams via frb `StreamSink<T>`. Dart consumes each as a
//! broadcast `Stream` behind a Riverpod `@riverpod` stream provider (remember:
//! FFI providers must opt out of Riverpod 3 auto-retry).
//!
//! Three feeds map to the three UI chrome elements:
//!   * A2uiEvent   -> chat transcript (ContentBlock folding)
//!   * ChangeEvent -> entity cache invalidation (ref.invalidate bridge)
//!   * SyncStatus  -> the sync status chip
//!
//! C-007 lands the sink plumbing; Wave-1 lanes push real events into these sinks
//! from the agent loop (C-006) and sync engine (C-005). Until then the streams
//! open and stay idle.
//!
//! `StreamSink<T>` is the per-crate type emitted by codegen into
//! `frb_generated.rs` and brought into crate-root scope; this module compiles
//! only with the `frb-streams` feature (enabled after codegen runs). The bare
//! unqualified `StreamSink` name is the idiomatic frb 2.x form (see the frb
//! README's streaming example).
//!
//! All three sinks carry JSON `String`, not the gen_ui_types enum directly:
//! frb only mirrors a struct/enum into a real Dart class when it appears in a
//! `pub fn` argument or return type; a `StreamSink<T>` type parameter alone
//! doesn't count; the enum is emitted as an opaque handle with zero
//! Dart-visible fields instead (see docs/reference-app codegen notes).
//! gen_ui_types must stay frb-agnostic (it also compiles to wasm for Tauri/
//! React), so the fix lives here: serialize to JSON in gen_ui_ffi, and let
//! Dart's own `fromJson` (already generated on the PEM mirror types) decode
//! it — same serde tag shape on both ends.
use crate::frb_generated::StreamSink;

/// Subscribe to the A2UI event stream for a chat run. The core produces
/// ContentBlock-bearing events; the transport layer (`@flint/react`, flint_genui)
/// is bypassed by contract.
///
/// The broadcast is process-wide, not per-run (only `RunStarted`/`RunFinished`
/// carry a `run_id`; `Block` does not) — correct for this PoC's single-turn-at-
/// a-time chat, since only one run is ever in flight. Multiplexing by run_id is
/// a real gap for concurrent turns, deferred until that's an actual requirement.
pub fn chat_events(run_id: String, sink: StreamSink<String>) {
    let _ = run_id;
    let Ok(mut rx) = gen_ui_agent::state::subscribe() else {
        return;
    };
    gen_ui_runtime::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let Ok(json) = serde_json::to_string(&event) else {
                continue;
            };
            if sink.add(json).is_err() {
                break; // Dart side dropped the stream — stop forwarding.
            }
        }
    });
}

/// Subscribe to entity change events. One Dart listener fans these into
/// `ref.invalidate` calls per the PEM-Flutter cascade-invalidation design.
/// Emits `gen_ui_types::transport::ChangeEvent` JSON once Wave-1 (C-003)
/// forwards EntityTransport change notifications here.
pub fn entity_changes(sink: StreamSink<String>) {
    let _ = sink;
}

/// Subscribe to the sync status feed that drives the UI sync chip.
/// Emits `gen_ui_types::sync::SyncStatus` JSON once Wave-1 (C-005) forwards
/// SyncTransport status transitions here.
pub fn sync_status(sink: StreamSink<String>) {
    let _ = sink;
}
