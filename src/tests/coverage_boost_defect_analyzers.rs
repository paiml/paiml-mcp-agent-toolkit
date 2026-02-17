#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for defect_analyzers module
//! Tests for config structs and analyzer creation

use crate::services::defect_analyzer::DefectAnalyzer;
use crate::services::defect_analyzers::{
    ArchitectureConfig, ArchitectureDefectAnalyzer, ComplexityConfig, ComplexityDefectAnalyzer,
    DeadCodeConfig, DeadCodeDefectAnalyzer, DuplicationConfig, DuplicationDefectAnalyzer,
    PerformanceConfig, PerformanceDefectAnalyzer, SATDConfig, SATDDefectAnalyzer,
};

// ============ ComplexityConfig Tests ============

#[test]
fn test_complexity_config_default() {
    let config = ComplexityConfig::default();
    assert_eq!(config.max_tdg_score, 2.0);
    assert_eq!(config.high_threshold, 1.5);
}

#[test]
fn test_complexity_config_custom() {
    let config = ComplexityConfig {
        max_tdg_score: 3.0,
        high_threshold: 2.0,
    };
    assert_eq!(config.max_tdg_score, 3.0);
    assert_eq!(config.high_threshold, 2.0);
}

#[test]
fn test_complexity_config_clone() {
    let config = ComplexityConfig::default();
    let cloned = config.clone();
    assert_eq!(config.max_tdg_score, cloned.max_tdg_score);
}

// ============ SATDConfig Tests ============

#[test]
fn test_satd_config_default() {
    let config = SATDConfig::default();
    assert!(!config.include_test_files);
}

#[test]
fn test_satd_config_custom() {
    let config = SATDConfig {
        include_test_files: true,
    };
    assert!(config.include_test_files);
}

#[test]
fn test_satd_config_clone() {
    let config = SATDConfig {
        include_test_files: true,
    };
    let cloned = config.clone();
    assert_eq!(config.include_test_files, cloned.include_test_files);
}

// ============ DeadCodeConfig Tests ============

#[test]
fn test_dead_code_config_default() {
    let config = DeadCodeConfig::default();
    assert_eq!(config.min_confidence, 0.7);
}

#[test]
fn test_dead_code_config_custom() {
    let config = DeadCodeConfig {
        min_confidence: 0.9,
    };
    assert_eq!(config.min_confidence, 0.9);
}

#[test]
fn test_dead_code_config_clone() {
    let config = DeadCodeConfig::default();
    let cloned = config.clone();
    assert_eq!(config.min_confidence, cloned.min_confidence);
}

// ============ DuplicationConfig Tests ============

#[test]
fn test_duplication_config_default() {
    let config = DuplicationConfig::default();
    assert_eq!(config.min_similarity, 0.8);
}

#[test]
fn test_duplication_config_custom() {
    let config = DuplicationConfig {
        min_similarity: 0.95,
    };
    assert_eq!(config.min_similarity, 0.95);
}

#[test]
fn test_duplication_config_clone() {
    let config = DuplicationConfig::default();
    let cloned = config.clone();
    assert_eq!(config.min_similarity, cloned.min_similarity);
}

// ============ PerformanceConfig Tests ============

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();
    assert!(!config.include_nlogn);
}

#[test]
fn test_performance_config_custom() {
    let config = PerformanceConfig {
        include_nlogn: true,
    };
    assert!(config.include_nlogn);
}

#[test]
fn test_performance_config_clone() {
    let config = PerformanceConfig {
        include_nlogn: true,
    };
    let cloned = config.clone();
    assert_eq!(config.include_nlogn, cloned.include_nlogn);
}

// ============ ArchitectureConfig Tests ============

#[test]
fn test_architecture_config_default() {
    let config = ArchitectureConfig::default();
    assert_eq!(config.max_coupling, 10);
}

#[test]
fn test_architecture_config_custom() {
    let config = ArchitectureConfig { max_coupling: 20 };
    assert_eq!(config.max_coupling, 20);
}

#[test]
fn test_architecture_config_clone() {
    let config = ArchitectureConfig::default();
    let cloned = config.clone();
    assert_eq!(config.max_coupling, cloned.max_coupling);
}

// ============ Analyzer Creation Tests ============

#[test]
fn test_complexity_analyzer_category() {
    let analyzer = ComplexityDefectAnalyzer;
    let category = analyzer.category();
    assert_eq!(
        format!("{:?}", category),
        format!(
            "{:?}",
            crate::models::defect_report::DefectCategory::Complexity
        )
    );
}

#[test]
fn test_complexity_analyzer_supports_incremental() {
    let analyzer = ComplexityDefectAnalyzer;
    assert!(analyzer.supports_incremental());
}

#[test]
fn test_satd_analyzer_new() {
    let analyzer = SATDDefectAnalyzer::new();
    let category = analyzer.category();
    assert_eq!(
        format!("{:?}", category),
        format!(
            "{:?}",
            crate::models::defect_report::DefectCategory::TechnicalDebt
        )
    );
}

