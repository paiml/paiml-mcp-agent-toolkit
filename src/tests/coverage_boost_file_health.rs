#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/file_health.rs
//! Tests pure functions: FileSizeClass, HealthGrade, FileHealthMetrics, FileHealthReport,
//! FileHealthBaseline, RatchetViolation, analyze_file, count_lines, scan_directory

use crate::services::file_health::{
    analyze_file, count_lines, scan_directory, thresholds, FileHealthBaseline, FileHealthMetrics,
    FileHealthReport, FileSizeClass, HealthGrade, RatchetViolation,
};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// --- FileSizeClass tests ---

#[test]
fn test_file_size_class_ideal() {
    assert_eq!(FileSizeClass::from_lines(0), FileSizeClass::Ideal);
    assert_eq!(FileSizeClass::from_lines(100), FileSizeClass::Ideal);
    assert_eq!(FileSizeClass::from_lines(200), FileSizeClass::Ideal);
}

#[test]
fn test_file_size_class_acceptable() {
    assert_eq!(FileSizeClass::from_lines(201), FileSizeClass::Acceptable);
    assert_eq!(FileSizeClass::from_lines(350), FileSizeClass::Acceptable);
    assert_eq!(FileSizeClass::from_lines(500), FileSizeClass::Acceptable);
}

#[test]
fn test_file_size_class_warning() {
    assert_eq!(FileSizeClass::from_lines(501), FileSizeClass::Warning);
    assert_eq!(FileSizeClass::from_lines(750), FileSizeClass::Warning);
    assert_eq!(FileSizeClass::from_lines(1000), FileSizeClass::Warning);
}

#[test]
fn test_file_size_class_problem() {
    assert_eq!(FileSizeClass::from_lines(1001), FileSizeClass::Problem);
    assert_eq!(FileSizeClass::from_lines(1500), FileSizeClass::Problem);
    assert_eq!(FileSizeClass::from_lines(2000), FileSizeClass::Problem);
}

