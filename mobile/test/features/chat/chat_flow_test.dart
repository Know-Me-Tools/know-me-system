// TJ-ARCH-MOB-001 compliant
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

import 'package:knowme_poc/bridge/a2ui/a2ui_event.dart';
import 'package:knowme_poc/features/chat/domain/entities/chat_message.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_notifier.dart';

// A fake ONLY at the FFI edge: Rust-shaped A2uiEvents. Nothing internal mocked.
Stream<A2uiEvent> _cannedRun() async* {
  yield const RunStarted('run-1');
  yield const BlockEvent(TextBlock('Hello '));
  yield const BlockEvent(TextBlock('world'));
  yield const RunFinished('run-1');
}

void main() {
  test('ChatNotifier folds an A2uiEvent stream into ContentBlocks', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(chatProvider.notifier);
    await notifier.foldStream('msg-1', _cannedRun());

    final ChatState state = container.read(chatProvider);
    final msg = state.messages.firstWhere((m) => m.id == 'msg-1');
    expect(
      msg.isStreaming,
      isFalse,
      reason: 'RunFinished finalizes the message',
    );
    expect(msg.content, hasLength(2));
    expect(msg.content.every((b) => b is TextBlock), isTrue);
  });
}
