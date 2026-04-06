// ── SVG Extraction ──────────────────────────────────────────────

/// Extract text content from SVG `<text>` and `<tspan>` elements via regex.
///
/// No XML parser needed — SVG text elements are structurally simple.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn extract_svg(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read SVG {}: {e}", path.display()))?;

    let mut texts = Vec::new();

    // Match <text ...>content</text> and <tspan ...>content</tspan>
    // Simple regex approach — handles the common case without an XML parser
    if let Ok(re) = regex::Regex::new(r"<(?:text|tspan)[^>]*>([^<]+)</(?:text|tspan)>") {
        for cap in re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let t = m.as_str().trim();
                if !t.is_empty() {
                    texts.push(t.to_string());
                }
            }
        }
    }

    // Also extract title elements
    if let Ok(re) = regex::Regex::new(r"<title[^>]*>([^<]+)</title>") {
        for cap in re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let t = m.as_str().trim();
                if !t.is_empty() {
                    texts.push(format!("[title] {t}"));
                }
            }
        }
    }

    if texts.is_empty() {
        return Ok(vec![DocumentChunk {
            file_path: relative_path.to_string(),
            doc_type: DocumentType::Svg,
            chunk_index: 0,
            page_number: None,
            section_heading: None,
            text_content: format!("SVG: {} (no text content)", relative_path),
            file_checksum: checksum.to_string(),
            extraction_quality: 0.2,
        }]);
    }

    let combined = texts.join("\n");
    let quality = if combined.len() > 50 { 0.8 } else { 0.5 };

    Ok(vec![DocumentChunk {
        file_path: relative_path.to_string(),
        doc_type: DocumentType::Svg,
        chunk_index: 0,
        page_number: None,
        section_heading: None,
        text_content: combined,
        file_checksum: checksum.to_string(),
        extraction_quality: quality,
    }])
}
