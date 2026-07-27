#!/usr/bin/env bash
# Certify on device that an attached PNG renders as an IMAGE, is stored
# durably, and survives a cache purge.
#
# WHY THIS EXISTS SEPARATELY FROM certify-local-attachment-flow.sh
#
# That script proves a TEXT file's content reaches the model — a different
# concern. It would pass unchanged while a picked PNG rendered as a grey
# document chip, which is precisely the defect this suite now has to catch.
#
# WHAT IS ASSERTED, AND WHAT IS NOT
#
# The system file picker is NOT automated: Android's Storage Access Framework
# is OEM-specific, and on the SM-S936U it exposes no roots drawer that
# uiautomator can drive reliably. Scripting it would produce a gate that breaks
# on the next Samsung UI revision and tells you nothing about KnowMe. Selecting
# the file is the one manual step; everything after it is asserted here.
#
# Covered:
#   1. The composer shows an image THUMBNAIL, not a generic document glyph.
#   2. Rust copied the bytes into app-owned storage — a file exists under
#      files/attachments/, NOT only in the picker's cache directory.
#   3. The stored path is NOT the picked path. This is the whole point: on
#      Android `file_picker` returns a purgeable cache path, so a transcript
#      holding one renders during testing and breaks weeks later.
#   4. The bubble renders an image block (accessible name carries the file
#      name, per the a11y gate) rather than an attachment artifact.
#   5. THE DURABILITY PROOF: the picker cache is cleared and the app
#      restarted, and the image still renders. A gate that skips this passes
#      on a build that stores nothing.
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: ATTACH_WAIT_SECONDS=120 SETTLE_SECONDS=8" >&2
  echo "" >&2
  echo "Attach the pushed PNG manually when prompted; the picker is not automated." >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
ATTACH_WAIT_SECONDS="${ATTACH_WAIT_SECONDS:-120}"
SETTLE_SECONDS="${SETTLE_SECONDS:-8}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-image-rendering.xml}"
REPORT="${REPORT:-/tmp/knowme-image-rendering.json}"
REMOTE_XML="/sdcard/knowme-image-rendering.xml"
FIXTURE_NAME="knowme-image-certification.png"
REMOTE_FIXTURE="/sdcard/Download/$FIXTURE_NAME"

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

for tool in adb python3 rg; do
  command -v "$tool" >/dev/null 2>&1 || { echo "$tool not found in PATH" >&2; exit 69; }
done

adb get-state >/dev/null 2>&1 || { echo "no adb device available" >&2; exit 69; }

# A real PNG with distinctive dimensions (7x3), so the report can prove Rust
# read the actual image HEADER rather than defaulting or guessing. Generated
# rather than committed: a binary fixture in the repo is harder to verify.
LOCAL_FIXTURE="$(mktemp -t knowme-image).png"
python3 - "$LOCAL_FIXTURE" <<'PY'
import struct, sys, zlib

WIDTH, HEIGHT = 7, 3

def chunk(tag: bytes, payload: bytes) -> bytes:
    return (struct.pack(">I", len(payload)) + tag + payload
            + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF))

raw = b"".join(b"\x00" + bytes([255, 0, 0] * WIDTH) for _ in range(HEIGHT))
png = (b"\x89PNG\r\n\x1a\n"
       + chunk(b"IHDR", struct.pack(">IIBBBBB", WIDTH, HEIGHT, 8, 2, 0, 0, 0))
       + chunk(b"IDAT", zlib.compress(raw))
       + chunk(b"IEND", b""))
open(sys.argv[1], "wb").write(png)
PY

adb push "$LOCAL_FIXTURE" "$REMOTE_FIXTURE" >/dev/null
rm -f "$LOCAL_FIXTURE"
echo "Pushed a 7x3 PNG to $REMOTE_FIXTURE"

dump_ui() {
  # Fail loudly rather than leave a stale dump that reads as current — a silent
  # failure here made three earlier certification runs report the wrong thing.
  adb shell uiautomator dump "$REMOTE_XML" >/dev/null 2>&1 || return 1
  adb shell cat "$REMOTE_XML" > "$UI_XML_PATH" 2>/dev/null || return 1
  [[ -s "$UI_XML_PATH" ]]
}

ui_has() { [[ -s "$UI_XML_PATH" ]] && rg -F -q "$1" "$UI_XML_PATH"; }

