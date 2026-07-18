// TJ-ARCH-MOB-001 compliant
//! Native runtime: one global multi-thread Tokio per process (never create a second).
use once_cell::sync::OnceCell;
use tokio::runtime::{Builder, Handle, Runtime};

static RT: OnceCell<Runtime> = OnceCell::new();

pub fn init(worker_threads: Option<usize>) {
    let _ = RT.get_or_try_init(|| {
        let n = worker_threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get().max(2) - 1)
                .unwrap_or(4)
        });
        Builder::new_multi_thread()
            .worker_threads(n)
            .max_blocking_threads(8)
            .thread_name("gen-ui-worker")
            .enable_all()
            .build()
    });
}

pub fn handle() -> Handle {
    RT.get().expect("runtime not initialised").handle().clone()
}

pub fn spawn<F>(f: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    handle().spawn(f)
}

pub fn spawn_blocking<F, T>(f: F) -> tokio::task::JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    handle().spawn_blocking(f)
}
