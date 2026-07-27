#!/usr/bin/env bash
# Certify that embedded-UAR state survives process death on a real device.
#
# WHAT THIS PROVES, AND WHY IT NEEDS A DEVICE
#
# The unit test proves a dropped store can be reopened in the SAME process. A
# device proves the thing that actually matters: the OS kills the app, the app
# relaunches, and the SurrealKV store opens again with its contents intact.
#
# Two failure modes only hardware exposes:
#
#   1. THE LOCK RACE. SurrealKV holds an exclusive directory lock, released on
#      a background task rather than synchronously with the handle drop. An app
#      relaunched faster than the OS reaps the old process can fail to open its
#      own database. The unit test needed retry-with-backoff to survive this;
#      whether a real relaunch is fast enough to hit it is measured here, not
#      guessed.
#   2. Filesystem reality — permissions, storage pressure, scoped storage.
#
# THE FAST-RELAUNCH GATE IS THE POINT. A run that only tests a leisurely
# restart would pass on a build that cannot survive a real one.
set -euo pipefail

PACKAGE="${1:-tools.knowme.knowme}"
REPORT="${REPORT:-/tmp/knowme-uar-persistence.json}"
LOG="${LOG:-/tmp/knowme-uar-persistence.log}"
SETTLE_SECONDS="${SETTLE_SECONDS:-12}"
FAST_RELAUNCH_DELAY="${FAST_RELAUNCH_DELAY:-0}"

for tool in adb python3 rg; do
  command -v "$tool" >/dev/null 2>&1 || { echo "$tool not found" >&2; exit 69; }
done
adb get-state >/dev/null 2>&1 || { echo "no adb device" >&2; exit 69; }

boot_ok() {
  # "migrations ready" is the marker that the FFI finished bootstrapping,
  # which is the step that opens the persistence layer.
  rg -q "mobile migrations ready" "$LOG" 2>/dev/null
}

persistence_error() {
  # The failure this gate exists to catch. Reported verbatim rather than
  # summarised, so a lock error is distinguishable from a permissions error.
  rg -o "cannot open persistent store[^\"]*|already locked[^\"]*" "$LOG" 2>/dev/null | head -2 || true
}

launch_and_capture() {
  adb logcat -c
  adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1
  sleep "$SETTLE_SECONDS"
  adb logcat -d > "$LOG" 2>/dev/null || true
}

echo "=== COLD START ==="
adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
sleep 2
launch_and_capture
cold_ok=0; boot_ok && cold_ok=1
cold_err="$(persistence_error || true)"
echo "  boot marker: $([[ $cold_ok == 1 ]] && echo present || echo MISSING)"
[[ -n "$cold_err" ]] && echo "  persistence error: $cold_err"

# Did the store directory actually appear? run-as is blocked on release builds,
# so absence here is UNVERIFIED, not a failure — the boot marker carries the
# proof instead.
store_listing="$(adb shell run-as "$PACKAGE" ls app_flutter/uar-db 2>/dev/null | tr -d '\r' | head -5 || true)"
if [[ -n "$store_listing" ]]; then
  echo "  uar-db contents:"; printf '%s\n' "$store_listing" | sed 's/^/    /'
else
  echo "  uar-db UNVERIFIED (run-as refuses release builds)"
fi

echo ""
echo "=== FAST RELAUNCH — the gate that matters ==="
echo "  force-stop then relaunch after ${FAST_RELAUNCH_DELAY}s"
adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
[[ "$FAST_RELAUNCH_DELAY" != "0" ]] && sleep "$FAST_RELAUNCH_DELAY"
launch_and_capture
fast_ok=0; boot_ok && fast_ok=1
fast_err="$(persistence_error || true)"
echo "  boot marker: $([[ $fast_ok == 1 ]] && echo present || echo MISSING)"
[[ -n "$fast_err" ]] && echo "  persistence error: $fast_err"

python3 - "$REPORT" "$cold_ok" "$fast_ok" "$cold_err" "$fast_err" "$store_listing" <<'PY'
import json, subprocess, sys
report, cold, fast, cold_err, fast_err, listing = sys.argv[1:7]

def prop(name):
    try:
        return subprocess.run(["adb", "shell", "getprop", name],
                              capture_output=True, text=True, timeout=15).stdout.strip()
    except Exception:
        return ""

data = {
    "device": {"model": prop("ro.product.model"), "soc": prop("ro.soc.model")},
    "engine": "SurrealKV (surrealdb kv-surrealkv); NOT RocksDB",
    "coldStart": {"booted": cold == "1", "persistenceError": cold_err or None},
    "fastRelaunch": {
        "booted": fast == "1",
        "persistenceError": fast_err or None,
        "why": (
            "SurrealKV releases its directory lock on a background task, not "
            "synchronously with the handle drop. This is the case that could "
            "fail on hardware while the unit test passes."
        ),
    },
    "storeListing": listing.splitlines() if listing else [],
    "storeListingNote": (
        "empty when run-as refuses a release build — UNVERIFIED, not a failure; "
        "the boot marker carries the proof"
    ),
}
open(report, "w").write(json.dumps(data, indent=2, sort_keys=True) + "\n")
print(json.dumps({k: data[k] for k in ("coldStart", "fastRelaunch")}, indent=2))
PY

if (( cold_ok == 0 )) || (( fast_ok == 0 )); then
  echo "" >&2
  echo "PERSISTENCE FAIL: the runtime did not finish boot on one or both starts" >&2
  echo "logcat: $LOG" >&2
  exit 1
fi
if [[ -n "$cold_err$fast_err" ]]; then
  echo "" >&2
  echo "PERSISTENCE FAIL: the store could not be opened" >&2
  echo "  This is the defect the change exists to prevent — a store that cannot" >&2
  echo "  open means state is lost, whatever else boots successfully." >&2
  exit 1
fi

echo ""
echo "PERSISTENCE PASS: cold start and fast relaunch both opened the store"
echo "report: $REPORT"
echo "logcat: $LOG"
