// Cache operations for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

impl CargoDeadCodeAnalyzer {
    /// Get the cache file path
    fn cache_path(&self) -> PathBuf {
        self.project_path.join(".pmat").join("dead-code-cache.json")
    }

    /// Get current git tree hash for cache invalidation
    fn get_tree_hash(&self) -> Option<String> {
        let output = Command::new("git")
            .current_dir(&self.project_path)
            .args(["rev-parse", "HEAD:"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Try to load cached result if valid
    fn try_load_cache(&self) -> Option<AccurateDeadCodeReport> {
        if !self.use_cache || self.force_refresh {
            return None;
        }

        let cache_path = self.cache_path();
        let cache_content = std::fs::read_to_string(&cache_path).ok()?;
        let cached: CachedDeadCodeResult = serde_json::from_str(&cache_content).ok()?;

        // Validate cache
        let current_tree_hash = self.get_tree_hash()?;
        let current_version = env!("CARGO_PKG_VERSION");

        if cached.tree_hash == current_tree_hash && cached.pmat_version == current_version {
            tracing::debug!("Dead code cache hit (tree_hash: {})", current_tree_hash);
            Some(cached.report)
        } else {
            tracing::debug!(
                "Dead code cache miss (tree: {} vs {}, version: {} vs {})",
                cached.tree_hash,
                current_tree_hash,
                cached.pmat_version,
                current_version
            );
            None
        }
    }

    /// Save result to cache
    fn save_cache(&self, report: &AccurateDeadCodeReport) {
        if !self.use_cache {
            return;
        }

        let Some(tree_hash) = self.get_tree_hash() else {
            return;
        };

        let cached = CachedDeadCodeResult {
            tree_hash,
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now(),
            report: report.clone(),
        };

        // Ensure .pmat directory exists
        let cache_dir = self.project_path.join(".pmat");
        let _ = std::fs::create_dir_all(&cache_dir);

        // Write cache file
        if let Ok(content) = serde_json::to_string_pretty(&cached) {
            let _ = std::fs::write(self.cache_path(), content);
            tracing::debug!("Dead code cache saved");
        }
    }
}
