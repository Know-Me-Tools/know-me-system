// TJ-ARCH-MOB-001 compliant
import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../bridge/rust_bridge_provider.dart' as bridge;

part 'sync_status_provider.g.dart';

/// Read-only event feed from Rust — a stream provider (auto-dispose by default).
/// UI-only in effect, but it reaches the FFI, so retry stays off.
@Riverpod(retry: _noRetry)
Stream<SyncStatus> syncStatus(Ref ref) {
  const skipMobileBootForDemo = bool.fromEnvironment(
    'KNOWME_DEMO_SKIP_MOBILE_BOOT',
    defaultValue: false,
  );
  if (skipMobileBootForDemo) return Stream.value(const SyncStatus.offline());
  return bridge.syncStatus();
}

Duration? _noRetry(int retryCount, Object error) => null;
