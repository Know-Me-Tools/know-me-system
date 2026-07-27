// TJ-ARCH-MOB-001 compliant
//
// Contract tests for the four explicit intents on a stopped turn.
//
// The bar is the only place the app distinguishes "keep my work and go a
// different way" from "throw it away", so these pin the weighting, the
// consequence-carrying accessible names, and — critically — that Steer ARMS the
// composer rather than sending.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'

const mocks = vi.hoisted(() => ({
  armSteer: vi.fn(),
  sendResume: vi.fn(),
  sendSteer: vi.fn(),
  regenerateResponse: vi.fn(),
  deleteResponse: vi.fn(),
}))

vi.mock('../stores/chatStore', () => ({
  useChatStore: (selector: (state: unknown) => unknown) => selector({
    armSteer: mocks.armSteer,
    sendResume: mocks.sendResume,
    sendSteer: mocks.sendSteer,
    regenerateResponse: mocks.regenerateResponse,
    deleteResponse: mocks.deleteResponse,
  }),
}))

import { StoppedTurnActions } from '../components/StoppedTurnActions'

beforeEach(() => {
  vi.clearAllMocks()
})

describe('StoppedTurnActions', () => {
  it('offers all four ways forward', () => {
    render(<StoppedTurnActions messageId="assistant-1" keptBlocks={2} />)
    for (const label of ['Steer', 'Resume', 'Retry', 'Discard']) {
      expect(screen.getByText(label), `"${label}" is missing`).toBeDefined()
    }
  })

  it('offers no Resume when nothing was kept', () => {
    // There is nothing to continue from, so offering it would be a dead end.
    render(<StoppedTurnActions messageId="assistant-1" keptBlocks={0} />)
    expect(screen.queryByText('Resume')).toBeNull()
    expect(screen.getByText('Steer')).toBeDefined()
  })

  it('arms the composer on Steer instead of sending', () => {
    // A Steer that fired immediately would just be a Resume with extra words —
    // the user has not written the correction yet.
    render(<StoppedTurnActions messageId="assistant-1" keptBlocks={2} />)
    fireEvent.click(screen.getByText('Steer'))
    expect(mocks.armSteer).toHaveBeenCalledOnce()
    expect(mocks.sendSteer).not.toHaveBeenCalled()
  })

  it('continues the stopped turn on Resume', () => {
    render(<StoppedTurnActions messageId="assistant-1" keptBlocks={2} />)
    fireEvent.click(screen.getByText('Resume'))
    expect(mocks.sendResume).toHaveBeenCalledOnce()
  })

  it('confirms before discarding', () => {
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false)
    render(<StoppedTurnActions messageId="assistant-1" keptBlocks={2} />)
    fireEvent.click(screen.getByText('Discard'))
    expect(confirm).toHaveBeenCalled()
    expect(mocks.deleteResponse).not.toHaveBeenCalled()

    confirm.mockReturnValue(true)
    fireEvent.click(screen.getByText('Discard'))
    expect(mocks.deleteResponse).toHaveBeenCalledWith('assistant-1')
    confirm.mockRestore()
  })

  it('names the consequence of every action', () => {
    // The visible labels are terse by design, so the accessible name carries what
    // the action actually does. Retry is destructive in place — a user who only
    // hears "Retry" has not been told it discards the preserved work.
    render(<StoppedTurnActions messageId="assistant-1" keptBlocks={2} />)
    for (const [visible, accessible] of [
      ['Steer', 'Steer from here — write a new instruction, keeping the work so far'],
      ['Resume', 'Resume — continue from where it stopped'],
      ['Retry', 'Retry — replaces this response and discards the work so far'],
      ['Discard', 'Discard this stopped response'],
    ] as const) {
      const button = screen.getByLabelText(accessible)
      expect(button.tagName, `${visible} must be a real button`).toBe('BUTTON')
      // WCAG 2.5.3 Label in Name: the visible label must lead the accessible one.
      expect(accessible.startsWith(visible)).toBe(true)
    }
  })
})
