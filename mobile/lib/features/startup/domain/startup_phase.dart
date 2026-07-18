// TJ-ARCH-MOB-001 compliant
/// The three ordered boot phases (analysis §1.8). Shapes fail on unknown
/// columns, so the order is an invariant: migrate first, seed second, attach
/// sync shapes last. Kept as an enum so the UI switch is exhaustive.
enum StartupPhase { migrations, seeds, shapes, ready }

extension StartupPhaseLabel on StartupPhase {
  String get label => switch (this) {
        StartupPhase.migrations => 'Applying migrations',
        StartupPhase.seeds => 'Loading seed data',
        StartupPhase.shapes => 'Attaching sync shapes',
        StartupPhase.ready => 'Ready',
      };
  double get progress => switch (this) {
        StartupPhase.migrations => 0.25,
        StartupPhase.seeds => 0.5,
        StartupPhase.shapes => 0.85,
        StartupPhase.ready => 1.0,
      };
}
