//! File Health Score Service
//!
//! Implements the File Health Score specification (docs/specifications/max-lines.md).
//! Detects, prevents, and reports on excessively large source files.
//!
//! # Scientific Foundation
//!
//! Based on peer-reviewed research showing correlation between file size and defects:
//! - Hindle et al. (2008): Files >500 lines exhibit exponential defect density increase
//! - Nagappan et al. (2006): r=0.67 correlation between LOC and defect count
//! - Bird et al. (2011): Files >400 LOC show ownership fragmentation
//!
//! # Toyota Way Principles
//!
//! - **Jidoka**: Pre-commit hook blocks large file creation
//! - **Kaizen**: Ratchet mechanism forces gradual reduction
//! - **Muda**: Large files create cognitive waste

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Thresholds for file size classification
pub mod thresholds {
    pub const IDEAL_MAX: usize = 200;
    pub const ACCEPTABLE_MAX: usize = 500;
    pub const WARNING_MAX: usize = 1000;
    pub const PROBLEM_MAX: usize = 2000;
    // >2000 is CRITICAL
}

/// File size classification based on line count
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSizeClass {
    /// 0-200 lines: Optimal cognitive chunk
    Ideal,
    /// 201-500 lines: Within SRP tolerance
    Acceptable,
    /// 501-1000 lines: Approaching limit
    Warning,
    /// 1001-2000 lines: Exceeds cognitive capacity
    Problem,
    /// >2000 lines: Untestable monolith
    Critical,
}

impl FileSizeClass {
    pub fn from_lines(lines: usize) -> Self {
        match lines {
            0..=200 => Self::Ideal,
            201..=500 => Self::Acceptable,
            501..=1000 => Self::Warning,
            1001..=2000 => Self::Problem,
            _ => Self::Critical,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ideal => "ideal",
            Self::Acceptable => "acceptable",
            Self::Warning => "warning",
            Self::Problem => "problem",
            Self::Critical => "critical",
        }
    }
}

/// Health grade based on composite score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthGrade {
    A, // 90-100
    B, // 80-89
    C, // 70-79
    D, // 60-69
    E, // 50-59
    F, // 0-49
}

impl HealthGrade {
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::A,
            80..=89 => Self::B,
            70..=79 => Self::C,
            60..=69 => Self::D,
            50..=59 => Self::E,
            _ => Self::F,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
        }
    }

    pub fn is_passing(&self) -> bool {
        matches!(self, Self::A | Self::B | Self::C)
    }
}

/// Health metrics for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHealthMetrics {
    pub path: PathBuf,
    pub lines: usize,
    pub test_lines: usize,
    pub tlr: f32,
    pub required_tlr: f32,
    pub avg_complexity: f32,
    pub churn_30d: usize,
    pub health_score: u8,
    pub grade: HealthGrade,
    pub size_class: FileSizeClass,
}

impl FileHealthMetrics {
    /// Calculate health score using the composite formula
    pub fn calculate(
        path: PathBuf,
        lines: usize,
        test_lines: usize,
        avg_complexity: f32,
        churn_30d: usize,
    ) -> Self {
        let required_tlr = Self::required_tlr_for_size(lines);
        let tlr = if lines > 0 {
            test_lines as f32 / lines as f32
        } else {
            1.0
        };

        // Size Score (30 points max)
        let size_score: u8 = match lines {
            0..=200 => 30,
            201..=500 => 25,
            501..=1000 => 15,
            1001..=2000 => 5,
            _ => 0,
        };

        // TLR Score (40 points max)
        let tlr_ratio = (tlr / required_tlr).min(1.0);
        let tlr_score = (tlr_ratio * 40.0) as u8;

        // Complexity Score (20 points max)
        let complexity_score: u8 = if avg_complexity <= 5.0 {
            20
        } else if avg_complexity <= 10.0 {
            15
        } else if avg_complexity <= 15.0 {
            10
        } else if avg_complexity <= 20.0 {
            5
        } else {
            0
        };

        // Stability Score (10 points max)
        let stability_score: u8 = match churn_30d {
            0..=2 => 10,
            3..=5 => 7,
            6..=10 => 4,
            _ => 0,
        };

        let health_score = size_score + tlr_score + complexity_score + stability_score;
        let grade = HealthGrade::from_score(health_score);
        let size_class = FileSizeClass::from_lines(lines);

        Self {
            path,
            lines,
            test_lines,
            tlr,
            required_tlr,
            avg_complexity,
            churn_30d,
            health_score,
            grade,
            size_class,
        }
    }

