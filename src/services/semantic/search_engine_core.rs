// SemanticSearchEngine implementation
// Extracted from search_engine.rs for modularity

impl SemanticSearchEngine {
    /// Create new search engine with local embeddings
    ///
    /// # Arguments
    /// * `db_path` - Path to vector database
    ///
    /// # Note
    /// This version uses pure Rust TF-IDF embeddings via aprender.
    /// No external API keys or internet connection required.
    pub async fn new(db_path: &str) -> Result<Self, String> {
        let vector_db = TursoVectorDB::new_local(db_path).await?;

        Ok(Self {
            vector_db: Arc::new(vector_db),
            embedder: Arc::new(RwLock::new(LocalEmbedder::new())),
        })
    }

    /// Create new search engine (backward compatible - ignores api_key)
    #[deprecated(note = "Use new() without api_key - local embeddings don't require API keys")]
    pub async fn new_with_key(_api_key: &str, db_path: &str) -> Result<Self, String> {
        Self::new(db_path).await
    }

    /// Search code by natural language query
    ///
    /// # Arguments
    /// * `query` - Search query
    ///
    /// # Returns
    /// Ranked search results
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, String> {
        if query.query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        match query.mode {
            SearchMode::SemanticOnly => self.semantic_search(query).await,
            SearchMode::KeywordOnly => self.keyword_search(query).await,
            SearchMode::Hybrid => self.hybrid_search(query).await,
        }
    }

    /// Semantic search using vector similarity with local TF-IDF embeddings
    async fn semantic_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, String> {
        // Generate embedding for query using local embedder
        let query_embedding = {
            let embedder = self
                .embedder
                .read()
                .map_err(|e| format!("Lock error: {e}"))?;
            embedder.embed(&query.query)?
        };

        // Search vector database
        let db_results = self
            .vector_db
            .similarity_search(&query_embedding, query.limit * 2)
            .await?;

        // Apply filters and convert to SearchResult
        let mut results: Vec<SearchResult> = db_results
            .into_iter()
            .filter(|r| {
                // Language filter
                if let Some(ref lang) = query.language_filter {
                    if &r.language != lang {
                        return false;
                    }
                }

                // File pattern filter
                if let Some(ref pattern) = query.file_pattern {
                    if !Self::matches_pattern(&r.file_path, pattern) {
                        return false;
                    }
                }

                // Chunk type filter
                if let Some(ref chunk_type) = query.chunk_type_filter {
                    let chunk_type_str = match chunk_type {
                        ChunkType::Function => "function",
                        ChunkType::Class => "class",
                        ChunkType::Module => "module",
                        ChunkType::File => "file",
                        ChunkType::Struct => "struct",
                        ChunkType::Enum => "enum",
                        ChunkType::Trait => "trait",
                        ChunkType::TypeAlias => "type",
                        ChunkType::Impl => "impl",
                    };
                    if r.chunk_type != chunk_type_str {
                        return false;
                    }
                }

                true
            })
            .map(|r| {
                let snippet = format!(
                    "{} {} ({}:{})",
                    r.chunk_type, r.chunk_name, r.start_line, r.end_line
                );

                SearchResult {
                    file_path: r.file_path,
                    chunk_name: r.chunk_name,
                    chunk_type: r.chunk_type,
                    language: r.language,
                    similarity_score: r.similarity,
                    snippet,
                    start_line: r.start_line,
                    end_line: r.end_line,
                }
            })
            .collect();

        results.truncate(query.limit);
        Ok(results)
    }

    /// Keyword-only search using simple text matching
    async fn keyword_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, String> {
        // For keyword search, we search all embeddings and filter by content match
        // This is a simple implementation - could be enhanced with proper full-text search
        let all_results = self
            .vector_db
            .similarity_search(&vec![0.0; 256], query.limit * 10)
            .await?;

        let keywords: Vec<&str> = query.query.split_whitespace().collect();

        let mut results: Vec<SearchResult> = all_results
            .into_iter()
            .filter(|r| {
                // Check if any keyword matches
                let searchable =
                    format!("{} {} {}", r.file_path, r.chunk_name, r.chunk_type).to_lowercase();
                keywords
                    .iter()
                    .any(|kw| searchable.contains(&kw.to_lowercase()))
            })
            .filter(|r| {
                if let Some(ref lang) = query.language_filter {
                    &r.language == lang
                } else {
                    true
                }
            })
            .map(|r| {
                let snippet = format!(
                    "{} {} ({}:{})",
                    r.chunk_type, r.chunk_name, r.start_line, r.end_line
                );

                SearchResult {
                    file_path: r.file_path,
                    chunk_name: r.chunk_name,
                    chunk_type: r.chunk_type,
                    language: r.language,
                    similarity_score: 1.0, // Keyword matches are binary
                    snippet,
                    start_line: r.start_line,
                    end_line: r.end_line,
                }
            })
            .collect();

        results.truncate(query.limit);
        Ok(results)
    }

