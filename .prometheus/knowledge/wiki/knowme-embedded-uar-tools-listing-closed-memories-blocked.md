---
type: Reference
id: knowme-embedded-uar-tools-listing-closed-memories-blocked
title: KnowMe embedded UAR tools listing closed, memories blocked
tags:
- knowme
- embedded-uar-offline-agents
- uar
- tool-registry
- a2ui
- memory-system
- offline-agents
links:
- knowme-admin-tools-and-a2ui-listing-gaps-for-embedded-uar
sources:
- stdin
- manual:KnowMe/embedded-uar-offline-agents
timestamp: 2026-07-28T05:19:43.385780+00:00
created_at: 2026-07-28T05:19:43.385780+00:00
updated_at: 2026-07-28T05:19:43.385780+00:00
revision: 0
---

## Phase context

- Project: `KnowMe`
- Phase: `embedded-uar-offline-agents`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured at: `2026-07-28T05:16:38Z`
- Position: `embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`

## Current implementation state

- UAR commit: `2e566f5`.
- KnowMe wiring compiled cleanly and its test suite is running.
- Dependency graph is large; some logs buffer until process exit.
- This session follows the prior gap assessment in [KnowMe admin tools and A2UI listing gaps for embedded UAR](/knowme-admin-tools-and-a2ui-listing-gaps-for-embedded-uar.md).

## Item 3.2 gap outcomes

| Gap | Outcome |
|---|---|
| `tools` | Closed. Served from both registries; the `unavailable` marker was removed because it would now be false. |
| `A2UI` | Schema listing closed. Actions FFI remains absent and is explicitly recorded as missing. |
| `memories` | Still blocked at the storage layer; the `unavailable` marker remains correct. |

## Why `memories` remains unavailable

`PersistenceLayer` exposes only query-based retrieval:

```rust
search_memory(query_vec, …)
```

There is no storage-layer listing/enumeration operation. Enumerating memories would require inventing an embedding query vector solely to drive search, which would not be a real list operation. Therefore:

- The missing memory list is a genuine storage-layer capability limit, not unfinished UI/control-plane wiring.
- Removing the marker would cause the UI to claim “none configured” for data it cannot actually enumerate.
- The same marker discipline applies in both directions:
  - Remove `unavailable` immediately when it becomes false, as with `tools`.
  - Keep `unavailable` when the system still cannot answer truthfully, as with `memories`.

## Tool listing design decision

Tool listing preserves `source` instead of flattening all tools into one merged array.

Rationale:

- Native tools are in-process and require no network or child process.
- Native tools continue working on a phone with both radios off.
- MCP tools require a reachable server.
- Clients need to distinguish native vs. MCP sources to explain why some tools remain available offline while others do not.
- Preserving `source` is central to the embedded UAR offline-agent behavior and user-facing diagnostics.

## Next work

- Read the `gen_ui_agent` result and commit the KnowMe wiring.
- Then either:
  - implement A2UI actions FFI, or
  - rebuild the APK and re-run offline Android certification against a build carrying S1–S6.

# Citations

1. stdin
2. manual:KnowMe/embedded-uar-offline-agents