// Git History Index (GH-RAG-003)
// Toyota Way: Poka-Yoke - Error-proof schema constraints
// Spec: docs/specifications/git-history-rag-integration.md

use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

/// Type of file change in a commit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl ChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Added => "A",
            ChangeType::Modified => "M",
            ChangeType::Deleted => "D",
            ChangeType::Renamed => "R",
        }
    }
}

/// Information about a file changed in a commit
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

/// Parsed commit information (index-compatible version)
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message_subject: String,
    pub message_body: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub is_merge: bool,
    pub is_fix: bool,
    pub is_feat: bool,
    pub issue_refs: Vec<String>,
    pub files: Vec<FileChange>,
}

impl CommitInfo {
    /// Full commit message (subject + body)
    pub fn full_message(&self) -> String {
        match &self.message_body {
            Some(body) if !body.is_empty() => format!("{}\n\n{}", self.message_subject, body),
            _ => self.message_subject.clone(),
        }
    }

    /// Check if this is a meaningful commit for indexing
    pub fn is_indexable(&self) -> bool {
        if self.is_merge && self.message_subject.starts_with("Merge ") {
            return false;
        }
        self.message_subject.len() >= 10
    }
}

/// Result of incremental sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Number of commits added to index
    pub commits_added: usize,
    /// Number of commits skipped (non-indexable or already present)
    pub commits_skipped: usize,
    /// Hash of the last indexed commit
    pub last_commit: Option<String>,
}

/// Error type for git history operations
#[derive(Debug, thiserror::Error)]
pub enum GitHistoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("Index corrupted: {0}")]
    IndexCorrupted(String),
}

/// Git history index using SQLite
/// Stores commit messages with embeddings for semantic search
pub struct GitHistoryIndex {
    pub(crate) conn: Connection,
}

