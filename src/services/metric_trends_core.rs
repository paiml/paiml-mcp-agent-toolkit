impl MetricTrendStore {
    /// Create new trend store
    pub fn new() -> Result<Self> {
        let storage_path = PathBuf::from(".pmat-metrics/trends");
        std::fs::create_dir_all(&storage_path).context("Failed to create trends directory")?;

        Ok(Self {
            storage_path,
            cache: HashMap::new(),
            graph: CsrGraph::new(),
            node_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            hotness_cache: HashMap::new(),
            next_node_id: 0,
        })
    }

    /// Load from custom path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let storage_path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&storage_path).context("Failed to create trends directory")?;

        Ok(Self {
            storage_path,
            cache: HashMap::new(),
            graph: CsrGraph::new(),
            node_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            hotness_cache: HashMap::new(),
            next_node_id: 0,
        })
    }

    /// Record new metric observation
    pub fn record(&mut self, metric: &str, value: f64, timestamp: i64) -> Result<()> {
        let obs = MetricObservation {
            metric: metric.to_string(),
            value,
            timestamp,
        };

        // Add to cache
        self.cache
            .entry(metric.to_string())
            .or_default()
            .push(obs.clone());

        // Phase 3.2: Add to CSR graph
        self.add_to_graph(&obs)?;

        // Persist to JSON (dual-write mode)
        self.persist(metric)?;

        Ok(())
    }

    /// Add observation to CSR graph (Phase 3.2)
    fn add_to_graph(&mut self, obs: &MetricObservation) -> Result<()> {
        // Check if this observation is already in the graph (prevent duplicates)
        if self.node_map.contains_key(&obs.timestamp) {
            return Ok(()); // Already added
        }

        // Create node for this observation
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        // Store mapping
        self.node_map.insert(obs.timestamp, node_id);
        self.reverse_node_map.insert(node_id, obs.timestamp);

        // Find previous observation for this metric (to create temporal edge)
        if let Some(observations) = self.cache.get(&obs.metric) {
            if observations.len() > 1 {
                // Get second-to-last observation (before we just pushed the new one)
                let prev_obs = &observations[observations.len() - 2];

                if let Some(prev_node_id) = self.node_map.get(&prev_obs.timestamp) {
                    // Create temporal edge: prev → current
                    // Weight = Δt (time between measurements in seconds)
                    let delta_t = (obs.timestamp - prev_obs.timestamp) as f32;
                    self.graph.add_edge(*prev_node_id, node_id, delta_t)?;
                }
            }
        }

        Ok(())
    }

    /// Get trend analysis for metric (last N days)
    pub fn trend(&mut self, metric: &str, days: usize) -> Result<TrendAnalysis> {
        // Load from disk if not cached
        if !self.cache.contains_key(metric) {
            self.load(metric)?;
        }

        let observations = self.cache.get(metric).context("Metric not found")?;

        // Filter to last N days
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - (days as i64 * 86400);
        let filtered: Vec<_> = observations
            .iter()
            .filter(|obs| obs.timestamp >= cutoff)
            .cloned()
            .collect();

        if filtered.is_empty() {
            anyhow::bail!("No observations in last {} days", days);
        }

        // Compute statistics
        let values: Vec<f64> = filtered.iter().map(|obs| obs.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Linear regression (simple slope calculation)
        let (slope, p_value) = self.compute_trend(&filtered);

        // Determine direction (p < 0.05 for significance)
        let direction = if p_value > 0.05 {
            TrendDirection::Stable
        } else if slope < 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Regressing
        };

        Ok(TrendAnalysis {
            metric: metric.to_string(),
            count: filtered.len(),
            mean,
            std_dev,
            min,
            max,
            direction,
            slope,
            p_value,
        })
    }

    /// Compute linear regression trend
    fn compute_trend(&self, observations: &[MetricObservation]) -> (f64, f64) {
        if observations.len() < 2 {
            return (0.0, 1.0); // Not enough data
        }

        // Normalize timestamps to days since first observation
        let first_ts = observations[0].timestamp;
        let xs: Vec<f64> = observations
            .iter()
            .map(|obs| (obs.timestamp - first_ts) as f64 / 86400.0)
            .collect();
        let ys: Vec<f64> = observations.iter().map(|obs| obs.value).collect();

        // Simple linear regression
        let n = xs.len() as f64;
        let mean_x = xs.iter().sum::<f64>() / n;
        let mean_y = ys.iter().sum::<f64>() / n;

        let cov = xs
            .iter()
            .zip(&ys)
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>();
        let var_x = xs.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>();

        let slope = if var_x > 0.0 { cov / var_x } else { 0.0 };

        // Compute p-value (t-test for slope)
        let residuals: Vec<f64> = xs
            .iter()
            .zip(&ys)
            .map(|(x, y)| y - (slope * x + mean_y - slope * mean_x))
            .collect();
        let sse = residuals.iter().map(|r| r.powi(2)).sum::<f64>();
        let mse = sse / (n - 2.0).max(1.0);
        let se_slope = (mse / var_x).sqrt();
        let t_stat = slope / se_slope;

        // Rough p-value (two-tailed t-test, approximation)
        let p_value = if t_stat.abs() > 2.0 {
            0.01 // Significant
        } else if t_stat.abs() > 1.5 {
            0.05
        } else {
            0.5 // Not significant
        };

        (slope, p_value)
    }

}
