// TJ-ARCH-MOB-001 compliant
import { createRouter, createRoute, createRootRoute, Outlet } from '@tanstack/react-router'
import { ChatScreen } from '@/features/chat/screens/ChatScreen'
import { MemoryScreen } from '@/features/memory/screens/MemoryScreen'
import { HomeScreen } from '@/features/home/screens/HomeScreen'
import { HandsScreen } from '@/features/hands/screens/HandsScreen'
import { ModelsScreen } from '@/features/models/screens/ModelsScreen'
import { SettingsScreen } from '@/features/settings/screens/SettingsScreen'
import { AppShell } from './AppShell'

// AppShell wraps every route, so the nav chrome is mounted once rather than by
// each screen. See AppShell for why placement responds to width, not platform.
const rootRoute = createRootRoute({
  component: () => (
    <AppShell>
      <Outlet />
    </AppShell>
  ),
})

// Authentication is intentionally absent until a real backend and route gate exist.
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: HomeScreen,
})

const chatRoute = createRoute({ getParentRoute: () => rootRoute, path: '/chat', component: ChatScreen })
const handsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/hands', component: HandsScreen })

// Surfaces the memory/graph-RAG panel, which was built and wired but unreachable
// until C-113 added navigation. Paths here must match src/app/navigation.ts.
const memoryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/memory',
  component: MemoryScreen,
})
const modelsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/models', component: ModelsScreen })
const settingsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/settings', component: SettingsScreen })

export const router = createRouter({
  routeTree: rootRoute.addChildren([indexRoute, chatRoute, handsRoute, memoryRoute, modelsRoute, settingsRoute]),
})

declare module '@tanstack/react-router' {
  interface Register { router: typeof router }
}
