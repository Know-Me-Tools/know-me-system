// TJ-ARCH-MOB-001 compliant
import 'note.dart';

abstract interface class NotesRepository {
  Future<List<Note>> listNotes();
  Future<void> createNote({required String title, required String body});
  Future<void> deleteNote(String id);
}
