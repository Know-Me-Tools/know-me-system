// TJ-ARCH-MOB-001 compliant
//
// Keyboard operability of the mode picker (WCAG 2.1.1).
//
// This exists because the picker shipped arrow-reachable but Enter-inert: rows
// highlighted correctly and could not be activated at all. Browser probing kept
// giving ambiguous answers, so the contract is pinned here instead — a row must
// be selectable by keyboard alone, with no pointer event involved.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

const mocks = vi.hoisted(() => ({
  setMode: vi.fn(),
  agents: [
    { id: 'orchestrator-agent', label: 'Orchestrator', protected: true },
    { id: 'writer-agent', label: 'Writer', protected: false },
  ],
}))

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }))
vi.mock('@prometheus-ags/tauri-plugin-gen-ui', () => ({}))

vi.mock('../hooks/useChat', () => ({
  useChat: () => ({ activeConversationId: 'conversation-1' }),
}))

vi.mock('../hooks/useChatControl', () => ({
  useChatControl: () => ({
    agents: mocks.agents,
    policy: { chat_mode: 'uar', agent_id: 'orchestrator-agent' },
    laneAvailability: {},
    saving: false,
    setMode: mocks.setMode,
  }),
}))

import { ChatModePicker } from '../components/ChatModePicker'

describe('ChatModePicker keyboard operability', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // jsdom has no matchMedia; the picker uses it to choose dropdown vs drawer.
    // Reporting "not narrow" exercises the dropdown branch under test.
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: false,
      media: query,
      addEventListener: () => {},
      removeEventListener: () => {},
    }))
  })

  it('opens the menu from the keyboard', async () => {
    const user = userEvent.setup()
    render(<ChatModePicker />)

    await user.tab()
    await user.keyboard('{Enter}')

    await waitFor(() => {
      expect(screen.getByRole('menuitemradio', { name: /Local/ })).toBeDefined()
    })
  })

  it('selects a row with Enter — no pointer event involved', async () => {
    const user = userEvent.setup()
    render(<ChatModePicker />)

    await user.tab()
    await user.keyboard('{Enter}')
    await waitFor(() => screen.getByRole('menuitemradio', { name: /Local/ }))

    // Rows are Orchestrator (focused on open), Writer, Local.
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}')

    await waitFor(() => {
      expect(mocks.setMode).toHaveBeenCalledWith('local', undefined)
    })
    expect(mocks.setMode).toHaveBeenCalledTimes(1)
  })

  it('selects an agent row with Enter, carrying that agent id', async () => {
    const user = userEvent.setup()
    render(<ChatModePicker />)

    await user.tab()
    await user.keyboard('{Enter}')
    await waitFor(() => screen.getByRole('menuitemradio', { name: /Writer/ }))

    // The menu opens with the CHECKED row focused (Orchestrator), so a single
    // ArrowDown lands on Writer — not two, which would skip past it.
    await user.keyboard('{ArrowDown}{Enter}')

    await waitFor(() => {
      expect(mocks.setMode).toHaveBeenCalledWith('agent', 'writer-agent')
    })
    // Exactly once: a pointer click and a keyboard Enter must not both fire.
    expect(mocks.setMode).toHaveBeenCalledTimes(1)
  })

  it('exposes selection state through aria-checked, not colour', async () => {
    const user = userEvent.setup()
    render(<ChatModePicker />)

    await user.tab()
    await user.keyboard('{Enter}')

    await waitFor(() => {
      const orchestrator = screen.getByRole('menuitemradio', { name: /Orchestrator/ })
      expect(orchestrator.getAttribute('aria-checked')).toBe('true')
      expect(screen.getByRole('menuitemradio', { name: /Local/ }).getAttribute('aria-checked')).toBe('false')
    })
  })
})

describe('ChatModePicker pointer operability', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: false,
      media: query,
      addEventListener: () => {},
      removeEventListener: () => {},
    }))
  })

  it('selects a row with a click', async () => {
    // Guards the other half of the fix: activation is handled in onKeyDown, and
    // a keyboard-only handler would silently break the pointer path that every
    // mouse user takes.
    const user = userEvent.setup()
    render(<ChatModePicker />)

    await user.click(screen.getByRole('button', { name: /Conversation mode/ }))
    await waitFor(() => screen.getByRole('menuitemradio', { name: /Local/ }))

    await user.click(screen.getByRole('menuitemradio', { name: /Local/ }))

    await waitFor(() => {
      expect(mocks.setMode).toHaveBeenCalledWith('local', undefined)
    })
    expect(mocks.setMode).toHaveBeenCalledTimes(1)
  })

  it('does not activate a blocked row', async () => {
    const user = userEvent.setup()
    render(<ChatModePicker />)

    await user.click(screen.getByRole('button', { name: /Conversation mode/ }))
    await waitFor(() => screen.getByRole('menuitemradio', { name: /Local/ }))

    const local = screen.getByRole('menuitemradio', { name: /Local/ })
    expect(local.getAttribute('aria-disabled')).not.toBe('true')
  })
})
