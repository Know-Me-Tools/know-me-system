// TJ-ARCH-MOB-001 compliant
import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../features/startup/domain/startup_phase.dart';
import '../../features/startup/presentation/providers/startup_notifier.dart';
import 'theme_repository.dart';

final themeRepositoryProvider = Provider<ThemeRepository>(
  (_) => const ThemeRepository(),
);

final themeModeProvider = NotifierProvider<ThemeModeNotifier, shad.ThemeMode>(
  ThemeModeNotifier.new,
);

class ThemeModeNotifier extends Notifier<shad.ThemeMode> {
  bool _loaded = false;

  @override
  shad.ThemeMode build() {
    final startup = ref.watch(startupProvider);
    if (!_loaded && startup.value == StartupPhase.ready) {
      _loaded = true;
      unawaited(_load());
    }
    return shad.ThemeMode.system;
  }

  Future<void> _load() async {
    try {
      final value = await ref.read(themeRepositoryProvider).load();
      if (!ref.mounted) return;
      state = _decode(value);
    } on Object {
      // System mode remains the safe default if the preference is unavailable.
    }
  }

  Future<void> select(shad.ThemeMode mode) async {
    state = mode;
    try {
      await ref.read(themeRepositoryProvider).save(mode.name);
    } on Object catch (error, stackTrace) {
      // A transient persistence failure must not make the visible setting
      // appear broken. Keep the user's active selection and report the failed
      // durable write so diagnostics retain the underlying cause.
      FlutterError.reportError(
        FlutterErrorDetails(
          exception: error,
          stack: stackTrace,
          library: 'KnowMe theme settings',
          context: ErrorDescription('while persisting the selected theme'),
        ),
      );
    }
  }

  shad.ThemeMode _decode(String value) => switch (value) {
    'light' => shad.ThemeMode.light,
    'dark' => shad.ThemeMode.dark,
    _ => shad.ThemeMode.system,
  };
}
