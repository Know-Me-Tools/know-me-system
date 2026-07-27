// TJ-ARCH-MOB-001 compliant
// ContentBlock — the cross-platform UI contract, mirrored from gen_ui_types
// (Rust). serde emits { type: "text", text: "..." } etc (tag="type", camelCase).
// Keep in lockstep with crates/gen_ui_types/src/content_block.rs.

export type ContentBlock =
  | { type: "text"; text: string }
  | { type: "thinking"; text: string }
  | { type: "code"; language: string; code: string }
  | { type: "citation"; source: string; quote: string }
  | { type: "memory"; operation: string; key: string; value: string | null }
  | { type: "toolUse"; id: string; name: string; inputJson: string }
  | { type: "toolResult"; toolUseId: string; outputJson: string; isError: boolean }
  | { type: "skill"; name: string; status: string }
  | { type: "artifact"; id: string; kind: string; content: string }
  /**
   * Renderable media. Exactly one of `url`, `dataBase64` or `path` carries the
   * bytes.
   *
   * `path` points into app-owned storage, never at a path a file picker
   * returned — a picked path is purgeable on Android and expires on iOS, so a
   * transcript holding one renders now and breaks later.
   *
   * `alt` is required for any image a user can see; a missing accessible name
   * fails the a11y gate. `width`/`height` let the renderer reserve space before
   * the bytes decode, so an arriving image does not shift the layout.
   */
  | {
      type: "image"
      url: string | null
      dataBase64: string | null
      mime: string
      path?: string
      alt?: string
      width?: number
      height?: number
    }
  | { type: "divider" };
