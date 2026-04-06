// Search engine query methods - included by search_engine.rs
// Toyota Way: Jidoka - Automation with quality built-in

impl<'a> GitHistorySearchEngine<'a> {
    /// Create a new search engine
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(index: &'a GitHistoryIndex) -> Self {
        Self {
            index,
            embedder: CommitEmbedder::new(),
        }
    }

    /// Search git history for commits matching a query
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn search(
        &mut self,
        query: &str,
        options: GitSearchOptions,
    ) -> Result<Vec<GitSearchResult>, GitHistoryError> {
        let limit = if options.limit == 0 {
            10
        } else {
            options.limit
        };

        // Get candidate commits based on filters
        let candidates = self.get_candidates(&options)?;

        if candidates.is_empty() {
            return Ok(vec![]);
        }

        // Embed query and all candidate messages
        let messages: Vec<String> = candidates.iter().map(|c| c.full_message()).collect();

        // Include query in corpus for proper TF-IDF
        let mut corpus = messages.clone();
        corpus.push(query.to_string());

        let embeddings = self.embedder.embed_batch(&corpus);
        let query_embedding = embeddings.last().expect("embeddings must contain query");

        // Score candidates by similarity to query
        let mut scored: Vec<(usize, f32)> = embeddings[..candidates.len()]
            .iter()
            .enumerate()
            .map(|(i, emb)| (i, cosine_similarity(query_embedding, emb)))
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let results: Vec<GitSearchResult> = scored
            .into_iter()
            .take(limit)
            .filter(|(_, score)| *score > 0.0)
            .map(|(idx, score)| {
                let commit = candidates[idx].clone();
                let files = self.get_files_for_commit(&commit.hash).unwrap_or_default();
                GitSearchResult {
                    commit,
                    relevance_score: score,
                    files,
                }
            })
            .collect();

        Ok(results)
    }

    /// Search by file - find commits that touched a specific file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn search_by_file(
        &self,
        file_path: &str,
        limit: usize,
    ) -> Result<Vec<GitSearchResult>, GitHistoryError> {
        let commits = self.index.get_commits_for_file(file_path, limit)?;

        let results: Vec<GitSearchResult> = commits
            .into_iter()
            .filter_map(|hash| {
                self.get_commit_by_hash(&hash).ok().flatten().map(|commit| {
                    let files = self.get_files_for_commit(&hash).unwrap_or_default();
                    GitSearchResult {
                        commit,
                        relevance_score: 1.0, // File match is always relevant
                        files,
                    }
                })
            })
            .collect();

        Ok(results)
    }

    /// Get candidates based on filter options
    fn get_candidates(
        &self,
        options: &GitSearchOptions,
    ) -> Result<Vec<CommitInfo>, GitHistoryError> {
        let mut sql = String::from(
            r#"
            SELECT commit_hash, message_subject, message_body, author_name, author_email,
                   timestamp, is_merge, is_fix, is_feat, issue_refs
            FROM git_commits
            WHERE 1=1
            "#,
        );

        let mut conditions = Vec::new();

        if options.author_email.is_some() {
            conditions.push("author_email = ?");
        }
        if options.since_timestamp.is_some() {
            conditions.push("timestamp >= ?");
        }
        if options.until_timestamp.is_some() {
            conditions.push("timestamp <= ?");
        }
        if options.only_fixes {
            conditions.push("is_fix = 1");
        }
        if options.only_features {
            conditions.push("is_feat = 1");
        }

        for cond in &conditions {
            sql.push_str(" AND ");
            sql.push_str(cond);
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT 1000"); // Cap for performance

        // Build params dynamically — extract Options once to avoid unwrap in each branch
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&sql)?;

        let author = options.author_email.as_deref();
        let since = options.since_timestamp;
        let until = options.until_timestamp;

        let commits: Vec<CommitInfo> = match (author, since, until) {
            (Some(a), Some(s), Some(u)) => stmt.query_map(params![a, s, u], Self::row_to_commit),
            (Some(a), Some(s), None) => stmt.query_map(params![a, s], Self::row_to_commit),
            (Some(a), None, Some(u)) => stmt.query_map(params![a, u], Self::row_to_commit),
            (None, Some(s), Some(u)) => stmt.query_map(params![s, u], Self::row_to_commit),
            (Some(a), None, None) => stmt.query_map(params![a], Self::row_to_commit),
            (None, Some(s), None) => stmt.query_map(params![s], Self::row_to_commit),
            (None, None, Some(u)) => stmt.query_map(params![u], Self::row_to_commit),
            (None, None, None) => stmt.query_map([], Self::row_to_commit),
        }?
        .filter_map(|r| r.ok())
        .collect();

        Ok(commits)
    }

    fn row_to_commit(row: &rusqlite::Row<'_>) -> Result<CommitInfo, rusqlite::Error> {
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
            files: vec![],
        })
    }

    /// Get commit by hash
    fn get_commit_by_hash(&self, hash: &str) -> Result<Option<CommitInfo>, GitHistoryError> {
        let conn = self.get_connection()?;
        let result = conn.query_row(
            r#"
            SELECT commit_hash, message_subject, message_body, author_name, author_email,
                   timestamp, is_merge, is_fix, is_feat, issue_refs
            FROM git_commits
            WHERE commit_hash = ?1
            "#,
            [hash],
            Self::row_to_commit,
        );

        match result {
            Ok(commit) => Ok(Some(commit)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(GitHistoryError::Database(e)),
        }
    }

    /// Get files changed in a commit
    fn get_files_for_commit(&self, hash: &str) -> Result<Vec<String>, GitHistoryError> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT file_path FROM commit_files WHERE commit_hash = ?1")?;

        let files = stmt
            .query_map([hash], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(files)
    }

    /// Get connection reference (workaround for borrowing)
    fn get_connection(&self) -> Result<&rusqlite::Connection, GitHistoryError> {
        // Access the connection through the index
        // This requires making conn pub(crate) in GitHistoryIndex
        Ok(&self.index.conn)
    }
}