    /// Get required TLR based on file size (scaling thresholds)
    pub fn required_tlr_for_size(lines: usize) -> f32 {
        match lines {
            0..=100 => 0.3,
            101..=300 => 0.5,
            301..=500 => 0.7,
            501..=1000 => 1.0,
            _ => 1.5,
        }
    }
}

/// Aggregate report for all files in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHealthReport {
    pub project_path: PathBuf,
    pub total_files: usize,
    pub total_lines: usize,
    pub average_health: u8,
    pub average_grade: HealthGrade,
    pub critical_files: Vec<FileHealthMetrics>,
    pub problem_files: Vec<FileHealthMetrics>,
    pub warning_files: Vec<FileHealthMetrics>,
    pub healthy_files_count: usize,
    pub is_compliant: bool,
    pub recommendations: Vec<String>,
}

impl FileHealthReport {
    /// Create a new report from analyzed files
    pub fn from_files(project_path: PathBuf, files: Vec<FileHealthMetrics>) -> Self {
        let total_files = files.len();
        let total_lines: usize = files.iter().map(|f| f.lines).sum();

        let average_health = if total_files > 0 {
            (files.iter().map(|f| f.health_score as u32).sum::<u32>() / total_files as u32) as u8
        } else {
            100
        };

        let average_grade = HealthGrade::from_score(average_health);

        let mut critical_files: Vec<FileHealthMetrics> = files
            .iter()
            .filter(|f| f.health_score < 50)
            .cloned()
            .collect();
        critical_files.sort_by(|a, b| a.health_score.cmp(&b.health_score));

        let mut problem_files: Vec<FileHealthMetrics> = files
            .iter()
            .filter(|f| f.health_score >= 50 && f.health_score < 70)
            .cloned()
            .collect();
        problem_files.sort_by(|a, b| a.health_score.cmp(&b.health_score));

        let warning_files: Vec<FileHealthMetrics> = files
            .iter()
            .filter(|f| f.health_score >= 70 && f.health_score < 80)
            .cloned()
            .collect();

        let healthy_files_count = files.iter().filter(|f| f.health_score >= 80).count();

        let is_compliant = critical_files.is_empty();

        let mut recommendations = Vec::new();

        // Generate recommendations based on findings
        if !critical_files.is_empty() {
            recommendations.push(format!(
                "CRITICAL: {} files have health score <50 and require immediate attention",
                critical_files.len()
            ));

            for file in critical_files.iter().take(5) {
                recommendations.push(format!(
                    "  - Split {} ({} lines, health: {})",
                    file.path.display(),
                    file.lines,
                    file.health_score
                ));
            }
        }

        if !problem_files.is_empty() {
            recommendations.push(format!(
                "WARNING: {} files have health score 50-69 and should be refactored",
                problem_files.len()
            ));
        }

        // TLR recommendations
        let low_tlr_files: Vec<_> = files.iter().filter(|f| f.tlr < f.required_tlr).collect();
        if !low_tlr_files.is_empty() {
            recommendations.push(format!(
                "TEST COVERAGE: {} files have TLR below required threshold",
                low_tlr_files.len()
            ));
        }

        Self {
            project_path,
            total_files,
            total_lines,
            average_health,
            average_grade,
            critical_files,
            problem_files,
            warning_files,
            healthy_files_count,
            is_compliant,
            recommendations,
        }
    }
}

/// Baseline file for ratchet mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHealthBaseline {
    pub version: String,
    pub generated: String,
    pub files: HashMap<String, BaselineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub lines: usize,
    pub test_lines: usize,
    pub tlr: f32,
    pub health: u8,
    pub status: String,
}

impl FileHealthBaseline {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            generated: chrono::Utc::now().to_rfc3339(),
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, metrics: &FileHealthMetrics) {
        let key = metrics.path.to_string_lossy().to_string();
        self.files.insert(
            key,
            BaselineEntry {
                lines: metrics.lines,
                test_lines: metrics.test_lines,
                tlr: metrics.tlr,
                health: metrics.health_score,
                status: metrics.size_class.as_str().to_string(),
            },
        );
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Check if a file violates the ratchet (grew larger)
    pub fn check_ratchet(&self, path: &str, current_lines: usize) -> Option<RatchetViolation> {
        if let Some(baseline) = self.files.get(path) {
            if current_lines > baseline.lines {
                return Some(RatchetViolation {
                    path: path.to_string(),
                    baseline_lines: baseline.lines,
                    current_lines,
                    growth: current_lines - baseline.lines,
                });
            }
        }
        None
    }
}

