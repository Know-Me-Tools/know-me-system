//! Personal-class OFP peer replication (AFC C12).
//!
//! Native services exchange endpoint-bound credentials over authenticated QUIC,
//! consume durable explicit consent, and merge selected personal Loro documents.
//! Authority records, organizations, runtime grants/policies and execution state
//! are excluded from the replicated document vocabulary.
//! The host drives bounded rounds; this crate installs no gossip scheduler.
//! Keys and credentials are session-only. Public identity, consent history,
//! validity and document tombstones persist independently of CRDT payloads.
//!
//! Historical instance/sync UI domain types below remain compatibility types;
//! their defaults grant no access through the personal replication service.

use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
pub mod consent;
#[cfg(not(target_arch = "wasm32"))]
mod replica;
#[cfg(not(target_arch = "wasm32"))]
pub mod service;
#[cfg(not(target_arch = "wasm32"))]
pub use frf_domain::{EntityId, TenantId};
#[cfg(not(target_arch = "wasm32"))]
pub use frf_ports::{CrdtSnapshot, CrdtStore, PortError};
#[cfg(not(target_arch = "wasm32"))]
pub use frf_p2p::{EndpointAddr, EndpointId, SecretKey};

/// The four instance types (FUNC-SPEC §07 table).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum InstanceKind {
    /// Primary — the local KnowMe install, in-process.
    ThisDevice,
    /// Personal peer (e.g. a home Mac Pro). Larger models, always-on Hands.
    PersonalPeer { url: String },
    /// Rented VM the user owns. Off-device runtime while mobile.
    PersonalCloud { url: String },
    /// Multi-user shared instance. Org-controlled, member-scoped data.
    Team { url: String },
}

/// Role pills shown in Settings → Instances: ember "owner" vs cyan "member".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PeerRole {
    Owner,
    Member,
}

/// A configured peer row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInstance {
    pub id: String,
    pub name: String,
    pub kind: InstanceKind,
    pub role: PeerRole,
    pub reachable: bool,
}

/// Entity classes that can cross the wire, each with a default sync policy
/// (FUNC-SPEC §07, "What syncs, by default").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncEntityKind {
    /// Default ON — definitions and schedules; run history stays per-instance.
    Hands,
    /// Default ON — AES-256-GCM encrypted at rest and in transit.
    ConversationHistory,
    /// Default ON — the personal knowledge graph; CRDT merge.
    Memory,
    /// Default ON — MCP servers, Cedar policies, grants.
    SkillRegistrations,
    /// Default OFF — GGUF files are 1–20 GB; Wi-Fi only when enabled.
    Models,
    /// Default per-device — theme/breakpoints rarely want to follow.
    Settings,
}

impl SyncEntityKind {
    pub fn default_enabled(&self) -> bool {
        !matches!(self, Self::Models | Self::Settings)
    }
}

/// Lamport clock carried by every synced entity (FUNC-SPEC §07, gossip section).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportClock(pub u64);

impl LamportClock {
    pub fn tick(&mut self) -> Self {
        self.0 += 1;
        *self
    }
    pub fn observe(&mut self, other: Self) {
        self.0 = self.0.max(other.0) + 1;
    }
}

/// Summary of the most recent gossip round — drives the Sync card
/// (23s · 7 Hands · 312 conversations · 2.4 MB in UX-MOOD Screen 06).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GossipRoundSummary {
    pub completed_at_ms: u64,
    pub peers: usize,
    pub hands: usize,
    pub conversations: usize,
    pub bytes_transferred: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum OfpError {
    #[error("personal peer authority denied: {0}")]
    Authority(String),
    #[error("personal peer storage: {0}")]
    Storage(#[from] std::io::Error),
    #[error("personal peer frame: {0}")]
    Codec(#[from] serde_json::Error),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("personal peer transport: {0}")]
    Transport(#[from] frf_p2p::P2pError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("personal peer persistence: {0}")]
    Persistence(#[from] frf_ports::PortError),
    #[error("peer unreachable: {0}")]
    PeerUnreachable(String),
    #[error("entity kind '{0:?}' is not enabled for sync")]
    SyncDisabled(SyncEntityKind),
    #[error("transport not wired yet — see docs/implementation-roadmap.md (M2)")]
    NotImplemented,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conservative_defaults_match_spec() {
        assert!(SyncEntityKind::Hands.default_enabled());
        assert!(SyncEntityKind::ConversationHistory.default_enabled());
        assert!(!SyncEntityKind::Models.default_enabled());
        assert!(!SyncEntityKind::Settings.default_enabled());
    }

    #[test]
    fn lamport_observe_advances_past_peer() {
        let mut a = LamportClock(3);
        a.observe(LamportClock(10));
        assert_eq!(a, LamportClock(11));
    }
}
