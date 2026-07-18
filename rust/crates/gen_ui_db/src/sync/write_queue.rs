// TJ-ARCH-MOB-001 compliant
//! DIY write queue: durable local action log replayed through a [`WriteSink`]
//! (forge Quarry API) with idempotent keys, exponential backoff, and a poison
//! handler. Read-path is Electric; this is the write-path half of local-first.
//!
//! The in-memory queue here is the replay engine; durability (survive restart) is
//! delegated to the same [`LocalStore`]-backed action-log table C-003 owns — this
//! module keeps the seam so persistence wires in without changing the replay logic.
use super::config::SyncConfig;
use super::seam::{PendingWrite, WriteOutcome, WriteSink};
use super::status::SyncStatusHandle;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Backoff schedule for transient write failures (capped exponential).
const BACKOFF_BASE: Duration = Duration::from_millis(200);
const BACKOFF_MAX: Duration = Duration::from_secs(30);

fn backoff_for(attempt: u32) -> Duration {
    // 200ms, 400ms, 800ms, … capped at 30s. saturating shift avoids overflow panics.
    let factor = 1u64.checked_shl(attempt.min(20)).unwrap_or(u64::MAX);
    BACKOFF_BASE
        .saturating_mul(factor.min(u32::MAX as u64) as u32)
        .min(BACKOFF_MAX)
}

pub(crate) struct WriteQueue {
    sink: Arc<dyn WriteSink>,
    status: SyncStatusHandle,
    max_attempts: u32,
    batch: usize,
    // tokio::Mutex: guard is held across the sink `.await`, so a std/parking_lot
    // mutex would be !Send here. Contention is low (one drain task + enqueues).
    pending: Mutex<VecDeque<PendingWrite>>,
    poison: Mutex<Vec<PendingWrite>>,
}

impl WriteQueue {
    pub(crate) fn new(
        cfg: &SyncConfig,
        sink: Arc<dyn WriteSink>,
        status: SyncStatusHandle,
    ) -> Self {
        Self {
            sink,
            status,
            max_attempts: cfg.max_write_attempts,
            batch: cfg.write_batch,
            pending: Mutex::new(VecDeque::new()),
            poison: Mutex::new(Vec::new()),
        }
    }

    /// Enqueue a local mutation for replay. Idempotency-keyed so retries dedupe
    /// server-side. Updates the pending-writes count that drives the UI chip.
    pub(crate) async fn enqueue(&self, write: PendingWrite) {
        let len = {
            let mut q = self.pending.lock().await;
            q.push_back(write);
            q.len() as u32
        };
        self.status.set_pending(len);
    }

    /// Drain up to `batch` writes, replaying each through the sink. Transient
    /// failures are re-queued (front) after a backoff sleep; poison writes are moved
    /// to the poison list and surfaced. Returns how many writes remain pending.
    pub(crate) async fn drain(&self) -> u32 {
        for _ in 0..self.batch {
            let Some(mut write) = ({ self.pending.lock().await.pop_front() }) else {
                break; // queue empty
            };

            match self.sink.send(&write).await {
                WriteOutcome::Applied => {
                    // dropped — the write succeeded (or idempotently deduped).
                }
                WriteOutcome::Retry => {
                    write.attempts += 1;
                    if write.attempts >= self.max_attempts {
                        self.quarantine(write, "max attempts exceeded").await;
                    } else {
                        let delay = backoff_for(write.attempts);
                        tracing::warn!(
                            key = %write.idempotency_key,
                            attempt = write.attempts,
                            ?delay,
                            "write retry scheduled"
                        );
                        tokio::time::sleep(delay).await;
                        self.pending.lock().await.push_front(write);
                    }
                }
                WriteOutcome::Poison { reason } => self.quarantine(write, &reason).await,
            }
        }

        let remaining = self.pending.lock().await.len() as u32;
        self.status.set_pending(remaining);
        remaining
    }

    async fn quarantine(&self, write: PendingWrite, reason: &str) {
        tracing::error!(
            key = %write.idempotency_key,
            table = %write.table,
            reason,
            "write quarantined (poison)"
        );
        self.poison.lock().await.push(write);
    }

