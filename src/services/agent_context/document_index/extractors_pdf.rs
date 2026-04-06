// ── PDF Extraction ──────────────────────────────────────────────

/// Extract text from a PDF file.
///
/// Requires the `doc-indexing` feature for full text extraction.
/// Without it, returns metadata-only chunk (filename + size).
#[cfg(feature = "doc-indexing")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn extract_pdf(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read PDF {}: {e}", path.display()))?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| format!("Failed to extract PDF text from {}: {e}", path.display()))?;

    if text.trim().is_empty() {
        return Ok(vec![DocumentChunk {
            file_path: relative_path.to_string(),
            doc_type: DocumentType::Pdf,
            chunk_index: 0,
            page_number: None,
            section_heading: None,
            text_content: format!("PDF: {} (no extractable text)", relative_path),
            file_checksum: checksum.to_string(),
            extraction_quality: 0.1,
        }]);
    }

    Ok(split_into_chunks(
        &text,
        relative_path,
        DocumentType::Pdf,
        checksum,
        1.0,
    ))
}

#[cfg(not(feature = "doc-indexing"))]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn extract_pdf(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    Ok(vec![DocumentChunk {
        file_path: relative_path.to_string(),
        doc_type: DocumentType::Pdf,
        chunk_index: 0,
        page_number: None,
        section_heading: None,
        text_content: format!(
            "PDF: {} ({} bytes) — full text extraction requires --features doc-indexing",
            relative_path, size
        ),
        file_checksum: checksum.to_string(),
        extraction_quality: 0.1,
    }])
}
