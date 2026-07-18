// TJ-ARCH-MOB-001 compliant
//! wasm runtime: browser has no threads; drive futures on the JS microtask queue.
pub fn spawn<F>(f: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f);
}
