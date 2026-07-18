// TJ-ARCH-MOB-001 compliant
/// SyncStatus — Dart mirror of gen_ui_types::sync::SyncStatus. Drives the UI
/// sync chip. Sealed: a switch that misses a variant is a compile error.
///
/// fromJson mirrors the Rust enum's externally-tagged serde shape (no `tag`
/// attribute, `rename_all = "snake_case"`): unit variants serialize as a bare
/// string (`"offline"`), struct variants as a single-key object
/// (`{"syncing": {"pending_writes": N}}`).
sealed class SyncStatus {
  const SyncStatus();
  const factory SyncStatus.offline() = SyncOffline;
  const factory SyncStatus.syncing(int pendingWrites) = SyncSyncing;
  const factory SyncStatus.live() = SyncLive;
  const factory SyncStatus.error(String message) = SyncError;

  factory SyncStatus.fromJson(dynamic json) => switch (json) {
        'offline' => const SyncStatus.offline(),
        'live' => const SyncStatus.live(),
        {'syncing': {'pending_writes': final int pendingWrites}} =>
          SyncStatus.syncing(pendingWrites),
        {'error': {'message': final String message}} =>
          SyncStatus.error(message),
        _ => throw FormatException('Unknown SyncStatus: $json'),
      };
}

class SyncOffline extends SyncStatus {
  const SyncOffline();
}

class SyncSyncing extends SyncStatus {
  final int pendingWrites;
  const SyncSyncing(this.pendingWrites);
}

class SyncLive extends SyncStatus {
  const SyncLive();
}

class SyncError extends SyncStatus {
  final String message;
  const SyncError(this.message);
}
