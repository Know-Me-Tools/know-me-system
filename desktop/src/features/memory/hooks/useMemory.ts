// TJ-ARCH-MOB-001 compliant — hooks compose stores; no invoke() here.
import { useMemoryStore } from '../stores/memoryStore'

export function useMemory() {
  return {
    query: useMemoryStore((s) => s.query),
    hits: useMemoryStore((s) => s.hits),
    isIngesting: useMemoryStore((s) => s.isIngesting),
    isSearching: useMemoryStore((s) => s.isSearching),
    error: useMemoryStore((s) => s.error),
    mode: useMemoryStore((s) => s.mode),
    ingest: useMemoryStore((s) => s.ingest),
    search: useMemoryStore((s) => s.search),
    setMode: useMemoryStore((s) => s.setMode),
  }
}
