// ── Markdown Extraction ─────────────────────────────────────────

/// Extract structured text from Markdown, splitting at `##` headings.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn extract_markdown(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read markdown {}: {e}", path.display()))?;

    if content.trim().is_empty() {
        return Ok(vec![]);
    }

    let mut chunks = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_text = String::new();

    for line in content.lines() {
        if line.starts_with("## ") || line.starts_with("# ") {
            // Flush previous section
            if !current_text.trim().is_empty() {
                flush_markdown_section(
                    &mut chunks,
                    relative_path,
                    &current_heading,
                    &current_text,
                    checksum,
                );
            }
            current_heading = Some(line.trim_start_matches('#').trim().to_string());
            current_text.clear();
        } else {
            current_text.push_str(line);
            current_text.push('\n');
        }
    }

    // Flush final section
    if !current_text.trim().is_empty() {
        flush_markdown_section(
            &mut chunks,
            relative_path,
            &current_heading,
            &current_text,
            checksum,
        );
    }

    // If no sections found, create a single chunk
    if chunks.is_empty() && !content.trim().is_empty() {
        chunks.push(DocumentChunk {
            file_path: relative_path.to_string(),
            doc_type: DocumentType::Markdown,
            chunk_index: 0,
            page_number: None,
            section_heading: None,
            text_content: truncate_to_max_chunk(&content),
            file_checksum: checksum.to_string(),
            extraction_quality: 1.0,
        });
    }

    Ok(chunks)
}

fn flush_markdown_section(
    chunks: &mut Vec<DocumentChunk>,
    relative_path: &str,
    heading: &Option<String>,
    text: &str,
    checksum: &str,
) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    let chunk_index = chunks.len() as u32;
    let content = if trimmed.len() > MAX_CHUNK_SIZE {
        truncate_to_max_chunk(trimmed)
    } else {
        trimmed.to_string()
    };
    chunks.push(DocumentChunk {
        file_path: relative_path.to_string(),
        doc_type: DocumentType::Markdown,
        chunk_index,
        page_number: None,
        section_heading: heading.clone(),
        text_content: content,
        file_checksum: checksum.to_string(),
        extraction_quality: 1.0,
    });
}
