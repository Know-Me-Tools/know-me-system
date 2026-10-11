#!/usr/bin/env bash
# TJ-ARCH-MOB-001 compliant
set -euo pipefail
site="${1:-site}"
test -f "$site/package-lock.json"
site="$(cd "$site" && pwd)"
(
  cd "$site"
  npm ci
  npm run sanitize
  npm run build
)
if rg -n '/Users/|\.prometheus/|BEGIN .*PRIVATE KEY' "$site/build"; then
  echo "private material found in site output" >&2
  exit 1
fi
echo "branded Docusaurus verification passed"
