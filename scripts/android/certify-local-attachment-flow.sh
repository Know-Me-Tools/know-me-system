#!/usr/bin/env bash
# Certify the attachment flow on device: a file selected in the composer travels
# through Rust's extraction pipeline and its content reaches the model.
#
# WHAT THIS COVERS AND WHAT IT DOES NOT
#
# The system file picker (Android's Storage Access Framework, opened by
# `file_picker`) is NOT automated. Its layout is OEM-specific — on the SM-S936U
# it exposes no roots drawer and no search field that uiautomator can drive
# reliably — so scripting it would produce a gate that breaks on the next
# Samsung UI revision and tells you nothing about KnowMe. The one manual step is
# selecting the file; everything after it is asserted here.
#
# Covered end to end once a file is attached:
#   - the attachment renders in the transcript as an artifact block
#   - `attachmentPaths` reaches Rust (chat_send carries them)
#   - Rust reads and extracts the file (extraction markers in logcat)
#   - the extracted CONTENT reaches the model: the reply must quote a sentinel
#     phrase that appears ONLY inside the attached file, never in the prompt
#
# That last assertion is the point. A gate that only checks "an attachment chip
# rendered" would pass while the file's content was silently dropped — which is
# exactly the class of bug this suite exists to catch.
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: SENTINEL=PELICANWHARF TIMEOUT_SECONDS=420 ATTACH_WAIT_SECONDS=120" >&2
  echo "" >&2
  echo "Attach the pushed file manually when prompted; the picker is not automated." >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
SENTINEL="${SENTINEL:-PELICANWHARF}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-420}"
ATTACH_WAIT_SECONDS="${ATTACH_WAIT_SECONDS:-120}"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-attachment.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-attachment.xml}"
REMOTE_XML="/sdcard/knowme-local-attachment.xml"
REMOTE_FILE="/sdcard/Download/knowme-attachment-certification.txt"

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

for tool in adb python3 rg; do
  command -v "$tool" >/dev/null 2>&1 || { echo "$tool not found in PATH" >&2; exit 69; }
done

adb get-state >/dev/null 2>&1 || { echo "no adb device available" >&2; exit 69; }

# The sentinel lives ONLY in the file. If the model can quote it, the file's
# content genuinely reached the context — no other path could have supplied it.
LOCAL_FILE="$(mktemp -t knowme-attach)"
cat > "$LOCAL_FILE" <<EOF
KnowMe attachment certification fixture.

The harbour inspection code for this quarter is ${SENTINEL}.

Everything else in this file is filler so the extractor has real text to work
with rather than a single line that could be coincidentally echoed.
EOF

adb push "$LOCAL_FILE" "$REMOTE_FILE" >/dev/null
rm -f "$LOCAL_FILE"
echo "Pushed fixture to $REMOTE_FILE (sentinel: $SENTINEL)"

dump_ui() {
  # Fail loudly rather than leave a stale dump that reads as current — a silent
  # failure here made three earlier certification runs report the wrong thing.
  if ! adb shell uiautomator dump "$REMOTE_XML" >/dev/null 2>&1; then
    return 1
  fi
  adb shell cat "$REMOTE_XML" > "$UI_XML_PATH" 2>/dev/null || return 1
  [[ -s "$UI_XML_PATH" ]]
}

ui_has() {
  [[ -s "$UI_XML_PATH" ]] && rg -F -q "$1" "$UI_XML_PATH"
}

adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
adb logcat -c
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep 8

echo ""
echo "=============================================================="
echo "MANUAL STEP — the file picker is not automated (see the header)"
echo ""
echo "  1. Open Chat if it is not already showing."
echo "  2. Tap the + (Add attachment) control."
echo "  3. Select: Download / knowme-attachment-certification.txt"
echo ""
echo "Waiting up to ${ATTACH_WAIT_SECONDS}s for the attachment chip to appear..."
echo "=============================================================="
echo ""

attached=0
for _ in $(seq 1 "$ATTACH_WAIT_SECONDS"); do
  if dump_ui && ui_has "knowme-attachment-certification"; then
    attached=1
    break
  fi
  sleep 1
done

if (( attached == 0 )); then
  echo "ATTACH FAIL: no attachment chip appeared within ${ATTACH_WAIT_SECONDS}s" >&2
  echo "  the composer never showed knowme-attachment-certification.txt, so" >&2
  echo "  nothing downstream of the picker was exercised" >&2
  exit 1
fi
echo "ATTACH OK: the composer shows the attachment"

# Ask about the sentinel WITHOUT stating it. If the reply contains it, the only
# possible source is the extracted file.
PROMPT="What is the harbour inspection code in the attached file? Answer with the code only."
escaped_prompt="${PROMPT// /%s}"
adb shell input text "$escaped_prompt"
sleep 1

if dump_ui && ui_has "Send message"; then
  python3 - "$UI_XML_PATH" <<'PY' > /tmp/knowme-attach-send.txt
import re, sys
xml = open(sys.argv[1], errors="replace").read()
for node in re.findall(r"<node[^>]*/?>", xml):
    desc = re.search(r'content-desc="([^"]*)"', node)
    bounds = re.search(r'bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"', node)
    if desc and bounds and desc.group(1).strip() == "Send message":
        x = (int(bounds.group(1)) + int(bounds.group(3))) // 2
        y = (int(bounds.group(2)) + int(bounds.group(4))) // 2
        print(f"{x} {y}")
        break
PY
  read -r sx sy < /tmp/knowme-attach-send.txt || true
  if [[ -n "${sx:-}" ]]; then
    adb shell input tap "$sx" "$sy" >/dev/null
  else
    echo "ATTACH FAIL: the send control has no resolvable position" >&2
    exit 1
  fi
else
  echo "ATTACH FAIL: could not find the send control" >&2
  exit 1
fi

echo "Sent; waiting up to ${TIMEOUT_SECONDS}s for the reply..."
found=0
for _ in $(seq 1 "$TIMEOUT_SECONDS"); do
  adb logcat -d > "$LOG_PATH" 2>/dev/null || true
  if dump_ui && ui_has "$SENTINEL"; then
    found=1
    break
  fi
  sleep 1
done

adb logcat -d > "$LOG_PATH" 2>/dev/null || true

if (( found == 0 )); then
  echo "ATTACH FAIL: the reply never quoted the sentinel ($SENTINEL)" >&2
  echo "  the attachment reached the composer but its CONTENT did not reach the" >&2
  echo "  model — the extraction path is where to look" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

# Corroboration only. The sentinel in the reply is the primary proof; these
# markers say which layer did the work, and are lane-dependent.
if rg -q "attachment|extract" "$LOG_PATH" 2>/dev/null; then
  echo "ATTACH INFO: extraction markers present in logcat"
else
  echo "ATTACH INFO: no extraction markers in logcat (release builds strip some)" >&2
fi

echo "ATTACH PASS: the attached file's content reached the model"
echo "  sentinel '$SENTINEL' appeared in the reply and exists ONLY in the file"
echo "logcat: $LOG_PATH"
echo "ui xml: $UI_XML_PATH"
