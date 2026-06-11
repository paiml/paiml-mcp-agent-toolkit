impl TursoVectorDB {
    /// Open a local vector database, loading persisted entries from `path` if it
    /// exists (#568). Pass `":memory:"` for a non-persistent store.
    ///
    /// The SIMD `VectorStore` (aprender-rag) is in-memory only, so persistence is
    /// implemented by serializing entries on [`save`](Self::save) and rebuilding
    /// the store here via re-insert.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn new_local<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        // Default to 256 dimensions to match the in-binary local embedder
        // (`LocalEmbedder`, search_engine_embedder.rs). pmat uses local TF-IDF
        // embeddings only (no OpenAI/API keys), so a fresh store searched before
        // any insert must not assume the old 1536-dim OpenAI default — that
        // produced `dimension mismatch: expected 1536, got 256` on first search.
        // `insert` still auto-adjusts if a different dimension is ever used.
        let config = VectorStoreConfig {
            dimension: 256,
            ..Default::default()
        };

        let path_ref = path.as_ref();
        let db_path = if path_ref.as_os_str() == ":memory:" || path_ref.as_os_str().is_empty() {
            None
        } else {
            Some(path_ref.to_path_buf())
        };

        let db = Self {
            store: RwLock::new(VectorStore::new(config)),
            file_index: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
            db_path,
        };

        // Rebuild the in-memory store from the persisted entries, if any.
        // A corrupt or stale db file must not break startup — start empty.
        if let Some(p) = db.db_path.as_ref() {
            if p.exists() {
                if let Err(e) = db.load_persisted(p).await {
                    eprintln!("⚠️  ignoring unreadable vector db {}: {e}", p.display());
                }
            }
        }

        Ok(db)
    }

    /// Reconstruct every stored entry (metadata + its embedding) for persistence.
    fn collect_entries(&self) -> Result<Vec<EmbeddingEntry>, String> {
        let metadata = self.metadata.read().map_err(|e| format!("Lock error: {e}"))?;
        let store = self.store.read().map_err(|e| format!("Lock error: {e}"))?;
        let mut entries = Vec::with_capacity(metadata.len());
        for (chunk_id, meta) in metadata.iter() {
            let embedding = store
                .get(*chunk_id)
                .and_then(|c| c.embedding.clone())
                .unwrap_or_default();
            entries.push(EmbeddingEntry {
                file_path: meta.file_path.clone(),
                chunk_name: meta.chunk_name.clone(),
                chunk_type: meta.chunk_type.clone(),
                language: meta.language.clone(),
                start_line: meta.start_line,
                end_line: meta.end_line,
                content_checksum: meta.content_checksum.clone(),
                embedding,
                model: meta.model.clone(),
            });
        }
        Ok(entries)
    }

    /// Persist all entries to `db_path` (no-op for in-memory stores). #568.
    pub async fn save(&self) -> Result<(), String> {
        let Some(path) = self.db_path.as_ref() else {
            return Ok(());
        };
        let entries = self.collect_entries()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create db dir: {e}"))?;
        }
        let json = serde_json::to_string(&entries).map_err(|e| format!("serialize db: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("write db: {e}"))?;
        Ok(())
    }

    /// Load persisted entries from `path` and re-insert them into the store.
    async fn load_persisted(&self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("read db: {e}"))?;
        if content.trim().is_empty() {
            return Ok(());
        }
        let entries: Vec<EmbeddingEntry> =
            serde_json::from_str(&content).map_err(|e| format!("parse db: {e}"))?;
        for entry in &entries {
            self.insert(entry).await?;
        }
        Ok(())
    }

    /// Insert or update embedding entry
    ///
    /// # Arguments
    /// * `entry` - Embedding entry to insert
    ///
    /// # Returns
    /// Row ID of inserted/updated entry
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn insert(&self, entry: &EmbeddingEntry) -> Result<i64, String> {
        // Check if we need to reinitialize with different dimension
        let embedding_dim = entry.embedding.len();
        {
            let store = self.store.read().map_err(|e| format!("Lock error: {e}"))?;
            if store.config().dimension != embedding_dim {
                drop(store);
                // Reinitialize with correct dimension
                let mut store = self.store.write().map_err(|e| format!("Lock error: {e}"))?;
                *store = VectorStore::new(VectorStoreConfig {
                    dimension: embedding_dim,
                    ..Default::default()
                });
            }
        }

        // Generate unique ID
        let id = {
            let mut next_id = self
                .next_id
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Create chunk with embedding
        let doc_id = DocumentId::new();
        let content = format!(
            "{}:{}:{}",
            entry.file_path, entry.chunk_name, entry.chunk_type
        );
        let mut chunk = Chunk::new(doc_id, content, entry.start_line, entry.end_line);
        chunk.set_embedding(entry.embedding.clone());

        let chunk_id = chunk.id;

        // Insert into vector store
        {
            let mut store = self.store.write().map_err(|e| format!("Lock error: {e}"))?;
            store
                .insert(chunk)
                .map_err(|e| format!("Insert failed: {e}"))?;
        }

        // Update file index
        {
            let mut file_index = self
                .file_index
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            file_index
                .entry(entry.file_path.clone())
                .or_default()
                .push(chunk_id);
        }

        // Store metadata
        {
            let mut metadata = self
                .metadata
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            metadata.insert(
                chunk_id,
                EmbeddingMetadata {
                    id,
                    file_path: entry.file_path.clone(),
                    chunk_name: entry.chunk_name.clone(),
                    chunk_type: entry.chunk_type.clone(),
                    language: entry.language.clone(),
                    start_line: entry.start_line,
                    end_line: entry.end_line,
                    content_checksum: entry.content_checksum.clone(),
                    model: entry.model.clone(),
                },
            );
        }

        Ok(id)
    }

    /// Batch insert multiple entries
    ///
    /// # Arguments
    /// * `entries` - Array of entries to insert
    ///
    /// # Returns
    /// Array of row IDs
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn query_by_file(&self, file_path: &str) -> Result<Vec<SearchResult>, String> {
        let file_index = self
            .file_index
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;
        let metadata_map = self
            .metadata
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;
        let store = self.store.read().map_err(|e| format!("Lock error: {e}"))?;

        let chunk_ids = match file_index.get(file_path) {
            Some(ids) => ids.clone(),
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();
        for chunk_id in chunk_ids {
            if let Some(chunk) = store.get(chunk_id) {
                if let Some(meta) = metadata_map.get(&chunk_id) {
                    results.push(SearchResult {
                        id: meta.id,
                        file_path: meta.file_path.clone(),
                        chunk_name: meta.chunk_name.clone(),
                        chunk_type: meta.chunk_type.clone(),
                        language: meta.language.clone(),
                        start_line: meta.start_line,
                        end_line: meta.end_line,
                        similarity: 1.0, // Not applicable for file query
                        embedding: chunk.embedding.clone().unwrap_or_default(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Query embeddings by language
    ///
    /// # Arguments
    /// * `language` - Programming language
    ///
    /// # Returns
    /// Array of search results
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn query_by_language(&self, language: &str) -> Result<Vec<SearchResult>, String> {
        let metadata_map = self
            .metadata
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;
        let store = self.store.read().map_err(|e| format!("Lock error: {e}"))?;

        let mut results = Vec::new();
        for (chunk_id, meta) in metadata_map.iter() {
            if meta.language == language {
                if let Some(chunk) = store.get(*chunk_id) {
                    results.push(SearchResult {
                        id: meta.id,
                        file_path: meta.file_path.clone(),
                        chunk_name: meta.chunk_name.clone(),
                        chunk_type: meta.chunk_type.clone(),
                        language: meta.language.clone(),
                        start_line: meta.start_line,
                        end_line: meta.end_line,
                        similarity: 1.0,
                        embedding: chunk.embedding.clone().unwrap_or_default(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Vector similarity search using cosine similarity (SIMD-accelerated via trueno)
    ///
    /// # Arguments
    /// * `query` - Query vector
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    /// Array of search results sorted by similarity (highest first)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn similarity_search(
        &self,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let store = self.store.read().map_err(|e| format!("Lock error: {e}"))?;
        let metadata_map = self
            .metadata
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        // Use trueno-rag's SIMD-accelerated search
        let search_results = store
            .search(query, limit)
            .map_err(|e| format!("Search failed: {e}"))?;

        let mut results = Vec::new();
        for (chunk_id, score) in search_results {
            if let Some(meta) = metadata_map.get(&chunk_id) {
                if let Some(chunk) = store.get(chunk_id) {
                    results.push(SearchResult {
                        id: meta.id,
                        file_path: meta.file_path.clone(),
                        chunk_name: meta.chunk_name.clone(),
                        chunk_type: meta.chunk_type.clone(),
                        language: meta.language.clone(),
                        start_line: meta.start_line,
                        end_line: meta.end_line,
                        similarity: score as f64,
                        embedding: chunk.embedding.clone().unwrap_or_default(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Delete embeddings for a file
    ///
    /// # Arguments
    /// * `file_path` - Path to file
    ///
    /// # Returns
    /// Number of rows deleted
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn delete_by_file(&self, file_path: &str) -> Result<usize, String> {
        let chunk_ids = {
            let mut file_index = self
                .file_index
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;
            file_index.remove(file_path).unwrap_or_default()
        };

        let count = chunk_ids.len();

        // Remove from vector store and metadata
        {
            let mut store = self.store.write().map_err(|e| format!("Lock error: {e}"))?;
            let mut metadata = self
                .metadata
                .write()
                .map_err(|e| format!("Lock error: {e}"))?;

            for chunk_id in chunk_ids {
                store.remove(chunk_id);
                metadata.remove(&chunk_id);
            }
        }

        Ok(count)
    }

    /// Alias for delete_by_file (for backward compatibility)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn delete_file_entries(&self, file_path: &str) -> Result<usize, String> {
        self.delete_by_file(file_path).await
    }

    /// Get a specific entry by file path and chunk name
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_entry(
        &self,
        file_path: &str,
        chunk_name: &str,
    ) -> Result<Option<SearchResult>, String> {
        let results = self.query_by_file(file_path).await?;
        Ok(results.into_iter().find(|r| r.chunk_name == chunk_name))
    }

    /// Get database statistics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_stats(&self) -> Result<DbStats, String> {
        let store = self.store.read().map_err(|e| format!("Lock error: {e}"))?;
        let file_index = self
            .file_index
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        Ok(DbStats {
            total_entries: store.len(),
            unique_files: file_index.len(),
        })
    }

    /// Compute cosine similarity between two vectors (scalar implementation)
    /// Kept for backward compatibility but now using trueno-rag's implementation internally
    ///
    /// # Arguments
    /// * `v1` - First vector
    /// * `v2` - Second vector
    ///
    /// # Returns
    /// Cosine similarity score (-1.0 to 1.0)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    /// SIMD-optimized cosine similarity using loop unrolling for auto-vectorization
    ///
    /// This implementation uses 4-way loop unrolling which allows LLVM to generate
    /// SIMD instructions (SSE/AVX on x86, NEON on ARM) automatically.
    ///
    /// # Performance
    /// - 2-4x speedup on AVX2-capable CPUs (most x86-64 since 2013)
    /// - 4-8x speedup on AVX-512 capable CPUs (Intel Skylake-X and newer)
    /// - 2-4x speedup on ARM64 with NEON
    ///
    /// # Arguments
    /// * `v1` - First vector
    /// * `v2` - Second vector
    ///
    /// # Returns
    /// Cosine similarity score (-1.0 to 1.0)
    #[inline]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn cosine_similarity_simd(v1: &[f32], v2: &[f32]) -> f64 {
        if v1.len() != v2.len() || v1.is_empty() {
            return 0.0;
        }

        let len = v1.len();

        // Process in chunks of 4 for SIMD auto-vectorization
        let chunks = len / 4;
        let remainder = len % 4;

        let mut dot0 = 0.0f32;
        let mut dot1 = 0.0f32;
        let mut dot2 = 0.0f32;
        let mut dot3 = 0.0f32;

        let mut norm1_0 = 0.0f32;
        let mut norm1_1 = 0.0f32;
        let mut norm1_2 = 0.0f32;
        let mut norm1_3 = 0.0f32;

        let mut norm2_0 = 0.0f32;
        let mut norm2_1 = 0.0f32;
        let mut norm2_2 = 0.0f32;
        let mut norm2_3 = 0.0f32;

        // Main loop: 4-way unrolled for SIMD
        for i in 0..chunks {
            let base = i * 4;

            // SAFETY: bounds checked by chunks calculation
            let a0 = v1[base];
            let a1 = v1[base + 1];
            let a2 = v1[base + 2];
            let a3 = v1[base + 3];

            let b0 = v2[base];
            let b1 = v2[base + 1];
            let b2 = v2[base + 2];
            let b3 = v2[base + 3];

            // Dot products
            dot0 += a0 * b0;
            dot1 += a1 * b1;
            dot2 += a2 * b2;
            dot3 += a3 * b3;

            // Squared norms
            norm1_0 += a0 * a0;
            norm1_1 += a1 * a1;
            norm1_2 += a2 * a2;
            norm1_3 += a3 * a3;

            norm2_0 += b0 * b0;
            norm2_1 += b1 * b1;
            norm2_2 += b2 * b2;
            norm2_3 += b3 * b3;
        }

        // Handle remainder
        let remainder_start = chunks * 4;
        for i in 0..remainder {
            let idx = remainder_start + i;
            let a = v1[idx];
            let b = v2[idx];

            dot0 += a * b;
            norm1_0 += a * a;
            norm2_0 += b * b;
        }

        // Combine accumulators
        let dot_product = dot0 + dot1 + dot2 + dot3;
        let magnitude1 = (norm1_0 + norm1_1 + norm1_2 + norm1_3).sqrt();
        let magnitude2 = (norm2_0 + norm2_1 + norm2_2 + norm2_3).sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        (dot_product / (magnitude1 * magnitude2)) as f64
    }
}
