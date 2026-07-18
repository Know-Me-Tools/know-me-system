# prometheus_entity_management

Rust-backed entity management for Flutter. The canonical store lives in Rust
(`gen_ui_core`, reached via the `gen_ui_flutter` FFI plugin) — this package does
**not** re-implement a Dart graph store. Riverpod provider *families* are the
normalization map:

- `entityListProvider(view)` — one instance per `ViewDescriptor`
- `entityProvider(type, id)` — one instance per record
- `entityCrudProvider(type, id, initial)` — edit buffer + optimistic save/rollback
- `entityChangeBridgeProvider` — folds the single Rust `ChangeEvent` feed into
  targeted `ref.invalidate` calls (mount once at app root)

The view/entity/change types are freezed mirrors of `gen_ui_types` and serialize
snake_case to match the Rust serde wire format 1:1.

## Wiring (host app)

```dart
final container = ProviderScope(
  overrides: [
    entityTransportProvider.overrideWithValue(FrbEntityTransport(genUiCore)),
  ],
  child: const App(),
);
```

`FrbEntityTransport` is a thin adapter over the frb-generated `entityList` /
`entityGet` / `entityCreate` / `entityUpdate` / `entityDelete` / `entityChanges`
functions. Nothing above the transport touches the FFI directly, and no internal
code is mocked in tests — a fake `EntityTransport` at this boundary is the only
seam tests override.

## Codegen

```bash
flutter pub run build_runner build
```

Generates the `*.freezed.dart` and `*.g.dart` parts. All FFI-backed providers
opt out of Riverpod 3 automatic retry (a Rust domain error is terminal).
