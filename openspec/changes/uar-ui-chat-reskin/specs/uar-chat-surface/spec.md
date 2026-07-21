## ADDED Requirements

### Requirement: KnowMe chat anatomy in the UAR thread

The UAR chat surface SHALL render the KnowMe thread idioms defined in `docs/knowme-ui-ux-standard.md` §7: assistant prose as authored content on the canvas, user messages as ember-tinted trailing bubbles, and a filled-surface composer anchored at the viewport foot.

#### Scenario: Composer is a filled surface

- **WHEN** the chat composer renders in either theme
- **THEN** the shell is a filled `--color-surface` rounded surface with no border, shadow, ring, or backdrop blur
- **AND** focus-within shifts the shell to `--color-card-hov`
- **AND** no shimmer or decorative streaming animation is present

#### Scenario: User message anatomy

- **WHEN** a user message renders
- **THEN** it is trailing-aligned at max 80% width on an `--ember-soft` surface with `rounded-2xl rounded-ee-md` shaping
- **AND** no avatar is shown
- **AND** copy/resubmit/edit actions appear adjacent to the bubble on hover or focus

#### Scenario: Assistant message anatomy

- **WHEN** an assistant message renders
- **THEN** text parts sit on a `--color-card` surface with `rounded-2xl rounded-es-md` shaping
- **AND** no avatar is shown
- **AND** a footer row provides branch navigation and copy/regenerate/delete actions

### Requirement: Collapsed reasoning and tool grouping

Reasoning content SHALL be collapsed by default on a cyan-tinted surface with a visible streaming state, and consecutive tool calls SHALL collapse into a single grouped disclosure.

#### Scenario: Reasoning is opt-in

- **WHEN** an assistant message contains reasoning parts
- **THEN** the reasoning group renders collapsed by default with a trigger indicating streaming versus complete
- **AND** expanding the trigger reveals the reasoning content with `aria-busy` set while streaming

#### Scenario: Tool calls group

- **WHEN** an assistant message contains multiple consecutive tool calls
- **THEN** they render inside one collapsed group showing the call count and running state
- **AND** a single standalone tool call renders as an individual surface card with args and result panes

### Requirement: ContentBlock presentation parity

UAR's bespoke message-part renderers SHALL follow the KnowMe ContentBlock presentation conventions: memory on a cyan-eyebrow surface, citations with an ember eyebrow and expandable card, skill activations with an amber eyebrow, and artifacts on a card surface with sanitized markup.

#### Scenario: Memory chunk presentation

- **WHEN** a memory recall, mutation, or update part renders
- **THEN** it shows a cyan eyebrow label on a `--color-surface` rounded surface with no border

#### Scenario: Artifact safety

- **WHEN** an A2UI artifact part contains SVG or HTML
- **THEN** the markup is sanitized before render (DOMPurify or equivalent)

### Requirement: Token purity in the chat feature

The chat feature (`src/components/assistant-ui/`, `src/features/chat/`) SHALL contain no hardcoded palette utilities (`zinc-*`, `stone-*`, raw hex classes) and no elevation utilities (`shadow-*`), and motion SHALL be limited to short opacity/background transitions that honor `prefers-reduced-motion`.

#### Scenario: Grep gate

- **WHEN** the chat feature source is searched for `zinc-`, `stone-`, or `shadow-sm|md|lg|xl|2xl`
- **THEN** zero matches exist outside test fixtures
