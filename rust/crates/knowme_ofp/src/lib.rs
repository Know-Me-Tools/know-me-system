//! # knowme_ofp — Open Fang Protocol peer sync (FUNC-SPEC §07)
//!
//! OFP is the peer-to-peer gossip layer that lets the user's devices talk to
//! the user's own BossFang instances directly — no hub, no broker, no
//! KnowMe LLC server in the path. Transport is QUIC; application payloads are
//! encrypted with the user's identity key (a `did:key:` the user controls) and
//! peers authenticate with mutual TLS. Conflict resolution is structural:
//! Lamport clocks and CRDT-friendly merges, never "server says so".
//!
//! ## What is real in this skeleton
//! The sync domain model: instance types, peer roles, per-entity sync policy
//! (with the spec's conservative defaults), Lamport clocks, and the gossip
//! round summary that drives the Settings → Sync card in the UI.
//!
//! ## What is stubbed
//! The wire. QUIC transport, the gossip scheduler, payload encryption, and
//! CRDT merge against `gen_ui_db_graph` are tracked in
//! `docs/implementation-roadmap.md` (M2). The existing `gen_ui_types::sync`
//! module and `gen_ui_agent::sync_sink` are the integration points.
//!
//! Spec anchors: FUNC-SPEC §07 (BossFang Sync), UX-MOOD Screens 05–06.

use serde::{Deserialize, Serialize};

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
