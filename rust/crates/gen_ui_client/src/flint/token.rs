// TJ-ARCH-MOB-001 compliant
//! Token lifecycle for the gate auth flow.
//!
//! The gate mints short-lived (default 300s) JWTs; there is no refresh endpoint, so
//! "refresh" means re-authenticating from the held credential (anon key or Kratos
//! session). We model the ladder as a state enum so the caller cannot, e.g., call an
//! agent-scoped endpoint while still on the anon boot token.
//!
//! Claims mirror flint-forge `forge-identity::Claims` EXACTLY: only `sub`/`role`/
//! `tenant_id` are first-class; everything else (`act`, `agent_id`, `workflow_id`,
//! `principal_type`, `scope`) rides the untyped `extra` map — the platform itself
//! keeps them untyped, so typing them here would be a fiction that drifts.
use gen_ui_types::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The role ladder. `service_role` bypasses Postgres RLS and never rides a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Anon,
    Authenticated,
    Agent,
    ServiceRole,
}

impl Role {
    fn from_claim(s: Option<&str>) -> Self {
        match s {
            Some("authenticated") => Role::Authenticated,
            Some("agent") => Role::Agent,
            Some("service_role") => Role::ServiceRole,
            // forge-identity coerces an absent/unknown role to "anon".
            _ => Role::Anon,
        }
    }
}

/// Decoded gate/forge JWT claims. Only the three typed fields are guaranteed; the
/// rest are read out of `extra` by key when a caller needs them (e.g. `agent_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
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

impl Claims {
    pub fn role(&self) -> Role {
        Role::from_claim(self.role.as_deref())
    }
    /// An untyped extra claim by key (`act`, `agent_id`, `workflow_id`, ...).
    pub fn extra_str(&self, key: &str) -> Option<&str> {
        self.extra.get(key).and_then(|v| v.as_str())
    }
    /// Decode WITHOUT signature verification — gate/forge own verification (JWKS).
    /// We only need to read `exp`/`role`/`tenant_id` to drive the client state.
    pub fn decode_unverified(jwt: &str) -> CoreResult<Self> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.required_spec_claims.clear();
        let key = jsonwebtoken::DecodingKey::from_secret(&[]);
        jsonwebtoken::decode::<Claims>(jwt, &key, &validation)
            .map(|d| d.claims)
            .map_err(|e| CoreError::Terminal(format!("jwt decode: {e}")))
    }
}

/// A token plus its decoded claims. `is_expired` uses `exp` with a small skew.
#[derive(Debug, Clone)]
pub struct Token {
    pub raw: String,
    pub claims: Claims,
}

impl Token {
    const SKEW_SECS: i64 = 10;

    pub fn parse(raw: impl Into<String>) -> CoreResult<Self> {
        let raw = raw.into();
        let claims = Claims::decode_unverified(&raw)?;
        Ok(Self { raw, claims })
    }

    /// Expired relative to `now_unix` (caller supplies time — this crate does no IO
    /// and stays wasm-safe; the runtime layer provides the clock).
    pub fn is_expired(&self, now_unix: i64) -> bool {
        match self.claims.exp {
            Some(exp) => now_unix + Self::SKEW_SECS >= exp,
            None => false,
        }
    }

    pub fn role(&self) -> Role {
        self.claims.role()
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
