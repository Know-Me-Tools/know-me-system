#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: TIMEOUT_SECONDS=300 CERT_CLEAN_CHAT_STATE=1" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-300}"
CERT_CLEAN_CHAT_STATE="${CERT_CLEAN_CHAT_STATE:-1}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
TOKEN="${TOKEN:-KMBRANCH$(date +%H%M%S)}"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-branch-actions.log}"
UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-branch-actions.xml}"
DB_PATH="${DB_PATH:-/tmp/knowme-local-branch-actions.sqlite3}"
REMOTE_UI_XML="/sdcard/knowme-local-branch-actions.xml"

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

for tool in adb rg python3 sqlite3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "$tool not found in PATH" >&2
    exit 69
  fi
done

if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

source "$SCRIPT_DIR/lib-chat-state.sh"
trap 'cert_restore_clean_chat_state "$PACKAGE"' EXIT

adb install -r "$APK" >/dev/null
cert_prepare_clean_chat_state "$PACKAGE"

PROMPT="Reply exactly ${TOKEN}." \
  EXPECTED_SUBSTRINGS="$TOKEN" \
  TIMEOUT_SECONDS="$TIMEOUT_SECONDS" \
  LOG_PATH="/tmp/knowme-local-branch-seed.log" \
  UI_XML_PATH="/tmp/knowme-local-branch-seed.xml" \
  CERT_CLEAN_CHAT_STATE=0 \
  "$PROMPT_GATE" "$APK" "$TOKEN" "$PACKAGE"

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

tap_exact_node() {
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
    values = [
        (node.attrib.get("text") or "").lower(),
        (node.attrib.get("content-desc") or "").lower(),
        (node.attrib.get("hint") or "").lower(),
    ]
    if needle not in values:
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

dump_db() {
  adb exec-out run-as "$PACKAGE" cat app_flutter/entity-records.sqlite3 > "$DB_PATH"
}

sql_value() {
  local query="$1"
  dump_db
  sqlite3 "$DB_PATH" "$query"
}

message_count_where() {
  local where_clause="$1"
  sql_value "select count(*) from entity_records where entity_type='chat_message' and ${where_clause};"
}

assistant_ids() {
  sql_value "select group_concat(id, ',') from entity_records where entity_type='chat_message' and json_extract(data_json, '$.role')='assistant' order by json_extract(data_json, '$.createdAt');"
}

wait_for_db_shape() {
  local label="$1"
  local total="$2"
  local users="$3"
  local assistants="$4"
  local branches="$5"
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  while (( SECONDS < deadline )); do
    local actual_total actual_users actual_assistants actual_branches
    actual_total="$(message_count_where "1=1")"
    actual_users="$(message_count_where "json_extract(data_json, '$.role')='user'")"
    actual_assistants="$(message_count_where "json_extract(data_json, '$.role')='assistant'")"
    actual_branches="$(sql_value "select count(distinct json_extract(data_json, '$.branchId')) from entity_records where entity_type='chat_message';")"
    if [[ "$actual_total" == "$total" &&
          "$actual_users" == "$users" &&
          "$actual_assistants" == "$assistants" &&
          "$actual_branches" == "$branches" ]]; then
      return 0
    fi
    sleep 2
  done
  echo "BRANCH FAIL: timed out waiting for DB shape after $label" >&2
  echo "expected total/users/assistants/branches: $total/$users/$assistants/$branches" >&2
  dump_db || true
  sqlite3 "$DB_PATH" "select id, json_extract(data_json, '$.role'), json_extract(data_json, '$.branchId'), json_extract(data_json, '$.parentMessageId') from entity_records where entity_type='chat_message' order by json_extract(data_json, '$.createdAt');" >&2 || true
  echo "db: $DB_PATH" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
}

wait_for_db_shape "seed" 2 1 1 1

adb logcat -c
if ! tap_matching_node "Resubmit message"; then
  echo "BRANCH FAIL: Resubmit message action was not visible" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi
wait_for_db_shape "resubmit" 4 2 2 2
after_resubmit_assistants="$(assistant_ids)"

if ! tap_matching_node "Delete response and later branch messages"; then
  echo "BRANCH FAIL: Delete response action was not visible" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi
sleep 1
if ! tap_exact_node "Delete"; then
  echo "BRANCH FAIL: Delete confirmation action was not visible" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi
wait_for_db_shape "delete" 3 2 1 2
remaining_assistant="$(assistant_ids)"
if [[ -z "$remaining_assistant" || "$remaining_assistant" == "$after_resubmit_assistants" ]]; then
  echo "BRANCH FAIL: delete did not remove one assistant response" >&2
  echo "before: $after_resubmit_assistants" >&2
  echo "after: $remaining_assistant" >&2
  exit 1
fi

if ! tap_matching_node "Regenerate response"; then
  echo "BRANCH FAIL: Regenerate response action was not visible" >&2
  echo "ui xml: $UI_XML_PATH" >&2
  exit 1
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  wait_for_db_shape "regenerate-shape" 3 2 1 2
  regenerated_assistant="$(assistant_ids)"
  if [[ -n "$regenerated_assistant" && "$regenerated_assistant" != "$remaining_assistant" ]]; then
    dump_ui || true
    echo "BRANCH PASS: resubmit, delete, and regenerate actions updated persisted branch records"
    echo "token: $TOKEN"
    echo "assistants after resubmit: $after_resubmit_assistants"
    echo "assistant after delete: $remaining_assistant"
    echo "assistant after regenerate: $regenerated_assistant"
    echo "db: $DB_PATH"
    echo "ui xml: $UI_XML_PATH"
    exit 0
  fi
  sleep 2
done

echo "BRANCH FAIL: regenerate did not replace the remaining assistant response" >&2
echo "before regenerate: $remaining_assistant" >&2
echo "after regenerate: ${regenerated_assistant:-}" >&2
echo "db: $DB_PATH" >&2
echo "ui xml: $UI_XML_PATH" >&2
exit 1
