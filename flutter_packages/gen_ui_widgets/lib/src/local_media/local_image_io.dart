// TJ-ARCH-MOB-001 compliant
//
// Native implementation (iOS, Android, macOS, Windows, Linux). Only ever
// reached through the conditional import in local_image.dart, so `dart:io` is
// never pulled into a web build.
import 'dart:io' show File;

import 'package:flutter/widgets.dart';

Widget? localImage(
  String path, {
  required String semanticLabel,
  required Widget Function() onError,
}) {
  if (path.isEmpty) return null;
  return Image.file(
    File(path),
    fit: BoxFit.contain,
    semanticLabel: semanticLabel,
    // A stored file can still go missing — the user may have cleared app data,
    // or the row may outlive its bytes. Show the caller's error state rather
    // than letting the exception escape into the widget tree.
    errorBuilder: (_, __, ___) => onError(),
  );
}

bool get supportsLocalMedia => true;
