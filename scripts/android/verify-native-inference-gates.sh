#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/app.apk" >&2
  exit 64
fi

APK="$1"
if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

READ_ELF="${READELF:-}"
if [[ -z "$READ_ELF" ]]; then
  if command -v llvm-readelf >/dev/null 2>&1; then
    READ_ELF="$(command -v llvm-readelf)"
  elif command -v readelf >/dev/null 2>&1; then
    READ_ELF="$(command -v readelf)"
  elif command -v xcrun >/dev/null 2>&1 && xcrun --find llvm-readelf >/dev/null 2>&1; then
    READ_ELF="$(xcrun --find llvm-readelf)"
  else
    echo "readelf/llvm-readelf not found; install LLVM or set READELF=/path/to/readelf" >&2
    exit 69
  fi
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

unzip -q "$APK" 'lib/*/*.so' -d "$TMP_DIR"

if find "$TMP_DIR/lib" -mindepth 1 -maxdepth 1 -type d ! -name arm64-v8a | grep -q .; then
  echo "G2 FAIL: APK contains non-arm64 native libraries:" >&2
  find "$TMP_DIR/lib" -mindepth 1 -maxdepth 1 -type d ! -name arm64-v8a -print >&2
  echo "Build device APKs with: flutter build apk --target-platform android-arm64" >&2
  exit 1
fi

for required in libgen_ui_ffi.so liblitertlm_jni.so; do
  if [[ ! -f "$TMP_DIR/lib/arm64-v8a/$required" ]]; then
    echo "G2 FAIL: missing lib/arm64-v8a/$required" >&2
    exit 1
  fi
done

if find "$TMP_DIR/lib" -type f \( \
  -iname 'libOpenCL*.so' -o \
  -iname 'libGLES_mali*.so' -o \
  -iname 'libPVROCL*.so' -o \
  -iname 'libvndksupport.so' \
  \) | grep -q .; then
  echo "G2 FAIL: APK bundles vendor GPU/OpenCL libraries:" >&2
  find "$TMP_DIR/lib" -type f \( \
    -iname 'libOpenCL*.so' -o \
    -iname 'libGLES_mali*.so' -o \
    -iname 'libPVROCL*.so' -o \
    -iname 'libvndksupport.so' \
    \) -print >&2
  exit 1
fi

while IFS= read -r native_library; do
  if "$READ_ELF" -d "$native_library" | grep -E 'NEEDED.*(OpenCL|GLES_mali|PVROCL|vndksupport)' >/dev/null; then
    echo "G1 FAIL: $(basename "$native_library") hard-links a vendor GPU/OpenCL library" >&2
    "$READ_ELF" -d "$native_library" | grep -E 'NEEDED.*(OpenCL|GLES_mali|PVROCL|vndksupport)' >&2
    exit 1
  fi
done < <(find "$TMP_DIR/lib/arm64-v8a" -type f -name '*.so' | sort)

echo "G1 PASS: APK native libraries have no vendor GPU/OpenCL DT_NEEDED entries"
echo "G2 PASS: APK is arm64-only and contains required KnowMe/LiteRT-LM native libraries"
echo "G2 PASS: APK does not bundle vendor GPU/OpenCL libraries"
echo "G3-G6 require install/launch/logcat/runtime self-test on a physical device"
