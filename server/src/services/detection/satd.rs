// Toyota Way: Unified SATD Detection Strategy

use super::{
    DetectionConfig, DetectionInput, DetectionOutput, Detector, DetectorCapabilities,
    DetectorSpecificConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// SATD detection strategy using the existing SATD detector
pub struct SATDDetector;

impl Default for SATDDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SATDDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Detector for SATDDetector {
    type Input = DetectionInput;
    type Output = DetectionOutput;
    type Config = DetectionConfig;

    async fn detect(&self, input: Self::Input, config: Self::Config) -> Result<Self::Output> {
        // Extract SATD-specific config
        let satd_config = match config.detector_specific {
            DetectorSpecificConfig::SATD(config) => config,
            _ => SATDConfig::default(),
        };

        // Delegate to the existing SATD detector functionality
        let result = match input {
            DetectionInput::SingleFile(path) => {
                self.detect_satd_in_file(&path, &satd_config).await?
            }
            DetectionInput::MultipleFiles(files) => {
                self.detect_satd_in_files(&files, &satd_config).await?
            }
            DetectionInput::ProjectDirectory(dir) => {
                self.detect_satd_in_directory(&dir, &satd_config).await?
            }
            DetectionInput::Content(content) => {
                self.detect_satd_in_content(&content, &satd_config)?
            }
        };

        Ok(DetectionOutput::SATD(result))
    }

    fn name(&self) -> &'static str {
        "satd"
    }

    fn capabilities(&self) -> DetectorCapabilities {
        DetectorCapabilities {
            supports_batch: true,
            supports_streaming: true,
            language_agnostic: true,
            requires_ast: false,
        }
    }
}

impl SATDDetector {
    async fn detect_satd_in_file(
        &self,
        file_path: &Path,
        _config: &SATDConfig,
    ) -> Result<SATDAnalysisResult> {
        // Delegate to the existing satd_detector module functionality
        let detector = crate::services::satd_detector::SATDDetector::new();

        let content = std::fs::read_to_string(file_path)?;
        let debt_items = detector
            .extract_from_content(&content, file_path)
            .map_err(|e| anyhow::anyhow!("SATD analysis failed: {}", e))?;

        self.create_analysis_result(vec![debt_items], 1)
    }

