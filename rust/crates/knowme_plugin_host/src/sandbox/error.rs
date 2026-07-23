//! Error taxonomy for the sandbox (spec: "Error taxonomy stability").
//!
//! Two types, one mapping:
//!
//! * [`SandboxError`] — the runtime-facing `thiserror` enum. Every failure of
//!   `PluginHost` (`load`, `instantiate`, `invoke_*`, `unload`) is one of
//!   these eight variants.
//! * [`HostError`] — the guest-facing nine-case WIT `host-error` variant.
//!   This is the crate-owned type every `bindgen!` `with:` table maps
//!   `knowme:plugin/host-types/host-error` onto, so error classification
//!   lives here and only here (design D1).
//!
//! The mapping [`SandboxError::to_host_error`] is spec-pinned (design D5):
//! out-of-fuel and memory-ceiling map to `budget-exceeded`, wall-clock to
//! `timeout`, traps to `internal` with the message preserved. Display
//! strings are pinned by the snapshot tests below — changing them is a
//! breaking change to the guest-visible contract.

/// The nine-case `host-error` variant from `wit/v1/types.wit` — the
/// bindgen-generated type owned by the canonical `hook` world module.
/// Display (via the generated Debug impl) and `std::error::Error` are
/// generated alongside it.
pub use crate::sandbox::bindings::hook::knowme::plugin::host_types::HostError;

/// Runtime errors of the sandbox host. Spec-pinned variant set:
/// compilation / instantiation / missing-capability / unsupported-world /
/// execution / out-of-fuel / memory-exceeded / timeout.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SandboxError {
    /// The component bytes failed to compile (validation or Cranelift).
    #[error("component compilation failed: {0}")]
    Compilation(String),
    /// The component failed to instantiate for reasons other than a
    /// missing capability (link failures, trap in the guest's start
    /// function, rejected `init`).
    #[error("component instantiation failed: {0}")]
    Instantiation(String),
    /// The component imports a host interface outside the granted
    /// capability set. Carries the WIT interface name (e.g. `host-net`).
    /// Raised before any guest code executes.
    #[error("missing capability: component imports `{0}` but the capability set does not grant it")]
    MissingCapability(String),
    /// The requested world is `unstable`-annotated and cannot be
    /// instantiated yet (`agent`, `provider`, `service`, `workflow`).
    #[error("unsupported world: `{0}` is marked unstable and cannot be instantiated yet")]
    UnsupportedWorld(String),
    /// Guest code trapped or returned an error during an invoke. The
    /// guest-visible trap/error message is preserved verbatim.
    #[error("execution failed: {0}")]
    Execution(String),
    /// The per-invocation fuel budget was exhausted (spec: maps to
    /// `host-error.budget-exceeded`).
    #[error("fuel budget exhausted before invocation completed")]
    OutOfFuel,
    /// The `ResourceLimiter` rejected a linear-memory growth past the
    /// configured ceiling (spec: maps to `host-error.budget-exceeded`).
    #[error("linear memory budget exceeded: limit is {limit_bytes} bytes")]
    MemoryExceeded { limit_bytes: u64 },
    /// The invocation exceeded its wall-clock budget and was interrupted
    /// by epoch interruption (spec: maps to `host-error.timeout`).
    #[error("wall-clock timeout after {elapsed_ms} ms (budget {budget_ms} ms)")]
    Timeout { elapsed_ms: u64, budget_ms: u64 },
}

impl SandboxError {
    /// The spec-pinned mapping into the WIT nine-case `host-error`.
    ///
    /// | SandboxError        | host-error case     |
    /// |---------------------|---------------------|
    /// | Compilation         | internal            |
    /// | Instantiation       | internal            |
    /// | MissingCapability   | capability-denied   |
    /// | UnsupportedWorld    | invalid-argument    |
    /// | Execution           | internal            |
    /// | OutOfFuel           | budget-exceeded     |
    /// | MemoryExceeded      | budget-exceeded     |
    /// | Timeout             | timeout             |
    ///
    /// Detail strings are the variant's `Display` output so the pinned
    /// message reaches the guest verbatim (librefang's string-contract
    /// discipline, adopted per design D5).
    pub fn to_host_error(&self) -> HostError {
        match self {
            SandboxError::Compilation(_) => HostError::Internal(self.to_string()),
            SandboxError::Instantiation(_) => HostError::Internal(self.to_string()),
            SandboxError::MissingCapability(_) => HostError::CapabilityDenied(self.to_string()),
            SandboxError::UnsupportedWorld(_) => HostError::InvalidArgument(self.to_string()),
            SandboxError::Execution(_) => HostError::Internal(self.to_string()),
            SandboxError::OutOfFuel => HostError::BudgetExceeded(self.to_string()),
            SandboxError::MemoryExceeded { .. } => HostError::BudgetExceeded(self.to_string()),
            SandboxError::Timeout { .. } => HostError::Timeout,
        }
    }
}

