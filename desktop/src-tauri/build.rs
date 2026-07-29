use std::path::Path;
use std::process::Command;

fn main() {
    build_frontend();
    tauri_build::build()
}

/// Build the React app so this crate owns its own frontend dependency.
///
/// WHY HERE AND NOT `beforeBuildCommand`
///
/// `frontendDist` (`../dist`) is embedded into the binary by
/// `tauri_build::build()` below, which makes the web bundle a genuine build
/// INPUT of this crate. Build inputs belong in `build.rs` — the only place
/// Cargo tracks them. With the config hook, `cargo build -p knowme-poc`
/// happily produced a binary carrying whatever stale `dist/` sat on disk,
/// because only the `tauri` CLI ever ran the hook; Cargo did not know a
/// dependency existed. The `cargo:rerun-if-changed` lines below are the point:
/// editing a `.tsx` file now correctly invalidates the Rust build.
///
/// WHY IT IS GATED
///
/// `build.rs` also runs for `cargo check` and rust-analyzer, where a Vite build
/// is pure waste — tens of seconds on every typecheck. `TAURI_ENV_TARGET_TRIPLE`
/// is exported by the `tauri` CLI (and read by `tauri_build` itself), so its
/// presence distinguishes "a real bundle is being produced" from "somebody is
/// typechecking" without sniffing rustc flags.
fn build_frontend() {
    let desktop = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri always has a parent directory");

    println!("cargo:rerun-if-changed={}/src", desktop.display());
    println!("cargo:rerun-if-changed={}/index.html", desktop.display());
    println!("cargo:rerun-if-changed={}/package.json", desktop.display());
    println!("cargo:rerun-if-changed={}/vite.config.ts", desktop.display());
    println!("cargo:rerun-if-env-changed=TAURI_ENV_TARGET_TRIPLE");
    println!("cargo:rerun-if-env-changed=KNOWME_SKIP_FRONTEND_BUILD");

    if std::env::var_os("KNOWME_SKIP_FRONTEND_BUILD").is_some() {
        return;
    }

    // Not a bundle — a bare `cargo check`/`cargo build` or rust-analyzer.
    // Typechecking Rust does not need a web bundle.
    if std::env::var_os("TAURI_ENV_TARGET_TRIPLE").is_none() {
        return;
    }

    let status = Command::new("pnpm")
        .arg("build")
        .current_dir(desktop)
        .status();

    match status {
        Ok(status) if status.success() => {}
        // FAIL LOUD. Continuing past a failed frontend build would embed a
        // stale or missing bundle and ship an app that launches to a blank
        // window — far harder to diagnose than a build error right here.
        Ok(status) => panic!("`pnpm build` failed with {status}; the embedded bundle would be stale"),
        Err(error) => panic!(
            "could not run `pnpm build` in {}: {error}. Install pnpm, or set \
             KNOWME_SKIP_FRONTEND_BUILD=1 to build against the existing dist/.",
            desktop.display()
        ),
    }
}
