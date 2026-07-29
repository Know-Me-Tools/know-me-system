---
type: Reference
id: knowme-s6-web-fetch-ssrf-hardening-with-fetch-guard
title: KnowMe S6 web_fetch SSRF hardening with fetch_guard
tags:
- knowme
- embedded-uar-offline-agents
- web-fetch
- ssrf-hardening
- fetch-guard
- native-skill
- dns-rebinding
links:
- embedded-uar-offline-agents-session-ended-at-2026-07-28t04-00z
sources:
- stdin
- manual:KnowMe/embedded-uar-offline-agents
timestamp: 2026-07-28T04:07:52.897594+00:00
created_at: 2026-07-28T04:07:52.897594+00:00
updated_at: 2026-07-28T04:07:52.897594+00:00
revision: 0
---

## Phase context

- Project: `KnowMe`
- Phase: `embedded-uar-offline-agents`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured at: `2026-07-28T04:00:26Z`
- Position: `embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`

## Verified state before S6

| Area | Result |
|---|---:|
| KnowMe `gen_ui_agent` | `92/92 pass, 0 failed` |
| UAR `admin::` | `9/9 pass` |

- S4 and S5 are verified and committed.
- Item `3.2` is marked `[/]`, with `tools`, `memories`, and A2UI FFI still listed as genuine remaining work rather than closed.

## S6 finding: existing `web_fetch` is latent SSRF risk

S6 was reframed from a greenfield implementation to a hardening task because `web_fetch` already exists as a registered `NativeSkill`.

Observed properties of the existing implementation:

- `web_fetch_enabled` defaults to `false` and gates registration, so default deployments are not affected.
- When enabled, the risk becomes exploitable; classify as **latent**, not live-by-default.
- `allowed_domains` defaults to empty, meaning allow-everything.
- Domain matching is a string comparison on the raw URL:
  - no URL parsing
  - no DNS resolution checks
  - DNS rebinding can bypass the check
- The tool accepts `POST` with arbitrary headers and body, making it a general-purpose HTTP client that a model can direct to arbitrary destinations.

## Hardening direction: `fetch_guard`

Instead of adding a competing fetch tool, a `fetch_guard` was written beside the existing `web_fetch` implementation.

Design decision:

- Address rules are deliberately **not configurable**.
- Rationale: a configurable allowlist that an operator can widen to `*` is not an effective control when:
  - the model chooses the URL, and
  - fetched pages are untrusted input that can carry instructions.

## Compatibility contract retained

`fetch_guard` keeps the upstream `web_fetch` contract verbatim:

- `url`
- `max_length`
- `start_index`
- `raw`
- dual user-agent behavior

Binary content is routed through the document extractor via `extraction_for`, filling a gap left undefined upstream and allowing linked PDFs or other binary documents to become usable context.

## Security test coverage

Tests target bypass classes rather than only the happy path:

- cloud metadata endpoint
- carrier-grade NAT, which is routable inside many cloud environments
- IPv4 loopback disguised as IPv6: `::ffff:127.0.0.1`
- DNS rebinding case
- host with both one public and one private DNS record
- disallowed scheme: `file://`
- ordinary public address acceptance, to prove the guard is not merely a blanket-deny implementation

## Pending implementation work

Next steps recorded for S6:

1. Read the `fetch_guard` test result.
2. Wire the guard into `web_fetch::execute`.
3. Enforce resolve-then-check semantics.
4. Re-check the guard on every redirect.
5. Add agent-level and conversation-level enable flags on top of the existing system-level `web_fetch_enabled` gate.

This record contains concrete verification and implementation findings beyond generic completion markers such as [embedded-UAR offline-agents session ended at 2026-07-28T04:00Z](/embedded-uar-offline-agents-session-ended-at-2026-07-28t04-00z.md).

# Citations

1. [1] stdin
2. [2] manual:KnowMe/embedded-uar-offline-agents