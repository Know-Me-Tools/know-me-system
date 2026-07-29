import 'package:flutter_test/flutter_test.dart';
import 'package:knowme_poc/features/chat/presentation/widgets/chat_policy_toolbar.dart';

void main() {
  const embedded = [
    {
      'resource': 'skills',
      'reason': 'not available on the embedded runtime yet',
      'platformLimitation': true,
    },
    {
      'resource': 'knowledgeBases',
      'reason': 'not available on the embedded runtime yet',
      'platformLimitation': true,
    },
  ];

  test('an empty registry is distinguishable from an unavailable one', () {
    // The defect this closes: with only an empty list, a user cannot tell
    // "nothing configured" from "this device cannot provide it".
    expect(unavailableEntryFor(embedded, 'skills'), isNotNull);
    expect(unavailableEntryFor(embedded, 'skills')!['reason'], isNotEmpty);
    expect(unavailableEntryFor(embedded, 'agents'), isNull);
  });

  test('matches the sheet snake_case key against the camelCase wire name', () {
    // The sheet's policy key is knowledge_bases; Rust reports knowledgeBases.
    // A mismatch here would silently show no reason at all.
    expect(unavailableEntryFor(embedded, 'knowledge_bases'), isNotNull);
    expect(unavailableEntryFor(embedded, 'knowledgeBases'), isNotNull);
  });

  test('a service deployment omits the field entirely', () {
    expect(unavailableEntryFor(null, 'skills'), isNull);
    expect(unavailableEntryFor(const [], 'skills'), isNull);
  });
}
