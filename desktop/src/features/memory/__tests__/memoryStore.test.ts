// TJ-ARCH-MOB-001 compliant
import { beforeEach, describe, expect, it, vi } from 'vitest'

// Fake ONLY the Tauri IPC edge — a real IO boundary. The store logic is real.
//
// Both modules must be faked. The store guards on `isTauri()` from
// @tauri-apps/api/core, but it reaches Rust through the PLUGIN's typed wrappers,
// and the plugin imports its own `invoke` — mocking core alone leaves the real
// wrapper in place and no call ever lands (0 invocations, not a wrong one).
const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
  isTauri: () => true,
}))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({
  memorySearch: (query: string, k: number, mode: string) =>
    invoke('plugin:gen-ui|memory_search', { query, k, mode }),
  memoryIngest: (text: string) => invoke('plugin:gen-ui|memory_ingest', { text }),
}))

import { useMemoryStore, type MemoryHit } from '../stores/memoryStore'

const initial = useMemoryStore.getState()
beforeEach(() => {
  useMemoryStore.setState(initial, true)
  invoke.mockReset()
})

describe('memoryStore', () => {
  it('folds ranked hits from memory_search into state', async () => {
    // The real MemoryHit shape (id/text/kind/score) — mirrors
    // gen_ui_db_graph::MemoryHit. This used to assert name/snippet, which the
    // Rust type dropped in C-104; an `as unknown as` cast in the store hid the
    // drift from tsc until the binding became properly typed.
    const hits: MemoryHit[] = [
      { id: 'entity:1', text: 'Alpha', kind: 'note', score: 0.91 },
      { id: 'entity:2', text: 'Beta', kind: 'note', score: 0.42 },
    ]
    invoke.mockResolvedValueOnce(hits)

    await useMemoryStore.getState().search('alpha')

    const s = useMemoryStore.getState()
    expect(invoke).toHaveBeenCalledWith('plugin:gen-ui|memory_search', {
      query: 'alpha',
      k: 8,
      mode: 'hybrid',
    })
    expect(s.query).toBe('alpha')
    expect(s.hits).toHaveLength(2)
    expect(s.hits[0].score).toBeGreaterThan(s.hits[1].score)
    expect(s.isSearching).toBe(false)
  })

  it('surfaces a Rust domain error instead of throwing or retrying', async () => {
    invoke.mockRejectedValueOnce('surreal: index not ready')

    await useMemoryStore.getState().search('alpha')

    const s = useMemoryStore.getState()
    expect(s.error).toContain('index not ready')
    expect(s.hits).toEqual([])
    expect(s.isSearching).toBe(false)
    expect(invoke).toHaveBeenCalledTimes(1) // terminal — no silent retry
  })
})
