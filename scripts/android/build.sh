#!/usr/bin/env bash
# Build gen_ui_core for all Android ABIs, then regenerate the Dart bridge.
set -euo pipefail
PROFILE="${1:-release}"
ROOT="$(dirname "$(dirname "$(dirname "$0")")")"
RUST_DIR="$ROOT/rust"
OUT="$ROOT/mobile/android/app/src/main/jniLibs"
declare -A ABIS=(["arm64-v8a"]="aarch64-linux-android" ["armeabi-v7a"]="armv7-linux-androideabi" ["x86_64"]="x86_64-linux-android")
for ABI in "${!ABIS[@]}"; do
  TARGET="${ABIS[$ABI]}"
  cargo ndk --target "$ABI" --platform 24 -- build --manifest-path "$RUST_DIR/Cargo.toml" --target "$TARGET" $([[ "$PROFILE" == "release" ]] && echo "--release")
  mkdir -p "$OUT/$ABI"
  cp "$RUST_DIR/target/$TARGET/$PROFILE/libgen_ui_core.so" "$OUT/$ABI/"
  echo "✓ $ABI"
done
if command -v flutter_rust_bridge_codegen &>/dev/null; then
  flutter_rust_bridge_codegen generate --config-file "$RUST_DIR/flutter_rust_bridge.yaml"
  echo "✓ Dart bindings generated"
fi
