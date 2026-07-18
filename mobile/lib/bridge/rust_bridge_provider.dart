// TJ-ARCH-MOB-001 compliant
// The FFI facade. Every call here delegates straight to the frb-generated
// bindings (gen_ui_core, Rust) — this file is a pass-through, no networking/
// inference/persistence logic belongs here.
import 'dart:async';
import 'dart:convert';

import 'a2ui/a2ui_event.dart' as bridge_events;
import 'api/boot.dart' as ffi;
import 'api/chat.dart' as ffi;
import 'api/entity.dart' as ffi;
import 'api/streams.dart' as ffi;
import 'frb_generated.dart' show GenUiCore;
import 'third_party/gen_ui_db_graph.dart' show MemoryHit, RelatedEntity;
import 'third_party/gen_ui_types/transport.dart' as transport_wire;
import 'third_party/gen_ui_types/view.dart' as view_wire;
import 'package:prometheus_entity_management/prometheus_entity_management.dart';

export 'third_party/gen_ui_db_graph.dart' show MemoryHit, RelatedEntity;

Future<void> initRustBridge({String? dataDir}) async {
  await GenUiCore.init();
  final _ = dataDir; // boot.dart's runMigrations takes dataDir directly.
}

/// chat_send(thread_id, message) -> run_id. FFI call; terminal on Rust error
/// (thrown as a Dart exception — see gen_ui_ffi/src/api/chat.rs's doc comment
/// on why CoreError, not CoreResult, is the literal return-type spelling).
Future<String> chatSend(
  String threadId,
  String message,
  List<String> messages,
) =>
    ffi.chatSend(threadId: threadId, message: message, messages: messages);

/// chat_events(run_id) -> Stream<A2uiEvent>. Fold into ContentBlocks.
/// The FFI stream carries JSON (see gen_ui_ffi::api::streams's doc comment on
/// why — frb can't mirror an enum only ever seen as a StreamSink<T> param),
/// decoded here into the app's own sealed A2uiEvent using the same serde tag
/// shape ("type", snake_case) as gen_ui_types::events::A2uiEvent.
Stream<bridge_events.A2uiEvent> chatEvents(String runId) => ffi
    .chatEvents(runId: runId)
    .map((json) => bridge_events.A2uiEvent.fromJson(jsonDecode(json)));

/// entity_changes() -> Stream<ChangeEvent>. Bridged to ref.invalidate by PEM.
Stream<ChangeEvent> entityChanges() =>
    ffi.entityChanges().map((json) => ChangeEvent.fromJson(jsonDecode(json)));

/// sync_status() -> Stream<SyncStatus>. Drives the sync chip.
Stream<SyncStatus> syncStatus() =>
    ffi.syncStatus().map((json) => SyncStatus.fromJson(jsonDecode(json)));

// ── Entity transport intents (wired into PEM's EntityTransport adapter) ──────
Future<ListResult> entityList(ViewDescriptor view) async {
  final wire = await ffi.entityList(view: _toWireViewDescriptor(view));
  return ListResult(
    items: wire.items.map(_toEntityRecord).toList(),
    nextCursor: wire.nextCursor,
  );
}

Future<EntityRecord?> entityGet(String entityType, String id) async {
  final wire = await ffi.entityGet(entityType: entityType, id: id);
  return wire == null ? null : _toEntityRecord(wire);
}

Future<EntityRecord> entityCreate(EntityRecord record) async => _toEntityRecord(
      await ffi.entityCreate(record: _toWireEntityRecord(record)),
    );

Future<EntityRecord> entityUpdate(EntityRecord record) async => _toEntityRecord(
      await ffi.entityUpdate(record: _toWireEntityRecord(record)),
    );

Future<void> entityDelete(String entityType, String id) =>
    ffi.entityDelete(entityType: entityType, id: id);

// ── Memory / graph-RAG intents (gen_ui_db::graph — never raw SurrealQL in Dart) ─
// memory_search / memory_ingest / graph_expand are the ONLY graph surface the UI
// sees. Fusion (vector recall → graph expansion → BM25 → RRF) happens in Rust.

/// memory_ingest(text) -> id. Embeds on-device (fastembed) + upserts in Rust.
Future<String> memoryIngest(String text) => ffi.memoryIngest(text: text);

/// memory_search(query, k) -> ranked hits (hybrid vector+graph+BM25 fusion).
Future<List<MemoryHit>> memorySearch(String query, int k) =>
    ffi.memorySearch(query: query, k: k);

/// graph_expand(entity_id, depth) -> neighbourhood hits (recursive RELATE walk).
Future<List<RelatedEntity>> graphExpand(String entityId, int depth) =>
    ffi.graphExpand(entityId: entityId, depth: depth);

// ── Startup orchestrator intents (gen_ui_db startup — analysis §1.8) ─────────
// Boot-order invariant: migrations → seeds → shapes. Each step runs in Rust;
// the startup feature sequences these in order and gates the app until ready.

/// run_migrations() — apply the additive schema migration set for this dialect.
/// Mobile resolves the platform data directory itself (path_provider) since
/// Rust has no portable way to ask the OS for it — see boot.rs's doc comment.
Future<void> runMigrations({required String dataDir}) =>
    ffi.runMigrations(dataDir: dataDir);

/// load_seeds() — copy bundled seed/lookup data on first run (idempotent).
Future<void> loadSeeds() => ffi.loadSeeds();

/// attach_sync_shapes() — subscribe Electric shapes AFTER migrations+seeds
/// (shapes fail on unknown columns, so this must run last).
Future<void> attachSyncShapes() => ffi.attachSyncShapes();

// ── Wire <-> PEM adapters (wire shapes are transport-agnostic; PEM owns the
// richer domain shape) ───────────────────────────────────────────────────────

EntityRecord _toEntityRecord(transport_wire.EntityRecord wire) => EntityRecord(
      id: wire.id,
      entityType: wire.entityType,
      dataJson: wire.dataJson,
    );

transport_wire.EntityRecord _toWireEntityRecord(EntityRecord record) =>
    transport_wire.EntityRecord(
      id: record.id,
      entityType: record.entityType,
      dataJson: record.dataJson,
    );

view_wire.ViewDescriptor _toWireViewDescriptor(ViewDescriptor view) =>
    view_wire.ViewDescriptor(
      entityType: view.entityType,
      filters: view.filters
          .map(
            (f) => view_wire.FilterSpec(
              field: f.field,
              op: _toWireFilterOp(f.op),
              valueJson: f.valueJson,
            ),
          )
          .toList(),
      sorts: view.sorts
          .map(
            (s) => view_wire.SortSpec(field: s.field, descending: s.descending),
          )
          .toList(),
      limit: view.limit,
      cursor: view.cursor,
    );

// PEM's FilterOp.inList renders `in` (a Dart keyword) as a distinct member
// name from the wire enum's FilterOp.in_ — same serde wire value, different
// Dart identifier on either side of the bridge.
view_wire.FilterOp _toWireFilterOp(FilterOp op) => switch (op) {
      FilterOp.eq => view_wire.FilterOp.eq,
      FilterOp.ne => view_wire.FilterOp.ne,
      FilterOp.lt => view_wire.FilterOp.lt,
      FilterOp.lte => view_wire.FilterOp.lte,
      FilterOp.gt => view_wire.FilterOp.gt,
      FilterOp.gte => view_wire.FilterOp.gte,
      FilterOp.inList => view_wire.FilterOp.in_,
      FilterOp.like => view_wire.FilterOp.like,
    };
