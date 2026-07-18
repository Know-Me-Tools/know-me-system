// TJ-ARCH-MOB-001 compliant
// Design tokens — travisjames.ai brand system.
import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

abstract final class T {
  // Backgrounds
  static const bgPrimary = Color(0xFF0D0D18);
  static const bgSurface = Color(0xFF121220);
  static const bgElevated = Color(0xFF181828);
  static const bgOverlay = Color(0xFF1E1E35);

  // Accents
  static const ember = Color(0xFFFF6A3D);
  static const violet = Color(0xFF8B78FF);
  static const cyan = Color(0xFF22D3EE);
  static const amber = Color(0xFFF5A623);
  static const green = Color(0xFF34D399);
  static const red = Color(0xFFF87171);

  // Text
  static const textPrimary = Color(0xFFF2F2FF);
  static const textSecondary = Color(0xFF9898C0);
  static const textTertiary = Color(0xFF5E5E88);
  static const textDisabled = Color(0xFF3A3A60);

  // Typography
  static TextStyle get displayLg => GoogleFonts.spaceGrotesk(
        fontSize: 32,
        fontWeight: FontWeight.w700,
        letterSpacing: -0.03,
        color: textPrimary,
      );
  static TextStyle get uiMd => GoogleFonts.inter(
        fontSize: 13,
        fontWeight: FontWeight.w500,
        color: textSecondary,
      );
  static TextStyle get prose => GoogleFonts.roboto(
        fontSize: 15,
        fontWeight: FontWeight.w400,
        color: textPrimary,
        height: 1.75,
      );
  static TextStyle get mono => GoogleFonts.jetBrainsMono(
        fontSize: 12.5,
        fontWeight: FontWeight.w400,
        color: textPrimary,
        height: 1.55,
      );
}
