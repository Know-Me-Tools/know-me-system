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
    this.loading = false,
    this.saving = false,
    this.error,
  });
  final Map<String, dynamic> catalog;
  final Map<String, dynamic> policy;
  final Map<String, dynamic> effective;
  final bool loading;
  final bool saving;
  final String? error;

  ChatControlState copyWith({
    Map<String, dynamic>? catalog,
    Map<String, dynamic>? policy,
    Map<String, dynamic>? effective,
    bool? loading,
    bool? saving,
    String? error,
    bool clearError = false,
  }) => ChatControlState(
    catalog: catalog ?? this.catalog,
    policy: policy ?? this.policy,
    effective: effective ?? this.effective,
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
    if (conversationId.isEmpty || state.loading) return;
    state = state.copyWith(loading: true, clearError: true);
    try {
      final results = await Future.wait<Object?>([
        _repository.catalog(),
        _repository.policy(conversationId),
      ]);
      state = state.copyWith(
        catalog: results[0]! as Map<String, dynamic>,
        policy: results[1] as Map<String, dynamic>? ?? _defaultPolicy(),
      );
    } catch (error) {
      state = state.copyWith(error: await _repository.errorMessage(error));
    } finally {
      state = state.copyWith(loading: false);
    }
  }

  Future<void> setMode(String mode, {String? agentId}) async {
    // `selectLane` persists `active_lane`, which is what Rust actually reads to
    // route a send — it is the authority for the lane, not the conversation
    // policy. Reflect the new mode in state immediately so the toolbar shows
    // the tab the user tapped: `patch` below persists the conversation policy,
    // but it no-ops when there is no conversation yet (a brand-new chat has an
    // empty threadId) and swallows persist failures into `error`. Without this
    // local update the lane switched in Rust while the UI kept showing the old
    // tab, which read as "the Local tab does not work".
    final agentIdForMode = switch (mode) {
      'agent' => agentId,
      'uar' => 'orchestrator-agent',
      _ => null,
    };
    state = state.copyWith(
      policy: {...state.policy, 'chat_mode': mode, 'agent_id': agentIdForMode},
    );
    await _repository.selectLane(mode == 'local' ? 'local' : 'uar');
    await patch({'chat_mode': mode, 'agent_id': agentIdForMode});
  }

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
      final saved = await _repository.savePolicy(conversationId, policy);
      final effective = await _repository.effective(conversationId);
      state = state.copyWith(policy: saved, effective: effective);
    } catch (error) {
      state = state.copyWith(error: await _repository.errorMessage(error));
    } finally {
      state = state.copyWith(saving: false);
    }
  }
}

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
