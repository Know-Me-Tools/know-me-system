# feature: audio-scribe (Tile 04 · FUNC-SPEC §04 "Audio Scribe")

On-device transcription/diarization/translation via whisper.cpp
(`rust/crates/gen_ui_audio` already carries whisper-rs + cpal).
Not yet wired into `app/navigation.ts` / `app/router.tsx`.

Planned decomposition:

- `screens/AudioScribeScreen.tsx` — record/import, live streaming transcript,
  speaker labels, auto-summary, "send to Chat" / "save to memory" actions.
- `hooks/useScribeSession.ts` — subscribe to streaming transcript slices.
- `stores/audioScribeStore.ts` — owns Tauri channel subscription + commands
  `scribe_start`, `scribe_stop`, `scribe_import_file`.
- Rust: `gen_ui_audio` is the engine; add Tauri command handlers in
  `desktop/src-tauri` bridging to it.

Spec anchors: FUNC-SPEC §04 tile 4; UX-MOOD journey "Drafting Case Notes
Offline"; roadmap M4.
