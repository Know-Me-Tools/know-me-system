#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 3 ]]; then
  echo "usage: $0 /path/to/app.apk [expected-visible-text] [package.name]" >&2
  echo "env: PROMPT='Reply exactly TOKEN.' TIMEOUT_SECONDS=240" >&2
  exit 64
fi

APK="$1"
if [[ $# -ge 2 ]]; then
  EXPECTED_TEXT="$2"
else
  EXPECTED_TEXT="KMOK$(date +%H%M%S)"
fi
PACKAGE="${3:-tools.knowme.knowme}"
PROMPT="${PROMPT:-Reply exactly ${EXPECTED_TEXT}.}"
EXPECTED_SUBSTRINGS="${EXPECTED_SUBSTRINGS:-$EXPECTED_TEXT}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-model-certification.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-model-certification.xml}"
REMOTE_UI_XML="/sdcard/knowme-local-model-certification.xml"

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$SCRIPT_DIR/verify-device-runtime-gates.sh" "$APK" "$PACKAGE"

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

tap_composer_fallback() {
  local width height
  read -r width height < <(screen_dimensions)
  adb shell input tap "$((width / 2))" "$((height - 240))"
}

tap_send_fallback() {
  local width height
  read -r width height < <(screen_dimensions)
  adb shell input tap "$((width - 72))" "$((height - 220))"
}

ui_contains_expected_substrings() {
  local expected
  IFS='|' read -r -a expected_items <<< "$EXPECTED_SUBSTRINGS"
  for expected in "${expected_items[@]}"; do
    if [[ -z "$expected" ]]; then
      continue
    fi
    if ! rg -F -q "$expected" "$UI_XML_PATH"; then
      return 1
    fi
  done
  return 0
}

adb logcat -c

if ! tap_matching_node "Ask KnowMe anything"; then
  if ! tap_matching_node "Chat"; then
    echo "Chat semantics not found; using bottom-navigation tap fallback" >&2
    tap_chat_fallback
  fi
  sleep 1
fi

if ! tap_matching_node "Ask KnowMe anything"; then
  echo "composer placeholder not found; using bottom-screen tap fallback" >&2
  tap_composer_fallback
fi

escaped_prompt="${PROMPT// /%s}"
adb shell input text "$escaped_prompt"

if ! tap_matching_node "Send message"; then
  echo "send semantics not found; using lower-right tap fallback" >&2
  tap_send_fallback
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  adb logcat -d -v time > "$LOG_PATH"
  tap_matching_node "Latest" >/dev/null 2>&1 || true
  dump_ui || true
  if rg -q "FATAL EXCEPTION|tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|dlopen failed.*libgen_ui_ffi|liblitertlm.*dlopen failed|dlopen failed.*liblitertlm|JNI_OnLoad.*failed" "$LOG_PATH"; then
    echo "G6 FAIL: fatal or KnowMe native-load failure found while certifying local model prompt" >&2
    rg -n "FATAL EXCEPTION|tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|dlopen failed.*libgen_ui_ffi|liblitertlm.*dlopen failed|dlopen failed.*liblitertlm|JNI_OnLoad.*failed" "$LOG_PATH" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi
  if rg -q "LiteRT-LM generation starting" "$LOG_PATH" &&
     rg -q "LiteRT-LM generation produced text" "$LOG_PATH" &&
     ui_contains_expected_substrings; then
    echo "G5 PASS: LiteRT-LM generation started"
    echo "G6 PASS: LiteRT-LM produced text"
    echo "G6 PASS: expected visible UI text found: $EXPECTED_SUBSTRINGS"
    echo "prompt: $PROMPT"
    echo "logcat: $LOG_PATH"
    echo "ui xml: $UI_XML_PATH"
    exit 0
  fi
  sleep 2
done

echo "G6 FAIL: timed out waiting for LiteRT-LM generation logs and visible expected text" >&2
echo "expected visible text: $EXPECTED_TEXT" >&2
echo "prompt: $PROMPT" >&2
echo "logcat: $LOG_PATH" >&2
echo "ui xml: $UI_XML_PATH" >&2
exit 1
