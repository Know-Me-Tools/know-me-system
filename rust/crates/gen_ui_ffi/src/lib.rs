// TJ-ARCH-MOB-001 compliant
//! gen_ui_ffi (LEAF) — flutter_rust_bridge surface. Thin: editing app logic here
//! does not retrigger deep recompiles. Re-exports intent-level APIs + streams so
//! Dart never touches raw SurrealQL / SQL. Wave-1 lanes wire implementations
//! behind these signatures.
//!
//! Generated glue (`frb_generated.rs`) is produced by
//! `flutter_rust_bridge_codegen generate` and is gitignored — do not hand-edit.
pub mod api;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "system" fn JNI_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    if vm.is_null() {
        log::error!("failed to capture Android JavaVM for LiteRT-LM: null JavaVM pointer");
        return jni::sys::JNI_ERR;
    }
    // SAFETY: Android invokes JNI_OnLoad with its live, non-null process JavaVM.
    let java_vm = unsafe { jni::JavaVM::from_raw(vm) };
    gen_ui_inference::android_litert_jni::set_java_vm(java_vm);
    jni::sys::JNI_VERSION_1_6
}
