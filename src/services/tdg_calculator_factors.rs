// Factor calculation: complexity gradient and heuristic complexity
// Churn-related code moved to tdg_calculator_factors_churn.rs

fn is_function_start(trimmed: &str) -> bool {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    trimmed.starts_with("fn ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("function ")
        || trimmed.starts_with("func ")
}

fn is_control_flow(trimmed: &str) -> bool {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    trimmed.starts_with("if ")
        || trimmed.starts_with("elif ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("match ")
        || trimmed.starts_with("case ")
}

fn score_from_complexities(function_complexities: &[usize], line_count: usize) -> f64 {
    let mean = function_complexities.iter().sum::<usize>() as f64
        / function_complexities.len() as f64;
    let variance = function_complexities
        .iter()
        .map(|&c| (c as f64 - mean).powi(2))
        .sum::<f64>()
        / function_complexities.len() as f64;

    let base_complexity = mean / 5.0;
    let variance_factor = variance.sqrt() / 3.0;
    let max_complexity = function_complexities.iter().max().copied().unwrap_or(0) as f64;
    let hotspot_factor = (max_complexity / 10.0).min(1.0);
    let loc_factor = (line_count as f64 / 100.0).min(1.0);

    let score =
        base_complexity * 0.4 + variance_factor * 0.2 + hotspot_factor * 0.2 + loc_factor * 0.2;
    score.min(5.0)
}

impl TDGCalculator {
    /// Compute complexity gradient with variance analysis
    #[allow(dead_code)]
    fn compute_complexity_gradient(&self, ast: &UnifiedAstNode) -> ComplexityVariance {
        let mut analyzer = VerifiedComplexityAnalyzer::new();
        let complexities: Vec<u32> = if matches!(ast.kind, AstKind::Function(_)) {
            vec![analyzer.analyze_function(ast).cyclomatic]
        } else {
            vec![]
        };

        if complexities.is_empty() {
            return ComplexityVariance {
                mean: 0.0,
                variance: 0.0,
                gini: 0.0,
                percentile_90: 0.0,
            };
        }

        let sum: u32 = complexities.iter().sum();
        let mean = f64::from(sum) / complexities.len() as f64;

        let squared_diff_sum: f64 = complexities
            .iter()
            .map(|&c| (f64::from(c) - mean).powi(2))
            .sum();
        let variance = squared_diff_sum / complexities.len() as f64;

        let mut sorted = complexities;
        sorted.sort_unstable();

        let mut gini_sum = 0.0;
        for (i, &value) in sorted.iter().enumerate() {
            gini_sum += (2.0 * (i + 1) as f64 - sorted.len() as f64 - 1.0) * f64::from(value);
        }
        let gini = if sum == 0 {
            0.0
        } else {
            gini_sum / (sorted.len() as f64 * f64::from(sum))
        };

        let percentile_idx = ((sorted.len() as f64 * 0.9) as usize).min(sorted.len() - 1);
        let percentile_90 = f64::from(sorted[percentile_idx]);

        ComplexityVariance {
            mean,
            variance,
            gini,
            percentile_90,
        }
    }

    /// Calculate complexity factor (normalized 0-5)
    async fn calculate_complexity_factor(&self, path: &Path) -> Result<f64> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content.lines().collect();

        let mut complexity = 0usize;
        let mut nesting_level = 0usize;
        let mut function_complexities = Vec::<usize>::new();

        for line in &lines {
            let trimmed = line.trim();

            if is_function_start(trimmed) {
                if complexity > 0 {
                    function_complexities.push(complexity);
                }
                complexity = 1;
                nesting_level = 0;
            }

            if is_control_flow(trimmed) {
                complexity += 1 + nesting_level;
            }

            nesting_level += trimmed.matches('{').count();
            nesting_level = nesting_level.saturating_sub(trimmed.matches('}').count());
        }

        if complexity > 0 {
            function_complexities.push(complexity);
        }

        if function_complexities.is_empty() {
            return Ok(0.5);
        }

        Ok(score_from_complexities(&function_complexities, lines.len()))
    }
}
