#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name]" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
LAUNCH_WAIT_SECONDS="${LAUNCH_WAIT_SECONDS:-8}"
LOG_PATH="${LOG_PATH:-/tmp/knowme-device-runtime-gates.log}"

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

if ! command -v adb >/dev/null 2>&1; then
  echo "adb not found in PATH" >&2
  exit 69
fi

if ! adb get-state >/dev/null 2>&1; then
  echo "no adb device available; connect one device or set ANDROID_SERIAL" >&2
  exit 69
fi

DEVICE="$(adb shell getprop ro.product.model | tr -d '\r')"
MANUFACTURER="$(adb shell getprop ro.product.manufacturer | tr -d '\r')"
SDK="$(adb shell getprop ro.build.version.sdk | tr -d '\r')"
RELEASE="$(adb shell getprop ro.build.version.release | tr -d '\r')"
ABI="$(adb shell getprop ro.product.cpu.abi | tr -d '\r')"
SOC="$(adb shell getprop ro.soc.model | tr -d '\r')"

echo "Device: ${MANUFACTURER:-unknown} ${DEVICE:-unknown}, Android ${RELEASE:-unknown}/API ${SDK:-unknown}, ABI ${ABI:-unknown}, SoC ${SOC:-unknown}"
echo "Installing: $APK"
adb install -r "$APK" >/dev/null

adb shell am force-stop "$PACKAGE" >/dev/null 2>&1 || true
adb logcat -c

echo "Launching: $PACKAGE"
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep "$LAUNCH_WAIT_SECONDS"
adb logcat -d -v time > "$LOG_PATH"

PID="$(adb shell pidof "$PACKAGE" | tr -d '\r' || true)"
if [[ -z "$PID" ]]; then
  echo "G4 FAIL: app process is not alive after launch" >&2
  echo "logcat: $LOG_PATH" >&2
  exit 1
fi

if ! rg -q "Loaded gen_ui_ffi through System\\.loadLibrary" "$LOG_PATH"; then
  echo "G4 FAIL: libgen_ui_ffi load confirmation was not found in logcat" >&2
  echo "logcat: $LOG_PATH" >&2
  exit 1
fi

if ! rg -q "mobile migrations ready" "$LOG_PATH"; then
  echo "G4 FAIL: Rust mobile boot/migration ready signal was not found in logcat" >&2
  echo "logcat: $LOG_PATH" >&2
  exit 1
fi

if rg -q "FATAL EXCEPTION|tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|dlopen failed.*libgen_ui_ffi|liblitertlm.*dlopen failed|dlopen failed.*liblitertlm|JNI_OnLoad.*failed" "$LOG_PATH"; then
  echo "G4 FAIL: fatal or KnowMe native-load failure found in logcat" >&2
  rg -n "FATAL EXCEPTION|tools\\.knowme\\.knowme E AndroidRuntime|UnsatisfiedLinkError|libgen_ui_ffi.*dlopen failed|dlopen failed.*libgen_ui_ffi|liblitertlm.*dlopen failed|dlopen failed.*liblitertlm|JNI_OnLoad.*failed" "$LOG_PATH" >&2
  echo "logcat: $LOG_PATH" >&2
  exit 1
fi

echo "G3 PASS: APK installed on attached Android device"
echo "G4 PASS: app launched and process stayed alive pid=$PID"
echo "G4 PASS: libgen_ui_ffi loaded through System.loadLibrary"
echo "G4 PASS: Rust mobile boot reached migrations-ready state"
echo "G5-G6 remain model/runtime self-test gates; run a chat/model certification workflow separately"
echo "logcat: $LOG_PATH"
