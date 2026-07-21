## MODIFIED Requirements

### Requirement: KnowMe token parity in the UAR web UI

The `a2ui-uar` generative-UI renderer SHALL render its scoped surface themes (`data-a2ui-theme="dark|light|high-contrast"`) with values identical to the KnowMe token set, and each theme SHALL be self-contained: the surface paints its own themed background and foreground.

#### Scenario: Dark A2UI theme matches KnowMe

- **WHEN** a `uar.a2ui/1` surface renders with `data-a2ui-theme="dark"`
- **THEN** its canvas, card, muted, foreground, ember, and cyan tokens resolve to the same values as the frontend dark theme
- **AND** the surface root paints its themed background rather than inheriting the host page canvas

#### Scenario: High-contrast theme is contrast-valid on any host

- **WHEN** a surface renders with `data-a2ui-theme="high-contrast"` on any host page (dark, light, or unstyled)
- **THEN** text and controls measure ≥7:1 (body) against the surface's own painted background
- **AND** the storybook a11y suite reports zero `color-contrast` violations for every theme story

### Requirement: Flat 2.0 is enforced by CSS

Agent-generated A2UI surfaces SHALL render Flat 2.0 inside the surface scope: no visible borders, divider lines, or shadows; controls and regions separate by filled ladder surfaces; dividers render as spacing.

#### Scenario: Package components cannot draw lines

- **WHEN** package components (`TextField`, `Card`, `ChoicePicker`, `CheckBox`, `Button`, `Divider`) render in any theme
- **THEN** no visible border, outline, or shadow appears, inputs are filled surfaces, and focus indication remains a `--ring` ring at ≥3:1

#### Scenario: Embedded container separation

- **WHEN** an A2UI surface is embedded in the chat thread via `a2ui-surface-renderer.tsx`
- **THEN** the container separates from the message canvas by background color alone, with rounded clipping preserved

### Requirement: Scoped exceptions are explicit

The re-tokened package themes SHALL remain scoped to `.uar-a2ui-surface` and SHALL NOT leak into or depend on the host page theme; the admin CRT shell exception is unaffected.

#### Scenario: Admin testing page isolation

- **WHEN** the admin `A2uiTestingPage` renders fixtures with explicit themes inside the CRT admin shell
- **THEN** the fixtures render the package's KnowMe-valued themes while the surrounding admin page keeps its terminal aesthetic
