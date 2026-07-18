// TJ-ARCH-MOB-001 compliant
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:prometheus_entity_management/prometheus_entity_management.dart';

class _FailingTransport implements EntityTransport {
  @override
  Future<EntityRecord> update(EntityRecord record) async =>
      throw StateError('rust domain error');
  @override
  Future<ListResult> list(ViewDescriptor view) async => const ListResult();
  @override
  Future<EntityRecord?> get(String entityType, String id) async => null;
  @override
  Future<EntityRecord> create(EntityRecord record) async => record;
  @override
  Future<void> delete(String entityType, String id) async {}
  @override
  Stream<ChangeEvent> changes() => const Stream.empty();
}

void main() {
  test('EntityCrud rolls back the edit buffer when the transport fails',
      () async {
    final container = ProviderContainer(
      overrides: [
        entityTransportProvider.overrideWithValue(_FailingTransport()),
      ],
    );
    addTearDown(container.dispose);

    final ctrl = container.read(
      entityCrudProvider('note', 'n1', const {'title': 'Original'}).notifier,
    );
    ctrl.edit('title', 'Edited');
    expect(
      container
          .read(entityCrudProvider('note', 'n1', const {'title': 'Original'}))
          .isDirty,
      isTrue,
    );

    await expectLater(
      ctrl.save((m) => m.toString()),
      throwsA(isA<StateError>()),
    );

    final buffer = container
        .read(entityCrudProvider('note', 'n1', const {'title': 'Original'}));
    expect(
      buffer.isDirty,
      isTrue,
      reason: 'save rolled back — edits are dirty again',
    );
    expect(buffer.value('title'), 'Edited');
  });
}
