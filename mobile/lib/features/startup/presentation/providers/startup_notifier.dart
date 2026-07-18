// TJ-ARCH-MOB-001 compliant
// StartupNotifier drives the first-run boot sequence and exposes the current
// phase to a gate widget. The real work (schema migrations, seed bundles, sync
// shape attach) runs in Rust's gen_ui_db startup orchestrator over the FFI
// facade; here we sequence the intents in the invariant order and surface
// progress. FFI-backed → retry OFF: a failed migration is terminal, not
// transient, and must be shown, not silently retried.
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../../../bridge/rust_bridge_provider.dart' as bridge;
import '../../../../shared/providers/data_dir.dart';
import '../../domain/startup_phase.dart';

part 'startup_notifier.g.dart';

Duration? _noRetry(int retryCount, Object error) => null;

/// Runs migrations → seeds → shapes once, yielding each phase as it begins and
/// [StartupPhase.ready] when the store is usable. A stream provider so the gate
/// rebuilds per phase; auto-dispose by default. Retry is off — see file header.
@Riverpod(retry: _noRetry)
Stream<StartupPhase> startup(Ref ref) async* {
  yield StartupPhase.migrations;
  await bridge.runMigrations(dataDir: dataDirOverride);
  yield StartupPhase.seeds;
  await bridge.loadSeeds();
  yield StartupPhase.shapes;
  await bridge.attachSyncShapes();
  yield StartupPhase.ready;
}
