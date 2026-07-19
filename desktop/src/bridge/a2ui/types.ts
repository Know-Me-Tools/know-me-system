// TJ-ARCH-MOB-001 compliant
export type { ContentBlock } from '@prometheus-ags/gen-ui-react'
import type { ContentBlock } from '@prometheus-ags/gen-ui-react'
export interface MessageUsage { inputTokens?: number; outputTokens?: number; cacheReadTokens?: number; thinkingTokens?: number }
export type RunStatus = 'queued' | 'preparing' | 'thinking' | 'generating' | 'complete' | 'cancelled' | 'error'
export interface ChatRunError { code: string; phase: string; message: string; retryable: boolean; diagnostics?: string }
export interface Message {
  id: string
  role: 'user' | 'assistant' | 'system'
  content: ContentBlock[]
  timestamp: string
  updatedAt?: string
  isStreaming?: boolean
  usage?: MessageUsage
  runId?: string
  parentMessageId?: string
  branchId?: string
  runStatus?: RunStatus
  runPhase?: string
  statusMessage?: string
  error?: ChatRunError
  generationSettings?: Record<string, unknown>
  diagnostics?: Record<string, unknown>
  contextPlanVersion?: string
  contextTelemetry?: unknown
}
