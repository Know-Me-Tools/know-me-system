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
import 'third_party/gen_ui_types/inference.dart';
import 'third_party/gen_ui_types/error.dart' as error_wire;
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
  List<InferenceMessage> messages,
  List<String> attachmentPaths,
  String branchId,
  String userMessageId,
) => ffi.chatSend(
  threadId: threadId,
  message: message,
  messages: messages,
  attachmentPaths: attachmentPaths,
  branchId: branchId,
  userMessageId: userMessageId,
);

Future<String> chatAction(
  String threadId,
  String interactionJson,
  List<InferenceMessage> messages,
  String branchId,
) => ffi.chatAction(
  threadId: threadId,
  interactionJson: interactionJson,
  messages: messages,
  branchId: branchId,
);

Future<String> approveToolConfirmation(String approvalId) =>
    ffi.chatApproveToolConfirmation(approvalId: approvalId);

Future<bool> chatCancel(String runId) => ffi.chatCancel(runId: runId);

Future<void> prepareLocalModel() => ffi.prepareLocalModel();

Future<Map<String, dynamic>> localModelStatus() async =>
    jsonDecode(await ffi.localModelStatus()) as Map<String, dynamic>;

Stream<Map<String, dynamic>> watchLocalModelStatus() => ffi
    .localModelStatusEvents()
    .map((value) => jsonDecode(value) as Map<String, dynamic>);

Future<void> cancelModelOperation() => ffi.cancelModelOperation();

Future<Map<String, dynamic>?> localRuntimeDiagnostics() async =>
    jsonDecode(await ffi.localRuntimeDiagnostics()) as Map<String, dynamic>?;

Future<Map<String, dynamic>> uarRuntimeStatus() async =>
    jsonDecode(await ffi.uarRuntimeStatus()) as Map<String, dynamic>;

Future<String> activeLane() => ffi.activeLane();

Future<void> setActiveLane(String lane) => ffi.setActiveLane(lane: lane);

/// Which lanes are selectable right now, as `[{lane, available, reason}]`.
/// Lets the mode picker disable unavailable modes with a reason rather than
/// offering a control that fails when tapped.
Future<List<dynamic>> laneAvailability() async {
  final raw = await ffi.laneAvailability();
  final decoded = jsonDecode(raw);
  return decoded is List ? decoded : const [];
}

/// Resolve FRB's opaque workspace error into the safe Rust `Display` message.
/// Generated bindings intentionally stay untouched; every feature reaches this
/// through its repository layer.
Future<String> bridgeErrorMessage(Object error) async {
  if (error is error_wire.CoreError) {
    try {
      return await ffi.coreErrorMessage(error: error);
    } catch (_) {
      // Fall through to the stable generic message if the error object was
      // already consumed by another observer.
    }
  }
  final message = error.toString().replaceFirst('Exception: ', '').trim();
  if (message == "Instance of 'CoreErrorImpl'" ||
      message == 'Instance of CoreErrorImpl') {
    return 'The operation failed before its diagnostic could be decoded. Retry it or review the runtime configuration.';
  }
  return message;
}

Future<Map<String, dynamic>> uarServiceConfig() async =>
    jsonDecode(await ffi.uarServiceConfig()) as Map<String, dynamic>;

Future<void> saveUarService({
  required String serviceUrl,
  String? apiKey,
  String? sessionId,
}) => ffi.uarServiceSave(
  serviceUrl: serviceUrl,
  apiKey: apiKey,
  sessionId: sessionId,
);

Future<Map<String, dynamic>> probeUarService() async =>
    jsonDecode(await ffi.uarServiceProbe()) as Map<String, dynamic>;

Future<Map<String, dynamic>> chatControlCatalog() async =>
    jsonDecode(await ffi.chatControlCatalog()) as Map<String, dynamic>;

Future<Map<String, dynamic>> uarAdminSnapshot() async =>
    jsonDecode(await ffi.uarAdminSnapshot()) as Map<String, dynamic>;

Future<List<Map<String, dynamic>>> providerCatalog() async =>
    (jsonDecode(await ffi.providerCatalog()) as List<dynamic>)
        .cast<Map<String, dynamic>>();

Future<List<Map<String, dynamic>>> configuredProviders() async =>
    (jsonDecode(await ffi.providerList()) as List<dynamic>)
        .cast<Map<String, dynamic>>();

Future<void> saveProvider(Map<String, dynamic> request) =>
    ffi.providerSave(requestJson: jsonEncode(request));

Future<void> deleteProvider(String id) => ffi.providerDelete(id: id);

Future<Map<String, dynamic>> testProvider(String id, String? model) async =>
    jsonDecode(await ffi.providerTest(id: id, model: model))
        as Map<String, dynamic>;

Future<Map<String, dynamic>?> cloudModel() async {
  final value = jsonDecode(await ffi.cloudModelGet());
  return value is Map<String, dynamic> ? value : null;
}

Future<void> saveCloudModel(String providerId, String modelId) =>
    ffi.cloudModelSave(providerId: providerId, modelId: modelId);

