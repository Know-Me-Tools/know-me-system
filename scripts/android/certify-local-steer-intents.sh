#!/usr/bin/env bash
# Certify the four explicit intents on a stopped turn: Steer, Resume, Retry, Discard.
#
# Stopping already preserves partial work. What this gates is INTENT: that the
# stopped turn offers the four distinct ways forward, that the two which keep the
# user's work lead, and that Resume actually continues the same thought instead of
# restarting — the thing a widget test cannot prove because it depends on the real
# model seeing the cancelled partial turn in its history.
#
# Depends on certify-local-second-send.sh's fix: Resume sends AFTER a cancel, so a
# poisoned engine would make this untestable.
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=300 CANCEL_AFTER_GENERATION_SECONDS=6" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-300}"
CANCEL_AFTER_GENERATION_SECONDS="${CANCEL_AFTER_GENERATION_SECONDS:-6}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-steer-intents.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-steer-intents.xml}"
REMOTE_UI_XML="/sdcard/knowme-local-steer-intents.xml"
PROMPT="${PROMPT:-Tell me a long continuous story about a lighthouse keeper. Write at least 2000 words of flowing prose and do not stop early.}"

for tool in adb rg python3; do
  command -v "$tool" >/dev/null 2>&1 || { echo "$tool not found in PATH" >&2; exit 69; }
done
[[ -f "$APK" ]] || { echo "APK not found: $APK" >&2; exit 66; }

source "$SCRIPT_DIR/lib-chat-state.sh"
trap 'cert_restore_clean_chat_state "$PACKAGE"; cert_exit_offline' EXIT

"$SCRIPT_DIR/verify-device-runtime-gates.sh" "$APK" "$PACKAGE"

if [[ "${CERT_CLEAN_CHAT_STATE:-1}" == "1" ]]; then
  cert_enter_offline || exit 1
cert_prepare_clean_chat_state "$PACKAGE"
  adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
  sleep "${CERT_CLEAN_CHAT_RELAUNCH_WAIT_SECONDS:-8}"
fi

dump_ui() {
  # `uiautomator dump` fails with "could not get idle state" while the app is
  # animating — which is EXACTLY when a streaming run needs snapshotting. The
  # old `|| true` form left a stale XML behind, so a later grep read the
  # previous screen as if it were the current one and the Stop control appeared
  # to be missing. Retry, then fail loudly rather than lying.
  local attempt=0
  while (( attempt < 6 )); do
    attempt=$((attempt + 1))
    adb shell rm -f "$REMOTE_UI_XML" >/dev/null 2>&1
    if adb shell uiautomator dump "$REMOTE_UI_XML" >/dev/null 2>&1 \
      && adb exec-out cat "$REMOTE_UI_XML" > "$UI_XML_PATH" 2>/dev/null \
      && [[ -s "$UI_XML_PATH" ]] \
      && python3 -c "import sys,xml.etree.ElementTree as ET; ET.parse(sys.argv[1])" \
           "$UI_XML_PATH" 2>/dev/null; then
      return 0
    fi
    sleep 1
  done
  echo "UI dump failed after $attempt attempts (app never went idle)" >&2
  return 1
}

node_point() {
  python3 - "$UI_XML_PATH" "$1" <<'PY'
import re
import sys
import xml.etree.ElementTree as ET

path, needle = sys.argv[1], sys.argv[2].lower()
root = ET.parse(path).getroot()
for node in root.iter("node"):
    haystack = " ".join(
        (node.attrib.get(key) or "").lower()
        for key in ("text", "content-desc", "hint")
    )
    if needle not in haystack:
        continue
    match = re.search(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]", node.attrib.get("bounds", ""))
    if not match:
        continue
    x1, y1, x2, y2 = map(int, match.groups())
    print(f"{(x1 + x2) // 2} {(y1 + y2) // 2}")
    raise SystemExit(0)
raise SystemExit(1)
PY
}

tap_matching_node() {
  dump_ui
  local point
  point="$(node_point "$1")" || return 1
  [[ -n "$point" ]] || return 1
  adb shell input tap $point
}

