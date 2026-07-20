// TJ-ARCH-MOB-001 compliant
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:flutter/foundation.dart';

import '../../data/rust_settings_repository.dart';
import '../../domain/settings_repository.dart';
import '../../domain/settings_snapshot.dart';

part 'settings_notifier.g.dart';

@immutable
class SettingsState {
  const SettingsState({
    this.snapshot = const SettingsSnapshot(),
    this.loading = false,
    this.saving = false,
    this.error,
    this.notice,
  });

  final SettingsSnapshot snapshot;
  final bool loading;
  final bool saving;
  final String? error;
  final String? notice;

  SettingsState copyWith({
    SettingsSnapshot? snapshot,
    bool? loading,
    bool? saving,
    String? error,
    String? notice,
    bool clearError = false,
    bool clearNotice = false,
  }) => SettingsState(
    snapshot: snapshot ?? this.snapshot,
    loading: loading ?? this.loading,
    saving: saving ?? this.saving,
    error: clearError ? null : error ?? this.error,
    notice: clearNotice ? null : notice ?? this.notice,
  );
}

@Riverpod(keepAlive: true)
class SettingsNotifier extends _$SettingsNotifier {
  SettingsRepository get _repository => const RustSettingsRepository();

  @override
  SettingsState build() => const SettingsState();

  Future<void> load() async {
    if (state.loading) return;
    state = state.copyWith(loading: true, clearError: true);
    try {
      state = state.copyWith(snapshot: await _repository.load());
    } catch (error) {
      state = state.copyWith(error: await _repository.errorMessage(error));
    } finally {
      state = state.copyWith(loading: false);
    }
  }

  Future<void> selectLane(String lane) => _mutate(() async {
    await _repository.selectLane(lane);
    state = state.copyWith(
      snapshot: state.snapshot.copyWith(activeLane: lane),
      notice: lane == 'local'
          ? 'Chat now uses the on-device model.'
          : 'Chat now runs through the Universal Agent Runtime.',
    );
  });

  Future<void> saveRuntime({
    required String serviceUrl,
    String? apiKey,
    String? sessionId,
  }) => _mutate(() async {
    await _repository.saveRuntime(
      serviceUrl: serviceUrl,
      apiKey: apiKey,
      sessionId: sessionId,
    );
    state = state.copyWith(notice: 'UAR connection saved.');
    await _reloadSnapshot();
  });

  Future<void> probeRuntime() => _mutate(() async {
    final result = await _repository.probeRuntime();
    state = state.copyWith(
      notice: result['message']?.toString() ?? 'UAR connection checked.',
    );
  });

  Future<void> saveSetting(String key, Object? value) => _mutate(() async {
    await _repository.saveSetting(key, value);
    state = state.copyWith(notice: 'Setting saved by UAR.');
    await _reloadSnapshot();
  });

  Future<void> saveAgent(Map<String, dynamic> agent) => _mutate(() async {
    await _repository.saveAgent(agent);
    state = state.copyWith(notice: 'Agent saved by UAR.');
    await _reloadSnapshot();
  });

  Future<void> deleteAgent(String id) => _mutate(() async {
    await _repository.deleteAgent(id);
    state = state.copyWith(notice: 'Agent deleted.');
    await _reloadSnapshot();
  });

  Future<void> saveMcpServer(String name, Map<String, dynamic> server) =>
      _mutate(() async {
        await _repository.saveMcpServer(name, server);
        state = state.copyWith(notice: 'MCP server saved by UAR.');
        await _reloadSnapshot();
      });

  Future<void> deleteMcpServer(String name) => _mutate(() async {
    await _repository.deleteMcpServer(name);
    state = state.copyWith(notice: 'MCP server removed.');
    await _reloadSnapshot();
  });

  Future<void> deleteMemory(String id) => _mutate(() async {
    await _repository.deleteMemory(id);
    state = state.copyWith(notice: 'Memory removed.');
    await _reloadSnapshot();
  });

  Future<void> createKnowledgeBase(String name, String description) =>
      _mutate(() async {
        await _repository.createKnowledgeBase({
          'name': name,
          'description': description.trim().isEmpty ? null : description.trim(),
        });
        state = state.copyWith(notice: 'Knowledge base created.');
        await _reloadSnapshot();
      });

  Future<void> deleteKnowledgeBase(String id) => _mutate(() async {
    await _repository.deleteKnowledgeBase(id);
    state = state.copyWith(notice: 'Knowledge base deleted.');
    await _reloadSnapshot();
  });

  Future<void> prepareLocalModel() => _mutate(() async {
    await _repository.prepareLocalModel();
    state = state.copyWith(notice: 'Local model preparation started.');
    await _reloadSnapshot();
  });

  Future<void> cancelModelOperation() => _mutate(() async {
    await _repository.cancelModelOperation();
    state = state.copyWith(notice: 'Local model operation cancelled.');
    await _reloadSnapshot();
  });

  Future<void> saveProvider(Map<String, dynamic> request) => _mutate(() async {
    await _repository.saveProvider(request);
    final defaultModel = request['defaultModel']?.toString();
    if (defaultModel != null && defaultModel.isNotEmpty) {
      await _repository.saveCloudModel(request['id'].toString(), defaultModel);
    }
    state = state.copyWith(notice: 'Provider configuration saved by UAR.');
    await _reloadSnapshot();
  });

  Future<void> deleteProvider(String id) => _mutate(() async {
    await _repository.deleteProvider(id);
    state = state.copyWith(notice: 'Provider configuration removed.');
    await _reloadSnapshot();
  });

  Future<void> testProvider(String id, String? model) => _mutate(() async {
    final result = await _repository.testProvider(id, model);
    state = state.copyWith(
      notice: result['ok'] == true
          ? 'Provider connected with ${result['modelId']} in ${result['latencyMs']} ms.'
          : 'Provider test did not return a usable response.',
    );
  });

  Future<void> _reloadSnapshot() async {
    state = state.copyWith(snapshot: await _repository.load());
  }

  Future<void> _mutate(Future<void> Function() action) async {
    if (state.saving) return;
    state = state.copyWith(saving: true, clearError: true, clearNotice: true);
    try {
      await action();
    } catch (error) {
      state = state.copyWith(error: await _repository.errorMessage(error));
    } finally {
      state = state.copyWith(saving: false);
    }
  }
}
