#!/usr/bin/env bash
# Capture device evidence for the local-runtime fallback contract.
#
# The phase goal calls for "safe memory/thermal fallback" evidence. Two things
# are in scope and they are NOT the same:
#
#   1. Backend fallback — the engine reports which backend it actually chose and
#      WHY it is not on a faster one. Rust always populates `fallback_reason`
#      (litert_lm.rs: "CPU is the certification baseline; GPU/NPU require device
#      proof"), so this is observable on any run and is what this script asserts.
#
#   2. Memory-preflight downgrade — refusing an over-budget model, or picking a
#      smaller catalogued one. This is covered by unit tests
#      (`cargo test -p gen_ui_inference --lib --features local-llama`, 6 tests
#      including downgrade selection and the budget override) but CANNOT be
#      driven from here: the budget lever is the process environment variable
#      KNOWME_LOCAL_MODEL_MAX_BYTES / KNOWME_DEVICE_MEMORY_BUDGET_BYTES, and
#      Android offers no way to inject an env var into an app launched by
#      `monkey` or `am start`. Forcing it would require an app-side debug hook
#      that does not exist. This script therefore reports that gap explicitly
#      rather than implying the downgrade path was exercised on device.
#
# Thermal state is captured as a reading, not a fallback trigger — nothing in
# the engine currently changes behaviour on thermal status.
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  echo "env: LAUNCH_WAIT_SECONDS=10" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
LAUNCH_WAIT_SECONDS="${LAUNCH_WAIT_SECONDS:-10}"
LOG_PATH="${LOG_PATH:-/tmp/knowme-local-fallback.log}"
OUTPUT="${OUTPUT:-/tmp/knowme-local-fallback.json}"

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

for tool in adb python3; do
  command -v "$tool" >/dev/null 2>&1 || { echo "$tool not found in PATH" >&2; exit 69; }
done

adb get-state >/dev/null 2>&1 || { echo "no adb device available" >&2; exit 69; }

MODEL="$(adb shell getprop ro.product.model | tr -d '\r')"
SOC="$(adb shell getprop ro.soc.model | tr -d '\r')"
echo "Device: ${MODEL:-unknown} (${SOC:-unknown})"

# The engine loads lazily on the first SEND, not at launch. An earlier version of
# this script cleared logcat, launched the app, and then asserted on diagnostics
# that only a generation produces — it destroyed the evidence it was looking for
# and reported a product failure. Drive a real turn through the prompt gate so
# the load actually happens, and keep ITS log rather than taking a fresh dump.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_GATE="$SCRIPT_DIR/certify-local-model-prompt.sh"
if [[ ! -x "$PROMPT_GATE" ]]; then
  echo "prompt gate script is not executable: $PROMPT_GATE" >&2
  exit 66
fi

TOKEN="KMFALLBACK$(date +%H%M%S)"
echo "Driving one generation to force a model load..." >&2
PROMPT="Reply exactly ${TOKEN}." \
  EXPECTED_SUBSTRINGS="$TOKEN" \
  TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-420}" \
  UI_XML_PATH="${UI_XML_PATH:-/tmp/knowme-local-fallback.xml}" \
  # Deliberately NOT starting a fresh conversation: a new thread resets the
  # conversation mode to the default agent, which routes through the UAR and
  # never loads a local model — so the very diagnostics this script asserts on
  # would never be emitted. The tool-call gate needs a clean thread; this one
  # needs the existing Local selection.
  CERT_NEW_CONVERSATION="${CERT_NEW_CONVERSATION:-0}" \
  CERT_SKIP_INSTALL="${CERT_SKIP_INSTALL:-1}" \
  "$PROMPT_GATE" "$APK" "$TOKEN" "$PACKAGE" >&2

# Thermal reading. Status 0 means "no throttling"; a non-zero reading during a
# certification run is itself worth recording, because a throttled device
# invalidates the throughput numbers captured alongside it.
THERMAL="$(adb shell dumpsys thermalservice 2>/dev/null | grep -m1 -oE 'Thermal Status: [0-9]+' | grep -oE '[0-9]+' || echo '')"

GATE_LOG="/tmp/knowme-local-model-certification.log"
if [[ -f "$GATE_LOG" ]]; then
  cp "$GATE_LOG" "$LOG_PATH"
else
  echo "FALLBACK FAIL: the prompt gate wrote no log at $GATE_LOG" >&2
  exit 1
fi

python3 - "$LOG_PATH" "$OUTPUT" "${THERMAL:-}" "${MODEL:-}" "${SOC:-}" <<'PY'
import json
import re
import sys
from pathlib import Path

log = Path(sys.argv[1]).read_text(errors="replace")
output = Path(sys.argv[2])
thermal = sys.argv[3]
model = sys.argv[4]
soc = sys.argv[5]

# The adapter logs the backend it settled on. Absence is meaningful: a release
# build strips Flutter's own logging, but the Rust tracing lines survive.
backend = None
match = re.search(r"LiteRT-LM native adapter loaded model=(\S+)", log)
if match:
    backend = match.group(1)

# Matches the tracing field emitted alongside the backend on model load:
#   ... backend=litert-lm-cpu fallback_reason=CPU is the certification baseline...
# The reason was previously computed but never logged, so a device run had no
# way to record it; the assertion below is what makes that regression visible.
# `fallback_reason` is the LAST field on the line, so a lookahead for a
# following key= never matches — take the rest of the line instead.
reason = None
match = re.search(r"fallback_reason=(.+)", log)
if match:
    reason = match.group(1).strip()

report = {
    "device": {"model": model or None, "soc": soc or None},
    "thermalStatus": int(thermal) if thermal.isdigit() else None,
    "backendObserved": backend,
    "fallbackReasonObserved": reason,
    # Stated, not inferred: this script cannot drive the memory path.
    "memoryDowngradeExercised": False,
    "memoryDowngradeCoverage": (
        "unit tests only — cargo test -p gen_ui_inference --lib "
        "--features local-llama (downgrade selection, budget override, "
        "preflight rejection). Not drivable on device: the budget lever is a "
        "process env var and Android cannot inject one into a monkey/am launch."
    ),
}
output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")

problems = []
if backend is None:
    problems.append(
        "no backend line in logcat — the adapter never reported a loaded model, "
        "so this run carries no backend evidence"
    )
if report["thermalStatus"] is None:
    problems.append("thermal status unreadable")
if reason is None:
    problems.append(
        "no fallback_reason in logcat — the engine reports which backend it "
        "chose but not what it fell back from, so this run carries no fallback "
        "evidence"
    )

if problems:
    for problem in problems:
        print(f"FALLBACK FAIL: {problem}", file=sys.stderr)
    raise SystemExit(1)

print(f"backend: {backend}")
print(f"thermal status: {report['thermalStatus']}")
print(f"fallback reason: {reason if reason else '(not present in this run)'}")
PY

echo "FALLBACK PASS: backend and thermal evidence captured"
echo "report: $OUTPUT"
echo "logcat: $LOG_PATH"
echo "NOTE: memory-preflight downgrade is NOT exercised here — see the header." >&2
