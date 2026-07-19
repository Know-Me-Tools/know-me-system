#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=180" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-180}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
TOKEN="${TOKEN:-KMRESTART$(date +%H%M%S)}"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-restart-recovery.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-restart-recovery.xml}"
REMOTE_UI_XML="/sdcard/knowme-local-restart-recovery.xml"

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

if ! command -v adb >/dev/null 2>&1; then
  echo "adb not found in PATH" >&2
  exit 69
fi

if ! command -v rg >/dev/null 2>&1; then
  echo "rg not found in PATH" >&2
  exit 69
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found in PATH" >&2
  exit 69
fi

dump_ui() {
  adb shell uiautomator dump "$REMOTE_UI_XML" >/dev/null 2>&1
  adb exec-out cat "$REMOTE_UI_XML" > "$UI_XML_PATH"
}

tap_matching_node() {
  local needle="$1"
  dump_ui
  local point
  point="$(python3 - "$UI_XML_PATH" "$needle" <<'PY'
import re
import sys
import xml.etree.ElementTree as ET

path, needle = sys.argv[1], sys.argv[2].lower()
root = ET.parse(path).getroot()
for node in root.iter("node"):
    text = (node.attrib.get("text") or "").lower()
    desc = (node.attrib.get("content-desc") or "").lower()
    hint = (node.attrib.get("hint") or "").lower()
    if needle not in text and needle not in desc and needle not in hint:
        continue
    match = re.search(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]", node.attrib.get("bounds", ""))
    if not match:
        continue
    x1, y1, x2, y2 = map(int, match.groups())
    print(f"{(x1 + x2) // 2} {(y1 + y2) // 2}")
    raise SystemExit(0)
raise SystemExit(1)
PY
)" || return 1
  if [[ -z "$point" ]]; then
    return 1
  fi
  adb shell input tap $point
}

screen_dimensions() {
  adb shell wm size | tr -d '\r' | python3 -c 'import re,sys
text=sys.stdin.read()
match=re.search(r"Physical size:\s*(\d+)x(\d+)", text)
if not match:
    raise SystemExit(1)
print(match.group(1), match.group(2))'
}

tap_chat_fallback() {
  local width height
  read -r width height < <(screen_dimensions)
  adb shell input tap "$((width * 3 / 10))" "$((height - 220))"
}

echo "Creating persisted local-model response with marker: $TOKEN"
PROMPT="Reply exactly ${TOKEN}." \
  EXPECTED_SUBSTRINGS="$TOKEN" \
  TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
  LOG_PATH="/tmp/knowme-local-restart-seed.log" \
  UI_XML_PATH="/tmp/knowme-local-restart-seed.xml" \
  "$PROMPT_GATE" "$APK" "$TOKEN" "$PACKAGE"

adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
adb logcat -c
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  adb logcat -d -v time > "$LOG_PATH"
  tap_matching_node "Chat" >/dev/null 2>&1 || tap_chat_fallback || true
  tap_matching_node "Latest" >/dev/null 2>&1 || true
  dump_ui || true

  if [[ -z "$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)" ]]; then
    echo "RESTART FAIL: app process is not alive after relaunch" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi

  if rg -q "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|liblitertlm.*dlopen failed|JNI_OnLoad.*failed" "$LOG_PATH"; then
    echo "RESTART FAIL: KnowMe native-load failure found after relaunch" >&2
    rg -n "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|liblitertlm.*dlopen failed|JNI_OnLoad.*failed" "$LOG_PATH" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi

  if rg -F -q "$TOKEN" "$UI_XML_PATH"; then
    PID="$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)"
    if [[ -z "$PID" ]]; then
      echo "RESTART FAIL: marker was dumped but app process is not alive" >&2
      exit 1
    fi
    echo "RESTART PASS: app relaunched and persisted marker is visible: $TOKEN"
    echo "pid: $PID"
    echo "logcat: $LOG_PATH"
    echo "ui xml: $UI_XML_PATH"
    exit 0
  fi

  sleep 2
done

echo "RESTART FAIL: timed out waiting for persisted marker after relaunch: $TOKEN" >&2
echo "logcat: $LOG_PATH" >&2
echo "ui xml: $UI_XML_PATH" >&2
exit 1
