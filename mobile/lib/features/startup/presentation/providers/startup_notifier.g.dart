// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'startup_notifier.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// Runs migrations → seeds → shapes once, yielding each phase as it begins and
/// [StartupPhase.ready] when the store is usable. A stream provider so the gate
/// rebuilds per phase; auto-dispose by default. Retry is off — see file header.

@ProviderFor(startup)
final startupProvider = StartupProvider._();

/// Runs migrations → seeds → shapes once, yielding each phase as it begins and
/// [StartupPhase.ready] when the store is usable. A stream provider so the gate
/// rebuilds per phase; auto-dispose by default. Retry is off — see file header.

final class StartupProvider extends $FunctionalProvider<
        AsyncValue<StartupPhase>, StartupPhase, Stream<StartupPhase>>
    with $FutureModifier<StartupPhase>, $StreamProvider<StartupPhase> {
  /// Runs migrations → seeds → shapes once, yielding each phase as it begins and
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
          $ProviderPointer pointer) =>
      $StreamProviderElement(pointer);

  @override
  Stream<StartupPhase> create(Ref ref) {
    return startup(ref);
  }
}

String _$startupHash() => r'd5df62aed7fa44f9f95a690fdb21c6da55758d4a';
