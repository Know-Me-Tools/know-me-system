// TJ-ARCH-MOB-001 compliant
//! Token lifecycle for the gate auth flow.
//!
//! The gate mints short-lived (default 300s) JWTs; there is no refresh endpoint, so
//! "refresh" means re-authenticating from the held credential (anon key or Kratos
//! session). We model the ladder as a state enum so the caller cannot, e.g., call an
//! agent-scoped endpoint while still on the anon boot token.
//!
//! Payload parsing in this module is display-only. Authorization and trusted
//! tenant/role state come from a server-created `VerifiedSession` after JWKS or
//! authoritative session validation.
use gen_ui_types::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Display-only role hint. Never use this value for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Anon,
    Authenticated,
    Agent,
}

impl Role {
    fn from_claim(s: Option<&str>) -> Self {
        match s {
            Some("authenticated") => Role::Authenticated,
            Some("agent") => Role::Agent,
            // Service and unknown roles are deliberately not client-visible.
            _ => Role::Anon,
        }
    }
}

/// Unverified JWT payload hints for display and refresh timing only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UntrustedTokenHints {
    pub sub: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub exp: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl UntrustedTokenHints {
    pub fn role(&self) -> Role {
        Role::from_claim(self.role.as_deref())
    }
    /// An untyped extra claim by key (`act`, `agent_id`, `workflow_id`, ...).
    pub fn extra_str(&self, key: &str) -> Option<&str> {
        self.extra.get(key).and_then(|v| v.as_str())
    }
    /// Parse display-only hints without signature verification.
    pub fn parse_display_only(jwt: &str) -> CoreResult<Self> {
        jsonwebtoken::dangerous::insecure_decode::<UntrustedTokenHints>(jwt)
            .map(|d| d.claims)
            .map_err(|e| CoreError::Terminal(format!("jwt decode: {e}")))
    }
}

/// A bearer token plus untrusted display hints.
#[derive(Debug, Clone)]
pub struct Token {
    pub raw: String,
    pub hints: UntrustedTokenHints,
}

impl Token {
    const SKEW_SECS: i64 = 10;

    pub fn parse(raw: impl Into<String>) -> CoreResult<Self> {
        let raw = raw.into();
        let hints = UntrustedTokenHints::parse_display_only(&raw)?;
        Ok(Self { raw, hints })
    }

    /// Expired relative to `now_unix` (caller supplies time — this crate does no IO
    /// and stays wasm-safe; the runtime layer provides the clock).
    pub fn is_expired(&self, now_unix: i64) -> bool {
        match self.hints.exp {
            Some(exp) => now_unix + Self::SKEW_SECS >= exp,
            None => false,
        }
    }

    pub fn role(&self) -> Role {
        self.hints.role()
    }
}

/// The auth state machine. Illegal transitions (agent call while Anon) are
/// unrepresentable: a caller pattern-matches to get the active token.
#[derive(Debug, Clone, Default)]
pub enum AuthState {
    /// No credential yet.
    #[default]
    Unauthenticated,
    /// Booted with the static publishable anon key.
    Anon { token: Token },
    /// Exchanged a Kratos session for an authenticated/agent JWT.
    Authenticated { token: Token },
}

impl AuthState {
    /// The Bearer token to attach to an outbound request, if any.
    pub fn bearer(&self) -> Option<&str> {
        match self {
            AuthState::Unauthenticated => None,
            AuthState::Anon { token } | AuthState::Authenticated { token } => Some(&token.raw),
        }
    }

    /// Display-only role hint. Backend policy must use `VerifiedSession`.
    pub fn role(&self) -> Role {
        match self {
            AuthState::Unauthenticated | AuthState::Anon { .. } => Role::Anon,
            AuthState::Authenticated { token } => token.role(),
        }
    }

    /// True when the active token has expired and a re-auth is due.
    pub fn needs_refresh(&self, now_unix: i64) -> bool {
        match self {
            AuthState::Unauthenticated => false,
            AuthState::Anon { token } | AuthState::Authenticated { token } => {
                token.is_expired(now_unix)
            }
        }
    }
}
