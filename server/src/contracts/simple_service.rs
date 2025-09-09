//! Simplified service layer that compiles and demonstrates uniform contracts
//! This is a minimal implementation to show the contract system working

use super::*;
use anyhow::Result;
use serde_json::Value;

/// Simplified service that implements all contracts
pub struct SimpleContractService;

impl SimpleContractService {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Process analyze complexity contract
    pub async fn analyze_complexity(&self, contract: AnalyzeComplexityContract) -> Result<Value> {
        contract.validate()?;

        let results = vec![ComplexityResult {
            file: contract.base.path.display().to_string(),
            cyclomatic: 5,
            cognitive: 3,
            halstead: 2.5,
            functions: vec![FunctionResult {
                name: "example_function".to_string(),
                cyclomatic: 3,
                cognitive: 2,
                line_start: 10,
                line_end: 25,
            }],
        }];

        Ok(serde_json::to_value(AnalysisResponse {
            results,
            summary: format!(
                "Analyzed {} with format {:?}",
                contract.base.path.display(),
                contract.base.format
            ),
            metadata: self.create_metadata(&contract.base),
        })?)
    }

    /// Process analyze SATD contract
    pub async fn analyze_satd(&self, contract: AnalyzeSatdContract) -> Result<Value> {
        contract.validate()?;

        let results = vec![SatdResult {
            file: contract.base.path.display().to_string(),
            comment: "// Sample technical debt comment".to_string(),
            line: 42,
            severity: SatdSeverity::Medium,
            debt_type: "SAMPLE".to_string(),
        }];

        Ok(serde_json::to_value(AnalysisResponse {
            results,
            summary: format!(
                "Found SATD in {} with strict mode: {}",
                contract.base.path.display(),
                contract.strict
            ),
            metadata: self.create_metadata(&contract.base),
        })?)
    }

    /// Process analyze dead code contract
    pub async fn analyze_dead_code(&self, contract: AnalyzeDeadCodeContract) -> Result<Value> {
        contract.validate()?;

        let results = vec![DeadCodeResult {
            file: contract.base.path.display().to_string(),
            dead_lines: 15,
            total_lines: 100,
            percentage: 15.0,
            unreachable_blocks: if contract.include_unreachable { 2 } else { 0 },
        }];

        Ok(serde_json::to_value(AnalysisResponse {
            results,
            summary: format!(
                "Dead code analysis for {} with threshold {}",
                contract.base.path.display(),
                contract.max_percentage
            ),
            metadata: self.create_metadata(&contract.base),
        })?)
    }

    /// Process analyze TDG contract
    pub async fn analyze_tdg(&self, contract: AnalyzeTdgContract) -> Result<Value> {
        contract.validate()?;

        let results = vec![TdgResult {
            file: contract.base.path.display().to_string(),
            score: 2.3,
            components: if contract.include_components {
                Some(TdgComponents {
                    complexity: 1.5,
                    churn: 0.8,
                    coverage: 0.0,
                })
            } else {
                None
            },
            grade: "B".to_string(),
        }];

        Ok(serde_json::to_value(AnalysisResponse {
            results,
            summary: format!(
                "TDG analysis for {} with threshold {}",
                contract.base.path.display(),
                contract.threshold
            ),
            metadata: self.create_metadata(&contract.base),
        })?)
    }

    /// Process analyze lint hotspot contract
    pub async fn analyze_lint_hotspot(
        &self,
        contract: AnalyzeLintHotspotContract,
    ) -> Result<Value> {
        contract.validate()?;

        let results = vec![LintHotspotResult {
            file: contract.base.path.display().to_string(),
            density: 3.2,
            violations: vec!["unused_variable".to_string(), "dead_code".to_string()],
            total_lines: 150,
            fixable: !contract.dry_run,
        }];

        Ok(serde_json::to_value(AnalysisResponse {
            results,
            summary: format!("Lint hotspot analysis for {}", contract.base.path.display()),
            metadata: self.create_metadata(&contract.base),
        })?)
    }

