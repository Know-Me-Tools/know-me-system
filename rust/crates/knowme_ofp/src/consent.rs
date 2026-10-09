//! Durable local authority. Peer CRDT payloads can never mutate this ledger.
use crate::OfpError;
use frf_domain::{EntityId, TenantId};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum PersonalClass {
    PersonalNote,
    Conversation,
    Memory,
    PersonalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PersonalDocument {
    pub entity_id: EntityId,
    pub class: PersonalClass,
    pub tombstone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PeerConsent {
    pub endpoint_id: String,
    pub credential_hash: String,
    pub documents: Vec<EntityId>,
    pub valid_until_ms: u64,
    pub offline_valid_until_ms: u64,
    pub revision: u64,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentEvent {
    pub revision: u64,
    pub at_ms: u64,
    pub endpoint_id: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConsentState {
    pub version: u8,
    pub owner: String,
    pub tenant: TenantId,
    pub revision: u64,
    pub local_endpoint: Option<String>,
    pub peers: BTreeMap<String, PeerConsent>,
    pub documents: BTreeMap<String, PersonalDocument>,
    pub history: Vec<ConsentEvent>,
}

pub struct ConsentStore {
    path: PathBuf,
    pub(crate) gate: Mutex<()>,
}

impl std::fmt::Debug for ConsentStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsentStore").finish_non_exhaustive()
    }
}

pub fn now_ms() -> Result<u64, OfpError> {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| OfpError::Authority(e.to_string()))?
            .as_millis(),
    )
    .map_err(|e| OfpError::Authority(e.to_string()))
}

