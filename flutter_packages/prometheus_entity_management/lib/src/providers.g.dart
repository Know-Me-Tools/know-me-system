// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// Host wiring seam. The app overrides this with its frb-backed adapter:
/// `entityTransportProvider.overrideWithValue(FrbEntityTransport(...))`.
/// Tests override it with a fake at the FFI boundary — nothing internal is mocked.

@ProviderFor(entityTransport)
final entityTransportProvider = EntityTransportProvider._();

/// Host wiring seam. The app overrides this with its frb-backed adapter:
/// `entityTransportProvider.overrideWithValue(FrbEntityTransport(...))`.
/// Tests override it with a fake at the FFI boundary — nothing internal is mocked.

final class EntityTransportProvider
    extends
        $FunctionalProvider<EntityTransport, EntityTransport, EntityTransport>
    with $Provider<EntityTransport> {
  /// Host wiring seam. The app overrides this with its frb-backed adapter:
  /// `entityTransportProvider.overrideWithValue(FrbEntityTransport(...))`.
  /// Tests override it with a fake at the FFI boundary — nothing internal is mocked.
  EntityTransportProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'entityTransportProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$entityTransportHash();

  @$internal
  @override
  $ProviderElement<EntityTransport> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  EntityTransport create(Ref ref) {
    return entityTransport(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(EntityTransport value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<EntityTransport>(value),
    );
  }
}

String _$entityTransportHash() => r'643285fc63aa9d0dee84bdaa3d76810e595155be';

/// One list instance per ViewDescriptor — the family key normalizes queries.

@ProviderFor(entityList)
final entityListProvider = EntityListFamily._();

/// One list instance per ViewDescriptor — the family key normalizes queries.

final class EntityListProvider
    extends
        $FunctionalProvider<
          AsyncValue<ListResult>,
          ListResult,
          FutureOr<ListResult>
        >
    with $FutureModifier<ListResult>, $FutureProvider<ListResult> {
  /// One list instance per ViewDescriptor — the family key normalizes queries.
  EntityListProvider._({
    required EntityListFamily super.from,
    required ViewDescriptor super.argument,
  }) : super(
         retry: _noRetry,
         name: r'entityListProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$entityListHash();

  @override
  String toString() {
    return r'entityListProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<ListResult> $createElement($ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<ListResult> create(Ref ref) {
    final argument = this.argument as ViewDescriptor;
    return entityList(ref, argument);
  }

  @override
  bool operator ==(Object other) {
    return other is EntityListProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$entityListHash() => r'a40b8fd639b528f15a3bcc0d4333edef052ae83c';

/// One list instance per ViewDescriptor — the family key normalizes queries.

final class EntityListFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<ListResult>, ViewDescriptor> {
  EntityListFamily._()
    : super(
        retry: _noRetry,
        name: r'entityListProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  /// One list instance per ViewDescriptor — the family key normalizes queries.

  EntityListProvider call(ViewDescriptor view) =>
      EntityListProvider._(argument: view, from: this);

  @override
  String toString() => r'entityListProvider';
}

/// One record instance per (type, id). Features decode `dataJson` themselves.

@ProviderFor(entity)
final entityProvider = EntityFamily._();

/// One record instance per (type, id). Features decode `dataJson` themselves.

final class EntityProvider
    extends
        $FunctionalProvider<
          AsyncValue<EntityRecord?>,
          EntityRecord?,
          FutureOr<EntityRecord?>
        >
    with $FutureModifier<EntityRecord?>, $FutureProvider<EntityRecord?> {
  /// One record instance per (type, id). Features decode `dataJson` themselves.
  EntityProvider._({
    required EntityFamily super.from,
    required (String, String) super.argument,
  }) : super(
         retry: _noRetry,
         name: r'entityProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$entityHash();

  @override
  String toString() {
    return r'entityProvider'
        ''
        '$argument';
  }

  @$internal
  @override
  $FutureProviderElement<EntityRecord?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<EntityRecord?> create(Ref ref) {
    final argument = this.argument as (String, String);
    return entity(ref, argument.$1, argument.$2);
  }

  @override
  bool operator ==(Object other) {
    return other is EntityProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$entityHash() => r'b10dbbaebb0261ef76dfe4cc0d060e6df653f383';

/// One record instance per (type, id). Features decode `dataJson` themselves.

final class EntityFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<EntityRecord?>, (String, String)> {
  EntityFamily._()
    : super(
        retry: _noRetry,
        name: r'entityProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  /// One record instance per (type, id). Features decode `dataJson` themselves.

  EntityProvider call(String entityType, String id) =>
      EntityProvider._(argument: (entityType, id), from: this);

  @override
  String toString() => r'entityProvider';
}

/// Bridges the single Rust ChangeEvent feed into targeted invalidations. Mount
/// it once (e.g. `ref.watch(entityChangeBridgeProvider)` at app root). It never
/// holds state — it only translates change ops into `ref.invalidate`.

@ProviderFor(entityChangeBridge)
final entityChangeBridgeProvider = EntityChangeBridgeProvider._();

/// Bridges the single Rust ChangeEvent feed into targeted invalidations. Mount
/// it once (e.g. `ref.watch(entityChangeBridgeProvider)` at app root). It never
/// holds state — it only translates change ops into `ref.invalidate`.

final class EntityChangeBridgeProvider
    extends
        $FunctionalProvider<
          AsyncValue<ChangeEvent>,
          ChangeEvent,
          Stream<ChangeEvent>
        >
    with $FutureModifier<ChangeEvent>, $StreamProvider<ChangeEvent> {
  /// Bridges the single Rust ChangeEvent feed into targeted invalidations. Mount
  /// it once (e.g. `ref.watch(entityChangeBridgeProvider)` at app root). It never
  /// holds state — it only translates change ops into `ref.invalidate`.
  EntityChangeBridgeProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'entityChangeBridgeProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$entityChangeBridgeHash();

  @$internal
  @override
  $StreamProviderElement<ChangeEvent> $createElement(
    $ProviderPointer pointer,
  ) => $StreamProviderElement(pointer);

  @override
  Stream<ChangeEvent> create(Ref ref) {
    return entityChangeBridge(ref);
  }
}

String _$entityChangeBridgeHash() =>
    r'626230c4449e6f6b8a40fa31c9f39b8087a16b09';

/// CRUD controller for one entity type. Composes the list family, an edit
/// buffer, and optimistic snapshot/rollback. It performs writes through the
/// transport (FFI → Rust) and reconciles via the ChangeEvent bridge, so the
/// canonical store stays in Rust and the UI holds only edit-in-flight state.

@ProviderFor(EntityCrud)
final entityCrudProvider = EntityCrudFamily._();

/// CRUD controller for one entity type. Composes the list family, an edit
/// buffer, and optimistic snapshot/rollback. It performs writes through the
/// transport (FFI → Rust) and reconciles via the ChangeEvent bridge, so the
/// canonical store stays in Rust and the UI holds only edit-in-flight state.
final class EntityCrudProvider
    extends $NotifierProvider<EntityCrud, EditBuffer> {
  /// CRUD controller for one entity type. Composes the list family, an edit
  /// buffer, and optimistic snapshot/rollback. It performs writes through the
  /// transport (FFI → Rust) and reconciles via the ChangeEvent bridge, so the
  /// canonical store stays in Rust and the UI holds only edit-in-flight state.
  EntityCrudProvider._({
    required EntityCrudFamily super.from,
    required (String, String, Map<String, Object?>) super.argument,
  }) : super(
         retry: null,
         name: r'entityCrudProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$entityCrudHash();

  @override
  String toString() {
    return r'entityCrudProvider'
        ''
        '$argument';
  }

  @$internal
  @override
  EntityCrud create() => EntityCrud();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(EditBuffer value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<EditBuffer>(value),
    );
  }

  @override
  bool operator ==(Object other) {
    return other is EntityCrudProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$entityCrudHash() => r'a680ff02d46e3e9ed736e8c7ce47af15c1b1e304';

/// CRUD controller for one entity type. Composes the list family, an edit
/// buffer, and optimistic snapshot/rollback. It performs writes through the
/// transport (FFI → Rust) and reconciles via the ChangeEvent bridge, so the
/// canonical store stays in Rust and the UI holds only edit-in-flight state.

final class EntityCrudFamily extends $Family
    with
        $ClassFamilyOverride<
          EntityCrud,
          EditBuffer,
          EditBuffer,
          EditBuffer,
          (String, String, Map<String, Object?>)
        > {
  EntityCrudFamily._()
    : super(
        retry: null,
        name: r'entityCrudProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  /// CRUD controller for one entity type. Composes the list family, an edit
  /// buffer, and optimistic snapshot/rollback. It performs writes through the
  /// transport (FFI → Rust) and reconciles via the ChangeEvent bridge, so the
  /// canonical store stays in Rust and the UI holds only edit-in-flight state.

  EntityCrudProvider call(
    String entityType,
    String id,
    Map<String, Object?> initial,
  ) => EntityCrudProvider._(argument: (entityType, id, initial), from: this);

  @override
  String toString() => r'entityCrudProvider';
}

/// CRUD controller for one entity type. Composes the list family, an edit
/// buffer, and optimistic snapshot/rollback. It performs writes through the
/// transport (FFI → Rust) and reconciles via the ChangeEvent bridge, so the
/// canonical store stays in Rust and the UI holds only edit-in-flight state.

abstract class _$EntityCrud extends $Notifier<EditBuffer> {
  late final _$args = ref.$arg as (String, String, Map<String, Object?>);
  String get entityType => _$args.$1;
  String get id => _$args.$2;
  Map<String, Object?> get initial => _$args.$3;

  EditBuffer build(String entityType, String id, Map<String, Object?> initial);
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<EditBuffer, EditBuffer>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<EditBuffer, EditBuffer>,
              EditBuffer,
              Object?,
              Object?
            >;
    return element.handleCreate(
      ref,
      () => build(_$args.$1, _$args.$2, _$args.$3),
    );
  }
}
