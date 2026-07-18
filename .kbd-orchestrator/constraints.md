# KBD Constraints — KnowMe

These constraints are derived from `AGENTS.md`, `AGENT_BASE_RULES.md`, and the
TJ-ARCH-MOB-001 architecture standard. KBD and every execution tool must apply
them before certifying or archiving a change.

## Blocking constraints

```yaml
constraints:
  - id: preserve-gen-ui-core-boundary
    severity: blocking
    description: "Networking, LLM, and persistence logic must remain in the shared Rust core."
    source: "AGENTS.md; docs/tj-arch-mob-001.html"

  - id: enforce-react-layering
    severity: blocking
    description: "React data flow must remain Component -> Hook -> Store -> invoke()/API; stores are the only invoke() layer."
    source: "AGENTS.md; AGENT_BASE_RULES.md Rule 16"

  - id: enforce-flutter-layering
    severity: blocking
    description: "Flutter data flow must remain Widget -> Riverpod provider -> Repository -> Rust FFI."
    source: "AGENTS.md; AGENT_BASE_RULES.md Rule 16"

  - id: preserve-feature-clean-architecture
    severity: blocking
    description: "Code must remain organized by feature or bounded context with strongly typed interfaces."
    source: "AGENT_BASE_RULES.md Rules 15 and 29"

  - id: preserve-entity-management
    severity: blocking
    description: "Normalized React server, async, and entity state must use @prometheus-ags/prometheus-entity-management 3.x; do not introduce TanStack Query."
    source: "AGENTS.md"

  - id: preserve-conversation-ownership
    severity: blocking
    description: "Assistant UI owns chat interaction state; durable conversations are Prometheus Entity Management entities backed by PGlite on web and pglite-oxide through typed Rust commands on Tauri."
    source: "AGENTS.md"

  - id: no-hardcoded-secrets
    severity: blocking
    description: "No hardcoded credentials, API keys, tokens, passwords, or private keys may enter source or logs."
    source: "AGENT_BASE_RULES.md Rule 33"

  - id: build-health-passes
    severity: blocking
    description: "The configured Rust, desktop, and mobile build-health checks must pass."
    command: "cd rust && cargo check --workspace && cd ../desktop && pnpm build && cd ../mobile && flutter analyze"

  - id: tests-pass
    severity: blocking
    description: "The configured Rust, desktop, and mobile test suites must pass for affected behavior."
    command: "cd rust && cargo test --workspace && cd ../desktop && pnpm test && cd ../mobile && flutter test"
```

## Warning constraints

```yaml
constraints:
  - id: flat-2-ui
    severity: warning
    description: "UI changes must preserve Flat 2.0: no visible borders, divider lines, decorative outlines, or layout shadows; regions differ by background color."
    source: "AGENTS.md"

  - id: shared-ui-components
    severity: warning
    description: "Prefer shadcn/ui on React and shadcn_flutter plus shared tokens on Flutter over raw controls."
    source: "AGENTS.md"

  - id: tests-for-new-behavior
    severity: warning
    description: "Add or update focused tests for every behavior changed."
    source: "AGENT_BASE_RULES.md Rule 30"

  - id: lint-clean
    severity: warning
    description: "Rust clippy, desktop ESLint, and Dart formatting checks should pass without warnings."
    command: "cd rust && cargo clippy --workspace --all-targets -- -D warnings && cd ../desktop && pnpm lint && cd ../mobile && dart format --output=none --set-exit-if-changed ."

  - id: verify-dependency-versions
    severity: warning
    description: "Any dependency addition or update must be verified against official current compatibility information and versions.toml."
    source: "AGENT_BASE_RULES.md Rules 22, 23, and 27"

  - id: cross-platform-parity
    severity: warning
    description: "Feature decisions must account for web, desktop, mobile, local, cloud, and degraded or offline operation where relevant."
    source: "AGENT_BASE_RULES.md Rule 14"
```

## Workflow triggers

```yaml
workflow_triggers:
  - event: on_iteration_complete
    action:
      type: command
      target: "<build_health_command from .kbd-orchestrator/project.json>"

  - event: on_change_complete
    action:
      type: command
      target: "<test_command from .kbd-orchestrator/project.json>"
```
