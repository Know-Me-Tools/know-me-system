// TJ-ARCH-MOB-001 compliant
import 'dart:convert';

import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:uuid/uuid.dart';

import '../domain/note.dart';
import '../domain/notes_repository.dart';

class PemNotesRepository implements NotesRepository {
  PemNotesRepository({required this.transport, required this.loadRecords});

  final EntityTransport transport;
  final Future<ListResult> Function() loadRecords;
  static const _uuid = Uuid();

  @override
  Future<List<Note>> listNotes() async {
    final result = await loadRecords();
    return result.items.map((record) {
      final data = jsonDecode(record.dataJson) as Map<String, Object?>;
      return Note(
        id: record.id,
        title: data['title']?.toString() ?? record.id,
        body: data['body']?.toString() ?? '',
      );
    }).toList(growable: false);
  }

  @override
  Future<void> createNote({required String title, required String body}) async {
    await transport.create(
      EntityRecord(
        id: _uuid.v4(),
        entityType: 'note',
        dataJson: jsonEncode({'title': title, 'body': body}),
      ),
    );
  }

  @override
  Future<void> deleteNote(String id) => transport.delete('note', id);
}
