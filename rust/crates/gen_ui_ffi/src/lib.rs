// TJ-ARCH-MOB-001 compliant
//! gen_ui_ffi (LEAF) — flutter_rust_bridge surface. Thin: editing app logic here
//! does not retrigger deep recompiles. Re-exports intent-level APIs + streams so
//! Dart never touches raw SurrealQL / SQL. Wave-1 lanes wire implementations
//! behind these signatures.
//!
//! Generated glue (`frb_generated.rs`) is produced by
//! `flutter_rust_bridge_codegen generate` and is gitignored — do not hand-edit.
pub mod api;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
