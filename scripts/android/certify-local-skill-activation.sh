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
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-skill-activation.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-skill-activation.xml}"
MARKER="${MARKER:-KMSKILL$(date +%H%M%S)}"
PROMPT="${PROMPT:-You MUST call the knowme_diagnostics__local_runtime_status tool now. Do not answer directly. After the tool result is provided, answer exactly ${MARKER}.}"

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

PROMPT="$PROMPT" \
  EXPECTED_SUBSTRINGS="Skill knowme_diagnostics, running|Skill knowme_diagnostics, complete|Tool use knowme_diagnostics__local_runtime_status|Tool completed" \
  TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
  LOG_PATH="$LOG_PATH" \
  UI_XML_PATH="$UI_XML_PATH" \
  CERT_CLEAN_CHAT_STATE=1 \
  "$PROMPT_GATE" "$APK" "Skill knowme_diagnostics, complete" "$PACKAGE"

if ! rg -q "LiteRT-LM generation produced tool call .*name=knowme_diagnostics__local_runtime_status" "$LOG_PATH"; then
  echo "SKILL FAIL: LiteRT-LM did not emit the expected local_runtime_status tool call" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

if ! rg -F -q "Skill knowme_diagnostics, running" "$UI_XML_PATH"; then
  echo "SKILL FAIL: running skill activation block was not visible" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

if ! rg -F -q "Skill knowme_diagnostics, complete" "$UI_XML_PATH"; then
  echo "SKILL FAIL: complete skill activation block was not visible" >&2
  echo "logcat: $LOG_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

echo "SKILL PASS: local diagnostics skill activation rendered outside answer content"
echo "prompt: $PROMPT"
echo "marker: $MARKER"
echo "logcat: $LOG_PATH"
echo "ui xml: $UI_XML_PATH"
