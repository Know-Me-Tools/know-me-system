fn main() {
    if let Ok(dir) = std::env::var("DEP_LLAMA_BACKENDS_DIR") {
        println!("cargo:rustc-env=GGML_BACKENDS_DIR={}", dir);
    }

    // `llama-cpp-sys-2` builds cpp-httplib as a private CMake target when
    // the `common` feature is enabled. Cargo does not receive that private
    // transitive link item, so the common wrapper reaches the final iOS link
    // without the implementation of `httplib::Client`. Locate the sys crate's
    // sibling build output and forward the static library explicitly. The
    // search is scoped to this dependency's Cargo target directory and works
    // for both Flutter's simulator and device target directories.
    let mut cursor = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").expect("OUT_DIR must be set for llama-cpp-2"),
    );
    while let Some(parent) = cursor.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if !name.to_string_lossy().starts_with("llama-cpp-sys-2-") {
                    continue;
                }
                let httplib_dir = entry
                    .path()
                    .join("out/build/vendor/cpp-httplib");
                if httplib_dir.join("libcpp-httplib.a").is_file() {
                    println!("cargo:rustc-link-search=native={}", httplib_dir.display());
                    println!("cargo:rustc-link-lib=static=cpp-httplib");
                    return;
                }
            }
        }
        cursor = parent.to_path_buf();
    }
}