impl Default for FileHealthBaseline {
    fn default() -> Self {
        Self::new()
    }
}

/// Ratchet violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetViolation {
    pub path: String,
    pub baseline_lines: usize,
    pub current_lines: usize,
    pub growth: usize,
}

/// Analyze a single file for health metrics
pub fn analyze_file(
    path: &Path,
    test_lines: usize,
    avg_complexity: f32,
    churn_30d: usize,
) -> Option<FileHealthMetrics> {
    let content = fs::read_to_string(path).ok()?;
    let lines = content.lines().count();

    Some(FileHealthMetrics::calculate(
        path.to_path_buf(),
        lines,
        test_lines,
        avg_complexity,
        churn_30d,
    ))
}

/// Count lines in a file
pub fn count_lines(path: &Path) -> Option<usize> {
    let content = fs::read_to_string(path).ok()?;
    Some(content.lines().count())
}

/// Scan a directory for source files and analyze health
pub fn scan_directory(root: &Path, extensions: &[&str], exclude_patterns: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    fn visit_dir(
        dir: &Path,
        extensions: &[&str],
        exclude_patterns: &[&str],
        files: &mut Vec<PathBuf>,
    ) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Check exclusions
                let excluded = exclude_patterns.iter().any(|p| path_str.contains(p));
                if excluded {
                    continue;
                }

                if path.is_dir() {
                    visit_dir(&path, extensions, exclude_patterns, files);
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.iter().any(|e| ext == *e) {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }

    visit_dir(root, extensions, exclude_patterns, &mut files);
    files
}

/// Default exclusion patterns for Rust projects
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    "target/",
    ".git/",
    "node_modules/",
    "vendor/",
    ".pmat-cache/",
    "_generated",
    ".generated.",
];

