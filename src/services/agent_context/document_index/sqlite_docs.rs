#![cfg_attr(coverage_nightly, coverage(off))]

//! SQLite + FTS5 backend for the document index.
//!
//! Stores document chunks in the same `context.db` as the function index.
//! Uses a separate FTS5 virtual table (`documents_fts`) for BM25 ranking.

use super::types::{DocumentChunk, DocumentResult};
use rusqlite::{params, Connection};

/// Create the documents schema tables.
///
/// Called from the main `create_schema()` in `sqlite_backend.rs` to ensure
/// tables exist even without the `doc-indexing` feature (they'll just be empty).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn create_documents_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            doc_type TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            page_number INTEGER,
            section_heading TEXT,
            text_content TEXT NOT NULL,
            file_checksum TEXT NOT NULL,
            extraction_quality REAL NOT NULL DEFAULT 1.0,
            UNIQUE(file_path, chunk_index)
        );

        CREATE INDEX IF NOT EXISTS idx_documents_file ON documents(file_path);
        CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(doc_type);
        CREATE INDEX IF NOT EXISTS idx_documents_checksum ON documents(file_checksum);",
    )
    .map_err(|e| format!("Failed to create documents schema: {e}"))?;

    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            text_content,
            section_heading,
            file_path,
            tokenize='porter unicode61 remove_diacritics 2'
        );",
    )
    .map_err(|e| format!("Failed to create documents FTS5 table: {e}"))?;

    Ok(())
}

