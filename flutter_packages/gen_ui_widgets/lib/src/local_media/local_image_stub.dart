// TJ-ARCH-MOB-001 compliant
//
// Web implementation. There is no filesystem to read, so a stored path cannot
// be rendered at all — the caller falls back to its unavailable state.
import 'package:flutter/widgets.dart';

Widget? localImage(
  String path, {
  required String semanticLabel,
  required Widget Function() onError,
}) => null;

bool get supportsLocalMedia => false;
