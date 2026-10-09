//! One host-owned peer service; no background replication scheduler.
use crate::{
    OfpError,
    consent::{ConsentStore, PeerConsent, now_ms},
};
use async_trait::async_trait;
use frf_p2p::{
    EndpointAddr, EndpointId, P2pError, PeerConfig, PeerIdentity, PeerSession, PeerTransport,
    SecretKey, TokenVerifier,
};
use frf_ports::CrdtStore;
use frf_store_redb::RedbOpStore;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path, sync::Arc};
use tokio::sync::Mutex;

pub fn credential_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

struct ConsentVerifier {
    store: Arc<ConsentStore>,
    local_endpoint: String,
    local_hash: String,
}
impl std::fmt::Debug for ConsentVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsentVerifier").finish_non_exhaustive()
    }
}
#[async_trait]
impl TokenVerifier for ConsentVerifier {
    async fn verify(&self, token: &str, endpoint: &str) -> Result<PeerIdentity, P2pError> {
        let state = self
            .store
            .state()
            .await
            .map_err(|_| P2pError::Unauthenticated("consent authority unavailable".into()))?;
        let hash = credential_hash(token);
        let authorized = if endpoint == self.local_endpoint {
            state.local_endpoint.as_deref() == Some(endpoint) && hash == self.local_hash
        } else {
            let now = now_ms()
                .map_err(|_| P2pError::Unauthenticated("validity clock unavailable".into()))?;
            ConsentStore::current_peer(&state, endpoint, now)
                .is_ok_and(|peer| peer.credential_hash == hash)
        };
        if !authorized {
            return Err(P2pError::Unauthenticated(
                "endpoint credential/current consent denied".into(),
            ));
        }
        Ok(PeerIdentity {
            endpoint_id: endpoint.into(),
            tenant_id: state.tenant,
            subject: state.owner,
        })
    }
}

pub struct PersonalPeerService {
    pub(crate) store: Arc<ConsentStore>,
    pub(crate) snapshots: Arc<dyn CrdtStore>,
    pub(crate) ops: RedbOpStore,
    pub(crate) transaction: Mutex<()>,
    transport: PeerTransport,
    credential: String,
    sessions: Mutex<BTreeMap<String, Arc<PeerSession>>>,
}

impl PersonalPeerService {
    /// The host supplies unlocked identity material. Never generate a replacement on restart.
    pub async fn bind(
        store: Arc<ConsentStore>,
        key: SecretKey,
        credential: String,
        snapshots: Arc<dyn CrdtStore>,
        op_path: impl AsRef<Path>,
    ) -> Result<Self, OfpError> {
        let state = store.state().await?;
        if state.local_endpoint.as_deref() != Some(key.public().to_string().as_str())
            || credential.len() < 32
        {
            return Err(OfpError::Authority(
                "explicit identity import and credential of at least 32 characters required".into(),
            ));
        }
        let transport = PeerTransport::bind(
            &PeerConfig {
                relay_enabled: false,
                mdns_discovery: false,
                paired_endpoints: state.peers.keys().cloned().collect(),
                ..PeerConfig::default()
            },
            Some(key),
        )
        .await?
        .with_verifier(Arc::new(ConsentVerifier {
            store: store.clone(),
            local_endpoint: state.local_endpoint.unwrap_or_default(),
            local_hash: credential_hash(&credential),
        }));
        let ops = RedbOpStore::open(op_path).map_err(|e| OfpError::Authority(e.to_string()))?;
        Ok(Self {
            store,
            snapshots,
            ops,
            transaction: Mutex::new(()),
            transport,
            credential,
            sessions: Mutex::new(BTreeMap::new()),
        })
    }

    pub fn address(&self) -> EndpointAddr {
        self.transport.address()
    }

    pub async fn grant(&self, consent: PeerConsent, expected: u64) -> Result<(), OfpError> {
        let id: EndpointId = consent
            .endpoint_id
            .parse()
            .map_err(|_| OfpError::Authority("invalid endpoint key".into()))?;
        self.store.grant(consent, expected).await?;
        self.transport.set_paired(id, true)?;
        Ok(())
    }

    pub async fn revoke(&self, endpoint: &str, expected: u64) -> Result<(), OfpError> {
        self.store.revoke(endpoint, expected).await?;
        if let Some(session) = self.sessions.lock().await.remove(endpoint) {
            session.connection().close(2u8.into(), b"consent revoked");
        }
        let id: EndpointId = endpoint
            .parse()
            .map_err(|_| OfpError::Authority("invalid endpoint key".into()))?;
        self.transport.set_paired(id, false)?;
        Ok(())
    }

    pub async fn connect(&self, address: EndpointAddr) -> Result<Arc<PeerSession>, OfpError> {
        let session = Arc::new(self.transport.connect(address, &self.credential).await?);
        self.sessions
            .lock()
            .await
            .insert(session.endpoint_id().into(), session.clone());
        Ok(session)
    }

    /// Admit one inbound peer. The host explicitly drives subsequent rounds.
    pub async fn accept(&self) -> Result<Arc<PeerSession>, OfpError> {
        let session = Arc::new(self.transport.accept(&self.credential).await?);
        self.sessions
            .lock()
            .await
            .insert(session.endpoint_id().into(), session.clone());
        Ok(session)
    }

    pub async fn close(&self) {
        self.sessions.lock().await.clear();
        self.transport.close().await;
    }
}
