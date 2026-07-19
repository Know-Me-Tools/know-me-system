// TJ-ARCH-MOB-001 compliant
//! Shared document/media normalization for chat and graph-RAG.
//!
//! Xberg is deliberately isolated in this leaf crate. The agent and FFI layers
//! receive bounded, normalized text and metadata instead of depending on xberg's
//! format/OCR implementation details. Every native platform uses the same path.

use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChunkStrategy {
    Atomic,
    SemanticBoundary,
    RecursiveOverlap { overlap_chars: usize },
    ParentChild { child_target_chars: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProvenance {
    pub conversation_id: String,
    pub branch_id: String,
    pub message_id: String,
    pub run_id: String,
    pub source_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChunk {
    pub id: String,
    pub parent_id: Option<String>,
    pub text: String,
    pub start_char: usize,
    pub end_char: usize,
    pub provenance: DocumentProvenance,
}

pub fn chunk_document(
    document: &ProcessedDocument,
    provenance: &DocumentProvenance,
    strategy: ChunkStrategy,
    target_chars: usize,
) -> Vec<DocumentChunk> {
    let target = target_chars.max(64);
    match strategy {
        ChunkStrategy::Atomic => vec![make_chunk(
            document.text.clone(),
            0,
            document.text.chars().count(),
            None,
            provenance,
        )],
        ChunkStrategy::SemanticBoundary => semantic_chunks(&document.text, target, provenance),
        ChunkStrategy::RecursiveOverlap { overlap_chars } => overlapping_chunks(
            &document.text,
            target,
            overlap_chars.min(target / 2),
            provenance,
            None,
        ),
        ChunkStrategy::ParentChild { child_target_chars } => {
            let parents = semantic_chunks(&document.text, target, provenance);
            let mut output = parents.clone();
            for parent in parents {
                output.extend(overlapping_chunks(
                    &parent.text,
                    child_target_chars.max(64),
                    child_target_chars / 8,
                    provenance,
                    Some(parent.id),
                ));
            }
            output
        }
    }
}

fn semantic_chunks(
    text: &str,
    target_chars: usize,
    provenance: &DocumentProvenance,
) -> Vec<DocumentChunk> {
    let mut output = Vec::new();
    let mut current = String::new();
    let mut start = 0;
    let mut cursor = 0;
    for paragraph in text.split_inclusive("\n\n") {
        let length = paragraph.chars().count();
        if !current.is_empty() && current.chars().count() + length > target_chars {
            let end = start + current.chars().count();
            output.push(make_chunk(current, start, end, None, provenance));
            current = String::new();
            start = cursor;
        }
        current.push_str(paragraph);
        cursor += length;
    }
    if !current.trim().is_empty() {
        output.push(make_chunk(
            current.clone(),
            start,
            start + current.chars().count(),
            None,
            provenance,
        ));
    }
    output
}

fn overlapping_chunks(
    text: &str,
    target_chars: usize,
    overlap_chars: usize,
    provenance: &DocumentProvenance,
    parent_id: Option<String>,
) -> Vec<DocumentChunk> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + target_chars).min(chars.len());
        output.push(make_chunk(
            chars[start..end].iter().collect(),
            start,
            end,
            parent_id.clone(),
            provenance,
        ));
        if end == chars.len() {
            break;
        }
        start = end.saturating_sub(overlap_chars);
    }
    output
}

fn make_chunk(
    text: String,
    start_char: usize,
    end_char: usize,
    parent_id: Option<String>,
    provenance: &DocumentProvenance,
) -> DocumentChunk {
    let mut digest = Sha256::new();
    for value in [
        provenance.conversation_id.as_bytes(),
        provenance.branch_id.as_bytes(),
        provenance.message_id.as_bytes(),
        provenance.source_name.as_bytes(),
    ] {
        digest.update(value);
        digest.update([0]);
    }
    digest.update(start_char.to_le_bytes());
    digest.update(end_char.to_le_bytes());
    digest.update(text.as_bytes());
    DocumentChunk {
        id: format!("chunk:{:x}", digest.finalize()),
        parent_id,
        text,
        start_char,
        end_char,
        provenance: provenance.clone(),
    }
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

    #[test]
    fn chunks_are_unicode_safe_idempotent_and_provenanced() {
        let document = ProcessedDocument {
            file_name: "notes.md".into(),
            mime_type: "text/markdown".into(),
            text: "One paragraph.\n\nSecond 🧠 paragraph with details.\n\nThird paragraph.".into(),
        };
        let provenance = DocumentProvenance {
            conversation_id: "c1".into(),
            branch_id: "b1".into(),
            message_id: "m1".into(),
            run_id: "r1".into(),
            source_name: document.file_name.clone(),
        };
        let first = chunk_document(
            &document,
            &provenance,
            ChunkStrategy::RecursiveOverlap { overlap_chars: 16 },
            64,
        );
        let second = chunk_document(
            &document,
            &provenance,
            ChunkStrategy::RecursiveOverlap { overlap_chars: 16 },
            64,
        );
        assert_eq!(first, second);
        assert!(first.iter().all(|chunk| chunk.id.starts_with("chunk:")));
        assert!(first.iter().all(|chunk| chunk.provenance.run_id == "r1"));
    }
}
