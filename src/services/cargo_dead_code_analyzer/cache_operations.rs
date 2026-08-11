// Cache operations for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

impl CargoDeadCodeAnalyzer {
    /// Get the cache file path
    ///
    /// The key has to cover every input that changes the ANSWER, not just the
    /// tree hash and the pmat version: two runs that differ only in
    /// `--include-tests` (or the traversal depth) analyse different file sets,
    /// and they used to share one entry, so the second run replayed the first
    /// and the flag looked like a no-op even after the walk started honouring
    /// it.
    fn cache_path(&self) -> PathBuf {
        let included: String = [
            (self.exclude_tests, 't'),
            (self.exclude_examples, 'e'),
            (self.exclude_benches, 'b'),
        ]
        .iter()
        .filter(|(excluded, _)| !excluded)
        .map(|(_, tag)| *tag)
        .collect();
        let scope = if included.is_empty() {
            "default".to_string()
        } else {
            included
        };
        self.project_path.join(".pmat").join(format!(
            "dead-code-cache-{scope}-d{}.json",
            self.max_depth
        ))
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

#[cfg(test)]
mod cache_key_tests {
    use super::*;

    /// Toggling a flag that changes WHICH files are analysed must not hit the
    /// entry written by the other configuration — with a shared key, the second
    /// `analyze dead-code` run replayed the first and `--include-tests` looked
    /// like a no-op even on a correct walk.
    #[test]
    fn test_cache_key_separates_include_tests_from_the_default() {
        let root = std::path::Path::new("/p");
        let default_path = CargoDeadCodeAnalyzer::new(root).cache_path();
        let with_tests = CargoDeadCodeAnalyzer::new(root).include_tests().cache_path();

        // `include_examples()` no longer separates keys, because examples and
        // benches are in scope by default — it re-asserts the default rather
        // than widening the walk, so there is no second file set to key apart.
        assert_ne!(default_path, with_tests);
        assert!(default_path.starts_with("/p/.pmat"), "{default_path:?}");
    }

    /// Depth changes the walk, so it changes the answer too.
    #[test]
    fn test_cache_key_separates_traversal_depths() {
        let root = std::path::Path::new("/p");
        assert_ne!(
            CargoDeadCodeAnalyzer::new(root).with_max_depth(2).cache_path(),
            CargoDeadCodeAnalyzer::new(root).with_max_depth(8).cache_path()
        );
    }
}
