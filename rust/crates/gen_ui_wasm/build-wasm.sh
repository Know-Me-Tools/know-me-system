#!/usr/bin/env bash
# TJ-ARCH-MOB-001 compliant
# Build the gen_ui_wasm web module. Output: pkg/ (wasm-pack bundler layout) that
# @prometheus-ags/gen-ui-wasm re-exports. Requires: wasm-pack, wasm-opt (binaryen).
set -euo pipefail
cd "$(dirname "$0")"

OUT_DIR="${1:-pkg}"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found. Install: cargo install wasm-pack" >&2; exit 1
fi

# Build against the size-optimized profile from the workspace Cargo.toml.
# --target bundler suits Vite/webpack consumers (the web app).
wasm-pack build --release --target bundler --out-dir "$OUT_DIR" \
  -- --profile wasm-release

# Extra size pass. wasm-pack runs wasm-opt when present, but pin -Oz explicitly.
if command -v wasm-opt >/dev/null 2>&1; then
  WASM_FILE="$(find "$OUT_DIR" -name '*_bg.wasm' | head -n1)"
  [ -n "$WASM_FILE" ] && wasm-opt -Oz --enable-bulk-memory -o "$WASM_FILE" "$WASM_FILE"
  echo "  wasm-opt -Oz applied to $WASM_FILE"
else
  echo "  (wasm-opt not found — skipping size pass; install binaryen)"
fi

echo "✅ gen_ui_wasm built to $OUT_DIR/"
