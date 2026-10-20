# KnowMe Praxis 1.1 publication status

Prepared from commit `f18bbbc03b27fec9ced0e875c1ed9326ea179224`. This candidate adds WEB-01–14, revision history, responsive screenshots and complete image/social metadata. All 17 chapters, 45 retained source records and 811 coverage mappings remain intact. The original `publication-receipt.json` continues to describe edition 1.0 only.

## External blocker

`POST https://rpc.prometheusags.ai/api/v0/version` returned HTTP 401 on this Mac. The committed forge configuration has no authentication setting; no IPFS/forge credential is configured in the process environment, `.netrc` is absent, and the local IPFS configuration contains no remote pinning service. No upload has been attempted with guessed credentials. No services or persistent access were changed. An existing authorized credential for this endpoint is required before publishing.

## Image first

`site/screenshots/revision-1.1-og.png` is the newly rendered 2400×1260 PNG. Its locally computed CIDv1 using Kubo's default raw-leaf importer is `bafybeic6mr557ji36o53xo5bc6uw7jpxckzcdqo4kjzis3zimwjchem6bq`. This CID is **predicted, not an upload or pin receipt**. `source/social.json` embeds its absolute gateway URL so the candidate is ready to publish, but its public response/MIME has not been verified. Upload this exact asset first using the authorized endpoint and verify returned CID, HTTP 200, PNG MIME, decoded dimensions and SHA256 before publishing the page. If importer settings change its CID, update `source/social.json`, rebuild and refresh `candidate-manifest.json` first.

The screenshot is available in ChatGPT Library as `libfile_2cbed1e4ec50819193ba34efb5fdf89c` (`knowme-praxis-1.1-og.png`). No Library URL is substituted for the required public IPFS image.

## Canonical strategy

There is no verified stable mutable canonical route for this edition. Edition 1.0's immutable CID is an earlier version, so it is not used as 1.1's canonical. The candidate deliberately omits `rel=canonical`, `og:url` and `twitter:url`; it supplies title, description, article type/site name/locale/author/date, image URL/secure URL/type/dimensions/alt and Twitter large-image metadata. This avoids claiming the final immutable CID can name itself.

After the image is public, upload the exact candidate and record its returned URL externally in a new edition 1.1 receipt and this README. If an absolute canonical is required before a stable mutable route exists, use a documented two-pass publication: the final pass points at the verified first-pass edition URL (which lacks only canonical metadata). Keep both CIDs and explain that relationship. Do not iterate CIDs trying to obtain self-reference or claim the first pass equals the final CID.

## Verification

The pinned forge reproduced all 137 original files exactly before edits. The candidate rebuild matches all candidate files in `candidate-manifest.json`. The headless forge check reported no desktop/phone console errors or page overflow, and compact mode switched correctly. Its overall result is failed because ten pre-existing external source links returned 403/429/522; those historical links were preserved. Runtime WebMCP conformance is specified, not implemented or certified by document rendering.

No UI typing/clicking, UAR edits, existing session operations, or changes to other worktrees were used. A new scratch clone and headless rendering environment were used in the task workspace.
