// TJ-ARCH-MOB-001 compliant — direct UAR HTTP transport, called only by the store.
import type { A2uiWireEvent } from '@/bridge/a2ui/driver'

export interface HostedByok {
  provider: string
  model: string
  apiKey: string
  baseUrl: string | null
}

type JsonObject = Record<string, unknown>

function object(value: unknown): JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value) ? value as JsonObject : {}
}

function string(value: unknown): string {
  return typeof value === 'string' ? value : ''
}

function apiError(body: unknown, fallback: string): string {
  const root = object(body)
  const error = object(root.error)
  return string(error.message) || string(root.message) || string(root.error) || fallback
}

function artifact(_runId: string, event: string, payload: JsonObject): A2uiWireEvent {
  return {
    type: 'block',
    block: {
      type: 'artifact',
      id: string(payload.eventId) || crypto.randomUUID(),
      kind: event,
      content: JSON.stringify(payload),
    },
  }
}

function mapAgui(event: string, payload: JsonObject, fallbackRunId: string): A2uiWireEvent | null {
  const runId = string(payload.runId) || string(payload.run_id) || fallbackRunId
  switch (event) {
    case 'RUN_STARTED': return { type: 'run_started', run_id: runId }
    case 'TEXT_MESSAGE_CONTENT': return { type: 'block', block: { type: 'text', text: string(payload.delta) } }
    case 'REASONING_MESSAGE_CONTENT': return { type: 'block', block: { type: 'thinking', text: string(payload.delta) } }
    // Framing/delta events are represented by their completed semantic block
    // or by message lifecycle state. They are known protocol envelopes, not
    // user-visible artifacts.
    case 'TEXT_MESSAGE_START':
    case 'TEXT_MESSAGE_END':
    case 'TOOL_CALL_START':
    case 'TOOL_CALL_ARGS':
    case 'STEP_FINISHED': return null
    case 'STEP_STARTED': return { type: 'run_phase_changed', run_id: runId, phase: string(payload.stepName) || 'generation', status: 'generating', message: 'Agent is working' }
    case 'STATE_DELTA': return { type: 'context_telemetry', run_id: runId, payload: payload.delta }
    case 'TOOL_CALL_END': return { type: 'block', block: { type: 'toolUse', id: string(payload.toolCallId), name: string(payload.toolCallName), inputJson: JSON.stringify(payload.arguments ?? {}) } }
    case 'TOOL_CALL_RESULT': return { type: 'block', block: { type: 'toolResult', toolUseId: string(payload.toolCallId), outputJson: string(payload.content), isError: false } }
    case 'agui.artifact': {
      const kind = string(payload.artifact_type) || string(payload.artifactType) || 'artifact'
      const isA2ui = /(^a2ui$|application\/a2ui\+json)/i.test(kind)
      return { type: 'block', block: {
      type: 'artifact',
      id: (isA2ui ? string(payload.request_id) || string(payload.requestId) : '')
        || string(payload.artifact_id) || string(payload.artifactId) || runId || crypto.randomUUID(),
      kind,
      content: string(payload.content) || JSON.stringify(payload),
    } }
    }
    case 'RUN_FINISHED': return { type: 'run_finished', run_id: runId }
    case 'RUN_ERROR': {
      const code = string(payload.code) || 'uar_run_error'
      // Same phrase as every Rust lane (gen_ui_agent/src/chat.rs) and the other
      // engines. One state, one wording — the user sees a stop, not a lane name.
      if (code === 'CANCELLED') return { type: 'run_cancelled', run_id: runId, phase: 'generation', message: string(payload.message) || 'Cancelled by user' }
      return { type: 'run_error', run_id: runId, error: { code, phase: 'generation', message: string(payload.message) || 'UAR run failed', retryable: true } }
    }
    case 'CUSTOM': {
      const name = string(payload.name)
      const value = object(payload.value)
      if (name === 'uar.skill.activated') {
        const skill = object(value.skill)
        return { type: 'block', block: { type: 'skill', name: string(skill.title) || string(skill.id), status: 'activated' } }
      }
      if (name === 'uar.citation.added') {
        const citation = object(value.citation)
        return { type: 'block', block: { type: 'citation', source: string(citation.source) || string(citation.url) || 'UAR citation', quote: string(citation.quote) || JSON.stringify(value.citation ?? value) } }
      }
      if (name === 'uar.memory.recall') {
        return { type: 'block', block: { type: 'memory', operation: 'recall', key: 'items', value: JSON.stringify(value.items ?? []) } }
      }
      if (name.startsWith('uar.artifact.')) {
        const kind = string(value.artifactType) || name
        const isA2ui = /(^a2ui$|application\/a2ui\+json)/i.test(kind)
          || string(value.language) === 'application/a2ui+json'
        return { type: 'block', block: {
          type: 'artifact',
          // Official AG-UI carries UAR artifacts as CUSTOM events. The
          // artifact ID identifies the document, while actions must target
          // the producing run that owns the live A2UI surface.
          id: (isA2ui ? string(value.sourceRunId) || runId : '')
            || string(value.artifactId) || crypto.randomUUID(),
          kind,
          content: string(value.content) || JSON.stringify(value),
        } }
      }
      return artifact(runId, name || event, payload)
    }
    default:
      // runtime.* events duplicate the official AG-UI events and can be safely
      // ignored. Every other custom event is preserved as an artifact.
      return event.startsWith('runtime.') || !event ? null : artifact(runId, event, payload)
  }
}

