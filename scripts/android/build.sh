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
# llama.cpp's Vulkan CMake checks for the desktop SDK's SPIRV-Headers CONFIG
# package. The NDK contains the same headers but omits that package, so expose
# the checked-in compatibility package without requiring a host Vulkan SDK.
export VULKAN_SDK="${VULKAN_SDK:-$ROOT/scripts/android/vulkan-sdk}"
export CMAKE_PREFIX_PATH="$VULKAN_SDK${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
# The NDK provides the Android Vulkan C headers and glslc, while the C++ RAII
# header used by llama.cpp is supplied by the host Vulkan-Headers formula.
VULKAN_HEADERS_PREFIX="${VULKAN_HEADERS_PREFIX:-$(brew --prefix vulkan-headers 2>/dev/null || true)}"
if [[ -d "$VULKAN_HEADERS_PREFIX/include" ]]; then
  export CXXFLAGS="-I$VULKAN_HEADERS_PREFIX/include${CXXFLAGS:+ $CXXFLAGS}"
else
  echo "Vulkan-Headers not found; install it or set VULKAN_HEADERS_PREFIX." >&2
  exit 1
fi
# Android's toolchain sets CMAKE_FIND_ROOT_PATH_MODE_PACKAGE=ONLY, so expose
# the compatibility CONFIG file inside the selected NDK sysroot where CMake
# will discover it without weakening the toolchain's package isolation.
SPIRV_PACKAGE_DIR="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/cmake/SPIRV-Headers"
mkdir -p "$SPIRV_PACKAGE_DIR"
cp "$ROOT/scripts/android/vulkan-sdk/lib/cmake/SPIRV-Headers/SPIRV-HeadersConfig.cmake" \
  "$SPIRV_PACKAGE_DIR/SPIRV-HeadersConfig.cmake"
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
