---
type: Reference
id: knowme-uar-shell-navigation-verified-persistence-task-unblocked
title: KnowMe UAR shell navigation verified; persistence task unblocked
tags:
- knowme
- model-runtime-per-device
- uar
- ui-shell
- persistence
- submodules
- openspec
links:
- knowme-model-runtime-per-device-phase-status-and-proof-gaps
- knowme-uar-persistence-task-remains-blocked-by-submodule-checkout
- uar-durable-persistence-blocked-by-uninitialized-submodule
sources:
- stdin
timestamp: 2026-07-26T08:00:48.941690+00:00
created_at: 2026-07-26T08:00:48.941690+00:00
updated_at: 2026-07-26T08:00:48.941690+00:00
revision: 0
---

## Phase context

Active phase: `model-runtime-per-device` for KnowMe. Current position:

`model-runtime-per-device › embedded-uar-offline-agents › embed-uar-mobile-offline`

Status marker:

- Status: `implementation_ready`
- Progress: `changes 4/5`
- Last: `uar-ui-shell-navigation` complete — 17/17 tasks, OpenSpec validation clean.
- Next: resume `embed-uar-mobile-offline` task 3.1: durable sandboxed persistence.

Primary phase goals remain aligned with [KnowMe model-runtime-per-device phase status and proof gaps](/knowme-model-runtime-per-device-phase-status-and-proof-gaps.md):

- Ship Qwen3.5-9B Q4_K_M with matching `mmproj` vision support on iPhone 17 Pro Max using Metal.
- Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, with CPU as the universal correctness baseline.
- Enable device-aware model manifests, resumable downloads, integrity checks, runtime gate diagnostics, and safe memory/thermal fallback.
- Raise local generation limits and verify tool-capable model plumbing for future agent loops.

## `uar-ui-shell-navigation` completion

`uar-ui-shell-navigation` is complete:

- Task count: `17/17`.
- OpenSpec validation: clean.
- Smoke coverage: verified on the running app at `localhost:1906`.

Verified viewport/theme matrix:

| Theme | 1440px | 320px |
|---|---|---|
| Dark | rail + readiness card | bottom bar |
| Light | rail + readiness card | bottom bar |

## Task 3.3: layout stacking verification

Live DOM measurements confirmed the mobile bottom bar does not occlude page content:

- `main` ends at `648px`.
- Bottom bar starts at `648px`.
- Overlap: exactly `0px`.
- The bottom bar is a flex sibling of the content row, not an overlay.
- No horizontal overflow at 320px: `scrollWidth` `320` ≤ viewport `320`.
- Rail behavior:
  - `display: none` at `320px`.
  - visible at `1440px`.
  - `md:` breakpoint at `768px` sits between the smoke widths.

This verifies the shell geometry that prevents occlusion structurally. It does not verify composer occlusion because the composer did not render without backend configuration.

## Task 5.3: shell idiom verification

Verified shell navigation behavior:

- Desktop rail includes:
  - brand header,
  - three destinations,
  - footer readiness card.
- Readiness card uses dot plus text: `Unreachable / Embedded · local`; status is not communicated by color alone.
- Active states resolve correctly in both themes:
  - dark: `rgb(42, 26, 24)` dark ember-soft active pill,
  - light: cream active state with ember text.
- Flat 2.0 visual idiom holds: regions differ by background color only.
- Destinations render as real `<a href>` elements.
- Desktop and mobile navs have distinct `aria-label` values per the `mobile-navigation` skill.

Correction from visual inspection: the dark-mode active pill looked light in a screenshot, but computed style confirmed `rgb(42, 26, 24)`. The screenshot issue was image scaling, not a UI defect.

## Backend-dependent verification gaps

The composer, hamburger, and sidebar drawer did not render in the tested app state. With no backend, the chat page short-circuits to a `No Model Configured` guard. At 320px the DOM contained:

- `0` textareas,
- `0` composer elements,
- `0` hamburger buttons.

Therefore:

- Task 3.3 verifies shell layout and structural non-occlusion, not composer behavior.
- Task 5.3 does not verify hamburger/drawer interaction.
- These gaps were recorded in `openspec/changes/uar-ui-shell-navigation/verification-notes.md` and align with backend-dependent checks already deferred to `uar-ui-verify-gates`.

## Findings

### Theme toggle unreachable below 768px

`ThemeToggle` mounts only inside the desktop rail:

```text
rust/vendor/universal-agent-runtime/frontend/src/app/AppShell.tsx:86
```

The rail is `hidden md:flex`, so at 320px the accessibility tree includes only the three destination links and no theme control. Both themes render correctly at narrow width, but users cannot switch theme below the `md` breakpoint. This was filed rather than fixed because it is outside the completed change scope.

### Dev setup prerequisite is undocumented

`@prometheus-ags/prometheus-entity-management` is declared as a `workspace:*` dependency and ships `src/`, but resolves through `dist/`. Until `pnpm -r build` is run in `frontend/packages/prometheus-entity-management`, Vite import analysis fails and the app renders only an error overlay.

The package was built locally and the prerequisite was recorded in verification notes.

## Submodule correction: persistence work is unblocked

Previous records that treated durable persistence as blocked by an uninitialized submodule are superseded by this correction. The submodule is fully checked out:

- File count: `15,894`.
- Hash state: clean.
- Version: `v1.0.0-rc.9-210-g5343f29`.

The earlier failure was caused by a relative-path error in the shell command used to inspect the checkout, not by repository state. Therefore `embed-uar-mobile-offline` task 3.1 was never blocked by the submodule checkout. This corrects the blocker described in [KnowMe UAR persistence task remains blocked by submodule checkout](/knowme-uar-persistence-task-remains-blocked-by-submodule-checkout.md) and [UAR durable persistence blocked by uninitialized submodule](/uar-durable-persistence-blocked-by-uninitialized-submodule.md).

## Next implementation step

Resume `embed-uar-mobile-offline` task 3.1:

1. Read the `PersistenceLayer` trait and `providers/surreal.rs` in the confirmed-present submodule.
2. Decide reuse-vs-write for the app-sandboxed store based on code evidence.
3. Implement durable sandboxed persistence.
4. Keep the two pinned-shape tests green.
5. Run the Samsung restart gate.
6. Optionally fold the narrow-width theme-toggle gap into `uar-ui-verify-gates` alongside deferred hamburger/drawer checks.

# Citations

1. stdin