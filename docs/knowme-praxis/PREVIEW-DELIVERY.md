# Preview delivery correction — published

The existing public preview PNG was fetched again without authentication and matched the saved image byte-for-byte: 272805 bytes, image/png, 2400 × 1260; SHA256 `b8e975bde9bbddfd2e4a97bcc903dbcb4fb26801b53f128e0a130f6067423b5d`. Its existing URL is retained. No new image was generated or uploaded.

Apple TN3156 documents a 1 MB main-resource limit and a 10 MB aggregate limit for associated preview resources, while noting that guidelines can change. The current index was 1345601 bytes. The new `index.html` is 10878 bytes, with static OpenGraph/Twitter metadata immediately after charset/title and all required metadata in the first 8192 bytes. It has no redirect, script requirement, automatic navigation or authentication gate. The stable canonical and verified image URL are unchanged.

`specification.html` is byte-identical to the published 1.2 index, SHA256 `e5e051482624ad71423d1dc0015161032f224fcba901f57850e91183b7ba9a0f`. All 17 chapters, 811 source mappings, technical content, WebMCP additions and accessibility corrections remain intact. Only relative hrefs changed in 47 supporting HTML pages to reach the relocated specification. Their visible text was compared exactly. The historical 1.2 transfer manifest and published output remain unchanged.

Build: `python3 build_share_preview.py --base-dir /path/to/exact/1.2/site` after obtaining the exact 158 manifest files. The builder validates their hashes before copying into `dist/knowme-praxis-share/`; it never reads credentials or makes network requests. Template: `source/share-index.html`. Tests: `source/verify_share_preview.py --forge-root /path/to/pinned/forge --axe-script /path/to/axe.min.js`.

Four headless views (1440px/390px, light/dark) passed automated accessibility, computed contrast, overflow, skip-link and keyboard navigation checks. JavaScript-disabled navigation passed. All 3700 local links resolved with valid targets; no missing files or anchors. Screenshots and complete measured output are in `checks-preview-delivery/`. The preview image was freshly verified publicly; render tests intercepted its request with those exact verified local bytes. This is not a claim of actual iMessage rendering, screen-reader testing, real-device testing or full WCAG conformance.

The parent completed publication after explicit user approval (`Sentinel_cea2a343a4e08191b55afdc994307e6d`) for exactly two authenticated operations: document add and updating the existing document-specific IPNS name. The existing image was reused without regeneration or reupload. This receipt task made no additional IPFS calls or credential accesses.

- [Stable share page](https://ipfs.prometheusags.ai/ipns/k51qzi5uqu5dl4048r1wozopco1c2b0vzfx0kj3rbs0g9rhadfjs8msvi4ll3t/index.html)
- [Immutable share page](https://ipfs.prometheusags.ai/ipfs/bafybeianggntnigojmuqpsn2bskslb3mob5lkbaagd3wcgtfjokgv2dazi/index.html)
- [Complete specification](https://ipfs.prometheusags.ai/ipfs/bafybeianggntnigojmuqpsn2bskslb3mob5lkbaagd3wcgtfjokgv2dazi/specification.html)

Parent independently verified all 159 local hashes before upload. Public immutable and stable indexes both returned HTTP 200, `text/html`, 10878 bytes and matched the candidate. The full specification returned HTTP 200, `text/html`, 1345601 bytes and matched the earlier published index exactly. The existing image returned HTTP 200, `image/png`, 272805 bytes with its verified hash. A `bytes=0-8191` range request returned HTTP 206 and the exact image metadata. No bulk public asset crawl or independent pin-state inspection is claimed. Actual iMessage rendering remains untested.

`publication-receipt-preview-delivery.json` records exact hashes, URLs and evidence attribution. `preview-delivery-manifest.json` retains its historical pre-upload candidate status; this external receipt supersedes that status. The package's 159 files remain unchanged. Resume from `docs/knowme-praxis-v1.2-accessibility`; do not republish merely to resume. Future publication or pointer updates require their own authorization.
