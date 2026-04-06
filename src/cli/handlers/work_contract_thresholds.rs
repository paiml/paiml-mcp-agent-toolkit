/// Contract thresholds (non-negotiable defaults)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractThresholds {
    /// Minimum total coverage (absolute, not relative)
    pub min_coverage_pct: f64,

    /// Minimum per-file coverage (v2.6 comply spec)
    pub min_per_file_coverage_pct: f64,

    /// Maximum allowed TDG regression (0 = no regression)
    pub max_tdg_regression: f64,

    /// Maximum cyclomatic complexity per function
    pub max_function_complexity: u32,

    /// Maximum file size in lines
    pub max_file_lines: usize,

    /// Minimum spec score for completion
    pub min_spec_score: u32,

    /// Require GitHub push on completion
    pub require_github_sync: bool,

    /// Require spec update for feature work
    pub require_spec_update: bool,

    /// Require roadmap update
    pub require_roadmap_update: bool,

    /// Block on new SATD markers (v2.6 comply spec)
    pub block_on_new_satd: bool,

    /// Block on new dead code (v2.6 comply spec)
    pub block_on_new_dead_code: bool,

    /// Require lint to pass (v2.6 comply spec)
    pub require_lint_pass: bool,

    /// Maximum consecutive fix commits on same file before blocking (v3.1 defect churn)
    #[serde(default = "default_max_fix_chain")]
    pub max_fix_chain: usize,

    /// Block on untested match arm variants (v3.1 defect churn)
    #[serde(default = "default_true")]
    pub block_on_untested_variants: bool,

    /// Block on cross-crate parity failures (v3.1 defect churn)
    #[serde(default)]
    pub block_on_cross_crate_failure: bool,

    /// Block on performance regression (v3.1 defect churn)
    #[serde(default)]
    pub block_on_regression: bool,

    /// Require proof verification for Lean 4 projects (v4.0 provable contracts)
    #[serde(default)]
    pub require_proof_verification: bool,

    /// Maximum allowed sorry count in Lean 4 files (v4.0 provable contracts)
    #[serde(default)]
    pub max_sorry_count: usize,

    /// Minimum theorem coverage ratio (v4.0 provable contracts)
    #[serde(default)]
    pub min_theorem_coverage: f64,

    /// Block on file size regression (v4.1 file health enforcement)
    #[serde(default = "default_true")]
    pub block_on_file_size: bool,
}

impl Default for ContractThresholds {
    fn default() -> Self {
        Self {
            min_coverage_pct: 95.0,
            min_per_file_coverage_pct: 95.0,
            max_tdg_regression: 0.0,
            max_function_complexity: 20,
            max_file_lines: 500,
            min_spec_score: 95,
            require_github_sync: true,
            require_spec_update: true,
            require_roadmap_update: true,
            block_on_new_satd: true,
            block_on_new_dead_code: true,
            require_lint_pass: true,
            max_fix_chain: 3,
            block_on_untested_variants: true,
            block_on_cross_crate_failure: false, // Off by default — requires sibling project config
            block_on_regression: false,          // Off by default — requires benchmark cache
            require_proof_verification: false,   // Off by default — opt-in for Lean 4 projects
            max_sorry_count: 0, // Zero sorrys allowed when proof verification enabled
            min_theorem_coverage: 0.0, // No minimum theorem coverage by default
            block_on_file_size: true, // Block on file size regression by default (v4.1)
        }
    }
}

fn default_max_fix_chain() -> usize {
    debug_assert!(true, "contract: default_max_fix_chain");
    3
}
fn default_true() -> bool {
    debug_assert!(true, "contract: default_true");
    true
}
