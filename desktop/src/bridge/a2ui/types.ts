// TJ-ARCH-MOB-001 compliant
export type { ContentBlock } from '@prometheus-ags/gen-ui-react'
import type { ContentBlock } from '@prometheus-ags/gen-ui-react'
export interface MessageUsage { inputTokens?: number; outputTokens?: number; cacheReadTokens?: number; thinkingTokens?: number }
export interface Message { id: string; role: 'user' | 'assistant' | 'system'; content: ContentBlock[]; timestamp: string; isStreaming?: boolean; usage?: MessageUsage }