# Tap the composer by class, not by placeholder: the hint becomes "Steer the
# stopped response…" once a steer is armed.
tap_composer() {
  dump_ui
  local point
  point="$(python3 - "$UI_XML_PATH" <<'PY'
import re
import sys
import xml.etree.ElementTree as ET

root = ET.parse(sys.argv[1]).getroot()
for node in root.iter("node"):
    if node.attrib.get("class") != "android.widget.EditText":
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
  [[ -n "$point" ]] || return 1
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

current_lane() {
  dump_ui || true
  python3 - "$UI_XML_PATH" <<'PY'
import re
import sys
import xml.etree.ElementTree as ET

root = ET.parse(sys.argv[1]).getroot()
for node in root.iter("node"):
    desc = (node.attrib.get("content-desc") or "").strip()
    if desc not in ("Orchestrator", "Agent", "Local"):
        continue
    match = re.search(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]", node.attrib.get("bounds", ""))
    if not match:
        continue
    x1, y1, x2, y2 = map(int, match.groups())
    print(desc, (x1 + x2) // 2, (y1 + y2) // 2)
    raise SystemExit(0)
raise SystemExit(1)
PY
}

select_local_lane() {
  local lane x y
  read -r lane x y < <(current_lane) || {
    echo "INTENTS FAIL: mode trigger not found" >&2
    exit 1
  }
  if [[ "$lane" == "Local" ]]; then
    echo "lane: Local (already selected)"
    return
  fi
  echo "lane: $lane — switching to Local"
  adb shell input tap "$x" "$y"
  sleep 2
  tap_matching_node "Local, On-device model" || {
    echo "INTENTS FAIL: no Local option in the mode sheet" >&2
    exit 1
  }
  sleep 3
  read -r lane x y < <(current_lane) || true
  [[ "$lane" == "Local" ]] || {
    echo "INTENTS FAIL: lane is '$lane'; refusing to certify another lane" >&2
    exit 1
  }
  echo "lane: Local (confirmed)"
}

open_chat() {
  tap_composer && return
  tap_matching_node "Chat" || {
    local width height
    read -r width height < <(screen_dimensions)
    adb shell input tap "$((width * 3 / 10))" "$((height - 220))"
  }
  sleep 2
}

adb logcat -c
APP_PID="$(adb shell pidof "$PACKAGE" | tr -d '\r' | awk '{print $1}')"
[[ -n "$APP_PID" ]] || { echo "INTENTS FAIL: no app pid" >&2; exit 1; }
capture_log() { adb logcat -d -v time --pid "$APP_PID" > "$LOG_PATH"; }
echo "app pid: $APP_PID"

open_chat
select_local_lane

echo "== generate, then stop mid-stream"
tap_composer || { echo "INTENTS FAIL: composer not found" >&2; exit 1; }
adb shell input text "${PROMPT// /%s}"
sleep 1
dump_ui || { echo "INTENTS FAIL: could not read the composer row" >&2; exit 1; }
SEND_NOW="$(node_point "Send message")" || {
  echo "INTENTS FAIL: send control not found" >&2
  exit 1
}
adb shell input tap $SEND_NOW

# Measure the Stop slot AFTER sending, not before.
#
# The composer grows to 5 lines while holding the long prompt, which lifts the
# button ~480px; clearing the text on send collapses it back down. A slot read
# before the send therefore points at empty space by the time Stop needs
# tapping, and the run finishes uncancelled. There is a brief window right after
# the send — before the first token arrives and the app starts animating
# continuously — in which `uiautomator dump` still succeeds.
sleep 2
dump_ui || { echo "INTENTS FAIL: could not read the collapsed composer row" >&2; exit 1; }
STOP_SLOT="$(node_point "Stop generating")" || STOP_SLOT="$(node_point "Send message")" || {
  echo "INTENTS FAIL: send/stop slot not found after sending" >&2
  exit 1
}
echo "stop slot: $STOP_SLOT (send was at $SEND_NOW)"

deadline=$((SECONDS + TIMEOUT_SECONDS))
stopped=0
while (( SECONDS < deadline )); do
  capture_log
  if (( stopped == 0 )) && rg -q "generation produced text chars=" "$LOG_PATH"; then
    sleep "$CANCEL_AFTER_GENERATION_SECONDS"
    adb shell input tap $STOP_SLOT
    stopped=1
    stop_tapped_at=$SECONDS
  fi
  if (( stopped == 1 )) && rg -q "streaming stopped by user" "$LOG_PATH"; then
    break
  fi
  # A tap that misses leaves the run to finish on its own. Catch that here
  # instead of letting the missing kept-char count report it obliquely.
  if (( stopped == 1 )) && rg -q "generation timing generate_ms=" "$LOG_PATH"; then
    echo "INTENTS FAIL: the run completed instead of stopping; the Stop tap missed" >&2
    echo "tapped $STOP_SLOT at t+${stop_tapped_at}s" >&2
    echo "logcat: $LOG_PATH" >&2
    exit 1
  fi
  sleep 2
done
(( stopped == 1 )) || { echo "INTENTS FAIL: never reached a mid-stream stop" >&2; exit 1; }

kept="$(rg -N "streaming stopped by user" "$LOG_PATH" | rg -o 'chars=\d+' | rg -o '\d+' | tail -1 || true)"
[[ -n "$kept" ]] || { echo "INTENTS FAIL: no kept-char count on the stop" >&2; exit 1; }
echo "stopped mid-stream; kept chars=$kept"

echo "== the stopped turn offers four explicit intents"
sleep 3
# The bar sits BELOW the (long) assistant message, and four actions wrap onto two
# rows on a phone, so part of it is off-screen after a run. Use the app's own
# scroll-to-bottom control rather than guessing swipe distances — hand-rolled
# swipes overshot in both directions during development.
scroll_attempt=0
while (( scroll_attempt < 8 )); do
  dump_ui || { echo "INTENTS FAIL: UI dump failed while locating the bar" >&2; exit 1; }
  if node_point "Steer from here" >/dev/null 2>&1; then
    break
  fi
  scroll_attempt=$((scroll_attempt + 1))
  if latest="$(node_point "Latest")"; then
    adb shell input tap $latest
  else
    # Already at the bottom: nudge upward to reveal the wrapped first row.
    adb shell input swipe 540 1400 540 1250 250
  fi
  sleep 2
done
dump_ui
for action in Steer Resume Retry Discard; do
  node_point "$action" >/dev/null || {
    echo "INTENTS FAIL: '$action' is missing from the stopped turn" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  }
  echo "  present: $action"
done
# The kept-count line is the reassurance that the work survived; without it the
# four actions are just buttons with no stated consequence.
rg -F -q "still in context" "$UI_XML_PATH" || {
  echo "INTENTS FAIL: the stopped turn does not say the work is still in context" >&2
  exit 1
}
echo "  present: kept-work reassurance"

echo "== Resume continues rather than restarting"
starts_before="$(rg -c "LiteRT-LM generation starting" "$LOG_PATH" 2>/dev/null || echo 0)"
log_lines_before="$(wc -l < "$LOG_PATH" | tr -d ' ')"
tap_matching_node "Resume" || { echo "INTENTS FAIL: Resume not tappable" >&2; exit 1; }

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  capture_log
  if tail -n "+$((log_lines_before + 1))" "$LOG_PATH" | rg -q "CANCELLED: Task cancelled"; then
    echo "INTENTS FAIL: Resume hit a poisoned engine" >&2
    exit 1
  fi
  if (( "$(rg -c "LiteRT-LM generation starting" "$LOG_PATH" 2>/dev/null || echo 0)" > starts_before )); then
    produced="$(tail -n "+$((log_lines_before + 1))" "$LOG_PATH" \
      | rg -o "generation timing generate_ms=\d+ chars=\d+" \
      | rg -o "chars=\d+" | rg -o "\d+" | tail -1 || true)"
    if [[ -n "$produced" && "$produced" -gt 0 ]]; then
      # The model must have SEEN the cancelled partial turn. History replay is
      # what makes Resume work without any Rust change, so assert the assistant
      # partial is in the request rather than trusting it.
      history="$(tail -n "+$((log_lines_before + 1))" "$LOG_PATH" \
        | rg -N "local chat history received for context assembly" | tail -1)"
      echo "$history" | rg -q "assistant:" || {
        echo "INTENTS FAIL: Resume sent no assistant turn in history, so there was nothing to continue from" >&2
        echo "$history" >&2
        exit 1
      }
      echo "RESUME PASS: continued after the stop chars=$produced"
      echo "$history" | rg -o "history_summary=\S+" | tail -1
      echo "logcat: $LOG_PATH"
      echo
      echo "INTENTS PASS: four intents present, work kept ($kept chars), Resume continued"
      exit 0
    fi
    if [[ -n "$produced" && "$produced" -eq 0 ]]; then
      echo "INTENTS FAIL: Resume started but produced chars=0" >&2
      exit 1
    fi
  fi
  sleep 2
done

echo "INTENTS FAIL: timed out waiting for Resume to produce text" >&2
echo "logcat: $LOG_PATH" >&2
exit 1
