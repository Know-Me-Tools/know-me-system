// TJ-ARCH-MOB-001 compliant
// Re-export the wasm-pack bundler output. Populate pkg/ first:
//   npm run build:wasm   (runs rust/crates/gen_ui_wasm/build-wasm.sh)
// The generated pkg/ carries its own .d.ts; this wrapper gives a stable package
// name + a place to add JS-side ergonomics later.
export * from "../pkg/gen_ui_wasm";
