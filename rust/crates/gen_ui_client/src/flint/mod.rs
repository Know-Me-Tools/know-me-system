// TJ-ARCH-MOB-001 compliant
//! Flint integration façade. One [`FlintClient`] wires the three planes together and
//! shares a single auth handle (a refreshed token is seen everywhere at once).
//!
//! `token` (JWT claims/lifecycle) is pure and cross-target. The IO planes (`gate`,
//! `forge`, `frf`) and the `FlintClient` façade are reqwest/tonic-driven and native-
//! only — on the browser the same planes are reached from JS (Connect-web / PGlite /
//! `@flint/react`) per the TJ-ARCH-MOB-001 layer contract.
pub mod token;

pub use token::{AuthState as FlintAuthState, Claims, Role, Token};

#[cfg(not(target_arch = "wasm32"))]
pub mod forge;
#[cfg(not(target_arch = "wasm32"))]
pub mod frf;
#[cfg(not(target_arch = "wasm32"))]
pub mod gate;

#[cfg(not(target_arch = "wasm32"))]
pub use client_impl::{FlintClient, FlintConfig};

#[cfg(not(target_arch = "wasm32"))]
mod client_impl {
    use super::forge::{ForgeClient, ForgeConfig, SharedAuth};
    use super::frf;
    use super::gate::{GateClient, GateConfig};
    use super::token::{AuthState, Role};
    use gen_ui_mcp::McpRegistry;
    use gen_ui_types::CoreResult;
    use std::sync::Arc;

    /// Everything needed to talk to a Flint deployment. Ports/paths carry verified
    /// defaults (gate :4456/:4457, forge :8080) — override per environment.
    #[derive(Debug, Clone)]
    pub struct FlintConfig {
        pub gate: GateConfig,
        pub forge: ForgeConfig,
        pub frf: Option<frf::FrfConfig>,
    }

    /// The single entry point the agent loop / db-sync lane depends on. Owns the shared
    /// auth state; `gate()`/`forge()` hand out plane clients bound to it.
    pub struct FlintClient {
        http: reqwest::Client,
        config: FlintConfig,
        auth: SharedAuth,
        registry: McpRegistry,
    }

    impl FlintClient {
        pub fn new(config: FlintConfig) -> Self {
            Self {
                http: reqwest::Client::new(),
                config,
                auth: Arc::new(parking_lot::RwLock::new(AuthState::Unauthenticated)),
                registry: McpRegistry::new(),
            }
        }

        pub fn gate(&self) -> GateClient {
            GateClient::new(self.http.clone(), self.config.gate.clone())
        }

        pub fn forge(&self) -> ForgeClient {
            ForgeClient::new(
                self.http.clone(),
                self.config.forge.clone(),
                self.auth.clone(),
            )
        }

        pub fn registry(&self) -> &McpRegistry {
            &self.registry
        }

        /// Boot with the static anon key so unauthenticated (public) surfaces work
        /// immediately; a later Kratos exchange upgrades the same shared auth handle.
        pub fn boot_anon(&self) -> CoreResult<()> {
            let state = self.gate().boot_anon()?;
            *self.auth.write() = state;
            Ok(())
        }

        /// Exchange a Kratos session for an authenticated/agent JWT and store it.
        pub async fn login_with_kratos(&self, kratos_cookie: &str) -> CoreResult<Role> {
            let state = self.gate().exchange_kratos_session(kratos_cookie).await?;
            let role = state.role();
            *self.auth.write() = state;
            Ok(role)
        }

        /// Register forge's A2UI registry as an MCP server into this client's registry.
        pub fn register_forge_mcp(&self) -> Arc<gen_ui_mcp::McpServerHandle> {
            self.forge().register_a2ui_mcp(&self.registry)
        }

        pub fn auth_role(&self) -> Role {
            self.auth.read().role()
        }
    }
} // mod client_impl (native-only)