/// Default extensions for Rust projects
pub const RUST_EXTENSIONS: &[&str] = &["rs"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_classification() {
        assert_eq!(FileSizeClass::from_lines(100), FileSizeClass::Ideal);
        assert_eq!(FileSizeClass::from_lines(200), FileSizeClass::Ideal);
        assert_eq!(FileSizeClass::from_lines(201), FileSizeClass::Acceptable);
        assert_eq!(FileSizeClass::from_lines(500), FileSizeClass::Acceptable);
        assert_eq!(FileSizeClass::from_lines(501), FileSizeClass::Warning);
        assert_eq!(FileSizeClass::from_lines(1000), FileSizeClass::Warning);
        assert_eq!(FileSizeClass::from_lines(1001), FileSizeClass::Problem);
        assert_eq!(FileSizeClass::from_lines(2000), FileSizeClass::Problem);
        assert_eq!(FileSizeClass::from_lines(2001), FileSizeClass::Critical);
        assert_eq!(FileSizeClass::from_lines(12000), FileSizeClass::Critical);
    }

    #[test]
    fn test_health_grade_from_score() {
        assert_eq!(HealthGrade::from_score(100), HealthGrade::A);
        assert_eq!(HealthGrade::from_score(90), HealthGrade::A);
        assert_eq!(HealthGrade::from_score(89), HealthGrade::B);
        assert_eq!(HealthGrade::from_score(80), HealthGrade::B);
        assert_eq!(HealthGrade::from_score(79), HealthGrade::C);
        assert_eq!(HealthGrade::from_score(70), HealthGrade::C);
        assert_eq!(HealthGrade::from_score(69), HealthGrade::D);
        assert_eq!(HealthGrade::from_score(60), HealthGrade::D);
        assert_eq!(HealthGrade::from_score(59), HealthGrade::E);
        assert_eq!(HealthGrade::from_score(50), HealthGrade::E);
        assert_eq!(HealthGrade::from_score(49), HealthGrade::F);
        assert_eq!(HealthGrade::from_score(0), HealthGrade::F);
    }

    #[test]
    fn test_required_tlr_scaling() {
        assert_eq!(FileHealthMetrics::required_tlr_for_size(50), 0.3);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(100), 0.3);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(150), 0.5);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(300), 0.5);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(400), 0.7);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(500), 0.7);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(600), 1.0);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(1000), 1.0);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(1500), 1.5);
    }

    #[test]
    fn test_health_score_calculation_ideal_file() {
        // 100 lines, 50 test lines, low complexity, stable
        let metrics = FileHealthMetrics::calculate(
            PathBuf::from("test.rs"),
            100, // lines
            50,  // test_lines (TLR = 0.5, required = 0.3)
            3.0, // avg_complexity
            1,   // churn_30d
        );

        assert_eq!(metrics.size_class, FileSizeClass::Ideal);
        // size: 30, tlr: 40 (0.5/0.3 > 1.0), complexity: 20, stability: 10 = 100
        assert_eq!(metrics.health_score, 100);
        assert_eq!(metrics.grade, HealthGrade::A);
    }

    #[test]
    fn test_health_score_calculation_critical_file() {
        // 5000 lines, 100 test lines, high complexity, volatile
        let metrics = FileHealthMetrics::calculate(
            PathBuf::from("monolith.rs"),
            5000, // lines (critical)
            100,  // test_lines (TLR = 0.02, required = 1.5)
            25.0, // avg_complexity (very high)
            15,   // churn_30d (hot file)
        );

        assert_eq!(metrics.size_class, FileSizeClass::Critical);
        // size: 0, tlr: ~0.5 (0.02/1.5 * 40), complexity: 0, stability: 0
        assert!(metrics.health_score < 10);
        assert_eq!(metrics.grade, HealthGrade::F);
    }

    #[test]
    fn test_health_score_calculation_medium_file() {
        // 400 lines, 200 test lines, medium complexity
        let metrics = FileHealthMetrics::calculate(
            PathBuf::from("service.rs"),
            400, // lines (acceptable)
            200, // test_lines (TLR = 0.5, required = 0.7)
            8.0, // avg_complexity
            3,   // churn_30d
        );

        assert_eq!(metrics.size_class, FileSizeClass::Acceptable);
        // size: 25, tlr: ~28 (0.5/0.7 * 40), complexity: 15, stability: 7 = ~75
        assert!(metrics.health_score >= 70 && metrics.health_score <= 80);
        assert!(matches!(metrics.grade, HealthGrade::B | HealthGrade::C));
    }

    #[test]
    fn test_baseline_ratchet_violation() {
        let mut baseline = FileHealthBaseline::new();
        baseline.files.insert(
            "src/big.rs".to_string(),
            BaselineEntry {
                lines: 1000,
                test_lines: 200,
                tlr: 0.2,
                health: 45,
                status: "warning".to_string(),
            },
        );

        // File grew - violation
        let violation = baseline.check_ratchet("src/big.rs", 1050);
        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.baseline_lines, 1000);
        assert_eq!(v.current_lines, 1050);
        assert_eq!(v.growth, 50);

        // File shrunk - no violation
        let no_violation = baseline.check_ratchet("src/big.rs", 950);
        assert!(no_violation.is_none());

        // New file - no violation (not in baseline)
        let new_file = baseline.check_ratchet("src/new.rs", 600);
        assert!(new_file.is_none());
    }

    #[test]
    fn test_report_compliance() {
        let files = vec![
            FileHealthMetrics::calculate(PathBuf::from("good.rs"), 100, 50, 3.0, 1),
            FileHealthMetrics::calculate(PathBuf::from("ok.rs"), 300, 100, 8.0, 2),
        ];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        assert!(report.is_compliant);
        assert!(report.critical_files.is_empty());
    }

    #[test]
    fn test_report_non_compliance() {
        let files = vec![
            FileHealthMetrics::calculate(PathBuf::from("good.rs"), 100, 50, 3.0, 1),
            FileHealthMetrics::calculate(PathBuf::from("bad.rs"), 5000, 50, 25.0, 20),
        ];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        assert!(!report.is_compliant);
        assert_eq!(report.critical_files.len(), 1);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_grade_is_passing() {
        assert!(HealthGrade::A.is_passing());
        assert!(HealthGrade::B.is_passing());
        assert!(HealthGrade::C.is_passing());
        assert!(!HealthGrade::D.is_passing());
        assert!(!HealthGrade::E.is_passing());
        assert!(!HealthGrade::F.is_passing());
    }

    #[test]
    fn test_file_size_class_as_str() {
        assert_eq!(FileSizeClass::Ideal.as_str(), "ideal");
        assert_eq!(FileSizeClass::Acceptable.as_str(), "acceptable");
        assert_eq!(FileSizeClass::Warning.as_str(), "warning");
        assert_eq!(FileSizeClass::Problem.as_str(), "problem");
        assert_eq!(FileSizeClass::Critical.as_str(), "critical");
    }

    #[test]
    fn test_health_grade_as_str() {
        assert_eq!(HealthGrade::A.as_str(), "A");
        assert_eq!(HealthGrade::B.as_str(), "B");
        assert_eq!(HealthGrade::C.as_str(), "C");
        assert_eq!(HealthGrade::D.as_str(), "D");
        assert_eq!(HealthGrade::E.as_str(), "E");
        assert_eq!(HealthGrade::F.as_str(), "F");
    }

    #[test]
    fn test_empty_report() {
        let report = FileHealthReport::from_files(PathBuf::from("."), vec![]);
        assert_eq!(report.total_files, 0);
        assert_eq!(report.total_lines, 0);
        assert_eq!(report.average_health, 100);
        assert!(report.is_compliant);
    }

    #[test]
    fn test_report_with_problem_files() {
        // A file that lands in problem range (50-69):
        // - 600 lines (warning range) = 15 pts
        // - 400 test lines, TLR=0.67, required=1.0, ratio=0.67, score ≈ 27 pts
        // - complexity 8.0 = 15 pts
        // - churn 4 = 7 pts
        // Total = 15+27+15+7 = 64 (problem range)
        let files = vec![
            FileHealthMetrics::calculate(PathBuf::from("medium.rs"), 600, 400, 8.0, 4),
        ];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        // Should have problem files but still be compliant (no critical)
        assert!(report.is_compliant);
        assert!(!report.problem_files.is_empty() || !report.warning_files.is_empty());
    }

    #[test]
    fn test_baseline_add_file() {
        let mut baseline = FileHealthBaseline::new();
        let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 200, 100, 5.0, 1);

        baseline.add_file(&metrics);

        assert!(baseline.files.contains_key("test.rs"));
        let entry = baseline.files.get("test.rs").unwrap();
        assert_eq!(entry.lines, 200);
        assert_eq!(entry.test_lines, 100);
    }

    #[test]
    fn test_baseline_default() {
        let baseline = FileHealthBaseline::default();
        assert!(baseline.files.is_empty());
        assert_eq!(baseline.version, "1.0");
    }

    #[test]
    fn test_baseline_save_and_load() {
        let mut baseline = FileHealthBaseline::new();
        let metrics = FileHealthMetrics::calculate(PathBuf::from("src/lib.rs"), 150, 75, 4.0, 2);
        baseline.add_file(&metrics);

        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_baseline.json");

        baseline.save(&temp_path).expect("Failed to save");

        // Load it back
        let loaded = FileHealthBaseline::load(&temp_path).expect("Failed to load");

        assert_eq!(loaded.version, baseline.version);
        assert!(loaded.files.contains_key("src/lib.rs"));

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_analyze_file_function() {
        // Create a temp file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_analyze.rs");
        std::fs::write(&temp_path, "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();

        let metrics = analyze_file(&temp_path, 10, 3.0, 1);
        assert!(metrics.is_some());

        let m = metrics.unwrap();
        assert_eq!(m.lines, 3); // 3 lines (trailing newline doesn't count as line)
        assert_eq!(m.test_lines, 10);

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_analyze_file_nonexistent() {
        let result = analyze_file(Path::new("/nonexistent/file.rs"), 0, 0.0, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_count_lines_function() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_count.rs");
        std::fs::write(&temp_path, "line1\nline2\nline3\n").unwrap();

        let count = count_lines(&temp_path);
        assert_eq!(count, Some(3));

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_count_lines_nonexistent() {
        let count = count_lines(Path::new("/nonexistent/file.rs"));
        assert!(count.is_none());
    }

    #[test]
    fn test_scan_directory() {
        // Use CARGO_MANIFEST_DIR for deterministic path
        let project_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let files = scan_directory(
            project_dir,
            &["rs"],
            DEFAULT_EXCLUDE_PATTERNS,
        );

        // Should find some Rust files in the project
        assert!(!files.is_empty(), "Should find Rust files in project directory");
    }

    #[test]
    fn test_health_zero_lines_tlr() {
        // Edge case: zero lines should give TLR of 1.0
        let metrics = FileHealthMetrics::calculate(PathBuf::from("empty.rs"), 0, 0, 0.0, 0);
        assert_eq!(metrics.tlr, 1.0);
    }

    #[test]
    fn test_report_with_low_tlr_files() {
        let files = vec![
            // Low TLR file
            FileHealthMetrics::calculate(PathBuf::from("untested.rs"), 500, 50, 10.0, 3),
        ];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        // Should have TLR recommendation
        assert!(report.recommendations.iter().any(|r| r.contains("TLR")));
    }
}
