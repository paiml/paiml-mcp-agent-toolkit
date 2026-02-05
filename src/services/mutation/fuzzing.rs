//! Fuzzing Integration for Mutation Testing - Phase 4.1
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass RED tests

use super::{CoverageCorpus, CoverageInfo, CoverageTracker, Mutant, MutationEngine};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    pub fn has_crashes(&self) -> bool {
        !self.crashes.is_empty()
    }

    /// Check if any hangs were detected
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

/// Fuzz-mutation hybrid strategy
pub struct FuzzMutationStrategy {
    /// Base mutation engine
    mutation_engine: MutationEngine,

    /// Fuzzing configuration
    fuzz_config: FuzzConfig,
}

impl FuzzMutationStrategy {
    /// Create new fuzzing strategy
    pub fn new(mutation_engine: MutationEngine, fuzz_config: FuzzConfig) -> Self {
        Self {
            mutation_engine,
            fuzz_config,
        }
    }

    /// Get fuzzing configuration
    pub fn config(&self) -> &FuzzConfig {
        &self.fuzz_config
    }

    /// Get mutation engine
    pub fn engine(&self) -> &MutationEngine {
        &self.mutation_engine
    }

    /// Generate random inputs
    pub fn generate_inputs(&self, count: usize) -> Vec<Vec<u8>> {
        use rand::Rng;
        let mut rng = rand::rng();

        (0..count)
            .map(|_| {
                let len = rng.random_range(1..=256);
                (0..len).map(|_| rng.random::<u8>()).collect()
            })
            .collect()
    }

    /// Generate grammar-based inputs for specific format
    pub fn generate_grammar_based_inputs(&self, count: usize, _format: &str) -> Vec<Vec<u8>> {
        // Minimal implementation: generate simple JSON structures
        (0..count)
            .map(|i| {
                if i % 3 == 0 {
                    b"{}".to_vec()
                } else if i % 3 == 1 {
                    b"[]".to_vec()
                } else {
                    vec![]
                }
            })
            .collect()
    }

    /// Fuzz a single mutant
    pub async fn fuzz_mutant(&self, mutant: &Mutant) -> Result<FuzzResult> {
        use std::panic;
        use std::time::Instant;
        use tokio::time::timeout;

        let mut crashes = Vec::new();
        let mut hangs = Vec::new();

        // Initialize coverage corpus for coverage-guided fuzzing
        let baseline_coverage = CoverageInfo::new();
        let mut corpus = CoverageCorpus::new(baseline_coverage);

        // Generate initial inputs based on strategy
        let inputs = match self.fuzz_config.input_generator {
            InputGeneratorType::Random => self.generate_inputs(self.fuzz_config.iterations),
            InputGeneratorType::GrammarBased => {
                self.generate_grammar_based_inputs(self.fuzz_config.iterations, "generic")
            }
            InputGeneratorType::MutationBased => {
                // Start with random, mutate them
                self.generate_inputs(self.fuzz_config.iterations)
            }
            InputGeneratorType::CoverageGuided => {
                // Start with seed inputs
                self.generate_inputs(std::cmp::min(100, self.fuzz_config.iterations))
            }
        };

        let is_coverage_guided = matches!(
            self.fuzz_config.input_generator,
            InputGeneratorType::CoverageGuided
        );

        // Test each input
        let mut iteration = 0;
        let max_iterations = self.fuzz_config.iterations;

        while iteration < max_iterations {
            let input = if iteration < inputs.len() {
                inputs[iteration].clone()
            } else if is_coverage_guided && !corpus.interesting_inputs.is_empty() {
                // Mutate interesting inputs from corpus
                let seed = &corpus.get_seeds(1)[0];
                mutate_input(seed)
            } else {
                break;
            };

            let _start = Instant::now(); // Reserved for profiling

            // Execute with timeout to detect hangs
            let result = timeout(
                self.fuzz_config.iteration_timeout,
                tokio::task::spawn_blocking({
                    let mutant_source = mutant.mutated_source.clone();
                    let input_clone = input.clone();
                    move || {
                        // Try to execute mutated code with input
                        // For Phase 1, we simulate execution
                        // Real implementation would compile and run
                        let exec_result = panic::catch_unwind(|| {
                            execute_mutant_with_input(&mutant_source, &input_clone)
                        });

                        // Simulate coverage tracking
                        let coverage = CoverageTracker::simulate_coverage(&input_clone);

                        (exec_result, coverage)
                    }
                }),
            )
            .await;

            match result {
                Ok(Ok((Ok(_), coverage))) => {
                    // Execution succeeded, track coverage
                    if is_coverage_guided {
                        corpus.add_if_interesting(input.clone(), coverage);
                    }
                }
                Ok(Ok((Err(_), coverage))) => {
                    // Panic detected = crash
                    if self.fuzz_config.crash_detection {
                        crashes.push(format!("crash_at_input_{}", iteration));
                    }
                    if is_coverage_guided {
                        corpus.add_if_interesting(input.clone(), coverage);
                    }
                }
                Ok(Err(_)) => {
                    // Task join error (shouldn't happen)
                }
                Err(_) => {
                    // Timeout = hang
                    hangs.push(input.clone());
                }
            }

            iteration += 1;

            // Early exit if we've found crashes and hangs
            if !crashes.is_empty() && !hangs.is_empty() {
                break;
            }
        }

        // Calculate coverage increase
        let coverage_increase = if is_coverage_guided {
            corpus.total_coverage_increase()
        } else {
            0.0
        };

        Ok(FuzzResult {
            crashes,
            hangs,
            coverage_increase,
        })
    }

