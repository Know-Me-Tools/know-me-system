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
  | { type: "image"; url: string | null; dataBase64: string | null; mime: string }
  | { type: "divider" };
