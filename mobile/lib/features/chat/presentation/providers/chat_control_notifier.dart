// TJ-ARCH-MOB-001 compliant
import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../data/chat_control_repository.dart';

part 'chat_control_notifier.g.dart';

@immutable
class ChatControlState {
  const ChatControlState({
    this.catalog = const {},
    this.policy = const {},
    this.effective = const {},
    this.laneAvailability = const {},
    this.loading = false,
    this.saving = false,
    this.error,
  });
  final Map<String, dynamic> catalog;
  final Map<String, dynamic> policy;
  final Map<String, dynamic> effective;

  /// Lane id -> why it cannot be selected, or null when it is selectable.
  ///
  /// Rust rejects switching to a lane whose runtime is unavailable, so the mode
  /// picker renders those entries disabled with the reason instead of offering a
  /// control that fails when tapped. An absent key means "not probed yet";
  /// treat that as available so the UI never blocks on a missing probe.
  final Map<String, String?> laneAvailability;
  final bool loading;
  final bool saving;
  final String? error;

  /// True when [lane] has been probed and reported unavailable.
  bool laneBlocked(String lane) =>
      laneAvailability.containsKey(lane) && laneAvailability[lane] != null;

  ChatControlState copyWith({
    Map<String, dynamic>? catalog,
    Map<String, dynamic>? policy,
    Map<String, dynamic>? effective,
    Map<String, String?>? laneAvailability,
    bool? loading,
    bool? saving,
    String? error,
    bool clearError = false,
  }) => ChatControlState(
    catalog: catalog ?? this.catalog,
    policy: policy ?? this.policy,
    effective: effective ?? this.effective,
    laneAvailability: laneAvailability ?? this.laneAvailability,
    loading: loading ?? this.loading,
    saving: saving ?? this.saving,
    error: clearError ? null : error ?? this.error,
  );
}

@riverpod
class ChatControl extends _$ChatControl {
  static const _repository = ChatControlRepository();

  @override
  ChatControlState build(String conversationId) =>
      ChatControlState(policy: _defaultPolicy());

  Future<void> load() async {
    if (state.loading) return;
    state = state.copyWith(loading: true, clearError: true);
    try {
      // The catalog and lane availability describe the app, not a conversation,
      // so they load even before the first message exists. Only the stored
      // policy needs a conversation id — previously the whole load bailed out on
      // an empty id, which left a brand-new chat with no agents to choose from.
      final results = await Future.wait<Object?>([
        _repository.catalog(),
        _repository.laneAvailability(),
        if (conversationId.isNotEmpty) _repository.policy(conversationId),
      ]);
      state = state.copyWith(
        catalog: results[0]! as Map<String, dynamic>,
        laneAvailability: _laneAvailability(results[1]),
        policy: mergeConversationPolicy(results.length > 2 ? results[2] : null),
      );
    } catch (error) {
      state = state.copyWith(error: await _repository.errorMessage(error));
    } finally {
      state = state.copyWith(loading: false);
    }
  }

  /// Switch this conversation's mode.
  ///
  /// Rust rejects a lane whose runtime is unavailable, and that rejection must
  /// reach the user: this mirrors the error discipline of [load] and [patch] —
  /// catch, record the reason in `error`, and leave the displayed selection
  /// alone. Moving the selection optimistically would make the UI claim a mode
  /// that Rust refused, which is worse than an unresponsive control because the
  /// next send would silently route somewhere else.
  Future<void> setMode(String mode, {String? agentId}) async {
    if (state.saving) return;
    final agentIdForMode = switch (mode) {
      'agent' => agentId,
      'uar' => 'orchestrator-agent',
      _ => null,
    };
    state = state.copyWith(saving: true, clearError: true);
    try {
      await _repository.selectLane(_laneForMode(mode));
    } catch (error) {
      state = state.copyWith(
        error: await _repository.errorMessage(error),
        saving: false,
      );
      return;
    }
    // The lane switch succeeded, so the selection is real. Reflect it now — a
    // brand-new conversation has no id yet, so `patch` cannot persist and this
    // is the only place the choice is recorded until the thread exists.
    state = state.copyWith(
      policy: {...state.policy, 'chat_mode': mode, 'agent_id': agentIdForMode},
      saving: false,
    );
    await patch({'chat_mode': mode, 'agent_id': agentIdForMode});
  }

  /// Map the picker's user-facing mode onto an execution lane. Mirrors
  /// `lane_for_chat_mode` in `chat.rs`: `agent` and `uar` both run through the
  /// embedded UAR and differ only in which agent artifact the run resolves.
  static String _laneForMode(String mode) => switch (mode) {
    'local' => 'local',
    'cloud' => 'cloud',
    _ => 'uar',
  };

  Future<void> setSelection(String key, String mode, List<String> ids) =>
      patch({
        key: {
          'mode': mode,
          'ids': mode == 'selected' ? ids : <String>[],
          'denied_ids': <String>[],
        },
      });

  Future<void> patch(Map<String, dynamic> values) async {
    if (conversationId.isEmpty || state.saving) return;
    final policy = {...state.policy, ...values};
    state = state.copyWith(policy: policy, saving: true, clearError: true);
    try {
      // `savePolicy` echoes the stored policy in the same bare shape it accepts,
      // so it can be layered back over the defaults exactly like a fresh load.
      final saved = await _repository.savePolicy(conversationId, policy);
      final effective = await _repository.effective(conversationId);
      state = state.copyWith(
        policy: mergeConversationPolicy(saved),
        effective: effective,
      );
    } catch (error) {
      state = state.copyWith(error: await _repository.errorMessage(error));
    } finally {
      state = state.copyWith(saving: false);
    }
  }
}

/// Parse Rust's `lane_availability()` list into `lane -> reason?`.
///
/// Rust reports `[{lane, available, reason}]`. Only unavailable lanes carry a
/// reason; available ones map to null so [ChatControlState.laneBlocked] can
/// distinguish "usable" from "not probed".
Map<String, String?> _laneAvailability(Object? raw) {
  final entries = <String, String?>{};
  if (raw is List) {
    for (final item in raw) {
      if (item is! Map) continue;
      final lane = item['lane']?.toString();
      if (lane == null || lane.isEmpty) continue;
      entries[lane] = item['available'] == true
          ? null
          : item['reason']?.toString() ?? 'Unavailable on this device';
    }
  }
  return entries;
}

/// Layer a stored policy over the defaults.
///
/// Rust returns the **bare** `RunPolicy` (or null when the conversation has no
/// saved policy), so its keys sit at the top level alongside the defaults'. A
/// policy written before a field existed simply omits it, and the default fills
/// in — which also keeps `savePolicy` receiving a complete `RunPolicy` rather
/// than a sparse patch.
@visibleForTesting
Map<String, dynamic> mergeConversationPolicy(Object? stored) => {
  ..._defaultPolicy(),
  if (stored is Map<String, dynamic>) ...stored,
};

Map<String, dynamic> _defaultPolicy() => {
  'version': 1,
  'chat_mode': 'uar',
  'agent_id': 'orchestrator-agent',
  'model': null,
  'skills': _inherit(),
  'tools': _inherit(),
  'mcp_servers': _inherit(),
  'knowledge_bases': _inherit(),
  'memory_enabled': null,
  'context_strategy': null,
  'tool_approval': 'ask',
};

Map<String, dynamic> _inherit() => {
  'mode': 'inherit',
  'ids': <String>[],
  'denied_ids': <String>[],
};
