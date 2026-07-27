//! Durable, app-owned storage for user-selected attachments.
//!
//! The file a picker hands back is NOT safe to render from later. On Android
//! `file_picker` returns a path inside the app's *cache* directory — the
//! package ships `clearTemporaryFiles()` for exactly this reason — so a
//! transcript that points at the picked path renders correctly during testing
//! and shows a broken image weeks later, after the OS or the user clears the
//! cache. iOS security-scoped URLs and the web's blob URLs have the same
//! shape of problem: valid now, gone later.
//!
//! So every attachment is copied into app-owned storage at attach time and the
//! block carries *that* path. The copy is the only thing standing between a
//! working transcript and one that silently rots.
//!
//! Files are named by content hash, which makes the copy idempotent: attaching
//! the same image to five messages writes one file. It also means a rename in
//! the source directory does not produce a duplicate.

use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;
use sha2::{Digest, Sha256};

use crate::error::AgentError;

/// Process-wide root for stored attachments, set once during host bootstrap.
///
/// Deliberately a separate cell rather than a fifth parameter on
/// `state::init_with_inference_and_uar`: the storage root is orthogonal to the
/// agent's config/memory/inference composition, and tests can point it at a
/// tempdir without standing up a full agent state.
static ATTACHMENT_ROOT: OnceCell<PathBuf> = OnceCell::new();

/// Set the attachment storage root. Called once from host bootstrap.
///
/// A second call is ignored rather than silently replacing the root: two roots
/// in one process would strand every file already written under the first.
pub fn set_root(root: impl Into<PathBuf>) {
    let root = root.into();
    if let Err(existing) = ATTACHMENT_ROOT.set(root) {
        tracing::warn!(
            attempted = %existing.display(),
            "attachment root already set; keeping the original"
        );
    }
}

/// The configured root, or an error naming the missing bootstrap step.
///
/// Returns an error rather than falling back to a temp directory. A silent
/// fallback would put attachments somewhere that gets cleared — reintroducing
/// the exact bug this module exists to prevent, but harder to spot because
/// everything would appear to work.
fn root() -> Result<&'static Path, AgentError> {
    ATTACHMENT_ROOT.get().map(PathBuf::as_path).ok_or_else(|| {
        AgentError::Attachment(
            "attachment storage root is not configured; \
             call gen_ui_agent::attachment_store::set_root during host bootstrap"
                .into(),
        )
    })
}

/// Sanitise a path component so a hostile or odd conversation id cannot escape
/// the attachment root. Anything outside `[A-Za-z0-9._-]` becomes `_`, and the
/// traversal-capable forms are rejected outright.
fn safe_component(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() || cleaned.chars().all(|c| c == '.') {
        "unscoped".to_owned()
    } else {
        cleaned
    }
}

/// Where a conversation's attachments live.
pub fn conversation_dir(conversation_id: &str) -> Result<PathBuf, AgentError> {
    Ok(conversation_dir_in(root()?, conversation_id))
}

fn conversation_dir_in(root: &Path, conversation_id: &str) -> PathBuf {
    root.join(safe_component(conversation_id))
}

/// Copy `bytes` into app-owned storage and return the stable path.
///
/// Idempotent by content hash: the same bytes under the same conversation
/// resolve to the same file and are written once.
pub async fn store(
    conversation_id: &str,
    extension: &str,
    bytes: &[u8],
) -> Result<PathBuf, AgentError> {
    store_in(root()?, conversation_id, extension, bytes).await
}

async fn store_in(
    root: &Path,
    conversation_id: &str,
    extension: &str,
    bytes: &[u8],
) -> Result<PathBuf, AgentError> {
    let dir = conversation_dir_in(root, conversation_id);
    tokio::fs::create_dir_all(&dir).await.map_err(|error| {
        AgentError::Attachment(format!("cannot create attachment directory: {error}"))
    })?;

    let digest = Sha256::digest(bytes);
    let stem = hex_encode(&digest);
    let extension = safe_component(extension);
    let file_name = if extension.is_empty() {
        stem
    } else {
        format!("{stem}.{extension}")
    };
    let destination = dir.join(file_name);

    // Content-addressed: if it is already there with the right length, the
    // bytes are identical by construction and rewriting would be pure I/O.
    if let Ok(existing) = tokio::fs::metadata(&destination).await {
        if existing.len() == bytes.len() as u64 {
            return Ok(destination);
        }
    }

    tokio::fs::write(&destination, bytes)
        .await
        .map_err(|error| AgentError::Attachment(format!("cannot store attachment: {error}")))?;
    Ok(destination)
}

/// Remove every stored attachment for a conversation. Missing is success —
/// a conversation that never had attachments must not fail its own deletion.
pub async fn remove_conversation(conversation_id: &str) -> Result<(), AgentError> {
    let Ok(root) = root() else {
        // Nothing was ever stored if the root was never configured.
        return Ok(());
    };
    remove_conversation_in(root, conversation_id).await
}

async fn remove_conversation_in(root: &Path, conversation_id: &str) -> Result<(), AgentError> {
    let dir = conversation_dir_in(root, conversation_id);
    match tokio::fs::remove_dir_all(&dir).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AgentError::Attachment(format!(
            "cannot remove conversation attachments: {error}"
        ))),
    }
}

/// Delete stored attachment directories whose conversation is no longer known.
///
/// Runs at startup. A crash between writing a file and committing the message
/// row leaves bytes with no owner; without this they accumulate forever.
/// Returns the number of directories removed.
pub async fn sweep_orphans(known_conversation_ids: &[String]) -> Result<usize, AgentError> {
    let Ok(root) = root() else {
        return Ok(0);
    };
    sweep_orphans_in(root, known_conversation_ids).await
}

