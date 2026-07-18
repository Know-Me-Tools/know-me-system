// TJ-ARCH-MOB-001 compliant
/// prometheus_entity_management — Rust-backed entity management for Flutter.
///
/// The canonical entity store lives in Rust (gen_ui_core); this package exposes
/// it to Flutter through Riverpod, using provider families as the normalization
/// map. See analysis §1.5. Wire `entityTransportProvider` to a gen_ui_flutter
/// adapter, mount `entityChangeBridgeProvider` once, then read
/// `entityListProvider(view)` / `entityProvider(type, id)` and drive edits
/// through `entityCrudProvider`.
library;

export 'src/view.dart';
export 'src/entity.dart';
export 'src/sync.dart';
export 'src/transport.dart';
export 'src/providers.dart';
