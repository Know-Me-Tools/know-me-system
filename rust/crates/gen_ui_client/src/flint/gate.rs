// TJ-ARCH-MOB-001 compliant
//! flint-gate client — auth boot, Kratos session exchange, Cedar approval polling.
//!
//! VERIFIED contract (gate HEAD 2026-07-15):
//!  * There is NO anon-token issuance endpoint. `FLINT_ANON_KEY` is a static,
//!    pre-shared publishable JWT — boot = hold it and send `Authorization: Bearer`.
//!  * Kratos is proxied: gate resolves a session via `GET /sessions/whoami` with the
//!    `ory_kratos_session` cookie; the authenticated/agent JWT is then minted by gate
//!    per-request. No mint endpoint, no refresh endpoint (refresh = re-auth).
//!  * `@require_approval` (human-in-the-loop) surfaces via the ADMIN approvals API
//!    (`/approvals/:id` → `decision` field, null = pending). No `isApprovalRequired`
//!    boolean and no per-request status/header at this HEAD.
use crate::flint::token::{AuthState, Token};
use gen_ui_types::{CoreError, CoreResult};
use serde::Deserialize;

/// Gate endpoints. Proxy (:4456) carries app traffic; admin (:4457) is private and
/// only reachable from trusted backends — the approvals poll is an admin call.
#[derive(Debug, Clone)]
pub struct GateConfig {
    pub proxy_base: String,
    pub admin_base: Option<String>,
    /// Static publishable anon key (FLINT_ANON_KEY).
    pub anon_key: Option<String>,
}

pub struct GateClient {
    http: reqwest::Client,
    config: GateConfig,
}

/// A pending Cedar approval as surfaced by the admin API. `decision` is `None` while
/// pending; a caller polls until it flips to approved/rejected.
#[derive(Debug, Clone, Deserialize)]
pub struct ApprovalStatus {
    pub id: String,
    /// "approved" | "rejected" | null(pending).
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

impl ApprovalStatus {
    pub fn is_pending(&self) -> bool {
        self.decision.is_none()
    }
    pub fn is_approved(&self) -> bool {
        self.decision.as_deref() == Some("approved")
    }
}

impl GateClient {
    pub fn new(http: reqwest::Client, config: GateConfig) -> Self {
        Self { http, config }
    }

    /// Boot with the static anon key. Errors if none is configured — a client with no
    /// credential cannot reach RLS-guarded planes.
    pub fn boot_anon(&self) -> CoreResult<AuthState> {
        let key = self
            .config
            .anon_key
            .as_deref()
            .ok_or_else(|| CoreError::Terminal("flint: no FLINT_ANON_KEY configured".into()))?;
        let token = Token::parse(key)?;
        Ok(AuthState::Anon { token })
    }

    /// Exchange a Kratos session cookie for an authenticated/agent JWT.
    ///
    /// Gate resolves the session (`/sessions/whoami`) and mints the outbound JWT via
    /// its `claims_enhancement` hook; we read the minted token from the response. The
    /// exact carrier (body field vs `Authorization` on the response) is route-config
    /// dependent — we accept both shapes.
    pub async fn exchange_kratos_session(&self, kratos_cookie: &str) -> CoreResult<AuthState> {
        let url = format!(
            "{}/sessions/whoami",
            self.config.proxy_base.trim_end_matches('/')
        );
        let resp = self
            .http
            .get(&url)
            .header(
                reqwest::header::COOKIE,
                format!("ory_kratos_session={kratos_cookie}"),
            )
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED
            || resp.status() == reqwest::StatusCode::FORBIDDEN
        {
            return Err(CoreError::Terminal("flint: kratos session invalid".into()));
        }
        if !resp.status().is_success() {
            return Err(CoreError::Transient(format!(
                "flint gate whoami http {}",
                resp.status()
            )));
        }

        // Preferred: minted JWT echoed on a response header.
        if let Some(hv) = resp.headers().get(reqwest::header::AUTHORIZATION) {
            if let Ok(s) = hv.to_str() {
                let raw = s.strip_prefix("Bearer ").unwrap_or(s);
                let token = Token::parse(raw)?;
                return Ok(AuthState::Authenticated { token });
            }
        }
        // Fallback: JSON body { "token": "..." } or { "jwt": "..." }.
        #[derive(Deserialize)]
        struct MintBody {
            #[serde(alias = "jwt")]
            token: Option<String>,
        }
        let body: MintBody = resp
            .json()
            .await
            .map_err(|e| CoreError::Serde(e.to_string()))?;
        let raw = body
            .token
            .ok_or_else(|| CoreError::Terminal("flint: gate returned no minted token".into()))?;
        Ok(AuthState::Authenticated {
            token: Token::parse(raw)?,
        })
    }

    /// Poll a Cedar approval by id (admin API). One shot — the caller drives the wait
    /// loop with the runtime's timer so this stays IO-only and testable.
    pub async fn approval_status(&self, approval_id: &str) -> CoreResult<ApprovalStatus> {
        let admin =
            self.config.admin_base.as_deref().ok_or_else(|| {
                CoreError::Terminal("flint: no gate admin_base for approvals".into())
            })?;
        let url = format!("{}/approvals/{approval_id}", admin.trim_end_matches('/'));
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Transient(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CoreError::NotFound(format!("approval {approval_id}")));
        }
        if !resp.status().is_success() {
            return Err(CoreError::Transient(format!(
                "flint approval http {}",
                resp.status()
            )));
        }
        resp.json()
            .await
            .map_err(|e| CoreError::Serde(e.to_string()))
    }
}
