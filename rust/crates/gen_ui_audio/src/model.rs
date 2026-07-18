// TJ-ARCH-MOB-001 compliant
//! whisper-tiny model resolution.
//!
//! DECISION (flagged for follow-up, not silently picked): download-on-first-run
//! into a per-OS cache dir, NOT bundled into the app binary/IPA/APK.
//!   - whisper-tiny.en ggml (q5_1 quantised) is ~31MB / the multilingual fp16
//!     original is ~75MB. Bundling either into every mobile app download (iOS
//!     App Store / Play Store size budgets, plus doubling on every OTA update
//!     that touches the binary) was judged worse for a PoC than one deliberate
//!     first-run fetch with a visible progress state.
//!   - Tradeoff this creates: first Scribe use requires network + a wait (small
//!     model, but still not "instant offline"), and there is no server-side
//!     integrity story here yet (only a size sanity check, no checksum pin).
//!   - If a fully offline first-run experience becomes a real requirement
//!     (e.g. field use with no connectivity), flip this to bundle the model as
//!     a platform asset (Flutter `assets:`, Tauri `resources`) and swap
//!     `ensure_model` below for a copy-from-bundle path — the `ModelHandle`
//!     seam does not change, only how `path` gets populated.
use crate::error::{ScribeError, ScribeResult};
use std::path::PathBuf;

/// ggml whisper-tiny (multilingual, fp16) — smallest stock whisper.cpp model.
/// Hosted on the official whisper.cpp Hugging Face mirror.
const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin";
const MODEL_FILE_NAME: &str = "ggml-tiny.bin";
/// Sanity floor: the real file is ~75MB; anything drastically smaller means a
/// truncated/failed download, not a valid model.
const MODEL_MIN_BYTES: u64 = 10 * 1024 * 1024;

/// A resolved, on-disk whisper model ready to hand to `whisper_rs::WhisperContext`.
pub struct ModelHandle {
    pub path: PathBuf,
}

/// Directory Scribe caches the downloaded model in (per-OS app-data dir via
/// `dirs::data_local_dir`, mirroring where `gen_ui_db`/`gen_ui_db_graph` already
/// keep their embedded-store files).
fn model_cache_dir() -> ScribeResult<PathBuf> {
    let base = dirs::data_local_dir().ok_or_else(|| ScribeError::ModelMissing {
        path: MODEL_FILE_NAME.to_string(),
        reason: "could not resolve platform data directory".to_string(),
    })?;
    Ok(base.join("knowme-poc").join("models"))
}

/// Resolve the whisper-tiny model, downloading it into the cache dir on first
/// use. Idempotent: a valid cached file is reused; a missing/truncated one is
/// re-fetched. Network I/O + file I/O — call from `spawn_blocking` or an async
/// context with its own client, never on a UI thread.
pub async fn ensure_model() -> ScribeResult<ModelHandle> {
    let dir = model_cache_dir()?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(MODEL_FILE_NAME);

    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() >= MODEL_MIN_BYTES {
            return Ok(ModelHandle { path });
        }
    }

    download_model(&path).await?;
    Ok(ModelHandle { path })
}

async fn download_model(dest: &std::path::Path) -> ScribeResult<()> {
    tracing::info!(url = MODEL_URL, "downloading whisper-tiny model");
    let bytes = reqwest::get(MODEL_URL)
        .await
        .map_err(|e| ScribeError::ModelMissing {
            path: dest.display().to_string(),
            reason: format!("download request failed: {e}"),
        })?
        .bytes()
        .await
        .map_err(|e| ScribeError::ModelMissing {
            path: dest.display().to_string(),
            reason: format!("reading download body failed: {e}"),
        })?;

    if (bytes.len() as u64) < MODEL_MIN_BYTES {
        return Err(ScribeError::ModelMissing {
            path: dest.display().to_string(),
            reason: format!("downloaded file too small ({} bytes)", bytes.len()),
        });
    }

    // Write to a temp path then rename — avoids leaving a truncated file in
    // place if the process is killed mid-write.
    let tmp = dest.with_extension("bin.part");
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, dest)?;
    Ok(())
}
