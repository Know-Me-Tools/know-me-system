# KnowMe — document system

The contract for KnowMe engineering documents produced with htmx-doc-forge: specifications, setup guides, class summaries and playbooks. Agents read this file before writing a page; `tokens.css` and `brand.json` carry the same decisions as values.

## Identity

- Deep charcoal (`#0B0F14`) and a single ember (`#E04E28`) are the identity colours. They belong to the logo, the PDF cover band and the app icon. They are not used for running text or controls.
- Ember is also the working accent, under the single-accent restraint principle: one accent per view, used for links, section numbers, the active element and the primary button only: links, active contents entries, section numbers, diagram highlights and primary buttons.
- The mark is the horizontal KnowMe lockup. In the web header its navy and lime are recoloured through `--logo-ink` and `--logo-accent` so it turns white and light lime in dark mode.
- Documents are prepared by KnowMe, LLC; that credit appears in the footer and on the PDF cover, never in the title.

## Voice

- Calm, precise, engineering register. Short declarative sentences; numbers with units; dates as `30 Sep 2026`.
- Say what a thing is before what it does. Name the evidence for every claim; mark inferences as inferences.
- No hype words, no exclamation marks, no emoji. Second person (“you”) for procedures, third person for specifications.
- One name per artifact. Never let two documents or two code paths share a name.

## Color

- Surfaces separate by tone, never by borders (Flat 2.0): canvas `#F7F8FB`, surface `#FFFFFF`, surface-2 `#F1F3F8`, surface-3 `#E3E6EE`.
- Text: body `#2E3340`, muted `#565D72`, headings `#15025D`.
- Status: success `#4E7A00`, warning `#9A5B00`, danger `#B4231C`, info `#2A4597`, each with a soft tint for callouts and chips.
- Dark mode is first-class: canvas `#08090E`, violet becomes `#9593FF`, lime `#A6D45C`. Every pairing meets WCAG AA in both modes.

## Typography

- Display: Space Grotesk 600–800 for titles, section heads, labels and diagram titles.
- Body: Roboto 400–600 at 16 px / 1.62 on screen, 9.6 pt in print.
- Mono: JetBrains Mono for code, paths, commands and identifiers.
- Eyebrows and small labels are uppercase with 0.12–0.16 em tracking. Titles balance across lines and stay under 22 characters per line where possible.

## Layout

- Sticky top bar with the lockup; a hero with eyebrow, title, lede and meta chips; a sticky contents rail beside a reading column of at most 74 characters.
- Below 860 px the rail folds away, a bottom tab bar appears (Contents, Previous, Next, Top, PDF) and the contents open as a sheet. Transitions glide; nothing jumps.
- Sections are numbered `00`, `01` … in violet. Each section is a surface card with 14 px radius.

## Components

- Tables: zebra rows, uppercase Space Grotesk header on surface-3, first column in heading colour, no rules.
- Callouts: info (default), warn, good and accent tints with an uppercase label.
- Tiles for three headline facts; chips for status (done, in progress, to do, review, new, critical).
- Code blocks on `#171A24` with muted comments; inline code on surface-2.
- Blockquotes for attributed quotes from meetings and classes, with the speaker and date as the citation.

## Diagrams

- Inline SVG, 900 units wide, drawn with the `--d-*` tokens so they switch with the theme.
- Every multi-step diagram plays as a sequence: one caption per step, the active element outlined in violet.
- Boxes are flat surfaces with a 1.2 px edge; arrows use the edge colour; lime marks the positive or reviewed path; warn tints mark exceptions.
- Labels must fit inside their boxes at 1440 px and remain legible on a phone after horizontal scroll.

## Print

- US Letter. Cover page on surface-2 with the lockup, eyebrow, title, lede, meta chips and a navy band carrying the document footer.
- A contents page with dotted leaders and page numbers; each numbered section starts a new page; running header with document title and current section; “Page X of Y” footer.
- Interactive chrome (play buttons, “press play” hints, step captions) never reaches the PDF.

## Do and do not

- Do keep one accent per view; do let tone, not lines, separate surfaces; do label every figure.
- Do not put lime text on white (it fails contrast); do not use navy for links; do not add drop shadows except the mobile tab bar.
- Do not invent KnowMe facts, people or decisions; cite the meeting, document or code they come from.
