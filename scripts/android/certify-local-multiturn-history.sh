#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=240 CERT_CLEAN_CHAT_STATE=1" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
TOKEN="${TOKEN:-KMMULTI$(date +%H%M%S)}"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-multiturn-history.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-multiturn-history.xml}"
REMOTE_UI_XML="/sdcard/knowme-local-multiturn-history.xml"

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

if ! command -v adb >/dev/null 2>&1; then
  echo "adb not found in PATH" >&2
  exit 69
fi

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

if ! command -v rg >/dev/null 2>&1; then
  echo "rg not found in PATH" >&2
  exit 69
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found in PATH" >&2
  exit 69
fi

source "$SCRIPT_DIR/lib-chat-state.sh"
trap 'cert_restore_clean_chat_state "$PACKAGE"' EXIT

if [[ "$CERT_CLEAN_CHAT_STATE" == "1" ]]; then
  adb install -r "$APK" >/dev/null
fi
cert_prepare_clean_chat_state "$PACKAGE"

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

ui_contains_all_expected_substrings() {
  local expected
  IFS='|' read -r -a expected_items <<< "$1"
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

echo "Creating first local-model turn with marker: $TOKEN"
PROMPT="Reply exactly ${TOKEN}." \
  EXPECTED_SUBSTRINGS="$TOKEN" \
  TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
  LOG_PATH="/tmp/knowme-local-multiturn-seed.log" \
  UI_XML_PATH="/tmp/knowme-local-multiturn-seed.xml" \
  CERT_CLEAN_CHAT_STATE=0 \
  "$PROMPT_GATE" "$APK" "$TOKEN" "$PACKAGE"

if ! tap_matching_node "Ask KnowMe anything"; then
  if ! tap_matching_node "Chat"; then
    tap_chat_fallback
  fi
  sleep 1
fi

if ! tap_matching_node "Ask KnowMe anything"; then
  tap_composer_fallback
fi

adb logcat -c
SECOND_PROMPT="Read the immediately previous assistant message It contains one all caps codeword beginning with KMMULTI Reply with exactly that codeword then HISTORYOK Do not explain"
adb shell input text "${SECOND_PROMPT// /%s}"

if ! tap_matching_node "Send message"; then
  tap_send_fallback
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  adb logcat -d -v time > "$LOG_PATH"
  tap_matching_node "Latest" >/dev/null 2>&1 || true
  dump_ui || true

  if [[ -z "$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)" ]]; then
    echo "MULTITURN FAIL: app process is not alive" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi

  if rg -q "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|liblitertlm.*dlopen failed|JNI_OnLoad.*failed" "$LOG_PATH"; then
    echo "MULTITURN FAIL: KnowMe native-load failure found" >&2
    rg -n "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|liblitertlm.*dlopen failed|JNI_OnLoad.*failed" "$LOG_PATH" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi

  if rg -q "LiteRT-LM generation starting" "$LOG_PATH" &&
     rg -q "LiteRT-LM generation produced text" "$LOG_PATH" &&
     ui_contains_all_expected_substrings "${TOKEN}|HISTORYOK"; then
    echo "MULTITURN PASS: second turn recovered prior marker from chat history"
    echo "marker: $TOKEN"
    echo "prompt: $SECOND_PROMPT"
    echo "logcat: $LOG_PATH"
    echo "ui xml: $UI_XML_PATH"
    exit 0
  fi

  sleep 2
done

echo "MULTITURN FAIL: timed out waiting for second-turn history evidence: ${TOKEN}|HISTORYOK" >&2
echo "prompt: $SECOND_PROMPT" >&2
echo "logcat: $LOG_PATH" >&2
echo "ui xml: $UI_XML_PATH" >&2
exit 1
