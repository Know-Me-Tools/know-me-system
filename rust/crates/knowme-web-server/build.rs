// TJ-ARCH-MOB-001 compliant
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=KNOWME_WEB_DIST_DIR");
    println!("cargo:rerun-if-changed=../../../desktop/src");
    println!("cargo:rerun-if-changed=../../../desktop/package.json");
    println!("cargo:rerun-if-changed=../../../desktop/vite.config.ts");

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set")).join("knowme-web");
    if output.exists() {
        fs::remove_dir_all(&output).expect("remove stale embedded web output");
    }
    fs::create_dir_all(&output).expect("create embedded web output");

    if let Some(source) = env::var_os("KNOWME_WEB_DIST_DIR") {
        let source = PathBuf::from(source);
        require_index(&source);
        copy_tree(&source, &output).expect("copy KNOWME_WEB_DIST_DIR");
    } else {
        let frontend = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../desktop");
        let status = Command::new("pnpm")
            .args(["exec", "vite", "build", "--outDir"])
            .arg(&output)
            .arg("--emptyOutDir")
            .current_dir(&frontend)
            .status()
            .expect("run tracked Vite build; install frozen frontend dependencies first");
        assert!(status.success(), "tracked Vite build failed");
        require_index(&output);
    }

    println!(
        "cargo:rustc-env=KNOWME_EMBEDDED_WEB_DIR={}",
        output.display()
    );
}

fn require_index(directory: &Path) {
    assert!(
        directory.join("index.html").is_file(),
        "compiled web root must contain index.html: {}",
        directory.display()
    );
}

fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&target)?;
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
