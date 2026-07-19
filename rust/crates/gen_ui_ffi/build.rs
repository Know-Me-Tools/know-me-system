fn main() {
    // llama.cpp's Vulkan backend is linked into the mobile FFI library. Android
    // exposes Vulkan from the system library, but it is not pulled in
    // transitively from the static ggml archive, so declare the dependency on
    // the final cdylib explicitly.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo:rustc-link-lib=vulkan");
    }
}
