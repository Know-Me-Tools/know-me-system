// TJ-ARCH-MOB-001 compliant
import 'package:flutter_test/flutter_test.dart';

import 'package:knowme_poc/features/startup/domain/startup_phase.dart';

void main() {
  test('boot phases are ordered migrations → seeds → shapes → ready', () {
    // The order is an invariant: sync shapes fail on unknown columns, so
    // migrations and seeds MUST precede shape attach (analysis §1.8).
    expect(StartupPhase.values, [
      StartupPhase.migrations,
      StartupPhase.seeds,
      StartupPhase.shapes,
      StartupPhase.ready,
    ]);
    // Progress is monotonic across the boot order.
    final progresses = StartupPhase.values.map((p) => p.progress).toList();
    for (var i = 1; i < progresses.length; i++) {
      expect(progresses[i], greaterThan(progresses[i - 1]));
    }
    expect(StartupPhase.ready.progress, 1.0);
  });
}
