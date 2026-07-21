// TJ-ARCH-MOB-001 compliant — hook composes the lane store; no IPC here.
import { isTauri } from '@tauri-apps/api/core'
import { useEffect } from 'react'
import { useLaneStore } from '@/features/chat/stores/laneStore'

export function useModels() {
  const lane = useLaneStore((state) => state.lane)
  const uarSupported = useLaneStore((state) => state.uarSupported)
  const uarConfigured = useLaneStore((state) => state.uarConfigured)
  const switchLane = useLaneStore((state) => state.switchLane)
  const initLane = useLaneStore((state) => state.init)
  useEffect(() => { void initLane() }, [initLane])
  const desktop = isTauri()
  return { models: [
    ...(desktop && uarSupported ? [{
      id: 'uar' as const,
      name: 'Universal Agent Runtime',
      tags: [
        'Agent runtime',
        'Orchestrator LLM access',
        'Skills + tools',
        uarConfigured ? 'Configured' : 'Configure in Settings',
      ],
      action: lane === 'uar' ? 'Active' : 'Use UAR',
    }] : []),
    {
      id: 'local' as const,
      name: desktop ? 'Qwen 2.5 0.5B Instruct Q4' : 'Qwen 2.5 0.5B WebLLM',
      tags: ['On-device', desktop ? 'Rust · Metal' : 'WebGPU', 'Zero configuration'],
      action: lane === 'local' ? 'Active' : 'Use on-device',
    },
    ...(!desktop ? [{
      id: 'cloud' as const,
      name: 'Cloud providers',
      tags: ['liter-llm', 'Bring your own key', 'Explicit network use'],
      action: lane === 'cloud' ? 'Active' : 'Use cloud',
    }] : []),
  ], switchLane }
}
