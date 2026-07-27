// TJ-ARCH-MOB-001 compliant
//
// Saving media out of a chat bubble. The share sheet itself needs a platform
// host and is covered on device; what IS testable here is the part that can
// silently destroy a user's file.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:knowme_poc/features/chat/data/media_actions.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

/// Redirects every path_provider lookup into a tempdir, so the tests never
/// touch a real Downloads folder.
class _FakePathProvider extends PathProviderPlatform
    with MockPlatformInterfaceMixin {
  _FakePathProvider(this.root);
  final String root;

  @override
  Future<String?> getApplicationDocumentsPath() async => root;
  @override
  Future<String?> getDownloadsPath() async => root;
  @override
  Future<String?> getTemporaryPath() async => root;
  @override
  Future<String?> getApplicationSupportPath() async => root;
}

void main() {
  late Directory temp;

  setUp(() async {
    temp = await Directory.systemTemp.createTemp('media-actions-test');
    PathProviderPlatform.instance = _FakePathProvider(temp.path);
  });

  tearDown(() async {
    if (temp.existsSync()) await temp.delete(recursive: true);
  });

  test(
    'saving a missing file reports it rather than failing silently',
    () async {
      // The row can outlive its bytes — app data cleared, or a sweep that ran
      // too eagerly. A user who taps Save and sees nothing cannot tell whether
      // it worked.
      final result = await const MediaActions().save(
        path: '${temp.path}/never-existed.png',
        fileName: 'never-existed.png',
      );

      expect(result, isA<MediaActionFailed>());
      expect(
        (result as MediaActionFailed).message,
        contains('no longer available'),
      );
    },
  );

  test('saving twice keeps both copies instead of overwriting', () async {
    final source = File('${temp.path}/source.png')..writeAsBytesSync([1, 2, 3]);
    final destination = Directory('${temp.path}/out')..createSync();
    PathProviderPlatform.instance = _FakePathProvider(destination.path);

    final first = await const MediaActions().save(
      path: source.path,
      fileName: 'photo.png',
    );
    final second = await const MediaActions().save(
      path: source.path,
      fileName: 'photo.png',
    );

    expect(first, isA<MediaActionSaved>());
    expect(second, isA<MediaActionSaved>());
    // The second save must NOT replace the first: the user asked to keep it.
    expect(File('${destination.path}/photo.png').existsSync(), isTrue);
    expect(File('${destination.path}/photo (1).png').existsSync(), isTrue);
  });

  test(
    'sharing a missing file reports it rather than opening an empty sheet',
    () async {
      final result = await const MediaActions().share(
        path: '${temp.path}/gone.png',
        fileName: 'gone.png',
        mime: 'image/png',
      );

      expect(result, isA<MediaActionFailed>());
    },
  );

  test('file sharing is unavailable on Linux and available elsewhere', () {
    // share_plus 13.3.0 has NO Linux file support — it falls back to mailto,
    // which cannot carry an attachment. The UI hides the control there rather
    // than offering one that silently does nothing.
    expect(MediaActions.canShareFiles, !Platform.isLinux);
  });
}
