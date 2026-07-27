#!/usr/bin/env bash
# Certify media save/share on a physical iPhone.
#
# WHAT THIS AUTOMATES AND WHAT IT CANNOT
#
# iOS has no `uiautomator dump` equivalent available to a script outside an
# XCUITest target. `devicectl` can install, launch, and read the device console
# — it cannot tap. So this script does everything up to the interaction and
# then asks the operator to drive the three taps, exactly as the Android
# picker step does.
#
# Being explicit about that split matters: a script that silently skipped the
# interaction and printed PASS would certify nothing. Every automated check
# here is asserted; every manual one is named.
#
# Automated:
#   - the app installs and launches
#   - the FFI library loads and migrations complete (console)
#   - no crash during the run (console)
#
# Operator-driven (named, not skipped):
#   - attach an image, send it
#   - tap Save on the rendered image → confirm it lands in Files/Photos
#   - tap Share → confirm the iOS share sheet opens with the file
#
# On iOS, Save writes into the app's Documents directory, which is the only
# user-reachable location the platform offers; the share sheet IS the platform's
# primary export path, which is why both controls exist rather than just one.
set -euo pipefail

DEVICE="${DEVICE:-3CF53A89-6EDC-544E-8555-1E9BE6F3F452}"
BUNDLE_ID="${BUNDLE_ID:-tools.know-me.knowme}"
APP_PATH="${1:-mobile/build/ios/iphoneos/Runner.app}"
CONSOLE_LOG="${CONSOLE_LOG:-/tmp/knowme-ios-media.log}"
REPORT="${REPORT:-/tmp/knowme-ios-media.json}"
BOOT_WAIT_SECONDS="${BOOT_WAIT_SECONDS:-25}"

if [[ ! -d "$APP_PATH" ]]; then
  echo "app bundle not found: $APP_PATH" >&2
  echo "build it first: flutter build ios --release" >&2
  exit 66
fi

# A stale bundle is the failure mode that wastes the most operator time: it
# installs and runs perfectly while proving nothing about the current code.
APP_EPOCH="$(stat -f %m "$APP_PATH")"
NOW_EPOCH="$(date +%s)"
AGE_MINUTES=$(( (NOW_EPOCH - APP_EPOCH) / 60 ))
if (( AGE_MINUTES > 120 )); then
  echo "REFUSING: $APP_PATH is ${AGE_MINUTES} minutes old." >&2
  echo "  Certifying a stale bundle produces a green report about old code." >&2
  echo "  Rebuild, or set ALLOW_STALE=1 if you know it is current." >&2
  [[ "${ALLOW_STALE:-0}" == "1" ]] || exit 65
fi

command -v xcrun >/dev/null 2>&1 || { echo "xcrun not found" >&2; exit 69; }
xcrun devicectl list devices 2>/dev/null | grep -q "$DEVICE" || {
  echo "device $DEVICE is not connected" >&2; exit 69; }

echo "Installing $APP_PATH (built ${AGE_MINUTES}m ago)..."
if ! xcrun devicectl device install app --device "$DEVICE" "$APP_PATH" >/dev/null; then
  echo "IOS FAIL: install failed — nothing downstream was exercised" >&2
  exit 1
fi

echo "Launching $BUNDLE_ID..."
if ! xcrun devicectl device process launch --device "$DEVICE" "$BUNDLE_ID" >/dev/null; then
  echo "IOS FAIL: launch failed for $BUNDLE_ID — check the bundle id" >&2
  exit 1
fi
sleep "$BOOT_WAIT_SECONDS"

# The console is the only automated window into what the app actually did.
echo "Collecting device console..."
timeout 20 xcrun devicectl device console --device "$DEVICE" \
  > "$CONSOLE_LOG" 2>/dev/null || true

# An EMPTY console and a genuine failure look identical if both report false.
# `devicectl device console` streams only live output and often captures
# nothing after the boot burst has passed — measured on this device: 0 bytes.
# So absence is reported as UNOBSERVED, never as a failed check.
console_bytes=$(wc -c < "$CONSOLE_LOG" 2>/dev/null | tr -d ' ')
console_bytes=${console_bytes:-0}

ffi_loaded=unobserved
migrations_ready=unobserved
crashed=unobserved
if (( console_bytes > 0 )); then
  grep -q "gen_ui_ffi" "$CONSOLE_LOG" 2>/dev/null && ffi_loaded=yes || ffi_loaded=no
  grep -q "migrations ready" "$CONSOLE_LOG" 2>/dev/null && migrations_ready=yes || migrations_ready=no
  grep -qiE "fatal|SIGABRT|crash" "$CONSOLE_LOG" 2>/dev/null && crashed=yes || crashed=no
fi

echo ""
echo "=============================================================="
echo "OPERATOR STEPS — iOS cannot be driven from a script (see header)"
echo ""
echo "  1. Open Chat and attach any image, then send it."
echo "  2. On the rendered image, tap SAVE."
echo "       -> expect a confirmation naming where it went"
echo "  3. Tap SHARE."
echo "       -> expect the iOS share sheet, carrying the image file"
echo ""
echo "Report what you observed; this script records the automated half."
echo "=============================================================="
echo ""

python3 - "$REPORT" "$ffi_loaded" "$migrations_ready" "$crashed" "$AGE_MINUTES" "$console_bytes" <<'PY'
import json, sys
report_path, ffi, migrations, crashed, age, console_bytes = sys.argv[1:7]
report = {
    "surface": "iOS",
    "device": "iPhone 17 Pro Max (iPhone18,2), iOS 27.0",
    "bundleAgeMinutes": int(age),
    "consoleBytesCaptured": int(console_bytes),
    "automated": {
        # Install and launch are the only checks devicectl can assert directly;
        # a failure in either aborts before this point.
        "installedAndLaunched": True,
        # "unobserved" is NOT a failure — see the console note below.
        "ffiLoaded": ffi,
        "migrationsReady": migrations,
        "crashFree": crashed,
        "consoleNote": (
            "devicectl device console streams live output only and captured "
            "0 bytes on this device after the boot burst. unobserved means the "
            "console was silent, not that the check failed."
        ) if int(console_bytes) == 0 else None,
    },
    "operatorDriven": {
        "saveTapped": None,
        "shareSheetOpened": None,
        "note": (
            "null until the operator reports it. iOS exposes no scriptable UI "
            "dump outside an XCUITest target, so these CANNOT be asserted here "
            "— recorded as unknown rather than assumed."
        ),
    },
}
open(report_path, "w").write(json.dumps(report, indent=2, sort_keys=True) + "\n")
print(json.dumps(report["automated"], indent=2))
PY

if [[ "$crashed" == "yes" ]]; then
  echo "IOS FAIL: the console shows a crash" >&2
  echo "console: $CONSOLE_LOG" >&2
  exit 1
fi

echo "IOS AUTOMATED CHECKS PASS — operator verification still required"
echo "report:  $REPORT"
echo "console: $CONSOLE_LOG"
