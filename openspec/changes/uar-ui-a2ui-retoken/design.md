## Context

`a2ui-uar` is UAR's React renderer for the `uar.a2ui/1` generative-UI protocol. It deliberately ships a self-contained theme mechanism: `.uar-a2ui-surface` sets HSL-channel tokens scoped to the surface root, with `data-a2ui-theme="dark|light|high-contrast"` variants in `src/styles.css`, so surfaces render consistently no matter where they are embedded (chat, admin testing page, arbitrary hosts). The values predate the KnowMe convergence (old `#0F1923` navy family), and the surface root sets `color` but never `background` — embedding hosts are expected to paint one, which produces a measured white-on-light-canvas 1.07:1 contrast failure in the package's own high-contrast storybook test (the repo's only failing vitest).

The KnowMe values to port are authoritative in `frontend/src/index.css` (HSL channels), themselves a port of `desktop/src/index.css`.

## Goals / Non-Goals

**Goals:**

- All three package themes are value-identical to the KnowMe tokens (same HSL channels as `frontend/src/index.css` for dark/light; high-contrast rebased the same way).
- Every package theme is self-contained: the surface paints its own themed `background` and `color`; axe-measured contrast passes on every story, in every theme, on any host canvas.
- Package components render Flat 2.0: no borders, divider lines, or shadows; `--border: transparent` in the surface scope; controls separate by filled surfaces.
- The admin A2UI testing page still renders its fixtures (it embeds surfaces with explicit themes inside the CRT shell).
- vitest back to 153/153.

**Non-Goals:**

- No `uar.a2ui/1` protocol, schema, or `a2ui-surface-processor.ts` changes.
- No new components and no component API changes.
- No re-theme of the vendored Google A2UI sibling packages (`a2ui-core`, `a2ui-lit`, `a2ui-svelte`, `a2ui-inspector`) — reference-only code.
- No admin CRT shell changes.

## Decisions

### Keep the scoped-surface mechanism, port only values

The package's `.uar-a2ui-surface` + `data-a2ui-theme` design is the right embedding contract (a surface may render inside admin's CRT page or a third-party host; inheriting the page theme would break that). Only the channel values change, mirroring `frontend/src/index.css`:

- `dark` → KnowMe dark ladder (`--background: 213 29% 6%`, `--card: 218 31% 16%`, `--muted: 219 30% 21%`, `--muted-foreground: 216 26% 40%`, `--foreground: 213 31% 93%`, `--primary: 14 100% 62%`, `--ring: 14 100% 62%`, status ramp, `--cyan: 187 100% 43%`, `--ember-soft: 6 28% 13%`).
- `light` (the unprefixed default block) → KnowMe light (`240 7% 97%`, cards `0 0% 100%`, muted `216 24% 96%`, `--muted-foreground: 215 14% 34%`, `--primary: 12 75% 52%`, `--ring: 12 75% 52%`).
- `high-contrast` → the same rebase as the frontend HC theme (monochrome + yellow, new token names added: `--card-hov`, `--ember-soft`, `--cyan`, `--fg-sub`, `--fg-faint`).

### Paint the surface background — the a11y fix

`.uar-a2ui-surface` gains `background: hsl(var(--background))` (and keeps `color: hsl(var(--foreground))`). Rationale: a theme that controls text color but not its canvas is not a theme — the host's canvas leaks in and breaks contrast unpredictably (exactly the story failure). Embedded-in-chat surfaces sit on the chat block's `bg-card` wrapper; the painted surface background (`--background` dark → `#0B0F14`) replaces the wrapper visually at the surface root, which is the correct generative-UI canvas per theme. Rounded corners are preserved via the wrapper's `rounded-xl overflow-hidden`.

### Flat 2.0 inside the package

- Add `--border: transparent` to each theme block (package components reference `--border` for input/table chrome; it goes invisible like the frontend).
- Purge explicit border/shadow utilities in `src/components/` (`TextField` `border-input shadow-xs`, `Card` border classes, `Divider` line styles, `ChoicePicker` option borders) → ladder fills (`bg-input` → `bg-surface` for fields, `bg-card` cards, divider = vertical spacing with `aria-hidden`).
- Focus rings stay `--ring` at ≥3:1 (validated by the storybook a11y suite).

### Admin testing page renders package themes as-is

`A2uiTestingPage` passes explicit `theme` props; no admin-side changes beyond confirming fixtures. The CRT shell exception is untouched.

## Risks / Trade-offs

- **Painting the surface background changes chat artifact appearance** — surfaces become themed cards (`#0B0F14` dark) rather than blending into the message canvas. This is the correct contract (themed, self-contained surfaces) and matches how KnowMe renders artifacts (on-card, sanitized). Accepted.
- **Embedded third-party hosts** see a visual change (painted background). Mitigated by the surface being explicitly themed by the producer (`data-a2ui-theme` is set by the artifact payload/host, not guessed).
- **High-contrast story locale/direction combos** (the failing story is `ja` + `rtl` + HC) — the fix is theme-level, not story-level, so all variants pass together; verified by the full storybook suite, not just that story.
