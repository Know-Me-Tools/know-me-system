// TJ-ARCH-MOB-001 compliant
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useModels } from '../hooks/useModels'

export function ModelsScreen() {
  const { models, switchLane } = useModels()
  return <div className="h-full overflow-y-auto px-6 py-10 sm:px-12 lg:px-16"><div className="mx-auto max-w-5xl"><h1 className="text-4xl font-bold">Models</h1><p className="mt-2 text-lg text-[color:var(--color-fg-sub)]">Your model library, not a storefront.</p><div className="mt-8 space-y-4">{models.map((model) => <Card key={model.name} className={model.action === 'Active' ? 'border-0 bg-[color:var(--color-ember-soft)] shadow-none' : 'border-0 bg-[color:var(--color-surface)] shadow-none'}><CardContent className="flex items-center justify-between gap-6 p-6"><div><h2 className="text-xl font-bold">{model.name}</h2><div className="mt-3 flex flex-wrap gap-2">{model.tags.map((tag) => <span key={tag} className="rounded-full bg-[color:var(--color-muted)] px-3 py-1 font-mono text-xs text-[color:var(--color-fg-sub)]">{tag}</span>)}</div></div>{model.action === 'Active' ? <span className="font-bold text-[color:var(--color-ember)]">● Active</span> : <Button variant="secondary" onClick={() => void switchLane(model.id)}>{model.action}</Button>}</CardContent></Card>)}</div></div></div>
}
