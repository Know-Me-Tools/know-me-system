// TJ-ARCH-MOB-001 compliant
import 'settings_snapshot.dart';

abstract interface class SettingsRepository {
  Future<SettingsSnapshot> load();
  Future<void> selectLane(String lane);
  Future<void> saveRuntime({
    required String serviceUrl,
    String? apiKey,
    String? sessionId,
  });
  Future<Map<String, dynamic>> probeRuntime();
  Future<void> saveSetting(String key, Object? value);

  /// Persist the local lane's system prompt. A blank value clears the override
  /// so Rust's shipped default applies again.
  Future<void> saveLocalSystemPrompt(String prompt);

  /// The prompt actually in effect — the stored override, or Rust's default when
  /// none is set. Read back after a save so a cleared value shows the restored
  /// default rather than an empty editor.
  Future<String> localSystemPrompt();

  /// Rust's shipped default, used by the reset action so the copy lives in one
  /// place instead of being duplicated in Dart.
  Future<String> defaultLocalSystemPrompt();
  Future<void> saveAgent(Map<String, dynamic> agent);
  Future<void> deleteAgent(String id);
  Future<void> saveMcpServer(String name, Map<String, dynamic> server);
  Future<void> deleteMcpServer(String name);
  Future<void> deleteMemory(String id);
  Future<void> createKnowledgeBase(Map<String, dynamic> request);
  Future<void> deleteKnowledgeBase(String id);
  Future<void> prepareLocalModel();
  Future<void> cancelModelOperation();
  Future<void> saveProvider(Map<String, dynamic> request);
  Future<void> deleteProvider(String id);
  Future<Map<String, dynamic>> testProvider(String id, String? model);
  Future<void> saveCloudModel(String providerId, String modelId);
  Future<String> errorMessage(Object error);
}
