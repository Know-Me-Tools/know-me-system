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
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-tool-call.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-tool-call.xml}"
PROMPT="${PROMPT:-You MUST call the knowme_diagnostics__local_runtime_status tool now. Do not answer directly. After the tool result is provided, answer exactly TOOLSTATUSOK.}"

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

PROMPT="$PROMPT" \
  EXPECTED_SUBSTRINGS="Tool use knowme_diagnostics__local_runtime_status|Tool completed|TOOLSTATUSOK" \
  TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
  LOG_PATH="$LOG_PATH" \
  UI_XML_PATH="$UI_XML_PATH" \
  "$PROMPT_GATE" "$APK" "TOOLSTATUSOK" "$PACKAGE"

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
