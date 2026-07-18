// TJ-ARCH-MOB-001 compliant
import type { ComponentType, SVGProps } from 'react'
import { BrainCircuit, Grid2X2, Layers3, MessageSquare, SlidersHorizontal, Zap } from 'lucide-react'

export interface Destination {
  path: string
  label: string
  icon: ComponentType<SVGProps<SVGSVGElement>>
}

/** One product inventory shared by desktop rail and phone bottom navigation. */
export const DESTINATIONS: readonly Destination[] = [
  { path: '/', label: 'Home', icon: Grid2X2 },
  { path: '/chat', label: 'Chat', icon: MessageSquare },
  { path: '/hands', label: 'Hands', icon: Zap },
  { path: '/memory', label: 'Memory', icon: BrainCircuit },
  { path: '/models', label: 'Models', icon: Layers3 },
  { path: '/settings', label: 'Settings', icon: SlidersHorizontal },
] as const