#[test]
fn test_satd_analyzer_default() {
    let analyzer = SATDDefectAnalyzer::default();
    assert!(analyzer.supports_incremental());
}

#[test]
fn test_dead_code_analyzer_new() {
    let analyzer = DeadCodeDefectAnalyzer::new();
    let category = analyzer.category();
    assert_eq!(
        format!("{:?}", category),
        format!(
            "{:?}",
            crate::models::defect_report::DefectCategory::DeadCode
        )
    );
}

#[test]
fn test_dead_code_analyzer_default() {
    let analyzer = DeadCodeDefectAnalyzer::default();
    assert!(!analyzer.supports_incremental()); // Requires full graph
}

#[test]
fn test_duplication_analyzer_new() {
    let analyzer = DuplicationDefectAnalyzer::new();
    let category = analyzer.category();
    assert_eq!(
        format!("{:?}", category),
        format!(
            "{:?}",
            crate::models::defect_report::DefectCategory::Duplication
        )
    );
}

#[test]
fn test_duplication_analyzer_default() {
    let analyzer = DuplicationDefectAnalyzer::default();
    assert!(!analyzer.supports_incremental());
}

#[test]
fn test_performance_analyzer_new() {
    let analyzer = PerformanceDefectAnalyzer::new();
    let category = analyzer.category();
    assert_eq!(
        format!("{:?}", category),
        format!(
            "{:?}",
            crate::models::defect_report::DefectCategory::Performance
        )
    );
}

#[test]
fn test_performance_analyzer_default() {
    let analyzer = PerformanceDefectAnalyzer::default();
    assert!(analyzer.supports_incremental());
}

#[test]
fn test_architecture_analyzer_new() {
    let analyzer = ArchitectureDefectAnalyzer::new();
    let category = analyzer.category();
    assert_eq!(
        format!("{:?}", category),
        format!(
            "{:?}",
            crate::models::defect_report::DefectCategory::Architecture
        )
    );
}

#[test]
fn test_architecture_analyzer_default() {
    let analyzer = ArchitectureDefectAnalyzer::default();
    assert!(!analyzer.supports_incremental());
}

// ============ Config Boundary Tests ============

#[test]
fn test_complexity_config_zero_thresholds() {
    let config = ComplexityConfig {
        max_tdg_score: 0.0,
        high_threshold: 0.0,
    };
    assert_eq!(config.max_tdg_score, 0.0);
}

#[test]
fn test_dead_code_config_boundary() {
    let config = DeadCodeConfig {
        min_confidence: 1.0,
    };
    assert_eq!(config.min_confidence, 1.0);
}

#[test]
fn test_duplication_config_boundary() {
    let config = DuplicationConfig {
        min_similarity: 1.0,
    };
    assert_eq!(config.min_similarity, 1.0);
}

#[test]
fn test_architecture_config_zero() {
    let config = ArchitectureConfig { max_coupling: 0 };
    assert_eq!(config.max_coupling, 0);
}

#[test]
fn test_architecture_config_large() {
    let config = ArchitectureConfig { max_coupling: 1000 };
    assert_eq!(config.max_coupling, 1000);
}

// ============ Category Comparison Tests ============

#[test]
fn test_all_analyzers_have_different_categories() {
    use crate::models::defect_report::DefectCategory;

    let complexity = ComplexityDefectAnalyzer;
    let satd = SATDDefectAnalyzer::new();
    let dead_code = DeadCodeDefectAnalyzer::new();
    let duplication = DuplicationDefectAnalyzer::new();
    let performance = PerformanceDefectAnalyzer::new();
    let architecture = ArchitectureDefectAnalyzer::new();

    assert!(matches!(complexity.category(), DefectCategory::Complexity));
    assert!(matches!(satd.category(), DefectCategory::TechnicalDebt));
    assert!(matches!(dead_code.category(), DefectCategory::DeadCode));
    assert!(matches!(
        duplication.category(),
        DefectCategory::Duplication
    ));
    assert!(matches!(
        performance.category(),
        DefectCategory::Performance
    ));
    assert!(matches!(
        architecture.category(),
        DefectCategory::Architecture
    ));
}

#[test]
fn test_incremental_support_variations() {
    let complexity = ComplexityDefectAnalyzer;
    let satd = SATDDefectAnalyzer::new();
    let dead_code = DeadCodeDefectAnalyzer::new();
    let duplication = DuplicationDefectAnalyzer::new();
    let performance = PerformanceDefectAnalyzer::new();
    let architecture = ArchitectureDefectAnalyzer::new();

    // These support incremental analysis
    assert!(complexity.supports_incremental());
    assert!(satd.supports_incremental());
    assert!(performance.supports_incremental());

    // These require full analysis
    assert!(!dead_code.supports_incremental());
    assert!(!duplication.supports_incremental());
    assert!(!architecture.supports_incremental());
}
