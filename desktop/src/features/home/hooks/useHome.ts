// TJ-ARCH-MOB-001 compliant — hook composes runtime stores; no IPC here.
import { isTauri } from '@tauri-apps/api/core'
import { useEffect } from 'react'
import { useLaneStore } from '@/features/chat/stores/laneStore'

export function useHome() {
  const lane = useLaneStore((state) => state.lane)
  const localAvailable = useLaneStore((state) => state.localAvailable)
  const initLane = useLaneStore((state) => state.init)
  useEffect(() => { void initLane() }, [initLane])
  const desktop = isTauri()
  const model = lane === 'local' ? 'Qwen 2.5 0.5B Q4' : 'Cloud provider'
  const modelDetail = lane === 'local'
    ? `${desktop ? 'Native Rust' : 'WebLLM'} · ${localAvailable ? 'ready' : 'initializing'}`
    : 'Bring your own key · explicit network use'
  return {
    summary: [
      { label: 'Active model', value: model, detail: modelDetail, status: localAvailable ? 'ready' : 'loading' },
      { label: 'Memory', value: 'Local RAG', detail: 'Embedded graph and vectors', status: 'ready' },
      { label: 'Database', value: desktop ? 'pglite-oxide' : 'PGlite', detail: 'Conversation store ready', status: 'ready' },
    ],
    capabilities: [
      ['Chat', 'Ask anything, with citations and memory.', '/chat'],
      ['Hands', 'Reliable automations that run for you.', '/hands'],
      ['Memory', 'Inspect what KnowMe remembers.', '/memory'],
      ['Models', 'Your on-device and cloud model library.', '/models'],
      ['Settings', 'Control sync, privacy, and devices.', '/settings'],
    ] as const,
  }
}
