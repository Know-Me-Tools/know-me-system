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
