#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
BRAND_DIR="$REPO_ROOT/mobile/assets/branding"
ANDROID_RES="$REPO_ROOT/mobile/android/app/src/main/res"
IOS_ICONS="$REPO_ROOT/mobile/ios/Runner/Assets.xcassets/AppIcon.appiconset"
MACOS_ICONS="$REPO_ROOT/mobile/macos/Runner/Assets.xcassets/AppIcon.appiconset"

for command_name in rsvg-convert magick; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/knowme-app-icons.XXXXXX")
trap 'rm -rf "$WORK_DIR"' EXIT

rsvg-convert -w 1024 -h 1024 \
  -o "$WORK_DIR/rounded.png" "$BRAND_DIR/knowme-app-icon.svg"
rsvg-convert -w 1024 -h 1024 \
  -o "$WORK_DIR/opaque.png" "$BRAND_DIR/knowme-app-icon-ios.svg"
rsvg-convert -w 1024 -h 1024 \
  -o "$WORK_DIR/round.png" "$BRAND_DIR/knowme-app-icon-round.svg"

while IFS=' ' read -r density size; do
  destination="$ANDROID_RES/mipmap-$density"
  mkdir -p "$destination"
  magick "$WORK_DIR/rounded.png" -filter Lanczos -resize "${size}x${size}!" \
    "$destination/ic_launcher.png"
  magick "$WORK_DIR/round.png" -filter Lanczos -resize "${size}x${size}!" \
    "$destination/ic_launcher_round.png"
done <<'ANDROID_SIZES'
mdpi 48
hdpi 72
xhdpi 96
xxhdpi 144
xxxhdpi 192
ANDROID_SIZES

while IFS=' ' read -r filename size; do
  magick "$WORK_DIR/opaque.png" -filter Lanczos -resize "${size}x${size}!" \
    -alpha off "$IOS_ICONS/$filename"
done <<'IOS_SIZES'
Icon-App-20x20@1x.png 20
Icon-App-20x20@2x.png 40
Icon-App-20x20@3x.png 60
Icon-App-29x29@1x.png 29
Icon-App-29x29@2x.png 58
Icon-App-29x29@3x.png 87
Icon-App-40x40@1x.png 40
Icon-App-40x40@2x.png 80
Icon-App-40x40@3x.png 120
Icon-App-60x60@2x.png 120
Icon-App-60x60@3x.png 180
Icon-App-76x76@1x.png 76
Icon-App-76x76@2x.png 152
Icon-App-83.5x83.5@2x.png 167
Icon-App-1024x1024@1x.png 1024
IOS_SIZES

for size in 16 32 64 128 256 512 1024; do
  magick "$WORK_DIR/rounded.png" -filter Lanczos -resize "${size}x${size}!" \
    "$MACOS_ICONS/app_icon_${size}.png"
done

echo "Generated KnowMe Android, iOS, and macOS app icons."