async function* readAguiResponse(response: Response, fallbackRunId: string): AsyncGenerator<A2uiWireEvent> {
  if (!response.ok || !response.body) {
    const body: unknown = await response.json().catch(() => null)
    throw new Error(apiError(body, `UAR stream failed (${response.status})`))
  }
  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  let pending = ''
  let runId = fallbackRunId
  while (true) {
    const { value, done } = await reader.read()
    pending += decoder.decode(value, { stream: !done }).replaceAll('\r\n', '\n')
    const frames = pending.split('\n\n')
    pending = frames.pop() ?? ''
    for (const frame of frames) {
      const event = frame.split('\n').find((line) => line.startsWith('event:'))?.slice(6).trim() ?? ''
      const data = frame.split('\n').filter((line) => line.startsWith('data:')).map((line) => line.slice(5).trimStart()).join('\n')
      if (!data || data === '[DONE]') continue
      const payload = object(JSON.parse(data))
      const mapped = mapAgui(event, payload, runId)
      if (mapped?.type === 'run_started') runId = mapped.run_id
      if (mapped) yield mapped
    }
    if (done) break
  }
}

export async function* streamHostedChat(
  message: string,
  history: { role: 'user' | 'assistant'; content: string }[],
  _byok: HostedByok | null,
  conversationId: string,
  branchId: string,
): AsyncGenerator<A2uiWireEvent> {
  const response = await fetch('/api/chat/completion', {
    method: 'POST',
    headers: { 'content-type': 'application/json', accept: 'text/event-stream', 'x-knowme-branch-id': branchId },
    body: JSON.stringify({
      messages: [...history, { role: 'user', content: message }],
      stream: true,
      stream_mode: 'agui_spec',
      session_id: conversationId,
      agent_id: 'orchestrator-agent',
    }),
  })
  for await (const event of readAguiResponse(response, conversationId)) yield event
}

export async function* streamHostedA2uiAction(interaction: unknown, sourceRunId: string): AsyncGenerator<A2uiWireEvent> {
  const envelope = object(interaction)
  const action = object(envelope.action)
  const response = await fetch(`/api/uar/runs/${encodeURIComponent(sourceRunId)}/a2ui/actions`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      surfaceId: string(action.surfaceId) || string(envelope.surfaceId),
      name: string(action.name),
      sourceComponentId: string(action.sourceComponentId),
      timestamp: string(action.timestamp) || new Date().toISOString(),
      context: object(action.context),
      a2uiClientDataModel: envelope.a2uiClientDataModel,
    }),
  })
  const ack: unknown = await response.json().catch(() => null)
  if (!response.ok) throw new Error(apiError(ack, `A2UI action failed (${response.status})`))
  const continuationRunId = string(object(ack).continuation_run_id) || string(object(ack).continuationRunId)
  if (!continuationRunId) throw new Error('UAR did not return an A2UI continuation run')
  const stream = await fetch(
    `/api/uar/runs/${encodeURIComponent(continuationRunId)}/stream?stream_mode=agui_spec`,
    { headers: { accept: 'text/event-stream' } },
  )
  for await (const event of readAguiResponse(stream, continuationRunId)) yield event
}

export async function cancelHostedChat(conversationId: string): Promise<void> {
  const response = await fetch(`/api/uar/sessions/${encodeURIComponent(conversationId)}/cancel`, { method: 'POST' })
  if (!response.ok) throw new Error(`UAR cancellation failed (${response.status})`)
}
