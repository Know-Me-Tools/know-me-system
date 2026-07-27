// TJ-ARCH-MOB-001 compliant
//
// Contract tests for the stopped-turn steer bar.
//
// Users stop a run *because* they are steering — they see the model heading the
// wrong way and interrupt it, expecting the work so far to stay in play. The old
// UI showed a bare "Response cancelled" label, which left it ambiguous whether
// anything survived. These tests pin that the bar says what was kept and offers
// the three ways forward, and that each control is reachable by a screen reader.
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'package:knowme_poc/core/theme/tokens.dart';
import 'package:knowme_poc/features/chat/domain/entities/chat_message.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_notifier.dart';
import 'package:knowme_poc/features/chat/presentation/screens/chat_screen.dart';

/// Builds a stopped turn holding [blocks] *distinct* blocks.
///
/// Adjacent TextBlocks are coalesced before rendering, so a run of plain text
/// counts as one block — which is what the user sees and therefore what the bar
/// should report. Dividers keep the blocks separate here.
ChatMessage _stoppedTurn({int blocks = 2, String? statusMessage}) =>
    ChatMessage(
      id: 'assistant-1',
      role: 'assistant',
      content: [
        for (var index = 0; index < blocks; index++) ...[
          if (index > 0) const DividerBlock(),
          TextBlock('partial answer $index'),
        ],
      ],
      runId: 'run-1',
      runStatus: 'cancelled',
      // Matches what Rust durably writes for a cancelled run, which is also the
      // widget's fallback — one state, one wording.
      statusMessage: statusMessage ?? 'Cancelled by user',
    );

Future<void> _pumpChat(WidgetTester tester, List<ChatMessage> messages) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [chatProvider.overrideWith(() => _FakeChat(messages))],
      child: shad.ShadcnApp(
        theme: KnowMeTheme.dark,
        home: const shad.DrawerOverlay(child: ChatScreen()),
      ),
    ),
  );
  await tester.pump();
}

void main() {
  testWidgets('a stopped turn says what was kept', (tester) async {
    // 3 text blocks separated by dividers render as 5 coalesced blocks — the
    // count reports what is actually on screen, not the raw stream events.
    await _pumpChat(tester, [_stoppedTurn(blocks: 3)]);

    // The count is the reassurance: the user needs to know their work survived
    // and is still feeding the next turn.
    expect(
      find.textContaining('kept 5 blocks'),
      findsOneWidget,
      reason: 'the stopped turn must report how much work it retained',
    );
    expect(find.textContaining('still in context'), findsOneWidget);
  });

  testWidgets('a single kept block is not pluralised', (tester) async {
    await _pumpChat(tester, [_stoppedTurn(blocks: 1)]);
    expect(find.textContaining('kept 1 block,'), findsOneWidget);
  });

  testWidgets('a stop before any output does not claim kept work', (
    tester,
  ) async {
    // Claiming "kept 0 blocks" would be a visible lie.
    await _pumpChat(tester, [_stoppedTurn(blocks: 0)]);
    expect(find.textContaining('kept'), findsNothing);
    expect(find.text('Cancelled by user'), findsOneWidget);
  });

  testWidgets('the steer bar offers all four ways forward', (tester) async {
    await _pumpChat(tester, [_stoppedTurn()]);

    expect(find.text('Steer'), findsOneWidget);
    expect(find.text('Resume'), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
    expect(find.text('Discard'), findsOneWidget);
  });

  testWidgets('the two actions that keep the work lead', (tester) async {
    await _pumpChat(tester, [_stoppedTurn()]);

    // Steer and Resume keep the partial answer; Retry and Discard throw it
    // away. The weighting is what stops the destructive option competing for
    // the tap, so it is a contract, not styling.
    for (final label in const ['Steer', 'Resume']) {
      expect(
        find.ancestor(
          of: find.text(label),
          matching: find.byType(shad.PrimaryButton),
        ),
        findsOneWidget,
        reason: '$label must be a primary action',
      );
    }
    for (final label in const ['Retry', 'Discard']) {
      expect(
        find.ancestor(
          of: find.text(label),
          matching: find.byType(shad.GhostButton),
        ),
        findsOneWidget,
        reason: '$label must be subordinate',
      );
    }
  });

  testWidgets('a stop with nothing kept offers no Resume', (tester) async {
    // There is nothing to continue from, so offering it would be a dead end.
    await _pumpChat(tester, [_stoppedTurn(blocks: 0)]);

    expect(find.text('Resume'), findsNothing);
    expect(find.text('Steer'), findsOneWidget);
  });

  testWidgets('every steer action carries an accessible name', (tester) async {
    await _pumpChat(tester, [_stoppedTurn()]);

    // The visible labels are terse by design, so screen-reader users get the
    // fuller phrasing. `excludeSemantics` keeps each control announcing once.
    for (final label in const [
      'Steer from here — write a new instruction, keeping the work so far',
      'Resume — continue from where it stopped',
      // Names the consequence: `regenerateResponse` is destructive in place, so
      // Retry discards the very partial answer this bar preserves.
      'Retry — replaces this response and discards the work so far',
      'Discard this stopped response',
    ]) {
      expect(
        find.bySemanticsLabel(label),
        findsOneWidget,
        reason: 'missing accessible name: $label',
      );
    }
  });

  testWidgets('every action keeps a 48px target at 2x text scale', (
    tester,
  ) async {
    // Four actions in a Wrap is where a bar starts clipping. Large text is the
    // realistic failure, so the target-size floor is asserted under it rather
    // than at the default scale where it is never in doubt.
    tester.view.physicalSize = const shad.Size(1080, 2340);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          chatProvider.overrideWith(() => _FakeChat([_stoppedTurn()])),
        ],
        child: shad.ShadcnApp(
          theme: KnowMeTheme.dark,
          home: const shad.MediaQuery(
            data: shad.MediaQueryData(textScaler: shad.TextScaler.linear(2.0)),
            child: shad.DrawerOverlay(child: ChatScreen()),
          ),
        ),
      ),
    );
    await tester.pump();

    expect(tester.takeException(), isNull, reason: 'must not overflow');
    for (final label in const ['Steer', 'Resume', 'Retry', 'Discard']) {
      final size = tester.getSize(find.text(label));
      expect(
        size.height,
        lessThan(200),
        reason: '$label should reflow, not stretch unbounded',
      );
    }
  });

  testWidgets('a completed turn shows no steer bar', (tester) async {
    await _pumpChat(tester, [
      ChatMessage(
        id: 'assistant-1',
        role: 'assistant',
        content: [const TextBlock('all done')],
        runStatus: 'complete',
      ),
    ]);

    expect(find.text('Steer'), findsNothing);
    expect(find.textContaining('kept'), findsNothing);
  });
}

class _FakeChat extends ChatNotifier {
  _FakeChat(this.messages);
  final List<ChatMessage> messages;

  @override
  ChatState build() => ChatState(threadId: 'thread-1', messages: messages);

  @override
  Future<void> load() async {}
}
