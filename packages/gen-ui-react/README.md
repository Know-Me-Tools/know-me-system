# @prometheus-ags/gen-ui-react

ContentBlock React 19 components for the TJ-ARCH-MOB-001 hybrid architecture.

`ContentBlockView` renders one `ContentBlock` — the cross-platform UI contract
produced by the shared Rust core (`gen_ui_core`). The type mirrors
`gen_ui_types::ContentBlock` exactly; the render switch is exhaustive over all 11
variants (a missing case is a TypeScript compile error).

Presentational only: no data fetching, no `invoke()`. Feed it blocks from a
Zustand store that owns the stream subscription.

```tsx
import { ContentBlockView, type ContentBlock } from "@prometheus-ags/gen-ui-react";

function Transcript({ blocks }: { blocks: ContentBlock[] }) {
  return <>{blocks.map((b, i) => <ContentBlockView key={i} block={b} />)}</>;
}
```
