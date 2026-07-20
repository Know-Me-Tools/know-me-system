// TJ-ARCH-MOB-001 compliant — transient settings-overlay interaction state.
import { create } from 'zustand'

interface SettingsOverlayState {
  open: boolean
  openSettings: () => void
  closeSettings: () => void
}

export const useSettingsOverlayStore = create<SettingsOverlayState>((set) => ({
  open: false,
  openSettings: () => set({ open: true }),
  closeSettings: () => set({ open: false }),
}))
