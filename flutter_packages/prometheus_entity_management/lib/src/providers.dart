// TJ-ARCH-MOB-001 compliant
/// The heart of the Flutter PEM port. Riverpod provider *families* are the
/// normalization map: one `entityList(view)` instance per query, one
/// `entity(type,id)` instance per record — Riverpod caches and dedupes them, so
/// there is no hand-built Dart graph store. All FFI-backed providers opt out of
/// Riverpod 3 auto-retry via `_noRetry`: a Rust domain error is a terminal
/// result, and silent re-invocation would re-run the whole Rust operation.
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'entity.dart';
import 'transport.dart';
import 'view.dart';

part 'providers.g.dart';

/// FFI providers are terminal on error — see references/flutter/patterns.md.
Duration? _noRetry(int retryCount, Object error) => null;

/// Host wiring seam. The app overrides this with its frb-backed adapter:
/// `entityTransportProvider.overrideWithValue(FrbEntityTransport(...))`.
/// Tests override it with a fake at the FFI boundary — nothing internal is mocked.
@Riverpod(keepAlive: true)
EntityTransport entityTransport(Ref ref) =>
    throw UnimplementedError(
      'Override entityTransportProvider with a gen_ui_flutter-backed adapter.',
    );

/// One list instance per ViewDescriptor — the family key normalizes queries.
@Riverpod(retry: _noRetry)
Future<ListResult> entityList(Ref ref, ViewDescriptor view) {
  return ref.watch(entityTransportProvider).list(view);
}

/// One record instance per (type, id). Features decode `dataJson` themselves.
@Riverpod(retry: _noRetry)
Future<EntityRecord?> entity(Ref ref, String entityType, String id) {
  return ref.watch(entityTransportProvider).get(entityType, id);
}

/// Bridges the single Rust ChangeEvent feed into targeted invalidations. Mount
/// it once (e.g. `ref.watch(entityChangeBridgeProvider)` at app root). It never
/// holds state — it only translates change ops into `ref.invalidate`.
@Riverpod(keepAlive: true)
Stream<ChangeEvent> entityChangeBridge(Ref ref) {
  final stream = ref.watch(entityTransportProvider).changes();
  final sub = stream.listen((event) {
    switch (event) {
      case ChangeUpsert(:final record):
        ref.invalidate(entityProvider(record.entityType, record.id));
      case ChangeDelete(:final entityType, :final id):
        ref.invalidate(entityProvider(entityType, id));
      case ChangeInvalidate():
        // A list-shaped change: cheapest correct move is to drop list caches
        // for the affected type. entityList is a family; invalidating the
        // provider clears all of its instances.
        ref.invalidate(entityListProvider);
    }
  });
  ref.onDispose(sub.cancel);
  return stream;
}

/// A dirty-path edit buffer over a single record's decoded field map. Tracks
/// exactly which paths changed so partial updates and optimistic UI both know
/// the minimal diff. Immutable: every edit returns a new buffer.
class EditBuffer {
  const EditBuffer({
    required this.original,
    this.edits = const {},
  });

  /// The last-known-clean field map (decoded from EntityRecord.dataJson).
  final Map<String, Object?> original;

  /// Path → new value for every field the user touched.
  final Map<String, Object?> edits;

  bool get isDirty => edits.isNotEmpty;
  Set<String> get dirtyPaths => edits.keys.toSet();

  /// Effective value for a field: the edit if present, else the original.
  Object? value(String path) => edits.containsKey(path) ? edits[path] : original[path];

  EditBuffer set(String path, Object? value) {
    // No-op if the value equals the original — keeps the path out of dirtyPaths.
    if (original[path] == value && !edits.containsKey(path)) return this;
    final next = Map<String, Object?>.from(edits);
    if (original[path] == value) {
      next.remove(path);
    } else {
      next[path] = value;
    }
    return EditBuffer(original: original, edits: next);
  }

  EditBuffer revert() => EditBuffer(original: original, edits: const {});

  /// The merged field map to persist (original overlaid with edits).
  Map<String, Object?> merged() => {...original, ...edits};
}

/// CRUD controller for one entity type. Composes the list family, an edit
/// buffer, and optimistic snapshot/rollback. It performs writes through the
/// transport (FFI → Rust) and reconciles via the ChangeEvent bridge, so the
/// canonical store stays in Rust and the UI holds only edit-in-flight state.
@riverpod
class EntityCrud extends _$EntityCrud {
  @override
  EditBuffer build(String entityType, String id, Map<String, Object?> initial) {
    return EditBuffer(original: initial);
  }

  /// Record a field edit — no I/O, just updates the dirty buffer.
  void edit(String path, Object? value) {
    state = state.set(path, value);
  }

  /// Discard all pending edits.
  void revert() {
    state = state.revert();
  }

  /// Persist the buffer. Optimistically clears dirty state, then rolls back on a
  /// Rust domain error. Returns true on success. dataJson encoding is the host's
  /// concern (it owns the feature model) — this passes the merged map through
  /// the transport by re-reading the record and letting Rust own the write.
  Future<bool> save(String Function(Map<String, Object?>) encode) async {
    if (!state.isDirty) return true;
    final snapshot = state; // optimistic snapshot for rollback
    final merged = state.merged();
    state = EditBuffer(original: merged); // optimistic: buffer is now clean

    try {
      final transport = ref.read(entityTransportProvider);
      await transport.update(EntityRecord(
        id: id,
        entityType: entityType,
        dataJson: encode(merged),
      ));
      return true;
    } catch (_) {
      state = snapshot; // roll back — the edits are dirty again
      rethrow;
    }
  }

  /// Delete the record. Optimistic; the ChangeEvent bridge reconciles lists.
  Future<void> deleteRecord() async {
    await ref.read(entityTransportProvider).delete(entityType, id);
  }
}
