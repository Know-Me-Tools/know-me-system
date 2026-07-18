// TJ-ARCH-MOB-001 compliant
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

final themeModeProvider =
    NotifierProvider<ThemeModeNotifier, shad.ThemeMode>(ThemeModeNotifier.new);

class ThemeModeNotifier extends Notifier<shad.ThemeMode> {
  @override
  shad.ThemeMode build() => shad.ThemeMode.system;

  void select(shad.ThemeMode mode) => state = mode;
}
