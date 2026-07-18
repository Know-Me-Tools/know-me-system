// TJ-ARCH-MOB-001 compliant
/// Entry point + re-export shim for the generated bridge.
///
/// The frb-generated `GenUiCore` class (see rust/flutter_rust_bridge.yaml,
/// dart_entrypoint_class_name) is produced into the host app. Riverpod providers
/// wrap the async intent calls; FFI providers MUST opt out of Riverpod 3 auto-retry
/// (retry: (_, __) => null) so Rust Terminal errors aren't silently re-invoked.
///
/// Intent surface (mirrors crates/gen_ui_ffi/src/api):
///   * initCore(workerThreads)            — call once at startup
///   * chatSend(threadId, message) -> runId
///   * chatEvents(runId)     -> `Stream<A2uiEvent>`   (fold into ContentBlocks)
///   * entityList/Get/Create/Update/Delete
///   * entityChanges()       -> `Stream<ChangeEvent>` (bridge to ref.invalidate)
///   * syncStatus()          -> `Stream<SyncStatus>`  (drive the sync chip)
///   * memorySearch(query, k) / graphExpand(entityId, depth)
///
/// This file is intentionally a documentation + re-export seam: the concrete
/// bindings are generated, and the example app (C-010) wires the providers.
class GenUiFlutter {
  const GenUiFlutter._();
}
