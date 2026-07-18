// TJ-ARCH-MOB-001 compliant
/// EntityTransport — the entity data-access seam. Implemented by the host app as
/// a thin adapter over the frb-generated gen_ui_flutter bindings; it delegates
/// straight to gen_ui_core. Nothing above this layer talks to the FFI directly.
import 'entity.dart';
import 'view.dart';

abstract interface class EntityTransport {
  Future<ListResult> list(ViewDescriptor view);
  Future<EntityRecord?> get(String entityType, String id);
  Future<EntityRecord> create(EntityRecord record);
  Future<EntityRecord> update(EntityRecord record);
  Future<void> delete(String entityType, String id);

  /// Rust-emitted change feed — one stream for the whole store. The bridge
  /// provider folds it into targeted `ref.invalidate` calls.
  Stream<ChangeEvent> changes();
}
