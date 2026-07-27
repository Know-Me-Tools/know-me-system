# REFLECTION: model-runtime-per-device

Date: 2026-07-27
Status: **CLOSED** — 5/5, reconciled from evidence.

## Goal achievement

| Goal | Verdict |
|---|---|
| Ship Android local text-agent inference through LiteRT-LM Gemma 4 E2B, CPU as the correctness baseline | **MET** — `gemma-4-e2b-it-litertlm`, `modelLoadMs: 228`, `generateMs: 7726`, thermal 0 on the SM-S936U |
| Device-aware manifests, resumable downloads, integrity checks, runtime gate diagnostics, safe memory/thermal fallback | **MET, with one stated gap** — backend/thermal/fallback certified on device; memory-preflight downgrade is unit-test-only and the report says so |
| Raise local generation limits and verify tool-capable model plumbing | **MET** — permitted tool call passed on device, attempt 1 of 3 |
| Ship Qwen3.5-9B with matching mmproj **vision** on iPhone using Metal | **DEFERRED** — see below |

### The deferral, stated plainly

Image **recognition** moves to a future phase by operator decision (2026-07-27).
It was also not achievable as written: `chat.rs:292` gates the multimodal path
on `!cfg!(target_os = "android")` deliberately, because Android's Gemma 4 E2B
lane is text-only and never receives image tensors. "Verify image understanding
on both physical phones" could not be satisfied by any amount of testing.

Image **rendering** — a separate concern — is complete and certified here. The
two share a word, not a dependency: `chat-media-rendering` mentions vision zero
times.

## Delivered

**31 commits.** Two workstreams.

### Media rendering (`chat-media-rendering`, 37/37)

A PNG rendered as a grey document chip on every surface. The cause was not a
renderer bug: `ContentBlock::Image` was defined in `gen_ui_types` and decoded by
both UIs but **constructed by nothing**, while five sites independently
hardcoded the kind `attachment` regardless of mime. A renderer fix alone could
never have worked — the block never arrived.

C1 durable storage · C2 Rust decides kind/mime/dimensions once · C3 Flutter
local media behind a conditional import · C4 React image/video + the dead
composer thumbnail · C5 Flutter save/share · C6 React/Tauri save/share · C7
OpenGraph thumbnail · C8 attachment lifecycle.

### Device proof (`qwen35-device-proof` 5/5, `qwen35-local-tools` 6/6)

Android: backend, throughput, fallback, cancellation, four stopped-turn intents,
multi-turn, permitted tool call, and durable media surviving a real cache
eviction. iOS: release build installed and launched on the iPhone 17 Pro Max,
with media save and share operator-verified. Desktop: release bundle built with
the asset protocol and CSP compiled in.

## Artifact quality

**No artifact-refiner logs exist** — `.refiner/artifacts/` was never created, so
no pass rate is computable. Stated rather than substituted with test counts,
which measure something else. Verification rested on: 88 Rust, 157 React and 50
Flutter tests, `tsc`/`flutter analyze` clean, a desktop release bundle, a real
Flutter web build, and five device certification reports.

## The finding that justifies the effort

**The device caught a bug that 157 React, 88 Rust and 50 Flutter tests all
passed straight through.**

`_encodeBlock`'s image arm destructured only `(url, dataBase64, mime)`, silently
dropping `path`, `alt`, `width`, `height` on save. An attached image rendered
perfectly in-session and returned *"Image unavailable"* after a restart. The
bytes were safe on disk; the block pointing at them was not — defeating the
entire premise of C1.

No test covered persist → hydrate for images. **The gap was not a weak assertion
but a missing seam**: state crossing a persistence boundary. Round-trip tests
belong wherever a type gains fields. Fixed in `388f6b2` with three regression
tests.

## Corrections to my own work

1. **My plan asserted `ContentBlock::Image` required `alt`/`width`/`height`.** It
   had none. Written from memory of the contract rather than the contract.
2. **The browser found what unit tests could not, twice.** A stored path
   resolved to `''` on web, emitting `<img src="">`. And a dev server's 200 +
   HTML fallback made a *missing* file report `saved`, writing an HTML error
   page as a PNG.
3. **My own C1 had a real flaw.** One process-wide storage root meant one test's
   orphan sweep deleted another's files.
4. **My certification gate would have passed without purging anything.**
   `run-as` refuses a release build, so the cache `rm` silently no-opped.
   `pm clear --cache-only` hangs on this Samsung — measured twice.
   `pm trim-caches` works.
5. **My iOS gate conflated "unobserved" with "failed."** `devicectl console`
   captured 0 bytes, which would have read as a failed check.
6. **I deviated from my own plan on C6 and stand by it.** It called for two
   Tauri plugins — five registration points each, the trap that broke
   `12134b6`. Unnecessary: the asset scheme was already fetchable.

Every one of these was found by *running* the thing, not by reasoning about it.

## Technical debt

- **`run-as` is blocked on release builds**, so app-storage inspection cannot be
  asserted directly; the cache-purge gate carries the proof instead.
- **Pre-fix conversations stay broken.** Blocks saved before `388f6b2` have no
  path and cannot be resurrected. Deliberate — backfilling from paths that may
  no longer exist would be a lie.
- **iOS has no scriptable UI dump** outside an XCUITest target, so its evidence
  is operator-observed and recorded as such.

## Pre-existing issues found, NOT fixed (out of scope)

- **Three `settings_workspace_test` tests fail.** Confirmed pre-existing by
  stashing this session's work and re-running: identical failures without it.
- **Four of five bottom-nav tabs have no accessible name.** Only "Home" is
  labelled; a screen reader announces nothing for Chat, Memory or Models. A
  real a11y defect deserving its own change.

## Recommended next focus

1. **`embedded-uar-offline-agents`** — `embed-uar-mobile-offline` has 14 open
   tasks including airplane-mode certification on both platforms. Deliberately
   not folded into this phase.
2. **The deferred vision phase** — decide scope first: narrow to iOS, or ship a
   vision-capable Android model.
3. **Label the bottom-nav tabs.**
4. **Fix the three failing settings tests.**
