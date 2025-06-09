use anyhow::Result;
use rustc_hash::FxHashMap;

use crate::services::deep_context::{DeepContextResult, FunctionComplexityForQA};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAVerificationResult {
    pub timestamp: String,
    pub version: String,
    pub dead_code: DeadCodeVerification,
    pub complexity: ComplexityVerification,
    pub provability: ProvabilityVerification,
    pub overall: VerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeVerification {
    pub status: VerificationStatus,
    pub expected_range: [f64; 2],
    pub actual: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityVerification {
    pub status: VerificationStatus,
    pub entropy: f64,
    pub cv: f64,
    pub p99: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvabilityVerification {
    pub status: VerificationStatus,
    pub pure_reducer_coverage: f64,
    pub state_invariants_tested: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum VerificationStatus {
    Pass,
    Partial,
    Fail,
}

type QualityCheck = Box<dyn Fn(&DeepContextResult) -> Result<(), String> + Send + Sync>;

pub struct QAVerification {
    checks: Vec<(&'static str, QualityCheck)>,
}

impl QAVerification {
    pub fn new() -> Self {
        let mut checks: Vec<(&'static str, QualityCheck)> = vec![];

        // Add all quality checks
        Self::add_dead_code_checks(&mut checks);
        Self::add_complexity_checks(&mut checks);
        Self::add_coverage_checks(&mut checks);
        Self::add_section_checks(&mut checks);

        Self { checks }
    }

    fn add_dead_code_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // Dead code sanity check
        checks.push(("dead_code_sanity", Box::new(|result| {
            let total_lines = result.complexity_metrics.as_ref()
                .map(|m| m.files.iter().map(|f| f.total_lines).sum::<usize>())
                .unwrap_or(0);

            if total_lines == 0 {
                return Err("No lines analyzed - invalid result".into());
            }

            let dead_lines = result.dead_code_analysis.as_ref()
                .map(|d| d.summary.total_dead_lines)
                .unwrap_or(0);

            let ratio = dead_lines as f64 / total_lines as f64;

            if ratio == 0.0 && total_lines > 1000 {
                // Check if this is legitimate
                let has_ffi_or_wasm = result.file_tree.iter()
                    .any(|path| path.contains("wasm") || path.contains("ffi") || path.contains("bindgen"));

                let has_typescript = result.language_stats.as_ref()
                    .and_then(|stats| stats.get("TypeScript"))
                    .map(|&count| count > 0)
                    .unwrap_or(false);

                let has_python = result.language_stats.as_ref()
                    .and_then(|stats| stats.get("Python"))
                    .map(|&count| count > 0)
                    .unwrap_or(false);

                if has_ffi_or_wasm {
                    Err("Zero dead code with FFI/WASM code present - likely false negative".into())
                } else if has_typescript || has_python {
                    Err("Mixed language project with zero dead code - verify cross-language tracing".into())
                } else {
                    // Pure Rust project in early stages might legitimately have no dead code
                    Ok(())
                }
            } else if ratio > 0.15 {
                Err(format!("Excessive dead code: {:.1}%", ratio * 100.0))
            } else {
                Ok(())
            }
        })));
    }

    fn add_complexity_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // Complexity distribution check
        checks.push((
            "complexity_distribution",
            Box::new(|result| {
                let functions: Vec<_> = result
                    .complexity_metrics
                    .as_ref()
                    .map(|m| m.files.iter().flat_map(|f| &f.functions).collect())
                    .unwrap_or_default();

                if functions.len() < 50 {
                    return Ok(()); // Too small for distribution analysis
                }

                // Calculate coefficient of variation
                let mean = functions.iter().map(|f| f.cyclomatic as f64).sum::<f64>()
                    / functions.len() as f64;

                if mean == 0.0 {
                    return Err("All functions have zero complexity - parser error".into());
                }

                let variance = functions
                    .iter()
                    .map(|f| (f.cyclomatic as f64 - mean).powi(2))
                    .sum::<f64>()
                    / functions.len() as f64;
                let cv = (variance.sqrt() / mean) * 100.0;

                if cv < 30.0 {
                    Err(format!(
                        "Low complexity variation (CV={cv:.1}%) - possible parser issue"
                    ))
                } else {
                    Ok(())
                }
            }),
        ));

        // Entropy check for large codebases
        checks.push((
            "complexity_entropy",
            Box::new(|result| {
                let functions: Vec<_> = result
                    .complexity_metrics
                    .as_ref()
                    .map(|m| m.files.iter().flat_map(|f| &f.functions).collect())
                    .unwrap_or_default();

                if functions.len() < 100 {
                    return Ok(()); // Too small for entropy analysis
                }

                let entropy = calculate_complexity_entropy(&functions);

                if entropy < 2.0 {
                    Err(format!(
                        "Low complexity entropy: {entropy:.2} (expected >= 2.0)"
                    ))
                } else {
                    Ok(())
                }
            }),
        ));
    }

    fn add_coverage_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // AST coverage check
        checks.push((
            "ast_coverage",
            Box::new(|result| {
                let total_files = result.file_tree.len();
                let ast_files = result
                    .ast_summaries
                    .as_ref()
                    .map(|summaries| summaries.len())
                    .unwrap_or(0);

                if total_files == 0 {
                    return Err("No files found in project".into());
                }

                let coverage = ast_files as f64 / total_files as f64;

                if coverage < 0.5 {
                    Err(format!(
                        "Low AST coverage: {:.1}% (expected >= 50%)",
                        coverage * 100.0
                    ))
                } else {
                    Ok(())
                }
            }),
        ));
    }

    fn add_section_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // Empty sections check
        checks.push((
            "empty_sections",
            Box::new(|result| {
                let mut empty_sections = Vec::new();

                if result.dead_code_analysis.is_none() {
                    empty_sections.push("dead_code_analysis");
                }

                if result.complexity_metrics.is_none() {
                    empty_sections.push("complexity_metrics");
                }

                if result.ast_summaries.is_none() {
                    empty_sections.push("ast_summaries");
                }

                if result.churn_analysis.is_none() {
                    empty_sections.push("churn_analysis");
                }

                if !empty_sections.is_empty() {
                    Err(format!(
                        "Empty sections found: {}",
                        empty_sections.join(", ")
                    ))
                } else {
                    Ok(())
                }
            }),
        ));
    }

    pub fn verify(
        &self,
        result: &DeepContextResult,
    ) -> FxHashMap<&'static str, Result<(), String>> {
        self.checks
            .iter()
            .map(|(name, check)| (*name, check(result)))
            .collect()
    }

    pub fn generate_verification_report(&self, result: &DeepContextResult) -> QAVerificationResult {
        let verification_results = self.verify(result);

        // Calculate dead code metrics
        let total_lines = result
            .complexity_metrics
            .as_ref()
            .map(|m| m.files.iter().map(|f| f.total_lines).sum::<usize>())
            .unwrap_or(0);

        let dead_lines = result
            .dead_code_analysis
            .as_ref()
            .map(|d| d.summary.total_dead_lines)
            .unwrap_or(0);

        let dead_ratio = if total_lines > 0 {
            dead_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        // Calculate complexity metrics
        let functions: Vec<_> = result
            .complexity_metrics
            .as_ref()
            .map(|m| m.files.iter().flat_map(|f| &f.functions).collect())
            .unwrap_or_default();

        let (entropy, cv, p99) = if !functions.is_empty() {
            let entropy = calculate_complexity_entropy(&functions);

            let mean =
                functions.iter().map(|f| f.cyclomatic as f64).sum::<f64>() / functions.len() as f64;
            let variance = functions
                .iter()
                .map(|f| (f.cyclomatic as f64 - mean).powi(2))
                .sum::<f64>()
                / functions.len() as f64;
            let cv = if mean > 0.0 {
                (variance.sqrt() / mean) * 100.0
            } else {
                0.0
            };

            let mut complexities: Vec<_> = functions.iter().map(|f| f.cyclomatic).collect();
            complexities.sort_unstable();
            let p99 = complexities
                .get(complexities.len() * 99 / 100)
                .copied()
                .unwrap_or(0);

            (entropy, cv, p99)
        } else {
            (0.0, 0.0, 0)
        };

        // Determine statuses
        let dead_code_status = match verification_results.get("dead_code_sanity") {
            Some(Ok(_)) => VerificationStatus::Pass,
            Some(Err(msg)) if msg.contains("Mixed language") => VerificationStatus::Partial,
            _ => VerificationStatus::Fail,
        };

        let complexity_status = if verification_results
            .get("complexity_distribution")
            .map(|r| r.is_ok())
            .unwrap_or(false)
            && verification_results
                .get("complexity_entropy")
                .map(|r| r.is_ok())
                .unwrap_or(false)
        {
            VerificationStatus::Pass
        } else {
            VerificationStatus::Fail
        };

        let overall_status = if dead_code_status == VerificationStatus::Pass
            && complexity_status == VerificationStatus::Pass
        {
            VerificationStatus::Pass
        } else if dead_code_status == VerificationStatus::Fail
            || complexity_status == VerificationStatus::Fail
        {
            VerificationStatus::Fail
        } else {
            VerificationStatus::Partial
        };

        QAVerificationResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            dead_code: DeadCodeVerification {
                status: dead_code_status,
                expected_range: [0.005, 0.15],
                actual: dead_ratio,
                notes: verification_results
                    .get("dead_code_sanity")
                    .and_then(|r| r.as_ref().err().map(|s| s.to_string())),
            },
            complexity: ComplexityVerification {
                status: complexity_status,
                entropy,
                cv,
                p99,
                notes: None,
            },
            provability: ProvabilityVerification {
                status: VerificationStatus::Partial,
                pure_reducer_coverage: 0.82, // Placeholder - would need actual coverage data
                state_invariants_tested: 4,
                notes: Some("Provability verification not yet implemented".to_string()),
            },
            overall: overall_status,
        }
    }
}

fn calculate_complexity_entropy(functions: &[&FunctionComplexityForQA]) -> f64 {
    let mut freq_map = FxHashMap::default();
    for func in functions {
        *freq_map.entry(func.cyclomatic).or_insert(0) += 1;
    }

    let total = functions.len() as f64;
    freq_map
        .values()
        .map(|&count| {
            let p = count as f64 / total;
            -p * p.log2()
        })
        .sum()
}

impl Default for QAVerification {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::deep_context::{
        AnalysisResults, CacheStats, ComplexityMetricsForQA, ContextMetadata, DeadCodeAnalysis,
        DeadCodeSummary, DeepContextResult, DefectSummary, FileComplexityMetricsForQA,
        FunctionComplexityForQA, QualityScorecard,
    };
    use std::path::PathBuf;
    use std::time::Duration;

    // Helper function to create a test DeepContextResult
    fn create_test_deep_context_result() -> DeepContextResult {
        DeepContextResult {
            metadata: ContextMetadata {
                generated_at: chrono::Utc::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                project_root: PathBuf::from("."),
                cache_stats: CacheStats {
                    hit_rate: 0.8,
                    memory_efficiency: 0.9,
                    time_saved_ms: 1000,
                },
                analysis_duration: Duration::from_secs(1),
            },
            file_tree: vec![],
            analyses: AnalysisResults {
                ast_contexts: vec![],
                complexity_report: None,
                churn_analysis: None,
                dependency_graph: None,
                dead_code_results: None,
                satd_results: None,
                duplicate_code_results: None,
                provability_results: None,
                cross_language_refs: vec![],
                big_o_analysis: None,
            },
            quality_scorecard: QualityScorecard {
                overall_health: 0.8,
                complexity_score: 0.8,
                maintainability_index: 0.8,
                modularity_score: 0.8,
                test_coverage: Some(0.8),
                technical_debt_hours: 10.0,
            },
            template_provenance: None,
            defect_summary: DefectSummary {
                total_defects: 0,
                by_severity: FxHashMap::default(),
                by_type: FxHashMap::default(),
                defect_density: 0.0,
            },
            hotspots: vec![],
            recommendations: vec![],
            qa_verification: None,
            complexity_metrics: None,
            dead_code_analysis: None,
            ast_summaries: None,
            churn_analysis: None,
            language_stats: None,
            build_info: None,
            project_overview: None,
        }
    }

    #[test]
    fn test_qa_verification_dead_code() {
        let qa = QAVerification::new();

        // Test zero dead code with large codebase
        let mut result = create_test_deep_context_result();
        result.complexity_metrics = Some(ComplexityMetricsForQA {
            files: vec![FileComplexityMetricsForQA {
                path: PathBuf::from("src/main.rs"),
                functions: vec![],
                total_cyclomatic: 100,
                total_cognitive: 150,
                total_lines: 2000,
            }],
            summary: Default::default(),
        });

        result.dead_code_analysis = Some(DeadCodeAnalysis {
            summary: DeadCodeSummary {
                total_functions: 50,
                dead_functions: 0,
                total_lines: 2000,
                total_dead_lines: 0,
                dead_percentage: 0.0,
            },
            dead_functions: vec![],
            warnings: vec![],
        });

        let verification = qa.verify(&result);
        assert!(
            verification.get("dead_code_sanity").unwrap().is_ok(),
            "Pure Rust project with zero dead code should pass"
        );
    }

    #[test]
    fn test_qa_verification_complexity() {
        let qa = QAVerification::new();

        let mut result = create_test_deep_context_result();

        // Generate varied complexity distribution
        let functions: Vec<FunctionComplexityForQA> = (0..100)
            .map(|i| {
                FunctionComplexityForQA {
                    name: format!("func_{i}"),
                    cyclomatic: ((i % 20) + 1) as u32, // Varied complexity
                    cognitive: ((i % 15) + 1) as u32,
                    nesting_depth: ((i % 5) + 1) as u32,
                    start_line: (i * 10) as usize,
                    end_line: (i * 10 + 5) as usize,
                }
            })
            .collect();

        result.complexity_metrics = Some(ComplexityMetricsForQA {
            files: vec![FileComplexityMetricsForQA {
                path: PathBuf::from("src/lib.rs"),
                functions,
                total_cyclomatic: 1050,
                total_cognitive: 800,
                total_lines: 1500,
            }],
            summary: Default::default(),
        });

        let verification = qa.verify(&result);
        assert!(verification.get("complexity_distribution").unwrap().is_ok());
        assert!(verification.get("complexity_entropy").unwrap().is_ok());
    }

    #[test]
    fn test_qa_report_generation() {
        let qa = QAVerification::new();
        let result = create_test_deep_context_result();

        let report = qa.generate_verification_report(&result);

        assert!(!report.timestamp.is_empty());
        assert_eq!(report.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(report.dead_code.expected_range, [0.005, 0.15]);
    }
}
