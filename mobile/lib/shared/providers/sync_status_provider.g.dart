// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'sync_status_provider.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// Read-only event feed from Rust — a stream provider (auto-dispose by default).
/// UI-only in effect, but it reaches the FFI, so retry stays off.

@ProviderFor(syncStatus)
final syncStatusProvider = SyncStatusProvider._();

/// Read-only event feed from Rust — a stream provider (auto-dispose by default).
/// UI-only in effect, but it reaches the FFI, so retry stays off.

final class SyncStatusProvider extends $FunctionalProvider<
        AsyncValue<SyncStatus>, SyncStatus, Stream<SyncStatus>>
    with $FutureModifier<SyncStatus>, $StreamProvider<SyncStatus> {
  /// Read-only event feed from Rust — a stream provider (auto-dispose by default).
  /// UI-only in effect, but it reaches the FFI, so retry stays off.
  SyncStatusProvider._()
      : super(
          from: null,
          argument: null,
          retry: _noRetry,
          name: r'syncStatusProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$syncStatusHash();

  @$internal
  @override
  $StreamProviderElement<SyncStatus> $createElement($ProviderPointer pointer) =>
      $StreamProviderElement(pointer);

  @override
  Stream<SyncStatus> create(Ref ref) {
    return syncStatus(ref);
  }
}

String _$syncStatusHash() => r'760960df105cf242b0340f948f275957e341fed4';
