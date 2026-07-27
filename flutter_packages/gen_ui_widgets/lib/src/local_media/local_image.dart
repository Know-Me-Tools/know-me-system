// TJ-ARCH-MOB-001 compliant
//
// Rendering a file path is platform-specific: `dart:io` does not exist on the
// web, and importing it unconditionally breaks the web build at COMPILE time —
// not at runtime, where a guard could catch it.
//
// So the choice is made by the compiler through a conditional import. Callers
// use `localImage(...)` and never mention `dart:io`.
import 'package:flutter/widgets.dart';

import 'local_image_stub.dart'
    if (dart.library.io) 'local_image_io.dart'
    as impl;

/// An image widget for a file on local disk, or null where that is impossible.
///
/// Returns null on the web: a filesystem path has no meaning in a browser tab,
/// and the caller renders its normal "unavailable" state rather than an
/// element that can never load.
Widget? localImage(
  String path, {
  required String semanticLabel,
  required Widget Function() onError,
}) => impl.localImage(path, semanticLabel: semanticLabel, onError: onError);

/// Whether this platform can render media from a filesystem path at all.
bool get supportsLocalMedia => impl.supportsLocalMedia;
