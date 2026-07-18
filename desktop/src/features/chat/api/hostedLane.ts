// TJ-ARCH-MOB-001 compliant — hosted Axum transport, called only by the store.
import type { A2uiWireEvent } from '@/bridge/a2ui/driver'

export interface HostedByok {
  provider: string
  model: string
  apiKey: string
  baseUrl: string | null
}

interface StartRunResponse {
  run_id: string
  events_url: string
}

function apiError(body: unknown, fallback: string): string {
  if (body && typeof body === 'object' && 'error' in body) {
    const error = (body as { error?: { message?: unknown } }).error
    if (typeof error?.message === 'string') return error.message
  }
  return fallback
}

export async function* streamHostedChat(
  message: string,
  history: { role: 'user' | 'assistant'; content: string }[],
  byok: HostedByok,
): AsyncGenerator<A2uiWireEvent> {
  const response = await fetch('/api/v1/chat/runs', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ message, history, byok: {
      provider: byok.provider,
      model: byok.model,
      api_key: byok.apiKey,
      base_url: byok.baseUrl,
    } }),
  })
  const body: unknown = await response.json().catch(() => null)
  if (!response.ok) throw new Error(apiError(body, `Chat request failed (${response.status})`))
  const run = body as StartRunResponse
  const streamResponse = await fetch(run.events_url, { headers: { accept: 'text/event-stream' } })
  if (!streamResponse.ok || !streamResponse.body) throw new Error(`Chat event stream failed (${streamResponse.status})`)

  const reader = streamResponse.body.getReader()
  const decoder = new TextDecoder()
  let pending = ''
  while (true) {
    const { value, done } = await reader.read()
    pending += decoder.decode(value, { stream: !done }).replaceAll('\r\n', '\n')
    const frames = pending.split('\n\n')
    pending = frames.pop() ?? ''
    for (const frame of frames) {
      const event = frame.split('\n').find((line) => line.startsWith('event:'))?.slice(6).trim()
      const data = frame.split('\n').filter((line) => line.startsWith('data:')).map((line) => line.slice(5).trimStart()).join('\n')
      if (event === 'a2ui' && data) yield JSON.parse(data) as A2uiWireEvent
      if (event === 'transport_error' && data) throw new Error(apiError(JSON.parse(data), 'Chat stream failed'))
    }
    if (done) break
  }
}
