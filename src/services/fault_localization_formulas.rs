// SBFL scoring formulas and SbflLocalizer implementation.
// Contains: tarantula(), ochiai(), dstar() free functions and SbflLocalizer methods
// (localize, calculate_score, generate_explanation, calculate_confidence).

/// Classic Tarantula suspiciousness formula
///
/// Formula: (failed/totalFailed) / ((passed/totalPassed) + (failed/totalFailed))
///
/// Reference: Jones, J.A., Harrold, M.J. (2005). ASE '05
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn tarantula(failed: usize, passed: usize, total_failed: usize, total_passed: usize) -> f32 {
    let failed_ratio = if total_failed > 0 {
        failed as f32 / total_failed as f32
    } else {
        0.0
    };

    let passed_ratio = if total_passed > 0 {
        passed as f32 / total_passed as f32
    } else {
        0.0
    };

    let denominator = passed_ratio + failed_ratio;
    if denominator == 0.0 {
        0.0
    } else {
        failed_ratio / denominator
    }
}

/// Ochiai suspiciousness formula (from molecular biology)
///
/// Formula: failed / sqrt(totalFailed * (failed + passed))
///
/// Reference: Abreu et al. (2009). JSS 82(11)
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn ochiai(failed: usize, passed: usize, total_failed: usize) -> f32 {
    let denominator = ((total_failed * (failed + passed)) as f32).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        failed as f32 / denominator
    }
}

/// DStar suspiciousness formula with configurable exponent
///
/// Formula: failed^* / (passed + (totalFailed - failed))
///
/// Reference: Wong et al. (2014). IEEE TSE 40(1)
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn dstar(failed: usize, passed: usize, total_failed: usize, star: u32) -> f32 {
    let numerator = (failed as f32).powi(star as i32);
    let not_failed = total_failed.saturating_sub(failed);
    let denominator = passed as f32 + not_failed as f32;

    if denominator == 0.0 {
        if numerator > 0.0 {
            f32::MAX // Avoid infinity, use max finite value
        } else {
            0.0
        }
    } else {
        numerator / denominator
    }
}

impl SbflLocalizer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            formula: SbflFormula::Tarantula,
            top_n: 10,
            include_explanations: true,
            min_confidence_threshold: 0.0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_formula(mut self, formula: SbflFormula) -> Self {
        self.formula = formula;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_top_n(mut self, n: usize) -> Self {
        self.top_n = n;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_explanations(mut self, include: bool) -> Self {
        self.include_explanations = include;
        self
    }

    
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_min_confidence(mut self, threshold: f32) -> Self {
        self.min_confidence_threshold = threshold;
        self
    }

    /// Localize faults using the configured SBFL formula
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn localize(
        &self,
        coverage: &[StatementCoverage],
        total_passed: usize,
        total_failed: usize,
    ) -> FaultLocalizationResult {
        info!(
            "Running {:?} fault localization on {} statements",
            self.formula,
            coverage.len()
        );

        // Calculate suspiciousness for each statement
        let mut scored: Vec<(StatementId, f32, usize, usize)> = coverage
            .iter()
            .map(|cov| {
                let score = self.calculate_score(
                    cov.executed_by_failed,
                    cov.executed_by_passed,
                    total_failed,
                    total_passed,
                );
                (
                    cov.id.clone(),
                    score,
                    cov.executed_by_failed,
                    cov.executed_by_passed,
                )
            })
            .collect();

        // Sort by suspiciousness (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N
        let rankings: Vec<SuspiciousnessRanking> = scored
            .into_iter()
            .take(self.top_n)
            .enumerate()
            .filter(|(_, (_, score, _, _))| *score >= self.min_confidence_threshold)
            .map(|(rank, (stmt, score, failed, passed))| {
                let explanation = if self.include_explanations {
                    self.generate_explanation(failed, passed, total_failed, total_passed, score)
                } else {
                    String::new()
                };

                // Calculate all formula scores for comparison
                let mut scores = HashMap::new();
                scores.insert(
                    "tarantula".to_string(),
                    tarantula(failed, passed, total_failed, total_passed),
                );
                scores.insert("ochiai".to_string(), ochiai(failed, passed, total_failed));
                scores.insert("dstar2".to_string(), dstar(failed, passed, total_failed, 2));
                scores.insert("dstar3".to_string(), dstar(failed, passed, total_failed, 3));

                SuspiciousnessRanking {
                    rank: rank + 1,
                    statement: stmt,
                    suspiciousness: score,
                    scores,
                    explanation,
                    failed_coverage: failed,
                    passed_coverage: passed,
                }
            })
            .collect();

        // Calculate confidence based on test coverage density
        let confidence = self.calculate_confidence(coverage.len(), total_passed, total_failed);

        debug!(
            "Localized {} suspicious statements with confidence {}",
            rankings.len(),
            confidence
        );

        FaultLocalizationResult {
            rankings,
            formula_used: self.formula,
            confidence,
            total_passed_tests: total_passed,
            total_failed_tests: total_failed,
        }
    }

    fn calculate_score(
        &self,
        failed: usize,
        passed: usize,
        total_failed: usize,
        total_passed: usize,
    ) -> f32 {
        match self.formula {
            SbflFormula::Tarantula => tarantula(failed, passed, total_failed, total_passed),
            SbflFormula::Ochiai => ochiai(failed, passed, total_failed),
            SbflFormula::DStar { exponent } => dstar(failed, passed, total_failed, exponent),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn generate_explanation(
        &self,
        failed: usize,
        passed: usize,
        total_failed: usize,
        total_passed: usize,
        score: f32,
    ) -> String {
        let failed_pct = if total_failed > 0 {
            (failed as f32 / total_failed as f32 * 100.0) as u32
        } else {
            0
        };

        let passed_pct = if total_passed > 0 {
            (passed as f32 / total_passed as f32 * 100.0) as u32
        } else {
            0
        };

        format!(
            "Executed by {}% of failing tests ({}/{}) and {}% of passing tests ({}/{}). \
             Suspiciousness score: {:.3}",
            failed_pct, failed, total_failed, passed_pct, passed, total_passed, score
        )
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_confidence(
        &self,
        statement_count: usize,
        total_passed: usize,
        total_failed: usize,
    ) -> f32 {
        let total_tests = total_passed + total_failed;
        if total_tests == 0 || total_failed == 0 {
            return 0.0;
        }

        // Factor 1: Log scale for failing test count (diminishing returns)
        let fail_factor = (total_failed as f32).ln().min(3.0) / 3.0;

        // Factor 2: Failing ratio (sweet spot around 5-20%)
        let fail_ratio = total_failed as f32 / total_tests as f32;
        let ratio_factor = if fail_ratio < 0.01 {
            fail_ratio * 10.0 // Very few failures = low confidence
        } else if fail_ratio > 0.5 {
            1.0 - (fail_ratio - 0.5) // Too many failures = less localizing
        } else {
            1.0
        };

        // Factor 3: Statement coverage (more covered = more context)
        let coverage_factor = (statement_count as f32).ln().min(7.0) / 7.0;

        (fail_factor * ratio_factor * coverage_factor).min(1.0)
    }
}
