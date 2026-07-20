// TJ-ARCH-MOB-001 compliant
import '../../../bridge/rust_bridge_provider.dart' as bridge;

class ChatControlRepository {
  const ChatControlRepository();

  Future<Map<String, dynamic>> catalog() => bridge.chatControlCatalog();
  Future<Map<String, dynamic>?> policy(String conversationId) =>
      bridge.conversationPolicy(conversationId);
  Future<Map<String, dynamic>> savePolicy(
    String conversationId,
    Map<String, dynamic> policy,
  ) => bridge.saveConversationPolicy(conversationId, policy);
  Future<Map<String, dynamic>> effective(String conversationId) =>
      bridge.conversationEffectiveConfig(conversationId);
  Future<void> selectLane(String lane) => bridge.setActiveLane(lane);
  Future<String> errorMessage(Object error) => bridge.bridgeErrorMessage(error);
}
