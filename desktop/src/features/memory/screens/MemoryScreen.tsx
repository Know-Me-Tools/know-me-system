// TJ-ARCH-MOB-001 compliant — screen; composes the feature's components only.
import { MemoryPanel } from '../components/MemoryPanel'

/**
 * Memory (graph-RAG) surface — the React counterpart to Flutter's
 * memory_screen.dart.
 *
 * MemoryPanel and its store/hooks were already built and fully wired to the Rust
 * memory intents, but nothing mounted them: the router had a single '/' route
 * rendering chat. C-113's navigation is what finally surfaces this.
 */
export function MemoryScreen() {
  return (
    <div className="h-full overflow-y-auto p-4">
      <MemoryPanel />
    </div>
  )
}
