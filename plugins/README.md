# KnowMe Plugins

Third-party (and first-party) extensions built against the `knowme:plugin@0.1.0`
WIT contract. See FUNC-SPEC §08 (`docs/knowme-functional-spec.html`) for the
full trust and distribution model.

## Layout

```
plugins/
├── README.md              ← this file
└── examples/
    └── hello-knowme/      ← minimal component targeting the v0.1 contract
```

The canonical WIT contract lives with the host:
`rust/crates/knowme_plugin_host/wit/knowme-plugin.wit`. Plugins vendor a copy
(or reference it by path) at build time.

## The v0.1 surface

Host imports available to a plugin: `log`, `storage-get/set/delete`,
`uar-invoke`, `emit-event`. Plugin exports: `init`, `handle-event`, `shutdown`.
There is **no direct network access** — the only route to the network is
`uar-invoke("web-fetch", …)`, which passes through Cedar policy and the
manifest's declared allow-list.

## Building a plugin (Rust)

```bash
rustup target add wasm32-wasip2
cd plugins/examples/hello-knowme
cargo build --release --target wasm32-wasip2
```

The output component plus a `manifest.json` (see
`rust/crates/knowme_plugin_host/src/lib.rs::PluginManifest` for the schema) is
pinned to IPFS as a directory; the directory CID is what users paste into the
installer. Sign the canonical manifest JSON with your Ed25519 publisher key
(`did:key:`) — unsigned plugins install only behind an explicit
"trust unverified" toggle.

## v0.2 candidates (not yet in the contract)

`librefang-call`, `register-navigation`, and the sandboxed-iframe UI surface
are deferred until real plugins demonstrate the need, each with its own
threat-model review (`docs/assessment-2026-07-16.md`, rec #9).
