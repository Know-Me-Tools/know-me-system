// TJ-ARCH-MOB-001 compliant
import { RouterProvider } from '@tanstack/react-router'
import { TooltipProvider } from "@/components/ui/tooltip"
import { router } from './router'

export function AppProviders() {
  return (
    <TooltipProvider>
      <RouterProvider router={router} />
    </TooltipProvider>
  )
}
