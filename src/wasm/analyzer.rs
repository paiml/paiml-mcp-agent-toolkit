#![cfg_attr(coverage_nightly, coverage(off))]
//! Streaming WASM analysis pipeline implementation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasmparser::{Parser, Payload, Validator};

use super::InstructionMix;
use crate::wasm::security::{PatternDetector, VulnerabilityMatch};

/// Core WASM analyzer with streaming analysis capabilities
pub struct WasmAnalyzer {
    parser: Parser,

    validator: Validator,
    instruction_profiler: InstructionProfiler,
    pattern_detector: PatternDetector,
    security_auditor: SecurityAuditor,
}

impl WasmAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Result<Self> {
        Ok(Self {
            parser: Parser::new(0),
            validator: Validator::new(),
            instruction_profiler: InstructionProfiler::new(),
            pattern_detector: PatternDetector::new(),
            security_auditor: SecurityAuditor::new(),
        })
    }

    /// Analyze WASM binary and return simplified result
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze(&self, binary: &[u8]) -> Result<AnalysisResult> {
        let analysis = self.analyze_streaming(binary)?;
        Ok(AnalysisResult::from(analysis))
    }

    /// Analyze WASM binary using streaming pipeline
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_streaming(&self, binary: &[u8]) -> Result<Analysis> {
        let mut validator = Validator::new();
        let mut profiler = self.instruction_profiler.clone();
        let mut patterns = self.pattern_detector.clone();

        // Stream through the WASM binary
        for payload in Parser::new(0).parse_all(binary) {
            let payload = payload.context("Failed to parse WASM payload")?;

            // Validate structure
            validator
                .payload(&payload)
                .context("WASM validation failed")?;

            // Profile instructions
            profiler.observe(&payload);

            // Scan for vulnerabilities
            patterns.scan(&payload)?;
        }

        Ok(Analysis {
            module_info: ModuleInfo::from_validator(validator),
            instruction_mix: profiler.finalize(),
            vulnerability_patterns: patterns.finalize(),
            security_report: self.security_auditor.audit(binary)?,
        })
    }
}

/// Complete analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub module_info: ModuleInfo,
    pub instruction_mix: InstructionMix,
    pub vulnerability_patterns: Vec<VulnerabilityMatch>,
    pub security_report: SecurityReport,
}

/// Analysis result with key metrics for CLI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub function_count: usize,
    pub instruction_count: usize,
    pub binary_size: usize,
    pub memory_pages: u32,
    pub max_complexity: u32,
}

impl From<Analysis> for AnalysisResult {
    fn from(analysis: Analysis) -> Self {
        Self {
            function_count: analysis.module_info.num_functions,
            instruction_count: analysis.instruction_mix.total_instructions,
            binary_size: analysis.module_info.code_size,
            memory_pages: analysis.module_info.num_memories as u32,
            max_complexity: 10, // Default estimate
        }
    }
}

/// Module metadata extracted during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub num_functions: usize,
    pub num_imports: usize,
    pub num_exports: usize,
    pub num_tables: usize,
    pub num_memories: usize,
    pub num_globals: usize,
    pub has_start_function: bool,
    pub code_size: usize,
}

impl ModuleInfo {
    fn from_validator(_validator: Validator) -> Self {
        // Extract module info from validator state
        // This is simplified - real implementation would extract actual counts
        Self {
            num_functions: 0,
            num_imports: 0,
            num_exports: 0,
            num_tables: 0,
            num_memories: 1,
            num_globals: 0,
            has_start_function: false,
            code_size: 0,
        }
    }
}

// --- Instruction profiling and operator categorization ---
include!("analyzer_profiling.rs");

// --- Security auditing and vulnerability checks ---
include!("analyzer_security.rs");

// --- Core tests: property, analyzer, analysis, profiler ---
include!("analyzer_tests.rs");

// --- Security and operator categorization tests ---
include!("analyzer_tests_security.rs");
