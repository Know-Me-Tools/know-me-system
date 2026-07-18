// TJ-ARCH-MOB-001 compliant
/// gen_ui_flutter — public surface. Re-exports the flutter_rust_bridge generated
/// bindings (chat/entity/memory intents + ContentBlock event streams) plus the
/// init helper. The generated code lives in the host app's lib/bridge (produced
/// by `flutter_rust_bridge_codegen generate`); this package wraps the ergonomics.
library;

export 'src/gen_ui.dart';
