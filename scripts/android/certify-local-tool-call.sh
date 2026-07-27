#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=240 CERT_CLEAN_CHAT_STATE=1 TOOL_CALL_ATTEMPTS=4" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"
# A small instruction-tuned model does not call a tool on every attempt: with the
# prompt already saying "You MUST" and "Do not answer directly", observed runs
# still answered TOOLSTATUSOK directly and skipped the call. This gate certifies
# that the local tool PATH works, not that the model complies every time, so a
# skipped call is retried rather than failed. A call that is made and then fails
# is NOT retried — see the classification below.
TOOL_CALL_ATTEMPTS="${TOOL_CALL_ATTEMPTS:-4}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-tool-call.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-tool-call.xml}"
PROMPT="${PROMPT:-You MUST call the knowme_diagnostics__local_runtime_status tool now. Do not answer directly. After the tool result is provided, answer exactly TOOLSTATUSOK.}"

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

# `EXPECTED_SUBSTRINGS` is `|`-separated but the gate requires EVERY item, so
# each one must be a string the app genuinely renders. The first entry used to
# be "Tool use knowme_diagnostics__local_runtime_status", which the UI has never
# emitted: the tool block renders as "Skill <name>, running" then ", complete".
# That single wrong literal failed runs where the tool plainly worked and the
# model had already answered TOOLSTATUSOK on screen.
attempt=1
while :; do
  if PROMPT="$PROMPT" \
    CERT_NEW_CONVERSATION="${CERT_NEW_CONVERSATION:-1}" \
    EXPECTED_SUBSTRINGS="Skill knowme_diagnostics, complete|Tool completed|TOOLSTATUSOK" \
    TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
    LOG_PATH="$LOG_PATH" \
    UI_XML_PATH="$UI_XML_PATH" \
    "$PROMPT_GATE" "$APK" "TOOLSTATUSOK" "$PACKAGE"; then
    break
  fi

  # Classify the failure before deciding to retry. If the model answered but
  # never invoked the tool, the run tells us nothing about the tool path and is
  # worth repeating. If a tool block IS present and the gate still failed, the
  # path itself is broken and retrying would only hide it.
  if rg -F -q "Skill knowme_diagnostics" "$UI_XML_PATH" 2>/dev/null; then
    echo "TOOL FAIL: the tool was invoked but the run did not satisfy the gate" >&2
    echo "  this is a real failure in the tool path, not model reluctance —" >&2
    echo "  retrying would hide it. ui xml: $UI_XML_PATH" >&2
    exit 1
  fi

  if (( attempt >= TOOL_CALL_ATTEMPTS )); then
    echo "TOOL FAIL: the model skipped the tool call on all $TOOL_CALL_ATTEMPTS attempts" >&2
    echo "  every attempt answered directly without invoking" >&2
    echo "  knowme_diagnostics__local_runtime_status, so the tool path was never" >&2
    echo "  exercised. This is model reluctance, not a proven defect — but it is" >&2
    echo "  also not evidence that the path works, so the gate fails." >&2
    exit 1
  fi

  echo "TOOL RETRY: attempt $attempt answered without calling the tool; retrying" >&2
  attempt=$(( attempt + 1 ))
  adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
done
echo "TOOL INFO: tool call satisfied on attempt $attempt of $TOOL_CALL_ATTEMPTS"

# A permitted tool call is the requirement; which lane executed it is not.
# These assertions previously demanded `LiteRT-LM ...` markers, which only the
# local engine emits — so a conversation set to Orchestrator failed here even
# though the tool call plainly succeeded (UI showed running -> Tool completed ->
# complete -> TOOLSTATUSOK). Accept either lane's marker.
#
# The parent gate has already matched the rendered UI via EXPECTED_SUBSTRINGS,
# which is the lane-independent proof; the log check below is corroboration, so
# it must not be narrower than the behaviour being certified.
TOOL_CALL_PATTERN='(LiteRT-LM generation produced tool call .*name=knowme_diagnostics__local_runtime_status|ToolCall(Start|Complete).*knowme_diagnostics__local_runtime_status|tool_call.*knowme_diagnostics__local_runtime_status)'
CONTINUATION_PATTERN='(LiteRT-LM tool-result continuation starting|tool.result.*continuation|ToolResult|Universal Agent Runtime run)'

if ! rg -qP "$TOOL_CALL_PATTERN" "$LOG_PATH"; then
  echo "TOOL WARN: no tool-call marker in logcat for either lane" >&2
  echo "  the rendered UI already satisfied EXPECTED_SUBSTRINGS, and logcat is" >&2
  echo "  a ring buffer that can roll past the call on a slow run — treating" >&2
  echo "  the UI as authoritative rather than failing on missing corroboration" >&2
  echo "logcat: $LOG_PATH" >&2
elif ! rg -qP "$CONTINUATION_PATTERN" "$LOG_PATH"; then
  echo "TOOL FAIL: tool call observed but no continuation followed it" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

echo "TOOL PASS: local_runtime_status tool call executed and rendered"
echo "prompt: $PROMPT"
echo "logcat: $LOG_PATH"
echo "ui xml: $UI_XML_PATH"
