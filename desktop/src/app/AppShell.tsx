// TJ-ARCH-MOB-001 compliant — component imports router hooks and Shadcn primitives only.
import { Link, useRouterState } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
} from '@/components/ui/sidebar'
import { KnowMeLogo, KnowMeWordmark } from '@/shared/components/KnowMeLogo'
import { DESTINATIONS } from './navigation'

function useIsActive(path: string): boolean {
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  return path === '/' ? pathname === '/' : pathname.startsWith(path)
}

function DesktopDestination({ path, label, icon: Icon }: (typeof DESTINATIONS)[number]) {
  const active = useIsActive(path)
  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        render={<Link to={path} />}
        isActive={active}
        tooltip={label}
        className="h-12 rounded-xl px-4 text-sm font-semibold data-active:bg-[color:var(--color-ember-soft)] data-active:text-[color:var(--color-ember)]"
      >
        <Icon aria-hidden />
        <span>{label}</span>
        {label === 'Memory' ? <span className="ml-auto size-2 rounded-full bg-[color:var(--color-ember)]" aria-label="Memory updates pending" /> : null}
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}

function MobileDestination({ path, label, icon: Icon }: (typeof DESTINATIONS)[number]) {
  const active = useIsActive(path)
  return (
    <Link
      to={path}
      aria-current={active ? 'page' : undefined}
      className={active
        ? 'flex min-h-14 flex-1 flex-col items-center justify-center gap-1 rounded-xl bg-[color:var(--color-ember-soft)] px-1 text-[11px] font-semibold text-[color:var(--color-ember)]'
        : 'flex min-h-14 flex-1 flex-col items-center justify-center gap-1 rounded-xl px-1 text-[11px] font-semibold text-[color:var(--color-fg-sub)] hover:bg-[color:var(--color-card-hov)]'}
    >
      <Icon aria-hidden width={20} height={20} />
      <span>{label}</span>
    </Link>
  )
}

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <SidebarProvider defaultOpen className="h-[calc(100dvh-2.25rem)] min-h-0">
      <div className="flex h-full min-h-0 w-full flex-col overflow-hidden">
      <div className="flex min-h-0 w-full flex-1 bg-[color:var(--color-bg)]">
        <Sidebar collapsible="none" className="hidden w-64 bg-[color:var(--color-bg-2)] md:flex">
          <SidebarHeader className="gap-1 px-5 py-6">
            <div className="flex items-center gap-3 text-[color:var(--color-fg)]">
              <KnowMeLogo size={28} />
              <KnowMeWordmark className="text-xl" />
            </div>
            <p className="pl-10 text-xs text-[color:var(--color-fg-faint)]">AI that understands you.</p>
          </SidebarHeader>
          <SidebarContent>
            <SidebarGroup className="px-3">
              <SidebarGroupContent>
                <SidebarMenu className="gap-1">
                  {DESTINATIONS.map((destination) => <DesktopDestination key={destination.path} {...destination} />)}
                </SidebarMenu>
              </SidebarGroupContent>
            </SidebarGroup>
          </SidebarContent>
          <div className="m-4 rounded-xl bg-[color:var(--color-surface)] p-4 text-xs text-[color:var(--color-fg-sub)]">
            <div className="flex items-center gap-2 font-semibold text-[color:var(--color-fg)]">
              <span className="size-2 rounded-full bg-[color:var(--color-green)]" /> Ready
            </div>
            <p className="mt-1">On-device · local first</p>
          </div>
        </Sidebar>

        <main className="min-h-0 min-w-0 flex-1 overflow-hidden bg-[color:var(--color-bg)]">{children}</main>
      </div>

      <nav aria-label="Primary" className="flex shrink-0 gap-1 bg-[color:var(--color-bg-2)] p-2 pb-[calc(.5rem+env(safe-area-inset-bottom))] md:hidden">
        {DESTINATIONS.map((destination) => <MobileDestination key={destination.path} {...destination} />)}
      </nav>
      </div>
    </SidebarProvider>
  )
}
