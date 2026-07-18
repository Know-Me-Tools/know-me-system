// TJ-ARCH-MOB-001 compliant
//! flint-realtime-fabric (FRF) Spine wrapper — feature = "frf", native-only.
//!
//! FRF is tonic gRPC over HTTP/2, which does not build for wasm32-unknown-unknown;
//! the browser surface uses frf-wasm / Connect-web from the JS side instead. This
//! module is compiled only when the `frf` feature is on AND the target is not wasm.
//!
//! VERIFIED (FRF HEAD 2026-07-15): SDK crate `frf-sdk-rust`, entry `FrfClient::connect
//! (endpoint, token)`; `SpineService` publish/subscribe(channel_id,consumer_id,from)/
//! ack over `EventEnvelope`; `EntityService::WatchEntity` streams `EntityChange`.
//! Peer-sync (feature = "peer-crdt") layers `frf-crdt` (Loro) + `frf-store-redb`.
//!
//! We keep this to a thin façade over the SDK so a `SyncTransport` impl (gen_ui_db::
//! sync, C-005) or the agent loop can drive the spine without re-learning proto types.

use gen_ui_types::sync::SyncStatus;

/// FRF connection parameters. `token` is the gate-minted Bearer the SDK's
/// `AuthInterceptor` injects on every RPC.
#[derive(Debug, Clone)]
pub struct FrfConfig {
    pub endpoint: String,
    pub token: Option<String>,
    pub tenant_id: String,
}

/// Thin handle around `frf_sdk_rust::FrfClient`. Constructed lazily by the façade so
/// a build without a reachable spine (offline-first boot) does not fail at startup.
pub struct FrfSpine {
    config: FrfConfig,
    #[cfg(feature = "frf")]
    client: parking_lot::Mutex<Option<frf_sdk_rust::FrfClient>>,
    status: parking_lot::RwLock<SyncStatus>,
}

impl FrfSpine {
    pub fn new(config: FrfConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "frf")]
            client: parking_lot::Mutex::new(None),
            status: parking_lot::RwLock::new(SyncStatus::Offline),
        }
    }

    pub fn status(&self) -> SyncStatus {
        self.status.read().clone()
    }

    pub fn tenant_id(&self) -> &str {
        &self.config.tenant_id
    }

    /// Connect (or reconnect) the Spine client. Only compiled with `frf`.
    #[cfg(feature = "frf")]
    pub async fn connect(&self) -> gen_ui_types::CoreResult<()> {
        use gen_ui_types::CoreError;
        let client = frf_sdk_rust::FrfClient::connect(
            self.config.endpoint.clone(),
            self.config.token.clone(),
        )
        .await
        .map_err(|e| CoreError::Transient(format!("frf connect: {e}")))?;
        *self.client.lock() = Some(client);
        *self.status.write() = SyncStatus::Live;
        Ok(())
    }

    /// Placeholder connect for builds without the `frf` feature — the offline path.
    #[cfg(not(feature = "frf"))]
    pub async fn connect(&self) -> gen_ui_types::CoreResult<()> {
        Err(gen_ui_types::CoreError::Terminal(
            "frf feature not enabled (native-only spine)".into(),
        ))
    }
}

// Peer CRDT op-log lane (feature = "peer-crdt"): re-export the on-device store + Loro
// applier so gen_ui_db::sync (C-005) can build the OFP-style peer path without adding
// FRF as a direct dependency. Kept behind the feature so redb never enters the default
// dependency graph.
#[cfg(feature = "peer-crdt")]
pub mod peer {
    pub use frf_crdt::LoroDeltaApplier;
    pub use frf_store_redb::RedbOpStore;
}
