// TJ-ARCH-MOB-001 compliant — UI consumes a hook, not the transport store.
import { useEffect } from 'react'
import { useChatControlStore } from '../stores/chatControlStore'

export function useChatControl(conversationId: string) {
  const state = useChatControlStore()
  useEffect(() => { void state.load(conversationId) }, [conversationId, state.load])
  return state
}
