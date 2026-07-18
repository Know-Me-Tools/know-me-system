// TJ-ARCH-MOB-001 compliant
// StartupGate blocks the app shell until the first-run boot sequence reaches
// StartupPhase.ready, rendering the current phase + progress. On error it shows
// the failure (no silent retry — a broken migration must be visible). Once
// ready, it renders [child]. Presentation only.
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../core/theme/tokens.dart';
import '../../domain/startup_phase.dart';
import '../providers/startup_notifier.dart';

class StartupGate extends ConsumerWidget {
  const StartupGate({required this.child, super.key});
  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final phase = ref.watch(startupProvider);
    return phase.when(
      data: (p) => p == StartupPhase.ready ? child : _BootScreen(phase: p),
      loading: () => const _BootScreen(phase: StartupPhase.migrations),
      error: (e, _) => _BootError(message: '$e'),
    );
  }
}

class _BootScreen extends StatelessWidget {
  const _BootScreen({required this.phase});
  final StartupPhase phase;

  @override
  Widget build(BuildContext context) => Scaffold(
        body: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              SizedBox(
                width: 200,
                child: LinearProgressIndicator(value: phase.progress),
              ),
              const SizedBox(height: 16),
              Text(phase.label, style: T.uiMd.copyWith(color: T.textSecondary)),
            ],
          ),
        ),
      );
}

class _BootError extends StatelessWidget {
  const _BootError({required this.message});
  final String message;

  @override
  Widget build(BuildContext context) => Scaffold(
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.error_outline, color: T.red, size: 40),
                const SizedBox(height: 12),
                Text('Startup failed', style: T.uiMd.copyWith(color: T.red)),
                const SizedBox(height: 8),
                Text(
                  message,
                  textAlign: TextAlign.center,
                  style: T.uiMd.copyWith(color: T.textTertiary),
                ),
              ],
            ),
          ),
        ),
      );
}
