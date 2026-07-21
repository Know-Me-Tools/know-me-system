## ADDED Requirements

### Requirement: KnowMe token parity in the UAR web UI

The UAR frontend SHALL render the KnowMe design-token values defined in `docs/knowme-ui-ux-standard.md` §4.1 as its single styling source, in both dark and light themes, across browser, Docker, and Tauri-webview deployments.

#### Scenario: Dark theme renders the KnowMe ladder

- **WHEN** the UAR web UI loads with the dark theme active
- **THEN** canvas, chrome, surface, and raised regions resolve to `#0B0F14`, `#111620`, `#161D29`, and `#1C2535` respectively
- **AND** the brand/action accent resolves to ember `#FF6A3D` with selection surface `#2B1A18`
- **AND** reasoning/citation/streaming accents resolve to cyan `#00C2DC`

#### Scenario: Light theme renders the KnowMe light values

- **WHEN** the light theme is active
- **THEN** the ladder resolves to `#F7F7F8`, `#FFFFFF`, `#FAFBFC`, `#FFFFFF` with ember `#E04E28` and selection surface `#FBE8E1`

#### Scenario: Tokens deploy identically on every surface

- **WHEN** the production `static/` bundle is served by the Axum binary in a browser, in Docker, or inside the Tauri webview
- **THEN** the same token values, fonts, and branding render without per-surface overrides

### Requirement: Flat 2.0 is enforced by CSS

The UAR frontend SHALL enforce Flat 2.0 at the stylesheet level: no visible borders, divider lines, decorative outlines, or layout shadows; adjacent regions SHALL differ only by background color, and status SHALL never be conveyed by color alone.

#### Scenario: Primitives cannot reintroduce lines

- **WHEN** any shadcn primitive, assistant-ui skin, or markdown content renders
- **THEN** borders, dividers, and layout shadows are neutralized by the global enforcement rules
- **AND** focus indication remains visible via an ember ring at ≥3:1 contrast against the adjacent surface

### Requirement: KnowMe typography stack

Body text SHALL use Geist Variable, display/titles Space Grotesk, and code/model IDs/timestamps JetBrains Mono; body size SHALL stay within 15–17px and never below 12px.

#### Scenario: Font loading has no third-party render dependency

- **WHEN** the UI loads without network access to font CDNs
- **THEN** Geist Variable renders from the bundled `@fontsource-variable/geist` package

### Requirement: Scoped exceptions are explicit

The TanStack Query state layer and the `/admin` CRT terminal theme SHALL remain as documented, deliberate exceptions to the KnowMe standard within this vendored submodule.

#### Scenario: Admin theme isolation

- **WHEN** a user navigates to `/admin`
- **THEN** the CRT terminal theme applies only while on admin routes
- **AND** product surfaces outside `/admin` never render terminal-theme tokens

#### Scenario: Exemptions are discoverable

- **WHEN** a contributor reads the change documentation or frontend README
- **THEN** the TanStack Query exemption, the Tailwind 3.4 port decision, and the admin-theme exception are each recorded with their rationale
