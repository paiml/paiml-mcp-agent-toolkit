//! Work Contract: Popperian Falsification-Based Quality Enforcement
//!
//! Every claim made by `pmat work complete` must be falsifiable.
//! If ANY claim cannot be verified, work is BLOCKED.
//!
//! Based on: docs/specifications/improve-pmat-work.md

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Popperian Work Contract
///
/// Every claim made by `pmat work complete` must be falsifiable.
/// If ANY claim cannot be verified, work is BLOCKED.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContract {
    /// Work item ID
    pub work_item_id: String,

    /// Contract creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    // === BASELINE (captured at work start, immutable via git) ===
    /// Git SHA of baseline commit (tamper-proof)
    pub baseline_commit: String,

    /// TDG score at baseline
    pub baseline_tdg: f64,

    /// Coverage percentage at baseline
    pub baseline_coverage: f64,

    /// Rust project score at baseline (if Rust project)
    pub baseline_rust_score: Option<f64>,

    /// File manifest for anti-gaming detection
    pub baseline_file_manifest: FileManifest,

    // === THRESHOLDS ===
    /// Quality thresholds for this contract
    pub thresholds: ContractThresholds,

    // === FALSIFICATION CLAIMS ===
    /// Claims that must survive falsification
    pub claims: Vec<FalsifiableClaim>,
}

impl WorkContract {
    /// Create a new work contract with baseline capture
    pub fn new(work_item_id: String, baseline_commit: String) -> Self {
        Self {
            work_item_id,
            created_at: chrono::Utc::now(),
            baseline_commit,
            baseline_tdg: 0.0,
            baseline_coverage: 0.0,
            baseline_rust_score: None,
            baseline_file_manifest: FileManifest::default(),
            thresholds: ContractThresholds::default(),
            claims: Self::default_claims(),
        }
    }

