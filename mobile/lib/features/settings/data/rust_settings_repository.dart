// TJ-ARCH-MOB-001 compliant
import '../../../bridge/rust_bridge_provider.dart' as bridge;
import '../domain/settings_repository.dart';
import '../domain/settings_snapshot.dart';

class RustSettingsRepository implements SettingsRepository {
  const RustSettingsRepository();

  @override
  Future<SettingsSnapshot> load() async {
    final lane = await bridge.activeLane();
    final localModel = await bridge.localModelStatus();
    final localDiagnostics = await bridge.localRuntimeDiagnostics() ?? const {};
    Map<String, dynamic> uarConfig = const {};
    Map<String, dynamic> admin = const {};
    Map<String, dynamic> catalog = const {};
    List<Map<String, dynamic>> providers = const [];
    List<Map<String, dynamic>> configuredProviders = const [];
    Map<String, dynamic>? cloudModel;
    try {
      uarConfig = await bridge.uarServiceConfig();
      if ((uarConfig['serviceUrl']?.toString() ?? '').isNotEmpty) {
        admin = await bridge.uarAdminSnapshot();
        catalog = await bridge.chatControlCatalog();
        providers = await bridge.providerCatalog();
        configuredProviders = await bridge.configuredProviders();
        cloudModel = await bridge.cloudModel();
      }
    } catch (_) {
      // A local-only installation remains fully usable. The notifier exposes
      // connection errors for explicit user operations such as Probe/Save.
    }
    return SettingsSnapshot(
      activeLane: lane,
      uarConfig: uarConfig,
      admin: admin,
      catalog: catalog,
      localModel: localModel,
      localDiagnostics: localDiagnostics,
      providers: providers,
      configuredProviders: configuredProviders,
      cloudModel: cloudModel,
    );
  }

  @override
  Future<void> selectLane(String lane) => bridge.setActiveLane(lane);

  @override
  Future<void> saveRuntime({
    required String serviceUrl,
    String? apiKey,
    String? sessionId,
  }) => bridge.saveUarService(
    serviceUrl: serviceUrl,
    apiKey: apiKey,
    sessionId: sessionId,
  );

  @override
  Future<Map<String, dynamic>> probeRuntime() => bridge.probeUarService();

  @override
  Future<void> saveSetting(String key, Object? value) async {
    await bridge.saveUarSetting(key, value);
  }

  @override
  Future<void> saveAgent(Map<String, dynamic> agent) async {
    await bridge.saveUarAgent(agent);
  }

  @override
  Future<void> deleteAgent(String id) => bridge.deleteUarAgent(id);

  @override
  Future<void> saveMcpServer(String name, Map<String, dynamic> server) async {
    await bridge.saveUarMcpServer(name, server);
  }

  @override
  Future<void> deleteMcpServer(String name) async {
    await bridge.deleteUarMcpServer(name);
  }

  @override
  Future<void> deleteMemory(String id) async {
    await bridge.deleteUarMemory(id);
  }

  @override
  Future<void> createKnowledgeBase(Map<String, dynamic> request) async {
    await bridge.createUarKnowledgeBase(request);
  }

  @override
  Future<void> deleteKnowledgeBase(String id) =>
      bridge.deleteUarKnowledgeBase(id);

  @override
  Future<void> prepareLocalModel() => bridge.prepareLocalModel();

  @override
  Future<void> cancelModelOperation() => bridge.cancelModelOperation();

  @override
  Future<void> saveProvider(Map<String, dynamic> request) =>
      bridge.saveProvider(request);

  @override
  Future<void> deleteProvider(String id) => bridge.deleteProvider(id);

  @override
  Future<Map<String, dynamic>> testProvider(String id, String? model) =>
      bridge.testProvider(id, model);

  @override
  Future<void> saveCloudModel(String providerId, String modelId) =>
      bridge.saveCloudModel(providerId, modelId);

  @override
  Future<String> errorMessage(Object error) => bridge.bridgeErrorMessage(error);
}
