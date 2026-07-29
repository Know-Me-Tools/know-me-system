#!/usr/bin/env bash
# Certify that a send AFTER a cancel still generates text.
#
# `cancelProcess()` poisons state inside the native LiteRT-LM engine that
# outlives the Kotlin `Conversation` wrapper: before the engine-rebuild fix,
# the turn after a stop failed immediately with
# "LiteRtLmJniException: ... CANCELLED: Task cancelled" and produced zero
# chunks. The single-cancel certification (certify-local-cancellation.sh)
# passes either way, because it stops looking once the first stop lands — so
# this second cycle is the only thing that actually gates the fix.
#
# PASS requires, after the stop: a fresh "LiteRT-LM generation starting" and
# streamed characters, with no CANCELLED exception in between.
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=240 CANCEL_AFTER_GENERATION_SECONDS=6" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"
CANCEL_AFTER_GENERATION_SECONDS="${CANCEL_AFTER_GENERATION_SECONDS:-6}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-second-send.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-second-send.xml}"
REMOTE_UI_XML="/sdcard/knowme-local-second-send.xml"
# Phrased to elicit plain prose, not a tool call. An earlier wording ("field
# guide about local-first personal productivity") made the model reach for
# knowme_diagnostics__local_runtime_status instead, which routes the turn down
# the non-streaming sendToolResults path — so the stop never exercised a
# mid-stream cancel and no partial text could be kept.
PROMPT1="${PROMPT1:-Tell me a long continuous story about a lighthouse keeper. Write at least 2000 words of flowing prose and do not stop early.}"
PROMPT2="${PROMPT2:-Name three colors.}"

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
  if [[ -z "$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)" ]]; then
    echo "SECOND-SEND FAIL: app process is not alive after clean chat-state relaunch" >&2
    exit 1
  fi
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

# Centre point of the first node whose text/content-desc/hint contains $1,
# read from the LAST dump. Separate from tapping so a slot can be measured now
# and tapped later — necessary because `uiautomator dump` stops working once the
# app animates continuously during streaming.
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
  dump_ui || return 1
  local point
  point="$(node_point "$1")" || return 1
  [[ -n "$point" ]] || return 1
  adb shell input tap $point
}

