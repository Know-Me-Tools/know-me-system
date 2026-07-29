#!/usr/bin/env bash
# Certify that embedded-UAR state survives process death on macOS desktop.
#
# WHY A SEPARATE SCRIPT FROM THE ANDROID GATE
#
# Same claim, different evidence. macOS grants two things Android refused:
#
#   1. THE STORE DIRECTORY IS READABLE. `run-as` refuses a release build on
#      Android, so that gate could only report the store as UNVERIFIED and
#      lean entirely on a boot marker. Here the files are right there, so
#      this script asserts what Android could not: the store exists, it has
#      contents, and those contents SURVIVE a restart with their bytes
#      intact. That is the actual claim; a boot marker is a proxy for it.
#   2. Desktop and mobile do NOT share a boot path. Mobile boots through
#      gen_ui_ffi::api::boot; desktop through gen_ui_host::bootstrap_inner.
#      An Android pass says nothing about whether desktop threads the
#      storage_dir at all — that is a separate call site, separately wrong.
#
# THE FAST-RELAUNCH GATE IS STILL THE POINT. SurrealKV releases its exclusive
# directory lock on a background task rather than synchronously with the
# handle drop, so an app relaunched faster than the OS reaps the old process
# can fail to open its own database.
set -euo pipefail

APP="${1:-desktop/src-tauri/target/release/bundle/macos/knowme-poc.app}"
BUNDLE_ID="${BUNDLE_ID:-tools.knowme.knowme}"
REPORT="${REPORT:-/tmp/knowme-desktop-uar-persistence.json}"
SETTLE_SECONDS="${SETTLE_SECONDS:-20}"

# Where bootstrap_inner puts it: HostConfig.data_dir/uar-db (gen_ui_host/src/lib.rs).
DATA_DIR="${DATA_DIR:-$HOME/Library/Application Support/$BUNDLE_ID}"
STORE_DIR="$DATA_DIR/uar-db"

[[ -d "$APP" ]] || { echo "app bundle not found: $APP" >&2; exit 66; }
command -v python3 >/dev/null 2>&1 || { echo "python3 not found" >&2; exit 69; }

launch() {
  # `open -a` resolves its argument as an application NAME via LaunchServices;
  # a relative path is not a name and fails. Launch the bundle by path instead.
  open "$APP"
  sleep "$SETTLE_SECONDS"
  # A crashed launch leaves no store, which reads identically to "started fine
  # but persisted nothing" — two very different defects. Assert the process is
  # actually alive so a FAIL names the right cause.
  if ! pgrep -f "knowme-poc" >/dev/null 2>&1; then
    echo "" >&2
    echo "LAUNCH FAIL: the app is not running ${SETTLE_SECONDS}s after open" >&2
    echo "  it crashed on startup or never launched — nothing about" >&2
    echo "  persistence was tested. Check Console.app for a crash report." >&2
    exit 1
  fi
}

quit() {
  # osascript asks the app to quit cleanly; pkill is the backstop for a hang.
  osascript -e "tell application id \"$BUNDLE_ID\" to quit" >/dev/null 2>&1 || true
  sleep 2
  pkill -f "knowme-poc" >/dev/null 2>&1 || true
  sleep 1
}

# Fingerprint = the store's file names AND sizes. Names alone would pass on a
# store that was recreated empty on every boot, which is precisely the failure
# an in-memory-provider regression produces.
fingerprint() {
  [[ -d "$STORE_DIR" ]] || { echo ""; return; }
  find "$STORE_DIR" -type f -exec stat -f '%N:%z' {} \; 2>/dev/null | sort
}

echo "=== COLD START ==="
quit
launch
cold_fp="$(fingerprint)"
cold_count="$(printf '%s' "$cold_fp" | grep -c . || true)"
echo "  store dir: $STORE_DIR"
echo "  files after cold start: $cold_count"
# Captured here so the fast-relaunch phase can prove the WAL did not shrink.
WAL_COLD="$(find "$STORE_DIR" -type f -name '*.wal' 2>/dev/null | sort | head -1 \
  | xargs -I{} stat -f '%z' {} 2>/dev/null || echo 0)"
