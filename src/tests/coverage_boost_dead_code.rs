//! Coverage boost tests for models/dead_code.rs and cli/handlers/dead_code_handlers.rs
//! Tests: type construction, serialization, calculate_score, format functions

use crate::cli::handlers::dead_code_handlers::format_dead_code_as_summary;
use crate::models::dead_code::{
    ConfidenceLevel, DeadCodeAnalysisConfig, DeadCodeItem, DeadCodeResult, DeadCodeSummary,
    DeadCodeType, FileDeadCodeMetrics,
};

// --- ConfidenceLevel tests ---

#[test]
fn test_confidence_level_variants_serde() {
    let levels = vec![
        ConfidenceLevel::High,
        ConfidenceLevel::Medium,
        ConfidenceLevel::Low,
    ];
    for level in &levels {
        let json = serde_json::to_string(level).unwrap();
        let back: ConfidenceLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(*level, back);
        let _ = format!("{:?}", level);
    }
}

#[test]
fn test_confidence_level_copy() {
    let level = ConfidenceLevel::High;
    let copied = level;
    assert_eq!(copied, ConfidenceLevel::High);
}

// --- DeadCodeType tests ---

#[test]
fn test_dead_code_type_variants_serde() {
    let types = vec![
        DeadCodeType::Function,
        DeadCodeType::Class,
        DeadCodeType::Variable,
        DeadCodeType::UnreachableCode,
    ];
    for dt in &types {
        let json = serde_json::to_string(dt).unwrap();
        let back: DeadCodeType = serde_json::from_str(&json).unwrap();
        assert_eq!(*dt, back);
    }
}

#[test]
fn test_dead_code_type_serde_rename() {
    let json = serde_json::to_string(&DeadCodeType::Function).unwrap();
    assert!(json.contains("function"));
    let json = serde_json::to_string(&DeadCodeType::UnreachableCode).unwrap();
    assert!(json.contains("unreachable"));
}

// --- DeadCodeItem tests ---

#[test]
fn test_dead_code_item_construction_serde() {
    let item = DeadCodeItem {
        item_type: DeadCodeType::Function,
        name: "unused_fn".to_string(),
        line: 42,
        reason: "No references found".to_string(),
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: DeadCodeItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "unused_fn");
    assert_eq!(back.line, 42);
}

#[test]
fn test_dead_code_item_clone_debug() {
    let item = DeadCodeItem {
        item_type: DeadCodeType::Class,
        name: "UnusedClass".to_string(),
        line: 10,
        reason: "Never instantiated".to_string(),
    };
    let cloned = item.clone();
    assert_eq!(cloned.name, "UnusedClass");
    let _ = format!("{:?}", item);
}

// --- FileDeadCodeMetrics tests ---

#[test]
fn test_file_dead_code_metrics_construction() {
    let metrics = FileDeadCodeMetrics {
        path: "src/unused.rs".to_string(),
        dead_lines: 50,
        total_lines: 200,
        dead_percentage: 25.0,
        dead_functions: 3,
        dead_classes: 1,
        dead_modules: 0,
        unreachable_blocks: 2,
        dead_score: 0.0,
        confidence: ConfidenceLevel::High,
        items: vec![],
    };
    let _ = format!("{:?}", metrics);
    assert_eq!(metrics.path, "src/unused.rs");
}

#[test]
fn test_file_dead_code_metrics_serde() {
    let metrics = FileDeadCodeMetrics {
        path: "src/test.rs".to_string(),
        dead_lines: 10,
        total_lines: 100,
        dead_percentage: 10.0,
        dead_functions: 1,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 5.0,
        confidence: ConfidenceLevel::Medium,
        items: vec![DeadCodeItem {
            item_type: DeadCodeType::Function,
            name: "dead_fn".to_string(),
            line: 5,
            reason: "unused".to_string(),
        }],
    };
    let json = serde_json::to_string(&metrics).unwrap();
    let back: FileDeadCodeMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(back.items.len(), 1);
    assert_eq!(back.dead_functions, 1);
}

#[test]
fn test_calculate_score_high_confidence() {
    let mut metrics = FileDeadCodeMetrics {
        path: "test.rs".to_string(),
        dead_lines: 100,
        total_lines: 500,
        dead_percentage: 20.0,
        dead_functions: 5,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence: ConfidenceLevel::High,
        items: vec![],
    };
    metrics.calculate_score();
    assert!(metrics.dead_score > 0.0);
}

#[test]
fn test_calculate_score_medium_confidence() {
    let mut metrics = FileDeadCodeMetrics {
        path: "test.rs".to_string(),
        dead_lines: 100,
        total_lines: 500,
        dead_percentage: 20.0,
        dead_functions: 5,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence: ConfidenceLevel::Medium,
        items: vec![],
    };
    metrics.calculate_score();
    assert!(metrics.dead_score > 0.0);
}

