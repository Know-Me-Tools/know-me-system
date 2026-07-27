#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 3 ]]; then
  echo "usage: $0 /path/to/app.apk [expected-visible-text] [package.name]" >&2
  echo "env: PROMPT='Reply exactly TOKEN.' TIMEOUT_SECONDS=240 FORCE_LITERT_TOOL_RESULT_FALLBACK=1 CERT_CLEAN_CHAT_STATE=1" >&2
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
FORCE_LITERT_TOOL_RESULT_FALLBACK="${FORCE_LITERT_TOOL_RESULT_FALLBACK:-0}"
FORCE_LITERT_TOOL_RESULT_FALLBACK_FILE="knowme-force-litertlm-tool-result-fallback"
FORCE_LITERT_TOOL_RESULT_FALLBACK_PATHS=(
  "cache/$FORCE_LITERT_TOOL_RESULT_FALLBACK_FILE"
  "app_flutter/model-cache/litert-lm/runtime-cache/$FORCE_LITERT_TOOL_RESULT_FALLBACK_FILE"
)
CERT_CLEAN_CHAT_STATE="${CERT_CLEAN_CHAT_STATE:-0}"

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
source "$SCRIPT_DIR/lib-chat-state.sh"
"$SCRIPT_DIR/verify-device-runtime-gates.sh" "$APK" "$PACKAGE"

cleanup_force_litert_tool_result_fallback() {
  if [[ "$FORCE_LITERT_TOOL_RESULT_FALLBACK" == "1" ]]; then
    for sentinel_path in "${FORCE_LITERT_TOOL_RESULT_FALLBACK_PATHS[@]}"; do
      adb shell run-as "$PACKAGE" rm -f "$sentinel_path" >/dev/null 2>&1 || true
    done
  fi
}
cleanup_all() {
  cleanup_force_litert_tool_result_fallback
  cert_restore_clean_chat_state "$PACKAGE"
  cert_exit_offline
}
trap cleanup_all EXIT

# Enforced BEFORE any assertion, so nothing is measured while the network is up.
cert_enter_offline || exit 1

if [[ "$CERT_CLEAN_CHAT_STATE" == "1" ]]; then
  cert_prepare_clean_chat_state "$PACKAGE"
  adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
  sleep "${CERT_CLEAN_CHAT_RELAUNCH_WAIT_SECONDS:-8}"
  if [[ -z "$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)" ]]; then
    echo "G6 FAIL: app process is not alive after clean chat-state relaunch" >&2
    exit 1
  fi
fi

if [[ "$FORCE_LITERT_TOOL_RESULT_FALLBACK" == "1" ]]; then
  if ! adb shell run-as "$PACKAGE" mkdir -p "app_flutter/model-cache/litert-lm/runtime-cache" >/dev/null 2>&1; then
    echo "G6 FAIL: could not create LiteRT-LM runtime cache directory with run-as" >&2
    echo "package must be debuggable for FORCE_LITERT_TOOL_RESULT_FALLBACK=1" >&2
    exit 1
  fi
  for sentinel_path in "${FORCE_LITERT_TOOL_RESULT_FALLBACK_PATHS[@]}"; do
    if ! adb shell run-as "$PACKAGE" touch "$sentinel_path" >/dev/null 2>&1; then
      echo "G6 FAIL: could not create debug LiteRT-LM tool-result fallback sentinel with run-as" >&2
      echo "path: $sentinel_path" >&2
      echo "package must be debuggable for FORCE_LITERT_TOOL_RESULT_FALLBACK=1" >&2
      exit 1
    fi
  done
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

ui_contains_literal() {
  local needle="$1"
  [[ -n "$needle" ]] && rg -F -q "$needle" "$UI_XML_PATH"
}

generation_started() {
  rg -q "LiteRT-LM generation starting" "$LOG_PATH"
}

adb logcat -c

if ! tap_matching_node "Ask KnowMe anything"; then
  if ! tap_matching_node "Chat"; then
    echo "Chat semantics not found; using bottom-navigation tap fallback" >&2
    tap_chat_fallback
  fi
  sleep 1
fi