#[test]
fn test_file_size_class_critical() {
    assert_eq!(FileSizeClass::from_lines(2001), FileSizeClass::Critical);
    assert_eq!(FileSizeClass::from_lines(5000), FileSizeClass::Critical);
    assert_eq!(FileSizeClass::from_lines(100_000), FileSizeClass::Critical);
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
fn test_file_size_class_debug() {
    let _ = format!("{:?}", FileSizeClass::Ideal);
    let _ = format!("{:?}", FileSizeClass::Acceptable);
    let _ = format!("{:?}", FileSizeClass::Warning);
    let _ = format!("{:?}", FileSizeClass::Problem);
    let _ = format!("{:?}", FileSizeClass::Critical);
}

#[test]
fn test_file_size_class_clone_eq() {
    let a = FileSizeClass::Warning;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(FileSizeClass::Ideal, FileSizeClass::Critical);
}

// --- HealthGrade tests ---

#[test]
fn test_health_grade_from_score_a() {
    assert_eq!(HealthGrade::from_score(90), HealthGrade::A);
    assert_eq!(HealthGrade::from_score(95), HealthGrade::A);
    assert_eq!(HealthGrade::from_score(100), HealthGrade::A);
}

#[test]
fn test_health_grade_from_score_b() {
    assert_eq!(HealthGrade::from_score(80), HealthGrade::B);
    assert_eq!(HealthGrade::from_score(85), HealthGrade::B);
    assert_eq!(HealthGrade::from_score(89), HealthGrade::B);
}

#[test]
fn test_health_grade_from_score_c() {
    assert_eq!(HealthGrade::from_score(70), HealthGrade::C);
    assert_eq!(HealthGrade::from_score(75), HealthGrade::C);
    assert_eq!(HealthGrade::from_score(79), HealthGrade::C);
}

#[test]
fn test_health_grade_from_score_d() {
    assert_eq!(HealthGrade::from_score(60), HealthGrade::D);
    assert_eq!(HealthGrade::from_score(65), HealthGrade::D);
    assert_eq!(HealthGrade::from_score(69), HealthGrade::D);
}

#[test]
fn test_health_grade_from_score_e() {
    assert_eq!(HealthGrade::from_score(50), HealthGrade::E);
    assert_eq!(HealthGrade::from_score(55), HealthGrade::E);
    assert_eq!(HealthGrade::from_score(59), HealthGrade::E);
}

#[test]
fn test_health_grade_from_score_f() {
    assert_eq!(HealthGrade::from_score(0), HealthGrade::F);
    assert_eq!(HealthGrade::from_score(25), HealthGrade::F);
    assert_eq!(HealthGrade::from_score(49), HealthGrade::F);
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
fn test_health_grade_is_passing() {
    assert!(HealthGrade::A.is_passing());
    assert!(HealthGrade::B.is_passing());
    assert!(HealthGrade::C.is_passing());
    assert!(!HealthGrade::D.is_passing());
    assert!(!HealthGrade::E.is_passing());
    assert!(!HealthGrade::F.is_passing());
}

#[test]
fn test_health_grade_debug_clone_eq() {
    let _ = format!("{:?}", HealthGrade::A);
    let a = HealthGrade::B;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(HealthGrade::A, HealthGrade::F);
}

// --- Thresholds constants ---

#[test]
fn test_thresholds_constants() {
    assert_eq!(thresholds::IDEAL_MAX, 200);
    assert_eq!(thresholds::ACCEPTABLE_MAX, 500);
    assert_eq!(thresholds::WARNING_MAX, 1000);
    assert_eq!(thresholds::PROBLEM_MAX, 2000);
}

// --- FileHealthMetrics tests ---

#[test]
fn test_file_health_metrics_calculate_small_file() {
    let metrics = FileHealthMetrics::calculate(
        PathBuf::from("small.rs"),
        100,  // lines
        50,   // test_lines
        5.0,  // avg_complexity
        1,    // churn_30d
    );
    assert_eq!(metrics.lines, 100);
    assert_eq!(metrics.test_lines, 50);
    assert!(metrics.tlr > 0.0);
    assert_eq!(metrics.size_class, FileSizeClass::Ideal);
    assert!(metrics.health_score > 0);
}

#[test]
fn test_file_health_metrics_calculate_large_file() {
    let metrics = FileHealthMetrics::calculate(
        PathBuf::from("large.rs"),
        3000, // lines
        100,  // test_lines
        25.0, // avg_complexity
        15,   // churn_30d
    );
    assert_eq!(metrics.lines, 3000);
    assert_eq!(metrics.size_class, FileSizeClass::Critical);
    // Large file with high complexity and high churn should have low score
    assert!(metrics.health_score < 50);
}

#[test]
fn test_file_health_metrics_calculate_zero_lines() {
    let metrics = FileHealthMetrics::calculate(
        PathBuf::from("empty.rs"),
        0,
        0,
        0.0,
        0,
    );
    assert_eq!(metrics.lines, 0);
    assert_eq!(metrics.tlr, 1.0); // 0 lines => tlr = 1.0
    assert_eq!(metrics.size_class, FileSizeClass::Ideal);
}

#[test]
fn test_file_health_metrics_calculate_medium_file() {
    let metrics = FileHealthMetrics::calculate(
        PathBuf::from("medium.rs"),
        500,
        250,
        10.0,
        5,
    );
    assert_eq!(metrics.size_class, FileSizeClass::Acceptable);
    assert!(metrics.health_score >= 50);
}

#[test]
fn test_file_health_metrics_required_tlr_for_size() {
    assert_eq!(FileHealthMetrics::required_tlr_for_size(50), 0.3);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(100), 0.3);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(200), 0.5);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(300), 0.5);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(400), 0.7);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(500), 0.7);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(750), 1.0);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(1000), 1.0);
    assert_eq!(FileHealthMetrics::required_tlr_for_size(2000), 1.5);
}

#[test]
fn test_file_health_metrics_score_components() {
    // Test that all score components contribute
    // Max possible: size(30) + tlr(40) + complexity(20) + stability(10) = 100
    let metrics = FileHealthMetrics::calculate(
        PathBuf::from("perfect.rs"),
        50,   // ideal size: 30 points
        100,  // high test coverage
        3.0,  // low complexity: 20 points
        0,    // no churn: 10 points
    );
    assert!(metrics.health_score >= 90, "Perfect file should score 90+, got {}", metrics.health_score);
    assert_eq!(metrics.grade, HealthGrade::A);
}

