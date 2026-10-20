# KnowMe Praxis — The Sovereign Agentic Workspace

Complete architecture and functional specification covering containers, home/dashboard/composer/suggestions, A2UI mini applications, optional private dots, native Flutter, Rust FFI, portable component plugins, context/model routing and LiveKit human/agent realtime voice/video.

**Edition 1.0 published and verified:** [Read KnowMe Praxis](https://ipfs.prometheusags.ai/ipfs/bafybeib5kuf3t4wqiadyefm36466qhd6d6q4o42ddclky53vhnuvqpocym/index.html).

**Edition 1.1 published:** [Read the WebMCP revision](https://ipfs.prometheusags.ai/ipfs/bafybeibmabyqe2f4jsvtpmjnugsxuwm3v2v3zzbd4nn6gqcaeuad4ovjei/index.html). It adds first-class proposed WebMCP requirements across ten existing chapters, fourteen supplemental coverage entries, G17 browser conformance fixtures and screenshot/social image metadata. A subsequent audit identified missing canonical/og:url identity and contrast/focus defects; edition 1.2 is being prepared to correct them. Root verified the public preview image and index against the tested candidate. See [PUBLICATION-1.1.md](PUBLICATION-1.1.md) and `publication-receipt-1.1.json` for evidence and verification limits; the local pre-upload manifest verified all 147 files, while optional bulk public verification was blocked by network policy mid-check and was not retried. `candidate-manifest.json` remains the prepared artifact's byte manifest, not a pin-state claim.

This is a documentation checkpoint, not an application release or certification of proposed capabilities. Code-observed findings, retained source requirements and proposed integration contracts are distinguished in the document.

## Contents

- `site/`: the historical edition 1.1 HTML and current reusable asset/input tree, preserving the 45 original source records and all 811 source mappings, plus fourteen supplemental WEB clauses and new responsive screenshots. The original publication's exact hashes remain in `publication-receipt.json`.
- `source/`: revised prose, detailed research additions, frozen chapter assembly data and the document module. Frozen chapter data retains the approved technical material and makes the delivered publication reproducible without private local repositories.
- `brand/`: KnowMe document brand assets.
- `publication-receipt.json`: historical edition 1.0 CID, recursive-pin confirmation and all 137 public file hash checks.
- `publication-receipt-1.1.json`: actual edition 1.1 page/image CIDs, root-reported public verification, and explicit pin/whole-package verification limits.
- `site/verification/`: successful coverage reconstruction and six viewport/theme render checks, including fully expanded technical records and HTMX fragment loading.

The public technical-record scope was explicitly approved on 20 October 2026 before publication. Credentials, private voice/personality substrate, unapproved raw source snapshots, unrelated application changes and redundant archives are excluded.

## Build the current candidate

Use Python 3.11 or newer and a local checkout of [htmx-doc-forge](https://github.com/Prometheus-AGS/htmx-doc-forge) at commit `42336a3eee61261ab3b3cd896fa41dfd9bc23aa7`. Its existing Python dependencies must be installed. No new dependency is added to the KnowMe application.

```sh
python3 docs/knowme-praxis/build.py --forge-root /path/to/htmx-doc-forge --draft
```

The command writes ignored `dist/` outputs, enforces the pinned forge commit and writes the full file inventory and hashes to `candidate-manifest-1.2.json`. Historical manifests/receipts preserve earlier edition hashes. It uses the audited icons already present in the approved package, so assembly requires neither a browser nor network access. `source/body.json` is the full-depth chapter assembly source; changing prose requires updating its corresponding chapter before rebuilding. Coverage records and source details remain linked to their revised owning chapters. The primary research report and source hashes are retained in `source/webmcp-research-2026-10-20.md`.

## Publication and resume

Editions 1.0 and 1.1 are published at distinct immutable CIDs. Do not republish merely to resume work. Edition 1.1 used exactly two explicitly approved authenticated uploads, image first; both requested pinning. No independent remote pin/list inspection was performed. No credential file or automatic publishing hook is included here.

## Standing authoring preference

User instruction, 20 October 2026: every new HTMX or HTML document should include a rendered screenshot uploaded to IPFS and complete OpenGraph/Twitter preview metadata. Publish the preview image first, verify its public MIME/dimensions, then embed its absolute URL. Document the canonical strategy and retain publication evidence. This preference is recorded here as project memory. Edition 1.1 completed image-first publication and public preview verification through root.

This branch only adds `docs/knowme-praxis`. It does not change the application, the active UAR work, deployment settings or the original source repository's untracked files. No merge or deployment is part of the checkpoint. All local rendering and publishing processes had exited before handoff.

## Edition 1.2 — tested release candidate

The isolated correction worktree starts from `e57624fc8539e5ebd09dde79b96943cbdbc83467`. Technical content, 811 source sections and the WebMCP additions are preserved. `dist/knowme-praxis/` is the prepared 1.2 package; it is **not published**. `site/` remains the asset/input tree, not the release upload directory.

For unpublished accessibility QA, use `build.py --forge-root /path/to/htmx-doc-forge --draft`. A release build requires `PUBLIC_CANONICAL_URL` and `PUBLIC_OG_IMAGE_URL` supplied after the parent's approved identity/image operations. The old social JSON is historical and is no longer used by the build. The build never creates a key, uploads, resolves credentials or changes a server. `checks-1.2/` contains measured contrast, axe and keyboard evidence plus new screenshots. See `PREPARED-1.2.md` for limitations and the proposed publication sequence.

The canonical now uses the document-specific key successfully created by the parent. `checks-1.2/release-metadata.json` contains the exact canonical and predicted image URL/importer settings. Public image verification and name publication remain parent-owned. To reproduce the final metadata, set `PUBLIC_CANONICAL_URL` and `PUBLIC_OG_IMAGE_URL` to those JSON fields and omit `--draft`. The release rebuild changed only index metadata; body content and all other 157 files are unchanged. Four additional rendered screenshots are byte-identical to the earlier tested views.

Full axe evidence is retained losslessly as `checks-1.2/accessibility.json.gz`; `summary.json` preserves measured results and limitations. Screenshots are retained once in `site/screenshots/revision-1.2-*.png`. The Library transfer manifest is `release-transfer-1.2.json`; its ZIP hash and 158 file hashes identify the prepared package, not a public-availability claim.
