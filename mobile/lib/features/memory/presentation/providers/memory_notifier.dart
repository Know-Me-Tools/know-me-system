// TJ-ARCH-MOB-001 compliant
// MemoryNotifier owns the memory-panel state. It calls the FFI memory intents
// (ingest / hybrid search) via the bridge facade — never SurrealQL, never a
// second data path. build() is synchronous and never throws, so provider-level
// retry does not apply; the async FFI work runs inside Mutations in the screen,
// whose own error handling is terminal (Rust domain errors don't auto-retry).
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../../../bridge/rust_bridge_provider.dart' as bridge;
import '../../domain/entities/memory_query.dart';

part 'memory_notifier.g.dart';

/// Default recall breadth for the hybrid search (k in memory_search).
const int kMemoryRecallK = 8;

@riverpod
class MemoryNotifier extends _$MemoryNotifier {
  @override
  MemoryResult build() => const MemoryResult.empty();

  /// Ingest a note into the graph store (embed on-device + upsert, in Rust).
  /// Returns the new record id. Terminal on Rust domain error.
  Future<String> ingest(String text) => bridge.memoryIngest(text);

  /// Run a hybrid search and fold the ranked hits into state. Terminal on error.
  Future<void> search(String query) async {
    final hits = await bridge.memorySearch(query, kMemoryRecallK);
    if (!ref.mounted) return; // provider paused/disposed while awaiting
    applyHits(query, hits);
  }

  /// Fold an already-fetched hit list into state (sync — no await, no FFI).
  /// The boundary seam: search() awaits the FFI, then delegates here. The test
  /// drives real MemoryHits straight through this fold path — nothing internal
  /// is mocked; the only fake is the Rust-shaped hit list.
  void applyHits(String query, List<MemoryHit> hits) {
    state = MemoryResult(query: query, hits: hits);
  }
}
