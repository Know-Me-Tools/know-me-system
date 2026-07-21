# Verification notes — uar-ui-chat-reskin

Recorded 2026-07-21 (kimi). Branch `uar-ui-chat-reskin` in the UAR submodule.

## Build gates

- `pnpm -C frontend typecheck` (`tsc -b`) → exit 0.
- `pnpm -C frontend lint` (`eslint .`) → exit 0.
- `pnpm -C frontend build` → `✓ built in 8.88s`; only pre-existing node_modules warnings (pglite `eval`, chunk size).
- `pnpm -C frontend test` (vitest, 32 files) → **152 passed / 1 failed**.

## The one failing test is pre-existing, not from this change

`storybook (chromium) | packages/a2ui-uar/src/components/protocol.stories.tsx > High Contrast Long Translation`
— axe `color-contrast` failures: white (`#ffffff`) text measured against page background `#f7f7f8`.

Mechanism: `.uar-a2ui-surface` (in `packages/a2ui-uar/src/styles.css`) sets `color:
hsl(var(--foreground))` but never paints `background`, so axe measures the page
canvas. The storybook test browser (headless chromium, no colorScheme configured in
`vitest.config.ts`) defaults to `prefers-color-scheme: light`, and both the old
(`#FAFAFA`, 1.04:1) and new (`#F7F7F8`, 1.07:1) light page values fail against the
package's white high-contrast foreground. The failure mechanism and near-identical
ratios predate this change and the token change; it belongs to
`uar-ui-a2ui-retoken` (the package's own theme system), where the surface should
paint its themed background.

## Grouped-parts anatomy

- `groupPartByType` + `MessagePrimitive.GroupedParts` verified available in the
  installed `@assistant-ui/react` 0.14.26 and compile clean.
- Reasoning groups: collapsed by default, cyan-tinted surface (`--cyan` at 7%),
  auto-open while streaming, auto-collapse after, manual toggle takes over.
- Tool groups: consecutive calls collapse into a count-labeled disclosure;
  standalone calls render the ToolCallBlock card on `--card`.
- UAR part dispatch (`__skill__`, `__context__`, `__memory_*`, `__a2ui_*`)
  preserved verbatim; DOM structure inside the message changed (group wrappers).

## Palette / motion purge (task 6.x)

- Grep gate on `src/components/assistant-ui/` and `src/features/chat/`: zero hits
  for `zinc-*`, `stone-*`, `shadow-sm|md|lg|xl|2xl`, `bg-emerald`, hardcoded
  `text-red/blue/gray-*`.
- Two strays fixed outside the gate paths: `lib/db-context.tsx` error dialog
  (border + shadow removed), `components/ui/tabs.tsx` (`data-active:shadow-sm`
  removed — missed by the earlier primitive sweep).
- Composer shimmer bar deleted; `shimmer` keyframe now has zero usages in `src/`
  (definition retained in `tailwind.config.ts`, harmless).
- Remaining motion: entrance fade/slide ≤200ms, streaming opacity pulses,
  background-color transitions, collapsible rotate animations with
  `motion-reduce:transition-none`.

## Deferred (need a running Axum backend / full loops)

- **7.3 Playwright e2e chat specs** — not run; the grouped-parts DOM restructure
  may require selector updates. Folded into `uar-ui-verify-gates`.
- **7.4 Two-theme manual smoke pass** vs the KnowMe reference — not run;
  screenshots at the matrix widths belong to `uar-ui-verify-gates`.
- **7.5 a11y-gate interactive checklist** — partially covered here (storybook a11y
  addon suite green except the pre-existing a2ui-uar failure; focus rings and
  contrast ratios verified in token-convergence notes). Full interactive pass
  stays in `uar-ui-verify-gates`.
- **Suggestion pills (1.5)** — intentionally omitted: UAR's external-store runtime
  exposes no suggestions adapter (design.md "otherwise omit" branch).
