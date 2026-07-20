// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'startup_notifier.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// Opens schema compatibility and sync once. Embedding models, demo seeds,
/// Xberg, and inference are deliberately lazy and never block the app shell.
/// [StartupPhase.ready] when the store is usable. A stream provider so the gate
/// rebuilds per phase; auto-dispose by default. Retry is off — see file header.

@ProviderFor(startup)
final startupProvider = StartupProvider._();

/// Opens schema compatibility and sync once. Embedding models, demo seeds,
/// Xberg, and inference are deliberately lazy and never block the app shell.
/// [StartupPhase.ready] when the store is usable. A stream provider so the gate
/// rebuilds per phase; auto-dispose by default. Retry is off — see file header.

final class StartupProvider
    extends
        $FunctionalProvider<
          AsyncValue<StartupPhase>,
          StartupPhase,
          Stream<StartupPhase>
        >
    with $FutureModifier<StartupPhase>, $StreamProvider<StartupPhase> {
  /// Opens schema compatibility and sync once. Embedding models, demo seeds,
  /// Xberg, and inference are deliberately lazy and never block the app shell.
  /// [StartupPhase.ready] when the store is usable. A stream provider so the gate
  /// rebuilds per phase; auto-dispose by default. Retry is off — see file header.
  StartupProvider._()
    : super(
        from: null,
        argument: null,
        retry: _noRetry,
        name: r'startupProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$startupHash();

  @$internal
  @override
  $StreamProviderElement<StartupPhase> $createElement(
    $ProviderPointer pointer,
  ) => $StreamProviderElement(pointer);

  @override
  Stream<StartupPhase> create(Ref ref) {
    return startup(ref);
  }
}

String _$startupHash() => r'c2dfa4f891b26e1c66c277cc67a93ddbac1df677';
