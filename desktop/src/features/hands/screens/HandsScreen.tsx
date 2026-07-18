// TJ-ARCH-MOB-001 compliant
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useHands } from '../hooks/useHands'

export function HandsScreen() {
  const { hands } = useHands()
  return <div className="h-full overflow-y-auto px-6 py-10 sm:px-12 lg:px-16"><div className="mx-auto max-w-5xl">
    <div className="flex items-start justify-between gap-6"><div><h1 className="text-4xl font-bold">Hands</h1><p className="mt-2 text-lg text-[color:var(--color-fg-sub)]">Reliable staff, not scripts.</p></div><Button size="lg">New hand</Button></div>
    <div className="mt-8 space-y-4">{hands.map((hand) => <Card key={hand.name} className="border-0 bg-[color:var(--color-surface)] shadow-none"><CardContent className="p-7"><div className="flex flex-wrap items-center gap-3"><h2 className="text-xl font-bold">{hand.name}</h2><span className="rounded-full bg-[color:var(--color-muted)] px-3 py-1 font-mono text-xs text-[color:var(--color-fg-sub)]">{hand.status}</span></div><p className="mt-3 text-[color:var(--color-fg-sub)]">{hand.purpose}</p><p className="mt-4 font-mono text-xs text-[color:var(--color-fg-faint)]">{hand.meta}</p></CardContent></Card>)}</div>
  </div></div>
}
