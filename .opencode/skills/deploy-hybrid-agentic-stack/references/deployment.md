# Deployment requirements

## Container image

Use a multi-stage build: Node 24 + pnpm for the tracked React build, Rust 1.96 for the
server, and a minimal non-root runtime. Pin base images and dependent Prometheus repo
revisions or digests. Include health and readiness probes and an SBOM/provenance step.

## Compose

Expose additive profiles named `local`, `web`, `realtime`, `authenticated`, and
`full-agentic`. Build or reference compatible pinned images for Flint Forge, Flint
Realtime Fabric, Flint Gate, and Ory Kratos. Do not place credentials in the YAML; load
them from ignored secret files or an external secret manager.

## Kubernetes

Kustomize is the primary application packaging format. Reuse existing upstream Helm
charts only where they remain the component's maintained interface. Provide startup,
readiness, and liveness probes; NetworkPolicies; Pod security settings; persistent volume
claims where required; and Secret references rather than inline values.

## Required proof

Render every selected profile, start it from a clean checkout, wait for readiness, run a
conversation through the public Axum boundary, observe streamed AG-UI events, restart the
stateful services, and prove the conversation remains available.