    /// Process analyze entropy contract
    pub async fn analyze_entropy(&self, contract: AnalyzeEntropyContract) -> Result<Value> {
        contract.validate()?;

        use crate::entropy::{EntropyAnalyzer, EntropyConfig};
        use crate::entropy::violation_detector::Severity;

        // Convert contract parameters to entropy config
        let mut config = EntropyConfig::default();
        
        if let Some(severity_str) = &contract.min_severity {
            config.min_severity = match severity_str.as_str() {
                "low" => Severity::Low,
                "medium" => Severity::Medium,
                "high" => Severity::High,
                _ => Severity::Medium,
            };
        }

        if !contract.base.include_tests {
            config.exclude_paths.push("**/*test*.rs".to_string());
            config.exclude_paths.push("tests/**".to_string());
        }

        // Create analyzer and run analysis
        let analyzer = EntropyAnalyzer::with_config(config);
        
        let analysis_path = if let Some(file_path) = &contract.file {
            file_path
        } else {
            &contract.base.path
        };

        let report = analyzer.analyze(analysis_path).await?;

        let total_violations = report.actionable_violations.len();
        let total_loc_reduction = report.total_loc_reduction();
        let reduction_percentage = report.reduction_percentage();

        // Limit violations if requested
        let violations = if let Some(limit) = contract.top_violations {
            if limit > 0 && report.actionable_violations.len() > limit {
                report.actionable_violations.into_iter().take(limit).collect()
            } else {
                report.actionable_violations
            }
        } else {
            report.actionable_violations
        };

        // Convert to the format expected by the contract system
        Ok(serde_json::to_value(EntropyResponse {
            violations,
            total_files_analyzed: report.total_files_analyzed,
            total_violations,
            potential_loc_reduction: total_loc_reduction,
            reduction_percentage,
            summary: format!(
                "Entropy analysis for {} found {} actionable violations with {:.1}% potential reduction",
                analysis_path.display(),
                total_violations,
                reduction_percentage
            ),
            metadata: self.create_metadata(&contract.base),
        })?)
    }

    /// Process quality gate contract
    pub async fn quality_gate(&self, contract: QualityGateContract) -> Result<Value> {
        contract.validate()?;

        let mut violations = Vec::new();

        // Check entropy violations if enabled
        if matches!(contract.profile, QualityProfile::Strict | QualityProfile::Extreme) {
            let entropy_contract = AnalyzeEntropyContract {
                base: contract.base.clone(),
                min_severity: Some("medium".to_string()),
                top_violations: Some(10),
                file: contract.file.clone(),
            };

            match self.analyze_entropy(entropy_contract).await {
                Ok(entropy_result) => {
                    if let Ok(entropy_response) = serde_json::from_value::<EntropyResponse>(entropy_result) {
                        // Convert high-severity entropy violations to quality gate violations
                        for violation in entropy_response.violations.iter().take(5) {
                            if matches!(violation.severity, crate::entropy::violation_detector::Severity::High) {
                                violations.push(QualityViolation {
                                    rule: format!("entropy_{:?}", violation.pattern.pattern_type),
                                    severity: ViolationSeverity::Warning,
                                    message: format!("High entropy pattern detected: {} ({} repetitions)", 
                                        violation.message, violation.pattern.repetitions),
                                    file: contract.base.path.display().to_string(),
                                    line: 1, // We don't have location info in PatternSummary yet
                                });
                            }
                        }
                    }
                }
                Err(_) => {
                    // Don't fail quality gate if entropy analysis fails
                    violations.push(QualityViolation {
                        rule: "entropy_analysis".to_string(),
                        severity: ViolationSeverity::Info,
                        message: "Entropy analysis could not be performed".to_string(),
                        file: contract.base.path.display().to_string(),
                        line: 1,
                    });
                }
            }
        }

        let passed = violations.is_empty();
        let violation_count = violations.len();

        if contract.fail_on_violation && !passed {
            return Err(anyhow::anyhow!(
                "Quality gate failed with {} violations",
                violation_count
            ));
        }

        Ok(serde_json::to_value(QualityGateResponse {
            passed,
            violations,
            profile: contract.profile,
            summary: format!(
                "Quality gate check for {} using {:?} profile - {} entropy violations found",
                contract.base.path.display(),
                contract.profile,
                violation_count
            ),
            metadata: self.create_metadata(&contract.base),
        })?)
    }

