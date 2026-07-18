// TJ-ARCH-MOB-001 compliant
// Branded custom titlebar. tauri.conf.json sets decorations:false, so the OS
// draws no native chrome — this component owns drag, minimize/maximize/close,
// and the KnowMe lockup. Window control placement follows platform convention:
// left (traffic-light order) on macOS, right on Windows/Linux. Every Tauri API
// call is gated behind isTauri() — this same bundle also runs as a plain web
// page (no __TAURI_INTERNALS__ bridge), where getCurrentWindow()/platform()
// throw synchronously if called unconditionally at module scope.
import { useState } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow, type Window as TauriWindow } from '@tauri-apps/api/window'
import { platform } from '@tauri-apps/plugin-os'
import { KnowMeLogo, KnowMeWordmark } from './KnowMeLogo'

const appWindow: TauriWindow | null = isTauri() ? getCurrentWindow() : null

function WindowControlsMac() {
  return (
    <div className="flex items-center gap-2 pl-2">
      <button
        aria-label="Close"
        onClick={() => appWindow?.close()}
        className="h-3 w-3 rounded-full bg-[#FF5F57] hover:brightness-90 active:brightness-75"
      />
      <button
        aria-label="Minimize"
        onClick={() => appWindow?.minimize()}
        className="h-3 w-3 rounded-full bg-[#FEBC2E] hover:brightness-90 active:brightness-75"
      />
      <button
        aria-label="Maximize"
        onClick={() => appWindow?.toggleMaximize()}
        className="h-3 w-3 rounded-full bg-[#28C840] hover:brightness-90 active:brightness-75"
      />
    </div>
  )
}

function WindowControlsWindows() {
  return (
    <div className="flex items-stretch">
      <button
        aria-label="Minimize"
        onClick={() => appWindow?.minimize()}
        className="flex h-8 w-11 items-center justify-center text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)]"
      >
        &#xE921;
      </button>
      <button
        aria-label="Maximize"
        onClick={() => appWindow?.toggleMaximize()}
        className="flex h-8 w-11 items-center justify-center text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)]"
      >
        &#xE922;
      </button>
      <button
        aria-label="Close"
        onClick={() => appWindow?.close()}
        className="flex h-8 w-11 items-center justify-center text-[color:var(--color-fg-sub)] hover:bg-[#E81123] hover:text-white"
      >
        &#xE8BB;
      </button>
    </div>
  )
}

export function Titlebar() {
  const [os] = useState(() => {
    if (!isTauri()) return null
    try {
      return platform()
    } catch {
      return null
    }
  })

  const isMac = os === 'macos'

  // data-tauri-drag-region marks the drag surface, but Tauri's own webview-side
  // listener only fires reliably on a direct mousedown against that element —
  // explicitly calling startDragging() here makes the whole bar (including the
  // flex spacers around the lockup) draggable rather than only whichever DOM
  // node the attribute landed on.
  const startDrag = (e: React.MouseEvent) => {
    if (e.button !== 0) return
    void appWindow?.startDragging()
  }

  return (
    <header
      data-tauri-drag-region
      onMouseDown={startDrag}
      className="flex h-9 shrink-0 items-center bg-[color:var(--color-bg-2)] select-none"
    >
      {isMac && <WindowControlsMac />}
      <div className="flex flex-1 items-center justify-center gap-2 text-[color:var(--color-fg)]">
        <KnowMeLogo size={16} className="text-[color:var(--color-fg)]" />
        <KnowMeWordmark className="text-[13px]" />
      </div>
      {isTauri() && os !== null && !isMac && <WindowControlsWindows />}
    </header>
  )
}
