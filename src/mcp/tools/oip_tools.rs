#![cfg_attr(coverage_nightly, coverage(off))]
/// Organizational Intelligence Plugin (OIP) MCP Tools
///
/// Provides MCP tools for defect pattern analysis and context-aware prompt generation.
/// Requires the `org-intelligence` feature to be enabled.
///
/// See: ../organizational-intelligence-plugin for the underlying implementation
use crate::prompts::defect_aware_prompts::DefectAwarePromptGenerator;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Request to generate context-aware prompts from OIP analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDefectAwarePromptRequest {
    /// Path to OIP summary file (from `oip summarize`)
    pub summary_path: String,
    /// Task description
    pub task: String,
    /// Additional context
    #[serde(default)]
    pub context: String,
}

/// Response containing the generated prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDefectAwarePromptResponse {
    /// The generated context-aware prompt
    pub prompt: String,
    /// Number of defect patterns included
    pub patterns_included: usize,
    /// Analysis date from the summary
    pub analysis_date: String,
}

/// Request to generate prevention-focused prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePreventionPromptRequest {
    /// Path to OIP summary file
    pub summary_path: String,
    /// Defect category to focus on (e.g., "ConfigurationErrors")
    pub defect_category: String,
}

/// Response containing the prevention prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePreventionPromptResponse {
    /// The generated prevention-focused prompt
    pub prompt: String,
    /// Whether the category was found
    pub found: bool,
}

/// Request to analyze OIP summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeOipSummaryRequest {
    /// Path to OIP summary file
    pub summary_path: String,
}

/// Response with OIP summary analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeOipSummaryResponse {
    /// Number of repositories analyzed
    pub repositories_analyzed: usize,
    /// Number of commits analyzed
    pub commits_analyzed: usize,
    /// Analysis date
    pub analysis_date: String,
    /// Top defect categories
    pub top_defects: Vec<DefectSummary>,
    /// Quality thresholds
    pub quality_thresholds: QualityThresholds,
}

/// Defect summary for response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectSummary {
    pub category: String,
    pub frequency: usize,
    pub avg_tdg_score: Option<f32>,
}

/// Quality thresholds summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub tdg_minimum: f32,
    pub test_coverage_minimum: f32,
    pub max_function_length: usize,
    pub max_cyclomatic_complexity: usize,
}

/// Generate context-aware prompt from OIP analysis
///
/// # Arguments
///
/// * `request` - Request containing summary path, task, and context
///
/// # Returns
///
/// A comprehensive prompt with organizational learnings
///
/// # Example
///
/// ```no_run
/// use pmat::mcp::tools::oip_tools::{generate_defect_aware_prompt, GenerateDefectAwarePromptRequest};
///
/// let req = GenerateDefectAwarePromptRequest {
///     summary_path: "baseline-summary.yaml".to_string(),
///     task: "Implement configuration parser".to_string(),
///     context: "Parse YAML files with validation".to_string(),
/// };
///
/// let response = generate_defect_aware_prompt(req)?;
/// println!("{}", response.prompt);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn generate_defect_aware_prompt(
    request: GenerateDefectAwarePromptRequest,
) -> Result<GenerateDefectAwarePromptResponse> {
    let generator = DefectAwarePromptGenerator::from_file(&request.summary_path)
        .context("Failed to load OIP summary")?;

    let prompt = generator.generate_prompt(&request.task, &request.context);

    let patterns_included = generator
        .defect_patterns
        .iter()
        .filter(|p| p.frequency >= 10)
        .count();

    Ok(GenerateDefectAwarePromptResponse {
        prompt,
        patterns_included,
        analysis_date: generator.metadata.analysis_date.clone(),
    })
}

/// Generate prevention-focused prompt for a specific defect category
///
/// # Arguments
///
/// * `request` - Request containing summary path and defect category
///
/// # Returns
///
/// A focused prompt for preventing the specified defect category
pub fn generate_prevention_prompt(
    request: GeneratePreventionPromptRequest,
) -> Result<GeneratePreventionPromptResponse> {
    let generator = DefectAwarePromptGenerator::from_file(&request.summary_path)
        .context("Failed to load OIP summary")?;

    let prompt_opt = generator.generate_prevention_prompt(&request.defect_category);
    let found = prompt_opt.is_some();

    Ok(GeneratePreventionPromptResponse {
        prompt: prompt_opt.unwrap_or_else(|| {
            format!(
                "Defect category '{}' not found in analysis",
                request.defect_category
            )
        }),
        found,
    })
}

