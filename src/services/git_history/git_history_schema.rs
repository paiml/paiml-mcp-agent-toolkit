impl GitHistoryIndex {
    /// Create or open git history index at the given path
    pub fn open(path: &Path) -> Result<Self, GitHistoryError> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let conn = Connection::open(path)?;
        // Enable WAL mode for concurrent access (#161)
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;
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
}
