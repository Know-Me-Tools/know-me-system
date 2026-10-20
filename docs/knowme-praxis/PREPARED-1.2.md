# KnowMe Praxis 1.2 — tested correction, not published

Base: `e57624fc8539e5ebd09dde79b96943cbdbc83467`, fetched into an isolated MacBook worktree. The public 1.1 index was fetched without authentication and matched this checkpoint byte-for-byte. No UAR interaction, credential access, key operation, upload, DNS change or server change was performed for this correction.

## Corrections

- Corrected `--surface2` to `--surface-2`; retained prose explicitly uses the body foreground and surface background rather than inheriting code-block colors. Dark summaries have an explicit readable foreground.
- Subtle text now uses `#566372` in light mode and `#A7B0BC` in dark mode. Audit links use the theme's readable soft foreground. The reusable amber foreground is `#805211` on its existing soft background.
- Standalone records have readable dark-mode source notes, visible keyboard focus and wrapping for long technical identifiers.
- Actual overflowing tables/content receive `tabindex="0"` at runtime, retain native semantics and receive a descriptive accessible name. Focusability updates after disclosure, resizing and HTMX swaps. Non-overflowing containers are not added to the tab order. Inset focus outlines keep summary/scroll-region focus visible inside clipped parents.
- Release metadata now requires explicit `PUBLIC_CANONICAL_URL` and `PUBLIC_OG_IMAGE_URL`. A previous immutable CID cannot serve as the canonical URL. The old 1.1 social JSON is historical, not build input. Author attribution remains in `meta name="author"`; no unverified profile URL is placed in `article:author`.

## Measured verification

`checks-1.2/summary.json` records the compact results and `checks-1.2/accessibility.json` retains full axe evidence. Axe-core 4.14.0 was retrieved from the official npm registry and its tarball integrity checked. Headless Chromium tested all 811 source sections expanded at 1440 px and 390 px, in light and dark themes. Representative standalone HTML/Markdown source records, home and coverage pages were also tested in both themes.

- Zero axe violations under WCAG A/AA tags and the scrollable-region-focusable rule in all 12 tested page/theme cases.
- Zero computed text-contrast failures across 16,074 text nodes per desktop main view and 16,041 per phone main view.
- Retained prose and summaries: 16.60:1 light / 14.58:1 dark. Audit links: 7.31:1 / 10.08:1. Light contents/source headings: 6.13:1. Light footer: 5.73:1.
- Zero unfocusable scroll regions. Actual Tab and ArrowRight checks passed for 3 scrolling regions per desktop view and 34 per phone view. All 69 summary focus indicators per view were contained inside their clipping parent.
- No page-level horizontal overflow in the tested views. The detected standalone-record overflow was corrected before the final confirmation run.
- Four offline metadata contract tests passed, including consistent URL/image tags, no invented draft identity, rejection of stale-CID canonicals and safe attribute escaping. Test fixture URLs were not inserted into the candidate.
- All 17 chapter text trees are unchanged. The 811 retained sections, source assembly JSON, complete Markdown, coverage matrices and 45 technical text records are preserved. Fifty technical files are byte-identical to 1.1.
- The new PNG preview is 2400 × 1260. Its SHA-256 is `b8e975bde9bbddfd2e4a97bcc903dbcb4fb26801b53f128e0a130f6067423b5d`.

This is not full WCAG certification. Screen-reader user testing and native-device testing were not performed. Axe reported incomplete contrast evaluations involving image/pseudo-element backgrounds, non-text glyphs and partially clipped table cells; the report preserves those reasons. Computed foreground/background checks and visual inspection supplement them, not an assertion that automated tooling proves every criterion.

## Package and build

The prepared package is `dist/knowme-praxis/`; do not upload the `site/` input tree. Current output is an explicitly unpublished draft: no fake canonical identity or stale preview image is emitted. The corrected PNG to upload first is `checks-1.2/screenshots/og-1.2.png` (also included in the package screenshots).

```sh
python3 build.py --forge-root /path/to/htmx-doc-forge --draft
```

A release build fails before building if either public identity is missing. After the parent supplies the actual approved identities:

```sh
PUBLIC_CANONICAL_URL='<actual stable URL supplied by parent>' \
PUBLIC_OG_IMAGE_URL='<actual verified preview URL supplied by parent>' \
python3 build.py --forge-root /path/to/htmx-doc-forge
```

The build writes `candidate-manifest-1.2.json` and never uploads or reads credentials. Verify static head tags and the final byte manifest after this metadata-only rebuild; the corrected body and CSS must match the tested draft.

## Exact proposed publication actions — parent only, pending approval

The proposed stable identity is a document-specific IPNS name. No existing appropriate identity was established; do not use a fabricated key ID, another application's name or a stale previous-CID canonical. The parent proposed obtaining explicit approval for exactly four authenticated operations at `https://rpc.prometheusags.ai`:

1. Generate the explicitly approved document-specific persistent IPNS key. Record only the returned public key identifier; do not export private key material.
2. Upload the corrected PNG once. Verify its public URL with unauthenticated GET: MIME `image/png`, dimensions 2400 × 1260 and the SHA-256 above.
3. Set `PUBLIC_CANONICAL_URL` to `https://ipfs.prometheusags.ai/ipns/<actual returned public identifier>/index.html`, set `PUBLIC_OG_IMAGE_URL` to that verified image URL, and build/inspect the release package locally. Upload the exact resulting directory once. Record the actual returned CID and compare public HTML/assets with the candidate manifest using unauthenticated GETs.
4. Publish only that document-specific IPNS name to the returned directory CID. Verify the stable public URL resolves to the expected HTML and static canonical/og:url/twitter:url fields. Report resolution delays honestly rather than claiming an unverified canonical endpoint works.

Do not reuse the earlier legacy publisher: it performs additional authenticated version/pin queries outside this four-operation budget. No extra key listing, pin-state query, key export, DNS mutation, deployment change or credential destination is proposed. Request authorization before any different authenticated action. A publication receipt must distinguish requested pinning from independently verified pin state.

No publication action has occurred in this correction worktree. Parent owns credentials, approval and publication.

## Approved identity and release handoff

Parent reported user approval at 12:11 and successful key creation (operation 1 of 4). Canonical is now `https://ipfs.prometheusags.ai/ipns/k51qzi5uqu5dl4048r1wozopco1c2b0vzfx0kj3rbs0g9rhadfjs8msvi4ll3t/index.html`. The name pointer has not yet been published; resolution is unverified. Local release metadata uses the mathematically predicted single-raw-block PNG CID. Parent must compare the actual image-upload CID and verify public PNG bytes before uploading the candidate.

The metadata-only rebuild changed index.html alone; body DOM is exactly unchanged and the other 157 files are byte-identical. Four metadata unit tests and four additional rendered viewport/theme checks passed. All four new screenshots are byte-identical to the accessibility-tested screenshots. The full prior 12-case accessibility evidence therefore remains applicable to the unchanged body/CSS/assets. The transfer ZIP contains the exact manifest files at archive root. Child has made zero authenticated RPC calls.

See `checks-1.2/release-metadata.json` for exact importer settings, canonical, expected image CID, image Library ID, hashes and checks. Historical draft status above describes the previous stage. No claim of public availability is made.