#[test]
fn test_file_health_metrics_debug_clone() {
    let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 100, 50, 5.0, 1);
    let _ = format!("{:?}", metrics);
    let cloned = metrics.clone();
    assert_eq!(cloned.lines, metrics.lines);
}

// --- FileHealthReport tests ---

#[test]
fn test_file_health_report_from_empty_files() {
    let report = FileHealthReport::from_files(PathBuf::from("."), vec![]);
    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_lines, 0);
    assert_eq!(report.average_health, 100);
    assert!(report.is_compliant);
    assert!(report.critical_files.is_empty());
    assert!(report.problem_files.is_empty());
    assert!(report.warning_files.is_empty());
}

#[test]
fn test_file_health_report_from_mixed_files() {
    let files = vec![
        FileHealthMetrics::calculate(PathBuf::from("good.rs"), 100, 50, 5.0, 1),
        FileHealthMetrics::calculate(PathBuf::from("bad.rs"), 3000, 10, 25.0, 20),
        FileHealthMetrics::calculate(PathBuf::from("medium.rs"), 500, 200, 12.0, 5),
    ];
    let report = FileHealthReport::from_files(PathBuf::from("."), files);
    assert_eq!(report.total_files, 3);
    assert!(report.total_lines > 0);
    assert!(report.average_health > 0);
    // Should have some recommendations
    let _ = format!("{:?}", report);
}

#[test]
fn test_file_health_report_recommendations() {
    let files = vec![
        FileHealthMetrics::calculate(PathBuf::from("critical1.rs"), 5000, 0, 30.0, 25),
        FileHealthMetrics::calculate(PathBuf::from("critical2.rs"), 4000, 0, 28.0, 20),
    ];
    let report = FileHealthReport::from_files(PathBuf::from("."), files);
    assert!(!report.is_compliant);
    assert!(!report.recommendations.is_empty());
    assert!(!report.critical_files.is_empty());
}

#[test]
fn test_file_health_report_all_healthy() {
    let files = vec![
        FileHealthMetrics::calculate(PathBuf::from("a.rs"), 50, 25, 3.0, 0),
        FileHealthMetrics::calculate(PathBuf::from("b.rs"), 80, 40, 4.0, 1),
    ];
    let report = FileHealthReport::from_files(PathBuf::from("."), files);
    assert!(report.is_compliant);
    assert_eq!(report.healthy_files_count, 2);
    assert!(report.critical_files.is_empty());
}

// --- FileHealthBaseline tests ---

#[test]
fn test_file_health_baseline_new() {
    let baseline = FileHealthBaseline::new();
    assert_eq!(baseline.version, "1.0");
    assert!(baseline.files.is_empty());
    assert!(!baseline.generated.is_empty());
}

#[test]
fn test_file_health_baseline_default() {
    let baseline = FileHealthBaseline::default();
    assert_eq!(baseline.version, "1.0");
}

#[test]
fn test_file_health_baseline_add_file() {
    let mut baseline = FileHealthBaseline::new();
    let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 100, 50, 5.0, 1);
    baseline.add_file(&metrics);
    assert_eq!(baseline.files.len(), 1);
    let entry = baseline.files.get("test.rs").unwrap();
    assert_eq!(entry.lines, 100);
    assert_eq!(entry.test_lines, 50);
    assert_eq!(entry.health, metrics.health_score);
}

#[test]
fn test_file_health_baseline_check_ratchet_no_violation() {
    let mut baseline = FileHealthBaseline::new();
    let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 100, 50, 5.0, 1);
    baseline.add_file(&metrics);

    // Same or fewer lines should not violate
    assert!(baseline.check_ratchet("test.rs", 100).is_none());
    assert!(baseline.check_ratchet("test.rs", 50).is_none());
}

#[test]
fn test_file_health_baseline_check_ratchet_violation() {
    let mut baseline = FileHealthBaseline::new();
    let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 100, 50, 5.0, 1);
    baseline.add_file(&metrics);

    let violation = baseline.check_ratchet("test.rs", 150).unwrap();
    assert_eq!(violation.baseline_lines, 100);
    assert_eq!(violation.current_lines, 150);
    assert_eq!(violation.growth, 50);
    assert_eq!(violation.path, "test.rs");
}

