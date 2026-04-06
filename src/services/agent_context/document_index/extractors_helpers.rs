// ── Internal helpers ──────────────────────────────────────────────

/// Split text into chunks of MAX_CHUNK_SIZE at paragraph boundaries.
fn split_into_chunks(
    text: &str,
    relative_path: &str,
    doc_type: DocumentType,
    checksum: &str,
    quality: f32,
) -> Vec<DocumentChunk> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut chunk_idx: u32 = 0;

    for line in text.lines() {
        if current.len() + line.len() + 1 > MAX_CHUNK_SIZE && !current.is_empty() {
            chunks.push(DocumentChunk {
                file_path: relative_path.to_string(),
                doc_type: doc_type.clone(),
                chunk_index: chunk_idx,
                page_number: None,
                section_heading: None,
                text_content: current.clone(),
                file_checksum: checksum.to_string(),
                extraction_quality: quality,
            });
            chunk_idx += 1;
            current.clear();
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }

    if !current.trim().is_empty() {
        chunks.push(DocumentChunk {
            file_path: relative_path.to_string(),
            doc_type: doc_type.clone(),
            chunk_index: chunk_idx,
            page_number: None,
            section_heading: None,
            text_content: current,
            file_checksum: checksum.to_string(),
            extraction_quality: quality,
        });
    }

    chunks
}

/// Truncate text to MAX_CHUNK_SIZE at a word boundary, respecting UTF-8.
fn truncate_to_max_chunk(text: &str) -> String {
    if text.len() <= MAX_CHUNK_SIZE {
        return text.to_string();
    }
    // Find a valid UTF-8 char boundary at or before MAX_CHUNK_SIZE
    let mut end = MAX_CHUNK_SIZE;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    // Then find the last space before that for a clean word break
    match text[..end].rfind(' ') {
        Some(pos) => text[..pos].to_string(),
        None => text[..end].to_string(),
    }
}
