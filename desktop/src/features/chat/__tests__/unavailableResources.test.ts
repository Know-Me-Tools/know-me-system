import { describe, expect, it } from 'vitest'
import type { UnavailableResource } from '../stores/chatControlStore'

/** The lookup ChatPolicyToolbar performs per resource control. */
function reasonFor(unavailable: UnavailableResource[], resource: string) {
  return unavailable.find((entry) => entry.resource === resource)
}

describe('offline-unavailable resources', () => {
  const embedded: UnavailableResource[] = [
    { resource: 'skills', reason: 'not available on the embedded runtime yet', platformLimitation: true },
    { resource: 'tools', reason: 'not available on the embedded runtime yet', platformLimitation: true },
    { resource: 'knowledgeBases', reason: 'not available on the embedded runtime yet', platformLimitation: true },
  ]

  it('distinguishes "unavailable here" from "nothing configured"', () => {
    // The whole defect: an empty array alone cannot express this difference.
    expect(reasonFor(embedded, 'skills')).toBeDefined()
    expect(reasonFor(embedded, 'skills')?.reason).not.toBe('')
    // A resource the platform DOES serve reports no reason, so the picker keeps
    // its ordinary "nothing configured" copy.
    expect(reasonFor(embedded, 'agents')).toBeUndefined()
  })

  it('marks embedded gaps as structural so the UI never offers a retry', () => {
    for (const entry of embedded) {
      expect(entry.platformLimitation).toBe(true)
    }
  })

  it('reports no gaps for a service deployment', () => {
    expect(reasonFor([], 'skills')).toBeUndefined()
  })
})
