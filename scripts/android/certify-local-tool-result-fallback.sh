#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=300" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-300}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-tool-result-fallback.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-tool-result-fallback.xml}"
MARKER="${MARKER:-KMFALLBACK$(date +%H%M%S)}"
PROMPT="${PROMPT:-You MUST call the knowme_diagnostics__local_runtime_status tool now. Do not answer directly. After the tool result is provided, answer exactly ${MARKER}.}"

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

PROMPT="$PROMPT" \
  EXPECTED_SUBSTRINGS="tool_result_fallback_applied" \
  TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
  LOG_PATH="$LOG_PATH" \
  UI_XML_PATH="$UI_XML_PATH" \
  CERT_CLEAN_CHAT_STATE=1 \
  FORCE_LITERT_TOOL_RESULT_FALLBACK=1 \
  "$PROMPT_GATE" "$APK" "tool_result_fallback_applied" "$PACKAGE"

if ! rg -q "LiteRT-LM generation produced tool call .*name=knowme_diagnostics__local_runtime_status" "$LOG_PATH"; then
  echo "FALLBACK FAIL: LiteRT-LM did not emit the expected local_runtime_status tool call" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

if ! rg -q "Forcing LiteRT-LM tool-result fallback for Android certification" "$LOG_PATH"; then
  echo "FALLBACK FAIL: Android bridge did not acknowledge the forced fallback sentinel" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

if ! rg -q "LiteRT-LM tool-result fallback follow-up applied" "$LOG_PATH"; then
  echo "FALLBACK FAIL: Rust did not preserve the fallback as a canonical stream event" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

if ! rg -q "tool_result_fallback_applied" "$LOG_PATH"; then
  echo "FALLBACK FAIL: fallback event name was not observed in runtime logs" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

echo "FALLBACK PASS: forced empty-continuation fallback executed and rendered"
echo "prompt: $PROMPT"
echo "marker: $MARKER"
echo "logcat: $LOG_PATH"
echo "ui xml: $UI_XML_PATH"
