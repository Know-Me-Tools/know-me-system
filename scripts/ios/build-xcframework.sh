#!/usr/bin/env bash
# Build universal XCFramework for iOS device + simulator.
set -euo pipefail
PROFILE="${1:-release}"
ROOT="$(dirname "$(dirname "$(dirname "$0")")")"
RUST_DIR="$ROOT/rust"
BUILD="$ROOT/scripts/ios/build"
FLAGS=(); [[ "$PROFILE" == "release" ]] && FLAGS+=(--release)
mkdir -p "$BUILD"
cargo build --manifest-path "$RUST_DIR/Cargo.toml" --target aarch64-apple-ios "${FLAGS[@]}"
cargo build --manifest-path "$RUST_DIR/Cargo.toml" --target aarch64-apple-ios-sim "${FLAGS[@]}"
cargo build --manifest-path "$RUST_DIR/Cargo.toml" --target x86_64-apple-ios "${FLAGS[@]}"
SIM_FAT="$BUILD/libgen_ui_core_sim.a"
lipo -create \
  "$RUST_DIR/target/aarch64-apple-ios-sim/$PROFILE/libgen_ui_core.a" \
  "$RUST_DIR/target/x86_64-apple-ios/$PROFILE/libgen_ui_core.a" \
  -output "$SIM_FAT"
XCFW="$BUILD/GenUICore.xcframework"
rm -rf "$XCFW"
xcodebuild -create-xcframework \
  -library "$RUST_DIR/target/aarch64-apple-ios/$PROFILE/libgen_ui_core.a" \
  -library "$SIM_FAT" \
  -output "$XCFW"
mkdir -p "$ROOT/mobile/ios/Frameworks"
cp -R "$XCFW" "$ROOT/mobile/ios/Frameworks/"
echo "✓ XCFramework → mobile/ios/Frameworks/"
