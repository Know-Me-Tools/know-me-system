// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'notes_provider.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(notesRepository)
final notesRepositoryProvider = NotesRepositoryProvider._();

final class NotesRepositoryProvider extends $FunctionalProvider<NotesRepository,
    NotesRepository, NotesRepository> with $Provider<NotesRepository> {
  NotesRepositoryProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'notesRepositoryProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$notesRepositoryHash();

  @$internal
  @override
  $ProviderElement<NotesRepository> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  NotesRepository create(Ref ref) {
    return notesRepository(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(NotesRepository value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<NotesRepository>(value),
    );
  }
}

String _$notesRepositoryHash() => r'f887a0613df4e8cdba766b932069bfb48f7aef15';

@ProviderFor(notes)
final notesProvider = NotesProvider._();

final class NotesProvider extends $FunctionalProvider<AsyncValue<List<Note>>,
        List<Note>, FutureOr<List<Note>>>
    with $FutureModifier<List<Note>>, $FutureProvider<List<Note>> {
  NotesProvider._()
      : super(
          from: null,
          argument: null,
          retry: _noRetry,
          name: r'notesProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$notesHash();

  @$internal
  @override
  $FutureProviderElement<List<Note>> $createElement($ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<Note>> create(Ref ref) {
    return notes(ref);
  }
}

String _$notesHash() => r'815b91a020b181aa40087724c04fe7df2603bea2';

@ProviderFor(NotesActions)
final notesActionsProvider = NotesActionsProvider._();

final class NotesActionsProvider
    extends $AsyncNotifierProvider<NotesActions, void> {
  NotesActionsProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'notesActionsProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$notesActionsHash();

  @$internal
  @override
  NotesActions create() => NotesActions();
}

String _$notesActionsHash() => r'e5234791d56dcc0fa7eaf987bf76f43689e260ac';

abstract class _$NotesActions extends $AsyncNotifier<void> {
  FutureOr<void> build();
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<AsyncValue<void>, void>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<void>, void>,
        AsyncValue<void>,
        Object?,
        Object?>;
    return element.handleCreate(ref, build);
  }
}