#[cfg(test)]
mod error_mapping_tests {
    use super::*;

    /// Pinned display strings — the guest-visible contract. If a test here
    /// fails, a display string changed; that is a breaking change.
    #[test]
    fn sandbox_error_display_strings_are_pinned() {
        let cases: Vec<(SandboxError, &str)> = vec![
            (
                SandboxError::Compilation("bad magic".into()),
                "component compilation failed: bad magic",
            ),
            (
                SandboxError::Instantiation("link missing".into()),
                "component instantiation failed: link missing",
            ),
            (
                SandboxError::MissingCapability("host-net".into()),
                "missing capability: component imports `host-net` but the capability set does not grant it",
            ),
            (
                SandboxError::UnsupportedWorld("workflow".into()),
                "unsupported world: `workflow` is marked unstable and cannot be instantiated yet",
            ),
            (
                SandboxError::Execution("wasm trap: unreachable".into()),
                "execution failed: wasm trap: unreachable",
            ),
            (
                SandboxError::OutOfFuel,
                "fuel budget exhausted before invocation completed",
            ),
            (
                SandboxError::MemoryExceeded {
                    limit_bytes: 64 * 1024 * 1024,
                },
                "linear memory budget exceeded: limit is 67108864 bytes",
            ),
            (
                SandboxError::Timeout {
                    elapsed_ms: 3050,
                    budget_ms: 3000,
                },
                "wall-clock timeout after 3050 ms (budget 3000 ms)",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    /// Pinned mapping: every SandboxError variant maps to exactly one
    /// documented host-error case (spec scenario "Mapping pinned").
    #[test]
    fn sandbox_error_maps_to_exactly_one_host_error_case() {
        let cases: Vec<(SandboxError, HostError)> = vec![
            (
                SandboxError::Compilation("c".into()),
                HostError::Internal("component compilation failed: c".into()),
            ),
            (
                SandboxError::Instantiation("i".into()),
                HostError::Internal("component instantiation failed: i".into()),
            ),
            (
                SandboxError::MissingCapability("host-net".into()),
                HostError::CapabilityDenied(
                    "missing capability: component imports `host-net` but the capability set does not grant it"
                        .into(),
                ),
            ),
            (
                SandboxError::UnsupportedWorld("agent".into()),
                HostError::InvalidArgument(
                    "unsupported world: `agent` is marked unstable and cannot be instantiated yet"
                        .into(),
                ),
            ),
            (
                SandboxError::Execution("trap".into()),
                HostError::Internal("execution failed: trap".into()),
            ),
            (
                SandboxError::OutOfFuel,
                HostError::BudgetExceeded(
                    "fuel budget exhausted before invocation completed".into(),
                ),
            ),
            (
                SandboxError::MemoryExceeded { limit_bytes: 1024 },
                HostError::BudgetExceeded(
                    "linear memory budget exceeded: limit is 1024 bytes".into(),
                ),
            ),
            (
                SandboxError::Timeout {
                    elapsed_ms: 10,
                    budget_ms: 5,
                },
                HostError::Timeout,
            ),
        ];
        for (sandbox, expected) in cases {
            assert_eq!(sandbox.to_host_error(), expected);
        }
    }

    /// Pinned display strings for the guest-facing variant. The generated
    /// Display impl delegates to the generated Debug impl (wasmtime 46
    /// bindgen), so these pin THAT format: `HostError::<Case>` /
    /// `HostError::<Case>("<payload>")`.
    #[test]
    fn host_error_display_strings_are_pinned() {
        let cases: Vec<(HostError, &str)> = vec![
            (
                HostError::CapabilityDenied("no net".into()),
                "HostError::CapabilityDenied(\"no net\")",
            ),
            (
                HostError::PathDenied("../x".into()),
                "HostError::PathDenied(\"../x\")",
            ),
            (
                HostError::SsrfDenied("169.254.169.254".into()),
                "HostError::SsrfDenied(\"169.254.169.254\")",
            ),
            (
                HostError::PolicyDenied("ns".into()),
                "HostError::PolicyDenied(\"ns\")",
            ),
            (
                HostError::BudgetExceeded("fuel".into()),
                "HostError::BudgetExceeded(\"fuel\")",
            ),
            (HostError::Io("disk".into()), "HostError::Io(\"disk\")"),
            (
                HostError::InvalidArgument("bad".into()),
                "HostError::InvalidArgument(\"bad\")",
            ),
            (HostError::Timeout, "HostError::Timeout"),
            (
                HostError::Internal("boom".into()),
                "HostError::Internal(\"boom\")",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }
}
