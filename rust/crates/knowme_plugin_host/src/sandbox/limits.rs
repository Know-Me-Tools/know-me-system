//! Resource limits (design D5): the `ResourceLimiter` memory ceiling and the
//! fuel-seeding convention.

/// `ResourceLimiter` capping WASM linear-memory growth at a byte ceiling
/// (default 64 MiB from `SandboxLimits`). Attached to every guest store.
///
/// When a growth is rejected the limiter records it: wasmtime only surfaces
/// the rejection to the guest (a failed `memory.grow`, usually followed by a
/// guest trap), so the host checks [`MemoryLimiter::take_denied`] after each
/// invocation to map the outcome to `SandboxError::MemoryExceeded`
/// (spec: memory ceiling → `host-error.budget-exceeded` naming the limit).
pub struct MemoryLimiter {
    max_bytes: usize,
    denied: bool,
}

impl MemoryLimiter {
    pub fn new(max_bytes: u64) -> Self {
        Self {
            max_bytes: max_bytes as usize,
            denied: false,
        }
    }

    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    /// Read and clear the denial flag. Called by the host around each
    /// invocation.
    pub fn take_denied(&mut self) -> bool {
        std::mem::take(&mut self.denied)
    }
}

impl wasmtime::ResourceLimiter for MemoryLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        let allowed = desired <= self.max_bytes;
        if !allowed {
            self.denied = true;
        }
        Ok(allowed)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        // Only linear memory is bounded here.
        Ok(true)
    }
}

/// Seed value for `Store::set_fuel` given a configured limit.
///
/// Convention (adopted from librefang per design D5): a configured limit of
/// 0 means "unlimited" — but wasmtime rejects values above `i64::MAX`, so
/// unlimited is seeded as `u64::MAX / 2` rather than `u64::MAX`.
pub fn fuel_seed(fuel_limit: u64) -> u64 {
    if fuel_limit == 0 {
        u64::MAX / 2
    } else {
        fuel_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::ResourceLimiter as _;

    #[test]
    fn memory_ceiling_allows_up_to_limit_and_records_denial() {
        let mut limiter = MemoryLimiter::new(64 * 1024 * 1024);
        assert!(limiter
            .memory_growing(0, 64 * 1024 * 1024, None)
            .expect("call"));
        assert!(!limiter.take_denied(), "no denial at exactly the ceiling");

        assert!(!limiter
            .memory_growing(0, 64 * 1024 * 1024 + 1, None)
            .expect("call"));
        assert!(limiter.take_denied(), "denial recorded past the ceiling");
        // Flag clears on take.
        assert!(!limiter.take_denied());
    }

    #[test]
    fn tables_are_unbounded() {
        let mut limiter = MemoryLimiter::new(1);
        assert!(limiter
            .table_growing(0, usize::MAX, None)
            .expect("call"));
    }

    #[test]
    fn fuel_zero_means_unlimited_seeded_below_wasmtime_max() {
        assert_eq!(fuel_seed(0), u64::MAX / 2);
        assert_eq!(fuel_seed(1_000_000), 1_000_000);
        assert_eq!(fuel_seed(1), 1);
    }
}