    /// Snapshot of quarantined writes for UI surfacing / manual retry tooling.
    pub(crate) async fn poison_writes(&self) -> Vec<PendingWrite> {
        self.poison.lock().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::seam::WriteOutcome;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A sink whose verdict is scripted per attempt. Fake at the real IO boundary
    /// (the server), not a mock of internal code — per CLAUDE.md's testing rules.
    struct ScriptedSink {
        /// One outcome per attempt; the last repeats once exhausted.
        script: Vec<WriteOutcome>,
        calls: AtomicU32,
    }

    impl ScriptedSink {
        fn new(script: Vec<WriteOutcome>) -> Arc<Self> {
            Arc::new(Self {
                script,
                calls: AtomicU32::new(0),
            })
        }
        fn calls(&self) -> u32 {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl WriteSink for ScriptedSink {
        async fn send(&self, _w: &PendingWrite) -> WriteOutcome {
            let n = self.calls.fetch_add(1, Ordering::SeqCst) as usize;
            self.script
                .get(n)
                .or_else(|| self.script.last())
                .cloned()
                .unwrap()
        }
    }

    fn queue(sink: Arc<dyn WriteSink>, max_attempts: u32) -> WriteQueue {
        let cfg = SyncConfig {
            electric_url: String::new(),
            shapes: Vec::new(),
            write_batch: 8,
            max_write_attempts: max_attempts,
        };
        WriteQueue::new(&cfg, sink, SyncStatusHandle::new())
    }

    fn write(key: &str) -> PendingWrite {
        PendingWrite {
            idempotency_key: key.into(),
            table: "notes".into(),
            change_json: r#"{"op":"insert","id":"n1"}"#.into(),
            attempts: 0,
        }
    }

    /// The core offline→reconnect promise: a write that fails while the network is down
    /// must not be lost — it is retried and lands once the server answers.
    ///
    /// Note `drain()` loops up to `batch` times, so a re-queued write is re-attempted
    /// within the SAME drain call rather than waiting for the next one. The contract
    /// that matters is the outcome (the write lands, is never quarantined), not how many
    /// drains it takes — asserting the latter would be testing the implementation.
    #[tokio::test(start_paused = true)]
    async fn a_write_that_fails_offline_replays_when_the_server_returns() {
        let sink = ScriptedSink::new(vec![WriteOutcome::Retry, WriteOutcome::Applied]);
        let q = queue(sink.clone(), 5);
        q.enqueue(write("k1")).await;

        assert_eq!(q.drain().await, 0, "the write must land, not be dropped");
        assert_eq!(sink.calls(), 2, "it should have been retried exactly once");
        assert!(
            q.poison_writes().await.is_empty(),
            "a recoverable write must never be quarantined"
        );
    }

    /// A permanently-rejected write must NOT be retried forever — that wedges the
    /// queue behind one bad row and blocks every write after it.
    #[tokio::test(start_paused = true)]
    async fn a_transient_write_is_quarantined_once_attempts_run_out() {
        let sink = ScriptedSink::new(vec![WriteOutcome::Retry]);
        let q = queue(sink.clone(), 3);
        q.enqueue(write("k1")).await;

        // Drain until it gives up. Each pass burns one attempt.
        for _ in 0..3 {
            q.drain().await;
        }
        assert_eq!(q.drain().await, 0, "queue must not hold a give-up write");
        let poison = q.poison_writes().await;
        assert_eq!(poison.len(), 1, "it belongs in poison, not limbo");
        assert_eq!(poison[0].idempotency_key, "k1");
    }

    /// A terminal rejection (4xx) is poison on the FIRST attempt: replaying the same
    /// bytes gets the same answer, so retrying only delays the bad news.
    #[tokio::test(start_paused = true)]
    async fn a_terminal_rejection_poisons_immediately_without_burning_retries() {
        let sink = ScriptedSink::new(vec![WriteOutcome::Poison {
            reason: "400".into(),
        }]);
        let q = queue(sink.clone(), 5);
        q.enqueue(write("k1")).await;

        assert_eq!(q.drain().await, 0);
        assert_eq!(sink.calls(), 1, "must not retry a terminal rejection");
        assert_eq!(q.poison_writes().await.len(), 1);
    }

    /// The idempotency key must survive replay unchanged — it is the ONLY thing making
    /// a retried write safe (the server dedupes on it). A key that changed per attempt
    /// would turn every retry into a duplicate row.
    #[tokio::test(start_paused = true)]
    async fn the_idempotency_key_is_stable_across_replays() {
        struct KeySpy {
            seen: Mutex<Vec<String>>,
        }
        #[async_trait::async_trait]
        impl WriteSink for KeySpy {
            async fn send(&self, w: &PendingWrite) -> WriteOutcome {
                let mut seen = self.seen.lock().await;
                seen.push(w.idempotency_key.clone());
                if seen.len() < 3 {
                    WriteOutcome::Retry
                } else {
                    WriteOutcome::Applied
                }
            }
        }
        let spy = Arc::new(KeySpy {
            seen: Mutex::new(Vec::new()),
        });
        let q = queue(spy.clone(), 5);
        q.enqueue(write("stable-key")).await;

        for _ in 0..3 {
            q.drain().await;
        }
        let seen = spy.seen.lock().await;
        assert_eq!(seen.len(), 3);
        assert!(
            seen.iter().all(|k| k == "stable-key"),
            "key drifted across replays: {seen:?}"
        );
    }

    /// Ordering: a retried write goes back to the FRONT, so a later write cannot
    /// overtake it. Out-of-order replay would apply an update before its insert.
    #[tokio::test(start_paused = true)]
    async fn a_retried_write_keeps_its_place_ahead_of_later_writes() {
        struct OrderSpy {
            seen: Mutex<Vec<String>>,
        }
        #[async_trait::async_trait]
        impl WriteSink for OrderSpy {
            async fn send(&self, w: &PendingWrite) -> WriteOutcome {
                let mut seen = self.seen.lock().await;
                seen.push(w.idempotency_key.clone());
                // Fail the first write once, then accept everything.
                if seen.len() == 1 {
                    WriteOutcome::Retry
                } else {
                    WriteOutcome::Applied
                }
            }
        }
        let spy = Arc::new(OrderSpy {
            seen: Mutex::new(Vec::new()),
        });
        let q = queue(spy.clone(), 5);
        q.enqueue(write("first")).await;
        q.enqueue(write("second")).await;

        q.drain().await;
        q.drain().await;

        let seen = spy.seen.lock().await;
        assert_eq!(seen[0], "first");
        assert_eq!(
            seen[1], "first",
            "the retried write must be re-tried before later ones"
        );
        assert_eq!(seen[2], "second");
    }

    // Boundary behavior: backoff grows exponentially from the base and never
    // exceeds the cap, even at absurd attempt counts (no overflow panic).
    #[test]
    fn backoff_is_exponential_and_capped() {
        assert_eq!(backoff_for(0), BACKOFF_BASE);
        assert_eq!(backoff_for(1), BACKOFF_BASE * 2);
        assert_eq!(backoff_for(2), BACKOFF_BASE * 4);
        assert_eq!(backoff_for(1000), BACKOFF_MAX);
        assert!(backoff_for(50) <= BACKOFF_MAX);
    }
}
