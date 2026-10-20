# KnowMe Praxis 1.2 publication and resume record

Published on 20 October 2026 from documentation checkpoint `148b685ebf197c329a0a97b2ff5c360b2c99091b`.

- [Stable document](https://ipfs.prometheusags.ai/ipns/k51qzi5uqu5dl4048r1wozopco1c2b0vzfx0kj3rbs0g9rhadfjs8msvi4ll3t/index.html)
- [Immutable edition](https://ipfs.prometheusags.ai/ipfs/bafybeidsh26hagvmfxm45a4q45lehcoi2mktmrnj33fwontaqboqkuxbe4/index.html)
- [Corrected preview image](https://ipfs.prometheusags.ai/ipfs/bafkreify5f2332n33x6s4suxxteqhw6lj6zgqanvh4ji4cqtb5qgoqr3lu)

The parent reports successful execution of exactly four explicitly approved authenticated operations: document-specific IPNS key generation, preview image add, document package add, and name publication. The returned name was `k51qzi5uqu5dl4048r1wozopco1c2b0vzfx0kj3rbs0g9rhadfjs8msvi4ll3t` and its published value was `/ipfs/bafybeidsh26hagvmfxm45a4q45lehcoi2mktmrnj33fwontaqboqkuxbe4`. Both add requests specified `pin=true`; no independent authenticated pin/list inspection was performed. This receipt task made no RPC calls or credential accesses.

Parent verification confirmed the public image bytes, MIME and 2400 × 1260 dimensions; the immutable index hash; a stable IPNS crawler GET returning HTTP 200 with `text/html`, exact index bytes and correct static canonical/og:url/image tags; and the corrected `praxis.css` hash. All 158 local files matched the manifest. No bulk public crawl was performed, so this record does not claim every remote asset was fetched. Exact values and attribution are in `publication-receipt-1.2.json`.

Twelve automated viewport/theme/page cases had zero axe violations, computed text-contrast failures or unfocusable scrolling regions. The main-page checks expanded all 811 retained source sections. Keyboard scrolling and summary focus passed. Four additional release views confirmed metadata and screenshots byte-identical to the tested draft. Axe incomplete evaluations remain preserved; no full WCAG conformance, screen-reader user-test or native-device validation is claimed. All 17 chapter text trees and 50 technical files remain unchanged from 1.1, including the WebMCP additions.

The immutable published package is represented by `release-transfer-1.2.json` and its exact file hashes. `candidate-manifest-1.2.json`, `PREPARED-1.2.md` and the release test report describe the earlier preparation stage; this external receipt supersedes their pending-publication status without rewriting historical evidence or published files. `site/` remains an input tree containing historical HTML; build outputs belong in `dist/knowme-praxis/`.

Resume from branch `docs/knowme-praxis-v1.2-accessibility`. Do not re-upload or republish merely to resume. To reproduce the final package, supply the stable URL as `PUBLIC_CANONICAL_URL` and the verified image URL as `PUBLIC_OG_IMAGE_URL` to the pinned-forge build, then compare all files with the transfer manifest. No automatic publishing hook or credential file is included. Future IPNS renewal or content updates are outside this completed publication record and require their own authorized action.
