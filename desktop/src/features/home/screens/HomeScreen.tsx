// TJ-ARCH-MOB-001 compliant
import { Link } from '@tanstack/react-router'
import { Card, CardContent } from '@/components/ui/card'
import { useHome } from '../hooks/useHome'

export function HomeScreen() {
  const { summary, capabilities } = useHome()
  return (
    <div className="h-full overflow-y-auto px-6 py-10 sm:px-12 lg:px-16">
      <div className="mx-auto max-w-6xl">
        <p className="eyebrow">// KnowMe · Today</p>
        <h1 className="mt-3 text-4xl font-bold tracking-tight sm:text-5xl">Good afternoon.</h1>
        <p className="mt-2 text-lg text-[color:var(--color-fg-sub)]">Here&apos;s where things stand.</p>

        <section aria-label="System summary" className="mt-8 grid gap-4 lg:grid-cols-3">
          {summary.map((item) => (
            <Card key={item.label} className="border-0 bg-[color:var(--color-surface)] shadow-none">
              <CardContent className="p-6">
                <p className="eyebrow">{item.label}</p>
                <p className="mt-2 text-2xl font-bold">{item.value}</p>
                <p className="mt-2 flex items-center gap-2 text-sm text-[color:var(--color-fg-sub)]">
                  <span className="size-2 rounded-full bg-[color:var(--color-green)]" />{item.detail}
                </p>
              </CardContent>
            </Card>
          ))}
        </section>

        <section className="mt-10">
          <h2 className="text-lg font-bold">Capabilities</h2>
          <div className="mt-4 grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
            {capabilities.map(([name, description, path]) => (
              <Link key={name} to={path} className="rounded-xl bg-[color:var(--color-surface)] p-6 transition-colors hover:bg-[color:var(--color-card-hov)]">
                <h3 className="font-bold">{name}</h3>
                <p className="mt-2 text-sm leading-relaxed text-[color:var(--color-fg-sub)]">{description}</p>
              </Link>
            ))}
          </div>
        </section>
      </div>
    </div>
  )
}
