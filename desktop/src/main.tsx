// TJ-ARCH-MOB-001 compliant
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { AppProviders } from './app/providers'
import { StartupGate } from './features/startup/components/StartupGate'
import { useChatStore } from './features/chat/stores/chatStore'
import { EntityRuntimeBoundary } from './features/entities/components/EntityRuntimeBoundary'
import './index.css'

// Apply the stored/system preference before React paints so the shell never
// flashes the wrong KnowMe theme. The Settings hook owns later changes.
const storedTheme = window.localStorage.getItem('knowme-theme')
const darkTheme = storedTheme
  ? storedTheme === 'dark'
  : window.matchMedia('(prefers-color-scheme: dark)').matches
document.body.classList.toggle('light', !darkTheme)
document.documentElement.classList.toggle('dark', darkTheme)

// Initialize Rust event listeners (store-level, not component-level)
const cleanup = useChatStore.getState().initListeners()
window.addEventListener('beforeunload', cleanup)

// StartupGate blocks the app until the first-run boot sequence (migrations →
// seeds → shapes) reaches ready — the boot-order invariant made visible.
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <StartupGate>
      <EntityRuntimeBoundary>
        <AppProviders />
      </EntityRuntimeBoundary>
    </StartupGate>
  </StrictMode>
)
