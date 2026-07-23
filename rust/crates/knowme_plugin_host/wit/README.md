# KnowMe plugin WIT contracts

This directory holds the WIT component contracts for KnowMe plugins.

- `knowme-plugin.wit` — **deprecated** `knowme:plugin@0.1.0` (kept for
  reference; see `MIGRATION.md`). Deleted in `plugin-manifest-v2`.
- `v1/` — the canonical `knowme:plugin@1.0.0` package: one flat directory,
  one package id, seven worlds (`hook`, `contributor`, `agent`, `provider`,
  `app-shell`, `service`, `workflow`) over a shared host-import surface.

Validate the 1.0.0 package with:

```bash
wasm-tools component wit rust/crates/knowme_plugin_host/wit/v1/
```

## Versioning and compatibility policy

Adopted from librefang's `wit/README` discipline (design D6).

1. **Additive-only within a major version.** Within 1.x you may add new
   functions, new interfaces, new worlds, and new variants on `host-error`
   (and other enums/variants). Anything already published stays exactly as
   it is.
2. **Renames and removals require a major bump.** Renaming or removing any
   interface, function, type, or variant requires a 2.0.0 package and a
   deprecation window in which both majors are accepted by the host.
3. **Version ranges, not exact pins.** Guest manifests declare a
   compatibility range (e.g. `knowme:plugin@^1.0.0`); the host resolves the
   range at load time and refuses manifests that exact-pin a single
   version.

The `agent`, `provider`, `service`, and `workflow` worlds are annotated
`// unstable: refined by <change-id>` — their signatures may be *extended*
by the named follow-up changes under rule 1 without a version bump.