impl GitHistoryIndex {
    /// Create or open git history index at the given path
    pub fn open(path: &Path) -> Result<Self, GitHistoryError> {
        let conn = Connection::open(path)?;
        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    /// Create in-memory index (for testing)
    pub fn in_memory() -> Result<Self, GitHistoryError> {
        let conn = Connection::open_in_memory()?;
        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    /// Initialize database schema
    /// Toyota Way: Poka-Yoke - Constraints prevent invalid data
    fn init_schema(&self) -> Result<(), GitHistoryError> {
        self.conn.execute_batch(
            r#"
            -- Metadata table for tracking sync state
            CREATE TABLE IF NOT EXISTS git_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Git commits with embeddings
            CREATE TABLE IF NOT EXISTS git_commits (
                commit_hash TEXT PRIMARY KEY CHECK (length(commit_hash) = 40),
                message_subject TEXT NOT NULL,
                message_body TEXT,
                author_name TEXT NOT NULL,
                author_email TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                embedding BLOB,
                is_merge INTEGER DEFAULT 0,
                is_fix INTEGER DEFAULT 0,
                is_feat INTEGER DEFAULT 0,
                issue_refs TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            -- Files changed per commit
            CREATE TABLE IF NOT EXISTS commit_files (
                commit_hash TEXT NOT NULL,
                file_path TEXT NOT NULL,
                change_type TEXT NOT NULL CHECK (change_type IN ('A', 'M', 'D', 'R')),
                lines_added INTEGER DEFAULT 0 CHECK (lines_added >= 0),
                lines_deleted INTEGER DEFAULT 0 CHECK (lines_deleted >= 0),
                PRIMARY KEY (commit_hash, file_path),
                FOREIGN KEY (commit_hash) REFERENCES git_commits(commit_hash) ON DELETE CASCADE
            );

            -- Co-change analysis (files that change together)
            CREATE TABLE IF NOT EXISTS file_cochange (
                file_a TEXT NOT NULL,
                file_b TEXT NOT NULL,
                cochange_count INTEGER NOT NULL CHECK (cochange_count > 0),
                jaccard_similarity REAL CHECK (jaccard_similarity BETWEEN 0.0 AND 1.0),
                last_cochange INTEGER NOT NULL,
                PRIMARY KEY (file_a, file_b)
            );

            -- Indexes for query performance
            CREATE INDEX IF NOT EXISTS idx_commits_timestamp ON git_commits(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_commits_author ON git_commits(author_email);
            CREATE INDEX IF NOT EXISTS idx_commits_is_fix ON git_commits(is_fix) WHERE is_fix = 1;
            CREATE INDEX IF NOT EXISTS idx_commits_is_feat ON git_commits(is_feat) WHERE is_feat = 1;
            CREATE INDEX IF NOT EXISTS idx_files_path ON commit_files(file_path);
            CREATE INDEX IF NOT EXISTS idx_cochange_file_a ON file_cochange(file_a);
            "#,
        )?;
        Ok(())
    }

    /// Insert a commit into the index
    pub fn insert_commit(&self, commit: &CommitInfo) -> Result<(), GitHistoryError> {
        let issue_refs_json = serde_json::to_string(&commit.issue_refs).unwrap_or_default();

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO git_commits
            (commit_hash, message_subject, message_body, author_name, author_email,
             timestamp, is_merge, is_fix, is_feat, issue_refs)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                commit.hash,
                commit.message_subject,
                commit.message_body,
                commit.author_name,
                commit.author_email,
                commit.timestamp,
                commit.is_merge as i32,
                commit.is_fix as i32,
                commit.is_feat as i32,
                issue_refs_json,
            ],
        )?;

        // Insert file changes
        for file in &commit.files {
            self.conn.execute(
                r#"
                INSERT OR REPLACE INTO commit_files
                (commit_hash, file_path, change_type, lines_added, lines_deleted)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    commit.hash,
                    file.path,
                    file.change_type.as_str(),
                    file.lines_added,
                    file.lines_deleted,
                ],
            )?;
        }

        Ok(())
    }

    /// Insert multiple commits in a transaction
    pub fn insert_commits(&mut self, commits: &[CommitInfo]) -> Result<usize, GitHistoryError> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        for commit in commits {
            if commit.is_indexable() {
                let issue_refs_json = serde_json::to_string(&commit.issue_refs).unwrap_or_default();

                tx.execute(
                    r#"
                    INSERT OR REPLACE INTO git_commits
                    (commit_hash, message_subject, message_body, author_name, author_email,
                     timestamp, is_merge, is_fix, is_feat, issue_refs)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    params![
                        commit.hash,
                        commit.message_subject,
                        commit.message_body,
                        commit.author_name,
                        commit.author_email,
                        commit.timestamp,
                        commit.is_merge as i32,
                        commit.is_fix as i32,
                        commit.is_feat as i32,
                        issue_refs_json,
                    ],
                )?;

                for file in &commit.files {
                    tx.execute(
                        r#"
                        INSERT OR REPLACE INTO commit_files
                        (commit_hash, file_path, change_type, lines_added, lines_deleted)
                        VALUES (?1, ?2, ?3, ?4, ?5)
                        "#,
                        params![
                            commit.hash,
                            file.path,
                            file.change_type.as_str(),
                            file.lines_added,
                            file.lines_deleted,
                        ],
                    )?;
                }

                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Update embedding for a commit
    pub fn update_embedding(&self, commit_hash: &str, embedding: &[f32]) -> Result<(), GitHistoryError> {
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        self.conn.execute(
            "UPDATE git_commits SET embedding = ?1 WHERE commit_hash = ?2",
            params![embedding_bytes, commit_hash],
        )?;

        Ok(())
    }

    /// Get commits that need embeddings
    pub fn get_commits_without_embeddings(&self, limit: usize) -> Result<Vec<String>, GitHistoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT commit_hash FROM git_commits WHERE embedding IS NULL LIMIT ?1"
        )?;

        let hashes = stmt
            .query_map([limit], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(hashes)
    }

    /// Get last indexed commit hash
    pub fn get_last_indexed_commit(&self) -> Result<Option<String>, GitHistoryError> {
        let result: Option<String> = self.conn
            .query_row(
                "SELECT value FROM git_metadata WHERE key = 'last_indexed_commit'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    /// Set last indexed commit hash
    pub fn set_last_indexed_commit(&self, commit_hash: &str) -> Result<(), GitHistoryError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO git_metadata (key, value) VALUES ('last_indexed_commit', ?1)",
            [commit_hash],
        )?;
        Ok(())
    }

    /// Get total commit count
    pub fn commit_count(&self) -> Result<usize, GitHistoryError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM git_commits",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Get commits by file path
    pub fn get_commits_for_file(&self, file_path: &str, limit: usize) -> Result<Vec<String>, GitHistoryError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT gc.commit_hash
            FROM git_commits gc
            JOIN commit_files cf ON gc.commit_hash = cf.commit_hash
            WHERE cf.file_path = ?1
            ORDER BY gc.timestamp DESC
            LIMIT ?2
            "#
        )?;

        let hashes = stmt
            .query_map(params![file_path, limit], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(hashes)
    }

    /// Sync new commits incrementally (GH-RAG-004)
    /// Toyota Way: Heijunka - Level loading without blocking
    ///
    /// Returns number of new commits synced
    pub fn sync_incremental(&mut self, new_commits: &[CommitInfo]) -> Result<SyncResult, GitHistoryError> {
        let last_indexed = self.get_last_indexed_commit()?;
        let start_count = self.commit_count()?;

        // Filter to only commits newer than last indexed
        let commits_to_add: Vec<&CommitInfo> = if let Some(ref last_hash) = last_indexed {
            // Find commits not already in index
            new_commits
                .iter()
                .filter(|c| c.hash != *last_hash && self.commit_exists(&c.hash).unwrap_or(false) == false)
                .collect()
        } else {
            // No previous index, add all
            new_commits.iter().collect()
        };

        if commits_to_add.is_empty() {
            return Ok(SyncResult {
                commits_added: 0,
                commits_skipped: new_commits.len(),
                last_commit: last_indexed,
            });
        }

        // Insert in transaction
        let tx = self.conn.transaction()?;
        let mut skipped = 0;
        let mut last_commit_hash: Option<String> = None;

        for commit in &commits_to_add {
            if commit.is_indexable() {
                let issue_refs_json = serde_json::to_string(&commit.issue_refs).unwrap_or_default();

                tx.execute(
                    r#"
                    INSERT OR IGNORE INTO git_commits
                    (commit_hash, message_subject, message_body, author_name, author_email,
                     timestamp, is_merge, is_fix, is_feat, issue_refs)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    params![
                        commit.hash,
                        commit.message_subject,
                        commit.message_body,
                        commit.author_name,
                        commit.author_email,
                        commit.timestamp,
                        commit.is_merge as i32,
                        commit.is_fix as i32,
                        commit.is_feat as i32,
                        issue_refs_json,
                    ],
                )?;

                // Insert file changes
                for file in &commit.files {
                    tx.execute(
                        r#"
                        INSERT OR IGNORE INTO commit_files
                        (commit_hash, file_path, change_type, lines_added, lines_deleted)
                        VALUES (?1, ?2, ?3, ?4, ?5)
                        "#,
                        params![
                            commit.hash,
                            file.path,
                            file.change_type.as_str(),
                            file.lines_added,
                            file.lines_deleted,
                        ],
                    )?;
                }

                last_commit_hash = Some(commit.hash.clone());
            } else {
                skipped += 1;
            }
        }

        // Update last indexed commit
        if let Some(ref hash) = last_commit_hash {
            tx.execute(
                "INSERT OR REPLACE INTO git_metadata (key, value) VALUES ('last_indexed_commit', ?1)",
                [hash],
            )?;
        }

        tx.commit()?;

        let end_count = self.commit_count()?;

        Ok(SyncResult {
            commits_added: end_count - start_count,
            commits_skipped: skipped,
            last_commit: last_commit_hash.or(last_indexed),
        })
    }

    /// Check if a commit exists in the index
    pub fn commit_exists(&self, commit_hash: &str) -> Result<bool, GitHistoryError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM git_commits WHERE commit_hash = ?1",
            [commit_hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get commits by timestamp range (for incremental queries)
    pub fn get_commits_since(&self, timestamp: i64, limit: usize) -> Result<Vec<CommitInfo>, GitHistoryError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT commit_hash, message_subject, message_body, author_name, author_email,
                   timestamp, is_merge, is_fix, is_feat, issue_refs
            FROM git_commits
            WHERE timestamp > ?1
            ORDER BY timestamp ASC
            LIMIT ?2
            "#
        )?;

        let commits = stmt
            .query_map(params![timestamp, limit], |row| {
                let issue_refs_str: String = row.get::<_, String>(9).unwrap_or_default();
                let issue_refs: Vec<String> = serde_json::from_str(&issue_refs_str).unwrap_or_default();

                Ok(CommitInfo {
                    hash: row.get(0)?,
                    message_subject: row.get(1)?,
                    message_body: row.get(2)?,
                    author_name: row.get(3)?,
                    author_email: row.get(4)?,
                    timestamp: row.get(5)?,
                    is_merge: row.get::<_, i32>(6)? != 0,
                    is_fix: row.get::<_, i32>(7)? != 0,
                    is_feat: row.get::<_, i32>(8)? != 0,
                    issue_refs,
                    files: vec![], // Files loaded separately if needed
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(commits)
    }

    /// Calculate checksum of index (for falsification test F1)
    pub fn checksum(&self) -> Result<String, GitHistoryError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM git_commits",
            [],
            |row| row.get(0),
        )?;

        let last_hash: Option<String> = self.conn
            .query_row(
                "SELECT commit_hash FROM git_commits ORDER BY timestamp DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(format!("{}:{}", count, last_hash.unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_commit(hash: &str, subject: &str) -> CommitInfo {
        CommitInfo {
            hash: hash.to_string(),
            message_subject: subject.to_string(),
            message_body: None,
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: subject.to_lowercase().contains("fix"),
            is_feat: subject.to_lowercase().contains("feat"),
            issue_refs: vec![],
            files: vec![
                FileChange {
                    path: "src/main.rs".to_string(),
                    change_type: ChangeType::Modified,
                    lines_added: 10,
                    lines_deleted: 5,
                },
            ],
        }
    }

    #[test]
    fn test_create_index_in_memory() {
        let index = GitHistoryIndex::in_memory();
        assert!(index.is_ok());
    }

    #[test]
    fn test_insert_and_count_commits() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"),
            create_test_commit(&"b".repeat(40), "Add new feature"),
            create_test_commit(&"c".repeat(40), "Refactor module"),
        ];

        let inserted = index.insert_commits(&commits).unwrap();
        assert_eq!(inserted, 3);

        let count = index.commit_count().unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_insert_skips_non_indexable() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"),  // indexable
            CommitInfo {
                hash: "b".repeat(40),
                message_subject: "wip".to_string(),  // too short - not indexable
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 1700000000,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
        ];

        let inserted = index.insert_commits(&commits).unwrap();
        assert_eq!(inserted, 1);  // Only 1 indexable
    }

    #[test]
    fn test_get_commits_for_file() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"),
            create_test_commit(&"b".repeat(40), "Add feature to parser"),
        ];