    /// Hybrid search combining semantic and keyword matching
    async fn hybrid_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, String> {
        // Get results from both methods
        let semantic_results = self.semantic_search(query).await?;
        let keyword_results = self.keyword_search(query).await?;

        // Merge results using Reciprocal Rank Fusion (RRF)
        use std::collections::HashMap;
        let mut scores: HashMap<String, f64> = HashMap::new();
        let k = 60.0; // RRF constant

        for (rank, result) in semantic_results.iter().enumerate() {
            let key = format!("{}:{}", result.file_path, result.chunk_name);
            *scores.entry(key).or_default() += 1.0 / (k + rank as f64 + 1.0);
        }

        for (rank, result) in keyword_results.iter().enumerate() {
            let key = format!("{}:{}", result.file_path, result.chunk_name);
            *scores.entry(key).or_default() += 1.0 / (k + rank as f64 + 1.0);
        }

        // Combine and sort
        let mut all_results: Vec<SearchResult> = semantic_results
            .into_iter()
            .chain(keyword_results.into_iter())
            .collect();

        // Deduplicate
        let mut seen = std::collections::HashSet::new();
        all_results.retain(|r| {
            let key = format!("{}:{}", r.file_path, r.chunk_name);
            seen.insert(key)
        });

        // Sort by RRF score
        all_results.sort_by(|a, b| {
            let key_a = format!("{}:{}", a.file_path, a.chunk_name);
            let key_b = format!("{}:{}", b.file_path, b.chunk_name);
            let score_a = scores.get(&key_a).unwrap_or(&0.0);
            let score_b = scores.get(&key_b).unwrap_or(&0.0);
            score_b
                .partial_cmp(score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        all_results.truncate(query.limit);
        Ok(all_results)
    }

    /// Find code similar to a reference file
    pub async fn find_similar(
        &self,
        file_path: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let file_embeddings = self.vector_db.query_by_file(file_path).await?;

        if file_embeddings.is_empty() {
            return Err(format!("File not indexed: {file_path}"));
        }

        let reference_embedding = &file_embeddings[0].embedding;

        let results = self
            .vector_db
            .similarity_search(reference_embedding, limit)
            .await?;

        let search_results = results
            .into_iter()
            .map(|r| {
                let snippet = format!(
                    "{} {} ({}:{})",
                    r.chunk_type, r.chunk_name, r.start_line, r.end_line
                );

                SearchResult {
                    file_path: r.file_path,
                    chunk_name: r.chunk_name,
                    chunk_type: r.chunk_type,
                    language: r.language,
                    similarity_score: r.similarity,
                    snippet,
                    start_line: r.start_line,
                    end_line: r.end_line,
                }
            })
            .collect();

        Ok(search_results)
    }

    /// Index a directory using local TF-IDF embeddings
    pub async fn index_directory(&self, path: &Path) -> Result<IndexStats, String> {
        let start = Instant::now();
        let mut stats = IndexStats {
            total_files: 0,
            total_chunks: 0,
            created: 0,
            updated: 0,
            skipped: 0,
            duration_ms: 0,
        };

        // First pass: collect all code chunks to build TF-IDF vocabulary
        let mut all_chunks: Vec<(std::path::PathBuf, super::chunker::CodeChunk)> = Vec::new();
        let mut all_contents: Vec<String> = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            let language = match Self::detect_language(file_path) {
                Some(lang) => lang,
                None => continue,
            };

            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let chunks = match chunk_code(&content, language) {
                Ok(chunks) => chunks,
                Err(_) => continue,
            };

            if chunks.is_empty() {
                continue;
            }

            stats.total_files += 1;

            for chunk in chunks {
                all_contents.push(chunk.content.clone());
                all_chunks.push((file_path.to_path_buf(), chunk));
            }
        }

        stats.total_chunks = all_chunks.len();

        // Fit the TF-IDF vectorizer on all collected content
        if !all_contents.is_empty() {
            let embedder = self
                .embedder
                .read()
                .map_err(|e| format!("Lock error: {e}"))?;
            embedder.fit(&all_contents)?;
        }

        // Second pass: generate embeddings and store
        for (file_path, chunk) in all_chunks {
            // Check if chunk already exists
            let existing = self
                .vector_db
                .query_by_file(file_path.to_str().expect("internal error"))
                .await?;

            let should_skip = existing.iter().any(|e| {
                e.chunk_name == chunk.chunk_name
                    && e.file_path == file_path.to_str().expect("internal error")
            });

            if should_skip {
                stats.skipped += 1;
                continue;
            }

            // Generate embedding using local TF-IDF
            let embedding = {
                let embedder = self
                    .embedder
                    .read()
                    .map_err(|e| format!("Lock error: {e}"))?;
                embedder.embed(&chunk.content)?
            };

            // Store in database
            let entry = EmbeddingEntry {
                file_path: file_path.to_str().expect("internal error").to_string(),
                chunk_name: chunk.chunk_name,
                chunk_type: format!("{:?}", chunk.chunk_type).to_lowercase(),
                language: chunk.language,
                start_line: chunk.start_line,
                end_line: chunk.end_line,
                content_checksum: chunk.content_checksum,
                embedding,
                model: "aprender-tfidf-local".to_string(),
            };

            self.vector_db.insert(&entry).await?;
            stats.created += 1;
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;

        Ok(stats)
    }

    /// Get total embedding count
    pub async fn embedding_count(&self) -> Result<usize, String> {
        // Get dimension without holding lock across await
        let dim = {
            let embedder = self
                .embedder
                .read()
                .map_err(|e| format!("Lock error: {e}"))?;
            embedder.dimension()
        };
        let all = self
            .vector_db
            .similarity_search(&vec![0.0; dim], usize::MAX)
            .await?;
        Ok(all.len())
    }

    /// Detect programming language from file extension
    fn detect_language(path: &Path) -> Option<Language> {
        let extension = path.extension()?.to_str()?;

        match extension {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" => Some(Language::TypeScript),
            "py" => Some(Language::Python),
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
            "go" => Some(Language::Go),
            _ => None,
        }
    }

    /// Check if path matches pattern
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        if let Some(suffix) = pattern.strip_prefix('*') {
            path.ends_with(suffix)
        } else {
            path.contains(pattern)
        }
    }
}
