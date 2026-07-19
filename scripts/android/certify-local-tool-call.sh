#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=240" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-tool-call.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-tool-call.xml}"
PROMPT="${PROMPT:-Use the knowme_diagnostics__local_runtime_status tool to inspect this device runtime. After the tool result, answer TOOLSTATUSOK.}"

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

if ! rg -q "LiteRT-LM generation produced tool call .*name=knowme_diagnostics__local_runtime_status" "$LOG_PATH"; then
  echo "TOOL FAIL: LiteRT-LM did not emit the expected local_runtime_status tool call" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

if ! rg -q "LiteRT-LM tool-result continuation starting" "$LOG_PATH"; then
  echo "TOOL FAIL: tool-result continuation was not observed" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

echo "TOOL PASS: local_runtime_status tool call executed and rendered"
echo "prompt: $PROMPT"
echo "logcat: $LOG_PATH"
echo "ui xml: $UI_XML_PATH"