impl ConsentStore {
    pub async fn open(
        path: impl AsRef<Path>,
        owner: String,
        tenant: TenantId,
    ) -> Result<Self, OfpError> {
        if owner.trim().is_empty() {
            return Err(OfpError::Authority("owner required".into()));
        }
        let store = Self {
            path: path.as_ref().into(),
            gate: Mutex::new(()),
        };
        match tokio::fs::read(&store.path).await {
            Ok(bytes) => {
                let state: ConsentState = serde_json::from_slice(&bytes)?;
                if state.version != 1 || state.owner != owner || state.tenant != tenant {
                    return Err(OfpError::Authority("consent owner/tenant mismatch".into()));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                store
                    .persist(&ConsentState {
                        version: 1,
                        owner,
                        tenant,
                        revision: 0,
                        local_endpoint: None,
                        peers: BTreeMap::new(),
                        documents: BTreeMap::new(),
                        history: Vec::new(),
                    })
                    .await?;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(store)
    }

    pub(crate) async fn load(&self) -> Result<ConsentState, OfpError> {
        Ok(serde_json::from_slice(&tokio::fs::read(&self.path).await?)?)
    }
    pub(crate) async fn persist(&self, state: &ConsentState) -> Result<(), OfpError> {
        use tokio::io::AsyncWriteExt;
        let pending = self.path.with_extension("pending");
        let mut file = tokio::fs::File::create(&pending).await?;
        file.write_all(&serde_json::to_vec(state)?).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(pending, &self.path).await?;
        Ok(())
    }

    pub async fn state(&self) -> Result<ConsentState, OfpError> {
        let _guard = self.gate.lock().await;
        self.load().await
    }

    async fn mutate(
        &self,
        expected: u64,
        endpoint: &str,
        action: &str,
        update: impl FnOnce(&mut ConsentState) -> Result<(), OfpError>,
    ) -> Result<ConsentState, OfpError> {
        let _guard = self.gate.lock().await;
        let mut state = self.load().await?;
        if state.revision != expected {
            return Err(OfpError::Authority(
                "consent revision changed; refresh before editing".into(),
            ));
        }
        state.revision = state
            .revision
            .checked_add(1)
            .ok_or_else(|| OfpError::Authority("revision exhausted".into()))?;
        update(&mut state)?;
        state.history.push(ConsentEvent {
            revision: state.revision,
            at_ms: now_ms()?,
            endpoint_id: endpoint.into(),
            action: action.into(),
        });
        self.persist(&state).await?;
        Ok(state)
    }

    /// First import records a public identity. Subsequent imports must be the same key.
    pub async fn bind_identity(
        &self,
        endpoint: &str,
        expected: u64,
    ) -> Result<ConsentState, OfpError> {
        self.mutate(expected, endpoint, "identity-unlocked", |state| {
            if state
                .local_endpoint
                .as_ref()
                .is_some_and(|old| old != endpoint)
            {
                return Err(OfpError::Authority(
                    "identity mismatch; explicitly recover before using a replacement key".into(),
                ));
            }
            state.local_endpoint = Some(endpoint.into());
            Ok(())
        })
        .await
    }

    /// Recovery revokes every prior peer grant; no authority silently follows a new key.
    pub async fn recover_identity(
        &self,
        endpoint: &str,
        expected: u64,
    ) -> Result<ConsentState, OfpError> {
        self.mutate(
            expected,
            endpoint,
            "identity-recovered-reconsent-required",
            |state| {
                state.local_endpoint = Some(endpoint.into());
                for peer in state.peers.values_mut() {
                    peer.revoked = true;
                    peer.revision = state.revision;
                }
                Ok(())
            },
        )
        .await
    }

    pub async fn register_document(
        &self,
        document: PersonalDocument,
        expected: u64,
    ) -> Result<ConsentState, OfpError> {
        self.mutate(expected, "local", "personal-document-registered", |state| {
            let key = document.entity_id.to_string();
            if state
                .documents
                .get(&key)
                .is_some_and(|old| old.class != document.class || old.tombstone)
            {
                return Err(OfpError::Authority(
                    "document class is immutable; tombstones cannot resurrect".into(),
                ));
            }
            state.documents.insert(key, document);
            Ok(())
        })
        .await
    }

    pub async fn grant(
        &self,
        mut consent: PeerConsent,
        expected: u64,
    ) -> Result<ConsentState, OfpError> {
        let endpoint = consent.endpoint_id.clone();
        self.mutate(expected, &endpoint, "personal-peer-consented", |state| {
            let now = now_ms()?;
            if consent.credential_hash.len() != 64
                || !consent
                    .credential_hash
                    .bytes()
                    .all(|c| c.is_ascii_hexdigit())
                || consent.valid_until_ms <= now
                || consent.offline_valid_until_ms <= now
                || consent.offline_valid_until_ms > consent.valid_until_ms
                || consent.documents.is_empty()
                || consent
                    .documents
                    .iter()
                    .any(|id| !state.documents.contains_key(&id.to_string()))
            {
                return Err(OfpError::Authority(
                    "explicit credential, registered documents and bounded validity required"
                        .into(),
                ));
            }
            consent.revoked = false;
            consent.revision = state.revision;
            state.peers.insert(endpoint.clone(), consent);
            Ok(())
        })
        .await
    }

    pub async fn revoke(&self, endpoint: &str, expected: u64) -> Result<ConsentState, OfpError> {
        self.mutate(expected, endpoint, "personal-peer-revoked", |state| {
            let peer = state
                .peers
                .get_mut(endpoint)
                .ok_or_else(|| OfpError::Authority("unknown peer".into()))?;
            peer.revoked = true;
            peer.revision = state.revision;
            Ok(())
        })
        .await
    }

    pub async fn tombstone(&self, id: EntityId, expected: u64) -> Result<ConsentState, OfpError> {
        self.mutate(expected, "local", "personal-document-tombstoned", |state| {
            state
                .documents
                .get_mut(&id.to_string())
                .ok_or_else(|| OfpError::Authority("unknown document".into()))?
                .tombstone = true;
            Ok(())
        })
        .await
    }

    pub fn current_peer<'a>(
        state: &'a ConsentState,
        endpoint: &str,
        now: u64,
    ) -> Result<&'a PeerConsent, OfpError> {
        state
            .peers
            .get(endpoint)
            .filter(|peer| {
                !peer.revoked && peer.valid_until_ms > now && peer.offline_valid_until_ms > now
            })
            .ok_or_else(|| {
                OfpError::Authority(
                    "peer consent absent, revoked or offline validity exceeded".into(),
                )
            })
    }
}
