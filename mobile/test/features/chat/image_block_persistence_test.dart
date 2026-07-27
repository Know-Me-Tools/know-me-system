// TJ-ARCH-MOB-001 compliant
//
// Image blocks must survive persist -> hydrate with their durable path intact.
//
// CAUGHT ON DEVICE, not by any earlier test. The persistence encoder
// destructured only (url, dataBase64, mime), silently dropping path/alt/
// width/height. An attached PNG therefore rendered correctly in-session and
// came back as "Image unavailable" after a restart: the bytes were on disk,
// but the block that pointed at them had been stripped on save.
//
// The whole point of durable storage is that a transcript still renders after
// the OS clears its cache. A block that loses its path on save defeats that
// completely, and no unit test covered the round trip.
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

/// The encoder under test, mirroring PemChatRepository._encodeBlock's image arm.
/// Kept here rather than reaching into the private method so the test states
/// the contract: these seven fields are what a persisted image block carries.
Map<String, dynamic> encodeImage(ImageBlock block) => {
  'type': 'image',
  'url': block.url,
  'dataBase64': block.dataBase64,
  'mime': block.mime,
  'path': block.path,
  'alt': block.alt,
  'width': block.width,
  'height': block.height,
};

void main() {
  test('an attached image survives persist -> hydrate with its path', () {
    const original = ImageBlock(
      null,
      null,
      'image/png',
      path:
          '/data/user/0/tools.knowme.knowme/app_flutter/attachments/c1/abc.png',
      alt: 'knowme-image-certification.png',
      width: 7,
      height: 3,
    );

    // Round-trip through JSON exactly as the entity store does.
    final revived = ContentBlock.fromJson(
      jsonDecode(jsonEncode(encodeImage(original))) as Map<String, dynamic>,
    );

    expect(revived, isA<ImageBlock>());
    final image = revived as ImageBlock;

    // The path is the carrier. Losing it is what produced "Image unavailable"
    // on a device after a restart.
    expect(
      image.path,
      original.path,
      reason: 'a rehydrated block with no path has no way to find its bytes',
    );
    // The file name is the accessible name; without it a screen reader
    // announces the generic "Generated image" fallback, which is what the
    // device dump actually showed.
    expect(image.alt, 'knowme-image-certification.png');
    expect(image.mime, 'image/png');
    // Dimensions keep the layout from shifting when the image decodes.
    expect(image.width, 7);
    expect(image.height, 3);
  });

  test('a remote image still round-trips without a path', () {
    // Model-produced media carries a url and never touches local storage;
    // the added fields must stay optional rather than becoming required.
    const original = ImageBlock(
      'https://example.test/a.png',
      null,
      'image/png',
    );

    final revived = ContentBlock.fromJson(
      jsonDecode(jsonEncode(encodeImage(original))) as Map<String, dynamic>,
    ) as ImageBlock;

    expect(revived.url, 'https://example.test/a.png');
    expect(revived.path, isNull);
    expect(revived.alt, isNull);
  });

  test('a legacy block persisted before path existed still decodes', () {
    // Conversations saved before this change carry only the original three
    // fields. They must not throw — they simply have no local carrier.
    final legacy = ContentBlock.fromJson({
      'type': 'image',
      'url': null,
      'dataBase64': 'AAAA',
      'mime': 'image/webp',
    }) as ImageBlock;

    expect(legacy.dataBase64, 'AAAA');
    expect(legacy.path, isNull);
    expect(legacy.width, isNull);
  });
}
