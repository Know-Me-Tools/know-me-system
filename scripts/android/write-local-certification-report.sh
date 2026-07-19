#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 3 ]]; then
  echo "usage: $0 /path/to/app.apk [package.name] [output.json]" >&2
  echo "env: CERT_REPORT_LAUNCH=1 CERT_REPORT_HASH_MODEL=1" >&2
  exit 64
fi

APK="$1"
PACKAGE="${2:-tools.knowme.knowme}"
OUTPUT="${3:-/tmp/knowme-android-certification-report.json}"
LAUNCH="${CERT_REPORT_LAUNCH:-1}"
HASH_MODEL="${CERT_REPORT_HASH_MODEL:-1}"

if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 66
fi

for tool in adb python3 shasum unzip; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "$tool not found in PATH" >&2
    exit 69
  fi
done

if [[ "$LAUNCH" == "1" ]]; then
  adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
  sleep "${CERT_REPORT_LAUNCH_WAIT_SECONDS:-5}"
fi

python3 - "$APK" "$PACKAGE" "$OUTPUT" "$HASH_MODEL" <<'PY'
import datetime as dt
import hashlib
import json
import os
import re
import subprocess
import sys
import zipfile
from pathlib import Path

apk = Path(sys.argv[1])
package = sys.argv[2]
output = Path(sys.argv[3])
hash_model = sys.argv[4] == "1"


def run(args, *, check=False, text=True):
    completed = subprocess.run(
        args,
        check=check,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=text,
    )
    if text:
        return completed.stdout.strip(), completed.stderr.strip(), completed.returncode
    return completed.stdout, completed.stderr, completed.returncode


def adb_shell(*parts):
    return run(["adb", "shell", *parts])[0]


def getprop(name):
    return adb_shell("getprop", name).replace("\r", "").strip()


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run_as(command):
    stdout, stderr, code = run(["adb", "exec-out", "run-as", package, "sh", "-c", command])
    return stdout.replace("\r", ""), stderr.replace("\r", ""), code


def parse_thermal(raw):
    status = None
    temps = []
    for line in raw.splitlines():
        match = re.search(r"Thermal Status:\s*(\d+)", line)
        if match:
            status = int(match.group(1))
        match = re.search(
            r"Temperature\{mValue=([^,]+), mType=([^,]+), mName=([^,]+), mStatus=([^}]+)\}",
            line,
        )
        if match:
            value = None
            try:
                value = float(match.group(1))
            except ValueError:
                pass
            temps.append(
                {
                    "valueCelsius": value,
                    "type": match.group(2),
                    "name": match.group(3),
                    "status": match.group(4),
                }
            )
    return {"status": status, "temperatures": temps}


def parse_meminfo(raw):
    result = {"processFound": "No process found" not in raw, "rawSummary": []}
    for line in raw.splitlines():
        if line.strip().startswith(("TOTAL", "Native Heap", "Dalvik Heap", "Graphics", "GL mtrack")):
            result["rawSummary"].append(line.strip())
    total = next((line for line in result["rawSummary"] if line.startswith("TOTAL")), None)
    if total:
        pieces = total.split()
        if len(pieces) > 1 and pieces[1].isdigit():
            result["totalPssKb"] = int(pieces[1])
    return result


def app_file_stat(paths):
    quoted = " ".join("'" + path.replace("'", "'\\''") + "'" for path in paths)
    script = (
        f"for p in {quoted}; do "
        'if [ -e "$p" ]; then '
        'size=$(stat -c %s "$p" 2>/dev/null || wc -c < "$p" 2>/dev/null || echo unknown); '
        'printf "%s|%s\\n" "$p" "$size"; '
        "fi; "
        "done"
    )
    stdout, _, _ = run_as(script)
    entries = []
    for line in stdout.splitlines():
        if "|" not in line:
            continue
        path, size = line.split("|", 1)
        entries.append({"path": path, "bytes": int(size) if size.isdigit() else None})
    return entries