    /// Process refactor auto contract
    pub async fn refactor_auto(&self, contract: RefactorAutoContract) -> Result<Value> {
        contract.validate()?;

        let plan = RefactorPlan {
            file: contract.file.display().to_string(),
            current_complexity: 15,
            target_complexity: contract.target_complexity,
            operations: vec![RefactorOperation {
                operation_type: "extract_method".to_string(),
                description: "Extract method to reduce complexity".to_string(),
                line_start: 20,
                line_end: 35,
                confidence: 0.9,
            }],
            estimated_reduction: 7,
            applied: !contract.dry_run,
        };

        Ok(serde_json::to_value(RefactorResponse {
            plan,
            dry_run: contract.dry_run,
            summary: format!(
                "Refactor plan for {} targeting complexity {}",
                contract.file.display(),
                contract.target_complexity
            ),
        })?)
    }

    fn create_metadata(&self, base: &BaseAnalysisContract) -> AnalysisMetadata {
        AnalysisMetadata {
            path: base.path.display().to_string(),
            format: base.format,
            include_tests: base.include_tests,
            timeout: base.timeout,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

// Response types
#[derive(serde::Serialize)]
struct AnalysisResponse<T> {
    results: Vec<T>,
    summary: String,
    metadata: AnalysisMetadata,
}

#[derive(serde::Serialize)]
struct QualityGateResponse {
    passed: bool,
    violations: Vec<QualityViolation>,
    profile: QualityProfile,
    summary: String,
    metadata: AnalysisMetadata,
}

#[derive(serde::Serialize)]
struct RefactorResponse {
    plan: RefactorPlan,
    dry_run: bool,
    summary: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EntropyResponse {
    violations: Vec<crate::entropy::violation_detector::ActionableViolation>,
    total_files_analyzed: usize,
    total_violations: usize,
    potential_loc_reduction: usize,
    reduction_percentage: f64,
    summary: String,
    metadata: AnalysisMetadata,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AnalysisMetadata {
    path: String,
    format: OutputFormat,
    include_tests: bool,
    timeout: u64,
    timestamp: u64,
}

// Result types
#[derive(serde::Serialize)]
struct ComplexityResult {
    file: String,
    cyclomatic: u32,
    cognitive: u32,
    halstead: f64,
    functions: Vec<FunctionResult>,
}

#[derive(serde::Serialize)]
struct FunctionResult {
    name: String,
    cyclomatic: u32,
    cognitive: u32,
    line_start: u32,
    line_end: u32,
}

#[derive(serde::Serialize)]
struct SatdResult {
    file: String,
    comment: String,
    line: u32,
    severity: SatdSeverity,
    debt_type: String,
}

#[derive(serde::Serialize)]
struct DeadCodeResult {
    file: String,
    dead_lines: u32,
    total_lines: u32,
    percentage: f64,
    unreachable_blocks: u32,
}

#[derive(serde::Serialize)]
struct TdgResult {
    file: String,
    score: f64,
    components: Option<TdgComponents>,
    grade: String,
}

#[derive(serde::Serialize)]
struct TdgComponents {
    complexity: f64,
    churn: f64,
    coverage: f64,
}

#[derive(serde::Serialize)]
struct LintHotspotResult {
    file: String,
    density: f64,
    violations: Vec<String>,
    total_lines: u32,
    fixable: bool,
}

#[derive(serde::Serialize)]
struct QualityViolation {
    rule: String,
    severity: ViolationSeverity,
    message: String,
    file: String,
    line: u32,
}

#[derive(serde::Serialize)]
#[allow(dead_code)]
enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(serde::Serialize)]
struct RefactorPlan {
    file: String,
    current_complexity: u32,
    target_complexity: u32,
    operations: Vec<RefactorOperation>,
    estimated_reduction: u32,
    applied: bool,
}

#[derive(serde::Serialize)]
struct RefactorOperation {
    operation_type: String,
    description: String,
    line_start: u32,
    line_end: u32,
    confidence: f64,
}
