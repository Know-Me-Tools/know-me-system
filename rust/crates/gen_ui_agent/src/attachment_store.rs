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
    ATTACHMENT_ROOT
        .get()
        .map(PathBuf::as_path)
        .ok_or_else(unconfigured_root_error)
}

fn unconfigured_root_error() -> AgentError {
    AgentError::Attachment(
        "attachment storage root is not configured; \
         call gen_ui_agent::attachment_store::set_root during host bootstrap"
            .into(),
    )
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
    // An extensionless file gets no extension. Routing through safe_component
    // would turn "" into its "unscoped" placeholder and produce `<hash>.unscoped`.
    let file_name = if extension.is_empty() {
        stem
    } else {
        format!("{stem}.{}", safe_component(extension))
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

/// Store raw bytes a UI already holds and describe the result.
///
/// The web/desktop composer hands React a `File` whose bytes are in memory with
/// no filesystem path; this is the entry point for that case.
pub async fn stage_bytes(
    conversation_id: &str,
    file_name: &str,
    bytes: &[u8],
) -> Result<StoredAttachment, AgentError> {
    stage_bytes_in(root()?, conversation_id, file_name, bytes).await
}

async fn stage_bytes_in(
    root: &Path,
    conversation_id: &str,
    file_name: &str,
    bytes: &[u8],
) -> Result<StoredAttachment, AgentError> {
    if bytes.len() as u64 > gen_ui_document::MAX_INPUT_BYTES {
        return Err(AgentError::Attachment(format!(
            "{file_name} exceeds the 25 MB attachment limit"
        )));
    }
    // Only the final component: a crafted name like "../../etc/passwd" must not
    // escape, and the extension drives the mime.
    let base = std::path::Path::new(file_name)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .filter(|name| !name.is_empty())
        .unwrap_or("attachment");
    let extension = std::path::Path::new(base)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    let path = store_in(root, conversation_id, &extension, bytes).await?;
    let (mime, is_image, is_document, is_video) = crate::chat::attachment_kind(&extension);
    let (width, height) = if is_image {
        image_dimensions(bytes).map_or((None, None), |(w, h)| (Some(w), Some(h)))
    } else {
        (None, None)
    };

    Ok(StoredAttachment {
        path: path.to_string_lossy().into_owned(),
        file_name: base.to_owned(),
        mime: mime.to_owned(),
        size_bytes: bytes.len() as u64,
        is_image,
        is_video,
        is_document,
        width,
        height,
    })
}

/// Store a file the user picked from the filesystem and describe the result.
pub async fn stage_path(
    conversation_id: &str,
    source_path: &str,
) -> Result<StoredAttachment, AgentError> {
    let metadata = tokio::fs::metadata(source_path)
        .await
        .map_err(|error| AgentError::Attachment(format!("cannot open selected file: {error}")))?;
    // Rejected before the read: an over-cap file must never be pulled into
    // memory first and refused after.
    if !metadata.is_file() || metadata.len() > gen_ui_document::MAX_INPUT_BYTES {
        return Err(AgentError::Attachment(
            "selected file is not a regular file or exceeds 25 MB".into(),
        ));
    }
    let file_name = std::path::Path::new(source_path)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("attachment")
        .to_owned();
    let bytes = tokio::fs::read(source_path)
        .await
        .map_err(|error| AgentError::Attachment(format!("cannot read selected file: {error}")))?;
    stage_bytes(conversation_id, &file_name, &bytes).await
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

/// What a UI needs to render an attachment the moment it is picked, without
/// guessing anything from the file name.
///
/// Every field is decided ONCE here, in Rust. The alternative — each surface
/// deciding for itself — is what produced five separate hardcodes of the kind
/// `attachment`, which is why a PNG rendered as a grey document chip on every
/// platform.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StoredAttachment {
    /// Durable path in app-owned storage. Safe to persist in a transcript.
    pub path: String,
    pub file_name: String,
    pub mime: String,
    pub size_bytes: u64,
    pub is_image: bool,
    pub is_video: bool,
    pub is_document: bool,
    /// Intrinsic pixel width, when the bytes are an image whose header we read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

impl StoredAttachment {
    /// The block a UI should render for this attachment.
    ///
    /// Images become a real `ContentBlock::Image` carrying the durable path;
    /// everything else stays an artifact. This is the single place that
    /// decision is made.
    pub fn to_content_block(&self) -> gen_ui_types::ContentBlock {
        if self.is_image {
            gen_ui_types::ContentBlock::Image {
                url: None,
                data_base64: None,
                mime: self.mime.clone(),
                path: Some(self.path.clone()),
                // The file name is the only honest alt text available at pick
                // time. It is a weak description, but a missing accessible
                // name fails the a11y gate outright.
                alt: Some(self.file_name.clone()),
                width: self.width,
                height: self.height,
            }
        } else {
            gen_ui_types::ContentBlock::Artifact {
                id: self.path.clone(),
                kind: if self.is_video { "video" } else { "attachment" }.to_owned(),
                content: serde_json::json!({
                    "name": self.file_name,
                    "mime": self.mime,
                    "size": self.size_bytes,
                    "path": self.path,
                })
                .to_string(),
            }
        }
    }
}

/// Read intrinsic dimensions from an image header.
///
/// Header-only on purpose: decoding a 25 MB photo to learn it is 4032x3024
/// would cost hundreds of milliseconds and a large allocation on the attach
/// path. Returns `None` for formats without a readable header (SVG has no
/// intrinsic pixel size at all) rather than guessing — a wrong dimension is
/// worse than none, because the renderer reserves the wrong space.
pub fn image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let size = imagesize::blob_size(bytes).ok()?;
    Some((
        u32::try_from(size.width).ok()?,
        u32::try_from(size.height).ok()?,
    ))
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

    /// Smallest valid PNG: 1x1, so the header parse is real rather than mocked.
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn a_png_becomes_an_image_block_carrying_its_durable_path() {
        // THE regression guard for the reported defect: a PNG rendered as a
        // grey document chip because nothing ever built an Image block.
        let stored = StoredAttachment {
            path: "/data/attachments/conv/abc.png".into(),
            file_name: "holiday.png".into(),
            mime: "image/png".into(),
            size_bytes: 1234,
            is_image: true,
            is_video: false,
            is_document: false,
            width: Some(800),
            height: Some(600),
        };

        let block = stored.to_content_block();

        let gen_ui_types::ContentBlock::Image {
            path,
            alt,
            mime,
            width,
            height,
            url,
            data_base64,
        } = block
        else {
            panic!("a PNG must produce an Image block, not {block:?}");
        };
        assert_eq!(path.as_deref(), Some("/data/attachments/conv/abc.png"));
        assert_eq!(mime, "image/png");
        assert_eq!((width, height), (Some(800), Some(600)));
        // Alt text is mandatory for the a11y gate; the file name is the only
        // description available at pick time.
        assert_eq!(alt.as_deref(), Some("holiday.png"));
        // The bytes live on disk, so neither inline carrier is populated.
        assert_eq!(url, None);
        assert_eq!(data_base64, None);
    }

    #[test]
    fn non_images_stay_artifacts_and_video_is_labelled_as_such() {
        let pdf = StoredAttachment {
            path: "/data/a/report.pdf".into(),
            file_name: "report.pdf".into(),
            mime: "application/pdf".into(),
            size_bytes: 10,
            is_image: false,
            is_video: false,
            is_document: true,
            width: None,
            height: None,
        };
        let video = StoredAttachment {
            is_document: false,
            is_video: true,
            mime: "video/mp4".into(),
            file_name: "clip.mp4".into(),
            path: "/data/a/clip.mp4".into(),
            ..pdf.clone()
        };

        match pdf.to_content_block() {
            gen_ui_types::ContentBlock::Artifact { kind, .. } => assert_eq!(kind, "attachment"),
            other => panic!("a PDF must stay an artifact, got {other:?}"),
        }
        match video.to_content_block() {
            gen_ui_types::ContentBlock::Artifact { kind, content, .. } => {
                assert_eq!(kind, "video");
                // The renderer needs the path from the payload, not the id.
                assert!(content.contains("/data/a/clip.mp4"), "payload: {content}");
            }
            other => panic!("video must be labelled video, got {other:?}"),
        }
    }

    // A `shared_root()` helper lived here to work around the process-wide
    // OnceCell — every staging test had to assert against whichever root won
    // the race. `stage_bytes_in` takes the root explicitly, so each test now
    // owns its own tempdir and the ordering hazard is gone entirely.

    #[tokio::test]
    async fn sweeping_against_an_empty_known_list_would_remove_everything() {
        // Documents WHY the FFI wrapper refuses an empty list rather than
        // passing it through: at this layer "nothing is known" genuinely means
        // "everything is an orphan", which is correct here and catastrophic if
        // the caller merely failed to load its conversations.
        let temp = tempfile::tempdir().expect("tempdir");
        store_in(temp.path(), "conv", "png", b"bytes")
            .await
            .expect("store");

        let removed = sweep_orphans_in(temp.path(), &[]).await.expect("sweep");

        assert_eq!(removed, 1, "an empty known-list orphans every conversation");
    }

    #[tokio::test]
    async fn staging_a_png_reads_its_real_dimensions() {
        let temp = tempfile::tempdir().expect("tempdir");

        let stored = stage_bytes_in(temp.path(), "dims-conv", "tiny.png", TINY_PNG)
            .await
            .expect("stage");

        assert!(stored.is_image);
        assert_eq!(stored.mime, "image/png");
        // Read from the header, never guessed: a wrong dimension makes the
        // renderer reserve the wrong space, which is worse than none.
        assert_eq!((stored.width, stored.height), (Some(1), Some(1)));
        assert_eq!(stored.size_bytes, TINY_PNG.len() as u64);
    }

    #[test]
    fn dimensions_are_none_rather_than_guessed_for_unreadable_bytes() {
        assert_eq!(image_dimensions(b"not an image at all"), None);
        // SVG has no intrinsic pixel size; claiming one would be a lie.
        assert_eq!(
            image_dimensions(b"<svg xmlns='http://www.w3.org/2000/svg'/>"),
            None
        );
    }

    #[tokio::test]
    async fn staging_rejects_a_traversal_filename_without_escaping() {
        let temp = tempfile::tempdir().expect("tempdir");
        let active_root = temp.path();

        let stored = stage_bytes_in(active_root, "trav-conv", "../../../etc/passwd", b"x")
            .await
            .expect("stage");

        assert_eq!(stored.file_name, "passwd");
        assert!(
            std::path::Path::new(&stored.path).starts_with(active_root),
            "stored outside the root {}: {}",
            active_root.display(),
            stored.path
        );
        // An extensionless name must not acquire a placeholder extension.
        assert!(
            !stored.path.ends_with(".unscoped"),
            "extensionless file gained a bogus extension: {}",
            stored.path
        );
    }

    #[test]
    fn an_unconfigured_root_reports_a_clear_error_rather_than_falling_back() {
        // The failure mode this guards against is a SILENT fallback to a temp
        // directory, which would put attachments somewhere that gets cleared —
        // reintroducing the exact bug this module prevents, but invisibly.
        //
        // Asserted against the error constructor rather than by calling
        // `store()` with the cell unset: the cell is process-wide and other
        // tests in this binary set it, so a test that depends on it being
        // EMPTY passes or fails purely on test ordering.
        let unset: OnceCell<PathBuf> = OnceCell::new();
        let error = unset
            .get()
            .map(PathBuf::as_path)
            .ok_or_else(unconfigured_root_error)
            .unwrap_err();

        let message = error.to_string();
        assert!(
            message.contains("attachment storage root is not configured"),
            "unhelpful error: {message}"
        );
        assert!(
            message.contains("set_root"),
            "the error must name the missing call: {message}"
        );
    }
}

#[cfg(test)]
mod fixture_tests {
    /// Byte-for-byte the PNG that scripts/android/certify-local-image-rendering.sh
    /// pushes to the device: 7x3, chosen so a default or a guess cannot pass.
    ///
    /// Built here rather than read from disk so the test is self-contained —
    /// a fixture file under /tmp would make this pass only on the machine that
    /// happened to run the device script.
    fn device_certification_png() -> Vec<u8> {
        fn chunk(tag: &[u8], payload: &[u8]) -> Vec<u8> {
            let mut out = (payload.len() as u32).to_be_bytes().to_vec();
            out.extend_from_slice(tag);
            out.extend_from_slice(payload);
            let mut crc_input = tag.to_vec();
            crc_input.extend_from_slice(payload);
            out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
            out
        }

        const WIDTH: u32 = 7;
        const HEIGHT: u32 = 3;

        let mut ihdr = WIDTH.to_be_bytes().to_vec();
        ihdr.extend_from_slice(&HEIGHT.to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit truecolour

        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend_from_slice(&chunk(b"IHDR", &ihdr));
        // imagesize reads the header only, so IDAT contents do not matter —
        // but the chunk must be present and well-formed.
        png.extend_from_slice(&chunk(
            b"IDAT",
            &[0x78, 0x9c, 0x63, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01],
        ));
        png.extend_from_slice(&chunk(b"IEND", &[]));
        png
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for byte in data {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    /// Pins the device gate's premise: if Rust cannot read 7x3 from this exact
    /// image, the on-device dimensions assertion means nothing.
    #[test]
    fn reads_the_device_certification_fixture_dimensions() {
        assert_eq!(
            super::image_dimensions(&device_certification_png()),
            Some((7, 3))
        );
    }

    /// The whole reported defect, end to end: a picked PNG becomes a durable
    /// file AND an image block that renders — not a grey document chip.
    ///
    /// Covers the seam the device gate exercises manually, so a regression is
    /// caught by `cargo test` rather than only by someone holding a phone.
    #[tokio::test]
    async fn a_picked_png_becomes_a_durable_image_block() {
        let temp = tempfile::tempdir().expect("tempdir");
        let png = device_certification_png();

        // Stand in for the picker's cache directory — the path the OS may
        // delete at any time.
        let picked = temp.path().join("picker-cache").join("holiday.png");
        std::fs::create_dir_all(picked.parent().expect("parent")).expect("cache dir");
        std::fs::write(&picked, &png).expect("write picked");

        let root = temp.path().join("app-storage");
        let stored = super::stage_bytes_in(&root, "conv-e2e", "holiday.png", &png)
            .await
            .expect("stage");

        // Classified as an image, with dimensions read from the header.
        assert!(stored.is_image, "a PNG must be classified as an image");
        assert_eq!(stored.mime, "image/png");
        assert_eq!((stored.width, stored.height), (Some(7), Some(3)));

        // The stored path is NOT the picked path. This is the entire point:
        // keeping the picker's path is what made transcripts rot.
        assert_ne!(
            std::path::Path::new(&stored.path),
            picked.as_path(),
            "the block must not carry the purgeable picker path"
        );

        // Deleting the picker's cache — what Android does on its own schedule
        // — must leave the stored bytes intact.
        std::fs::remove_dir_all(picked.parent().expect("parent")).expect("purge cache");
        let survived = std::fs::read(&stored.path).expect("stored file survives a cache purge");
        assert_eq!(survived, png);

        // And the block is a real Image carrying that durable path.
        match stored.to_content_block() {
            gen_ui_types::ContentBlock::Image {
                path,
                alt,
                width,
                height,
                ..
            } => {
                assert_eq!(path.as_deref(), Some(stored.path.as_str()));
                assert_eq!(alt.as_deref(), Some("holiday.png"));
                assert_eq!((width, height), (Some(7), Some(3)));
            }
            other => panic!("a PNG must render as an image block, got {other:?}"),
        }
    }
}
