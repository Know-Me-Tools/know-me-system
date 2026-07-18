// TJ-ARCH-MOB-001 compliant
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:knowme_poc/features/memory/domain/entities/memory_query.dart';
import 'package:knowme_poc/features/memory/presentation/providers/memory_notifier.dart';

void main() {
  test('MemoryNotifier folds ranked hits into MemoryResult', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    expect(container.read(memoryProvider).hits, isEmpty);

    // Rust-shaped hits at the FFI boundary — the only fake. Nothing else mocked.
    const hits = [
      MemoryHit(id: 'entity:1', text: 'Alpha', kind: 'note', score: 0.91),
      MemoryHit(id: 'entity:2', text: 'Beta', kind: 'note', score: 0.42),
    ];
    container.read(memoryProvider.notifier).applyHits('alpha', hits);

    final MemoryResult result = container.read(memoryProvider);
    expect(result.query, 'alpha');
    expect(result.hits, hasLength(2));
    expect(result.hits.first.text, 'Alpha');
    expect(result.hits.first.score, greaterThan(result.hits.last.score));
  });
}
