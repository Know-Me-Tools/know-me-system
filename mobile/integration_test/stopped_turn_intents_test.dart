// TJ-ARCH-MOB-001 compliant
//
// On-device certification for the four explicit intents on a stopped turn.
//
// The widget tests prove the bar renders and that `sendResume` builds the right
// request. What they cannot prove is the part that depends on a real engine: that
// a mid-stream stop preserves partial text on THIS platform, that the turn after
// a cancel still generates (the Android engine had to be rebuilt to make that
// true — iOS/MLX is a different engine and needs its own evidence), and that
// Resume continues the same thought instead of restarting.
//
// Runs in-process rather than through an external UI driver: iOS has no
// uiautomator equivalent, and reading `chatProvider` directly is stronger
// evidence than screen-scraping anyway.
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:integration_test/integration_test.dart';

import 'package:knowme_poc/features/chat/domain/entities/chat_message.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_control_notifier.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_notifier.dart';
import 'package:knowme_poc/features/chat/presentation/screens/chat_screen.dart';
import 'package:knowme_poc/main.dart' as app;

/// Long enough that a stop always lands mid-stream rather than after the run.
const _longPrompt =
    'Tell me a long continuous story about a lighthouse keeper. Write at '
    'least 2000 words of flowing prose and do not stop early.';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a stopped turn keeps its work and offers four intents', (
    tester,
  ) async {
    await app.main();
    await tester.pumpAndSettle(const Duration(seconds: 30));

    await tester.tap(find.byIcon(Icons.chat_bubble_outline));
    await tester.pumpAndSettle();
    expect(find.byType(ChatScreen), findsOneWidget);

    final container = ProviderScope.containerOf(
      tester.element(find.byType(ChatScreen)),
    );

    // ---- pin the Local lane and PROVE it took ----------------------------
    //
    // Without this the run can silently measure the Orchestrator lane. That
    // exact mistake produced a misleading chars=0 on Android and read as a
    // streaming failure when it was nothing of the kind.
    //
    // `setMode` records a refusal in `state.error` instead of throwing, so the
    // resulting state — not the absence of an exception — is the evidence.
    final threadId = container.read(chatProvider).threadId;
    expect(threadId, isNotEmpty, reason: 'no conversation was created');
    final control = container.read(chatControlProvider(threadId).notifier);
    await control.setMode('local');
    await tester.pumpAndSettle();

    final controlState = container.read(chatControlProvider(threadId));
    expect(
      controlState.error,
      isNull,
      reason: 'switching to the Local lane was refused: ${controlState.error}',
    );
    expect(
      controlState.policy['chat_mode'],
      'local',
      reason: 'lane is "${controlState.policy['chat_mode']}", not local; '
          'refusing to certify another lane',
    );
    debugPrint('KnowMe intents: lane=local confirmed on thread $threadId');

    // ---- stream, then stop mid-flight ------------------------------------
    await tester.enterText(find.byType(EditableText).last, _longPrompt);
    await tester.tap(find.byIcon(Icons.arrow_upward));
    await tester.pump();

    // Wait for real streamed text, not merely for the run to start. Stopping
    // before the first block arrives would prove nothing about preservation.
    final streamed = await _pumpUntil(
      tester,
      timeout: const Duration(minutes: 8),
      condition: () => _streamingAssistant(container)?.content.isNotEmpty == true,
      describe: () => _diagnostics(container.read(chatProvider)),
    );
    expect(
      streamed,
      isTrue,
      reason: 'no assistant text streamed, so no mid-stream stop was possible',
    );

    final streaming = _streamingAssistant(container)!;
    final blocksBeforeStop = streaming.content.length;
    final charsBeforeStop = _charCount(streaming);
    debugPrint(
      'KnowMe intents: stopping mid-stream with $blocksBeforeStop blocks '
      '/ $charsBeforeStop chars',
    );

    await container.read(chatProvider.notifier).cancelRun(streaming.id);
    final settled = await _pumpUntil(
      tester,
      timeout: const Duration(minutes: 2),
      condition: () {
        final message = _messageById(container, streaming.id);
        return message != null &&
            !message.isStreaming &&
            message.runStatus == 'cancelled';
      },
      describe: () => _diagnostics(container.read(chatProvider)),
    );
    expect(settled, isTrue, reason: 'the run never settled as cancelled');

    final stopped = _messageById(container, streaming.id)!;
    // The whole point of the feature: a stop must not discard the work.
    expect(
      _charCount(stopped),
      greaterThan(0),
      reason: 'the stop discarded every character the model had produced',
    );
    expect(stopped.error, isNull, reason: 'a stop must not read as a failure');
    debugPrint(
      'KnowMe intents: kept ${_charCount(stopped)} chars in '
      '${stopped.content.length} blocks, status="${stopped.statusMessage}"',
    );

    // ---- the four intents are on screen ----------------------------------
    await tester.pumpAndSettle();
    for (final label in const ['Steer', 'Resume', 'Retry', 'Discard']) {
      expect(
        find.text(label),
        findsOneWidget,
        reason: '"$label" is missing from the stopped turn',
      );
    }
    // The kept-count line is the reassurance that the work survived; without it
    // the four actions are buttons with no stated consequence.
    expect(find.textContaining('still in context'), findsOneWidget);

    // ---- Resume continues rather than restarting -------------------------
    final keptText = _plainText(stopped);
    await tester.tap(find.text('Resume'));
    await tester.pump();

    final resumed = await _pumpUntil(
      tester,
      timeout: const Duration(minutes: 10),
      condition: () {
        final latest = _latestAssistant(container);
        return latest != null &&
            latest.id != stopped.id &&
            !latest.isStreaming &&
            _charCount(latest) > 0;
      },
      describe: () => _diagnostics(container.read(chatProvider)),
    );
    expect(resumed, isTrue, reason: 'Resume never produced a finished turn');

    final continuation = _latestAssistant(container)!;
    expect(continuation.error, isNull, reason: 'Resume run failed');
    debugPrint(
      'KnowMe intents: Resume produced ${_charCount(continuation)} chars',
    );

    // The stopped partial must still be in the transcript — that is what the
    // next request replays, and it is the reason Resume needs no Rust change.
    final stoppedStillPresent = _messageById(container, stopped.id);
    expect(
      stoppedStillPresent,
      isNotNull,
      reason: 'Resume must not delete the partial turn it continues from',
    );
    expect(_plainText(stoppedStillPresent!), keptText);

    // The resume instruction is sent visibly rather than hidden, so the
    // transcript stays honest about what went to the model.
    final resumeTurn = container
        .read(chatProvider)
        .messages
        .where((message) => message.intent == 'resume');
    expect(
      resumeTurn,
      isNotEmpty,
      reason: 'the resume turn was not marked with intent=resume',
    );

    expect(find.textContaining('⚠️'), findsNothing);
  }, timeout: const Timeout(Duration(minutes: 25)));
}

