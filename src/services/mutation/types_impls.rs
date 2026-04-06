// MutationScore implementation methods
// Included from types.rs — no `use` imports or `#!` attributes allowed

impl MutationScore {
    /// Calculate mutation score from results
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_results(results: &[MutationResult]) -> Self {
        let total = results.len();
        let killed = results
            .iter()
            .filter(|r| r.status == MutantStatus::Killed)
            .count();
        let survived = results
            .iter()
            .filter(|r| r.status == MutantStatus::Survived)
            .count();
        let compile_errors = results
            .iter()
            .filter(|r| r.status == MutantStatus::CompileError)
            .count();
        let timeouts = results
            .iter()
            .filter(|r| r.status == MutantStatus::Timeout)
            .count();
        let equivalent = results
            .iter()
            .filter(|r| r.status == MutantStatus::Equivalent)
            .count();

        // Mutation score = killed / (total - equivalent)
        let valid_mutants = total.saturating_sub(equivalent + compile_errors);
        let score = if valid_mutants > 0 {
            killed as f64 / valid_mutants as f64
        } else {
            0.0
        };

        Self {
            score,
            total,
            killed,
            survived,
            compile_errors,
            timeouts,
            equivalent,
        }
    }
}
