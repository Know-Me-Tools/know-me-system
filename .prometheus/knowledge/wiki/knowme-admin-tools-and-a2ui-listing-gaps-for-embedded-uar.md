---
type: Reference
id: knowme-admin-tools-and-a2ui-listing-gaps-for-embedded-uar
title: KnowMe admin tools and A2UI listing gaps for embedded UAR
tags:
- knowme
- embedded-uar-offline-agents
- uar-admin
- tool-registry
- a2ui
- memory-system
- offline-agents
links:
- knowme-s6-web-fetch-ssrf-hardening-with-fetch-guard
sources:
- stdin
timestamp: 2026-07-28T04:55:11.688189+00:00
created_at: 2026-07-28T04:55:11.688189+00:00
updated_at: 2026-07-28T04:55:11.688189+00:00
revision: 0
---

## Phase context

- Project: `KnowMe`
- Phase: `embedded-uar-offline-agents`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured at: `2026-07-28T04:50:12Z`
- Position: `embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`

## Gap assessment for item 3.2

| Gap | Status | Reason |
|---|---|---|
| `tools` | Implementable | Tool registries are already available on the kernel, and `openai_tools_json()` exists. The embedded branch was returning `[]` despite having access to the data. |
| `A2UI` | Implementable | `A2uiRegistry::list()` exists and returns `Vec<ArtifactSchema>`. |
| `memories` | Genuinely blocked | `PersistenceLayer` has no list/enumeration method. It only exposes `search_memory(query_vec, …)`, which requires an embedding query vector. Enumerating memories would require inventing a query, so the `unavailable` marker should remain. |

This continues the remaining 3.2 work tracked after earlier embedded-UAR hardening; see [KnowMe S6 web_fetch SSRF hardening with fetch_guard](/knowme-s6-web-fetch-ssrf-hardening-with-fetch-guard.md) for prior phase context and verified state.

## Tools admin listing design

- Add `admin::tools` for embedded UAR/admin inspection.
- Include both tool backends:
  - **Native tools**: in-process tools that work without network access and therefore remain usable on a phone/offline client.
  - **MCP tools**: tools that require a reachable MCP server and may be unavailable offline.
- Preserve a `source` field instead of flattening all tools into an indistinguishable merged list.

Rationale: offline clients need to show that native tools still work while MCP tools do not. A merged tool list without `source` cannot express that distinction.

## A2UI admin listing design

- Add `admin::a2ui` as a read-only schema listing endpoint.
- Use `A2uiRegistry::list()` to expose registered schemas as `Vec<ArtifactSchema>`.
- Do **not** support schema registration through admin APIs.

Rationale:

- Device dumps in this suite have not shown an A2UI surface because mobile builds have not emitted artifacts.
- Listing registered schemas distinguishes:
  - renderer/schema support is absent, from
  - no artifact was produced during this run.
- Schema registration changes rendering for every future artifact, so it belongs in build-time composition, not in a runtime/admin call that could alter rendering mid-session.

## Implementation note

`A2uiRegistry::with_builtins()` is synchronous and returns `Arc<Self>`. It is not async and does not return a bare registry.

## Current progress marker

- Changes noted: `0/1`
- Last recorded work: added `admin::tools` with native+MCP merged while preserving `source`; added read-only `admin::a2ui` schema listing; confirmed memories remain blocked by storage-layer API limitations.
- Next steps:
  1. Read the `admin::` test result.
  2. Wire `tools` and `a2ui` through the SDK and KnowMe `control_plane`.
  3. Drop the `tools` marker when verified.
  4. Keep `memories` honestly marked unavailable until `PersistenceLayer` supports enumeration.

# Citations

1. stdin