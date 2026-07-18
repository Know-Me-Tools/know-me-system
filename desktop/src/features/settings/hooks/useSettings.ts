// TJ-ARCH-MOB-001 compliant
import { useEffect, useState } from 'react'

const THEME_KEY = 'knowme-theme'

function initialDarkMode(): boolean {
  const saved = window.localStorage.getItem(THEME_KEY)
  if (saved) return saved === 'dark'
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

export function useSettings() {
  const [settings, setSettings] = useState({ dark: initialDarkMode(), sync: true, cloud: false, diagnostics: false })

  useEffect(() => {
    document.body.classList.toggle('light', !settings.dark)
    document.documentElement.classList.toggle('dark', settings.dark)
    window.localStorage.setItem(THEME_KEY, settings.dark ? 'dark' : 'light')
  }, [settings.dark])

  const toggle = (key: keyof typeof settings) => setSettings((current) => ({ ...current, [key]: !current[key] }))
  return { settings, toggle, devices: [
    ['This Mac', 'Owner', 'Synced'], ['iPhone 16 Pro', 'Member', 'Synced 6m ago'], ['Home server', 'Owner · sync anchor', 'Reachable'],
  ] as const }
}
