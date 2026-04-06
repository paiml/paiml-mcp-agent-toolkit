// TRUENO-RAG-3-CHUNKER: Text Chunking with Overlap
// Integrates trueno-rag RecursiveChunker for RAG pipelines

/// Chunk text with fixed-size chunks and overlap for RAG retrieval
///
/// This function uses trueno-rag's chunking approach with overlap to ensure
/// that context is preserved across chunk boundaries, improving retrieval quality.
///
/// # Arguments
/// * `text` - The text to chunk
/// * `chunk_size` - Target chunk size in characters
/// * `overlap` - Number of characters to overlap between chunks
///
/// # Returns
/// Vector of text chunks with overlap applied
pub fn chunk_text_with_overlap(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    debug_assert!(!text.is_empty(), "text must not be empty");
    if text.is_empty() {
        return Vec::new();
    }

    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    // Use trueno-rag's RecursiveChunker internally
    use trueno_rag::chunk::{Chunker, RecursiveChunker};
    use trueno_rag::Document;

    let chunker = RecursiveChunker::new(chunk_size, overlap);
    let doc = Document::new(text);

    match chunker.chunk(&doc) {
        Ok(chunks) => chunks
            .into_iter()
            .map(|c: trueno_rag::Chunk| c.content)
            .collect(),
        Err(_) => {
            // Fallback to simple fixed-size chunking
            chunk_text_fixed(text, chunk_size, overlap)
        }
    }
}

/// Chunk text using recursive separators (paragraph, sentence, word boundaries)
///
/// This function prefers semantic boundaries over arbitrary character splits,
/// producing more coherent chunks for embedding and retrieval.
///
/// # Arguments
/// * `text` - The text to chunk
/// * `chunk_size` - Target chunk size in characters
/// * `overlap` - Number of characters to overlap between chunks
///
/// # Returns
/// Vector of text chunks respecting semantic boundaries
pub fn chunk_text_recursive(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    debug_assert!(!text.is_empty(), "text must not be empty");
    if text.is_empty() {
        return Vec::new();
    }

    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    // Use trueno-rag's RecursiveChunker with custom separators
    use trueno_rag::chunk::{Chunker, RecursiveChunker};
    use trueno_rag::Document;

    let chunker = RecursiveChunker::new(chunk_size, overlap).with_separators(vec![
        "\n\n".to_string(), // Paragraph boundary
        "\n".to_string(),   // Line boundary
        ". ".to_string(),   // Sentence boundary
        ", ".to_string(),   // Clause boundary
        " ".to_string(),    // Word boundary
    ]);

    let doc = Document::new(text);

    match chunker.chunk(&doc) {
        Ok(chunks) => chunks
            .into_iter()
            .map(|c: trueno_rag::Chunk| c.content)
            .collect(),
        Err(_) => {
            // Fallback to overlap chunking
            chunk_text_with_overlap(text, chunk_size, overlap)
        }
    }
}

/// Simple fixed-size text chunking with overlap (fallback implementation)
///
/// # Arguments
/// * `text` - The text to chunk
/// * `chunk_size` - Target chunk size in characters
/// * `overlap` - Number of characters to overlap between chunks
///
/// # Returns
/// Vector of text chunks with overlap applied
pub fn chunk_text_fixed(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    debug_assert!(!text.is_empty(), "text must not be empty");
    if text.is_empty() {
        return Vec::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);

        if end >= chars.len() {
            break;
        }

        // Move start, accounting for overlap
        let step = chunk_size.saturating_sub(overlap);
        start += if step == 0 { 1 } else { step };
    }

    chunks
}
