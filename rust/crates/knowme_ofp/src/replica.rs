//! Restricted personal CRDT frames. Authority never enters the merge engine.
use crate::{
    OfpError,
    consent::{ConsentState, ConsentStore, PersonalClass, now_ms},
    service::PersonalPeerService,
};
use frf_domain::EntityId;
use frf_p2p::{
    PeerSession,
    protocol::{MAX_FRAME_BYTES, read_frame, write_frame},
};
use frf_ports::{ApplyDelta, CrdtSnapshot, OpStore, PendingOp};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentFrame {
    entity_id: EntityId,
    class: PersonalClass,
    tombstone: bool,
    snapshot: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    version: u8,
    documents: Vec<DocumentFrame>,
}

impl PersonalPeerService {
    /// Durable local edit: write-ahead op then CRDT checkpoint, never an authority write.
    pub async fn apply_local(&self, id: EntityId, delta: Vec<u8>) -> Result<(), OfpError> {
        if delta.len() > MAX_FRAME_BYTES {
            return Err(OfpError::Authority("personal delta too large".into()));
        }
        let _transaction = self.transaction.lock().await;
        let _authority = self.store.gate.lock().await;
        let state = self.store.load().await?;
        if !state
            .documents
            .get(&id.to_string())
            .is_some_and(|doc| !doc.tombstone)
        {
            return Err(OfpError::Authority(
                "personal document not registered or tombstoned".into(),
            ));
        }
        let existing = self.restore_pending(id, &state).await?;
        let version = existing
            .as_ref()
            .map_or(0, |s| s.version)
            .checked_add(1)
            .ok_or_else(|| OfpError::Authority("document revision exhausted".into()))?;
        let merged = frf_crdt::LoroDeltaApplier.apply(
            existing.as_ref().map_or(&[], |s| s.encoded.as_ref()),
            &delta,
        )?;
        self.ops
            .queue_op(PendingOp::new(id, state.tenant, delta.into(), version))
            .await?;
        self.snapshots
            .checkpoint(CrdtSnapshot {
                entity_id: id,
                tenant_id: state.tenant,
                encoded: merged.into(),
                version,
            })
            .await?;
        Ok(())
    }

    async fn restore_pending(
        &self,
        id: EntityId,
        state: &ConsentState,
    ) -> Result<Option<CrdtSnapshot>, OfpError> {
        let mut existing = self.snapshots.restore(id, state.tenant).await?;
        for op in self.ops.drain_pending(&id, &state.tenant).await? {
            let merged = frf_crdt::LoroDeltaApplier.apply(
                existing.as_ref().map_or(&[], |s| s.encoded.as_ref()),
                &op.payload,
            )?;
            let restored = CrdtSnapshot {
                entity_id: id,
                tenant_id: state.tenant,
                encoded: merged.into(),
                version: existing
                    .as_ref()
                    .map_or(op.local_seq, |s| s.version.max(op.local_seq)),
            };
            self.snapshots.checkpoint(restored.clone()).await?;
            existing = Some(restored);
        }
        Ok(existing)
    }

    /// Personal text payload has no path into grant, policy, organization or runtime state.
    pub async fn set_text(&self, id: EntityId, text: &str) -> Result<(), OfpError> {
        if text.len() > MAX_FRAME_BYTES / 2 {
            return Err(OfpError::Authority(
                "personal text exceeds frame bound".into(),
            ));
        }
        let _transaction = self.transaction.lock().await;
        let _authority = self.store.gate.lock().await;
        let state = self.store.load().await?;
        if !state
            .documents
            .get(&id.to_string())
            .is_some_and(|doc| !doc.tombstone)
        {
            return Err(OfpError::Authority(
                "personal document not registered or tombstoned".into(),
            ));
        }
        let existing = self.restore_pending(id, &state).await?;
        let doc = loro::LoroDoc::new();
        if let Some(snapshot) = &existing {
            doc.import(&snapshot.encoded)
                .map_err(|e| OfpError::Authority(e.to_string()))?;
        }
        doc.get_map("personal")
            .insert("text", text)
            .map_err(|e| OfpError::Authority(e.to_string()))?;
        let delta = doc
            .export(loro::ExportMode::Snapshot)
            .map_err(|e| OfpError::Authority(e.to_string()))?;
        let version = existing
            .map_or(0, |s| s.version)
            .checked_add(1)
            .ok_or_else(|| OfpError::Authority("document revision exhausted".into()))?;
        self.ops
            .queue_op(PendingOp::new(
                id,
                state.tenant,
                delta.clone().into(),
                version,
            ))
            .await?;
        self.snapshots
            .checkpoint(CrdtSnapshot {
                entity_id: id,
                tenant_id: state.tenant,
                encoded: delta.into(),
                version,
            })
            .await?;
        Ok(())
    }

