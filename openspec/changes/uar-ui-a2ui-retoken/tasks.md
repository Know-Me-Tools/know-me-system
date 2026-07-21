# Tasks — uar-ui-a2ui-retoken

All paths relative to `rust/vendor/universal-agent-runtime/frontend/`. Skill gates: `hybrid-design-tokens` before token edits, `a11y-gate` for the contrast fix, `content-block-ui` for surface presentation conventions.

## 1. Package Token Port (`packages/a2ui-uar/src/styles.css`)

- [x] 1.1 Port KnowMe dark values into `.uar-a2ui-surface[data-a2ui-theme="dark"]` (ladder `213 29% 6%` / `220 31% 10%` / `218 30% 12%` / `218 31% 16%`, muted `219 30% 21%`, muted-fg `216 26% 40%`, fg ramp, ember `14 100% 62%`, ember-soft `6 28% 13%`, cyan `187 100% 43%`, status ramp)
- [x] 1.2 Port KnowMe light values into the default `.uar-a2ui-surface` block (`240 7% 97%`, cards `0 0% 100%`, muted `216 24% 96%`, muted-fg `215 14% 34%`, primary/ring `12 75% 52%`, ember-soft `16 76% 93%`, cyan `192 91% 36%`, light status ramp)
- [x] 1.3 Rebase `data-a2ui-theme="high-contrast"` onto the ladder structure with the new token names (`--card-hov`, `--ember-soft`, `--cyan`, `--fg-sub`, `--fg-faint`), monochrome + yellow, body pairs ≥7:1
- [x] 1.4 Add the new tokens (`--chrome`, `--card-hov`, `--fg-sub`, `--fg-faint`, `--cyan`, `--ember`, `--ember-soft`) to all three theme blocks so package components can consume the same vocabulary as the frontend

## 2. Self-Contained Surface Background (a11y fix)

- [x] 2.1 `.uar-a2ui-surface` paints `background: hsl(var(--background))` alongside `color: hsl(var(--foreground))`
- [x] 2.2 Verify the chat-embedded wrapper (`features/chat/components/a2ui-artifact-block.tsx`) keeps `rounded-xl overflow-hidden` so painted surfaces clip cleanly
- [x] 2.3 Run the storybook suite: `protocol.stories.tsx > High Contrast Long Translation` (and all other theme stories) pass axe `color-contrast`

## 3. Flat 2.0 Component Purge (`packages/a2ui-uar/src/components/`)

- [x] 3.1 Add `--border: transparent` to each theme block in `styles.css`
- [x] 3.2 `TextField`: replace `border border-input bg-transparent shadow-xs` with filled `bg-surface` (keep `focus-visible` ring on `--ring`)
- [x] 3.3 `Card`, `ChoicePicker`, `CheckBox`, `Button`: purge border/shadow utilities in favor of ladder fills (`bg-card`, hover `bg-card-hov`, selected `bg-ember-soft` + ember text)
- [x] 3.4 `Divider`: render as spacing with `aria-hidden` (no line), matching KnowMe "divider = spacing, never `<hr>`"
- [x] 3.5 `features/a2ui/a2ui-surface-renderer.tsx`: container wrapper drops `border border-border`, keeps `bg-card`

## 4. Verification

- [x] 4.1 `pnpm -C frontend typecheck`, `pnpm -C frontend lint`, `pnpm -C frontend build` green
- [x] 4.2 `pnpm -C frontend test` → **153/153** (the previously failing HC story must pass)
- [x] 4.3 `pnpm -C frontend build-storybook` green; review the A2UI stories in all three themes for border/shadow leaks
- [ ] 4.4 Manually render the admin `A2uiTestingPage` fixtures (dev server): surfaces render the new themes correctly inside the CRT admin shell; record before/after notes
- [x] 4.5 Contrast spot-check the package theme pairs (fg on each ladder step, ember on ember-soft, ring on surfaces, both light and dark) and record ratios in the change notes

## Apply notes (2026-07-21, kimi)

- 16/17 tasks complete; 4.4 (admin fixture review) deferred to uar-ui-verify-gates
  (needs dev server + backend).
- vitest is back to 153/153 — the pre-existing HC story failure is fixed by the
  surface background paint (task 2.1).
- Task 4.5 surfaced a second muted-foreground defect: dark was #4B6080 (2.07:1 on
  muted) — KnowMe maps muted-foreground to fg-sub, so dark is now #A7B0BC in both
  frontend/src/index.css and the a2ui-uar dark theme. See verification-notes.md.
