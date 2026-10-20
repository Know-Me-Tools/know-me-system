# KnowMe Praxis 1.1 publication status

Prepared from commit `f18bbbc03b27fec9ced0e875c1ed9326ea179224`. This candidate adds WEB-01–14, revision history, responsive screenshots and complete image/social metadata. All 17 chapters, 45 retained source records and 811 coverage mappings remain intact. The original `publication-receipt.json` continues to describe edition 1.0 only.

## Published edition

Root completed exactly two authenticated uploads after explicit approval for each action: the standalone preview image, then the unchanged 147-file page package. Credentials were neither saved nor forwarded. No reupload or HTML change followed.

- Document: [https://ipfs.prometheusags.ai/ipfs/bafybeibmabyqe2f4jsvtpmjnugsxuwm3v2v3zzbd4nn6gqcaeuad4ovjei/index.html](https://ipfs.prometheusags.ai/ipfs/bafybeibmabyqe2f4jsvtpmjnugsxuwm3v2v3zzbd4nn6gqcaeuad4ovjei/index.html).
- Preview: [https://ipfs.prometheusags.ai/ipfs/bafybeic6mr557ji36o53xo5bc6uw7jpxckzcdqo4kjzis3zimwjchem6bq](https://ipfs.prometheusags.ai/ipfs/bafybeic6mr557ji36o53xo5bc6uw7jpxckzcdqo4kjzis3zimwjchem6bq).
- Artifact commit: `5ba6ef7ccf6db6505fa7c90aaae91f75074dbca5`.
- Safe external evidence: `publication-receipt-1.1.json`; the original receipt remains edition 1.0 history.

The earlier unauthenticated HTTP 401 on this Mac is historical. Root obtained action-time authorization and performed publication separately; this documentation task did not access credentials or repeat an upload.

## Image-first verification

Root verified the published preview's public bytes, `image/png` MIME, 2400×1260 decoded dimensions and SHA256 `049f8af0b4cb8d6aa48472ded99a8a818dd8b0110ae18e2d1c8cd6acba7f6a57`. The returned image CID matched the predicted CID embedded in `source/social.json`. Root also verified that the public index hash matches the tested candidate and its OG/Twitter image URLs resolve to this preview.

Both authenticated add calls used `pin=true`. No separate authenticated pin/list call was made, and remote pin state was not independently inspected. Full verification of all 147 public file hashes is being performed independently by root and is pending in this receipt; it must not be inferred from the index check alone.

The screenshot is available in ChatGPT Library as `libfile_2cbed1e4ec50819193ba34efb5fdf89c` (`knowme-praxis-1.1-og.png`). No Library URL is substituted for the required public IPFS image.

## Canonical strategy

There is no verified stable mutable canonical route for this edition. Edition 1.0's immutable CID is an earlier version, so it is not used as 1.1's canonical. The published HTML deliberately omits `rel=canonical`, `og:url` and `twitter:url`; it supplies title, description, article type/site name/locale/author/date, image URL/secure URL/type/dimensions/alt and Twitter large-image metadata. This avoids claiming the final immutable CID can name itself.

The actual immutable edition URL is now recorded in this external publication evidence and the README. If a future edition needs an absolute canonical before a stable mutable route exists, a documented two-pass publication can point the final pass at a verified first-pass edition URL; retain both CIDs and explain that relationship. This edition did not use that strategy and does not claim self-reference.

## Verification

The pinned forge reproduced all 137 original files exactly before edits. The candidate rebuild matches all candidate files in `candidate-manifest.json`. The headless forge check reported no desktop/phone console errors or page overflow, and compact mode switched correctly. Its overall result is failed because ten pre-existing external source links returned 403/429/522; those historical links were preserved. Runtime WebMCP conformance is specified, not implemented or certified by document rendering.

No UI typing/clicking, UAR edits, existing session operations, or changes to other worktrees were used. A new scratch clone and headless rendering environment were used in the task workspace.