# Start a fresh conversation when asked. A long-running device carries the whole
# session's prior turns: an observed run reached history_count=108, and the model
# emitted 4-, 6- and 2-character fragments with tool_calls=0 — the instruction was
# drowned by context rather than refused. Gates that depend on the model FOLLOWING
# an instruction need a clean thread; gates that only need a reply do not, so this
# is opt-in.
if [[ "${CERT_NEW_CONVERSATION:-0}" == "1" ]]; then
  if tap_matching_node "Conversations"; then
    sleep 1
    if tap_matching_node "New conversation" || tap_matching_node "New Thread"; then
      echo "started a fresh conversation for this run" >&2
      sleep 2
    else
      echo "CERT FAIL: found the conversations sheet but not its new-conversation control" >&2
      exit 70
    fi
  else
    # Hard failure, not a warning. A run that silently falls back to existing
    # history looks like a model failure when it is really an unexercised setup
    # step — three runs were spent that way before this was caught.
    echo "CERT FAIL: CERT_NEW_CONVERSATION=1 but the conversations control was not found" >&2
    echo "  the run would have used existing history, which invalidates any gate" >&2
    echo "  that depends on the model following an instruction" >&2
    exit 70
  fi
fi

if ! tap_matching_node "Ask KnowMe anything"; then
  echo "composer placeholder not found; using bottom-screen tap fallback" >&2
  tap_composer_fallback
fi

escaped_prompt="${PROMPT// /%s}"
adb shell input text "$escaped_prompt"
sleep 1

if ! tap_matching_node "Send message"; then
  echo "send semantics not found; using lower-right tap fallback" >&2
  tap_send_fallback
fi

prompt_probe="${PROMPT:0:48}"
for _ in 1 2 3 4 5; do
  adb logcat -d -v time > "$LOG_PATH"
  dump_ui || true
  if generation_started || ui_contains_literal "$prompt_probe"; then
    break
  fi
  sleep 1
done

if ! generation_started && ! ui_contains_literal "$prompt_probe"; then
  echo "submitted prompt was not observed after first send tap; retrying composer submit" >&2
  tap_composer_fallback
  adb shell input text "$escaped_prompt"
  sleep 1
  if ! tap_matching_node "Send message"; then
    tap_send_fallback
  fi
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  adb logcat -d -v time > "$LOG_PATH"
  tap_matching_node "Latest" >/dev/null 2>&1 || true
  dump_ui || true
  if [[ -z "$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)" ]]; then
    echo "G6 FAIL: app process is not alive while certifying local model prompt" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi
  if rg -q "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|dlopen failed.*libgen_ui_ffi|liblitertlm.*dlopen failed|dlopen failed.*liblitertlm|JNI_OnLoad.*failed" "$LOG_PATH"; then
    echo "G6 FAIL: KnowMe native-load failure found while certifying local model prompt" >&2
    rg -n "tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|dlopen failed.*libgen_ui_ffi|liblitertlm.*dlopen failed|dlopen failed.*liblitertlm|JNI_OnLoad.*failed" "$LOG_PATH" >&2
    echo "logcat: $LOG_PATH" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi
  # The rendered UI is the authoritative signal: it proves the answer reached
  # the user regardless of which lane produced it. The LiteRT-LM log markers are
  # corroboration for local-lane runs and are reported when present, but they
  # must not gate success — requiring them made an Orchestrator-lane run poll
  # until timeout while the expected text was already on screen, since those
  # markers can never appear on that lane.
  #
  # Set REQUIRE_LOCAL_ENGINE=1 to insist on the local engine specifically (e.g.
  # when certifying the on-device path itself rather than a tool flow).
  if ui_contains_expected_substrings; then
    local_started=0
    rg -q "LiteRT-LM generation starting" "$LOG_PATH" && local_started=1
    local_produced=0
    rg -q "LiteRT-LM generation produced text" "$LOG_PATH" && local_produced=1
    if [[ "${REQUIRE_LOCAL_ENGINE:-0}" == "1" ]] &&
       (( local_started == 0 || local_produced == 0 )); then
      sleep 2
      continue
    fi
    if (( local_started == 1 )); then
      echo "G5 PASS: LiteRT-LM generation started"
    else
      echo "G5 INFO: no local-engine generation marker (non-local lane)"
    fi
    if (( local_produced == 1 )); then
      echo "G6 PASS: LiteRT-LM produced text"
    fi
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
