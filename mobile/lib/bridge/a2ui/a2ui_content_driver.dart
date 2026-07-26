// TJ-ARCH-MOB-001 compliant
// A2uiContentDriver — connects one run's A2uiEvent stream to a ChatNotifier.
// It appends ContentBlocks as Block events arrive and finalizes the message on
// RunFinished/RunError. No FFI here; feed it the stream from chatEvents().
import 'dart:async';

import 'package:gen_ui_widgets/gen_ui_widgets.dart';

import 'a2ui_event.dart';

class A2uiContentDriver {
  A2uiContentDriver({
    required this.messageId,
    required this.onBlock,
    required this.onFinalize,
    this.errorMessage,
    this.onPhase,
    this.onCancelled,
    this.onTelemetry,
    this.onError,
  });

  final String messageId;
  final void Function({required String messageId, required ContentBlock block})
  onBlock;
  final void Function(String messageId) onFinalize;
  final Future<String> Function(Object error)? errorMessage;
  final void Function(String messageId, RunPhaseChanged phase)? onPhase;
  final void Function(String messageId, RunCancelled event)? onCancelled;
  final void Function(String messageId, ContextTelemetry event)? onTelemetry;
  final void Function(String messageId, ChatRunError error)? onError;

  StreamSubscription<A2uiEvent>? _sub;
  final Completer<void> _done = Completer<void>();

  Future<void> connect(Stream<A2uiEvent> stream) {
    if (_sub != null) {
      throw StateError('A2uiContentDriver is already connected');
    }
    _sub = stream.listen(
      (event) {
        switch (event) {
          case RunStarted():
            break;
          case final RunPhaseChanged phase:
            onPhase?.call(messageId, phase);
          case BlockEvent(:final block):
            onBlock(messageId: messageId, block: block);
          case RunFinished():
            onFinalize(messageId);
            if (!_done.isCompleted) _done.complete();
          case final RunCancelled cancelled:
            onCancelled?.call(messageId, cancelled);
            onFinalize(messageId);
            if (!_done.isCompleted) _done.complete();
          case RunError(:final error):
            onError?.call(messageId, error);
            onFinalize(messageId);
            if (!_done.isCompleted) _done.complete();
          case final ContextTelemetry telemetry:
            onTelemetry?.call(messageId, telemetry);
        }
      },
      onError: (Object error, StackTrace stackTrace) {
        unawaited(_handleStreamError(error, stackTrace));
      },
      onDone: () {
        if (!_done.isCompleted) _done.complete();
      },
    );
    return _done.future;
  }

  Future<void> _handleStreamError(Object error, StackTrace stackTrace) async {
    final message = errorMessage == null
        ? error.toString().replaceFirst('Exception: ', '')
        : await errorMessage!(error);
    onError?.call(
      messageId,
      ChatRunError(
        code: 'event_stream_error',
        phase: 'generation',
        message: message,
        retryable: true,
        diagnostics: stackTrace.toString(),
      ),
    );
    onFinalize(messageId);
    if (!_done.isCompleted) _done.complete();
  }

  Future<void> dispose() async {
    await _sub?.cancel();
    _sub = null;
    if (!_done.isCompleted) _done.complete();
  }
}
