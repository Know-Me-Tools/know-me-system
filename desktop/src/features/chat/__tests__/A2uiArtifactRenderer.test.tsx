import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { A2uiArtifactRenderer } from '../components/A2uiArtifactRenderer'

const source = [
  '{"version":"v0.9","createSurface":{"surfaceId":"profile","catalogId":"https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json"}}',
  '{"version":"v0.9","updateComponents":{"surfaceId":"profile","components":[{"id":"root","component":"Column","children":["name","action"]},{"id":"name","component":"Text","text":{"path":"/person/name"}},{"id":"action","component":"Button","child":"label","action":{"event":{"name":"reviewProfile","context":{"id":"42"}}}},{"id":"label","component":"Text","text":"Review"}]}}',
  '{"version":"v0.9","updateDataModel":{"surfaceId":"profile","path":"/person","value":{"name":"Ada Lovelace"}}}',
].join('\n')

describe('A2uiArtifactRenderer', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('renders bound components and sends a standard A2UI action', async () => {
    const onAction = vi.fn().mockResolvedValue(undefined)
    render(<A2uiArtifactRenderer source={source} title="Profile" sourceRunId="uar-run-1" onAction={onAction} />)
    expect(screen.getByText('Ada Lovelace')).toBeTruthy()
    fireEvent.click(screen.getByRole('button', { name: 'Review' }))
    await waitFor(() => expect(onAction).toHaveBeenCalledOnce())
    expect(onAction.mock.calls[0][0]).toMatchObject({
      sourceRunId: 'uar-run-1',
      artifactId: 'profile',
      action: { surfaceId: 'profile', name: 'reviewProfile', sourceComponentId: 'action' },
    })
  })

  it('reveals source and promotes the component through UAR storage', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, status: 201 })
    vi.stubGlobal('fetch', fetchMock)
    render(<A2uiArtifactRenderer source={source} title="Profile" sourceRunId="uar-run-1" onAction={vi.fn()} />)
    fireEvent.click(screen.getByRole('button', { name: /^source$/i }))
    expect(document.querySelector('pre')?.textContent).toBe(source)
    fireEvent.click(screen.getByRole('button', { name: /add to uar library/i }))
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce())
    expect(fetchMock.mock.calls[0][0]).toBe('/api/uar/a2ui/components')
    await screen.findByRole('button', { name: /added to library/i })
  })
})
