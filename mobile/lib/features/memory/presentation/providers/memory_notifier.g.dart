// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'memory_notifier.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(MemoryNotifier)
final memoryProvider = MemoryNotifierProvider._();

final class MemoryNotifierProvider
    extends $NotifierProvider<MemoryNotifier, MemoryResult> {
  MemoryNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'memoryProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$memoryNotifierHash();

  @$internal
  @override
  MemoryNotifier create() => MemoryNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(MemoryResult value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<MemoryResult>(value),
    );
  }
}

String _$memoryNotifierHash() => r'da31a3d059a6c43a12e760de299de4919690a4da';

abstract class _$MemoryNotifier extends $Notifier<MemoryResult> {
  MemoryResult build();
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<MemoryResult, MemoryResult>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<MemoryResult, MemoryResult>,
        MemoryResult,
        Object?,
        Object?>;
    return element.handleCreate(ref, build);
  }
}
