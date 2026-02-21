#![cfg_attr(coverage_nightly, coverage(off))]

//! Document text extractors for PDF, SVG, images, markdown, and plaintext.
//!
//! Each extractor returns a `Vec<DocumentChunk>` split at natural boundaries
//! (pages, headings, paragraphs) with a 4KB max per chunk.

use super::types::{DocumentChunk, DocumentType, MAX_CHUNK_SIZE};
use std::path::Path;

/// Extract text from a PDF file.
///
/// Requires the `doc-indexing` feature for full text extraction.
/// Without it, returns metadata-only chunk (filename + size).
#[cfg(feature = "doc-indexing")]
pub(crate) fn extract_pdf(path: &Path, relative_path: &str, checksum: &str) -> Result<Vec<DocumentChunk>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read PDF {}: {e}", path.display()))?;
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

    Ok(split_into_chunks(&text, relative_path, DocumentType::Pdf, checksum, 1.0))
}

#[cfg(not(feature = "doc-indexing"))]
pub(crate) fn extract_pdf(path: &Path, relative_path: &str, checksum: &str) -> Result<Vec<DocumentChunk>, String> {
    let size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    Ok(vec![DocumentChunk {
        file_path: relative_path.to_string(),
        doc_type: DocumentType::Pdf,
        chunk_index: 0,
        page_number: None,
        section_heading: None,
        text_content: format!(
            "PDF: {} ({} bytes) — full text extraction requires --features doc-indexing",
            relative_path,
            size
        ),
        file_checksum: checksum.to_string(),
        extraction_quality: 0.1,
    }])
}