/// Analyze OIP summary and return key metrics
///
/// # Arguments
///
/// * `request` - Request containing summary path
///
/// # Returns
///
/// Summary metrics including top defects and quality thresholds
pub fn analyze_oip_summary(request: AnalyzeOipSummaryRequest) -> Result<AnalyzeOipSummaryResponse> {
    let generator = DefectAwarePromptGenerator::from_file(&request.summary_path)
        .context("Failed to load OIP summary")?;

    let top_defects: Vec<DefectSummary> = generator
        .defect_patterns
        .iter()
        .map(|p| DefectSummary {
            category: p.category.clone(),
            frequency: p.frequency,
            avg_tdg_score: p.quality_signals.avg_tdg_score,
        })
        .collect();

    let quality_thresholds = QualityThresholds {
        tdg_minimum: generator.quality_thresholds.tdg_minimum,
        test_coverage_minimum: generator.quality_thresholds.test_coverage_minimum,
        max_function_length: generator.quality_thresholds.max_function_length,
        max_cyclomatic_complexity: generator.quality_thresholds.max_cyclomatic_complexity,
    };

    Ok(AnalyzeOipSummaryResponse {
        repositories_analyzed: generator.metadata.repositories_analyzed,
        commits_analyzed: generator.metadata.commits_analyzed,
        analysis_date: generator.metadata.analysis_date.clone(),
        top_defects,
        quality_thresholds,
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::defect_aware_prompts::{
        AnalysisMetadata, DefectPattern, OipSummary, OrganizationalInsights, QualitySignals,
        QualityThresholds as PromptQualityThresholds,
    };
    use tempfile::NamedTempFile;

    fn create_test_summary_file() -> Result<NamedTempFile> {
        let summary = OipSummary {
            organizational_insights: OrganizationalInsights {
                top_defect_categories: vec![DefectPattern {
                    category: "ConfigurationErrors".to_string(),
                    frequency: 25,
                    confidence: 0.78,
                    quality_signals: QualitySignals {
                        avg_tdg_score: Some(45.2),
                        max_tdg_score: Some(98.0),
                        avg_complexity: Some(8.5),
                        avg_test_coverage: Some(0.58),
                        satd_instances: 12,
                        avg_lines_changed: 45.2,
                        avg_files_per_commit: 2.1,
                    },
                    examples: vec![],
                }],
            },
            code_quality_thresholds: PromptQualityThresholds {
                tdg_minimum: 85.0,
                test_coverage_minimum: 0.85,
                max_function_length: 50,
                max_cyclomatic_complexity: 10,
            },
            metadata: AnalysisMetadata {
                analysis_date: "2025-11-15".to_string(),
                repositories_analyzed: 25,
                commits_analyzed: 2500,
            },
        };

        let file = NamedTempFile::new()?;
        serde_yaml_ng::to_writer(&file, &summary)?;
        Ok(file)
    }

    #[test]
    fn test_generate_defect_aware_prompt() -> Result<()> {
        let file = create_test_summary_file()?;

        let request = GenerateDefectAwarePromptRequest {
            summary_path: file.path().to_string_lossy().to_string(),
            task: "Test task".to_string(),
            context: "Test context".to_string(),
        };

        let response = generate_defect_aware_prompt(request)?;

        assert!(response.prompt.contains("# Task"));
        assert!(response.prompt.contains("Test task"));
        assert!(response.prompt.contains("ConfigurationErrors"));
        assert_eq!(response.patterns_included, 1);
        assert_eq!(response.analysis_date, "2025-11-15");

        Ok(())
    }

    #[test]
    fn test_generate_prevention_prompt() -> Result<()> {
        let file = create_test_summary_file()?;

        let request = GeneratePreventionPromptRequest {
            summary_path: file.path().to_string_lossy().to_string(),
            defect_category: "ConfigurationErrors".to_string(),
        };

        let response = generate_prevention_prompt(request)?;

        assert!(response.found);
        assert!(response.prompt.contains("ConfigurationErrors"));
        assert!(response
            .prompt
            .contains("Historical Frequency: 25 occurrences"));

        Ok(())
    }

    #[test]
    fn test_analyze_oip_summary() -> Result<()> {
        let file = create_test_summary_file()?;

        let request = AnalyzeOipSummaryRequest {
            summary_path: file.path().to_string_lossy().to_string(),
        };

        let response = analyze_oip_summary(request)?;

        assert_eq!(response.repositories_analyzed, 25);
        assert_eq!(response.commits_analyzed, 2500);
        assert_eq!(response.analysis_date, "2025-11-15");
        assert_eq!(response.top_defects.len(), 1);
        assert_eq!(response.top_defects[0].category, "ConfigurationErrors");
        assert_eq!(response.quality_thresholds.tdg_minimum, 85.0);

        Ok(())
    }
}