Future<Map<String, dynamic>> saveUarSetting(String key, Object? value) async =>
    jsonDecode(await ffi.uarSettingSave(key: key, valueJson: jsonEncode(value)))
        as Map<String, dynamic>;

Future<Map<String, dynamic>> saveUarAgent(Object agent) async =>
    jsonDecode(await ffi.uarAgentSave(agentJson: jsonEncode(agent)))
        as Map<String, dynamic>;

Future<void> deleteUarAgent(String id) => ffi.uarAgentDelete(id: id);

Future<Map<String, dynamic>> saveUarMcpServer(
  String name,
  Object server,
) async => jsonDecode(
  await ffi.uarMcpServerSave(name: name, serverJson: jsonEncode(server)),
) as Map<String, dynamic>;

Future<Map<String, dynamic>> deleteUarMcpServer(String name) async =>
    jsonDecode(await ffi.uarMcpServerDelete(name: name))
        as Map<String, dynamic>;

Future<Map<String, dynamic>> deleteUarMemory(String id) async =>
    jsonDecode(await ffi.uarMemoryDelete(id: id)) as Map<String, dynamic>;

Future<Map<String, dynamic>> createUarKnowledgeBase(Object request) async =>
    jsonDecode(
      await ffi.uarKnowledgeBaseCreate(requestJson: jsonEncode(request)),
    ) as Map<String, dynamic>;

Future<void> deleteUarKnowledgeBase(String id) =>
    ffi.uarKnowledgeBaseDelete(id: id);

Future<Map<String, dynamic>?> conversationPolicy(String conversationId) async {
  final value = jsonDecode(
    await ffi.conversationPolicyGet(conversationId: conversationId),
  );
  return value is Map<String, dynamic> ? value : null;
}

Future<Map<String, dynamic>> saveConversationPolicy(
  String conversationId,
  Map<String, dynamic> policy,
) async => jsonDecode(
  await ffi.conversationPolicySave(
    conversationId: conversationId,
    policyJson: jsonEncode(policy),
  ),
) as Map<String, dynamic>;

Future<void> deleteConversationPolicy(String conversationId) =>
    ffi.conversationPolicyDelete(conversationId: conversationId);

Future<Map<String, dynamic>> conversationEffectiveConfig(
  String conversationId,
) async => jsonDecode(
  await ffi.conversationEffectiveConfig(conversationId: conversationId),
) as Map<String, dynamic>;

Future<String> getThemeMode() => ffi.themeMode();

Future<void> setThemeMode(String mode) => ffi.setThemeMode(mode: mode);

/// The local lane's system prompt, already resolved to the shipped default when
/// the user has not set one.
Future<String> localSystemPrompt() => ffi.localSystemPrompt();

/// Persist the local lane's system prompt; a blank value restores the default.
Future<void> setLocalSystemPrompt(String prompt) =>
    ffi.setLocalSystemPrompt(prompt: prompt);

/// The shipped default, so "reset" does not duplicate Rust's copy in Dart.
Future<String> defaultLocalSystemPrompt() => ffi.defaultLocalSystemPrompt();

Future<String> resolveRemoteContent(String url, String kind) =>
    ffi.resolveRemoteContent(url: url, kind: kind);

Future<int> deleteConversationContext(String conversationId) async =>
    (await ffi.memoryDeleteConversationContext(conversationId: conversationId))
        .toInt();

Future<int> deleteMessageContext(List<String> messageIds) async =>
    (await ffi.memoryDeleteMessageContext(messageIds: messageIds)).toInt();

Future<int> promoteContext(List<String> ids) async =>
    (await ffi.memoryPromoteContext(ids: ids)).toInt();

/// `chat_events(run_id) -> Stream<A2uiEvent>`. Fold into ContentBlocks.
/// The FFI stream carries JSON (see gen_ui_ffi::api::streams's doc comment on
/// why — frb can't mirror an enum only ever seen as a `StreamSink<T>` param),
/// decoded here into the app's own sealed A2uiEvent using the same serde tag
/// shape ("type", snake_case) as gen_ui_types::events::A2uiEvent.
Stream<bridge_events.A2uiEvent> chatEvents(String runId) => ffi
    .chatEvents(runId: runId)
    .map((json) => bridge_events.A2uiEvent.fromJson(jsonDecode(json)));

/// `entity_changes() -> Stream<ChangeEvent>`. Bridged to ref.invalidate by PEM.
Stream<ChangeEvent> entityChanges() =>
    ffi.entityChanges().map((json) => ChangeEvent.fromJson(jsonDecode(json)));

/// `sync_status() -> Stream<SyncStatus>`. Drives the sync chip.
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

/// Extract a document with xberg, then embed and store its normalized content.
Future<String> memoryIngestAttachment(String path) =>
    ffi.memoryIngestAttachment(path: path);

/// Extract and ingest up to six documents with xberg, returning record ids.
Future<List<String>> memoryIngestAttachments(List<String> paths) =>
    ffi.memoryIngestAttachments(paths: paths);

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
