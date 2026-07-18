// TJ-ARCH-MOB-001 compliant
import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../data/pem_chat_repository.dart';
import '../../domain/chat_repository.dart';

part 'chat_repository_provider.g.dart';

@Riverpod(keepAlive: true)
ChatRepository chatRepository(Ref ref) =>
    PemChatRepository(ref.watch(entityTransportProvider));