/// Pumps until [condition] holds, logging state transitions along the way.
///
/// `pumpAndSettle` is unusable here: a streaming run schedules frames
/// continuously, so it never settles.
Future<bool> _pumpUntil(
  WidgetTester tester, {
  required Duration timeout,
  required bool Function() condition,
  required String Function() describe,
}) async {
  final deadline = DateTime.now().add(timeout);
  String? previous;
  while (DateTime.now().isBefore(deadline)) {
    await tester.pump(const Duration(seconds: 2));
    if (condition()) return true;
    final current = describe();
    if (current != previous) {
      debugPrint('KnowMe intents state: $current');
      previous = current;
    }
  }
  debugPrint('KnowMe intents TIMEOUT: ${describe()}');
  return false;
}

ChatMessage? _messageById(ProviderContainer container, String id) => container
    .read(chatProvider)
    .messages
    .where((message) => message.id == id)
    .firstOrNull;

ChatMessage? _streamingAssistant(ProviderContainer container) => container
    .read(chatProvider)
    .messages
    .where((message) => message.role == 'assistant' && message.isStreaming)
    .firstOrNull;

ChatMessage? _latestAssistant(ProviderContainer container) => container
    .read(chatProvider)
    .messages
    .where((message) => message.role == 'assistant')
    .lastOrNull;

String _plainText(ChatMessage message) => message.content
    .whereType<TextBlock>()
    .map((block) => block.text)
    .join();

int _charCount(ChatMessage message) => _plainText(message).length;

String _diagnostics(ChatState state) => <String, Object?>{
  'threadId': state.threadId,
  'modelOperation': state.modelOperation,
  'messages': state.messages
      .map(
        (message) => <String, Object?>{
          'role': message.role,
          'status': message.runStatus,
          'phase': message.runPhase,
          'statusMessage': message.statusMessage,
          'intent': message.intent,
          'streaming': message.isStreaming,
          'blocks': message.content.length,
          'chars': _charCount(message),
          'error': message.error?.displayMessage,
        },
      )
      .toList(),
}.toString();
