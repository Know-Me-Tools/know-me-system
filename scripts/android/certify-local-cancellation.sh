#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=180 CANCEL_AFTER_GENERATION_SECONDS=0 CERT_CLEAN_CHAT_STATE=1" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-180}"
CANCEL_AFTER_GENERATION_SECONDS="${CANCEL_AFTER_GENERATION_SECONDS:-0}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-cancellation.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-cancellation.xml}"
REMOTE_UI_XML="/sdcard/knowme-local-cancellation.xml"
PROMPT="${PROMPT:-Write a long numbered field guide with 200 detailed items about local-first personal productivity. Keep going until complete.}"

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

source "$SCRIPT_DIR/lib-chat-state.sh"
trap 'cert_restore_clean_chat_state "$PACKAGE"' EXIT

"$SCRIPT_DIR/verify-device-runtime-gates.sh" "$APK" "$PACKAGE"

if [[ "$CERT_CLEAN_CHAT_STATE" == "1" ]]; then
  cert_prepare_clean_chat_state "$PACKAGE"
  adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
  sleep "${CERT_CLEAN_CHAT_RELAUNCH_WAIT_SECONDS:-8}"
  if [[ -z "$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)" ]]; then
    echo "CANCEL FAIL: app process is not alive after clean chat-state relaunch" >&2
    exit 1
  fi
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

tap_composer_fallback() {
  local width height
  read -r width height < <(screen_dimensions)
  adb shell input tap "$((width / 2))" "$((height - 240))"
}

tap_send_or_stop_fallback() {
  dump_ui || true
  local point
  point="$(python3 - "$UI_XML_PATH" <<'PY'
import re
import sys
import xml.etree.ElementTree as ET

root = ET.parse(sys.argv[1]).getroot()
candidates = []
for node in root.iter("node"):
    if node.attrib.get("class") != "android.widget.Button":
        continue
    if node.attrib.get("enabled") != "true":
        continue
    match = re.search(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]", node.attrib.get("bounds", ""))
    if not match:
        continue
    x1, y1, x2, y2 = map(int, match.groups())
    if y1 < 900:
        continue
    candidates.append((x2, y1, (x1 + x2) // 2, (y1 + y2) // 2))
if not candidates:
    raise SystemExit(1)
candidates.sort(reverse=True)
print(candidates[0][2], candidates[0][3])
PY
)" || {
    local width height
    read -r width height < <(screen_dimensions)
    adb shell input tap "$((width - 72))" "$((height * 3 / 4))"
    return
  }
  adb shell input tap $point
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

adb shell input text "${PROMPT// /%s}"

if ! tap_matching_node "Send message"; then
  echo "send semantics not found; using lower-right tap fallback" >&2
  tap_send_or_stop_fallback
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
generation_seen=0
cancel_tapped=0
while (( SECONDS < deadline )); do
  adb logcat -d -v time > "$LOG_PATH"
  dump_ui || true

  if [[ -z "$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)" ]]; then
    echo "CANCEL FAIL: app process is not alive" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi

  if rg -q "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|liblitertlm.*dlopen failed|JNI_OnLoad.*failed" "$LOG_PATH"; then
    echo "CANCEL FAIL: KnowMe native-load failure found" >&2
    rg -n "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|liblitertlm.*dlopen failed|JNI_OnLoad.*failed" "$LOG_PATH" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi

  if (( generation_seen == 0 )) && rg -q "LiteRT-LM generation starting" "$LOG_PATH"; then
    generation_seen=1
    sleep "$CANCEL_AFTER_GENERATION_SECONDS"
    if ! tap_matching_node "Stop generating"; then
      echo "stop semantics not found; using lower-right tap fallback" >&2
      tap_send_or_stop_fallback
    fi
    cancel_tapped=1
  fi

  if (( cancel_tapped == 1 )) && rg -F -q "Cancelled by user" "$UI_XML_PATH"; then
    echo "CANCEL PASS: run was cancelled through the visible Stop control"
    echo "prompt: $PROMPT"
    echo "logcat: $LOG_PATH"
    echo "ui xml: $UI_XML_PATH"
    exit 0
  fi

  if (( cancel_tapped == 1 )) && rg -q "run_cancelled|Cancelled by user" "$LOG_PATH"; then
    tap_matching_node "Latest" >/dev/null 2>&1 || true
    dump_ui || true
    if rg -F -q "Cancelled by user" "$UI_XML_PATH"; then
      echo "CANCEL PASS: run was cancelled through the visible Stop control"
      echo "prompt: $PROMPT"
      echo "logcat: $LOG_PATH"
      echo "ui xml: $UI_XML_PATH"
      exit 0
    fi
  fi

  sleep 2
done

echo "CANCEL FAIL: timed out waiting for visible cancellation" >&2
echo "generation_seen: $generation_seen" >&2
echo "cancel_tapped: $cancel_tapped" >&2
echo "prompt: $PROMPT" >&2
echo "logcat: $LOG_PATH" >&2
echo "ui xml: $UI_XML_PATH" >&2
exit 1
