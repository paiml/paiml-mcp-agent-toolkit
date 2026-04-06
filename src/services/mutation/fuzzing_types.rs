/// Fuzzing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzConfig {
    /// Number of fuzzing iterations per mutant
    pub iterations: usize,

    /// Input generation strategy
    pub input_generator: InputGeneratorType,

    /// Enable crash detection
    pub crash_detection: bool,

    /// Timeout per fuzz iteration
    pub iteration_timeout: Duration,
}

impl Default for FuzzConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            input_generator: InputGeneratorType::Random,
            crash_detection: true,
            iteration_timeout: Duration::from_millis(100),
        }
    }
}

impl FuzzConfig {
    /// Validate configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate(&self) -> Result<()> {
        if self.iterations == 0 {
            anyhow::bail!("iterations must be > 0");
        }
        if self.iteration_timeout.as_millis() == 0 {
            anyhow::bail!("iteration_timeout must be > 0");
        }
        Ok(())
    }
}

/// Input generation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputGeneratorType {
    /// Pure random byte generation
    Random,

    /// Grammar-based generation (for parsers)
    GrammarBased,

    /// Mutation of existing inputs
    MutationBased,

    /// Coverage-guided (AFL-style)
    CoverageGuided,
}

/// Result of fuzzing a single mutant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzResult {
    /// Inputs that caused crashes
    pub crashes: Vec<String>,

    /// Inputs that caused hangs/timeouts
    pub hangs: Vec<Vec<u8>>,

    /// Coverage increase (0.0 - 1.0)
    pub coverage_increase: f64,
}

impl FuzzResult {
    /// Check if any crashes were detected
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_crashes(&self) -> bool {
        !self.crashes.is_empty()
    }

    /// Check if any hangs were detected
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_hangs(&self) -> bool {
        !self.hangs.is_empty()
    }
}

/// Aggregated fuzzing report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzMutationReport {
    /// Total number of mutants tested
    pub total_mutants: usize,

    /// Number of mutants that caused crashes
    pub mutants_with_crashes: usize,

    /// Number of mutants that caused hangs
    pub mutants_with_hangs: usize,

    /// Total execution time
    pub execution_time: Duration,

    /// Individual fuzz results per mutant
    pub results: Vec<(Mutant, FuzzResult)>,
}
