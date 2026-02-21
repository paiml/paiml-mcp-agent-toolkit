#![cfg_attr(coverage_nightly, coverage(off))]

//! Document index types for non-code document search.

use std::fmt;

/// Supported document types for indexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DocumentType {
    Pdf,
    Svg,
    Image,
    Markdown,
    PlainText,
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pdf => write!(f, "pdf"),
            Self::Svg => write!(f, "svg"),
            Self::Image => write!(f, "image"),
            Self::Markdown => write!(f, "markdown"),
            Self::PlainText => write!(f, "plaintext"),
        }
    }
}

impl DocumentType {
    #[cfg(test)]
    pub(crate) fn from_str_label(s: &str) -> Option<Self> {
        match s {
            "pdf" => Some(Self::Pdf),
            "svg" => Some(Self::Svg),
            "image" => Some(Self::Image),
            "markdown" => Some(Self::Markdown),
            "plaintext" => Some(Self::PlainText),
            _ => None,
        }
    }
}

/// A chunk of extracted text from a document.
///
/// Documents are split into chunks at natural boundaries (pages, headings)
/// with a 4KB max per chunk for efficient FTS5 indexing.
#[derive(Debug, Clone)]
pub(crate) struct DocumentChunk {
    /// File path relative to project root
    pub file_path: String,
    /// Type of document
    pub doc_type: DocumentType,
    /// 0-based chunk index within the file
    pub chunk_index: u32,
    /// PDF page number (1-based), if applicable
    pub page_number: Option<u32>,
    /// Section heading (from markdown ## or PDF bookmarks)
    pub section_heading: Option<String>,
    /// Extracted text content (4KB max)
    pub text_content: String,
    /// SHA256 of the source file for incremental updates
    pub file_checksum: String,
    /// Extraction quality confidence (0.0-1.0)
    /// 1.0 = full text extraction, 0.3 = metadata only (images)
    pub extraction_quality: f32,
}

/// A search result from the document index.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct DocumentResult {
    pub file_path: String,
    pub doc_type: String,
    pub chunk_index: u32,
    pub page_number: Option<u32>,
    pub section_heading: Option<String>,
    /// Highlighted match excerpt
    pub snippet: String,
    pub relevance_score: f32,
    pub extraction_quality: f32,
}

/// Maximum chunk size in bytes (4KB)
pub(crate) const MAX_CHUNK_SIZE: usize = 4096;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_type_display() {
        assert_eq!(DocumentType::Pdf.to_string(), "pdf");
        assert_eq!(DocumentType::Svg.to_string(), "svg");
        assert_eq!(DocumentType::Image.to_string(), "image");
        assert_eq!(DocumentType::Markdown.to_string(), "markdown");
        assert_eq!(DocumentType::PlainText.to_string(), "plaintext");
    }

    #[test]
    fn test_document_type_from_str() {
        assert_eq!(DocumentType::from_str_label("pdf"), Some(DocumentType::Pdf));
        assert_eq!(DocumentType::from_str_label("svg"), Some(DocumentType::Svg));
        assert_eq!(
            DocumentType::from_str_label("image"),
            Some(DocumentType::Image)
        );
        assert_eq!(
            DocumentType::from_str_label("markdown"),
            Some(DocumentType::Markdown)
        );
        assert_eq!(
            DocumentType::from_str_label("plaintext"),
            Some(DocumentType::PlainText)
        );
        assert_eq!(DocumentType::from_str_label("unknown"), None);
    }

    #[test]
    fn test_document_chunk_construction() {
        let chunk = DocumentChunk {
            file_path: "docs/README.md".to_string(),
            doc_type: DocumentType::Markdown,
            chunk_index: 0,
            page_number: None,
            section_heading: Some("Introduction".to_string()),
            text_content: "Hello world".to_string(),
            file_checksum: "abc123".to_string(),
            extraction_quality: 1.0,
        };
        assert_eq!(chunk.file_path, "docs/README.md");
        assert_eq!(chunk.doc_type, DocumentType::Markdown);
        assert_eq!(chunk.extraction_quality, 1.0);
    }
}
