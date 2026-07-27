// TJ-ARCH-MOB-001 compliant — service layer; the UI calls this, never the
// plugins directly.
//
// Saving and sharing media out of a chat bubble.
//
// `dart:io` is imported directly here, unlike in gen_ui_widgets: this file
// lives in the mobile app, which targets iOS and Android only. The shared
// widget package is also consumed by web builds, which is why IT needs a
// conditional import and this does not.
import 'dart:io';

import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';

/// What happened when the user asked to save or share a file.
///
/// A sealed result rather than a thrown exception: "the user dismissed the
/// share sheet" is a normal outcome, not an error, and the two need to read
/// differently in the UI.
sealed class MediaActionResult {
  const MediaActionResult();
}

/// The file was written, and [message] names where it went.
final class MediaActionSaved extends MediaActionResult {
  const MediaActionSaved(this.message);
  final String message;
}

/// The share sheet opened and the user picked something, or dismissed it.
final class MediaActionShared extends MediaActionResult {
  const MediaActionShared({required this.dismissed});
  final bool dismissed;
}

/// The action could not be completed. [message] is user-facing.
final class MediaActionFailed extends MediaActionResult {
  const MediaActionFailed(this.message);
  final String message;
}

/// Save and share for attachments and model-produced media.
///
/// Every method takes the DURABLE path from app-owned storage. A picker path
/// would be wrong here for the same reason it is wrong in a transcript: it can
/// be purged out from under the app.
class MediaActions {
  const MediaActions();

  /// Whether this platform can open a native share sheet for FILES.
  ///
  /// `share_plus` 13.3.0 supports Android, iOS, macOS, Windows and web, but has
  /// NO Linux file support — it falls back to `mailto`, which cannot carry an
  /// attachment. The UI hides the control there rather than offering one that
  /// silently does nothing.
  static bool get canShareFiles => !Platform.isLinux;

  /// Copy a stored file somewhere the user can find it outside the app.
  ///
  /// On Android that is the shared Downloads directory. On iOS there is no
  /// user-visible filesystem, so the platform answer IS the share sheet — the
  /// UI offers Share instead of Save there.
  Future<MediaActionResult> save({
    required String path,
    required String fileName,
  }) async {
    final source = File(path);
    if (!await source.exists()) {
      // The row outlived its bytes: app data cleared, or a failed sweep.
      return const MediaActionFailed('That file is no longer available.');
    }
    try {
      final directory = await _saveDirectory();
      if (directory == null) {
        return const MediaActionFailed(
          'This device has no downloads folder to save into.',
        );
      }
      final destination = await _uniqueDestination(directory, fileName);
      await source.copy(destination.path);
      return MediaActionSaved('Saved to ${destination.path}');
    } on FileSystemException catch (error) {
      // Surfaced, not swallowed: a user who taps Save and sees nothing has no
      // way to tell whether it worked.
      return MediaActionFailed('Could not save the file: ${error.message}');
    }
  }

  /// Hand a stored file to the platform share sheet.
  Future<MediaActionResult> share({
    required String path,
    required String fileName,
    required String mime,
  }) async {
    if (!canShareFiles) {
      return const MediaActionFailed(
        'Sharing files is not supported on this platform.',
      );
    }
    if (!await File(path).exists()) {
      return const MediaActionFailed('That file is no longer available.');
    }
    try {
      final result = await SharePlus.instance.share(
        // ShareParams + SharePlus.instance is the 13.x API; Share.shareXFiles
        // is deprecated.
        ShareParams(
          files: [XFile(path, mimeType: mime, name: fileName)],
          fileNameOverrides: [fileName],
        ),
      );
      return MediaActionShared(
        dismissed: result.status == ShareResultStatus.dismissed,
      );
    } on Object catch (error) {
      return MediaActionFailed('Could not share the file: $error');
    }
  }

  /// Where a saved copy should land.
  ///
  /// `getDownloadsDirectory` returns null on Android, so that platform falls
  /// back to the conventional public Downloads path. On iOS the app documents
  /// directory is the only writable place, and it is surfaced through the Files
  /// app — the closest thing to a download the platform has.
  Future<Directory?> _saveDirectory() async {
    if (Platform.isAndroid) {
      final publicDownloads = Directory('/storage/emulated/0/Download');
      if (await publicDownloads.exists()) return publicDownloads;
      return getExternalStorageDirectory();
    }
    if (Platform.isIOS) return getApplicationDocumentsDirectory();
    return await getDownloadsDirectory() ?? getApplicationDocumentsDirectory();
  }

  /// A path that does not overwrite an existing file.
  ///
  /// Saving the same image twice should produce "photo.png" and "photo (1).png",
  /// not silently replace the first — the user asked to keep it.
  Future<File> _uniqueDestination(Directory directory, String fileName) async {
    final dotIndex = fileName.lastIndexOf('.');
    final stem = dotIndex > 0 ? fileName.substring(0, dotIndex) : fileName;
    final extension = dotIndex > 0 ? fileName.substring(dotIndex) : '';

    var candidate = File('${directory.path}/$fileName');
    var counter = 1;
    while (await candidate.exists()) {
      candidate = File('${directory.path}/$stem ($counter)$extension');
      counter++;
    }
    return candidate;
  }
}