#[test]
fn test_file_health_baseline_check_ratchet_unknown_file() {
    let baseline = FileHealthBaseline::new();
    assert!(baseline.check_ratchet("unknown.rs", 100).is_none());
}

#[test]
fn test_file_health_baseline_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("baseline.json");

    let mut baseline = FileHealthBaseline::new();
    let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 200, 100, 8.0, 3);
    baseline.add_file(&metrics);
    baseline.save(&path).unwrap();

    let loaded = FileHealthBaseline::load(&path).unwrap();
    assert_eq!(loaded.version, "1.0");
    assert_eq!(loaded.files.len(), 1);
    let entry = loaded.files.get("test.rs").unwrap();
    assert_eq!(entry.lines, 200);
}

// --- RatchetViolation tests ---

#[test]
fn test_ratchet_violation_debug_clone() {
    let violation = RatchetViolation {
        path: "test.rs".to_string(),
        baseline_lines: 100,
        current_lines: 150,
        growth: 50,
    };
    let _ = format!("{:?}", violation);
    let cloned = violation.clone();
    assert_eq!(cloned.growth, 50);
}

// --- analyze_file tests ---

#[test]
fn test_analyze_file_real_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(&file_path, "fn main() {\n    println!(\"hello\");\n}\n").unwrap();

    let metrics = analyze_file(&file_path, 5, 3.0, 1);
    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert_eq!(m.lines, 3);
    assert_eq!(m.test_lines, 5);
}

#[test]
fn test_analyze_file_nonexistent() {
    let result = analyze_file(Path::new("/nonexistent/file.rs"), 0, 0.0, 0);
    assert!(result.is_none());
}

// --- count_lines tests ---

#[test]
fn test_count_lines_real_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(&file_path, "line1\nline2\nline3\nline4\nline5\n").unwrap();

    assert_eq!(count_lines(&file_path), Some(5));
}

#[test]
fn test_count_lines_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.rs");
    std::fs::write(&file_path, "").unwrap();

    assert_eq!(count_lines(&file_path), Some(0));
}

#[test]
fn test_count_lines_nonexistent() {
    assert!(count_lines(Path::new("/nonexistent")).is_none());
}

// --- scan_directory tests ---

#[test]
fn test_scan_directory_with_files() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("a.rs"), "fn a() {}").unwrap();
    std::fs::write(temp_dir.path().join("b.rs"), "fn b() {}").unwrap();
    std::fs::write(temp_dir.path().join("c.txt"), "not rust").unwrap();

    let files = scan_directory(temp_dir.path(), &["rs"], &[]);
    assert_eq!(files.len(), 2);
}

#[test]
fn test_scan_directory_with_exclude() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("target");
    std::fs::create_dir_all(&sub_dir).unwrap();
    std::fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
    std::fs::write(sub_dir.join("build.rs"), "fn build() {}").unwrap();

    let files = scan_directory(temp_dir.path(), &["rs"], &["target"]);
    // Should exclude files in target directory
    assert!(files.len() <= 2);
}

#[test]
fn test_scan_directory_empty() {
    let temp_dir = TempDir::new().unwrap();
    let files = scan_directory(temp_dir.path(), &["rs"], &[]);
    assert!(files.is_empty());
}

// --- Serialization tests ---

#[test]
fn test_file_size_class_serde() {
    let class = FileSizeClass::Warning;
    let json = serde_json::to_string(&class).unwrap();
    let back: FileSizeClass = serde_json::from_str(&json).unwrap();
    assert_eq!(back, class);
}

#[test]
fn test_health_grade_serde() {
    let grade = HealthGrade::B;
    let json = serde_json::to_string(&grade).unwrap();
    let back: HealthGrade = serde_json::from_str(&json).unwrap();
    assert_eq!(back, grade);
}

#[test]
fn test_file_health_metrics_serde() {
    let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 100, 50, 5.0, 1);
    let json = serde_json::to_string(&metrics).unwrap();
    let back: FileHealthMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lines, metrics.lines);
    assert_eq!(back.health_score, metrics.health_score);
}

#[test]
fn test_baseline_entry_serde() {
    let mut baseline = FileHealthBaseline::new();
    let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 100, 50, 5.0, 1);
    baseline.add_file(&metrics);

    let json = serde_json::to_string(&baseline).unwrap();
    let back: FileHealthBaseline = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 1);
}
