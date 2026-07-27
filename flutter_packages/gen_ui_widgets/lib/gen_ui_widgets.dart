// TJ-ARCH-MOB-001 compliant
/// gen_ui_widgets — ContentBlock widget set for the TJ-ARCH-MOB-001 architecture.
library;

export 'src/content_block.dart';
export 'src/content_block_view.dart';
export 'src/a2ui_surface_view.dart';
// Platform-split local media loader. Exported so app surfaces (the composer
// chip, for one) can render a stored file without importing `dart:io` — which
// would break the web build at compile time.
export 'src/local_media/local_image.dart';