/// Insert document chunks into the database within a transaction.
///
/// Uses upsert (ON CONFLICT REPLACE) for incremental updates.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn insert_document_chunks(
    conn: &Connection,
    chunks: &[DocumentChunk],
) -> Result<usize, String> {
    debug_assert!(!chunks.is_empty(), "chunks must not be empty");
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    let mut inserted = 0;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT OR REPLACE INTO documents (
                    file_path, doc_type, chunk_index, page_number,
                    section_heading, text_content, file_checksum, extraction_quality
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .map_err(|e| format!("Failed to prepare document insert: {e}"))?;

        let mut fts_stmt = tx
            .prepare_cached(
                "INSERT INTO documents_fts (rowid, text_content, section_heading, file_path)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .map_err(|e| format!("Failed to prepare FTS insert: {e}"))?;

        for chunk in chunks {
            let doc_type_str = chunk.doc_type.to_string();
            stmt.execute(params![
                chunk.file_path,
                doc_type_str,
                chunk.chunk_index,
                chunk.page_number,
                chunk.section_heading,
                chunk.text_content,
                chunk.file_checksum,
                chunk.extraction_quality,
            ])
            .map_err(|e| format!("Failed to insert document chunk: {e}"))?;

            let rowid = tx.last_insert_rowid();

            // Delete any existing FTS entry for this rowid before inserting
            let _ = tx.execute("DELETE FROM documents_fts WHERE rowid = ?1", params![rowid]);

            fts_stmt
                .execute(params![
                    rowid,
                    chunk.text_content,
                    chunk.section_heading.as_deref().unwrap_or(""),
                    chunk.file_path,
                ])
                .map_err(|e| format!("Failed to insert FTS entry: {e}"))?;

            inserted += 1;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit document transaction: {e}"))?;

    Ok(inserted)
}

/// Query documents using FTS5 BM25 ranking.
///
/// Returns results ranked by relevance with snippet extraction.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn query_documents(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<DocumentResult>, String> {
    debug_assert!(!query.is_empty(), "query must not be empty");
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    // Sanitize query for FTS5: escape quotes and remove special chars
    let sanitized = sanitize_fts_query(query);
    if sanitized.is_empty() {
        return Ok(vec![]);
    }

    let sql = "SELECT
            d.file_path,
            d.doc_type,
            d.chunk_index,
            d.page_number,
            d.section_heading,
            snippet(documents_fts, 0, '>>>', '<<<', '...', 40) AS snippet,
            rank,
            d.extraction_quality
        FROM documents_fts
        JOIN documents d ON d.id = documents_fts.rowid
        WHERE documents_fts MATCH ?1
        ORDER BY rank
        LIMIT ?2";

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare document query: {e}"))?;

    let results = stmt
        .query_map(params![sanitized, limit as i64], |row| {
            Ok(DocumentResult {
                file_path: row.get(0)?,
                doc_type: row.get(1)?,
                chunk_index: row.get::<_, i64>(2)? as u32,
                page_number: row.get::<_, Option<i64>>(3)?.map(|n| n as u32),
                section_heading: row.get(4)?,
                snippet: row.get(5)?,
                relevance_score: row.get::<_, f64>(6)? as f32,
                extraction_quality: row.get::<_, f64>(7)? as f32,
            })
        })
        .map_err(|e| format!("Failed to query documents: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Check if a file has already been indexed with the given checksum.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn file_is_current(conn: &Connection, file_path: &str, checksum: &str) -> bool {
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
    debug_assert!(!checksum.is_empty(), "checksum must not be empty");
    conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE file_path = ?1 AND file_checksum = ?2",
        params![file_path, checksum],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// Remove all document chunks for a given file path.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn remove_file_documents(conn: &Connection, file_path: &str) -> Result<(), String> {
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
    // Remove FTS entries first (need rowids)
    conn.execute(
        "DELETE FROM documents_fts WHERE rowid IN (SELECT id FROM documents WHERE file_path = ?1)",
        params![file_path],
    )
    .map_err(|e| format!("Failed to remove FTS entries: {e}"))?;

    conn.execute(
        "DELETE FROM documents WHERE file_path = ?1",
        params![file_path],
    )
    .map_err(|e| format!("Failed to remove document entries: {e}"))?;

    Ok(())
}

/// Get the count of indexed documents.
#[cfg(test)]
pub(crate) fn document_count(conn: &Connection) -> usize {
    conn.query_row(
        "SELECT COUNT(DISTINCT file_path) FROM documents",
        [],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0) as usize
}

/// Sanitize a user query for FTS5 MATCH syntax.
///
/// FTS5 has special characters (*, ", ^, NEAR, OR, AND, NOT) that need handling.
/// We tokenize on whitespace and join with implicit AND.
fn sanitize_fts_query(query: &str) -> String {
    debug_assert!(!query.is_empty(), "query must not be empty");
    query
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| {
            // Remove FTS5 operators and special chars
            let cleaned: String = w
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            cleaned
        })
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::super::types::DocumentType;
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_documents_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_schema() {
        let conn = setup_db();
        // Verify tables exist by querying them
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_insert_and_query() {
        let conn = setup_db();

        let chunks = vec![
            DocumentChunk {
                file_path: "docs/architecture.md".to_string(),
                doc_type: DocumentType::Markdown,
                chunk_index: 0,
                page_number: None,
                section_heading: Some("Architecture Overview".to_string()),
                text_content:
                    "The system uses a microservices architecture with event-driven communication."
                        .to_string(),
                file_checksum: "abc123".to_string(),
                extraction_quality: 1.0,
            },
            DocumentChunk {
                file_path: "docs/api.md".to_string(),
                doc_type: DocumentType::Markdown,
                chunk_index: 0,
                page_number: None,
                section_heading: Some("API Reference".to_string()),
                text_content: "REST API endpoints for user management and authentication."
                    .to_string(),
                file_checksum: "def456".to_string(),
                extraction_quality: 1.0,
            },
        ];

        let inserted = insert_document_chunks(&conn, &chunks).unwrap();
        assert_eq!(inserted, 2);

        // Query for architecture
        let results = query_documents(&conn, "architecture microservices", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].file_path, "docs/architecture.md");

        // Query for API
        let results = query_documents(&conn, "authentication REST", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].file_path, "docs/api.md");
    }

    #[test]
    fn test_file_is_current() {
        let conn = setup_db();

        let chunks = vec![DocumentChunk {
            file_path: "test.md".to_string(),
            doc_type: DocumentType::Markdown,
            chunk_index: 0,
            page_number: None,
            section_heading: None,
            text_content: "Test content".to_string(),
            file_checksum: "checksum1".to_string(),
            extraction_quality: 1.0,
        }];

        insert_document_chunks(&conn, &chunks).unwrap();

        assert!(file_is_current(&conn, "test.md", "checksum1"));
        assert!(!file_is_current(&conn, "test.md", "different_checksum"));
        assert!(!file_is_current(&conn, "other.md", "checksum1"));
    }

    #[test]
    fn test_remove_file_documents() {
        let conn = setup_db();

        let chunks = vec![
            DocumentChunk {
                file_path: "a.md".to_string(),
                doc_type: DocumentType::Markdown,
                chunk_index: 0,
                page_number: None,
                section_heading: None,
                text_content: "File A content".to_string(),
                file_checksum: "h1".to_string(),
                extraction_quality: 1.0,
            },
            DocumentChunk {
                file_path: "b.md".to_string(),
                doc_type: DocumentType::Markdown,
                chunk_index: 0,
                page_number: None,
                section_heading: None,
                text_content: "File B content".to_string(),
                file_checksum: "h2".to_string(),
                extraction_quality: 1.0,
            },
        ];

        insert_document_chunks(&conn, &chunks).unwrap();
        assert_eq!(document_count(&conn), 2);

        remove_file_documents(&conn, "a.md").unwrap();
        assert_eq!(document_count(&conn), 1);
        assert!(!file_is_current(&conn, "a.md", "h1"));
        assert!(file_is_current(&conn, "b.md", "h2"));
    }

    #[test]
    fn test_document_count() {
        let conn = setup_db();
        assert_eq!(document_count(&conn), 0);

        let chunks = vec![
            DocumentChunk {
                file_path: "doc1.md".to_string(),
                doc_type: DocumentType::Markdown,
                chunk_index: 0,
                page_number: None,
                section_heading: None,
                text_content: "Content 1".to_string(),
                file_checksum: "h1".to_string(),
                extraction_quality: 1.0,
            },
            DocumentChunk {
                file_path: "doc1.md".to_string(),
                doc_type: DocumentType::Markdown,
                chunk_index: 1,
                page_number: None,
                section_heading: None,
                text_content: "Content 1 part 2".to_string(),
                file_checksum: "h1".to_string(),
                extraction_quality: 1.0,
            },
        ];

        insert_document_chunks(&conn, &chunks).unwrap();
        // Two chunks but same file = 1 distinct file
        assert_eq!(document_count(&conn), 1);
    }

    #[test]
    fn test_empty_query() {
        let conn = setup_db();
        let results = query_documents(&conn, "", 10).unwrap();
        assert!(results.is_empty());

        let results = query_documents(&conn, "   ", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_sanitize_fts_query() {
        assert_eq!(sanitize_fts_query("hello world"), "hello world");
        assert_eq!(sanitize_fts_query("  hello  world  "), "hello world");
        assert_eq!(sanitize_fts_query("hello \"world\""), "hello world");
        assert_eq!(sanitize_fts_query("test*query"), "testquery");
        assert_eq!(sanitize_fts_query(""), "");
        assert_eq!(sanitize_fts_query("***"), "");
    }

    #[test]
    fn test_incremental_upsert() {
        let conn = setup_db();

        // Insert initial version
        let chunks_v1 = vec![DocumentChunk {
            file_path: "doc.md".to_string(),
            doc_type: DocumentType::Markdown,
            chunk_index: 0,
            page_number: None,
            section_heading: None,
            text_content: "Original content".to_string(),
            file_checksum: "v1".to_string(),
            extraction_quality: 1.0,
        }];
        insert_document_chunks(&conn, &chunks_v1).unwrap();

        // Upsert updated version (same file_path + chunk_index)
        let chunks_v2 = vec![DocumentChunk {
            file_path: "doc.md".to_string(),
            doc_type: DocumentType::Markdown,
            chunk_index: 0,
            page_number: None,
            section_heading: None,
            text_content: "Updated content".to_string(),
            file_checksum: "v2".to_string(),
            extraction_quality: 1.0,
        }];
        insert_document_chunks(&conn, &chunks_v2).unwrap();

        // Should still be 1 file, 1 chunk
        assert_eq!(document_count(&conn), 1);
        assert!(file_is_current(&conn, "doc.md", "v2"));
        assert!(!file_is_current(&conn, "doc.md", "v1"));
    }
}
