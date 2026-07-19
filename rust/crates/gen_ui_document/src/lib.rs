// TJ-ARCH-MOB-001 compliant
//! Shared document/media normalization for chat and graph-RAG.
//!
//! Xberg is deliberately isolated in this leaf crate. The agent and FFI layers
//! receive bounded, normalized text and metadata instead of depending on xberg's
//! format/OCR implementation details. Every native platform uses the same path.

use std::path::Path;

use thiserror::Error;
use xberg::{extract, ExtractInput, ExtractionConfig};

/// Maximum input size accepted by the attachment pipeline.
pub const MAX_INPUT_BYTES: u64 = 25 * 1024 * 1024;
/// Maximum extracted text carried into a chat prompt or memory record.
pub const MAX_OUTPUT_CHARS: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedDocument {
    pub file_name: String,
    pub mime_type: String,
    pub text: String,
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("attachment is not a regular file or exceeds 25 MB")]
    InvalidFile,
    #[error("cannot read attachment: {0}")]
    Read(#[from] std::io::Error),
    #[error("xberg extraction failed: {0}")]
    Extraction(String),
    #[error("xberg returned no readable content")]
    Empty,
}

/// Extract and normalize a local file path using xberg.
pub async fn process_path(path: impl AsRef<Path>) -> Result<ProcessedDocument, DocumentError> {
    let path = path.as_ref();
    let metadata = tokio::fs::metadata(path).await?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
        return Err(DocumentError::InvalidFile);
    }
    let file_name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("attachment")
        .to_owned();
    let bytes = tokio::fs::read(path).await?;
    process_bytes(file_name, bytes).await
}

/// Extract and normalize in-memory attachment bytes using xberg.
pub async fn process_bytes(
    file_name: impl Into<String>,
    bytes: Vec<u8>,
) -> Result<ProcessedDocument, DocumentError> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(DocumentError::InvalidFile);
    }
    let file_name = file_name.into();
    let mime_type = mime_guess::from_path(&file_name)
        .first_or_octet_stream()
        .essence_str()
        .to_owned();
    let input = ExtractInput::from_bytes(bytes, mime_type.clone(), Some(file_name.clone()));
    let output = extract(input, &ExtractionConfig::default())
        .await
        .map_err(|error| DocumentError::Extraction(error.to_string()))?;
    let document = output
        .results
        .into_iter()
        .next()
        .ok_or(DocumentError::Empty)?;
    let text = document.content.trim();
    if text.is_empty() {
        return Err(DocumentError::Empty);
    }
    let text = text.chars().take(MAX_OUTPUT_CHARS).collect::<String>();
    Ok(ProcessedDocument {
        file_name,
        mime_type,
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn extracts_plain_text_with_filename_metadata() {
        let output = process_bytes("notes.txt", b"hello xberg".to_vec())
            .await
            .expect("text extraction succeeds");
        assert_eq!(output.file_name, "notes.txt");
        assert_eq!(output.mime_type, "text/plain");
        assert_eq!(output.text, "hello xberg");
    }

    #[tokio::test]
    async fn rejects_oversized_inputs_before_engine() {
        let output = process_bytes("large.bin", vec![0; MAX_INPUT_BYTES as usize + 1]).await;
        assert!(matches!(output, Err(DocumentError::InvalidFile)));
    }
}