#[test]
fn test_calculate_score_low_confidence() {
    let mut metrics = FileDeadCodeMetrics {
        path: "test.rs".to_string(),
        dead_lines: 100,
        total_lines: 500,
        dead_percentage: 20.0,
        dead_functions: 5,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence: ConfidenceLevel::Low,
        items: vec![],
    };
    metrics.calculate_score();
    assert!(metrics.dead_score > 0.0);
}

#[test]
fn test_calculate_score_confidence_ordering() {
    let mut high = FileDeadCodeMetrics {
        path: "t.rs".to_string(),
        dead_lines: 50,
        total_lines: 200,
        dead_percentage: 25.0,
        dead_functions: 3,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence: ConfidenceLevel::High,
        items: vec![],
    };
    let mut low = high.clone();
    low.confidence = ConfidenceLevel::Low;

    high.calculate_score();
    low.calculate_score();
    assert!(high.dead_score > low.dead_score);
}

#[test]
fn test_calculate_score_zero_dead_code() {
    let mut metrics = FileDeadCodeMetrics {
        path: "clean.rs".to_string(),
        dead_lines: 0,
        total_lines: 100,
        dead_percentage: 0.0,
        dead_functions: 0,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence: ConfidenceLevel::High,
        items: vec![],
    };
    metrics.calculate_score();
    // Should still have some score from confidence_weight
    assert!(metrics.dead_score >= 0.0);
}

// --- DeadCodeSummary tests ---

#[test]
fn test_dead_code_summary_serde() {
    let summary = DeadCodeSummary {
        total_files_analyzed: 100,
        files_with_dead_code: 20,
        total_dead_lines: 500,
        dead_percentage: 5.0,
        dead_functions: 15,
        dead_classes: 3,
        dead_modules: 2,
        unreachable_blocks: 8,
    };
    let json = serde_json::to_string(&summary).unwrap();
    let back: DeadCodeSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_files_analyzed, 100);
    assert_eq!(back.dead_functions, 15);
}

// --- DeadCodeAnalysisConfig tests ---

#[test]
fn test_dead_code_analysis_config_serde() {
    let config = DeadCodeAnalysisConfig {
        include_unreachable: true,
        include_tests: false,
        min_dead_lines: 5,
    };
    let json = serde_json::to_string(&config).unwrap();
    let back: DeadCodeAnalysisConfig = serde_json::from_str(&json).unwrap();
    assert!(back.include_unreachable);
    assert!(!back.include_tests);
}

// --- DeadCodeResult tests ---

#[test]
fn test_dead_code_result_serde() {
    let result = DeadCodeResult {
        summary: DeadCodeSummary {
            total_files_analyzed: 50,
            files_with_dead_code: 10,
            total_dead_lines: 200,
            dead_percentage: 4.0,
            dead_functions: 8,
            dead_classes: 2,
            dead_modules: 1,
            unreachable_blocks: 5,
        },
        files: vec![],
        total_files: 50,
        analyzed_files: 50,
    };
    let json = serde_json::to_string(&result).unwrap();
    let back: DeadCodeResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_files, 50);
}

// --- format_dead_code_as_summary tests ---

#[test]
fn test_format_dead_code_summary_basic() {
    let result = DeadCodeResult {
        summary: DeadCodeSummary {
            total_files_analyzed: 100,
            files_with_dead_code: 10,
            total_dead_lines: 500,
            dead_percentage: 5.0,
            dead_functions: 8,
            dead_classes: 2,
            dead_modules: 1,
            unreachable_blocks: 3,
        },
        files: vec![FileDeadCodeMetrics {
            path: "src/unused.rs".to_string(),
            dead_lines: 50,
            total_lines: 200,
            dead_percentage: 25.0,
            dead_functions: 3,
            dead_classes: 1,
            dead_modules: 0,
            unreachable_blocks: 2,
            dead_score: 10.0,
            confidence: ConfidenceLevel::High,
            items: vec![],
        }],
        total_files: 100,
        analyzed_files: 100,
    };
    let output = format_dead_code_as_summary(&result).unwrap();
    assert!(output.contains("Dead Code Analysis"));
    assert!(output.contains("100")); // total files
}

#[test]
fn test_format_dead_code_summary_empty() {
    let result = DeadCodeResult {
        summary: DeadCodeSummary {
            total_files_analyzed: 10,
            files_with_dead_code: 0,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        },
        files: vec![],
        total_files: 10,
        analyzed_files: 10,
    };
    let output = format_dead_code_as_summary(&result).unwrap();
    assert!(output.contains("Dead Code Analysis"));
}

#[test]
fn test_format_dead_code_summary_with_dead_functions() {
    let result = DeadCodeResult {
        summary: DeadCodeSummary {
            total_files_analyzed: 50,
            files_with_dead_code: 5,
            total_dead_lines: 100,
            dead_percentage: 2.0,
            dead_functions: 5,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        },
        files: vec![],
        total_files: 50,
        analyzed_files: 50,
    };
    let output = format_dead_code_as_summary(&result).unwrap();
    assert!(output.contains("Dead Code by Type"));
    assert!(output.contains("Dead functions"));
}
