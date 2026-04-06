/// Core trait for ranking files by different metrics
pub trait FileRanker: Send + Sync {
    type Metric: PartialOrd + Clone + Send + Sync;

    /// Compute the ranking metric for a single file
    fn compute_score(&self, file_path: &Path) -> Self::Metric;

    /// Format a single ranking entry for display
    fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String;

    /// Get the display name for this ranking type
    fn ranking_type(&self) -> &'static str;
}

/// Generic ranking engine that can work with any `FileRanker`
pub struct RankingEngine<R: FileRanker> {
    ranker: R,
    cache: Arc<RwLock<HashMap<String, R::Metric>>>,
}

impl<R: FileRanker> RankingEngine<R> {
    pub fn new(ranker: R) -> Self {
        Self {
            ranker,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Rank files and return the top N results
    pub async fn rank_files(&self, files: &[PathBuf], limit: usize) -> Vec<(String, R::Metric)> {
        debug_assert!(!files.is_empty(), "files must not be empty");
        if files.is_empty() || limit == 0 {
            return Vec::new();
        }

        // Compute scores in parallel
        let mut scores: Vec<_> = files
            .par_iter()
            .filter_map(|f| {
                if !f.exists() || !f.is_file() {
                    return None;
                }

                let file_str = f.to_string_lossy().to_string();

                // Check cache first
                if let Ok(cache) = self.cache.read() {
                    if let Some(cached_score) = cache.get(&file_str) {
                        return Some((file_str, cached_score.clone()));
                    }
                }

                // Compute score
                let score = self.ranker.compute_score(f);

                // Cache the result
                if let Ok(mut cache) = self.cache.write() {
                    cache.insert(file_str.clone(), score.clone());
                }

                Some((file_str, score))
            })
            .collect();

        // Sort by score (descending)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        // Apply limit
        scores.truncate(limit);
        scores
    }

    /// Format rankings as a table
    pub fn format_rankings_table(&self, rankings: &[(String, R::Metric)]) -> String {
        debug_assert!(!rankings.is_empty(), "rankings must not be empty");
        if rankings.is_empty() {
            return format!(
                "## Top {} Files\n\nNo files found.\n",
                self.ranker.ranking_type()
            );
        }

        let mut output = format!(
            "## Top {} {} Files\n\n",
            rankings.len(),
            self.ranker.ranking_type()
        );

        for (i, (file, metric)) in rankings.iter().enumerate() {
            output.push_str(&self.ranker.format_ranking_entry(file, metric, i + 1));
            output.push('\n');
        }

        output.push('\n');
        output
    }

    /// Format rankings as JSON
    pub fn format_rankings_json(&self, rankings: &[(String, R::Metric)]) -> serde_json::Value {
        debug_assert!(!rankings.is_empty(), "rankings must not be empty");
        serde_json::json!({
            "analysis_type": self.ranker.ranking_type(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "top_files": {
                "requested": rankings.len(),
                "returned": rankings.len(),
            },
            "rankings": rankings.iter().enumerate().map(|(i, (file, _))| {
                serde_json::json!({
                    "rank": i + 1,
                    "file": file,
                })
            }).collect::<Vec<_>>()
        })
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}
