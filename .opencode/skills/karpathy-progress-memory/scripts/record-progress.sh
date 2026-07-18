#!/usr/bin/env bash
# TJ-ARCH-MOB-001 compliant
set -euo pipefail

usage() {
  echo "usage: $0 --phase SLUG --title TEXT --summary TEXT --evidence TEXT --next TEXT [--status STATUS]" >&2
  exit 2
}

phase="" title="" summary="" evidence="" next_step="" status="in-progress"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --phase) phase="${2:-}"; shift 2 ;;
    --title) title="${2:-}"; shift 2 ;;
    --summary) summary="${2:-}"; shift 2 ;;
    --evidence) evidence="${2:-}"; shift 2 ;;
    --next) next_step="${2:-}"; shift 2 ;;
    --status) status="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done
[[ -n "$phase" && -n "$title" && -n "$summary" && -n "$evidence" && -n "$next_step" ]] || usage
[[ "$phase" =~ ^[a-z0-9][a-z0-9-]*$ ]] || { echo "phase must be a lowercase slug" >&2; exit 2; }

repo_root="$(git rev-parse --show-toplevel)"
project_slug="$(basename "$repo_root" | tr -cs '[:alnum:]-' '-' | sed 's/-$//')"
utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
entry_id="karpathy-progress-${stamp}-${phase}"
project_wiki="$repo_root/.prometheus/knowledge/wiki"
private_wiki="${PROMETHEUS_PRIVATE_ROOT:-$HOME/.prometheus}/knowledge/private/$project_slug/wiki"
private_project="${PROMETHEUS_PRIVATE_ROOT:-$HOME/.prometheus}/knowledge/private/$project_slug"
mkdir -p "$project_wiki" "$private_wiki"

combined="$title $summary $evidence $next_step"
if printf '%s' "$combined" | grep -Eiq '(api[_ -]?key|token|password|secret|private[_ -]?key)[[:space:]]*[:=][[:space:]]*[^$<{[]'; then
  echo "refusing to record text that appears to contain a secret value" >&2
  exit 3
fi

redact_root() { sed "s#${repo_root//\#/\\#}#\$REPO_ROOT#g"; }
safe_title="$(printf '%s' "$title" | redact_root)"
safe_summary="$(printf '%s' "$summary" | redact_root)"
safe_evidence="$(printf '%s' "$evidence" | redact_root)"
safe_next="$(printf '%s' "$next_step" | redact_root)"
project_file="$project_wiki/$entry_id.md"

{
  echo '---'
  echo 'type: Reference'
  echo "id: $entry_id"
  printf 'title: "%s"\n' "${safe_title//\"/\\\"}"
  echo 'tags:'
  echo '- karpathy-progress'
  echo "- $phase"
  echo "- $status"
  echo 'sources:'
  echo '- conversation:operator-agent'
  echo "timestamp: $utc"
  echo "created_at: $utc"
  echo "updated_at: $utc"
  echo 'revision: 1'
  echo '---'
  echo
  echo '## Intent'
  echo
  printf '%s\n' "$safe_summary"
  echo
  echo '## Observed state and verification'
  echo
  printf '%s\n' "$safe_evidence"
  echo
  echo '## Decision and lesson'
  echo
  echo "Status: $status. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal."
  echo
  echo '## Next experiment'
  echo
  printf '%s\n' "$safe_next"
} > "$project_file"

cp "$project_file" "$private_wiki/$entry_id.md"

if command -v jq >/dev/null 2>&1; then
  event_id="$(uuidgen | tr '[:upper:]' '[:lower:]')"
  event="$(jq -cn \
    --arg id "$event_id" --arg session "karpathy-progress-memory" \
    --arg ts "$utc" --arg entry "$entry_id" --arg title "$safe_title" \
    --arg phase "$phase" --arg status "$status" \
    '{id:$id,kind:"compiled",session_id:$session,project_root:"$REPO_ROOT",scope:"project",timestamp:$ts,payload:{entry_id:$entry,tags:["karpathy-progress",$phase,$status],title:$title,ts:$ts,type:"compiled"},affects:[$entry]}')"
  printf '%s\n' "$event" >> "$repo_root/.prometheus/events.jsonl"
  printf '%s\n' "$event" >> "$private_project/events.jsonl"
fi

echo "$project_file"
echo "$private_wiki/$entry_id.md"
