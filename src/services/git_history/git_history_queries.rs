impl GitHistoryIndex {
    /// Get commits that need embeddings
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_commits_without_embeddings(
        &self,
        limit: usize,
    ) -> Result<Vec<String>, GitHistoryError> {
        debug_assert!(limit > 0, "limit must be positive");
        let mut stmt = self
            .conn
            .prepare("SELECT commit_hash FROM git_commits WHERE embedding IS NULL LIMIT ?1")?;

        let hashes = stmt
            .query_map([limit], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(hashes)
    }

    /// Get last indexed commit hash
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_last_indexed_commit(&self) -> Result<Option<String>, GitHistoryError> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM git_metadata WHERE key = 'last_indexed_commit'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    /// Set last indexed commit hash
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "non_empty_index")]
    pub fn set_last_indexed_commit(&self, commit_hash: &str) -> Result<(), GitHistoryError> {
        debug_assert!(!commit_hash.is_empty(), "commit_hash must not be empty");
        self.conn.execute(
            "INSERT OR REPLACE INTO git_metadata (key, value) VALUES ('last_indexed_commit', ?1)",
            [commit_hash],
        )?;
        Ok(())
    }

    /// Get total commit count
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn commit_count(&self) -> Result<usize, GitHistoryError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM git_commits", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get commits by file path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_commits_for_file(
        &self,
        file_path: &str,
        limit: usize,
    ) -> Result<Vec<String>, GitHistoryError> {
        debug_assert!(limit > 0, "limit must be positive");
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT gc.commit_hash
            FROM git_commits gc
            JOIN commit_files cf ON gc.commit_hash = cf.commit_hash
            WHERE cf.file_path = ?1
            ORDER BY gc.timestamp DESC
            LIMIT ?2
            "#,
        )?;

        let hashes = stmt
            .query_map(params![file_path, limit], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(hashes)
    }

    /// Check if a commit exists in the index
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn commit_exists(&self, commit_hash: &str) -> Result<bool, GitHistoryError> {
        debug_assert!(!commit_hash.is_empty(), "commit_hash must not be empty");
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM git_commits WHERE commit_hash = ?1",
            [commit_hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get commits by timestamp range (for incremental queries)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_commits_since(
        &self,
        timestamp: i64,
        limit: usize,
    ) -> Result<Vec<CommitInfo>, GitHistoryError> {
        debug_assert!(limit > 0, "limit must be positive");
        let mut stmt = self.conn.prepare(
            r#"
            SELECT commit_hash, message_subject, message_body, author_name, author_email,
                   timestamp, is_merge, is_fix, is_feat, issue_refs
            FROM git_commits
            WHERE timestamp > ?1
            ORDER BY timestamp ASC
            LIMIT ?2
            "#,
        )?;

        let commits = stmt
            .query_map(params![timestamp, limit], |row| {
                let issue_refs_str: String = row.get::<_, String>(9).unwrap_or_default();
                let issue_refs: Vec<String> =
                    serde_json::from_str(&issue_refs_str).unwrap_or_default();

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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn checksum(&self) -> Result<String, GitHistoryError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM git_commits", [], |row| row.get(0))?;

        let last_hash: Option<String> = self
            .conn
            .query_row(
                "SELECT commit_hash FROM git_commits ORDER BY timestamp DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(format!("{}:{}", count, last_hash.unwrap_or_default()))
    }
}
