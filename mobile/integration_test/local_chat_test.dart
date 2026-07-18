// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:knowme_poc/features/chat/presentation/providers/chat_notifier.dart';
import 'package:knowme_poc/features/chat/presentation/screens/chat_screen.dart';
import 'package:knowme_poc/main.dart' as app;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'first-run local chat produces a finalized assistant bubble',
    (tester) async {
      await app.main();
      await tester.pumpAndSettle(const Duration(seconds: 30));

      await tester.tap(find.byIcon(Icons.chat_bubble_outline));
      await tester.pumpAndSettle();
      expect(find.byType(ChatScreen), findsOneWidget);

      await tester.enterText(
        find.byType(EditableText).last,
        'Reply briefly: local KnowMe chat is working.',
      );
      await tester.tap(find.byIcon(Icons.arrow_upward));
      await tester.pump();

      final container = ProviderScope.containerOf(
        tester.element(find.byType(ChatScreen)),
      );
      final deadline = DateTime.now().add(const Duration(minutes: 12));
      while (DateTime.now().isBefore(deadline)) {
        await tester.pump(const Duration(seconds: 2));
        final state = container.read(chatProvider);
        final assistant = state.messages.where(
          (message) =>
              message.role == 'assistant' && message.content.isNotEmpty,
        );
        if (assistant.any((message) => !message.isStreaming)) {
          expect(find.textContaining('⚠️'), findsNothing);
          return;
        }
      }
      fail(
        'local inference did not finalize an assistant bubble in 12 minutes',
      );
    },
    timeout: const Timeout(Duration(minutes: 15)),
  );
}
