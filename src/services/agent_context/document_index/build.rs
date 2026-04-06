#![cfg_attr(coverage_nightly, coverage(off))]

//! Document index builder — lazy indexing on first `--docs` query.
//!
//! Walks the project directory for document files (PDF, SVG, images, markdown, plaintext),
//! extracts text content, and inserts into SQLite for FTS5 search.
//! Uses SHA256 checksums for incremental updates.

use super::extractors::{extract_document, is_document_file};
use super::sqlite_docs::{file_is_current, insert_document_chunks, remove_file_documents};
use ignore::WalkBuilder;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;

/// Result of building the document index.
pub(crate) struct DocumentBuildResult {
    /// Number of files scanned
    pub files_scanned: usize,
    /// Number of new/updated files indexed
    pub files_indexed: usize,
    /// Number of files skipped (unchanged)
    pub files_skipped: usize,
    /// Total chunks inserted
    pub chunks_inserted: usize,
    /// Errors encountered (non-fatal)
    pub errors: Vec<String>,
}

/// Build the document index for a project.
///
/// Walks the project directory, finds document files, extracts text,
/// and inserts into the SQLite database. Uses checksums for incremental updates.
pub(crate) fn build_document_index(
    conn: &Connection,
    project_path: &Path,
) -> Result<DocumentBuildResult, String> {
    let project_root = project_path
        .canonicalize()
        .map_err(|e| format!("Invalid project path: {e}"))?;

    let mut result = DocumentBuildResult {
        files_scanned: 0,
        files_indexed: 0,
        files_skipped: 0,
        chunks_inserted: 0,
        errors: Vec::new(),
    };

    // Collect all document files currently in the index for stale detection
    let mut indexed_files: HashSet<String> = HashSet::new();
    if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT file_path FROM documents") {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                indexed_files.insert(row);
            }
        }
    }

    let mut seen_files: HashSet<String> = HashSet::new();

    // Walk the project directory respecting .gitignore
    for entry in WalkBuilder::new(&project_root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|e| !is_ignored_dir(e.path()))
        .build()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || !is_document_file(path) {
            continue;
        }

        result.files_scanned += 1;

        let relative_path = match path.strip_prefix(&project_root) {
            Ok(rel) => rel.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        seen_files.insert(relative_path.clone());

        // Compute checksum for incremental update detection
        let checksum = match compute_document_checksum(path) {
            Ok(cs) => cs,
            Err(e) => {
                result.errors.push(format!("{relative_path}: {e}"));
                continue;
            }
        };

        // Skip if file hasn't changed
        if file_is_current(conn, &relative_path, &checksum) {
            result.files_skipped += 1;
            continue;
        }

        // Remove old chunks for this file (if any)
        let _ = remove_file_documents(conn, &relative_path);

        // Extract and insert
        match extract_document(path, &relative_path, &checksum) {
            Ok(chunks) => {
                if !chunks.is_empty() {
                    match insert_document_chunks(conn, &chunks) {
                        Ok(count) => {
                            result.chunks_inserted += count;
                            result.files_indexed += 1;
                        }
                        Err(e) => {
                            result
                                .errors
                                .push(format!("{relative_path}: insert failed: {e}"));
                        }
                    }
                }
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("{relative_path}: extraction failed: {e}"));
            }
        }
    }

    // Remove stale entries (files that no longer exist on disk)
    for stale_file in indexed_files.difference(&seen_files) {
        let _ = remove_file_documents(conn, stale_file);
    }

    Ok(result)
}

/// Check if a directory should be skipped during document indexing.
///
/// Mirrors the skip list from `function_index/helpers.rs:is_ignored_dir()`.
fn is_ignored_dir(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    matches!(
        name,
        "target"
            | "node_modules"
            | ".git"
            | ".pmat"
            | "__pycache__"
            | "venv"
            | ".venv"
            | "dist"
            | "build"
            | ".next"
            | ".cache"
            | "vendor"
            | "third_party"
            | "third-party"
            | "external"
            | "deps"
            | "book"
            | "theme"
            | "fixtures"
            | ".cargo"
    )
}

/// Compute SHA256 checksum of a file's contents.
fn compute_document_checksum(path: &Path) -> Result<String, String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::super::sqlite_docs::{create_documents_schema, document_count};
    use super::*;

    fn setup_test_project() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        create_documents_schema(&conn).unwrap();

        // Create some document files
        std::fs::write(
            dir.path().join("README.md"),
            "# Project\n\nThis is a test project.\n\n## Usage\n\nRun the thing.\n",
        )
        .unwrap();

        std::fs::write(
            dir.path().join("notes.txt"),
            "Important notes about the architecture.\n",
        )
        .unwrap();

        std::fs::write(
            dir.path().join("diagram.svg"),
            r#"<svg><text x="10" y="20">System Architecture</text></svg>"#,
        )
        .unwrap();

        // Create a subdirectory with more docs
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::write(
            dir.path().join("docs/design.md"),
            "# Design Doc\n\nDetailed design.\n",
        )
        .unwrap();

        // Also create a code file (should be ignored)
        std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

        (dir, conn)
    }

    #[test]
    fn test_build_document_index() {
        let (dir, conn) = setup_test_project();

        let result = build_document_index(&conn, dir.path()).unwrap();
        assert!(result.files_scanned >= 4); // README.md, notes.txt, diagram.svg, docs/design.md
        assert!(result.files_indexed >= 4);
        assert_eq!(result.files_skipped, 0);
        assert!(result.chunks_inserted >= 4);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_incremental_build() {
        let (dir, conn) = setup_test_project();

        // First build
        let r1 = build_document_index(&conn, dir.path()).unwrap();
        assert!(r1.files_indexed >= 4);

        // Second build (no changes)
        let r2 = build_document_index(&conn, dir.path()).unwrap();
        assert_eq!(r2.files_indexed, 0);
        assert!(r2.files_skipped >= 4);

        // Modify a file
        std::fs::write(
            dir.path().join("notes.txt"),
            "Updated notes about the system.\n",
        )
        .unwrap();

        // Third build (one file changed)
        let r3 = build_document_index(&conn, dir.path()).unwrap();
        assert_eq!(r3.files_indexed, 1);
        assert!(r3.files_skipped >= 3);
    }

    #[test]
    fn test_stale_file_removal() {
        let (dir, conn) = setup_test_project();

        // Build index
        build_document_index(&conn, dir.path()).unwrap();
        let count_before = document_count(&conn);
        assert!(count_before >= 4);

        // Delete a file
        std::fs::remove_file(dir.path().join("notes.txt")).unwrap();

        // Rebuild — should remove stale entries
        build_document_index(&conn, dir.path()).unwrap();
        let count_after = document_count(&conn);
        assert!(count_after < count_before);
    }

    #[test]
    fn test_is_ignored_dir() {
        assert!(is_ignored_dir(Path::new("/project/target")));
        assert!(is_ignored_dir(Path::new("/project/node_modules")));
        assert!(is_ignored_dir(Path::new("/project/.git")));
        assert!(is_ignored_dir(Path::new("/project/.pmat")));
        assert!(!is_ignored_dir(Path::new("/project/src")));
        assert!(!is_ignored_dir(Path::new("/project/docs")));
    }

    #[test]
    fn test_compute_document_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello world").unwrap();

        let checksum1 = compute_document_checksum(&path).unwrap();
        let checksum2 = compute_document_checksum(&path).unwrap();
        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum1.len(), 64); // SHA256 hex

        std::fs::write(&path, "different content").unwrap();
        let checksum3 = compute_document_checksum(&path).unwrap();
        assert_ne!(checksum1, checksum3);
    }
}
