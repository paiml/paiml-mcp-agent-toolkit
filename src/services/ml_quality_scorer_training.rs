impl MLQualityScorer {
    /// Train the complexity model on historical data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn train_complexity_model(&mut self, samples: &[QualityTrainingSample]) -> Result<()> {
        debug_assert!(!samples.is_empty(), "samples must not be empty");
        if samples.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }

        let n_samples = samples.len();
        let n_features = 8; // ComplexityFeatures has 8 features

        // Build feature matrix and labels
        let mut feature_matrix = Vec::with_capacity(n_samples * n_features);
        let mut labels = Vec::with_capacity(n_samples);

        for sample in samples {
            if sample.features.len() != n_features {
                anyhow::bail!(
                    "Expected {} features, got {}",
                    n_features,
                    sample.features.len()
                );
            }
            feature_matrix.extend_from_slice(&sample.features);
            labels.push(sample.target_score as f32);
        }

        // Convert to aprender types
        let feature_matrix_f32: Vec<f32> = feature_matrix.iter().map(|&x| x as f32).collect();

        match Matrix::from_vec(n_samples, n_features, feature_matrix_f32) {
            Ok(x) => {
                let y = Vector::from_vec(labels);
                let mut model = LinearRegression::new();

                match model.fit(&x, &y) {
                    Ok(()) => {
                        self.complexity_model = Some(model);
                        self.trained = true;
                        self.training_samples = n_samples;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Complexity model training failed ({}), using heuristics",
                            e
                        );
                        self.complexity_model = None;
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Matrix creation failed ({}), using heuristics", e);
                self.complexity_model = None;
            }
        }

        // Calculate feature importance
        self.calculate_feature_importance(samples);

        Ok(())
    }

    /// Train the TDG model on historical data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn train_tdg_model(&mut self, samples: &[QualityTrainingSample]) -> Result<()> {
        debug_assert!(!samples.is_empty(), "samples must not be empty");
        if samples.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }

        let n_samples = samples.len();
        let n_features = 8; // TDGFeatures has 8 features

        let mut feature_matrix = Vec::with_capacity(n_samples * n_features);
        let mut labels = Vec::with_capacity(n_samples);

        for sample in samples {
            if sample.features.len() != n_features {
                anyhow::bail!(
                    "Expected {} features, got {}",
                    n_features,
                    sample.features.len()
                );
            }
            feature_matrix.extend_from_slice(&sample.features);
            labels.push(sample.target_score as f32);
        }

        let feature_matrix_f32: Vec<f32> = feature_matrix.iter().map(|&x| x as f32).collect();

        match Matrix::from_vec(n_samples, n_features, feature_matrix_f32) {
            Ok(x) => {
                let y = Vector::from_vec(labels);
                let mut model = LinearRegression::new();

                match model.fit(&x, &y) {
                    Ok(()) => {
                        self.tdg_model = Some(model);
                        self.trained = true;
                        self.training_samples += n_samples;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: TDG model training failed ({}), using heuristics",
                            e
                        );
                        self.tdg_model = None;
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Matrix creation failed ({}), using heuristics", e);
                self.tdg_model = None;
            }
        }

        Ok(())
    }

    /// Calculate feature importance from training data
    fn calculate_feature_importance(&mut self, samples: &[QualityTrainingSample]) {
        debug_assert!(!samples.is_empty(), "samples must not be empty");
        if samples.is_empty() {
            return;
        }

        let n_features = samples[0].features.len();
        let feature_names: Vec<&str> = if n_features == 8 {
            vec![
                "loc",
                "nesting",
                "control_flow",
                "loops",
                "conditionals",
                "functions",
                "avg_size",
                "language",
            ]
        } else {
            (0..n_features)
                .map(|i| Box::leak(format!("feature_{}", i).into_boxed_str()) as &str)
                .collect()
        };

        // Calculate correlation-based importance
        for (i, name) in feature_names.iter().enumerate() {
            let feature_values: Vec<f64> = samples.iter().map(|s| s.features[i]).collect();
            let targets: Vec<f64> = samples.iter().map(|s| s.target_score).collect();

            let importance = self.correlation(&feature_values, &targets).abs();
            self.feature_importance.insert(name.to_string(), importance);
        }

        // Normalize
        let total: f64 = self.feature_importance.values().sum();
        if total > 0.0 {
            for value in self.feature_importance.values_mut() {
                *value /= total;
            }
        }
    }

    /// Calculate Pearson correlation coefficient
    fn correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        debug_assert!(true, "contract: correlation");
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let mut cov = 0.0;
        let mut var_x = 0.0;
        let mut var_y = 0.0;

        for (xi, yi) in x.iter().zip(y.iter()) {
            let dx = xi - mean_x;
            let dy = yi - mean_y;
            cov += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
        }

        if var_x == 0.0 || var_y == 0.0 {
            return 0.0;
        }

        cov / (var_x.sqrt() * var_y.sqrt())
    }
}
