## MODIFIED Requirements

### Requirement: KnowMe token parity in the UAR web UI

Both themes SHALL be reachable by the user and the stored theme preference SHALL be applied at application boot.

#### Scenario: Stored theme applies at boot

- **WHEN** the app loads with `uar-theme` set to `light` in localStorage
- **THEN** the document renders the light theme without any user interaction

#### Scenario: Theme switching is reachable

- **WHEN** the shell renders
- **THEN** a theme control is visible and cycles dark → light → high-contrast → system

### Requirement: e2e suite integrity

The Playwright e2e suite SHALL assert against conditions that can actually occur (no impossible waits), and its selectors SHALL match the converged UI's real contracts.

#### Scenario: No impossible waits

- **WHEN** any e2e spec runs against the app with SSE streams open
- **THEN** no spec waits for `networkidle`, and every visibility assertion uses an explicit condition with a real budget

#### Scenario: Selector contracts

- **WHEN** a spec locates the agent selector
- **THEN** it uses the component's accessibility contract (`aria-label="Select agent"`), not display text

### Requirement: Verification evidence

The convergence SHALL carry a screenshot matrix and an a11y checklist record as committed evidence.

#### Scenario: Matrix coverage

- **WHEN** the change is reviewed
- **THEN** screenshots exist for `/threads`, `/about`, `/admin` at 320/768/1024/1440 in dark and light, with an inspection record

#### Scenario: Environment limits are documented

- **WHEN** a spec cannot pass in the default environment (embeddings, dedicated configs, Tauri window)
- **THEN** the limitation and its intended environment are recorded in verification notes rather than the spec being silently skipped
