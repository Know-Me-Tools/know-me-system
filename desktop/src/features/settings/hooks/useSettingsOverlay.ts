// TJ-ARCH-MOB-001 compliant — UI consumes a hook, never the store directly.
import { useSettingsOverlayStore } from '../stores/settingsOverlayStore'

export function useSettingsOverlay() {
  return useSettingsOverlayStore()
}
