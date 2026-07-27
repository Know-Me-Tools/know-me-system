# Chat media rendering, download, and share

## Why

A PNG attached in the composer renders as a grey document chip on every surface.
`ContentBlock::Image` is defined in `gen_ui_types` and decoded by both UIs but
constructed by nothing, so the block never arrives — a renderer fix alone cannot
work. Five sites hardcode the block kind `attachment` regardless of mime, and
Flutter's image renderer has no file-path arm.

Users cannot see images they attach, cannot see model-produced media, cannot see
OpenGraph thumbnails on links, and cannot save or share any of it.

## What changes

- Rust copies attachment bytes into app-owned storage with a stable
  content-hashed path, so rendering survives Android's purgeable picker cache.
- Rust constructs mime-aware `ContentBlock::Image` once; both UIs become
  consumers instead of each deciding the kind themselves.
- Flutter and React render local and remote images and video, including the
  React composer thumbnail, which is dead today because the adapter hardcodes
  `document`.
- Download on every surface; native share where the platform supports it
  (Android, iOS, macOS, Windows, Web). Linux gets download and no share control.
- React link previews render the OpenGraph image, matching Flutter.
- Attachment files are deleted with their messages and swept on startup.

## Success criteria

- An attached PNG renders inline in the bubble on Flutter (iOS, Android), React
  web, and Tauri desktop — and still renders after an Android cache purge.
- Video attachments and remote media render on all four surfaces.
- A link with OpenGraph metadata shows its image on both React and Flutter.
- Media can be saved to disk on all four surfaces, and shared natively on every
  platform except Linux, where the share control is absent rather than dead.
- Deleting a message or conversation removes its attachment files.
- Every rendered image carries alt text; download and share controls have
  accessible names and ≥44px targets.

## Non-goals

- Backfilling media into conversations persisted before this change. Those
  messages never stored bytes durably; claiming otherwise would be dishonest.
- Linux native share. `share_plus` 13.3.0 has no Linux file support and Tauri
  has no share sheet; building a bespoke path is out of scope.
- Thumbnail generation. Full images render directly; a thumbnail pipeline is a
  separate performance change if measurement shows it is needed.
