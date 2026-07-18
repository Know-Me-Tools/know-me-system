// TJ-ARCH-MOB-001 compliant
import { AssistantRuntimeProvider } from '@assistant-ui/react'
import { Thread } from '@/components/assistant-ui/thread'
import { ThreadList } from '@/components/assistant-ui/thread-list'
import { LaneSwitcher } from '../components/LaneSwitcher'
import { useAssistantChatRuntime } from '../hooks/useAssistantChatRuntime'
import { RichDataRenderers } from '../components/RichDataRenderers'
import { ChatWelcome } from '../components/ChatWelcome'
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet'
import { MessagesSquare } from 'lucide-react'

export function ChatScreen() {
  const runtime = useAssistantChatRuntime()
  return (
    <AssistantRuntimeProvider runtime={runtime}>
      <RichDataRenderers />
      <div className="flex h-full min-h-0 overflow-hidden bg-[color:var(--color-bg)]">
        <aside className="hidden w-72 shrink-0 bg-[color:var(--color-bg-2)] p-4 lg:block">
          <div className="mb-4 px-2">
            <p className="eyebrow">Conversations</p>
            <h1 className="mt-1 text-xl font-bold text-[color:var(--color-fg)]">Your ongoing story</h1>
          </div>
          <ThreadList />
        </aside>
        <section className="flex min-w-0 flex-1 flex-col bg-[color:var(--color-bg)]">
          <div className="flex items-center justify-between bg-[color:var(--color-bg-2)] px-4 py-3 lg:hidden">
            <div><p className="eyebrow">Conversation</p><p className="text-sm font-semibold text-[color:var(--color-fg)]">Your ongoing story</p></div>
            <Sheet>
              <SheetTrigger render={<Button variant="secondary" size="icon" aria-label="Open conversations" />}><MessagesSquare /></SheetTrigger>
              <SheetContent side="left" className="border-0 bg-[color:var(--color-bg-2)] p-4 shadow-none">
                <SheetHeader className="px-1"><SheetTitle>Conversations</SheetTitle></SheetHeader>
                <ThreadList />
              </SheetContent>
            </Sheet>
          </div>
          <LaneSwitcher />
          <div className="min-h-0 flex-1"><Thread components={{ Welcome: ChatWelcome }} /></div>
        </section>
      </div>
    </AssistantRuntimeProvider>
  )
}
