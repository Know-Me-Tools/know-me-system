// TJ-ARCH-MOB-001 compliant
// A2uiEvent — Dart mirror of gen_ui_types::events::A2uiEvent. serde tag =
// "type", snake_case. The FFI stream carries this shape as JSON (see
// gen_ui_ffi::api::streams's doc comment on why); fromJson decodes it.
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

sealed class A2uiEvent {
  const A2uiEvent();

  factory A2uiEvent.fromJson(Map<String, dynamic> json) =>
      switch (json['type'] as String) {
        'run_started' => RunStarted(json['run_id'] as String),
        'block' => BlockEvent(
            ContentBlock.fromJson(json['block'] as Map<String, dynamic>),
          ),
        'run_finished' => RunFinished(json['run_id'] as String),
        'run_error' => RunError(json['message'] as String),
        final type => throw FormatException('Unknown A2uiEvent type: $type'),
      };
}

class RunStarted extends A2uiEvent {
  final String runId;
  const RunStarted(this.runId);
}

class BlockEvent extends A2uiEvent {
  final ContentBlock block;
  const BlockEvent(this.block);
}

class RunFinished extends A2uiEvent {
  final String runId;
  const RunFinished(this.runId);
}

class RunError extends A2uiEvent {
  final String message;
  const RunError(this.message);
}