# Tap the composer by widget class, not by its placeholder text.
#
# The hint changes to "Steer the stopped response…" once a steer is armed, so
# matching on "Ask KnowMe anything" silently misses the composer in exactly the
# state this feature introduces.
tap_composer() {
  dump_ui
  local point
  point="$(python3 - "$UI_XML_PATH" <<'PYX'
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
PYX
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

# Read the conversation's current lane off the mode trigger.
#
# The trigger sits in the "Conversation agent and context controls" row and its
# content-desc IS the lane name ("Orchestrator" / "Agent" / "Local").
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

# Switch the conversation to the Local lane, then prove it took.
#
# The lane MUST be confirmed before any chars/latency number means anything: an
# earlier run silently measured the Orchestrator lane and reported chars=0,
# which read as a streaming failure and was not one.
select_local_lane() {
  local lane x y
  read -r lane x y < <(current_lane) || {
    echo "SECOND-SEND FAIL: mode trigger not found on the chat screen" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  }
  if [[ "$lane" == "Local" ]]; then
    echo "lane: Local (already selected)"
    return
  fi
  echo "lane: $lane — switching to Local"
  adb shell input tap "$x" "$y"
  sleep 2
  # Match the option's own label, not a bare "Local": the sheet's explanatory
  # blurb ("Agents run through the Universal Agent Runtime. Local...") also
  # contains the word and sits ABOVE the options, so a substring match taps the
  # description and silently leaves the lane unchanged.
  tap_matching_node "Local, On-device model" || {
    echo "SECOND-SEND FAIL: no Local option in the mode sheet" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  }
  sleep 3
  read -r lane x y < <(current_lane) || true
  if [[ "$lane" != "Local" ]]; then
    echo "SECOND-SEND FAIL: lane is '$lane' after selecting Local; refusing to certify another lane" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  fi
  echo "lane: Local (confirmed)"
}

# Navigate to the chat surface. A fresh launch lands on Home, where the mode
# trigger does not exist yet — checking the lane before this always fails.
open_chat() {
  if tap_composer; then
    return
  fi
  tap_matching_node "Chat" || {
    local width height
    read -r width height < <(screen_dimensions)
    adb shell input tap "$((width * 3 / 10))" "$((height - 220))"
  }
  sleep 2
}

compose_and_send() {
  local prompt="$1"
  if ! tap_composer; then
    local width height
    read -r width height < <(screen_dimensions)
    adb shell input tap "$((width / 2))" "$((height - 240))"
  fi
  adb shell input text "${prompt// /%s}"
  sleep 1
  tap_matching_node "Send message" || {
    echo "SECOND-SEND FAIL: Send control not found" >&2
    echo "ui xml: $UI_XML_PATH" >&2
    exit 1
  }
  # Measure the stop slot NOW, in the brief window after the send and before the
  # first token. Once tokens stream the app animates continuously and
  # `uiautomator dump` fails with "could not get idle state" for the rest of the
  # run, so Stop cannot be located later. It must also be measured AFTER the
  # send: the composer shrinks from 5 lines to 1 when the text clears, moving the
  # button ~480px down.
  sleep 2
  if dump_ui; then
    STOP_SLOT="$(node_point "Stop generating")" \
      || STOP_SLOT="$(node_point "Send message")" \
      || STOP_SLOT=""
  else
    STOP_SLOT=""
  fi
  # Confirm the send actually took. A tap that lands but does not submit leaves
  # the prompt sitting in the composer, and the run then reads whatever the
  # previous process left in the log as if it were this turn's output.
  #
  # Checked against the EditText's own text, not the whole dump: once sent, the
  # prompt also appears in the transcript as a user bubble, so a document-wide
  # match can never clear.
  local waited=0
  while (( waited < 20 )); do
    sleep 2
    waited=$((waited + 2))
    dump_ui || true
    if ! composer_holds_text "$UI_XML_PATH"; then
      return
    fi
  done
  echo "SECOND-SEND FAIL: the prompt is still in the composer; the send did not submit" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
}

# True when the composer EditText still contains user-typed text.
composer_holds_text() {
  python3 - "$1" <<'PY'
import sys
import xml.etree.ElementTree as ET

root = ET.parse(sys.argv[1]).getroot()
for node in root.iter("node"):
    if node.attrib.get("class") != "android.widget.EditText":
        continue
    if (node.attrib.get("text") or "").strip():
        raise SystemExit(0)
raise SystemExit(1)
PY
}

count_generation_starts() {
  rg -c "LiteRT-LM generation starting" "$LOG_PATH" 2>/dev/null || echo 0
}

# Clear AFTER the gate script and the clean-state relaunch: both drive the app.
adb logcat -c

# Clearing alone is NOT enough. The gate script launches the app itself, so a
# previous process can log after the clear, and its streamed chunks then read as
# this run's output — that is how an uncancelled 11992-char run once reported a
# pass. Every log read is therefore scoped to the pid alive right now.
APP_PID="$(adb shell pidof "$PACKAGE" | tr -d '\r' | awk '{print $1}')"
if [[ -z "$APP_PID" ]]; then
  echo "SECOND-SEND FAIL: could not resolve the app pid" >&2
  exit 1
fi
echo "app pid: $APP_PID"

capture_log() {
  adb logcat -d -v time --pid "$APP_PID" > "$LOG_PATH"
}

open_chat
select_local_lane

echo "== cycle 1: generate, then stop mid-stream"
compose_and_send "$PROMPT1"

deadline=$((SECONDS + TIMEOUT_SECONDS))
cancel_tapped=0
cancel_confirmed=0
while (( SECONDS < deadline )); do
  capture_log
  # Wait for STREAMED TEXT, not "generation starting". Prefill takes ~8s on this
  # device, and the Stop control does not exist until the first chunk lands — an
  # earlier version keyed off "starting", missed the control, fell back to a
  # blind lower-right tap, and let the run finish uncancelled (11992 chars) while
  # still reporting a pass.
  if (( cancel_tapped == 0 )) && rg -q "generation produced text chars=" "$LOG_PATH"; then
    sleep "$CANCEL_AFTER_GENERATION_SECONDS"
    [[ -n "$STOP_SLOT" ]] || {
      echo "SECOND-SEND FAIL: no stop slot was measured; refusing to blind-tap" >&2
      exit 1
    }
    adb shell input tap $STOP_SLOT
    cancel_tapped=1
  fi
  if (( cancel_tapped == 1 )) && rg -q "streaming stopped by user|generation cancelled by user|Cancelled by user" "$LOG_PATH"; then
    cancel_confirmed=1
    break
  fi
  sleep 2
done

if (( cancel_tapped == 0 )); then
  echo "SECOND-SEND FAIL: generation never streamed text, so no cancel was exercised" >&2
  echo "logcat: $LOG_PATH" >&2
  exit 1
fi

# Without a confirmed cancel the engine is never poisoned, so cycle 2 would pass
# for the wrong reason. This is the assertion that keeps the certification honest.
if (( cancel_confirmed == 0 )); then
  echo "SECOND-SEND FAIL: Stop was tapped but no cancellation was recorded" >&2
  echo "logcat: $LOG_PATH" >&2
  exit 1
fi

capture_log
starts_after_cancel="$(count_generation_starts)"
# The streaming path logs the running char count with the stop; the blocking
# fallback has none to report, which is itself the signal that no partial text
# survived.
# The field may render before or after the message depending on the tracing
# layer, so match the whole line and pull the chars= value out of it.
kept_chars="$(rg -N "streaming stopped by user" "$LOG_PATH" \
  | rg -o 'chars=\d+' | rg -o '\d+' | tail -1 || true)"
if [[ -z "$kept_chars" ]]; then
  echo "SECOND-SEND FAIL: the stop kept no streamed text (blocking path, nothing preserved)" >&2
  echo "logcat: $LOG_PATH" >&2
  exit 1
fi
echo "cycle 1 stopped mid-stream; kept chars=$kept_chars"

echo "== cycle 2: send again on the same conversation"
sleep 2
# Anchor the failure search: everything before this point belongs to cycle 1,
# whose cancel legitimately logs a CANCELLED on the blocking tool-result path.
log_lines_before_cycle2="$(wc -l < "$LOG_PATH" | tr -d ' ')"
compose_and_send "$PROMPT2"

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  capture_log

  if tail -n "+$((log_lines_before_cycle2 + 1))" "$LOG_PATH" \
      | rg -q "CANCELLED: Task cancelled"; then
    echo "SECOND-SEND FAIL: the engine is still poisoned after a cancel" >&2
    tail -n "+$((log_lines_before_cycle2 + 1))" "$LOG_PATH" \
      | rg -n "CANCELLED: Task cancelled" | tail -3 >&2
    echo "logcat: $LOG_PATH" >&2
    exit 1
  fi

  if (( "$(count_generation_starts)" > starts_after_cancel )); then
    # Total chars for the post-cancel turn, taken from its timing line. A
    # generation that starts and reports chars=0 is the poisoned-engine
    # signature, so the count is the assertion, not the start.
    produced="$(tail -n "+$((log_lines_before_cycle2 + 1))" "$LOG_PATH" \
      | rg -o "generation timing generate_ms=\d+ chars=\d+" \
      | rg -o "chars=\d+" | rg -o "\d+" | tail -1 || true)"
    if [[ -n "$produced" && "$produced" -gt 0 ]]; then
      echo "SECOND-SEND PASS: the turn after a cancel generated text chars=$produced"
      tail -n "+$((log_lines_before_cycle2 + 1))" "$LOG_PATH" \
        | rg -n "rebuilding LiteRT-LM engine after cancellation" | tail -1 \
        || echo "note: no rebuild log line (engine did not need one)"
      tail -n "+$((log_lines_before_cycle2 + 1))" "$LOG_PATH" \
        | rg -n "generation starting|generation timing" | tail -4
      echo "logcat: $LOG_PATH"
      exit 0
    fi
    if [[ -n "$produced" && "$produced" -eq 0 ]]; then
      echo "SECOND-SEND FAIL: the post-cancel turn started but produced chars=0" >&2
      echo "logcat: $LOG_PATH" >&2
      exit 1
    fi
  fi

  sleep 2
done

echo "SECOND-SEND FAIL: timed out waiting for the post-cancel turn to produce text" >&2
echo "generation starts before/after: $starts_after_cancel / $(count_generation_starts)" >&2
echo "logcat: $LOG_PATH" >&2
exit 1
