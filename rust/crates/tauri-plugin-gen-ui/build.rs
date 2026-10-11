// TJ-ARCH-MOB-001 compliant
//! Tauri 2 plugin build script. The versioned command manifest is the sole
//! source for the Rust handler registration and Tauri permission allowlist.
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandManifest {
    schema_version: u32,
    commands: Vec<String>,
}

fn main() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("command-manifest.json");
    println!("cargo:rerun-if-changed={}", manifest_path.display());
    let manifest: CommandManifest =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read command-manifest.json"))
            .expect("parse command-manifest.json");
    assert_eq!(manifest.schema_version, 1, "unsupported command manifest");
    assert!(!manifest.commands.is_empty(), "command manifest is empty");

    let mut unique = manifest.commands.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        manifest.commands.len(),
        "command manifest contains duplicates"
    );

    let registrations = manifest
        .commands
        .iter()
        .map(|name| if name.contains("::") { name.clone() } else { format!("commands::{name}") })
        .collect::<Vec<_>>()
        .join(",\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(
        output.join("command_handler.rs"),
        format!("tauri::generate_handler![\n{registrations}\n]\n"),
    )
    .expect("write generated command handler");

    // tauri_plugin::Builder retains this allowlist for the build process and
    // therefore requires a static slice. The build script is short-lived, so
    // promoting the manifest-owned names here is bounded to this process.
    let command_refs: &'static [&'static str] = Box::leak(
        manifest
            .commands
            .into_iter()
            .map(|name| Box::leak(name.rsplit("::").next().expect("command name").to_owned().into_boxed_str()) as &'static str)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    tauri_plugin::Builder::new(command_refs)
        .android_path("android")
        .ios_path("ios")
        .build();
}
