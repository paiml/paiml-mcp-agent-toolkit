//! WASM Quality Gates
//!
//! Defines and enforces WASM quality metrics per Toyota Way principles.

use crate::services::deep_wasm::{
    DeepWasmResult, IssueSeverity, QualityGateResults, QualityViolation, WasmModuleAnalysis,
};

/// WASM-specific quality issue types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmIssueType {
    UnreachableCode,
    UnboundedLoop,
    StackOverflow,
    MemoryLeak,
    UndefinedBehavior,
    TypeUnsafety,
}

/// WASM quality gates configuration
#[derive(Debug, Clone)]
pub struct WasmQualityGates {
    pub max_module_size: u64,
    pub max_wasm_complexity: u32,
    pub min_source_map_coverage: f64,
    pub max_stack_depth: u32,
    pub zero_tolerance: Vec<WasmIssueType>,
}

impl Default for WasmQualityGates {
    fn default() -> Self {
        Self {
            max_module_size: 10_485_760,        // 10 MB
            max_wasm_complexity: 20,
            min_source_map_coverage: 0.95,
            max_stack_depth: 1000,
            zero_tolerance: vec![
                WasmIssueType::UnreachableCode,
                WasmIssueType::UnboundedLoop,
                WasmIssueType::StackOverflow,
                WasmIssueType::MemoryLeak,
                WasmIssueType::UndefinedBehavior,
                WasmIssueType::TypeUnsafety,
            ],
        }
    }
}

impl WasmQualityGates {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(&self, analysis: &WasmModuleAnalysis) -> DeepWasmResult<QualityGateResults> {
        let mut violations = Vec::new();

        // Check module size
        if analysis.module_size_bytes > self.max_module_size {
            violations.push(QualityViolation {
                rule: "max_module_size".to_string(),
                severity: IssueSeverity::High,
                message: format!(
                    "Module size {} exceeds limit {}",
                    analysis.module_size_bytes, self.max_module_size
                ),
            });
        }

        // Check complexity
        if analysis.max_complexity > self.max_wasm_complexity {
            violations.push(QualityViolation {
                rule: "max_wasm_complexity".to_string(),
                severity: IssueSeverity::Medium,
                message: format!(
                    "Max complexity {} exceeds limit {}",
                    analysis.max_complexity, self.max_wasm_complexity
                ),
            });
        }

        // Check source map presence (if min coverage > 0)
        if self.min_source_map_coverage > 0.0 && !analysis.has_source_map {
            violations.push(QualityViolation {
                rule: "min_source_map_coverage".to_string(),
                severity: IssueSeverity::Low,
                message: "Source map is missing".to_string(),
            });
        }

        Ok(QualityGateResults {
            passed: violations.is_empty(),
            violations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gates_creation() {
        let gates = WasmQualityGates::new();
        assert_eq!(gates.max_module_size, 10_485_760);
    }

    #[test]
    fn test_quality_gates_default() {
        let gates = WasmQualityGates::default();
        assert_eq!(gates.zero_tolerance.len(), 6);
    }

    #[test]
    fn test_evaluate_passing_module() {
        let gates = WasmQualityGates::new();
        let analysis = WasmModuleAnalysis {
            module_size_bytes: 1000,
            function_count: 10,
            exported_functions: 5,
            max_complexity: 10,
            has_dwarf: false,
            has_source_map: true,
        };

        let result = gates.evaluate(&analysis);
        assert!(result.is_ok());
        let gate_results = result.unwrap();
        assert!(gate_results.passed);
        assert_eq!(gate_results.violations.len(), 0);
    }

    #[test]
    fn test_evaluate_oversized_module() {
        let gates = WasmQualityGates::new();
        let analysis = WasmModuleAnalysis {
            module_size_bytes: 20_000_000, // Exceeds 10MB
            function_count: 10,
            exported_functions: 5,
            max_complexity: 10,
            has_dwarf: false,
            has_source_map: true,
        };

        let result = gates.evaluate(&analysis);
        assert!(result.is_ok());
        let gate_results = result.unwrap();
        assert!(!gate_results.passed);
        assert!(!gate_results.violations.is_empty());
    }
}
