# feature: ask-image (Tile 03 · FUNC-SPEC §04 "Ask Image")

Vision Q&A against images/scans/screenshots/PDFs, fully on-device.
Not yet wired into `app/navigation.ts` / `app/router.tsx` — add the route when
the first screen lands so the build stays green until then.

Planned decomposition (mirror `features/models`):

- `screens/AskImageScreen.tsx` — drop zone, image preview, Q&A thread, batch
  mode for multi-page PDFs; extraction output routable into Chat or memory.
- `hooks/useAskImage.ts` — selectors over the store; no network here.
- `stores/askImageStore.ts` — the only layer calling APIs (Zustand + Immer).
- API: `src/api/` invoke wrappers for new Tauri commands
  `ask_image_analyze`, `ask_image_batch` (Rust side: `gen_ui_inference`
  vision path — Gemma-3n vision desktop / Phi-3.5-vision mobile).

Spec anchors: FUNC-SPEC §04 tile 3; roadmap M4 in `docs/implementation-roadmap.md`.
