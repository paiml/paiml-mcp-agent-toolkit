#![cfg_attr(coverage_nightly, coverage(off))]

//! Document text extractors for PDF, SVG, images, markdown, and plaintext.
//!
//! Each extractor returns a `Vec<DocumentChunk>` split at natural boundaries
//! (pages, headings, paragraphs) with a 4KB max per chunk.

use super::types::{DocumentChunk, DocumentType, MAX_CHUNK_SIZE};
use std::path::Path;

// ── Submodule includes ──────────────────────────────────────────
include!("extractors_helpers.rs");
include!("extractors_pdf.rs");
include!("extractors_svg.rs");
include!("extractors_markdown.rs");
include!("extractors_media.rs");
include!("extractors_tests.rs");

// ── Dispatcher ──────────────────────────────────────────────────

/// Dispatch to the appropriate extractor based on file extension.
pub(crate) fn extract_document(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("pdf") => extract_pdf(path, relative_path, checksum),
        Some("svg") => extract_svg(path, relative_path, checksum),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp") => {
            extract_image_metadata(path, relative_path, checksum)
        }
        Some("md" | "markdown") => extract_markdown(path, relative_path, checksum),
        Some("txt" | "rst" | "adoc") => extract_plaintext(path, relative_path, checksum),
        _ => Err(format!("Unsupported document type: {}", path.display())),
    }
}

/// Check if a file extension is a supported document type.
pub(crate) fn is_document_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some(
            "pdf"
                | "svg"
                | "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "md"
                | "markdown"
                | "txt"
                | "rst"
                | "adoc"
        )
    )
}
