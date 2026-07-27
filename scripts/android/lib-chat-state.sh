#!/usr/bin/env bash

# Shared, reversible chat-state isolation for Android physical-device
# certification. This intentionally touches only app-private conversation
# databases through `run-as`; model downloads and runtime caches are preserved.

CERT_CLEAN_CHAT_STATE="${CERT_CLEAN_CHAT_STATE:-0}"
CERT_CLEAN_CHAT_STATE_BACKUP_DIR="${CERT_CLEAN_CHAT_STATE_BACKUP_DIR:-cache/knowme-cert-chat-state-backup-$$}"
CERT_CHAT_STATE_PATHS=(
  "app_flutter/entity-records.sqlite3"
  "app_flutter/knowme-poc.db"
)

# ---------------------------------------------------------------------------
# OFFLINE ENFORCEMENT
#
# WHY A PROBE AND NOT JUST A FLAG
#
# `svc wifi disable` alone does NOT take the device offline. Measured on the
# SM-S936U: with `wifi_on` reading 0, `ping 8.8.8.8` still returned 0% packet
# loss over mobile data. A gate that flipped Wi-Fi and declared itself offline
# would certify nothing at all — which is precisely the hole this closes.
#
# So: disable BOTH radios, then PROVE unreachability before any assertion runs.
# If the probe still reaches the internet, the run aborts rather than reporting
# an offline pass it did not earn.
#
# Reversible: the prior radio state is captured and restored, because leaving
# an operator's phone with its radios off is a side effect this has no business
# causing.
CERT_REQUIRE_OFFLINE="${CERT_REQUIRE_OFFLINE:-0}"
CERT_OFFLINE_PROBE_HOST="${CERT_OFFLINE_PROBE_HOST:-8.8.8.8}"
_CERT_WIFI_WAS=""
_CERT_DATA_WAS=""

cert_network_reachable() {
  # 0 = reachable. Deliberately short: a slow-but-working network is still a
  # network, and a long timeout would make the gate crawl.
  adb shell ping -c 2 -W 2 "$CERT_OFFLINE_PROBE_HOST" 2>/dev/null \
    | rg -q "[1-9][0-9]* received"
}

cert_enter_offline() {
  [[ "$CERT_REQUIRE_OFFLINE" == "1" ]] || return 0

  _CERT_WIFI_WAS="$(adb shell settings get global wifi_on 2>/dev/null | tr -d '\r')"
  _CERT_DATA_WAS="$(adb shell settings get global mobile_data 2>/dev/null | tr -d '\r')"

  adb shell svc wifi disable >/dev/null 2>&1 || true
  adb shell svc data disable >/dev/null 2>&1 || true
  sleep 5

  if cert_network_reachable; then
    echo "OFFLINE FAIL: the device still reaches $CERT_OFFLINE_PROBE_HOST" >&2
    echo "  both radios were disabled and the network is STILL up, so nothing" >&2
    echo "  offline can be certified. Refusing to report a pass." >&2
    cert_exit_offline
    return 1
  fi

  # Toggling the radios can raise a One UI connectivity panel
  # (com.android.settings/.SubSettings) that STEALS FOREGROUND. Measured: a gate
  # then typed its prompt into Settings instead of the app and reported "timed
  # out waiting for generation" with zero engine lines — a failure that looks
  # like a broken model but is really a stolen window. It is intermittent, which
  # is why some runs passed and others did not.
  local top
  top="$(adb shell dumpsys activity activities 2>/dev/null \
         | rg -m1 -o 'topResumedActivity=[^ ]* [^ ]* ([a-zA-Z0-9_.]+)/' -r '$1' | tr -d '\r')"
  if [[ "$top" == "com.android.settings" ]]; then
    echo "  dismissing the settings panel that took foreground after the toggle"
    adb shell input keyevent KEYCODE_HOME >/dev/null 2>&1 || true
    sleep 2
  fi

  echo "  OFFLINE ENFORCED: $CERT_OFFLINE_PROBE_HOST unreachable (both radios down)"
}

cert_exit_offline() {
  [[ "$CERT_REQUIRE_OFFLINE" == "1" ]] || return 0
  [[ "$_CERT_WIFI_WAS" == "1" ]] && adb shell svc wifi enable >/dev/null 2>&1 || true
  [[ "$_CERT_DATA_WAS" == "1" ]] && adb shell svc data enable >/dev/null 2>&1 || true
}

cert_prepare_clean_chat_state() {
  local package="$1"
  local state_path basename

  if [[ "$CERT_CLEAN_CHAT_STATE" != "1" ]]; then
    return 0
  fi

  adb shell am force-stop "$package" >/dev/null 2>&1 || true
  if ! adb shell run-as "$package" mkdir -p "$CERT_CLEAN_CHAT_STATE_BACKUP_DIR" >/dev/null 2>&1; then
    echo "G6 FAIL: could not create reversible chat-state backup directory with run-as" >&2
    echo "package must be debuggable for CERT_CLEAN_CHAT_STATE=1" >&2
    return 1
  fi

  for state_path in "${CERT_CHAT_STATE_PATHS[@]}"; do
    if adb shell run-as "$package" ls "$state_path" >/dev/null 2>&1; then
      basename="$(basename "$state_path")"
      if ! adb shell run-as "$package" mv "$state_path" "$CERT_CLEAN_CHAT_STATE_BACKUP_DIR/$basename" >/dev/null 2>&1; then
        echo "G6 FAIL: could not move chat-state path into backup" >&2
        echo "path: $state_path" >&2
        return 1
      fi
    fi
  done
}

cert_restore_clean_chat_state() {
  local package="$1"
  local state_path basename

  if [[ "$CERT_CLEAN_CHAT_STATE" != "1" ]]; then
    return 0
  fi

  adb shell am force-stop "$package" >/dev/null 2>&1 || true
  for state_path in "${CERT_CHAT_STATE_PATHS[@]}"; do
    basename="$(basename "$state_path")"
    adb shell run-as "$package" rm -rf "$state_path" >/dev/null 2>&1 || true
    adb shell run-as "$package" mv "$CERT_CLEAN_CHAT_STATE_BACKUP_DIR/$basename" "$state_path" >/dev/null 2>&1 || true
  done
  adb shell run-as "$package" rm -rf "$CERT_CLEAN_CHAT_STATE_BACKUP_DIR" >/dev/null 2>&1 || true
}