def model_artifacts():
    stdout, _, _ = run_as("find app_flutter/model-cache/litert-lm -maxdepth 2 -type f -name '*.litertlm' -print 2>/dev/null")
    artifacts = []
    for path in [line.strip() for line in stdout.splitlines() if line.strip()]:
        size_out, _, _ = run_as(f"stat -c %s '{path}' 2>/dev/null || wc -c < '{path}' 2>/dev/null || true")
        artifact = {"path": path, "bytes": int(size_out) if size_out.isdigit() else None}
        if hash_model:
            hash_out, _, hash_code = run_as(f"sha256sum '{path}' 2>/dev/null || true")
            if hash_code == 0 and hash_out:
                artifact["sha256"] = hash_out.split()[0]
        artifacts.append(artifact)
    return artifacts


def known_artifacts():
    paths = sorted(
        str(path)
        for pattern in (
            "/tmp/knowme-local-*.log",
            "/tmp/knowme-local-*.xml",
            "/tmp/knowme-local-*.sqlite3",
        )
        for path in Path("/tmp").glob(Path(pattern).name)
    )
    records = []
    for path_str in paths:
        path = Path(path_str)
        try:
            records.append(
                {
                    "path": path_str,
                    "bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
        except FileNotFoundError:
            pass
    return records


def uar_metadata():
    git_root, _, root_code = run(["git", "rev-parse", "--show-toplevel"])
    if root_code != 0 or not git_root:
        git_root = os.getcwd()

    submodule_path = Path(git_root) / "rust" / "vendor" / "universal-agent-runtime"
    commit = None
    remote = None
    if submodule_path.exists():
        commit_out, _, commit_code = run(["git", "-C", str(submodule_path), "rev-parse", "HEAD"])
        remote_out, _, remote_code = run(["git", "-C", str(submodule_path), "remote", "get-url", "origin"])
        if commit_code == 0 and commit_out:
            commit = commit_out
        if remote_code == 0 and remote_out:
            remote = remote_out

    return {
        "runtimeAuthority": "universal-agent-runtime",
        "mobileModeTarget": "embedded_library",
        "currentMobileIntegration": "projection_only",
        "desktopWebModeTarget": "http_service_or_sidecar",
        "cloudModeTarget": "http_service",
        "localModelProviderRole": "knowme_local_provider_backend",
        "bossfangLibreFangMobileSidecar": False,
        "submodulePath": "rust/vendor/universal-agent-runtime",
        "submoduleCommit": commit,
        "submoduleRemote": remote,
    }


with zipfile.ZipFile(apk) as archive:
    native_libs = sorted(
        item.filename
        for item in archive.infolist()
        if item.filename.startswith("lib/") and item.filename.endswith(".so")
    )

thermal_raw = adb_shell("dumpsys", "thermalservice")
meminfo_raw = adb_shell("dumpsys", "meminfo", package)
pid = adb_shell("pidof", package)

report = {
    "schemaVersion": 1,
    "generatedAt": dt.datetime.now(dt.timezone.utc).isoformat(),
    "gitCommit": run(["git", "rev-parse", "HEAD"])[0] or None,
    "package": package,
    "apk": {
        "path": str(apk),
        "bytes": apk.stat().st_size,
        "sha256": sha256_file(apk),
        "nativeLibraries": native_libs,
    },
    "device": {
        "manufacturer": getprop("ro.product.manufacturer"),
        "model": getprop("ro.product.model"),
        "sdk": getprop("ro.build.version.sdk"),
        "release": getprop("ro.build.version.release"),
        "abi": getprop("ro.product.cpu.abi"),
        "soc": getprop("ro.soc.model"),
        "serial": run(["adb", "get-serialno"])[0],
    },
    "runtime": {
        "pid": pid or None,
        "expectedRuntimeKind": "litert-lm",
        "expectedBackend": "litert-lm-cpu",
        "modelArtifacts": model_artifacts(),
        "appPrivateFiles": app_file_stat(
            [
                "app_flutter/entity-records.sqlite3",
                "app_flutter/knowme-poc.db",
            ]
        ),
    },
    "uar": uar_metadata(),
    "memory": parse_meminfo(meminfo_raw),
    "thermal": parse_thermal(thermal_raw),
    "artifacts": known_artifacts(),
}

output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(output)
PY

echo "CERT REPORT PASS: wrote Android local certification report"
echo "report: $OUTPUT"
