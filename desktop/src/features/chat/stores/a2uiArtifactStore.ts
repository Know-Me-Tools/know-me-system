// TJ-ARCH-MOB-001 compliant — the store owns the Rust/HTTP transport boundary.
import { isTauri } from '@tauri-apps/api/core'
import { promoteA2uiComponent } from '@prometheus-ags/tauri-plugin-gen-ui'
import { create } from 'zustand'

const apiRoot = (import.meta.env.VITE_UAR_URL as string | undefined)?.replace(/\/$/, '') ?? ''

interface A2uiArtifactState {
  promoting: boolean
  error: string | null
  promote: (title: string, source: string) => Promise<boolean>
}

async function promoteHosted(title: string, source: string): Promise<void> {
  const response = await fetch(`${apiRoot}/api/uar/a2ui/components`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ title, source, description: 'Promoted from a KnowMe conversation' }),
  })
  if (!response.ok) {
    const body = await response.text()
    throw new Error(body || `UAR component library rejected this artifact (${response.status})`)
  }
}

export const useA2uiArtifactStore = create<A2uiArtifactState>((set) => ({
  promoting: false,
  error: null,
  promote: async (title, source) => {
    set({ promoting: true, error: null })
    try {
      if (isTauri()) await promoteA2uiComponent(title, source, 'Promoted from a KnowMe conversation')
      else await promoteHosted(title, source)
      return true
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to add component' })
      return false
    } finally {
      set({ promoting: false })
    }
  },
}))
