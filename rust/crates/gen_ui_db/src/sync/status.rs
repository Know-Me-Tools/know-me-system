// TJ-ARCH-MOB-001 compliant
//! [`SyncStatus`] broadcast — the UI sync chip subscribes to this stream.
use gen_ui_types::sync::SyncStatus;

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::SyncStatus;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };
    use tokio::sync::watch;

    /// Publishes [`SyncStatus`] transitions. Cheap to clone (`Arc`-backed). A
    /// `watch` channel (not `broadcast`) because status is last-value-wins: a late
    /// subscriber wants the current state, not a replay of every past transition.
    #[derive(Clone)]
    pub struct SyncStatusHandle {
        tx: Arc<watch::Sender<SyncStatus>>,
        pending: Arc<AtomicU32>,
    }

    /// A live subscription to status transitions (`watch::Receiver`).
    pub type SyncStatusStream = watch::Receiver<SyncStatus>;

    impl SyncStatusHandle {
        pub fn new() -> Self {
            let (tx, _rx) = watch::channel(SyncStatus::Offline);
            Self {
                tx: Arc::new(tx),
                pending: Arc::new(AtomicU32::new(0)),
            }
        }

        /// Subscribe to status transitions (drives one UI chip).
        pub fn subscribe(&self) -> SyncStatusStream {
            self.tx.subscribe()
        }

        /// Current status snapshot.
        pub fn current(&self) -> SyncStatus {
            self.tx.borrow().clone()
        }

        pub(crate) fn set(&self, status: SyncStatus) {
            // send_replace ignores the "no receivers" error — status is still
            // readable via `borrow`/`current` even with nobody subscribed.
            let _ = self.tx.send_replace(status);
        }

        pub(crate) fn set_pending(&self, n: u32) {
            self.pending.store(n, Ordering::Relaxed);
            if n > 0 {
                self.set(SyncStatus::Syncing { pending_writes: n });
            }
        }

        pub(crate) fn pending(&self) -> u32 {
            self.pending.load(Ordering::Relaxed)
        }
    }

    impl Default for SyncStatusHandle {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::SyncStatus;
    use std::{cell::RefCell, rc::Rc};

    /// wasm stub — browser sync status comes from the JS `pglite-sync` subscription,
    /// surfaced to the UI on that side. Kept so the type name resolves on wasm32.
    /// (No `#[derive(Default)]`: `SyncStatus` is a frozen seam with no `Default`.)
    #[derive(Clone)]
    pub struct SyncStatusHandle(Rc<RefCell<SyncStatus>>);

    /// No stream on wasm (the JS side owns status). Alias keeps signatures uniform.
    pub type SyncStatusStream = ();

    impl SyncStatusHandle {
        pub fn new() -> Self {
            Self(Rc::new(RefCell::new(SyncStatus::Offline)))
        }
        pub fn subscribe(&self) -> SyncStatusStream {}
        pub fn current(&self) -> SyncStatus {
            self.0.borrow().clone()
        }
    }

    impl Default for SyncStatusHandle {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub use imp::{SyncStatusHandle, SyncStatusStream};
