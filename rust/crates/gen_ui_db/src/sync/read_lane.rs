//! Sequential provider reads with transactionally persisted opaque cursor versions.
use super::{LocalStore, SyncStatusHandle};
use gen_ui_types::sync::{AuthorizedReadSource, ReadPoll, SyncStatus};
use gen_ui_types::CoreError;
use std::sync::Arc;

pub(super) async fn run(
    source: Arc<dyn AuthorizedReadSource>,
    store: Arc<dyn LocalStore>,
    status: SyncStatusHandle,
    mut stop: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        if *stop.borrow() { return; }
        let step = async {
            let cursor = store.read_cursor(source.shape_id()).await?;
            let poll = source.poll(cursor.as_ref()).await?;
            if !store.commit_read(source.shape_id(), source.table(), cursor.as_ref(), &poll).await? {
                return Ok(false);
            }
            match poll {
                ReadPoll::Revoked { reason } => {
                    status.set_read(source.shape_id(), SyncStatus::Error { message: reason });
                    Ok(true)
                }
                ReadPoll::Resnapshot => {
                    status.set_read(source.shape_id(), SyncStatus::Syncing { pending_writes: status.pending() });
                    Ok(false)
                }
                ReadPoll::Batch { caught_up, .. } => {
                    status.set_read(source.shape_id(), if caught_up && status.pending() == 0 { SyncStatus::Live }
                        else { SyncStatus::Syncing { pending_writes: status.pending() } });
                    Ok(false)
                }
            }
        };
        let result: Result<bool, CoreError> = tokio::select! {
            _ = stop.changed() => return,
            result = step => result,
        };
        match result {
            Ok(true) => return,
            Ok(false) => {}
            Err(CoreError::Transient(_)) | Err(CoreError::Io(_)) => {
                status.set_read(source.shape_id(), SyncStatus::Offline);
                tokio::select! {
                    _ = stop.changed() => return,
                    _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {}
                }
            }
            Err(error) => {
                status.set_read(source.shape_id(), SyncStatus::Error { message: error.to_string() });
                return;
            }
        }
    }
}