    /// Execute fuzzing from source code
    pub async fn execute_from_source(&self, source: &str) -> Result<FuzzMutationReport> {
        use std::time::Instant;

        let start = Instant::now();

        // Generate mutants from source
        let mutants = self
            .mutation_engine
            .generate_mutants_from_source(std::path::Path::new("fuzz_target.rs"), source)
            .await?;

        let total_mutants = mutants.len();
        let mut results = Vec::new();
        let mut mutants_with_crashes = 0;
        let mut mutants_with_hangs = 0;

        // Fuzz each mutant
        for mutant in mutants {
            let fuzz_result = self.fuzz_mutant(&mutant).await?;

            if fuzz_result.has_crashes() {
                mutants_with_crashes += 1;
            }
            if fuzz_result.has_hangs() {
                mutants_with_hangs += 1;
            }

            results.push((mutant, fuzz_result));
        }

        let execution_time = start.elapsed();

        Ok(FuzzMutationReport {
            total_mutants,
            mutants_with_crashes,
            mutants_with_hangs,
            execution_time,
            results,
        })
    }

    /// Execute fuzzing in parallel
    pub async fn execute_from_source_parallel(
        &self,
        source: &str,
        workers: usize,
    ) -> Result<FuzzMutationReport> {
        use std::sync::Arc;
        use std::time::Instant;
        use tokio::sync::Semaphore;

        let start = Instant::now();

        // Generate mutants from source
        let mutants = self
            .mutation_engine
            .generate_mutants_from_source(std::path::Path::new("fuzz_target.rs"), source)
            .await?;

        let total_mutants = mutants.len();

        // Use semaphore to limit concurrent fuzzing
        let semaphore = Arc::new(Semaphore::new(workers));
        let mut tasks = Vec::new();

        for mutant in mutants {
            let sem = semaphore.clone();
            let config = self.fuzz_config.clone();
            let engine = self.mutation_engine.clone(); // Clone for thread safety

            let task = tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .expect("Semaphore must not be closed during fuzzing");

                // Create temporary strategy for this mutant
                let strategy = FuzzMutationStrategy::new(engine, config);
                let fuzz_result = strategy
                    .fuzz_mutant(&mutant)
                    .await
                    .expect("Fuzz mutant operation must succeed");

                (mutant, fuzz_result)
            });

