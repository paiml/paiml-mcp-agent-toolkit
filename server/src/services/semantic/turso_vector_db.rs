// Turso Vector Database Integration
// PMAT-SEARCH-003: Store and query code embeddings using SQLite
//
// GREEN Phase: Full implementation with cosine similarity search

use rusqlite::{params, Connection};
use serde_json;
use std::path::Path;
use std::sync::Mutex;

/// Turso vector database for storing code embeddings
pub struct TursoVectorDB {
    connection: Mutex<Connection>,
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

impl TursoVectorDB {
    /// Create new local SQLite database
    ///
    /// # Arguments
    /// * `path` - Path to SQLite database file
    ///
    /// # Returns
    /// Database instance
    pub async fn new_local<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| format!("Failed to open database: {e}"))?;

        // Create schema
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS code_embeddings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    file_path TEXT NOT NULL,
                    chunk_name TEXT NOT NULL,
                    chunk_type TEXT NOT NULL,
                    language TEXT NOT NULL,
                    start_line INTEGER NOT NULL,
                    end_line INTEGER NOT NULL,
                    content_checksum TEXT NOT NULL,
                    embedding TEXT NOT NULL,
                    model TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    UNIQUE(file_path, chunk_name, content_checksum)
                )",
                [],
            )
            .map_err(|e| format!("Failed to create table: {e}"))?;

        // Create indexes
        connection
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_file_path ON code_embeddings(file_path)",
                [],
            )
            .map_err(|e| format!("Failed to create index: {e}"))?;

        connection
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_language ON code_embeddings(language)",
                [],
            )
            .map_err(|e| format!("Failed to create index: {e}"))?;

        connection
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_checksum ON code_embeddings(content_checksum)",
                [],
            )
            .map_err(|e| format!("Failed to create index: {e}"))?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Insert or update embedding entry
    ///
    /// # Arguments
    /// * `entry` - Embedding entry to insert
    ///
    /// # Returns
    /// Row ID of inserted/updated entry
    pub async fn insert(&self, entry: &EmbeddingEntry) -> Result<i64, String> {
        let embedding_json = serde_json::to_string(&entry.embedding)
            .map_err(|e| format!("Failed to serialize embedding: {e}"))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = self
            .connection
            .lock()
            .map_err(|e| format!("Failed to lock connection: {e}"))?;

        conn.execute(
            "INSERT INTO code_embeddings
                (file_path, chunk_name, chunk_type, language, start_line, end_line,
                 content_checksum, embedding, model, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(file_path, chunk_name, content_checksum)
                DO UPDATE SET
                    embedding = excluded.embedding,
                    model = excluded.model,
                    created_at = excluded.created_at",
            params![
                &entry.file_path,
                &entry.chunk_name,
                &entry.chunk_type,
                &entry.language,
                entry.start_line as i64,
                entry.end_line as i64,
                &entry.content_checksum,
                &embedding_json,
                &entry.model,
                timestamp,
            ],
        )
        .map_err(|e| format!("Failed to insert: {e}"))?;

        Ok(conn.last_insert_rowid())
    }

    /// Batch insert multiple entries
    ///
    /// # Arguments
    /// * `entries` - Array of entries to insert
    ///
    /// # Returns
    /// Array of row IDs
    pub async fn batch_insert(&self, entries: &[EmbeddingEntry]) -> Result<Vec<i64>, String> {
        let mut ids = Vec::new();

        for entry in entries {
            let id = self.insert(entry).await?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// Query all embeddings for a file
    ///
    /// # Arguments
    /// * `file_path` - Path to file
    ///
    /// # Returns
    /// Array of search results
    pub async fn query_by_file(&self, file_path: &str) -> Result<Vec<SearchResult>, String> {
        let conn = self
            .connection
            .lock()
            .map_err(|e| format!("Failed to lock connection: {e}"))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, chunk_name, chunk_type, language, start_line, end_line, embedding
                 FROM code_embeddings
                 WHERE file_path = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let results = stmt
            .query_map(params![file_path], |row| {
                let embedding_json: String = row.get(7)?;
                let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;

                Ok(SearchResult {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    chunk_name: row.get(2)?,
                    chunk_type: row.get(3)?,
                    language: row.get(4)?,
                    start_line: row.get::<_, i64>(5)? as usize,
                    end_line: row.get::<_, i64>(6)? as usize,
                    similarity: 1.0, // Not applicable for file query
                    embedding,
                })
            })
            .map_err(|e| format!("Failed to execute query: {e}"))?;

        let mut result_vec = Vec::new();
        for result in results {
            result_vec.push(result.map_err(|e| format!("Failed to process row: {e}"))?);
        }

        Ok(result_vec)
    }

    /// Query embeddings by language
    ///
    /// # Arguments
    /// * `language` - Programming language
    ///
    /// # Returns
    /// Array of search results
    pub async fn query_by_language(&self, language: &str) -> Result<Vec<SearchResult>, String> {
        let conn = self
            .connection
            .lock()
            .map_err(|e| format!("Failed to lock connection: {e}"))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, chunk_name, chunk_type, language, start_line, end_line, embedding
                 FROM code_embeddings
                 WHERE language = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let results = stmt
            .query_map(params![language], |row| {
                let embedding_json: String = row.get(7)?;
                let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;

                Ok(SearchResult {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    chunk_name: row.get(2)?,
                    chunk_type: row.get(3)?,
                    language: row.get(4)?,
                    start_line: row.get::<_, i64>(5)? as usize,
                    end_line: row.get::<_, i64>(6)? as usize,
                    similarity: 1.0,
                    embedding,
                })
            })
            .map_err(|e| format!("Failed to execute query: {e}"))?;

        let mut result_vec = Vec::new();
        for result in results {
            result_vec.push(result.map_err(|e| format!("Failed to process row: {e}"))?);
        }

        Ok(result_vec)
    }

    /// Vector similarity search using cosine similarity
    ///
    /// # Arguments
    /// * `query` - Query vector (1536 dimensions)
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    /// Array of search results sorted by similarity (highest first)
    pub async fn similarity_search(
        &self,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        // Get all embeddings (in-memory similarity for now)
        let conn = self
            .connection
            .lock()
            .map_err(|e| format!("Failed to lock connection: {e}"))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, file_path, chunk_name, chunk_type, language, start_line, end_line, embedding
                 FROM code_embeddings",
            )
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let results = stmt
            .query_map([], |row| {
                let embedding_json: String = row.get(7)?;
                let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;

                Ok(SearchResult {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    chunk_name: row.get(2)?,
                    chunk_type: row.get(3)?,
                    language: row.get(4)?,
                    start_line: row.get::<_, i64>(5)? as usize,
                    end_line: row.get::<_, i64>(6)? as usize,
                    similarity: 0.0, // Will be computed
                    embedding,
                })
            })
            .map_err(|e| format!("Failed to execute query: {e}"))?;

        let mut result_vec = Vec::new();
        for result in results {
            let mut entry = result.map_err(|e| format!("Failed to process row: {e}"))?;
            entry.similarity = Self::cosine_similarity(query, &entry.embedding);
            result_vec.push(entry);
        }

        // Sort by similarity (descending)
        result_vec.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // Apply limit
        result_vec.truncate(limit);

        Ok(result_vec)
    }

    /// Delete embeddings for a file
    ///
    /// # Arguments
    /// * `file_path` - Path to file
    ///
    /// # Returns
    /// Number of rows deleted
    pub async fn delete_by_file(&self, file_path: &str) -> Result<usize, String> {
        let conn = self
            .connection
            .lock()
            .map_err(|e| format!("Failed to lock connection: {e}"))?;

        conn.execute(
            "DELETE FROM code_embeddings WHERE file_path = ?1",
            params![file_path],
        )
        .map_err(|e| format!("Failed to delete: {e}"))
    }

    /// Compute cosine similarity between two vectors
    ///
    /// # Arguments
    /// * `v1` - First vector
    /// * `v2` - Second vector
    ///
    /// # Returns
    /// Cosine similarity score (0.0 to 1.0)
    pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f64 {
        if v1.len() != v2.len() {
            return 0.0;
        }

        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();

        let magnitude1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        (dot_product / (magnitude1 * magnitude2)) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        let sim = TursoVectorDB::cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.001);

        // Orthogonal vectors
        let v3 = vec![1.0, 0.0, 0.0];
        let v4 = vec![0.0, 1.0, 0.0];
        let sim2 = TursoVectorDB::cosine_similarity(&v3, &v4);
        assert!((sim2 - 0.0).abs() < 0.001);

        // Opposite vectors
        let v5 = vec![1.0, 0.0, 0.0];
        let v6 = vec![-1.0, 0.0, 0.0];
        let sim3 = TursoVectorDB::cosine_similarity(&v5, &v6);
        assert!((sim3 + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_serialization() {
        let embedding = vec![0.1, 0.2, 0.3];
        let json = serde_json::to_string(&embedding).unwrap();
        let deserialized: Vec<f32> = serde_json::from_str(&json).unwrap();
        assert_eq!(embedding, deserialized);
    }
}