    pub async fn read_text(&self, id: EntityId) -> Result<Option<String>, OfpError> {
        let _transaction = self.transaction.lock().await;
        let _authority = self.store.gate.lock().await;
        let state = self.store.load().await?;
        if !state
            .documents
            .get(&id.to_string())
            .is_some_and(|doc| !doc.tombstone)
        {
            return Ok(None);
        }
        let Some(snapshot) = self.restore_pending(id, &state).await? else {
            return Ok(None);
        };
        let doc = loro::LoroDoc::new();
        doc.import(&snapshot.encoded)
            .map_err(|e| OfpError::Authority(e.to_string()))?;
        let value = doc.get_map("personal").get_deep_value().to_json_value();
        Ok(value
            .get("text")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned))
    }

    async fn frame(&self, state: &ConsentState, endpoint: &str) -> Result<Frame, OfpError> {
        let peer = ConsentStore::current_peer(state, endpoint, now_ms()?)?;
        let mut documents = Vec::new();
        for id in &peer.documents {
            let doc = state
                .documents
                .get(&id.to_string())
                .ok_or_else(|| OfpError::Authority("unregistered consent document".into()))?;
            let snapshot = if doc.tombstone {
                Vec::new()
            } else {
                let existing = self.restore_pending(*id, state).await?;
                existing.map_or_else(Vec::new, |s| s.encoded.to_vec())
            };
            documents.push(DocumentFrame {
                entity_id: *id,
                class: doc.class,
                tombstone: doc.tombstone,
                snapshot,
            });
        }
        Ok(Frame {
            version: 1,
            documents,
        })
    }

    async fn merge(
        &self,
        state: &mut ConsentState,
        endpoint: &str,
        frame: Frame,
    ) -> Result<(), OfpError> {
        let peer = ConsentStore::current_peer(state, endpoint, now_ms()?)?;
        if frame.version != 1
            || frame.documents.iter().any(|incoming| {
                !peer.documents.contains(&incoming.entity_id)
                    || !state
                        .documents
                        .get(&incoming.entity_id.to_string())
                        .is_some_and(|local| local.class == incoming.class)
                    || (incoming.tombstone && !incoming.snapshot.is_empty())
            })
        {
            return Err(OfpError::Authority(
                "frame document, class or authority scope denied".into(),
            ));
        }
        for incoming in frame.documents {
            // Offline validity remains authoritative per document, not only at handshake.
            ConsentStore::current_peer(state, endpoint, now_ms()?)?;
            let local = state
                .documents
                .get_mut(&incoming.entity_id.to_string())
                .ok_or_else(|| OfpError::Authority("personal document missing".into()))?;
            if local.tombstone || incoming.tombstone {
                if !local.tombstone {
                    local.tombstone = true;
                    state.revision = state
                        .revision
                        .checked_add(1)
                        .ok_or_else(|| OfpError::Authority("consent revision exhausted".into()))?;
                    state.history.push(crate::consent::ConsentEvent {
                        revision: state.revision,
                        at_ms: now_ms()?,
                        endpoint_id: endpoint.into(),
                        action: "consented-document-tombstone".into(),
                    });
                    self.store.persist(state).await?;
                }
                self.snapshots
                    .purge(incoming.entity_id, state.tenant)
                    .await?;
            } else if !incoming.snapshot.is_empty() {
                let existing = self
                    .snapshots
                    .restore(incoming.entity_id, state.tenant)
                    .await?;
                let merged = frf_crdt::LoroDeltaApplier.apply(
                    existing.as_ref().map_or(&[], |s| s.encoded.as_ref()),
                    &incoming.snapshot,
                )?;
                let version = existing
                    .map_or(0, |s| s.version)
                    .checked_add(1)
                    .ok_or_else(|| OfpError::Authority("document revision exhausted".into()))?;
                self.snapshots
                    .checkpoint(CrdtSnapshot {
                        entity_id: incoming.entity_id,
                        tenant_id: state.tenant,
                        encoded: merged.into(),
                        version,
                    })
                    .await?;
            }
        }
        Ok(())
    }

    /// One bounded reconnect round; full Loro snapshots preserve convergence across retries.
    pub async fn sync(&self, session: &PeerSession, incoming: bool) -> Result<(), OfpError> {
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(30), async {
            let prior = self.store.state().await?;
            let revision =
                ConsentStore::current_peer(&prior, session.endpoint_id(), now_ms()?)?.revision;
            session.revalidate().await?;
            let _transaction = self.transaction.lock().await;
            // Serialize consent changes with the bounded disclosure transaction.
            let _authority = self.store.gate.lock().await;
            let mut state = self.store.load().await?;
            if ConsentStore::current_peer(&state, session.endpoint_id(), now_ms()?)?.revision
                != revision
            {
                return Err(OfpError::Authority(
                    "peer consent changed during frame admission".into(),
                ));
            }
            if session.identity().subject != state.owner
                || session.identity().tenant_id != state.tenant
            {
                return Err(OfpError::Authority("peer owner/tenant mismatch".into()));
            }
            ConsentStore::current_peer(&state, session.endpoint_id(), now_ms()?)?;
            let (mut send, mut recv) = if incoming {
                session.connection().accept_bi().await
            } else {
                session.connection().open_bi().await
            }
            .map_err(|e| OfpError::Authority(e.to_string()))?;
            if !incoming {
                let frame = self.frame(&state, session.endpoint_id()).await?;
                ConsentStore::current_peer(&state, session.endpoint_id(), now_ms()?)?;
                write_frame(&mut send, &serde_json::to_vec(&frame)?, MAX_FRAME_BYTES).await?;
            }
            let frame: Frame =
                serde_json::from_slice(&read_frame(&mut recv, MAX_FRAME_BYTES).await?)?;
            self.merge(&mut state, session.endpoint_id(), frame).await?;
            if incoming {
                let frame = self.frame(&state, session.endpoint_id()).await?;
                ConsentStore::current_peer(&state, session.endpoint_id(), now_ms()?)?;
                write_frame(&mut send, &serde_json::to_vec(&frame)?, MAX_FRAME_BYTES).await?;
            }
            send.finish()
                .map_err(|e| OfpError::Authority(e.to_string()))?;
            Ok::<(), OfpError>(())
        })
        .await
        .map_err(|_| OfpError::Authority("personal peer round deadline exceeded".into()))
        .and_then(std::convert::identity);
        if outcome.is_err() {
            session
                .connection()
                .close(3u8.into(), b"personal scope denied");
        }
        outcome
    }
}
