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
        'run_phase_changed' => RunPhaseChanged(
          json['run_id'] as String,
          json['phase'] as String,
          json['status'] as String,
          json['message'] as String,
        ),
        'block' => BlockEvent(
          json['run_id'] as String,
          ContentBlock.fromJson(json['block'] as Map<String, dynamic>),
        ),
        'run_finished' => RunFinished(json['run_id'] as String),
        'run_cancelled' => RunCancelled(
          json['run_id'] as String,
          json['phase'] as String,
          json['message'] as String,
        ),
        'run_error' => RunError(
          json['run_id'] as String,
          ChatRunError.fromJson(json['error'] as Map<String, dynamic>),
        ),
        'context_telemetry' => ContextTelemetry(
          json['run_id'] as String,
          json['payload'] as Map<String, dynamic>,
        ),
        final type => throw FormatException('Unknown A2uiEvent type: $type'),
      };
}

class RunStarted extends A2uiEvent {
  final String runId;
  const RunStarted(this.runId);
}

class RunPhaseChanged extends A2uiEvent {
  final String runId;
  final String phase;
  final String status;
  final String message;
  const RunPhaseChanged(this.runId, this.phase, this.status, this.message);
}

class BlockEvent extends A2uiEvent {
  final String runId;
  final ContentBlock block;
  const BlockEvent(this.runId, this.block);
}

class RunFinished extends A2uiEvent {
  final String runId;
  const RunFinished(this.runId);
}

class RunCancelled extends A2uiEvent {
  final String runId;
  final String phase;
  final String message;
  const RunCancelled(this.runId, this.phase, this.message);
}

class ChatRunError {
  final String code;
  final String phase;
  final String message;
  final bool retryable;
  final String? diagnostics;

  const ChatRunError({
    required this.code,
    required this.phase,
    required this.message,
    required this.retryable,
    this.diagnostics,
  });

  factory ChatRunError.fromJson(Map<String, dynamic> json) => ChatRunError(
    code: json['code'] as String,
    phase: json['phase'] as String,
    message: json['message'] as String,
    retryable: json['retryable'] as bool,
    diagnostics: json['diagnostics'] as String?,
  );

  Map<String, Object?> toJson() => {
    'code': code,
    'phase': phase,
    'message': message,
    'retryable': retryable,
    if (diagnostics != null) 'diagnostics': diagnostics,
  };
}

class RunError extends A2uiEvent {
  final String runId;
  final ChatRunError error;
  const RunError(this.runId, this.error);
}

class ContextTelemetry extends A2uiEvent {
  final String runId;
  final Map<String, dynamic> payload;
  const ContextTelemetry(this.runId, this.payload);
}