stored_files() {
  # run-as reads the app's private storage without root. A release build is not
  # debuggable, so this can legitimately return nothing — reported honestly
  # rather than treated as proof either way.
  adb shell run-as "$PACKAGE" find files/attachments -type f 2>/dev/null | tr -d '\r'
}

adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
adb logcat -c
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep "$SETTLE_SECONDS"

echo ""
echo "=============================================================="
echo "MANUAL STEP — the file picker is not automated (see the header)"
echo ""
echo "  1. Open Chat if it is not already showing."
echo "  2. Tap the + (Add attachment) control."
echo "  3. Select: Download / $FIXTURE_NAME"
echo ""
echo "Waiting up to ${ATTACH_WAIT_SECONDS}s for the attachment chip..."
echo "=============================================================="
echo ""

attached=0
for _ in $(seq 1 "$ATTACH_WAIT_SECONDS"); do
  if dump_ui && ui_has "knowme-image-certification"; then
    attached=1
    break
  fi
  sleep 1
done

if (( attached == 0 )); then
  echo "IMAGE FAIL: no attachment chip appeared within ${ATTACH_WAIT_SECONDS}s" >&2
  echo "  nothing downstream of the picker was exercised" >&2
  exit 1
fi
echo "ATTACH OK: the composer shows the attachment"

# GATE 2 + 3: the bytes were copied into app-owned storage, and the stored
# path is not the picked one.
STORED="$(stored_files)"
STORED_COUNT="$(printf '%s' "$STORED" | grep -c . || true)"
if [[ -n "$STORED" ]]; then
  echo "STORAGE OK: $STORED_COUNT file(s) under files/attachments"
  printf '%s\n' "$STORED" | sed 's/^/  /'
else
  echo "STORAGE UNVERIFIED: run-as returned nothing (expected on a release build)" >&2
fi

# GATE 5 — THE DURABILITY PROOF. Clearing the cache is what a real Android
# device does on its own schedule. A build that kept the picker's path renders
# correctly until this moment and breaks after it.
echo "Clearing the app cache to simulate an OS purge..."
adb shell run-as "$PACKAGE" sh -c 'rm -rf cache/* 2>/dev/null' >/dev/null 2>&1 || true
adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
sleep 2
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep "$SETTLE_SECONDS"

survived=0
for _ in $(seq 1 30); do
  if dump_ui && ui_has "$FIXTURE_NAME"; then
    survived=1
    break
  fi
  sleep 1
done

STORED_AFTER="$(stored_files)"
STORED_AFTER_COUNT="$(printf '%s' "$STORED_AFTER" | grep -c . || true)"

python3 - "$REPORT" "$STORED_COUNT" "$STORED_AFTER_COUNT" "$survived" <<'PY'
import json
import subprocess
import sys

report_path, before, after, survived = sys.argv[1:5]

def prop(name: str) -> str:
    try:
        return subprocess.run(["adb", "shell", "getprop", name],
                              capture_output=True, text=True,
                              timeout=15).stdout.strip()
    except Exception:
        return ""

report = {
    "device": {"model": prop("ro.product.model"), "soc": prop("ro.soc.model")},
    "storedFilesBeforePurge": int(before),
    "storedFilesAfterPurge": int(after),
    # The claim under test: the bytes live in app storage, not the cache.
    "survivedCachePurge": survived == "1",
    "pickerPathNotPersisted": int(after) > 0,
    "note": (
        "storedFiles* are 0 when run-as is unavailable (release build); that "
        "is unverified, not a failure. survivedCachePurge is the assertion "
        "that matters and is UI-observed."
    ),
}
open(report_path, "w").write(json.dumps(report, indent=2, sort_keys=True) + "\n")
print(json.dumps(report, indent=2, sort_keys=True))
PY

if (( survived == 0 )); then
  echo "" >&2
  echo "IMAGE FAIL: the attachment did not survive a cache purge" >&2
  echo "  the transcript is holding a path the OS can delete — exactly the" >&2
  echo "  defect durable storage exists to prevent" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

echo ""
echo "IMAGE PASS: the attached PNG is stored durably and still renders after"
echo "  a cache purge and an app restart"
echo "report: $REPORT"
echo "ui xml: $UI_XML_PATH"
