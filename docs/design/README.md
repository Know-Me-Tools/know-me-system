# KnowMe IAM portal — design prototype

Runnable HTML prototypes for the identity portal that fronts the Ory Kratos
instance in this system, plus its admin console. Copied from the OpenDesign
project on 2026-09-19.

Open any file directly in a browser — no build step, no server, no dependencies.

## Files

| File | What it is |
|---|---|
| `index.html` | Identity + portal. Login, second factor (AAL2), registration, recovery, reset password, verification, invite acceptance, the signed-in app launcher, personal settings, and the flow map. |
| `admin.html` | The admin console. 14 views: dashboard, users, invitations, sessions, applications, roles, audit log, five detail surfaces, settings, and the flow map. |
| `registration.html` | Standalone Kratos-facing account creation, as a single page with no router. |
| `screenshots/` | Two captures from the build sessions. |

`index.html` is the canonical entry. `admin.html` is the console and is
self-contained — it deliberately carries no dependency on `index.html`.

## Start with the flow map

Both `index.html` and `admin.html` have a **Flow map** view (`#flow` and
`#admin-flow`) that enumerates every screen, view and state, shows how they
connect, and links into each one. It is the fastest way to see the whole
surface area, and it marks known gaps rather than hiding them.

A `FLOW MAP` control is pinned bottom-left on every other screen to get back.

## Reaching states that normally need a trigger

Error paths, modals and empty states have their own routes so they can be
inspected directly:

- `index.html#login-error` — failed sign-in
- `index.html#reset-invalid` — expired recovery link
- `index.html#portal-empty` — launcher with no applications granted
- `admin.html#modal-delete-role` — delete confirmation, seeded with a real role
- `admin.html#admin-users-empty` — users table empty state

These are additive: the production routes carry no state descriptor and behave
normally.

## Demo credentials

Nothing is validated against a backend. To walk the sign-in journey:

- Any email, password `wrong` → the sign-in failure state
- Any email, any other password → proceeds to the second factor
- TOTP `000000` → the rejected-code state
- Any other 6 digits → signs in to the portal

`avery.chen@know-me.tools` already exists in the demo data, so registering it
reaches the duplicate-account error.

## What maps onto the real backend, and what does not

The data shapes mirror `IamApplication`, `IamInvitation`, `IamAuditEvent` and
the Kratos identity/session fields, so those should port directly.

Three things are **design proposals, not confirmed decisions** — they were
invented during prototyping and need to be reconciled with the real system:

1. **The role model.** `Admin` and `User` as built-in defaults, plus custom
   roles scoped to a single application. Kratos keeps role in identity traits
   or in Keto relation tuples; which one decides whether the role selector
   writes a field or a relation.

2. **The permission vocabulary.** Sixteen named permissions across six groups
   (identity, applications, roles, sessions, audit, self-service). If
   authorization is expressed as Keto tuples, the editor's shape needs
   rethinking — not the screen around it.

3. **The application visibility matrix.** Per-role access to each application
   in the suite. The reference `IamApplication` model carries only a
   `userFacing` boolean, so this whole surface is net-new.

Identity **state** is not in that list: it was corrected to Kratos's real
`active` / `inactive`, with "awaiting first sign-in" derived by joining the
invitation records rather than stored as a third state.

## Known limitations

- Application names (Memory Vault, Signal Desk, Reporting Console) are
  placeholders, not products.
- Avatar uploads are client-side data URIs — they survive navigation but not a
  reload, since a static prototype has nowhere to persist a file.
- The demo dataset is deliberately small (7 users, 5 sessions, 8 audit events),
  so dashboard counts read as small numbers. Every figure is derived from those
  arrays rather than hard-coded, so nothing can drift from the section it
  summarizes.
- One flow-map node is intentionally a dashed gap: **Launch target**, the
  application being launched into, which is outside this prototype's scope.

## Branding

Tokens come from the KnowMe brand guide: ember `#FF6A3D` (dark) / `#E04E28`
(light) on cool near-black `#0B0F14`, Space Grotesk for display, Inter for UI,
Roboto for body, JetBrains Mono for identity IDs and session data. Light and
dark are both drawn from the brand's own ramps. The ember accent marks one
element per viewport, per the brand's restraint principle.