    async fn detect_satd_in_files(
        &self,
        files: &[std::path::PathBuf],
        _config: &SATDConfig,
    ) -> Result<SATDAnalysisResult> {
        let detector = crate::services::satd_detector::SATDDetector::new();

        let mut all_debt_items = Vec::new();
        let mut files_analyzed = 0;

        for file_path in files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if let Ok(debt_items) = detector.extract_from_content(&content, file_path) {
                    all_debt_items.push(debt_items);
                    files_analyzed += 1;
                }
            }
        }

        self.create_analysis_result(all_debt_items, files_analyzed)
    }

    async fn detect_satd_in_directory(
        &self,
        dir_path: &Path,
        _config: &SATDConfig,
    ) -> Result<SATDAnalysisResult> {
        // Scan directory for source files
        let files = self.scan_directory_for_source_files(dir_path)?;
        self.detect_satd_in_files(&files, _config).await
    }

    fn detect_satd_in_content(
        &self,
        content: &str,
        _config: &SATDConfig,
    ) -> Result<SATDAnalysisResult> {
        let detector = crate::services::satd_detector::SATDDetector::new();

        // Create a temporary path for content analysis
        let temp_path = std::path::Path::new("<content>");
        let debt_items = detector
            .extract_from_content(content, temp_path)
            .map_err(|e| anyhow::anyhow!("SATD analysis failed: {}", e))?;

        self.create_analysis_result(vec![debt_items], 1)
    }

    fn create_analysis_result(
        &self,
        debt_items_collections: Vec<Vec<crate::services::satd_detector::TechnicalDebt>>,
        files_analyzed: usize,
    ) -> Result<SATDAnalysisResult> {
        // Flatten all debt items
        let mut all_items = Vec::new();
        for collection in debt_items_collections {
            for debt_item in collection {
                all_items.push(TechnicalDebt {
                    category: match debt_item.category {
                        crate::services::satd_detector::DebtCategory::Design => {
                            DebtCategory::Design
                        }
                        crate::services::satd_detector::DebtCategory::Defect => {
                            DebtCategory::Implementation
                        }
                        crate::services::satd_detector::DebtCategory::Requirement => {
                            DebtCategory::Documentation
                        }
                        crate::services::satd_detector::DebtCategory::Test => DebtCategory::Testing,
                        crate::services::satd_detector::DebtCategory::Performance => {
                            DebtCategory::Performance
                        }
                        crate::services::satd_detector::DebtCategory::Security => {
                            DebtCategory::Security
                        }
                    },
                    severity: match debt_item.severity {
                        crate::services::satd_detector::Severity::Critical => Severity::Critical,
                        crate::services::satd_detector::Severity::High => Severity::High,
                        crate::services::satd_detector::Severity::Medium => Severity::Medium,
                        crate::services::satd_detector::Severity::Low => Severity::Low,
                    },
                    text: debt_item.text,
                    file: debt_item.file,
                    line: debt_item.line,
                    column: debt_item.column,
                    context_hash: debt_item.context_hash,
                });
            }
        }

        // Create summary statistics
        let summary = self.create_summary(&all_items, files_analyzed);

        let files_with_debt = summary.files_with_satd;
        Ok(SATDAnalysisResult {
            items: all_items,
            summary,
            total_files_analyzed: files_analyzed,
            files_with_debt,
            analysis_timestamp: chrono::Utc::now(),
        })
    }

    fn create_summary(&self, items: &[TechnicalDebt], _files_analyzed: usize) -> SATDSummary {
        let mut by_severity = std::collections::HashMap::new();
        let mut by_category = std::collections::HashMap::new();
        let mut files_with_debt = std::collections::HashSet::new();

        for item in items {
            // Count by severity
            let severity_str = match item.severity {
                Severity::Critical => "Critical",
                Severity::High => "High",
                Severity::Medium => "Medium",
                Severity::Low => "Low",
            };
            *by_severity.entry(severity_str.to_string()).or_insert(0) += 1;

            // Count by category
            let category_str = match item.category {
                DebtCategory::Implementation => "Implementation",
                DebtCategory::Design => "Design",
                DebtCategory::Documentation => "Documentation",
                DebtCategory::Testing => "Testing",
                DebtCategory::Performance => "Performance",
                DebtCategory::Security => "Security",
            };
            *by_category.entry(category_str.to_string()).or_insert(0) += 1;

            // Track files with debt
            files_with_debt.insert(item.file.clone());
        }

        SATDSummary {
            total_items: items.len(),
            by_severity,
            by_category,
            files_with_satd: files_with_debt.len(),
            avg_age_days: 0.0, // Placeholder - would need git history analysis
        }
    }

    fn scan_directory_for_source_files(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    // Check if it's a source file
                    if let Some(extension) = path.extension() {
                        if let Some(ext_str) = extension.to_str() {
                            if matches!(
                                ext_str,
                                "rs" | "ts"
                                    | "js"
                                    | "py"
                                    | "c"
                                    | "cpp"
                                    | "h"
                                    | "hpp"
                                    | "java"
                                    | "kt"
                                    | "go"
                            ) {
                                files.push(path);
                            }
                        }
                    }
                } else if path.is_dir()
                    && !path.file_name().unwrap().to_string_lossy().starts_with('.')
                {
                    // Recursively scan non-hidden subdirectories
                    let mut subdir_files = self.scan_directory_for_source_files(&path)?;
                    files.append(&mut subdir_files);
                }
            }
        }

        Ok(files)
    }
}

/// SATD detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDConfig {
    pub strict_mode: bool,
    pub include_low_severity: bool,
    pub custom_patterns: Vec<String>,
}

impl Default for SATDConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            include_low_severity: true,
            custom_patterns: Vec::new(),
        }
    }
}

/// SATD analysis result containing all detected debt items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDAnalysisResult {
    pub items: Vec<TechnicalDebt>,
    pub summary: SATDSummary,
    pub total_files_analyzed: usize,
    pub files_with_debt: usize,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detected technical debt item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub category: DebtCategory,
    pub severity: Severity,
    pub text: String,
    pub file: std::path::PathBuf,
    pub line: u32,
    pub column: u32,
    pub context_hash: [u8; 16], // BLAKE3 hash for identity tracking
}

/// Summary statistics for SATD analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDSummary {
    pub total_items: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub by_category: std::collections::HashMap<String, usize>,
    pub files_with_satd: usize,
    pub avg_age_days: f64,
}

/// Categories of technical debt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebtCategory {
    Implementation,
    Design,
    Documentation,
    Testing,
    Performance,
    Security,
}

/// Severity levels for technical debt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}
