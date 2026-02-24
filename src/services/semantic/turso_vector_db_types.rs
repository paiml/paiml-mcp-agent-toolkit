/// Turso vector database for storing code embeddings
/// Now backed by trueno-rag's SIMD-accelerated VectorStore
pub struct TursoVectorDB {
    store: RwLock<VectorStore>,
    /// Maps file_path -> list of ChunkIds for that file
    file_index: RwLock<HashMap<String, Vec<ChunkId>>>,
    /// Maps ChunkId -> EmbeddingEntry metadata
    metadata: RwLock<HashMap<ChunkId, EmbeddingMetadata>>,
    /// Auto-increment ID counter
    next_id: RwLock<i64>,
}

/// Internal metadata for each embedding
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EmbeddingMetadata {
    id: i64,
    file_path: String,
    chunk_name: String,
    chunk_type: String,
    language: String,
    start_line: usize,
    end_line: usize,
    content_checksum: String,
    model: String,
}

/// Embedding entry to insert into database
#[derive(Debug, Clone)]
pub struct EmbeddingEntry {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content_checksum: String,
    pub embedding: Vec<f32>,
    pub model: String,
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: i64,
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub start_line: usize,
    pub end_line: usize,
    pub similarity: f64,
    pub embedding: Vec<f32>,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DbStats {
    pub total_entries: usize,
    pub unique_files: usize,
}
