import { afterEach, describe, expect, it, vi } from 'vitest'
import { streamHostedA2uiAction, streamHostedChat } from '../api/hostedLane'

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('hosted A2UI actions', () => {
  it('keeps the producing UAR run on streamed A2UI artifacts', async () => {
    const encoder = new TextEncoder()
    const response = new Response(new ReadableStream({
      start(controller) {
        controller.enqueue(encoder.encode('event: agui.artifact\ndata: {"request_id":"source-run-7","artifact_id":"artifact-9","artifact_type":"a2ui","content":"{}"}\n\n'))
        controller.close()
      },
    }), { status: 200 })
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response))

    const events = []
    for await (const event of streamHostedChat('render', [], null, 'conversation-1', 'main')) events.push(event)
    expect(events).toContainEqual({
      type: 'block',
      block: { type: 'artifact', id: 'source-run-7', kind: 'a2ui', content: '{}' },
    })
  })

  it('keeps the producing run on official AG-UI CUSTOM A2UI artifacts', async () => {
    const encoder = new TextEncoder()
    const response = new Response(new ReadableStream({
      start(controller) {
        controller.enqueue(encoder.encode('event: CUSTOM\ndata: {"type":"CUSTOM","name":"uar.artifact.available","runId":"source-run-8","value":{"artifactId":"a2ui:random-document","artifactType":"a2ui","language":"application/a2ui+json","content":"{}"}}\n\n'))
        controller.close()
      },
    }), { status: 200 })
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response))

    const events = []
    for await (const event of streamHostedChat('render', [], null, 'conversation-1', 'main')) events.push(event)
    expect(events).toContainEqual({
      type: 'block',
      block: { type: 'artifact', id: 'source-run-8', kind: 'a2ui', content: '{}' },
    })
  })

  it('continues the action run using the canonical AG-UI stream', async () => {
    const continuationRunId = 'continuation-run'
    const streamBody = [
      'event: RUN_STARTED',
      `data: ${JSON.stringify({ type: 'RUN_STARTED', runId: continuationRunId })}`,
      '',
      'event: TEXT_MESSAGE_CONTENT',
      `data: ${JSON.stringify({ type: 'TEXT_MESSAGE_CONTENT', runId: continuationRunId, delta: 'continued' })}`,
      '',
      'event: RUN_FINISHED',
      `data: ${JSON.stringify({ type: 'RUN_FINISHED', runId: continuationRunId })}`,
      '',
      '',
    ].join('\n')
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ continuation_run_id: continuationRunId }), {
        status: 202,
        headers: { 'content-type': 'application/json' },
      }))
      .mockResolvedValueOnce(new Response(streamBody, {
        status: 200,
        headers: { 'content-type': 'text/event-stream' },
      }))
    vi.stubGlobal('fetch', fetchMock)

    const events = []
    for await (const event of streamHostedA2uiAction({
      action: {
        surfaceId: 'demo',
        name: 'continueDemo',
        sourceComponentId: 'continue',
        context: {},
      },
    }, 'source-run')) events.push(event)

    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      `/api/uar/runs/${continuationRunId}/stream?stream_mode=agui_spec`,
      { headers: { accept: 'text/event-stream' } },
    )
    expect(events).toEqual([
      { type: 'run_started', run_id: continuationRunId },
      { type: 'block', block: { type: 'text', text: 'continued' } },
      { type: 'run_finished', run_id: continuationRunId },
    ])
  })
})
