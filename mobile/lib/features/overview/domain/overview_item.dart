// TJ-ARCH-MOB-001 compliant
import 'package:flutter/widgets.dart';

/// Immutable content contract for a KnowMe overview destination.
class OverviewItem {
  const OverviewItem({
    required this.title,
    required this.eyebrow,
    required this.heading,
    required this.body,
    required this.icon,
  });

  final String title;
  final String eyebrow;
  final String heading;
  final String body;
  final IconData icon;
}
