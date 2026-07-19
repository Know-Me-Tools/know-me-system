#!/usr/bin/env bash
# Validate gen_ui_ffi for current-generation 64-bit Android phones, then
# regenerate the Dart bridge. Gradle/CargoKit owns APK packaging; copying a
# second library into app/src/main/jniLibs would shadow fresh CargoKit output.
set -euo pipefail
PROFILE="${1:-release}"
ROOT="$(cd "$(dirname "$(dirname "$(dirname "$0")")")" && pwd)"
RUST_DIR="$ROOT/rust"
ANDROID_SDK="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-$HOME/Library/Android/sdk}}"
if [[ -z "${ANDROID_NDK:-}" ]]; then
  ANDROID_NDK="$(find "$ANDROID_SDK/ndk" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1)"
fi
if [[ ! -d "$ANDROID_NDK" ]]; then
  echo "Android NDK not found. Set ANDROID_NDK or install it in $ANDROID_SDK/ndk." >&2
  exit 1
fi
export ANDROID_NDK
export NDK_ROOT="$ANDROID_NDK"
export PYTHON3_EXECUTABLE="${PYTHON3_EXECUTABLE:-$(command -v python3)}"
declare -A ABIS=(["arm64-v8a"]="aarch64-linux-android")
for ABI in "${!ABIS[@]}"; do
  TARGET="${ABIS[$ABI]}"
  (
    cd "$RUST_DIR"
    cargo ndk --target "$ABI" --platform 29 -- build -p gen_ui_ffi --target "$TARGET" $([[ "$PROFILE" == "release" ]] && echo "--release")
  )
  echo "✓ $ABI"
done
if command -v flutter_rust_bridge_codegen &>/dev/null; then
  flutter_rust_bridge_codegen generate --config-file "$RUST_DIR/flutter_rust_bridge.yaml"
  echo "✓ Dart bindings generated"
fi
