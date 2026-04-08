// Code quality analyzer methods
// Included by refactor.rs — shares parent module scope

impl CodeAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(_profile: QualityProfile) -> Self {
        Self {}
    }

    /// Analyze code quality
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze(&self, code: &str) -> Result<CodeAnalysis> {
        Ok(CodeAnalysis {
            complexity: self.calculate_complexity(code),
            coverage: self.estimate_coverage(code),
            tdg: self.calculate_tdg(code),
            satd_count: self.count_satd(code),
            function_count: self.count_functions(code),
            quality_score: self.calculate_quality_score(code),
        })
    }

    fn calculate_complexity(&self, code: &str) -> u32 {
        let if_count = code.matches("if ").count() as u32;
        let match_count = code.matches("match ").count() as u32;
        let loop_count =
            code.matches("for ").count() as u32 + code.matches("while ").count() as u32;

        1 + if_count + match_count + loop_count
    }

    fn estimate_coverage(&self, code: &str) -> f64 {
        let test_lines = code.matches("#[test]").count() * 10; // Rough estimate
        let total_lines = code.lines().count().max(1);
        (test_lines as f64 / total_lines as f64 * 100.0).min(100.0)
    }

    fn calculate_tdg(&self, code: &str) -> u32 {
        let todo_count = code.matches("todo!").count() as u32;
        let unwrap_count = code.matches("unwrap").count() as u32;
        todo_count + unwrap_count
    }

    fn count_satd(&self, code: &str) -> u32 {
        (code.matches("TODO").count()
            + code.matches("FIXME").count()
            + code.matches("HACK").count()) as u32
    }

    fn count_functions(&self, code: &str) -> usize {
        code.matches("fn ").count()
    }

    fn calculate_quality_score(&self, code: &str) -> f64 {
        let complexity = f64::from(self.calculate_complexity(code));
        let coverage = self.estimate_coverage(code);
        let tdg = f64::from(self.calculate_tdg(code));

        let complexity_score = (20.0 - complexity).max(0.0) / 20.0 * 40.0;
        let coverage_score = coverage * 0.4;
        let tdg_score = (10.0 - tdg).max(0.0) / 10.0 * 20.0;

        complexity_score + coverage_score + tdg_score
    }
}
