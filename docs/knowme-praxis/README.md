# KnowMe Praxis — The Sovereign Agentic Workspace

Complete architecture and functional specification covering containers, home/dashboard/composer/suggestions, A2UI mini applications, optional private dots, native Flutter, Rust FFI, portable component plugins, context/model routing and LiveKit human/agent realtime voice/video.

**Edition 1.0 published and verified:** [Read KnowMe Praxis](https://ipfs.prometheusags.ai/ipfs/bafybeib5kuf3t4wqiadyefm36466qhd6d6q4o42ddclky53vhnuvqpocym/index.html).

**Edition 1.1 published:** [Read the WebMCP revision](https://ipfs.prometheusags.ai/ipfs/bafybeibmabyqe2f4jsvtpmjnugsxuwm3v2v3zzbd4nn6gqcaeuad4ovjei/index.html). It adds first-class proposed WebMCP requirements across ten existing chapters, fourteen supplemental coverage entries, G17 browser conformance fixtures and complete screenshot/social metadata. Root verified the public preview image and index against the tested candidate. See [PUBLICATION-1.1.md](PUBLICATION-1.1.md) and `publication-receipt-1.1.json` for evidence and verification limits; full 147-file public hash verification is pending independently with root. `candidate-manifest.json` remains the prepared artifact's byte manifest, not a pin-state claim.

This is a documentation checkpoint, not an application release or certification of proposed capabilities. Code-observed findings, retained source requirements and proposed integration contracts are distinguished in the document.

## Contents

- `site/`: the unchanged published edition 1.1 package, preserving the 45 original source records and all 811 source mappings, plus fourteen supplemental WEB clauses and new responsive screenshots. The original publication's exact hashes remain in `publication-receipt.json`.
- `source/`: revised prose, detailed research additions, frozen chapter assembly data and the document module. Frozen chapter data retains the approved technical material and makes the delivered publication reproducible without private local repositories.
- `brand/`: KnowMe document brand assets.
- `publication-receipt.json`: historical edition 1.0 CID, recursive-pin confirmation and all 137 public file hash checks.
- `publication-receipt-1.1.json`: actual edition 1.1 page/image CIDs, root-reported public verification, and explicit pin/whole-package verification limits.
- `site/verification/`: successful coverage reconstruction and six viewport/theme render checks, including fully expanded technical records and HTMX fragment loading.

The public technical-record scope was explicitly approved on 20 October 2026 before publication. Credentials, private voice/personality substrate, unapproved raw source snapshots, unrelated application changes and redundant archives are excluded.

## Reproduce the published package

Use Python 3.11 or newer and a local checkout of [htmx-doc-forge](https://github.com/Prometheus-AGS/htmx-doc-forge) at commit `42336a3eee61261ab3b3cd896fa41dfd9bc23aa7`. Its existing Python dependencies must be installed. No new dependency is added to the KnowMe application.

```sh
python3 docs/knowme-praxis/build.py --forge-root /path/to/htmx-doc-forge
```

The command writes ignored `dist/` outputs, enforces the pinned forge commit and verifies the full file inventory and hashes against `candidate-manifest.json` (falling back to the original receipt for edition 1.0). It uses the audited icons already present in the approved package, so assembly requires neither a browser nor network access. `source/body.json` is the full-depth chapter assembly source; changing prose requires updating its corresponding chapter before rebuilding. Coverage records and source details remain linked to their revised owning chapters. The primary research report and source hashes are retained in `source/webmcp-research-2026-10-20.md`.

## Publication and resume

Editions 1.0 and 1.1 are published at distinct immutable CIDs. Do not republish merely to resume work. Edition 1.1 used exactly two explicitly approved authenticated uploads, image first; both requested pinning. No independent remote pin/list inspection was performed. No credential file or automatic publishing hook is included here.

## Standing authoring preference

User instruction, 20 October 2026: every new HTMX or HTML document should include a rendered screenshot uploaded to IPFS and complete OpenGraph/Twitter preview metadata. Publish the preview image first, verify its public MIME/dimensions, then embed its absolute URL. Document the canonical strategy and retain publication evidence. This preference is recorded here as project memory. Edition 1.1 completed image-first publication and public preview verification through root.

This branch only adds `docs/knowme-praxis`. It does not change the application, the active UAR work, deployment settings or the original source repository's untracked files. No merge or deployment is part of the checkpoint. All local rendering and publishing processes had exited before handoff.
