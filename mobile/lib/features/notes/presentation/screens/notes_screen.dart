// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../../../shared/widgets/knowme_screen.dart';
import '../providers/notes_provider.dart';

class NotesScreen extends ConsumerWidget {
  const NotesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notes = ref.watch(notesProvider);
    final action = ref.watch(notesActionsProvider);

    final colors = shad.Theme.of(context).colorScheme;
    return KnowMeScreen(
      title: 'Notes',
      trailing: [
        shad.PrimaryButton(
          onPressed: action.isLoading
              ? null
              : () => ref.read(notesActionsProvider.notifier).create(),
          density: shad.ButtonDensity.icon,
          child: const Icon(Icons.add),
        ),
      ],
      child: notes.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('Error: $e')),
        data: (result) => ListView(
          padding: const EdgeInsets.all(16),
          children: [
            for (final note in result)
              Padding(
                padding: const EdgeInsets.only(bottom: 10),
                child: Container(
                  padding: const EdgeInsets.all(14),
                  decoration: BoxDecoration(
                    color: colors.card,
                    borderRadius: BorderRadius.circular(14),
                  ),
                  child: Row(
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(note.title),
                            const SizedBox(height: 3),
                            Text(
                              note.id,
                              style: TextStyle(
                                color: colors.mutedForeground,
                                fontSize: 11,
                              ),
                            ),
                          ],
                        ),
                      ),
                      shad.GhostButton(
                        onPressed: action.isLoading
                            ? null
                            : () => ref
                                .read(notesActionsProvider.notifier)
                                .delete(note.id),
                        density: shad.ButtonDensity.icon,
                        child: const Icon(Icons.delete_outline),
                      ),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