async fn sweep_orphans_in(
    root: &Path,
    known_conversation_ids: &[String],
) -> Result<usize, AgentError> {
    let known: std::collections::HashSet<String> = known_conversation_ids
        .iter()
        .map(|id| safe_component(id))
        .collect();

    let mut entries = match tokio::fs::read_dir(root).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => {
            return Err(AgentError::Attachment(format!(
                "cannot scan attachment root: {error}"
            )))
        }
    };

    let mut removed = 0usize;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if known.contains(&name) {
            continue;
        }
        if entry.path().is_dir() && tokio::fs::remove_dir_all(entry.path()).await.is_ok() {
            removed += 1;
            tracing::info!(conversation = %name, "swept orphaned attachment directory");
        }
    }
    Ok(removed)
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every test drives the `*_in` variants against its OWN tempdir. The
    // process-wide OnceCell is deliberately NOT used here: these tests share a
    // process, so a single global root let one test's sweep delete another
    // test's files — which is also why the public sweep must never be pointed
    // at a directory it does not exclusively own.

    #[tokio::test]
    async fn stores_bytes_and_returns_a_readable_path() {
        let temp = tempfile::tempdir().expect("tempdir");

        let path = store_in(temp.path(), "conv-1", "png", b"pretend-png-bytes")
            .await
            .expect("store");

        let read_back = tokio::fs::read(&path).await.expect("read back");
        assert_eq!(read_back, b"pretend-png-bytes");
    }

    #[tokio::test]
    async fn writing_the_same_bytes_twice_yields_one_file() {
        let temp = tempfile::tempdir().expect("tempdir");

        let first = store_in(temp.path(), "conv-dedupe", "png", b"identical")
            .await
            .expect("first");
        let second = store_in(temp.path(), "conv-dedupe", "png", b"identical")
            .await
            .expect("second");

        assert_eq!(first, second, "content-addressed writes must collapse");
        let dir = conversation_dir_in(temp.path(), "conv-dedupe");
        let mut entries = tokio::fs::read_dir(&dir).await.expect("read dir");
        let mut count = 0;
        while entries.next_entry().await.expect("entry").is_some() {
            count += 1;
        }
        assert_eq!(count, 1, "re-attaching the same file must not duplicate it");
    }

    #[tokio::test]
    async fn different_bytes_get_different_files() {
        let temp = tempfile::tempdir().expect("tempdir");

        let a = store_in(temp.path(), "conv-distinct", "png", b"first")
            .await
            .expect("a");
        let b = store_in(temp.path(), "conv-distinct", "png", b"second")
            .await
            .expect("b");

        assert_ne!(a, b);
    }

    #[test]
    fn traversal_attempts_cannot_escape_the_root() {
        // A conversation id is not user-authored today, but it crosses an FFI
        // boundary as a string — treating it as trusted is how directory
        // traversal gets in. `.` is permitted (real ids contain it), so the
        // guarantee is specifically that no SEPARATOR survives and a
        // dots-only component cannot name a parent.
        assert_eq!(safe_component("../../etc"), ".._.._etc");
        assert_eq!(safe_component(".."), "unscoped");
        assert_eq!(safe_component("."), "unscoped");
        assert_eq!(safe_component("/"), "_");
        assert_eq!(safe_component("ok-id_1.2"), "ok-id_1.2");

        for hostile in ["../../etc", "..", ".", "/", "a/../../b", "..\\..\\win"] {
            let component = safe_component(hostile);
            assert!(
                !component.contains('/') && !component.contains('\\'),
                "{hostile:?} produced a separator: {component:?}"
            );
            assert!(
                std::path::Path::new(&component).components().count() <= 1,
                "{hostile:?} produced a multi-component path: {component:?}"
            );
        }
    }

    #[tokio::test]
    async fn removing_a_conversation_deletes_its_bytes() {
        let temp = tempfile::tempdir().expect("tempdir");

        let path = store_in(temp.path(), "conv-doomed", "png", b"bytes")
            .await
            .expect("store");
        assert!(tokio::fs::metadata(&path).await.is_ok());

        remove_conversation_in(temp.path(), "conv-doomed")
            .await
            .expect("remove");

        assert!(
            tokio::fs::metadata(&path).await.is_err(),
            "deleting a conversation must not leak its attachment bytes"
        );
    }

    #[tokio::test]
    async fn removing_a_conversation_with_no_attachments_succeeds() {
        let temp = tempfile::tempdir().expect("tempdir");

        remove_conversation_in(temp.path(), "never-had-any")
            .await
            .expect("absence is not a failure");
    }

    #[tokio::test]
    async fn sweep_removes_unknown_conversations_and_keeps_known_ones() {
        let temp = tempfile::tempdir().expect("tempdir");

        store_in(temp.path(), "keep-me", "png", b"kept")
            .await
            .expect("keep");
        store_in(temp.path(), "orphan", "png", b"dropped")
            .await
            .expect("orphan");

        let removed = sweep_orphans_in(temp.path(), &["keep-me".to_owned()])
            .await
            .expect("sweep");

        assert_eq!(removed, 1);
        assert!(conversation_dir_in(temp.path(), "keep-me").exists());
        assert!(!conversation_dir_in(temp.path(), "orphan").exists());
    }

    #[tokio::test]
    async fn store_reports_a_clear_error_when_the_root_is_unconfigured() {
        // The failure mode this guards against is a SILENT fallback to a temp
        // directory, which would put attachments somewhere that gets cleared —
        // reintroducing the exact bug this module prevents, but invisibly.
        let error = store("conv", "png", b"bytes").await.unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("attachment storage root is not configured"),
            "unhelpful error: {message}"
        );
    }
}
