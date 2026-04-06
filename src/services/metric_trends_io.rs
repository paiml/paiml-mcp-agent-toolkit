fn urgency_recommendation(days: usize) -> String {
    if days <= 7 {
        "⚠️ URGENT: Threshold breach imminent - prioritize optimization".to_string()
    } else if days <= 14 {
        "⚠️ WARNING: Threshold breach in 2 weeks - schedule optimization".to_string()
    } else {
        format!("ℹ️ INFO: {days} days until breach - plan optimization")
    }
}

fn metric_specific_recommendations(metric: &str) -> Vec<String> {
    debug_assert!(!metric.is_empty(), "metric must not be empty");
    match metric {
        "lint" => vec![
            "Remove unused dependencies (saves ~2-3s)".into(),
            "Enable incremental clippy analysis".into(),
            "Review enabled clippy lints (disable pedantic if not needed)".into(),
            "Use cargo-cache to clean old artifacts".into(),
        ],
        "test-fast" => vec![
            "Parallelize test execution (use --test-threads)".into(),
            "Use #[ignore] for slow property tests".into(),
            "Implement test fixtures to reduce setup time".into(),
            "Profile tests to identify slowest ones".into(),
        ],
        "coverage" => vec![
            "Run coverage only in CI (skip locally)".into(),
            "Use --exclude for non-critical modules".into(),
            "Skip expensive property-based tests in coverage".into(),
            "Consider sampling coverage (not 100% runs)".into(),
        ],
        "build-release" => vec![
            "Enable LTO only in final release builds".into(),
            "Reduce codegen-units for faster linking".into(),
            "Use sccache with CARGO_INCREMENTAL=0 (incremental builds cannot be cached)".into(),
            "Use per-project target dirs (avoid shared CARGO_TARGET_DIR lock contention)".into(),
            "Review dependency tree for bloat".into(),
        ],
        _ => vec![
            format!("Review {metric} history for optimization opportunities"),
            "Profile to identify bottlenecks".into(),
        ],
    }
}

impl MetricTrendStore {
    /// Generate actionable recommendations (Phase 4)
    fn generate_recommendations(
        &self,
        metric: &str,
        breach_in_days: Option<usize>,
        _threshold: f64,
    ) -> Vec<String> {
        debug_assert!(!metric.is_empty(), "metric must not be empty");
        let days = match breach_in_days {
            Some(d) => d,
            None => return vec![
                "No threshold breach predicted in forecast period".to_string(),
                "Continue current practices".to_string(),
            ],
        };

        let mut recommendations = vec![urgency_recommendation(days)];
        recommendations.extend(metric_specific_recommendations(metric));
        recommendations
    }

    /// Persist cache to disk (JSON for simplicity, trueno-graph in Phase 3.1)
    fn persist(&self, metric: &str) -> Result<()> {
        debug_assert!(!metric.is_empty(), "metric must not be empty");
        if let Some(observations) = self.cache.get(metric) {
            let path = self.storage_path.join(format!("{}.json", metric));
            let json = serde_json::to_string_pretty(observations)?;
            std::fs::write(&path, json).context("Failed to write metric observations")?;
        }
        Ok(())
    }

    /// Load from disk
    fn load(&mut self, metric: &str) -> Result<()> {
        debug_assert!(!metric.is_empty(), "metric must not be empty");
        let path = self.storage_path.join(format!("{}.json", metric));
        if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            let observations: Vec<MetricObservation> = serde_json::from_str(&json)?;

            // Insert into cache FIRST (so add_to_graph can find previous observations)
            self.cache.insert(metric.to_string(), observations.clone());

            // Then add observations to graph for PageRank (in order)
            for (idx, obs) in observations.iter().enumerate() {
                // Check if this observation is already in the graph (prevent duplicates)
                if self.node_map.contains_key(&obs.timestamp) {
                    continue;
                }

                // Create node for this observation
                let node_id = NodeId(self.next_node_id);
                self.next_node_id += 1;

                // Store mapping
                self.node_map.insert(obs.timestamp, node_id);
                self.reverse_node_map.insert(node_id, obs.timestamp);

                // Create temporal edge from previous observation (if any)
                if idx > 0 {
                    let prev_obs = &observations[idx - 1];
                    if let Some(prev_node_id) = self.node_map.get(&prev_obs.timestamp) {
                        let delta_t = (obs.timestamp - prev_obs.timestamp) as f32;
                        self.graph.add_edge(*prev_node_id, node_id, delta_t)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Get all tracked metrics
    pub fn metrics(&mut self) -> Result<Vec<String>> {
        let mut metrics = Vec::new();
        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            if let Some(name) = entry.path().file_stem() {
                metrics.push(name.to_string_lossy().to_string());
            }
        }
        Ok(metrics)
    }
}
