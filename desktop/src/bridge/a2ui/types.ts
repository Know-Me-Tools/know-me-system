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
  /** Absolute filesystem paths of the files the user attached to this turn.
   *
   *  Kept alongside the `artifact` blocks that render them: the blocks are the
   *  transcript, these are the inference payload Rust reads and extracts. Both
   *  are persisted so a resubmit can re-send the same files. */
  attachmentPaths?: string[]
  /** Which agent + provider + model produced this assistant turn, projected from
   *  the run's effective_run_policy. Any field may be absent. */
  provenance?: MessageProvenance
  /** Why the user sent this turn: 'steer' (redirect after a stop, keeping the
   *  partial work) or 'resume' (carry on from where the stop landed). Absent on
   *  an ordinary turn.
   *
   *  Display and telemetry only — it is NOT how the intent reaches the model.
   *  That travels in the message content, the one carrier every lane forwards
   *  verbatim (see `sendSteer` in chatStore). Unlike `provenance`, this IS
   *  persisted: provenance is derived and recomputable, whereas this is a record
   *  of what the user did and is unrecoverable after a reload. */
  intent?: MessageIntent
}

/** What the user meant by a turn sent from a stopped run. */
export type MessageIntent = 'steer' | 'resume'

export interface MessageProvenance {
  agentId?: string
  providerId?: string
  modelId?: string
}
