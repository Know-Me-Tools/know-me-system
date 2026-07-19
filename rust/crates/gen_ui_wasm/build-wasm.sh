#!/usr/bin/env bash
# TJ-ARCH-MOB-001 compliant
# Build the gen_ui_wasm web module. Output: pkg/ (wasm-pack bundler layout) that
# @prometheus-ags/gen-ui-wasm re-exports. Requires: wasm-pack, wasm-opt (binaryen).
set -euo pipefail

CALLER_DIR="$(pwd)"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR_INPUT="${1:-pkg}"
if [[ "$OUT_DIR_INPUT" = /* ]]; then
  OUT_DIR="$OUT_DIR_INPUT"
else
  OUT_DIR="$CALLER_DIR/$OUT_DIR_INPUT"
fi
cd "$SCRIPT_DIR"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found. Install: cargo install wasm-pack" >&2; exit 1
fi

# Build against the size-optimized profile from the workspace Cargo.toml.
# --target bundler suits Vite/webpack consumers (the web app). Recent Cargo
# rejects combining --release with --profile, while wasm-pack assumes debug
# output when forced to --dev. Build the exact profile directly, then run the
# wasm-bindgen step explicitly.
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release

WASM_BINDGEN="$(command -v wasm-bindgen || true)"
if [[ -z "$WASM_BINDGEN" ]]; then
  WASM_PACK_CACHE="${XDG_CACHE_HOME:-$HOME/Library/Caches}/.wasm-pack"
  WASM_BINDGEN="$(find "$WASM_PACK_CACHE" -path '*/bin/wasm-bindgen' -type f 2>/dev/null | sort -r | head -n1 || true)"
fi
if [[ -z "$WASM_BINDGEN" ]]; then
  echo "wasm-bindgen not found. Install: cargo install wasm-bindgen-cli" >&2; exit 1
fi

case "$OUT_DIR" in
  ""|"/"|"$CALLER_DIR"|"$SCRIPT_DIR")
    echo "Refusing to clear unsafe WASM output directory: $OUT_DIR" >&2; exit 1
    ;;
esac

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
"$WASM_BINDGEN" "../../target/wasm32-unknown-unknown/wasm-release/gen_ui_wasm.wasm" \
  --out-dir "$OUT_DIR" \
  --typescript \
  --target bundler

cat > "$OUT_DIR/package.json" <<'JSON'
{
  "name": "@prometheus-ags/gen-ui-wasm-bindings",
  "version": "0.1.0",
  "type": "module",
  "sideEffects": false
}
JSON

# Extra size pass. wasm-pack runs wasm-opt when present, but pin -Oz explicitly.
if command -v wasm-opt >/dev/null 2>&1; then
  WASM_FILE="$(find "$OUT_DIR" -name '*_bg.wasm' | head -n1)"
  [ -n "$WASM_FILE" ] && wasm-opt -Oz --enable-bulk-memory -o "$WASM_FILE" "$WASM_FILE"
  echo "  wasm-opt -Oz applied to $WASM_FILE"
else
  echo "  (wasm-opt not found — skipping size pass; install binaryen)"
fi

echo "✅ gen_ui_wasm built to $OUT_DIR/"
