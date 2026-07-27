// TJ-ARCH-MOB-001 compliant
//
// The status line on a stopped turn: it is the reassurance that the work
// survived, and without it the four actions beneath are buttons with no stated
// consequence.
import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ComponentProps } from 'react'

import { RunCancelled } from '../components/RichDataRenderers'

// The renderer only reads `data`; the rest of MessagePartState is irrelevant
// here, so the props are cast rather than fully constructed.
type RunCancelledProps = ComponentProps<typeof RunCancelled>
const render_ = (data: { phase: string; message: string; keptBlocks?: number }) =>
  render(<RunCancelled {...({ type: 'data', name: 'runCancelled', data } as unknown as RunCancelledProps)} />)

describe('RunCancelled', () => {
  it('announces itself to screen readers', () => {
    // role="status" alone left the cancel notice unannounced, unlike its
    // RunLifecycle sibling. A user who cannot see the transcript otherwise gets
    // no signal that their run stopped.
    render_({ phase: 'generation', message: 'Cancelled by user', keptBlocks: 2 })
    const status = screen.getByRole('status')
    expect(status.getAttribute('aria-live')).toBe('polite')
  })

  it('says how much work was kept', () => {
    render_({ phase: 'generation', message: 'Cancelled by user', keptBlocks: 3 })
    expect(screen.getByText(/kept 3 blocks, still in context/)).toBeDefined()
  })

  it('does not pluralise a single block', () => {
    render_({ phase: 'generation', message: 'Cancelled by user', keptBlocks: 1 })
    expect(screen.getByText(/kept 1 block, still in context/)).toBeDefined()
  })

  it('claims no kept work when there is none', () => {
    // "kept 0 blocks" would be a lie the user can see through.
    render_({ phase: 'generation', message: 'Cancelled by user', keptBlocks: 0 })
    expect(screen.queryByText(/kept/)).toBeNull()
    expect(screen.getByText('Cancelled by user')).toBeDefined()
  })
})
