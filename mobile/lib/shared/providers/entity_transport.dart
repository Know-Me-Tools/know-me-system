// TJ-ARCH-MOB-001 compliant
// FrbEntityTransport — the host adapter that satisfies PEM's EntityTransport
// seam by delegating to the FFI facade. This is the ONLY EntityTransport impl;
// features consume PEM's providers, never this class directly. Tests override
// entityTransportProvider with a fake at this same boundary — nothing internal
// is mocked (see references/flutter/testing.md).
import 'package:prometheus_entity_management/prometheus_entity_management.dart';

import '../../bridge/rust_bridge_provider.dart' as bridge;

class FrbEntityTransport implements EntityTransport {
  const FrbEntityTransport();

  @override
  Future<ListResult> list(ViewDescriptor view) => bridge.entityList(view);

  @override
  Future<EntityRecord?> get(String entityType, String id) =>
      bridge.entityGet(entityType, id);

  @override
  Future<EntityRecord> create(EntityRecord record) =>
      bridge.entityCreate(record);

  @override
  Future<EntityRecord> update(EntityRecord record) =>
      bridge.entityUpdate(record);

  @override
  Future<void> delete(String entityType, String id) =>
      bridge.entityDelete(entityType, id);

  @override
  Stream<ChangeEvent> changes() => bridge.entityChanges();
}
