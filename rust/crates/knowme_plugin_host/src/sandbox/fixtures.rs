//! Test-time fixture builder (tasks 7.1–7.3).
//!
//! The guest fixtures under `tests/fixtures/` are standalone
//! wasm32-wasip2 crates. Building them at *test* time (via `Once`) keeps
//! `cargo check` / non-test builds free of guest compilation; every
//! artifact is `wasm-tools validate`d before use. Tests fail loudly when a
//! fixture cannot be built or validated — no silent skips.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

static BUILD: Once = Once::new();

/// Bytes of a built fixture component (e.g. `"knowme_fixture_hook"`).
pub fn fixture_bytes(name: &str) -> Vec<u8> {
    BUILD.call_once(build_all);
    let path = wasm_path(name);
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("fixture {name} missing at {}: {e}", path.display()))
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn wasm_path(name: &str) -> PathBuf {
    fixtures_root()
        .join("target/wasm32-wasip2/release")
        .join(format!("{name}.wasm"))
}

fn build_all() {
    // (crate dir, package/wasm name)
    let crates = [
        ("hook_guest", "knowme_fixture_hook"),
        ("contributor_guest", "knowme_fixture_contributor"),
        ("net_importer_guest", "knowme_fixture_net_importer"),
        ("http_types_guest", "knowme_fixture_http_types"),
    ];
    for (krate, wasm_name) in crates {
        let dir = fixtures_root().join(krate);
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&dir)
            .env("CARGO_TARGET_DIR", fixtures_root().join("target"))
            .status()
            .expect("spawn fixture build");
        assert!(status.success(), "fixture crate {krate} failed to build");

        let wasm = wasm_path(wasm_name);
        let valid = Command::new("wasm-tools")
            .arg("validate")
            .arg(&wasm)
            .status()
            .expect("spawn wasm-tools");
        assert!(
            valid.success(),
            "wasm-tools validate failed for {krate} at {}",
            wasm.display()
        );
    }
}
