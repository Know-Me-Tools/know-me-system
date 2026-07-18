---
name: hybrid-design-tokens
description: ALWAYS invoke before writing any color, spacing, typography, radius, or theme value on ANY surface (Tailwind config, shadcn CSS vars, shadcn_flutter ThemeData, Dart theme). One token source feeds both the React/Tailwind theme AND the Flutter theme. Triggers on design token, theme, palette, color, spacing, typography, dark mode, light mode, Tailwind config, shadcn theme, CSS variable, ThemeData, brand color, theme-factory.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** this skill operates under the 40 Prometheus Base Rules
> ([AGENT_BASE_RULES.md](../../AGENT_BASE_RULES.md) at the project root). Simplicity
> first, surgical changes, strict layering, strong typing, verified versions — the
> rules apply to every line this skill helps generate.

# Hybrid Design Tokens

There is **one source of truth** for design tokens in this project, and it feeds every
surface. Never hardcode a palette, spacing, radius, or type value twice. Never let the
Flutter theme and the Tailwind theme drift.

## Token source → outputs

```
tokens (single source: design/tokens.json)
   ├── web/desktop  → Tailwind 4 @theme + shadcn CSS custom properties (:root / .dark)
   └── mobile       → shadcn_flutter ThemeData (light + dark ColorScheme)
```

Use the project token compiler (`pnpm tokens:build`) to regenerate BOTH the Tailwind/shadcn
CSS token block and Flutter theme from `design/tokens.json`. Regenerate—do not hand-edit—the
derived theme files. `theme-factory` may help explore a palette, but it is not a cross-platform
token compiler and must never be cited as parity proof.

## Token categories (define once, reference everywhere)

- **Color** — semantic, not decorative: `canvas`, `chrome`, `surface`, `raised`, `hover`,
  `text`, `muted`, `accent`, `destructive`, `focus`. Provide a full light AND dark scale.
- **Spacing** — a scale (`space-1…space-section`), not per-component padding. Use fluid
  `clamp()` for section-level rhythm on web.
- **Typography** — at most two families with a deliberate pairing; fluid `clamp()` sizes
  (`text-base`, `text-hero`); `font-display: swap`; preload only the critical weight.
- **Radius / motion** — `radius`, `duration-*`, `ease-*`. KnowMe is strict Flat 2.0: no
  visible borders/divider lines and no layout shadows. Adjacent areas differ by background.

## Rules

1. **No magic values in components.** Web: reference `var(--…)` / Tailwind token classes.
   Flutter: reference `Theme.of(context).colorScheme` / a typed `AppTokens` extension —
   never a raw `Color(0xFF…)` or `EdgeInsets.all(13)` at a call site.
2. **Both themes must feel intentional.** Do not default to dark mode; style light AND dark
   deliberately and test both (see [[tauri-ui-review]], [[flutter-golden-ui]]).
3. **Semantic naming wins.** Name by role (`accent`, `destructive`), never by hue
   (`blue`, `red`) at the token-consumption site.
4. **Cross-surface parity.** The same semantic token must resolve to the same intent on
   web and mobile. When you change a token, regenerate BOTH outputs in the same change.
5. **ContentBlock styling flows from tokens.** Every [[content-block-ui]] variant styles
   from these tokens — no per-variant ad-hoc colors.

## Verify loop

- Web: inspect at 320/768/1024/1440 in both themes — see [[tauri-ui-review]].
- Flutter: golden test both `ColorScheme`s — see [[flutter-golden-ui]].
- Contrast: every text/background pair meets WCAG 2.2 AA — see [[a11y-gate]].

## Related skills

- `theme-factory` (external) — optional palette ideation only; not a compiler or parity proof
- [[content-block-ui]] — consumes these tokens for every variant
- [[a11y-gate]] — token contrast gate
