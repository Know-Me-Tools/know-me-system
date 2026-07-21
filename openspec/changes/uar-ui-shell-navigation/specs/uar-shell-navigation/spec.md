## ADDED Requirements

### Requirement: Single-inventory width-adaptive navigation

The UAR web UI SHALL render top-level destinations from one shared inventory as a left rail on wide layouts and a bottom navigation bar on narrow layouts, with the switch driven by CSS width (no user-agent or OS sniffing).

#### Scenario: Wide layout renders the rail

- **WHEN** the viewport is at or above the `md` width boundary
- **THEN** a left rail on the chrome surface shows the brand mark and every destination
- **AND** the active destination is distinguished by an ember-tinted background with stronger ember text and `aria-current="page"`

#### Scenario: Narrow layout renders the bottom bar

- **WHEN** the viewport is below the `md` width boundary
- **THEN** a bottom navigation bar shows every destination with icon and label at ≥44px (`min-h-14`) targets
- **AND** the bar pads for `env(safe-area-inset-bottom)`
- **AND** the rail is hidden

#### Scenario: Inventory consistency

- **WHEN** the rail and bottom bar are compared
- **THEN** both render the same destinations in the same order from the shared `DESTINATIONS` source

### Requirement: Persistent readiness lane

A readiness lane SHALL be visible whenever the shell renders, conveying server readiness and deployment mode without relying on color alone.

#### Scenario: Readiness card content

- **WHEN** the shell renders on a wide layout
- **THEN** the rail footer shows a status indicator plus Ready/Degraded text and a deployment-mode line (embedded/local versus remote host)
- **AND** a degraded or unreachable server changes both the indicator and the text

### Requirement: Branded Tauri titlebar

Inside the Tauri shell the window SHALL use branded web-chrome titlebar with a drag region and window controls; outside Tauri no titlebar SHALL render.

#### Scenario: Tauri window chrome

- **WHEN** the UI runs inside the Tauri webview
- **THEN** the OS title bar is replaced by a branded strip with a working drag region and minimize/maximize/close controls
- **AND** macOS traffic-light buttons remain unobstructed per the platform matrix

#### Scenario: Browser has no titlebar

- **WHEN** the UI runs in a plain browser or Docker deployment
- **THEN** no titlebar strip renders and no window-control APIs are invoked

### Requirement: Shell regions separate by background

Shell and chat chrome regions (rail, sidebar, toolbar, canvas, bottom bar) SHALL be distinguished only by background-color steps of the surface ladder, never by borders, divider lines, or shadows.

#### Scenario: Chat chrome grep gate

- **WHEN** `App.tsx`, `chat-page.tsx`, `left-sidebar.tsx`, and `AppShell.tsx` are inspected
- **THEN** no `border-r`/`border-b` region separators remain, and the thread-list sidebar and agent toolbar render on ladder backgrounds
