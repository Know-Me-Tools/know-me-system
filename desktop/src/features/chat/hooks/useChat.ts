// TJ-ARCH-MOB-001 compliant
import { useChatStore } from '../stores/chatStore'

export function useChat() {
  return {
    messages: useChatStore((state) => state.messages),
    isStreaming: useChatStore((state) => state.isStreaming),
    sendMessage: useChatStore((state) => state.sendMessage),
  }
}
