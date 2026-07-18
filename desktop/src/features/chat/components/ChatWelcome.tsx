// TJ-ARCH-MOB-001 compliant — presentational Assistant UI welcome surface.
import { Brain, ShieldCheck, Sparkles } from 'lucide-react'
import { KnowMeLogo, KnowMeWordmark } from '@/shared/components/KnowMeLogo'

export function ChatWelcome() {
  return (
    <div className="mx-auto mb-8 max-w-xl px-6 text-center">
      <div className="mx-auto flex size-16 items-center justify-center rounded-2xl bg-[color:var(--color-ember-soft)] text-[color:var(--color-ember)]"><KnowMeLogo size={34} /></div>
      <div className="mt-5 flex items-center justify-center"><KnowMeWordmark className="text-3xl" /></div>
      <h1 className="mt-3 text-2xl font-bold tracking-tight text-[color:var(--color-fg)]">What would you like to understand?</h1>
      <p className="mx-auto mt-2 max-w-md text-sm leading-relaxed text-[color:var(--color-fg-sub)]">Think out loud, revisit what matters, or ask KnowMe to connect the details you have shared—privately, on your device.</p>
      <div className="mt-6 flex flex-wrap justify-center gap-2 text-xs text-[color:var(--color-fg-sub)]">
        <span className="flex items-center gap-1.5 rounded-full bg-[color:var(--color-surface)] px-3 py-2"><ShieldCheck className="size-3.5 text-[color:var(--color-green)]" />Local first</span>
        <span className="flex items-center gap-1.5 rounded-full bg-[color:var(--color-surface)] px-3 py-2"><Brain className="size-3.5 text-[color:var(--color-cyan)]" />Memory aware</span>
        <span className="flex items-center gap-1.5 rounded-full bg-[color:var(--color-surface)] px-3 py-2"><Sparkles className="size-3.5 text-[color:var(--color-amber)]" />Agent ready</span>
      </div>
    </div>
  )
}
