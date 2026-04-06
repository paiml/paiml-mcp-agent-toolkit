impl DataScienceAnalyzer {
    /// Analyze metric trends
    pub fn analyze_trends(&self, metrics: &[(String, Vec<(i64, f64)>)]) -> Vec<MetricTrend> {
        debug_assert!(!metrics.is_empty(), "metrics must not be empty");
        metrics
            .iter()
            .map(|(name, data)| {
                if data.is_empty() {
                    return MetricTrend {
                        name: name.clone(),
                        current: 0.0,
                        direction: TrendDirection::Stable,
                        change_percent: 0.0,
                        sparkline: Vec::new(),
                        forecast: None,
                    };
                }

                let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();
                let current = *values.last().unwrap_or(&0.0);

                // Calculate trend using linear regression
                let direction = self.calculate_trend_direction(&values);

                // Calculate change percentage
                let first = *values.first().unwrap_or(&0.0);
                let change_percent = if first != 0.0 {
                    ((current - first) / first.abs()) * 100.0
                } else {
                    0.0
                };

                // Generate sparkline (normalize to 0-7)
                let sparkline = self.values_to_sparkline(&values);

                // Simple forecast (linear extrapolation)
                let forecast = self.forecast_next(&values);

                MetricTrend {
                    name: name.clone(),
                    current,
                    direction,
                    change_percent,
                    sparkline,
                    forecast,
                }
            })
            .collect()
    }

    /// Calculate trend direction from values
    fn calculate_trend_direction(&self, values: &[f64]) -> TrendDirection {
        debug_assert!(!values.is_empty(), "values must not be empty");
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean: f64 = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }

        if denominator == 0.0 {
            return TrendDirection::Stable;
        }

        let slope = numerator / denominator;
        let threshold = y_mean.abs() * 0.05;

        if slope > threshold {
            TrendDirection::Degrading
        } else if slope < -threshold {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    /// Convert values to sparkline indices (0-7)
    #[allow(clippy::cast_possible_truncation)]
    fn values_to_sparkline(&self, values: &[f64]) -> Vec<u8> {
        debug_assert!(!values.is_empty(), "values must not be empty");
        if values.is_empty() {
            return Vec::new();
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        if range == 0.0 {
            return vec![4; values.len()];
        }

        values
            .iter()
            .map(|&v| ((v - min) / range * 7.0).round() as u8)
            .collect()
    }

    /// Simple linear forecast for next value
    fn forecast_next(&self, values: &[f64]) -> Option<f64> {
        debug_assert!(!values.is_empty(), "values must not be empty");
        if values.len() < 2 {
            return None;
        }

        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean: f64 = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }

        if denominator == 0.0 {
            return Some(*values.last().expect("internal error"));
        }

        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;

        // Predict next value
        Some(slope * n + intercept)
    }
}
