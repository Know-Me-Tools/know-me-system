// TJ-ARCH-MOB-001 compliant
//! gen_ui_client (L2) — Flint platform integration for gen_ui_core.
//!
//! Owns ALL outbound connections (TJ-ARCH-MOB-001: networking lives only here, never
//! in Dart/TS). Three planes, one façade [`flint::FlintClient`]:
//!   * gate   — Kratos/JWT auth, token lifecycle, Cedar `@require_approval` polling.
//!   * forge  — Quarry REST (`EntityTransport`), A2UI-registry MCP server, AG-UI runs.
//!   * frf    — realtime spine (Spine subscribe/ack, EntityService watch) [feature = "frf"].
//!
//! Verified against flint-gate/forge/FRF HEADs 2026-07-15 (see the C-006 done log for
//! SHAs). gate + forge are plain HTTP/SSE/JSON-RPC (reqwest); FRF is tonic gRPC and is
//! therefore native-only + feature-gated so the default build stays wasm-safe.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

pub mod flint;

// The FlintClient façade is native-only (reqwest/tonic IO); the browser reaches the
// same planes from JS per the layer contract. Token types are cross-target.
pub use flint::token;
#[cfg(not(target_arch = "wasm32"))]
pub use flint::{FlintClient, FlintConfig};