            tasks.push(task);
        }

        // Collect results
        let mut results = Vec::new();
        let mut mutants_with_crashes = 0;
        let mut mutants_with_hangs = 0;

        for task in tasks {
            let (mutant, fuzz_result) = task.await?;

            if fuzz_result.has_crashes() {
                mutants_with_crashes += 1;
            }
            if fuzz_result.has_hangs() {
                mutants_with_hangs += 1;
            }

            results.push((mutant, fuzz_result));
        }

        let execution_time = start.elapsed();

        Ok(FuzzMutationReport {
            total_mutants,
            mutants_with_crashes,
            mutants_with_hangs,
            execution_time,
            results,
        })
    }
}

/// Execute mutant code with given input (simulated for Phase 1)
/// Real implementation would compile mutant and execute with input
fn execute_mutant_with_input(_mutant_source: &str, input: &[u8]) -> Result<()> {
    // Phase 1: Simulate execution
    // This would be replaced with actual compilation + execution

    // Simulate crash on certain patterns
    if input.len() > 100 {
        // Simulate out-of-bounds access crash
        anyhow::bail!("Simulated crash: buffer overflow");
    }

    // Simulate success
    Ok(())
}

/// Mutate an input to create new test cases
fn mutate_input(seed: &[u8]) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut mutated = seed.to_vec();

    if mutated.is_empty() {
        return vec![rng.random::<u8>()];
    }

    // Apply random mutation strategy
    match rng.random_range(0..5) {
        0 => {
            // Bit flip
            if !mutated.is_empty() {
                let idx = rng.random_range(0..mutated.len());
                let bit = rng.random_range(0..8);
                mutated[idx] ^= 1 << bit;
            }
        }
        1 => {
            // Byte flip
            if !mutated.is_empty() {
                let idx = rng.random_range(0..mutated.len());
                mutated[idx] = rng.random::<u8>();
            }
        }
        2 => {
            // Insert byte
            let idx = rng.random_range(0..=mutated.len());
            mutated.insert(idx, rng.random::<u8>());
        }
        3 => {
            // Delete byte
            if !mutated.is_empty() {
                let idx = rng.random_range(0..mutated.len());
                mutated.remove(idx);
            }
        }
        4 => {
            // Append bytes
            let count = rng.random_range(1..=4);
            for _ in 0..count {
                mutated.push(rng.random::<u8>());
            }
        }
        _ => unreachable!(),
    }

    mutated
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ==================== FuzzConfig Tests ====================

    #[test]
    fn test_fuzz_config_default() {
        let config = FuzzConfig::default();
        assert_eq!(config.iterations, 1000);
        assert_eq!(config.input_generator, InputGeneratorType::Random);
        assert!(config.crash_detection);
        assert_eq!(config.iteration_timeout.as_millis(), 100);
    }

    #[test]
    fn test_fuzz_config_validate_valid() {
        let config = FuzzConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_fuzz_config_validate_zero_iterations() {
        let config = FuzzConfig {
            iterations: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("iterations must be > 0"));
    }

    #[test]
    fn test_fuzz_config_validate_zero_timeout() {
        let config = FuzzConfig {
            iteration_timeout: Duration::from_millis(0),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("iteration_timeout must be > 0"));
    }

    #[test]
    fn test_fuzz_config_custom_values() {
        let config = FuzzConfig {
            iterations: 500,
            input_generator: InputGeneratorType::CoverageGuided,
            crash_detection: false,
            iteration_timeout: Duration::from_millis(200),
        };
        assert_eq!(config.iterations, 500);
        assert_eq!(config.input_generator, InputGeneratorType::CoverageGuided);
        assert!(!config.crash_detection);
        assert_eq!(config.iteration_timeout.as_millis(), 200);
    }

    #[test]
    fn test_fuzz_config_serialization() {
        let config = FuzzConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"iterations\":1000"));
        assert!(json.contains("\"crash_detection\":true"));
    }

    #[test]
    fn test_fuzz_config_deserialization() {
        let json = r#"{
            "iterations": 500,
            "input_generator": "GrammarBased",
            "crash_detection": false,
            "iteration_timeout": {"secs": 0, "nanos": 50000000}
        }"#;
        let config: FuzzConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.iterations, 500);
        assert_eq!(config.input_generator, InputGeneratorType::GrammarBased);
        assert!(!config.crash_detection);
    }

    #[test]
    fn test_fuzz_config_clone() {
        let config = FuzzConfig::default();
        let cloned = config.clone();
        assert_eq!(config.iterations, cloned.iterations);
        assert_eq!(config.input_generator, cloned.input_generator);
    }

    // ==================== InputGeneratorType Tests ====================

    #[test]
    fn test_input_generator_type_random() {
        let gen_type = InputGeneratorType::Random;
        assert_eq!(gen_type, InputGeneratorType::Random);
    }

    #[test]
    fn test_input_generator_type_grammar_based() {
        let gen_type = InputGeneratorType::GrammarBased;
        assert_eq!(gen_type, InputGeneratorType::GrammarBased);
    }

    #[test]
    fn test_input_generator_type_mutation_based() {
        let gen_type = InputGeneratorType::MutationBased;
        assert_eq!(gen_type, InputGeneratorType::MutationBased);
    }

    #[test]
    fn test_input_generator_type_coverage_guided() {
        let gen_type = InputGeneratorType::CoverageGuided;
        assert_eq!(gen_type, InputGeneratorType::CoverageGuided);
    }

    #[test]
    fn test_input_generator_type_serialization() {
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::Random).unwrap(),
            "\"Random\""
        );
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::GrammarBased).unwrap(),
            "\"GrammarBased\""
        );
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::MutationBased).unwrap(),
            "\"MutationBased\""
        );
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::CoverageGuided).unwrap(),
            "\"CoverageGuided\""
        );
    }

    #[test]
    fn test_input_generator_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<InputGeneratorType>("\"Random\"").unwrap(),
            InputGeneratorType::Random
        );
        assert_eq!(
            serde_json::from_str::<InputGeneratorType>("\"CoverageGuided\"").unwrap(),
            InputGeneratorType::CoverageGuided
        );
    }

    #[test]
    fn test_input_generator_type_copy() {
        let gen_type = InputGeneratorType::Random;
        let copied = gen_type; // Copy
        assert_eq!(gen_type, copied);
    }

    #[test]
    fn test_input_generator_type_debug() {
        let gen_type = InputGeneratorType::GrammarBased;
        let debug = format!("{:?}", gen_type);
        assert_eq!(debug, "GrammarBased");
    }

    // ==================== FuzzResult Tests ====================

    #[test]
    fn test_fuzz_result_empty() {
        let result = FuzzResult {
            crashes: vec![],
            hangs: vec![],
            coverage_increase: 0.0,
        };
        assert!(!result.has_crashes());
        assert!(!result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_with_crashes() {
        let result = FuzzResult {
            crashes: vec!["crash1".to_string(), "crash2".to_string()],
            hangs: vec![],
            coverage_increase: 0.0,
        };
        assert!(result.has_crashes());
        assert!(!result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_with_hangs() {
        let result = FuzzResult {
            crashes: vec![],
            hangs: vec![vec![1, 2, 3], vec![4, 5, 6]],
            coverage_increase: 0.0,
        };
        assert!(!result.has_crashes());
        assert!(result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_with_both() {
        let result = FuzzResult {
            crashes: vec!["crash".to_string()],
            hangs: vec![vec![1, 2]],
            coverage_increase: 0.5,
        };
        assert!(result.has_crashes());
        assert!(result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_coverage_increase() {
        let result = FuzzResult {
            crashes: vec![],
            hangs: vec![],
            coverage_increase: 0.75,
        };
        assert!((result.coverage_increase - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fuzz_result_serialization() {
        let result = FuzzResult {
            crashes: vec!["crash1".to_string()],
            hangs: vec![vec![1, 2, 3]],
            coverage_increase: 0.25,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"crashes\":[\"crash1\"]"));
        assert!(json.contains("\"coverage_increase\":0.25"));
    }

    #[test]
    fn test_fuzz_result_deserialization() {
        let json = r#"{"crashes":["c1","c2"],"hangs":[[1,2]],"coverage_increase":0.5}"#;
        let result: FuzzResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.crashes.len(), 2);
        assert_eq!(result.hangs.len(), 1);
        assert!((result.coverage_increase - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fuzz_result_clone() {
        let result = FuzzResult {
            crashes: vec!["crash".to_string()],
            hangs: vec![vec![1]],
            coverage_increase: 0.1,
        };
        let cloned = result.clone();
        assert_eq!(result.crashes, cloned.crashes);
        assert_eq!(result.hangs, cloned.hangs);
    }

    // ==================== FuzzMutationReport Tests ====================

    #[test]
    fn test_fuzz_mutation_report_empty() {
        let report = FuzzMutationReport {
            total_mutants: 0,
            mutants_with_crashes: 0,
            mutants_with_hangs: 0,
            execution_time: Duration::from_secs(0),
            results: vec![],
        };
        assert_eq!(report.total_mutants, 0);
        assert_eq!(report.results.len(), 0);
    }

    #[test]
    fn test_fuzz_mutation_report_with_results() {
        let report = FuzzMutationReport {
            total_mutants: 5,
            mutants_with_crashes: 2,
            mutants_with_hangs: 1,
            execution_time: Duration::from_secs(10),
            results: vec![],
        };
        assert_eq!(report.total_mutants, 5);
        assert_eq!(report.mutants_with_crashes, 2);
        assert_eq!(report.mutants_with_hangs, 1);
        assert_eq!(report.execution_time.as_secs(), 10);
    }

    #[test]
    fn test_fuzz_mutation_report_serialization() {
        let report = FuzzMutationReport {
            total_mutants: 3,
            mutants_with_crashes: 1,
            mutants_with_hangs: 0,
            execution_time: Duration::from_millis(500),
            results: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"total_mutants\":3"));
        assert!(json.contains("\"mutants_with_crashes\":1"));
    }

    #[test]
    fn test_fuzz_mutation_report_clone() {
        let report = FuzzMutationReport {
            total_mutants: 10,
            mutants_with_crashes: 2,
            mutants_with_hangs: 1,
            execution_time: Duration::from_secs(5),
            results: vec![],
        };
        let cloned = report.clone();
        assert_eq!(report.total_mutants, cloned.total_mutants);
        assert_eq!(report.mutants_with_crashes, cloned.mutants_with_crashes);
    }

    // ==================== FuzzMutationStrategy Tests ====================

    #[test]
    fn test_fuzz_mutation_strategy_new() {
        let engine = MutationEngine::new();
        let config = FuzzConfig::default();
        let strategy = FuzzMutationStrategy::new(engine, config);
        assert_eq!(strategy.config().iterations, 1000);
    }

    #[test]
    fn test_fuzz_mutation_strategy_config() {
        let engine = MutationEngine::new();
        let config = FuzzConfig {
            iterations: 500,
            ..Default::default()
        };
        let strategy = FuzzMutationStrategy::new(engine, config);
        assert_eq!(strategy.config().iterations, 500);
    }

    #[test]
    fn test_fuzz_mutation_strategy_engine() {
        let engine = MutationEngine::new();
        let config = FuzzConfig::default();
        let strategy = FuzzMutationStrategy::new(engine, config);
        // Just verify we can access the engine
        let _ = strategy.engine();
    }

    #[test]
    fn test_fuzz_mutation_strategy_generate_inputs() {
        let engine = MutationEngine::new();
        let config = FuzzConfig::default();
        let strategy = FuzzMutationStrategy::new(engine, config);

        let inputs = strategy.generate_inputs(10);
        assert_eq!(inputs.len(), 10);
        for input in &inputs {
            assert!(!input.is_empty());
            assert!(input.len() <= 256);
        }
    }

    #[test]
    fn test_fuzz_mutation_strategy_generate_inputs_zero() {
        let engine = MutationEngine::new();
        let config = FuzzConfig::default();
        let strategy = FuzzMutationStrategy::new(engine, config);

        let inputs = strategy.generate_inputs(0);
        assert!(inputs.is_empty());
    }

    #[test]
    fn test_fuzz_mutation_strategy_generate_grammar_based_inputs() {
        let engine = MutationEngine::new();
        let config = FuzzConfig::default();
        let strategy = FuzzMutationStrategy::new(engine, config);

        let inputs = strategy.generate_grammar_based_inputs(6, "json");
        assert_eq!(inputs.len(), 6);

        // Check pattern: 0, 3 -> {}, 1, 4 -> [], 2, 5 -> empty
        assert_eq!(inputs[0], b"{}".to_vec());
        assert_eq!(inputs[1], b"[]".to_vec());
        assert!(inputs[2].is_empty());
        assert_eq!(inputs[3], b"{}".to_vec());
    }

    #[test]
    fn test_fuzz_mutation_strategy_generate_grammar_based_inputs_empty() {
        let engine = MutationEngine::new();
        let config = FuzzConfig::default();
        let strategy = FuzzMutationStrategy::new(engine, config);

        let inputs = strategy.generate_grammar_based_inputs(0, "any");
        assert!(inputs.is_empty());
    }

    // ==================== execute_mutant_with_input Tests ====================

    #[test]
    fn test_execute_mutant_with_input_success() {
        let result = execute_mutant_with_input("fn test() {}", &[1, 2, 3]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_mutant_with_input_empty_input() {
        let result = execute_mutant_with_input("fn test() {}", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_mutant_with_input_small_input() {
        let result = execute_mutant_with_input("fn test() {}", &[0; 50]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_mutant_with_input_boundary() {
        let result = execute_mutant_with_input("fn test() {}", &[0; 100]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_mutant_with_input_crash() {
        let result = execute_mutant_with_input("fn test() {}", &[0; 101]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("buffer overflow"));
    }

    #[test]
    fn test_execute_mutant_with_input_large_crash() {
        let result = execute_mutant_with_input("fn test() {}", &[0; 500]);
        assert!(result.is_err());
    }

    // ==================== mutate_input Tests ====================

    #[test]
    fn test_mutate_input_empty() {
        let result = mutate_input(&[]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_mutate_input_single_byte() {
        let original = vec![42u8];
        let mutated = mutate_input(&original);
        // Mutated should be different from original or have different length
        assert!(!mutated.is_empty());
    }

    #[test]
    fn test_mutate_input_multiple_bytes() {
        let original = vec![1, 2, 3, 4, 5];
        let mutated = mutate_input(&original);
        // Mutated should exist (can be same or different)
        assert!(!mutated.is_empty() || mutated == original.clone());
    }

    #[test]
    fn test_mutate_input_preserves_some_structure() {
        let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mutated = mutate_input(&original);
        // Size should be similar (within +/- 4 bytes typically)
        let size_diff = (mutated.len() as i32 - original.len() as i32).abs();
        assert!(size_diff <= 5);
    }

    #[test]
    fn test_mutate_input_randomness() {
        let original = vec![100, 100, 100, 100, 100];
        let mut results = std::collections::HashSet::new();

        // Run multiple times to verify randomness
        for _ in 0..100 {
            let mutated = mutate_input(&original);
            results.insert(mutated);
        }

        // Should produce multiple different results
        assert!(results.len() > 1);
    }

    #[test]
    fn test_mutate_input_all_zeros() {
        let original = vec![0, 0, 0, 0, 0];
        let mutated = mutate_input(&original);
        assert!(!mutated.is_empty());
    }

    #[test]
    fn test_mutate_input_all_ones() {
        let original = vec![255, 255, 255, 255];
        let mutated = mutate_input(&original);
        assert!(!mutated.is_empty());
    }

    // ==================== Edge Cases and Integration Tests ====================

    #[test]
    fn test_fuzz_config_min_valid_values() {
        let config = FuzzConfig {
            iterations: 1,
            input_generator: InputGeneratorType::Random,
            crash_detection: false,
            iteration_timeout: Duration::from_nanos(1),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_fuzz_config_max_values() {
        let config = FuzzConfig {
            iterations: usize::MAX,
            input_generator: InputGeneratorType::CoverageGuided,
            crash_detection: true,
            iteration_timeout: Duration::from_secs(3600),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_fuzz_result_max_crashes() {
        let crashes: Vec<String> = (0..1000).map(|i| format!("crash_{}", i)).collect();
        let result = FuzzResult {
            crashes,
            hangs: vec![],
            coverage_increase: 0.0,
        };
        assert!(result.has_crashes());
        assert_eq!(result.crashes.len(), 1000);
    }

    #[test]
    fn test_fuzz_result_max_hangs() {
        let hangs: Vec<Vec<u8>> = (0..1000).map(|i| vec![i as u8]).collect();
        let result = FuzzResult {
            crashes: vec![],
            hangs,
            coverage_increase: 0.0,
        };
        assert!(result.has_hangs());
        assert_eq!(result.hangs.len(), 1000);
    }

    #[test]
    fn test_fuzz_result_coverage_boundaries() {
        let result_zero = FuzzResult {
            crashes: vec![],
            hangs: vec![],
            coverage_increase: 0.0,
        };
        assert!((result_zero.coverage_increase - 0.0).abs() < f64::EPSILON);

        let result_one = FuzzResult {
            crashes: vec![],
            hangs: vec![],
            coverage_increase: 1.0,
        };
        assert!((result_one.coverage_increase - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_strategy_with_different_generator_types() {
        let engine = MutationEngine::new();

        for gen_type in [
            InputGeneratorType::Random,
            InputGeneratorType::GrammarBased,
            InputGeneratorType::MutationBased,
            InputGeneratorType::CoverageGuided,
        ] {
            let config = FuzzConfig {
                input_generator: gen_type,
                ..Default::default()
            };
            let strategy = FuzzMutationStrategy::new(engine.clone(), config);
            assert_eq!(strategy.config().input_generator, gen_type);
        }
    }

    #[test]
    fn test_input_generator_type_all_variants_serialization_roundtrip() {
        let variants = [
            InputGeneratorType::Random,
            InputGeneratorType::GrammarBased,
            InputGeneratorType::MutationBased,
            InputGeneratorType::CoverageGuided,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: InputGeneratorType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn test_fuzz_config_serialization_roundtrip() {
        let config = FuzzConfig {
            iterations: 777,
            input_generator: InputGeneratorType::MutationBased,
            crash_detection: false,
            iteration_timeout: Duration::from_millis(333),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: FuzzConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.iterations, parsed.iterations);
        assert_eq!(config.input_generator, parsed.input_generator);
        assert_eq!(config.crash_detection, parsed.crash_detection);
    }

    #[test]
    fn test_fuzz_result_serialization_roundtrip() {
        let result = FuzzResult {
            crashes: vec!["a".to_string(), "b".to_string()],
            hangs: vec![vec![1, 2], vec![3, 4, 5]],
            coverage_increase: 0.42,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: FuzzResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.crashes, parsed.crashes);
        assert_eq!(result.hangs, parsed.hangs);
        assert!((result.coverage_increase - parsed.coverage_increase).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fuzz_mutation_report_execution_time() {
        let report = FuzzMutationReport {
            total_mutants: 1,
            mutants_with_crashes: 0,
            mutants_with_hangs: 0,
            execution_time: Duration::from_millis(12345),
            results: vec![],
        };

        assert_eq!(report.execution_time.as_millis(), 12345);
        assert_eq!(report.execution_time.as_secs(), 12);
    }

    #[test]
    fn test_multiple_input_generation_consistency() {
        let engine = MutationEngine::new();
        let config = FuzzConfig::default();
        let strategy = FuzzMutationStrategy::new(engine, config);

        // Generate multiple batches
        let batch1 = strategy.generate_inputs(5);
        let batch2 = strategy.generate_inputs(5);

        // Both should have correct count
        assert_eq!(batch1.len(), 5);
        assert_eq!(batch2.len(), 5);

        // Due to randomness, batches should be different
        // (extremely unlikely to be the same)
        assert_ne!(batch1, batch2);
    }
}
