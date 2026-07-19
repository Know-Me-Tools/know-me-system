#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: CASES=exact,office,mermaid,code,json TIMEOUT_SECONDS=240" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"
CASES="${CASES:-exact,office,mermaid,code,json}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
RUN_ID="$(date +%H%M%S)"

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

run_case() {
  local name="$1"
  local token="KM${name^^}${RUN_ID}"
  local prompt expected
  case "$name" in
    exact)
      prompt="Reply exactly ${token}."
      expected="$token"
      ;;
    office)
      prompt="Write three agenda bullets for a project sync and include marker ${token}."
      expected="${token}|agenda"
      ;;
    mermaid)
      prompt="Create a Mermaid graph TD from Start to End. Do not ask questions. Include marker ${token}."
      expected="Mermaid diagram"
      ;;
    code)
      prompt="Write a Dart function called greet and include marker ${token} in a comment."
      expected="${token}|greet"
      ;;
    json)
      prompt="Return JSON with fields status and marker. Include marker ${token}."
      expected="${token}|status"
      ;;
    *)
      echo "unknown prompt certification case: $name" >&2
      exit 64
      ;;
  esac

  echo "Running prompt-suite case: $name"
  PROMPT="$prompt" \
    EXPECTED_SUBSTRINGS="$expected" \
    TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
    LOG_PATH="/tmp/knowme-local-model-certification-${name}.log" \
    UI_XML_PATH="/tmp/knowme-local-model-certification-${name}.xml" \
    "$PROMPT_GATE" "$APK" "$token" "$PACKAGE"
}

IFS=',' read -r -a selected_cases <<< "$CASES"
for selected_case in "${selected_cases[@]}"; do
  selected_case="${selected_case//[[:space:]]/}"
  if [[ -z "$selected_case" ]]; then
    continue
  fi
  run_case "$selected_case"
done

echo "Prompt suite passed: $CASES"
