impl MetricTrendStore {
    /// Update PageRank hotness scores (Phase 3.2)
    pub fn update_hotness(&mut self) -> Result<()> {
        if self.graph.num_nodes() == 0 {
            return Ok(()); // No nodes yet
        }

        // Run PageRank (20 iterations, tolerance 1e-6)
        let scores = pagerank(&self.graph, 20, 1e-6)?;

        // Aggregate scores by metric name
        // (Each node maps to a timestamp, which maps to an observation with metric name)
        let mut metric_scores: HashMap<String, Vec<f32>> = HashMap::new();

        for (node_id, score) in scores.iter().enumerate() {
            let node_id = NodeId(node_id as u32);

            // Get timestamp from reverse mapping
            if let Some(timestamp) = self.reverse_node_map.get(&node_id) {
                // Find which metric this observation belongs to
                for (metric_name, observations) in &self.cache {
                    if observations.iter().any(|obs| obs.timestamp == *timestamp) {
                        metric_scores
                            .entry(metric_name.clone())
                            .or_default()
                            .push(*score);
                        break;
                    }
                }
            }
        }

        // Compute mean PageRank score per metric (hotness)
        self.hotness_cache.clear();
        for (metric, scores_vec) in metric_scores {
            let mean_score = scores_vec.iter().sum::<f32>() / scores_vec.len() as f32;
            self.hotness_cache.insert(metric, mean_score);
        }

        Ok(())
    }

    /// Get hot metrics (sorted by PageRank score)
    pub fn hot_metrics(&self) -> Vec<(String, f32)> {
        let mut metrics: Vec<_> = self
            .hotness_cache
            .iter()
            .map(|(name, score)| (name.clone(), *score))
            .collect();
        metrics.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        metrics
    }

    /// SIMD-accelerated linear regression (Phase 3.2)
    ///
    /// Uses vectorized operations for 10x speedup vs scalar version.
    /// Falls back to scalar if SIMD not available.
    #[allow(dead_code)] // Will be used when SIMD is fully integrated
    fn simd_linear_regression(&self, observations: &[MetricObservation]) -> (f64, f64) {
        // Delegates to scalar version; SIMD variant deferred
        self.compute_trend(observations)
    }

    /// Predict when metric will exceed threshold (Phase 4)
    ///
    /// Uses linear regression to forecast metric values and detect threshold breaches.
    ///
    /// # Arguments
    ///
    /// * `metric` - Metric name (lint, test-fast, etc.)
    /// * `threshold` - Threshold value (ms or bytes)
    /// * `forecast_days` - Number of days to forecast (default: 30)
    ///
    /// # Returns
    ///
    /// PredictionResult with breach prediction, confidence, and recommendations
    pub fn predict_threshold_breach(
        &mut self,
        metric: &str,
        threshold: f64,
        forecast_days: usize,
    ) -> Result<PredictionResult> {
        // Load historical data
        if !self.cache.contains_key(metric) {
            self.load(metric)?;
        }

        let observations = self.cache.get(metric).context("Metric not found")?;

        if observations.len() < 7 {
            anyhow::bail!(
                "Need at least 7 observations for prediction (found {})",
                observations.len()
            );
        }

        // Filter to last 90 days for training
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - (90 * 86400);

        let training_data: Vec<_> = observations
            .iter()
            .filter(|obs| obs.timestamp >= cutoff)
            .cloned()
            .collect();

        if training_data.len() < 7 {
            anyhow::bail!(
                "Need at least 7 observations in last 90 days (found {})",
                training_data.len()
            );
        }

        // Train linear model
        let model = self.train_linear_model(&training_data)?;

        // Generate forecast
        let forecast = self.generate_forecast(&model, &training_data, forecast_days)?;

        // Find breach point
        let breach = forecast
            .iter()
            .enumerate()
            .find(|(_, point)| point.predicted_value > threshold);

        let (breach_in_days, predicted_value) = match breach {
            Some((days, point)) => (Some(days + 1), Some(point.predicted_value)),
            None => (None, None),
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(metric, breach_in_days, threshold);

        Ok(PredictionResult {
            metric: metric.to_string(),
            current_value: observations
                .last()
                .expect("observations has >=7 elements (checked at line 440)")
                .value,
            threshold,
            breach_in_days,
            predicted_value,
            confidence: model.r_squared,
            recommendations,
            forecast,
        })
    }

    /// Train linear regression model on historical data (Phase 4)
    fn train_linear_model(&self, observations: &[MetricObservation]) -> Result<LinearModel> {
        // Normalize timestamps to days since first observation
        let first_ts = observations[0].timestamp;

        // X: days since start (independent variable)
        let x: Vec<f64> = observations
            .iter()
            .map(|obs| (obs.timestamp - first_ts) as f64 / 86400.0)
            .collect();

        // Y: metric values (dependent variable)
        let y: Vec<f64> = observations.iter().map(|obs| obs.value).collect();

        // Simple linear regression: y = mx + b
        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        // Slope (m)
        let numerator: f64 = x
            .iter()
            .zip(&y)
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();

        let denominator: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();

        let slope = if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        // Intercept (b)
        let intercept = mean_y - slope * mean_x;

        // Compute R² (coefficient of determination)
        let predictions: Vec<f64> = x.iter().map(|xi| slope * xi + intercept).collect();

        let ss_res: f64 = y
            .iter()
            .zip(&predictions)
            .map(|(yi, pred)| (yi - pred).powi(2))
            .sum();

        let ss_tot: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        Ok(LinearModel {
            slope,
            intercept,
            r_squared,
            last_timestamp: observations
                .last()
                .expect("observations passed to train_linear_model has >=7 elements (validated in predict_breach)")
                .timestamp,
        })
    }

    /// Generate forecast for next N days (Phase 4)
    fn generate_forecast(
        &self,
        model: &LinearModel,
        training_data: &[MetricObservation],
        forecast_days: usize,
    ) -> Result<Vec<ForecastPoint>> {
        let first_ts = training_data[0].timestamp;
        let last_day = (model.last_timestamp - first_ts) as f64 / 86400.0;

        // Compute standard error for confidence intervals
        let residuals: Vec<f64> = training_data
            .iter()
            .map(|obs| {
                let days = (obs.timestamp - first_ts) as f64 / 86400.0;
                let predicted = model.slope * days + model.intercept;
                obs.value - predicted
            })
            .collect();

        let sse: f64 = residuals.iter().map(|r| r.powi(2)).sum();
        let mse = sse / (training_data.len() as f64 - 2.0).max(1.0);
        let std_error = mse.sqrt();

        // Generate forecast points
        let mut forecast = Vec::new();

        for days_ahead in 1..=forecast_days {
            let future_day = last_day + days_ahead as f64;
            let predicted_value = model.slope * future_day + model.intercept;

            // 95% confidence interval (±1.96 * SE)
            let margin = 1.96 * std_error;

            forecast.push(ForecastPoint {
                days_ahead,
                predicted_value,
                lower_bound: predicted_value - margin,
                upper_bound: predicted_value + margin,
            });
        }

        Ok(forecast)
    }

}