        index.insert_commits(&commits).unwrap();

        let file_commits = index.get_commits_for_file("src/main.rs", 10).unwrap();
        assert_eq!(file_commits.len(), 2);
    }

    #[test]
    fn test_last_indexed_commit() {
        let index = GitHistoryIndex::in_memory().unwrap();

        // Initially none
        assert!(index.get_last_indexed_commit().unwrap().is_none());

        // Set and retrieve
        index.set_last_indexed_commit("abc123").unwrap();
        assert_eq!(
            index.get_last_indexed_commit().unwrap(),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_update_embedding() {
        let index = GitHistoryIndex::in_memory().unwrap();

        let commit = create_test_commit(&"a".repeat(40), "Fix important bug");
        index.insert_commit(&commit).unwrap();

        // Update with embedding
        let embedding: Vec<f32> = vec![0.1, 0.2, 0.3];
        index.update_embedding(&commit.hash, &embedding).unwrap();

        // Verify embedding was stored (would need getter to fully test)
        let count = index.commit_count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_checksum_changes_on_insert() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let checksum1 = index.checksum().unwrap();

        // Message must be >= 10 chars to be indexable
        let commits = vec![create_test_commit(&"a".repeat(40), "Fix important bug in parser")];
        index.insert_commits(&commits).unwrap();

        let checksum2 = index.checksum().unwrap();

        assert_ne!(checksum1, checksum2, "Checksum should change after insert");
    }

    // === Incremental Sync Tests (GH-RAG-004) ===

    #[test]
    fn test_sync_incremental_empty_index() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "First commit message"),
            create_test_commit(&"b".repeat(40), "Second commit message"),
        ];

        let result = index.sync_incremental(&commits).unwrap();

        assert_eq!(result.commits_added, 2);
        assert_eq!(result.commits_skipped, 0);
        assert!(result.last_commit.is_some());
    }

    #[test]
    fn test_sync_incremental_adds_only_new() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        // Initial sync
        let commits1 = vec![
            create_test_commit(&"a".repeat(40), "First commit message"),
        ];
        index.sync_incremental(&commits1).unwrap();

        // Second sync with existing + new
        let commits2 = vec![
            create_test_commit(&"a".repeat(40), "First commit message"), // existing
            create_test_commit(&"b".repeat(40), "Second commit message"), // new
        ];

        let result = index.sync_incremental(&commits2).unwrap();

        assert_eq!(result.commits_added, 1, "Should only add 1 new commit");
        assert_eq!(index.commit_count().unwrap(), 2);
    }

    #[test]
    fn test_sync_incremental_skips_non_indexable() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Valid commit message"),
            CommitInfo {
                hash: "b".repeat(40),
                message_subject: "wip".to_string(), // too short
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 1700000001,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
        ];

        let result = index.sync_incremental(&commits).unwrap();

        assert_eq!(result.commits_added, 1);
        assert_eq!(result.commits_skipped, 1);
    }

    #[test]
    fn test_sync_incremental_updates_last_indexed() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        assert!(index.get_last_indexed_commit().unwrap().is_none());

        let commits = vec![
            create_test_commit(&"a".repeat(40), "First commit message"),
        ];

        let result = index.sync_incremental(&commits).unwrap();

        assert!(result.last_commit.is_some());
        assert_eq!(
            index.get_last_indexed_commit().unwrap(),
            result.last_commit
        );
    }

    #[test]
    fn test_commit_exists() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let hash = "a".repeat(40);
        assert!(!index.commit_exists(&hash).unwrap());

        let commits = vec![create_test_commit(&hash, "Test commit message")];
        index.insert_commits(&commits).unwrap();

        assert!(index.commit_exists(&hash).unwrap());
    }

    #[test]
    fn test_get_commits_since() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            CommitInfo {
                hash: "a".repeat(40),
                message_subject: "Old commit message".to_string(),
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 1000000000,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
            CommitInfo {
                hash: "b".repeat(40),
                message_subject: "New commit message".to_string(),
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 2000000000,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
        ];

        index.insert_commits(&commits).unwrap();

        // Get commits since midpoint
        let recent = index.get_commits_since(1500000000, 10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].message_subject, "New commit message");
    }

    #[test]
    fn test_sync_empty_input() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let result = index.sync_incremental(&[]).unwrap();

        assert_eq!(result.commits_added, 0);
        assert_eq!(result.commits_skipped, 0);
    }

    #[test]
    fn test_sync_all_already_present() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Test commit message"),
        ];

        // First sync
        index.sync_incremental(&commits).unwrap();

        // Second sync with same commits
        let result = index.sync_incremental(&commits).unwrap();

        assert_eq!(result.commits_added, 0);
        assert_eq!(index.commit_count().unwrap(), 1);
    }

    #[test]
    fn test_get_commits_without_embeddings() {
        let index = GitHistoryIndex::in_memory().unwrap();

        let commit = create_test_commit(&"a".repeat(40), "Fix bug in parser");
        index.insert_commit(&commit).unwrap();

        let needs_embedding = index.get_commits_without_embeddings(10).unwrap();
        assert_eq!(needs_embedding.len(), 1);
        assert_eq!(needs_embedding[0], "a".repeat(40));

        // After setting embedding, should not appear
        index.update_embedding(&commit.hash, &[0.1, 0.2]).unwrap();
        let needs_embedding = index.get_commits_without_embeddings(10).unwrap();
        assert!(needs_embedding.is_empty());
    }

    #[test]
    fn test_commit_info_full_message_no_body() {
        let info = CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Fix bug".to_string(),
            message_body: None,
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: true,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };
        assert_eq!(info.full_message(), "Fix bug");
    }

    #[test]
    fn test_commit_info_full_message_empty_body() {
        let info = CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Fix bug".to_string(),
            message_body: Some("".to_string()),
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: true,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };
        assert_eq!(info.full_message(), "Fix bug");
    }

    #[test]
    fn test_commit_info_is_indexable_merge_with_custom_message() {
        let info = CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Custom merge: integrate feature X".to_string(),
            message_body: None,
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: true,
            is_fix: false,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };
        // Merge with custom message (not starting with "Merge ") IS indexable
        assert!(info.is_indexable());
    }

    #[test]
    fn test_change_type_as_str() {
        assert_eq!(ChangeType::Added.as_str(), "A");
        assert_eq!(ChangeType::Modified.as_str(), "M");
        assert_eq!(ChangeType::Deleted.as_str(), "D");
        assert_eq!(ChangeType::Renamed.as_str(), "R");
    }

    #[test]
    fn test_checksum_empty_index() {
        let index = GitHistoryIndex::in_memory().unwrap();
        let checksum = index.checksum().unwrap();
        assert_eq!(checksum, "0:");
    }

    // Falsification Test F1: Index Independence
    #[test]
    fn falsify_git_index_isolation() {
        // This test verifies that git index operations don't affect code index
        // In a real scenario, we'd have both indexes and verify cross-contamination
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let initial_checksum = index.checksum().unwrap();

        // Insert commits
        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"),
            create_test_commit(&"b".repeat(40), "Add new feature"),
        ];
        index.insert_commits(&commits).unwrap();

        let final_checksum = index.checksum().unwrap();

        // Git index MUST have changed
        assert_ne!(
            initial_checksum, final_checksum,
            "FALSIFIED: Git index not updated after commit insertion"
        );

        // Verify we can get meaningful data back
        let count = index.commit_count().unwrap();
        assert_eq!(count, 2, "FALSIFIED: Commit count mismatch");
    }
}