/// Extract text content from SVG `<text>` and `<tspan>` elements via regex.
///
/// No XML parser needed — SVG text elements are structurally simple.
pub(crate) fn extract_svg(path: &Path, relative_path: &str, checksum: &str) -> Result<Vec<DocumentChunk>, String> {
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

/// Extract image metadata (filename, directory context, file size).
///
/// No OCR — just structural metadata for discoverability.
pub(crate) fn extract_image_metadata(
    path: &Path,
    relative_path: &str,
    checksum: &str,
) -> Result<Vec<DocumentChunk>, String> {
    let size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);

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

/// Extract structured text from Markdown, splitting at `##` headings.
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

/// Extract plaintext content (.txt, .rst, .adoc).
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
        Some("pdf" | "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "md" | "markdown" | "txt" | "rst" | "adoc")
    )
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_document_file() {
        assert!(is_document_file(Path::new("docs/spec.pdf")));
        assert!(is_document_file(Path::new("diagram.svg")));
        assert!(is_document_file(Path::new("screenshot.png")));
        assert!(is_document_file(Path::new("photo.jpg")));
        assert!(is_document_file(Path::new("photo.JPEG")));
        assert!(is_document_file(Path::new("README.md")));
        assert!(is_document_file(Path::new("notes.txt")));
        assert!(is_document_file(Path::new("doc.rst")));
        assert!(is_document_file(Path::new("doc.adoc")));
        assert!(!is_document_file(Path::new("main.rs")));
        assert!(!is_document_file(Path::new("lib.py")));
        assert!(!is_document_file(Path::new("Cargo.toml")));
    }

    #[test]
    fn test_extract_svg_with_text() {
        let dir = tempfile::tempdir().unwrap();
        let svg_path = dir.path().join("diagram.svg");
        std::fs::write(
            &svg_path,
            r#"<svg><text x="10" y="20">Hello World</text><tspan>Sub text</tspan></svg>"#,
        )
        .unwrap();

        let chunks = extract_svg(&svg_path, "diagram.svg", "abc123").unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text_content.contains("Hello World"));
        assert!(chunks[0].text_content.contains("Sub text"));
        assert_eq!(chunks[0].doc_type, DocumentType::Svg);
    }

    #[test]
    fn test_extract_svg_no_text() {
        let dir = tempfile::tempdir().unwrap();
        let svg_path = dir.path().join("empty.svg");
        std::fs::write(&svg_path, r#"<svg><rect width="100" height="100"/></svg>"#).unwrap();

        let chunks = extract_svg(&svg_path, "empty.svg", "def456").unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text_content.contains("no text content"));
        assert_eq!(chunks[0].extraction_quality, 0.2);
    }

    #[test]
    fn test_extract_svg_with_title() {
        let dir = tempfile::tempdir().unwrap();
        let svg_path = dir.path().join("titled.svg");
        std::fs::write(
            &svg_path,
            r#"<svg><title>Architecture Diagram</title><text>Node A</text></svg>"#,
        )
        .unwrap();

        let chunks = extract_svg(&svg_path, "titled.svg", "ghi789").unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text_content.contains("Architecture Diagram"));
        assert!(chunks[0].text_content.contains("Node A"));
    }

    #[test]
    fn test_extract_image_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("screenshot.png");
        std::fs::write(&img_path, b"fake png data").unwrap();

        let chunks =
            extract_image_metadata(&img_path, "docs/screenshots/screenshot.png", "hash1").unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text_content.contains("screenshot.png"));
        assert!(chunks[0].text_content.contains("docs/screenshots"));
        assert_eq!(chunks[0].extraction_quality, 0.3);
        assert_eq!(chunks[0].doc_type, DocumentType::Image);
    }

    #[test]
    fn test_extract_markdown_with_headings() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("doc.md");
        std::fs::write(
            &md_path,
            "# Title\n\nIntro paragraph.\n\n## Section A\n\nContent A.\n\n## Section B\n\nContent B.\n",
        )
        .unwrap();

        let chunks = extract_markdown(&md_path, "doc.md", "hash2").unwrap();
        assert!(chunks.len() >= 2);
        // First chunk should be the "Title" section with "Intro paragraph"
        assert_eq!(chunks[0].section_heading, Some("Title".to_string()));
        assert!(chunks[0].text_content.contains("Intro paragraph"));
        // Second chunk should be "Section A"
        assert_eq!(chunks[1].section_heading, Some("Section A".to_string()));
        assert!(chunks[1].text_content.contains("Content A"));
    }

    #[test]
    fn test_extract_markdown_no_headings() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("flat.md");
        std::fs::write(&md_path, "Just some plain text\nwith no headings.\n").unwrap();

        let chunks = extract_markdown(&md_path, "flat.md", "hash3").unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text_content.contains("plain text"));
        assert_eq!(chunks[0].section_heading, None);
    }

    #[test]
    fn test_extract_markdown_empty() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("empty.md");
        std::fs::write(&md_path, "").unwrap();

        let chunks = extract_markdown(&md_path, "empty.md", "hash4").unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_extract_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let txt_path = dir.path().join("notes.txt");
        std::fs::write(&txt_path, "Line 1\nLine 2\nLine 3\n").unwrap();

        let chunks = extract_plaintext(&txt_path, "notes.txt", "hash5").unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].text_content.contains("Line 1"));
        assert_eq!(chunks[0].doc_type, DocumentType::PlainText);
    }

    #[test]
    fn test_extract_plaintext_empty() {
        let dir = tempfile::tempdir().unwrap();
        let txt_path = dir.path().join("empty.txt");
        std::fs::write(&txt_path, "  \n  \n").unwrap();

        let chunks = extract_plaintext(&txt_path, "empty.txt", "hash6").unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_into_chunks_large_content() {
        // Create content larger than MAX_CHUNK_SIZE
        let mut content = String::new();
        for i in 0..500 {
            content.push_str(&format!("Line {i}: This is a test line with some content.\n"));
        }
        assert!(content.len() > MAX_CHUNK_SIZE);

        let chunks = split_into_chunks(&content, "big.txt", DocumentType::PlainText, "hash7", 1.0);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.text_content.len() <= MAX_CHUNK_SIZE + 100); // allow some slack for last line
        }
    }

    #[test]
    fn test_truncate_to_max_chunk() {
        let short = "hello world";
        assert_eq!(truncate_to_max_chunk(short), "hello world");

        let long = "a ".repeat(MAX_CHUNK_SIZE);
        let truncated = truncate_to_max_chunk(&long);
        assert!(truncated.len() <= MAX_CHUNK_SIZE);
    }

    #[test]
    fn test_extract_document_dispatcher() {
        let dir = tempfile::tempdir().unwrap();

        // SVG dispatch
        let svg_path = dir.path().join("test.svg");
        std::fs::write(&svg_path, "<svg><text>Hello</text></svg>").unwrap();
        assert!(extract_document(&svg_path, "test.svg", "h1").is_ok());

        // Markdown dispatch
        let md_path = dir.path().join("test.md");
        std::fs::write(&md_path, "# Hello\nWorld").unwrap();
        assert!(extract_document(&md_path, "test.md", "h2").is_ok());

        // Plaintext dispatch
        let txt_path = dir.path().join("test.txt");
        std::fs::write(&txt_path, "Hello").unwrap();
        assert!(extract_document(&txt_path, "test.txt", "h3").is_ok());

        // Image dispatch
        let img_path = dir.path().join("test.png");
        std::fs::write(&img_path, b"PNG").unwrap();
        assert!(extract_document(&img_path, "test.png", "h4").is_ok());

        // Unsupported
        let rs_path = dir.path().join("test.rs");
        std::fs::write(&rs_path, "fn main() {}").unwrap();
        assert!(extract_document(&rs_path, "test.rs", "h5").is_err());
    }

    #[test]
    fn test_pdf_without_feature() {
        // Without doc-indexing feature, should return metadata-only chunk
        let dir = tempfile::tempdir().unwrap();
        let pdf_path = dir.path().join("test.pdf");
        let mut f = std::fs::File::create(&pdf_path).unwrap();
        f.write_all(b"%PDF-1.4 fake").unwrap();

        let result = extract_pdf(&pdf_path, "test.pdf", "hashpdf");
        // Without the doc-indexing feature, this returns a metadata-only result
        // With the feature, it would attempt real extraction (and likely fail on fake data)
        #[cfg(not(feature = "doc-indexing"))]
        {
            let chunks = result.unwrap();
            assert_eq!(chunks.len(), 1);
            assert!(chunks[0].text_content.contains("doc-indexing"));
            assert_eq!(chunks[0].extraction_quality, 0.1);
        }
    }
}
