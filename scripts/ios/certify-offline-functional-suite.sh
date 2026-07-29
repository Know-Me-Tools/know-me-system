#!/usr/bin/env bash
# Certify the iOS functional suite with networking disabled (task 5.5).
#
# WHY THIS DRIVES `flutter test integration_test` AND NOT `devicectl`
#
# `devicectl` can install, launch, and read the console — it CANNOT tap, and
# iOS exposes no `uiautomator dump` equivalent to a script outside an XCTest
# target. `scripts/ios/certify-media-save-share.sh` therefore has to hand its
# interaction steps to the operator.
#
# Flutter's `integration_test` does not have that limitation: it runs
# in-process as an XCTest, so it can drive the real UI AND read `chatProvider`
# directly. Reading provider state is stronger evidence than screen-scraping —
# it distinguishes "a run settled as cancelled" from "some text stopped
# appearing", which a screenshot cannot.
#
# WHY OFFLINE IS ENFORCED AND PROVEN, NOT ASSUMED
#
# The Android suite learned this the expensive way: `svc wifi disable` alone
# left the device reachable over mobile data, so gates that believed they ran
# offline actually ran online. Wi-Fi off is not proof of offline.
#
# iOS has no scriptable radio control at all — no `svc` equivalent, and
# `devicectl` cannot toggle airplane mode. So this gate REFUSES to run until it
# has PROVEN the device is unreachable, and the operator flips airplane mode by
# hand. Refusing is the point: a suite that silently ran online would certify
# the opposite of what 5.5 asks.
set -euo pipefail

DEVICE_UDID="${DEVICE_UDID:-00008150-00125461363A401C}"
DEVICE_NAME="${DEVICE_NAME:-attached iPhone}"
REPORT="${REPORT:-/tmp/knowme-ios-offline-suite.json}"
FLUTTER="${FLUTTER:-$HOME/development/flutter/bin/flutter}"
MOBILE_DIR="$(cd "$(dirname "$(dirname "$(dirname "$0")")")" && pwd)/mobile"

# Each test is named so a partial run reports WHICH half was certified rather
# than collapsing to a single pass/fail.
SUITES=(
  "integration_test/local_chat_test.dart"
  "integration_test/stopped_turn_intents_test.dart"
)

command -v xcrun >/dev/null 2>&1 || { echo "xcrun not found" >&2; exit 69; }
[[ -x "$FLUTTER" ]] || { echo "flutter not found at $FLUTTER" >&2; exit 69; }

device_connected() {
  xcrun devicectl list devices 2>/dev/null | rg -q "connected"
}

# WHY THERE IS NO HOST-SIDE OFFLINE PROBE HERE
#
# I wrote one first and it was WRONG in the most dangerous direction. It
# grepped `devicectl device info details` for wifi/cellular state; that command
# reports hardware and boot properties ONLY — no radio fields at all — so the
# grep always missed and the gate would have printed "OFFLINE VERIFIED" on a
# fully online phone. That is precisely the false-pass the Android suite
# already suffered when `svc wifi disable` left cellular up.
#
# The host genuinely cannot measure the DEVICE's internet reachability:
# `devicectl` exposes no connectivity state, the .coredevice.local name does
# not resolve, and USB stays attached while offline by design.
#
# So the proof moves INTO the app, where it belongs. `_assert_offline` in the
# integration suite attempts a real network call on-device and requires it to
# fail. That is evidence rather than inference — and if the operator forgets
# airplane mode, the suite fails loudly instead of certifying a lie.
#
# There is deliberately NO `device_has_network` helper here. A host-side
# function that always returns "offline" would read like a real check to the
# next person and re-introduce the false pass.

echo "=============================================================="
echo "iOS OFFLINE FUNCTIONAL SUITE — task 5.5"
echo ""
echo "Device: $DEVICE_NAME ($DEVICE_UDID)"
echo ""

device_connected || {
  echo "REFUSING: no iOS device is connected." >&2
  echo "  Attach the iPhone over USB and trust the host." >&2
  exit 69
}

if [[ "${SKIP_OFFLINE_CHECK:-0}" != "1" ]]; then
  echo "OFFLINE ENFORCEMENT: delegated to the device."
  echo ""
  echo "  Enable AIRPLANE MODE on the iPhone now; leave USB attached."
  echo "  The suite opens a real socket on-device and FAILS if anything"
  echo "  answers, so a forgotten radio surfaces as a failure rather than"
  echo "  a green report about an online run."
  echo ""
  DART_DEFINES=(--dart-define=REQUIRE_OFFLINE=true)
  OFFLINE_ENFORCED=true
else
  echo "WARNING: offline enforcement SKIPPED — this run does NOT satisfy 5.5."
  DART_DEFINES=()
  OFFLINE_ENFORCED=false
fi
echo "=============================================================="
echo ""

results=()
overall=0
for suite in "${SUITES[@]}"; do
  echo ">>> $suite"
  log="/tmp/knowme-ios-$(basename "$suite" .dart).log"
  if (cd "$MOBILE_DIR" && "$FLUTTER" test "$suite" -d "$DEVICE_UDID" \
        "${DART_DEFINES[@]}" 2>&1 | tee "$log"); then
    echo "PASS: $suite"
    results+=("{\"suite\":\"$suite\",\"result\":\"pass\",\"log\":\"$log\"}")
  else
    echo "FAIL: $suite (log: $log)" >&2
    results+=("{\"suite\":\"$suite\",\"result\":\"fail\",\"log\":\"$log\"}")
    overall=1
  fi
  echo ""
done

python3 - "$REPORT" "$OFFLINE_ENFORCED" "$DEVICE_UDID" "$overall" "${results[@]}" <<'PY'
import json, sys
report_path, offline, udid, overall = sys.argv[1:5]
suites = [json.loads(item) for item in sys.argv[5:]]
report = {
    "surface": "iOS",
    "task": "5.5 — certify iOS with networking disabled against the functional suite",
    "device": {"udid": udid, "model": "iPhone (physical)"},
    # The distinction that makes this report worth anything: a suite that
    # passed while online proves the code runs, NOT that it runs offline.
    "offlineEnforced": offline == "true",
    "satisfiesTask55": offline == "true" and overall == "0",
    "suites": suites,
    "harnessNote": (
        "Driven by flutter integration_test, which runs in-process as an "
        "XCTest and can therefore tap real UI and read chatProvider state. "
        "devicectl cannot tap, which is why the media gate is operator-driven "
        "and this one is not."
    ),
}
open(report_path, "w").write(json.dumps(report, indent=2, sort_keys=True) + "\n")
print(json.dumps(report, indent=2, sort_keys=True))
PY

exit "$overall"
