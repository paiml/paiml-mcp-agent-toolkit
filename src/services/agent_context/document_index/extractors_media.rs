// ── Image & Plaintext Extraction ────────────────────────────────

/// Extract image metadata (filename, directory context, file size).
///
/// No OCR — just structural metadata for discoverability.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn extract_image_metadata(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let parent = Path::new(relative_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    let text = if parent.is_empty() {
        format!("Image: {filename} ({size} bytes)")
    } else {
        format!("Image: {filename} (in {parent}/, {size} bytes)")
    };

    Ok(vec![DocumentChunk {
        file_path: relative_path.to_string(),
        doc_type: DocumentType::Image,
        chunk_index: 0,
        page_number: None,
        section_heading: None,
        text_content: text,
        file_checksum: checksum.to_string(),
        extraction_quality: 0.3,
    }])
}

/// Extract plaintext content (.txt, .rst, .adoc).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn extract_plaintext(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    if content.trim().is_empty() {
        return Ok(vec![]);
    }

    Ok(split_into_chunks(
        &content,
        relative_path,
        DocumentType::PlainText,
        checksum,
        1.0,
    ))
}
