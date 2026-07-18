// TJ-ARCH-MOB-001 compliant
import 'dart:async';

import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../data/pem_notes_repository.dart';
import '../../domain/note.dart';
import '../../domain/notes_repository.dart';

part 'notes_provider.g.dart';

const notesView = ViewDescriptor(
  entityType: 'note',
  sorts: [SortSpec(field: 'updated_at', descending: true)],
  limit: 100,
);

Duration? _noRetry(int retryCount, Object error) => null;

@riverpod
NotesRepository notesRepository(Ref ref) => PemNotesRepository(
      transport: ref.watch(entityTransportProvider),
      loadRecords: () => ref.read(entityListProvider(notesView).future),
    );

@Riverpod(retry: _noRetry)
Future<List<Note>> notes(Ref ref) {
  ref.watch(entityChangeBridgeProvider);
  ref.watch(entityListProvider(notesView));
  return ref.watch(notesRepositoryProvider).listNotes();
}

@riverpod
class NotesActions extends _$NotesActions {
  @override
  FutureOr<void> build() {}

  Future<void> create() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(() async {
      await ref.read(notesRepositoryProvider).createNote(
            title: 'New note',
            body: '',
          );
      ref.invalidate(notesProvider);
    });
  }

  Future<void> delete(String id) async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(() async {
      await ref.read(notesRepositoryProvider).deleteNote(id);
      ref.invalidate(notesProvider);
    });
  }
}
