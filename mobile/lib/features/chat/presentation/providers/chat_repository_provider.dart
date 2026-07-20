// TJ-ARCH-MOB-001 compliant
import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../data/demo_chat_repository.dart';
import '../../data/pem_chat_repository.dart';
import '../../domain/chat_repository.dart';

part 'chat_repository_provider.g.dart';

@Riverpod(keepAlive: true)
ChatRepository chatRepository(Ref ref) {
  const skipMobileBootForDemo = bool.fromEnvironment(
    'KNOWME_DEMO_SKIP_MOBILE_BOOT',
    defaultValue: false,
  );
  if (skipMobileBootForDemo) return DemoChatRepository();
  return PemChatRepository(ref.watch(entityTransportProvider));
}
