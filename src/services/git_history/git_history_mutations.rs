impl GitHistoryIndex {
    /// Insert a commit into the index
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update_embedding(
        &self,
        commit_hash: &str,
        embedding: &[f32],
    ) -> Result<(), GitHistoryError> {
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

        self.conn.execute(
            "UPDATE git_commits SET embedding = ?1 WHERE commit_hash = ?2",
            params![embedding_bytes, commit_hash],
        )?;

        Ok(())
    }

    /// Sync new commits incrementally (GH-RAG-004)
    /// Toyota Way: Heijunka - Level loading without blocking
    ///
    /// Returns number of new commits synced
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn sync_incremental(
        &mut self,
        new_commits: &[CommitInfo],
    ) -> Result<SyncResult, GitHistoryError> {
        let last_indexed = self.get_last_indexed_commit()?;
        let start_count = self.commit_count()?;

        // Filter to only commits newer than last indexed
        let commits_to_add: Vec<&CommitInfo> = if let Some(ref last_hash) = last_indexed {
            // Find commits not already in index
            new_commits
                .iter()
                .filter(|c| c.hash != *last_hash && !self.commit_exists(&c.hash).unwrap_or(false))
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
}
