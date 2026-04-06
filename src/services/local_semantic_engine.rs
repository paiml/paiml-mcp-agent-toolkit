// local_semantic_engine.rs — impl LocalSemanticEngine (included by local_semantic.rs)
// NO `use` imports here — they live in the parent module.

impl Default for LocalSemanticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalSemanticEngine {
    /// Create a new local semantic engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            dtm: None,
            vocabulary: HashMap::new(),
            reverse_vocabulary: Vec::new(),
        }
    }

    /// Index a directory of source files
    ///
    /// # Arguments
    /// * `path` - Directory path to scan
    /// * `language_filter` - Optional language filter (e.g., "rust", "python")
    ///
    /// # Returns
    /// Number of documents indexed
    pub fn index_directory(
        &mut self,
        path: &Path,
        language_filter: Option<&str>,
    ) -> Result<usize, String> {
        self.documents.clear();

        for entry in WalkDir::new(path)
            .max_depth(10)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

            let language = match extension {
                "rs" => "rust",
                "py" => "python",
                "js" => "javascript",
                "ts" => "typescript",
                "go" => "go",
                "java" => "java",
                "c" | "h" => "c",
                "cpp" | "hpp" | "cc" | "cxx" | "cu" | "cuh" => "cpp",
                "rb" => "ruby",
                "php" => "php",
                "swift" => "swift",
                "kt" => "kotlin",
                _ => continue, // Skip non-code files
            };

            // Apply language filter if specified
            if let Some(filter) = language_filter {
                if language != filter {
                    continue;
                }
            }

            // Read file content
            if let Ok(content) = std::fs::read_to_string(file_path) {
                // Skip very large files (> 100KB) and very small files (< 50 bytes)
                if content.len() > 100_000 || content.len() < 50 {
                    continue;
                }

                self.documents.push(CodeDocument {
                    file_path: file_path.to_path_buf(),
                    content,
                    language: language.to_string(),
                });
            }
        }

        if self.documents.is_empty() {
            return Err("No source files found to analyze".to_string());
        }

        // Build TF-IDF matrix
        self.build_tfidf_matrix()?;

        Ok(self.documents.len())
    }

    /// Build TF-IDF matrix from documents
    fn build_tfidf_matrix(&mut self) -> Result<(), String> {
        if self.documents.is_empty() {
            return Err("No documents to analyze".to_string());
        }

        // Prepare document texts
        let texts: Vec<&str> = self.documents.iter().map(|d| d.content.as_str()).collect();

        // Create TF-IDF vectorizer with code-friendly settings
        let mut vectorizer = TfidfVectorizer::new()
            .with_tokenizer(Box::new(WhitespaceTokenizer::new()))
            .with_min_df(2) // Minimum document frequency
            .with_max_df(0.95) // Maximum document frequency (exclude very common terms)
            .with_max_features(1000); // Limit vocabulary size (usize, not Option)

        // Fit and transform - returns Matrix<f64>
        let matrix = vectorizer
            .fit_transform(&texts)
            .map_err(|e| format!("TF-IDF vectorization failed: {}", e))?;

        // Store the vocabulary
        self.vocabulary = vectorizer.vocabulary().clone();

        // Build reverse vocabulary (index -> word)
        self.reverse_vocabulary = vec![String::new(); self.vocabulary.len()];
        for (word, &idx) in &self.vocabulary {
            if idx < self.reverse_vocabulary.len() {
                self.reverse_vocabulary[idx] = word.clone();
            }
        }

        self.dtm = Some(matrix);

        Ok(())
    }

    /// Extract topics using LDA
    ///
    /// # Arguments
    /// * `num_topics` - Number of topics to extract (1-20)
    /// * `language_filter` - Optional language filter
    ///
    /// # Returns
    /// Topic extraction results
    pub fn extract_topics(
        &mut self,
        num_topics: usize,
        language_filter: Option<String>,
    ) -> Result<LocalTopicResult, String> {
        debug_assert!(num_topics > 0, "num_topics must be positive");
        if num_topics == 0 || num_topics > 20 {
            return Err("num_topics must be between 1 and 20".to_string());
        }

        // Re-index if language filter changed
        if language_filter.is_some() {
            let path = self
                .documents
                .first()
                .map(|d| d.file_path.parent().unwrap_or(Path::new(".")))
                .unwrap_or(Path::new("."))
                .to_path_buf();
            self.index_directory(&path, language_filter.as_deref())?;
        }

        let dtm = self
            .dtm
            .as_ref()
            .ok_or("No documents indexed. Call index_directory first.")?;

        if dtm.n_rows() < num_topics {
            return Err(format!(
                "Need at least {} documents for {} topics, but only {} indexed",
                num_topics,
                num_topics,
                dtm.n_rows()
            ));
        }

        // Run LDA
        let mut lda = LatentDirichletAllocation::new(num_topics).with_random_seed(42);

        lda.fit(dtm, 50) // 50 iterations
            .map_err(|e| format!("LDA failed: {}", e))?;

        // Extract top terms per topic
        let topic_word = lda
            .topic_words()
            .map_err(|e| format!("Failed to get topic-word distribution: {}", e))?;

        let mut topics = Vec::new();

        for topic_id in 0..num_topics {
            // Get word weights for this topic
            let mut term_weights: Vec<(usize, f64)> = (0..self.reverse_vocabulary.len())
                .map(|word_idx| {
                    let weight = topic_word.get(topic_id, word_idx);
                    (word_idx, weight)
                })
                .filter(|(_, w)| *w > 0.0)
                .collect();

            // Sort by weight descending
            term_weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Take top 10 terms
            let top_terms: Vec<(String, f64)> = term_weights
                .into_iter()
                .take(10)
                .filter_map(|(idx, weight)| {
                    self.reverse_vocabulary
                        .get(idx)
                        .filter(|s| !s.is_empty())
                        .map(|term| (term.clone(), weight))
                })
                .collect();

            // Count documents with high probability for this topic
            let doc_count = if let Ok(dt) = lda.document_topics() {
                (0..dt.n_rows())
                    .filter(|&doc_idx| {
                        let p = dt.get(doc_idx, topic_id);
                        p > 0.1
                    })
                    .count()
            } else {
                0
            };

            topics.push(LocalTopic {
                id: topic_id,
                top_terms,
                document_count: doc_count,
            });
        }

        Ok(LocalTopicResult {
            topics,
            num_documents: self.documents.len(),
        })
    }

    /// Cluster documents using specified method
    ///
    /// # Arguments
    /// * `method` - Clustering method: "kmeans", "hierarchical", or "dbscan"
    /// * `k` - Number of clusters (required for kmeans)
    ///
    /// # Returns
    /// Clustering results
    pub fn cluster(&self, method: &str, k: Option<usize>) -> Result<LocalClusterResult, String> {
        let dtm = self
            .dtm
            .as_ref()
            .ok_or("No documents indexed. Call index_directory first.")?;

        // Convert f64 matrix to f32 for clustering
        let n_rows = dtm.n_rows();
        let n_cols = dtm.n_cols();
        let data_f32: Vec<f32> = (0..n_rows * n_cols)
            .map(|i| {
                let row = i / n_cols;
                let col = i % n_cols;
                dtm.get(row, col) as f32
            })
            .collect();

        let matrix_f32 = Matrix::from_vec(n_rows, n_cols, data_f32)
            .map_err(|e| format!("Matrix conversion failed: {}", e))?;

        let labels: Vec<i32> = match method {
            "kmeans" => {
                let k_val = k.ok_or("K-means requires --k parameter")?;
                if k_val > n_rows {
                    return Err(format!(
                        "Cannot create {} clusters from {} documents",
                        k_val, n_rows
                    ));
                }
                let mut kmeans = KMeans::new(k_val).with_max_iter(100).with_random_state(42);
                kmeans
                    .fit(&matrix_f32)
                    .map_err(|e| format!("K-means failed: {}", e))?;
                kmeans
                    .predict(&matrix_f32)
                    .into_iter()
                    .map(|l| l as i32)
                    .collect()
            }
            "hierarchical" => {
                let n_clusters = k.unwrap_or(5.min(n_rows));
                let mut agg =
                    AgglomerativeClustering::new(n_clusters, aprender::cluster::Linkage::Average);
                agg.fit(&matrix_f32)
                    .map_err(|e| format!("Hierarchical clustering failed: {}", e))?;
                agg.labels().iter().map(|&l| l as i32).collect()
            }
            "dbscan" => {
                let mut dbscan = DBSCAN::new(0.5, 2);
                dbscan
                    .fit(&matrix_f32)
                    .map_err(|e| format!("DBSCAN failed: {}", e))?;
                dbscan.labels().clone()
            }
            _ => return Err(format!("Unknown clustering method: {}", method)),
        };

        // Group documents by cluster
        let mut cluster_map: HashMap<i32, Vec<PathBuf>> = HashMap::new();
        for (idx, &label) in labels.iter().enumerate() {
            if label >= 0 {
                // Skip noise points (label = -1 in DBSCAN)
                cluster_map
                    .entry(label)
                    .or_default()
                    .push(self.documents[idx].file_path.clone());
            }
        }

        let mut clusters: Vec<LocalCluster> = cluster_map
            .into_iter()
            .map(|(id, files)| LocalCluster {
                id: id as usize,
                size: files.len(),
                files,
            })
            .collect();

        clusters.sort_by_key(|c| std::cmp::Reverse(c.size));

        Ok(LocalClusterResult {
            clusters,
            method: method.to_string(),
            num_documents: self.documents.len(),
        })
    }
}