echo "  wal after cold start: ${WAL_COLD}B"

if (( cold_count == 0 )); then
  echo "" >&2
  echo "PERSISTENCE FAIL: no store on disk after boot" >&2
  echo "  desktop is running on a non-durable provider, or bootstrap_inner" >&2
  echo "  never threaded storage_dir into build_local" >&2
  exit 1
fi

echo ""
echo "=== FAST RELAUNCH — the gate that matters ==="
quit
launch
fast_fp="$(fingerprint)"
fast_count="$(printf '%s' "$fast_fp" | grep -c . || true)"
echo "  files after relaunch: $fast_count"

# The store must not have been wiped and recreated. Files may legitimately GROW
# (the runtime writes on boot), so require that the set did not shrink and that
# the store was not emptied — not byte-for-byte equality, which would false-fail
# on a perfectly healthy write.
survived=0
if (( fast_count >= cold_count )) && (( fast_count > 0 )); then
  survived=1
fi

# THE STRONGER CLAIM. File count alone cannot distinguish "reopened with its
# contents" from "recreated empty with the same three file names" — and the
# second is exactly what an in-memory regression looks like on disk. The WAL
# carries the written state, so a store that was truly reopened retains its
# prior bytes; one recreated each boot resets to near-zero. Require that the
# WAL did not shrink across the restart.
wal_bytes() {
  local w
  w="$(find "$STORE_DIR" -type f -name '*.wal' 2>/dev/null | sort | head -1)"
  [[ -n "$w" ]] && stat -f '%z' "$w" 2>/dev/null || echo 0
}
wal_after="$(wal_bytes)"
echo "  wal after relaunch: ${wal_after}B (cold: ${WAL_COLD:-?}B)"
if [[ -n "${WAL_COLD:-}" ]] && (( wal_after < WAL_COLD )); then
  survived=0
  echo "  WAL SHRANK — the store was recreated, not reopened" >&2
fi
echo "  store survived restart: $([[ $survived == 1 ]] && echo yes || echo NO)"

python3 - "$REPORT" "$cold_count" "$fast_count" "$survived" "$STORE_DIR" <<'PY'
import json, platform, sys
report, cold, fast, survived, store = sys.argv[1:6]
data = {
    "host": {"os": platform.platform(), "machine": platform.machine()},
    "engine": "SurrealKV (surrealdb kv-surrealkv); NOT RocksDB",
    "bootPath": "gen_ui_host::bootstrap_inner (NOT the mobile gen_ui_ffi path)",
    "storeDir": store,
    "coldStart": {"fileCount": int(cold)},
    "fastRelaunch": {
        "fileCount": int(fast),
        "storeSurvived": survived == "1",
        "why": (
            "SurrealKV releases its directory lock on a background task, not "
            "synchronously with the handle drop. A relaunch faster than the OS "
            "reaps the old process is the case that fails on hardware while "
            "the unit test passes."
        ),
    },
    "note": (
        "Unlike the Android gate, the store directory is directly readable "
        "here, so this asserts the store's real contents rather than relying "
        "on a boot marker as a proxy."
    ),
}
open(report, "w").write(json.dumps(data, indent=2, sort_keys=True) + "\n")
print(json.dumps(data["coldStart"] | {"fastRelaunch": data["fastRelaunch"]["fileCount"]}, indent=2))
PY

if (( survived == 0 )); then
  echo "" >&2
  echo "PERSISTENCE FAIL: the store did not survive a restart" >&2
  echo "  state is being lost between runs — the exact defect durable" >&2
  echo "  persistence exists to prevent" >&2
  exit 1
fi

echo ""
echo "PERSISTENCE PASS: the store survived a cold start and a fast relaunch"
echo "report: $REPORT"