    /// Generate default falsifiable claims
    fn default_claims() -> Vec<FalsifiableClaim> {
        vec![
            FalsifiableClaim {
                hypothesis: "All baseline files still exist".to_string(),
                falsification_method: FalsificationMethod::ManifestIntegrity,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "The falsifier is active and detecting (Meta-Check)".to_string(),
                falsification_method: FalsificationMethod::MetaFalsification,
                evidence_required: EvidenceType::CounterExample {
                    details: "".into(),
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No coverage exclusion gaming".to_string(),
                falsification_method: FalsificationMethod::CoverageGaming,
                evidence_required: EvidenceType::FileList(vec![]),
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "All changed lines are covered".to_string(),
                falsification_method: FalsificationMethod::DifferentialCoverage,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 100.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "Total coverage >= 95%".to_string(),
                falsification_method: FalsificationMethod::AbsoluteCoverage,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 95.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "TDG score >= baseline".to_string(),
                falsification_method: FalsificationMethod::TdgRegression,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 0.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No function exceeds complexity 20".to_string(),
                falsification_method: FalsificationMethod::ComplexityRegression,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 20.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No vulnerable dependencies added".to_string(),
                falsification_method: FalsificationMethod::SupplyChainIntegrity,
                evidence_required: EvidenceType::CounterExample {
                    details: "".into(),
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "No file exceeds 500 lines".to_string(),
                falsification_method: FalsificationMethod::FileSizeRegression,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 500.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "Spec & Roadmap Quality".to_string(),
                falsification_method: FalsificationMethod::SpecQuality,
                evidence_required: EvidenceType::NumericComparison {
                    actual: 0.0,
                    threshold: 95.0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "All changes pushed".to_string(),
                falsification_method: FalsificationMethod::GitHubSync,
                evidence_required: EvidenceType::GitState {
                    unpushed_commits: 0,
                    dirty_files: 0,
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "All examples compile and run".to_string(),
                falsification_method: FalsificationMethod::ExamplesCompile,
                evidence_required: EvidenceType::CounterExample {
                    details: "".into(),
                },
                result: None,
                override_info: None,
            },
            FalsifiableClaim {
                hypothesis: "pmat-book validation passes".to_string(),
                falsification_method: FalsificationMethod::BookValidation,
                evidence_required: EvidenceType::CounterExample {
                    details: "".into(),
                },
                result: None,
                override_info: None,
            },
        ]
    }

    /// Load contract from file
    pub fn load(project_path: &Path, work_item_id: &str) -> Result<Self> {
        let contract_path = Self::contract_path(project_path, work_item_id);
        let content = std::fs::read_to_string(&contract_path)
            .with_context(|| format!("Failed to load contract from {}", contract_path.display()))?;
        serde_json::from_str(&content).context("Failed to parse contract JSON")
    }

    /// Save contract to file
    pub fn save(&self, project_path: &Path) -> Result<PathBuf> {
        let contract_dir = project_path.join(".pmat-work").join(&self.work_item_id);
        std::fs::create_dir_all(&contract_dir)?;

        let contract_path = contract_dir.join("contract.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&contract_path, json)?;

        Ok(contract_path)
    }

    /// Get contract path for a work item
    pub fn contract_path(project_path: &Path, work_item_id: &str) -> PathBuf {
        project_path
            .join(".pmat-work")
            .join(work_item_id)
            .join("contract.json")
    }

    /// Check if contract exists for work item
    pub fn exists(project_path: &Path, work_item_id: &str) -> bool {
        Self::contract_path(project_path, work_item_id).exists()
    }

    /// Acknowledge legacy debt by comparing current metrics against thresholds
    ///
    /// For projects that don't meet strict Popperian thresholds (95% coverage, etc.),
    /// this method creates tracked debt tickets and adds overrides to claims.
    ///
    /// This is the "managed migration path" for existing projects.
    pub fn acknowledge_legacy_debt(&mut self, project_path: &Path) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let mut debt_items: Vec<DebtItem> = Vec::new();

        // Check coverage against 95% threshold
        if self.baseline_coverage < self.thresholds.min_coverage_pct {
            let ticket_id = format!("DEBT-COV-{}", timestamp);
            debt_items.push(DebtItem {
                ticket_id: ticket_id.clone(),
                category: "coverage".to_string(),
                description: format!(
                    "Coverage {:.1}% is below required {:.1}%",
                    self.baseline_coverage, self.thresholds.min_coverage_pct
                ),
                current_value: self.baseline_coverage,
                required_value: self.thresholds.min_coverage_pct,
            });

            // Add override to AbsoluteCoverage claim
            for claim in &mut self.claims {
                if claim.falsification_method == FalsificationMethod::AbsoluteCoverage {
                    claim.override_info = Some(OverrideInfo {
                        reason: "Legacy Debt: Project predates strict coverage requirements".to_string(),
                        ticket_id: ticket_id.clone(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }

        // Check TDG baseline (if we have one, treat 0 as "no data yet")
        if self.baseline_tdg > 0.0 {
            // TDG already captured - no debt for this
        }

        // Check for large files in the manifest
        let large_files: Vec<_> = self
            .baseline_file_manifest
            .files
            .iter()
            .filter(|(_, entry)| entry.lines > self.thresholds.max_file_lines)
            .map(|(path, entry)| (path.clone(), entry.lines))
            .collect();

        if !large_files.is_empty() {
            let ticket_id = format!("DEBT-SIZE-{}", timestamp);
            let details: Vec<String> = large_files
                .iter()
                .take(10)
                .map(|(p, lines)| format!("{}: {} lines", p.display(), lines))
                .collect();

            debt_items.push(DebtItem {
                ticket_id: ticket_id.clone(),
                category: "file_size".to_string(),
                description: format!(
                    "{} file(s) exceed {} line limit: {}",
                    large_files.len(),
                    self.thresholds.max_file_lines,
                    details.join(", ")
                ),
                current_value: large_files.len() as f64,
                required_value: 0.0,
            });

            // Add override to FileSizeRegression claim
            for claim in &mut self.claims {
                if claim.falsification_method == FalsificationMethod::FileSizeRegression {
                    claim.override_info = Some(OverrideInfo {
                        reason: "Legacy Debt: Large files predate strict size requirements".to_string(),
                        ticket_id: ticket_id.clone(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }

        // Create debt tickets directory and write YAML files
        if !debt_items.is_empty() {
            let tickets_dir = project_path.join(".pmat-tickets");
            std::fs::create_dir_all(&tickets_dir)?;

            for item in &debt_items {
                let ticket_path = tickets_dir.join(format!("{}.yaml", item.ticket_id));
                let yaml_content = format!(
                    r#"# PMAT Legacy Debt Ticket
# Auto-generated by `pmat comply upgrade`
# DO NOT DELETE - Required for Popperian falsification override

ticket_id: "{}"
category: "{}"
created_at: "{}"
status: "open"

description: |
  {}

metrics:
  current: {:.2}
  required: {:.2}
  gap: {:.2}

resolution:
  # Update this section when addressing the debt
  plan: "TBD"
  target_date: null
  completed_at: null
"#,
                    item.ticket_id,
                    item.category,
                    chrono::Utc::now().to_rfc3339(),
                    item.description,
                    item.current_value,
                    item.required_value,
                    item.required_value - item.current_value,
                );

                std::fs::write(&ticket_path, yaml_content)?;
                println!(
                    "   📝 Created debt ticket: {}",
                    ticket_path.display()
                );
            }
        }

        Ok(())
    }
}

/// Internal struct for tracking debt items
#[derive(Debug)]
struct DebtItem {
    ticket_id: String,
    category: String,
    description: String,
    current_value: f64,
    required_value: f64,
}

/// Contract thresholds (non-negotiable defaults)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractThresholds {
    /// Minimum total coverage (absolute, not relative)
    pub min_coverage_pct: f64,

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
}

impl Default for ContractThresholds {
    fn default() -> Self {
        Self {
            min_coverage_pct: 95.0,
            max_tdg_regression: 0.0,
            max_function_complexity: 20,
            max_file_lines: 500,
            min_spec_score: 95,
            require_github_sync: true,
            require_spec_update: true,
            require_roadmap_update: true,
        }
    }
}

/// Immutable file manifest captured at work start
///
/// Detects file hiding/exclusion gaming by tracking ALL source files.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileManifest {
    /// All source files with metadata
    pub files: HashMap<PathBuf, FileEntry>,

    /// Files that MUST be included in coverage (no exclusion allowed)
    pub coverage_required: Vec<PathBuf>,

    /// Checksum of entire manifest (tamper detection)
    pub manifest_hash: String,
}

impl FileManifest {
    /// Build manifest from project directory
    pub fn build(project_path: &Path) -> Result<Self> {
        let mut files = HashMap::new();
        let mut coverage_required = Vec::new();

        // Walk source directories
        for entry in walkdir::WalkDir::new(project_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !Self::is_excluded(e.path()))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_entry) = FileEntry::from_path(path)? {
                    let rel_path = path
                        .strip_prefix(project_path)
                        .unwrap_or(path)
                        .to_path_buf();

                    // Mark CUDA/AVX files as coverage-required
                    if matches!(
                        file_entry.category,
                        FileCategory::CudaKernel | FileCategory::SimdAvx | FileCategory::RustSource
                    ) {
                        coverage_required.push(rel_path.clone());
                    }

                    files.insert(rel_path, file_entry);
                }
            }
        }

        // Compute manifest hash
        let mut hasher = Sha256::new();
        let mut sorted_paths: Vec<_> = files.keys().collect();
        sorted_paths.sort();
        for path in sorted_paths {
            hasher.update(path.to_string_lossy().as_bytes());
            if let Some(entry) = files.get(path) {
                hasher.update(entry.content_hash.as_bytes());
            }
        }
        let manifest_hash = format!("{:x}", hasher.finalize());

        Ok(Self {
            files,
            coverage_required,
            manifest_hash,
        })
    }

    /// Check if path should be excluded from manifest
    fn is_excluded(path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Exclude common non-source directories
        path_str.contains("/target/")
            || path_str.contains("/.git/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/.pmat-")
            || path_str.ends_with(".lock")
    }

    /// Verify manifest integrity (find missing files)
    pub fn verify_integrity(&self, project_path: &Path) -> Vec<PathBuf> {
        let mut missing = Vec::new();

        for (rel_path, _entry) in &self.files {
            let full_path = project_path.join(rel_path);
            if !full_path.exists() {
                missing.push(rel_path.clone());
            }
        }

        missing
    }
}

/// File entry in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// SHA256 of file content at baseline
    pub content_hash: String,

    /// Line count at baseline
    pub lines: usize,

    /// Function count at baseline (approximate)
    pub functions: usize,

    /// Maximum complexity at baseline
    pub max_complexity: u32,

    /// File category for coverage requirements
    pub category: FileCategory,
}

impl FileEntry {
    /// Create file entry from path
    pub fn from_path(path: &Path) -> Result<Option<Self>> {
        let extension = path.extension().and_then(|e| e.to_str());

        // Only process source files
        let category = match extension {
            Some("rs") => Self::categorize_rust_file(path)?,
            Some("cu" | "cuh") => FileCategory::CudaKernel,
            Some("c" | "cpp" | "cc" | "h" | "hpp") => FileCategory::CSource,
            Some("py") => FileCategory::PythonSource,
            Some("ts" | "tsx" | "js" | "jsx") => FileCategory::TypeScriptSource,
            Some("go") => FileCategory::GoSource,
            _ => return Ok(None),
        };

        let content = std::fs::read_to_string(path)?;
        let lines = content.lines().count();

        // Compute content hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());

        // Approximate function count (simple heuristic)
        let functions = Self::count_functions(&content, &category);

        Ok(Some(Self {
            content_hash,
            lines,
            functions,
            max_complexity: 0, // Will be computed by complexity analyzer
            category,
        }))
    }

    /// Categorize Rust file (detect SIMD, tests, etc.)
    fn categorize_rust_file(path: &Path) -> Result<FileCategory> {
        let content = std::fs::read_to_string(path)?;

        // Check for test files
        let path_str = path.to_string_lossy();
        if path_str.contains("/tests/")
            || path_str.contains("_test.rs")
            || path_str.ends_with("tests.rs")
        {
            return Ok(FileCategory::TestCode);
        }

        // Check for build scripts
        if path_str.ends_with("build.rs") {
            return Ok(FileCategory::BuildScript);
        }

        // Check for SIMD patterns
        if Self::contains_simd_patterns(&content) {
            return Ok(FileCategory::SimdAvx);
        }

        Ok(FileCategory::RustSource)
    }

    /// Check if content contains SIMD patterns
    fn contains_simd_patterns(content: &str) -> bool {
        let patterns = [
            "#[target_feature(enable",
            "std::arch::x86_64",
            "std::arch::aarch64",
            "_mm256_",
            "_mm512_",
            "_mm_",
            "vld1q_",
            "vst1q_",
            "is_x86_feature_detected!",
            "core::arch::",
        ];
        patterns.iter().any(|p| content.contains(p))
    }

    /// Simple function count heuristic
    fn count_functions(content: &str, category: &FileCategory) -> usize {
        match category {
            FileCategory::RustSource | FileCategory::SimdAvx => {
                // Count `fn ` occurrences (simple heuristic)
                content.matches("fn ").count()
            }
            FileCategory::CudaKernel => {
                // Count __global__ and __device__ functions
                content.matches("__global__").count() + content.matches("__device__").count()
            }
            FileCategory::PythonSource => content.matches("def ").count(),
            FileCategory::TypeScriptSource => {
                content.matches("function ").count() + content.matches("=> {").count()
            }
            FileCategory::GoSource => content.matches("func ").count(),
            _ => 0,
        }
    }
}

/// File category for coverage requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileCategory {
    /// Standard Rust code - must be covered
    RustSource,
    /// CUDA kernels - must be covered (no hiding allowed)
    CudaKernel,
    /// SIMD/AVX code - must be covered (no hiding allowed)
    SimdAvx,
    /// C/C++ source
    CSource,
    /// Python source
    PythonSource,
    /// TypeScript/JavaScript source
    TypeScriptSource,
    /// Go source
    GoSource,
    /// Test code - excluded from coverage
    TestCode,
    /// Build scripts - optional coverage
    BuildScript,
    /// Generated code - excluded
    Generated,
}

/// Falsifiable claim structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsifiableClaim {
    /// Human-readable hypothesis
    pub hypothesis: String,

    /// Method to attempt falsification
    pub falsification_method: FalsificationMethod,

    /// Evidence required to validate
    pub evidence_required: EvidenceType,

    /// Result of falsification attempt
    pub result: Option<FalsificationResult>,

    /// Optional override (requires justification AND ticket)
    pub override_info: Option<OverrideInfo>,
}

/// Information for an override (Popperian "Immunizing Stratagem")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideInfo {
    /// Reason for the override
    pub reason: String,

    /// Mandatory ticket ID (e.g., DEBT-123)
    pub ticket_id: String,

    /// Timestamp of override
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Methods for attempting to falsify claims
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FalsificationMethod {
    /// Try to find files in baseline missing from completion
    ManifestIntegrity,

    /// Try to find uncovered lines in changed code
    DifferentialCoverage,

    /// Try to find total coverage below threshold
    AbsoluteCoverage,

    /// Try to find TDG score regression
    TdgRegression,

    /// Try to find complexity regression
    ComplexityRegression,

    /// Try to find file size regression
    FileSizeRegression,

    /// Try to find spec score below threshold
    SpecQuality,

    /// Try to find roadmap not updated
    RoadmapUpdate,

    /// Try to find unpushed commits
    GitHubSync,

    /// Try to find #[cfg(not(coverage))] gaming
    CoverageGaming,

    /// Try to find vulnerable dependencies added (New in v1.1)
    SupplyChainIntegrity,

    /// Try to find flaws in the falsifier itself (Meta-Check) (New in v1.1)
    MetaFalsification,

    /// Try to find examples that don't compile/run (New in v1.2)
    ExamplesCompile,

    /// Try to find pmat-book validation failures (New in v1.2)
    BookValidation,
}

/// Evidence types for falsification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Numeric comparison (actual vs threshold)
    NumericComparison { actual: f64, threshold: f64 },

    /// File list (missing/added/modified)
    FileList(Vec<PathBuf>),

    /// Concrete counter-example details (better than boolean)
    CounterExample { details: String },

    /// Boolean check (Legacy, prefer CounterExample)
    BooleanCheck(bool),

    /// Git state
    GitState {
        unpushed_commits: usize,
        dirty_files: usize,
    },
}

/// Result of a falsification attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationResult {
    /// Did falsification succeed (found a problem)?
    pub falsified: bool,

    /// Evidence that caused falsification
    pub evidence: Option<EvidenceType>,

    /// Human-readable explanation
    pub explanation: String,
}

impl FalsificationResult {
    /// Create a passing result (hypothesis holds)
    pub fn passed(explanation: impl Into<String>) -> Self {
        Self {
            falsified: false,
            evidence: None,
            explanation: explanation.into(),
        }
    }

    /// Create a failing result (hypothesis falsified)
    pub fn failed(explanation: impl Into<String>, evidence: EvidenceType) -> Self {
        Self {
            falsified: true,
            evidence: Some(evidence),
            explanation: explanation.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_thresholds_default() {
        let thresholds = ContractThresholds::default();
        assert_eq!(thresholds.min_coverage_pct, 95.0);
        assert_eq!(thresholds.max_tdg_regression, 0.0);
        assert_eq!(thresholds.max_function_complexity, 20);
        assert_eq!(thresholds.max_file_lines, 500);
        assert_eq!(thresholds.min_spec_score, 95);
        assert!(thresholds.require_github_sync);
    }

    #[test]
    fn test_work_contract_default_claims() {
        let contract = WorkContract::new("test-item".to_string(), "abc123".to_string());
        assert_eq!(contract.claims.len(), 13); // 13 Popperian falsification claims

        // Verify all claim types are present
        let methods: Vec<_> = contract
            .claims
            .iter()
            .map(|c| &c.falsification_method)
            .collect();
        assert!(methods.contains(&&FalsificationMethod::ManifestIntegrity));
        assert!(methods.contains(&&FalsificationMethod::CoverageGaming));
        assert!(methods.contains(&&FalsificationMethod::AbsoluteCoverage));
        assert!(methods.contains(&&FalsificationMethod::TdgRegression));
    }

    #[test]
    fn test_falsification_result_passed() {
        let result = FalsificationResult::passed("Tests passed");
        assert!(!result.falsified);
        assert!(result.evidence.is_none());
    }

    #[test]
    fn test_falsification_result_failed() {
        let result = FalsificationResult::failed(
            "Coverage below threshold",
            EvidenceType::NumericComparison {
                actual: 80.0,
                threshold: 95.0,
            },
        );
        assert!(result.falsified);
        assert!(result.evidence.is_some());
    }

    #[test]
    fn test_simd_pattern_detection() {
        let simd_code = r#"
            use std::arch::x86_64::*;

            #[target_feature(enable = "avx2")]
            unsafe fn process() {
                let a = _mm256_set1_epi32(1);
            }
        "#;

        assert!(FileEntry::contains_simd_patterns(simd_code));

        let normal_code = r#"
            fn normal_function() {
                println!("Hello");
            }
        "#;

        assert!(!FileEntry::contains_simd_patterns(normal_code));
    }

    #[test]
    fn test_file_category_rust_source() {
        // Normal rust file should be RustSource
        let content = "fn main() { println!(\"hello\"); }";
        assert!(!FileEntry::contains_simd_patterns(content));
    }
}
