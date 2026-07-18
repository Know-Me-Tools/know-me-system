// TJ-ARCH-MOB-001 compliant
// Flutter projection of the shared KnowMe semantic palette in desktop/index.css
// and docs/knowme-ui-ux-standard.md. Components consume semantic roles only.
import 'package:flutter/widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

abstract final class KnowMeSpace {
  static const x1 = 4.0;
  static const x2 = 8.0;
  static const x3 = 12.0;
  static const x4 = 16.0;
  static const x5 = 20.0;
  static const x6 = 24.0;
}

@immutable
class KnowMePalette {
  const KnowMePalette({
    required this.canvas,
    required this.chrome,
    required this.surface,
    required this.raised,
    required this.hover,
    required this.muted,
    required this.text,
    required this.subtleText,
    required this.faintText,
    required this.accent,
    required this.accentSoft,
    required this.onAccent,
    required this.success,
    required this.warning,
    required this.info,
    required this.destructive,
  });

  final Color canvas;
  final Color chrome;
  final Color surface;
  final Color raised;
  final Color hover;
  final Color muted;
  final Color text;
  final Color subtleText;
  final Color faintText;
  final Color accent;
  final Color accentSoft;
  final Color onAccent;
  final Color success;
  final Color warning;
  final Color info;
  final Color destructive;

  static const dark = KnowMePalette(
    canvas: Color(0xFF0B0F14),
    chrome: Color(0xFF111620),
    surface: Color(0xFF161D29),
    raised: Color(0xFF1C2535),
    hover: Color(0xFF202B40),
    muted: Color(0xFF253044),
    text: Color(0xFFE8EDF3),
    subtleText: Color(0xFFA7B0BC),
    faintText: Color(0xFF8B96A5),
    accent: Color(0xFFFF6A3D),
    accentSoft: Color(0xFF2B1A18),
    onAccent: Color(0xFF0B0F14),
    success: Color(0xFF34D399),
    warning: Color(0xFFF5A623),
    info: Color(0xFF22D3EE),
    destructive: Color(0xFFF87171),
  );

  static const light = KnowMePalette(
    canvas: Color(0xFFF7F7F8),
    chrome: Color(0xFFFFFFFF),
    surface: Color(0xFFFAFBFC),
    raised: Color(0xFFFFFFFF),
    hover: Color(0xFFF2F4F7),
    muted: Color(0xFFF2F4F7),
    text: Color(0xFF0B0F14),
    subtleText: Color(0xFF4B5563),
    faintText: Color(0xFF5F6977),
    accent: Color(0xFFE04E28),
    accentSoft: Color(0xFFFBE8E1),
    // Dark foreground is 4.84:1 on the light-theme ember; white is only 3.97:1.
    onAccent: Color(0xFF0B0F14),
    success: Color(0xFF087A55),
    warning: Color(0xFF8A4B00),
    info: Color(0xFF00677A),
    destructive: Color(0xFFB42318),
  );

  static KnowMePalette of(BuildContext context) {
    final background = shad.Theme.of(context).colorScheme.background;
    return background.computeLuminance() < 0.5 ? dark : light;
  }
}

abstract final class KnowMeTheme {
  static shad.ThemeData get dark => _build(KnowMePalette.dark, true);
  static shad.ThemeData get light => _build(KnowMePalette.light, false);

  static shad.ThemeData _build(KnowMePalette palette, bool dark) {
    final base = dark
        ? shad.LegacyColorSchemes.darkZinc()
        : shad.LegacyColorSchemes.lightZinc();
    final scheme = base.copyWith(
      background: () => palette.canvas,
      foreground: () => palette.text,
      card: () => palette.raised,
      cardForeground: () => palette.text,
      primary: () => palette.accent,
      primaryForeground: () => palette.onAccent,
      secondary: () => palette.chrome,
      secondaryForeground: () => palette.text,
      muted: () => palette.muted,
      mutedForeground: () => palette.subtleText,
      accent: () => palette.hover,
      accentForeground: () => palette.text,
      destructive: () => palette.destructive,
      destructiveForeground: () =>
          dark ? palette.canvas : const Color(0xFFFFFFFF),
      border: () => const Color(0x00000000),
      input: () => palette.surface,
      ring: () => palette.accent,
    );
    return shad.ThemeData(colorScheme: scheme, radius: 0.75);
  }
}
