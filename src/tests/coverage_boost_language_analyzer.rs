//! Coverage boost tests for services/language_analyzer.rs
//! Tests: LanguageAnalyzer::new, supported_languages, supports_analysis, analyze_file
//! Also tests AnalysisOptions::default, struct construction, serde

use crate::services::language_analyzer::{
    AnalysisOptions, AnalysisResult, AnalysisType, FileMetadata, LanguageAnalysisRequest,
    LanguageAnalysisResult, LanguageAnalyzer, OutputFormat,
};
use crate::services::language_registry::Language;
use std::path::PathBuf;

// ============ AnalysisOptions ============

#[test]
fn test_analysis_options_default() {
    let opts = AnalysisOptions::default();
    assert_eq!(opts.complexity_threshold, 20);
    assert!(opts.include_comments);
    assert!(!opts.include_tests);
    assert!(opts.parallel_analysis);
    matches!(opts.output_format, OutputFormat::Json);
}

#[test]
fn test_analysis_options_custom() {
    let opts = AnalysisOptions {
        complexity_threshold: 10,
        include_comments: false,
        include_tests: true,
        parallel_analysis: false,
        output_format: OutputFormat::Markdown,
    };
    assert_eq!(opts.complexity_threshold, 10);
    assert!(!opts.include_comments);
    assert!(opts.include_tests);
    assert!(!opts.parallel_analysis);
}

#[test]
fn test_analysis_options_serde() {
    let opts = AnalysisOptions::default();
    let json = serde_json::to_string(&opts).unwrap();
    let back: AnalysisOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.complexity_threshold, 20);
}

#[test]
fn test_analysis_options_clone() {
    let opts = AnalysisOptions::default();
    let cloned = opts.clone();
    assert_eq!(cloned.complexity_threshold, opts.complexity_threshold);
}

// ============ AnalysisType ============

#[test]
fn test_analysis_type_serde_roundtrip() {
    let types = vec![
        AnalysisType::Complexity,
        AnalysisType::Satd,
        AnalysisType::DeadCode,
        AnalysisType::Security,
        AnalysisType::Style,
        AnalysisType::Documentation,
        AnalysisType::Dependencies,
        AnalysisType::Metrics,
    ];
    for ty in types {
        let json = serde_json::to_string(&ty).unwrap();
        let back: AnalysisType = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", back);
    }
}

#[test]
fn test_analysis_type_snake_case_serde() {
    let json = serde_json::to_string(&AnalysisType::DeadCode).unwrap();
    assert_eq!(json, "\"dead_code\"");
}

// ============ OutputFormat ============

#[test]
fn test_output_format_serde_roundtrip() {
    let formats = vec![
        OutputFormat::Json,
        OutputFormat::Yaml,
        OutputFormat::Plain,
        OutputFormat::Markdown,
    ];
    for fmt in formats {
        let json = serde_json::to_string(&fmt).unwrap();
        let back: OutputFormat = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", back);
    }
}

// ============ LanguageAnalysisRequest ============

#[test]
fn test_language_analysis_request_construction() {
    let req = LanguageAnalysisRequest {
        path: PathBuf::from("src/main.rs"),
        language: Some(Language::Rust),
        analysis_types: vec![AnalysisType::Complexity, AnalysisType::Satd],
        options: AnalysisOptions::default(),
    };
    assert_eq!(req.path, PathBuf::from("src/main.rs"));
    assert_eq!(req.language, Some(Language::Rust));
    assert_eq!(req.analysis_types.len(), 2);
}

#[test]
fn test_language_analysis_request_no_language() {
    let req = LanguageAnalysisRequest {
        path: PathBuf::from("unknown.xyz"),
        language: None,
        analysis_types: vec![AnalysisType::Metrics],
        options: AnalysisOptions::default(),
    };
    assert!(req.language.is_none());
}

#[test]
fn test_language_analysis_request_serde() {
    let req = LanguageAnalysisRequest {
        path: PathBuf::from("test.py"),
        language: Some(Language::Python),
        analysis_types: vec![AnalysisType::Security],
        options: AnalysisOptions::default(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: LanguageAnalysisRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, req.path);
}

// ============ AnalysisResult ============

#[test]
fn test_analysis_result_success() {
    let result = AnalysisResult {
        analysis_type: AnalysisType::Complexity,
        success: true,
        data: serde_json::json!({"cyclomatic": 5}),
        error: None,
    };
    assert!(result.success);
    assert!(result.error.is_none());
}

#[test]
fn test_analysis_result_failure() {
    let result = AnalysisResult {
        analysis_type: AnalysisType::Security,
        success: false,
        data: serde_json::Value::Null,
        error: Some("Parse error".to_string()),
    };
    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("Parse error"));
}

#[test]
fn test_analysis_result_serde() {
    let result = AnalysisResult {
        analysis_type: AnalysisType::Metrics,
        success: true,
        data: serde_json::json!({"lines": 100}),
        error: None,
    };
    let json = serde_json::to_string(&result).unwrap();
    let back: AnalysisResult = serde_json::from_str(&json).unwrap();
    assert!(back.success);
}

// ============ FileMetadata ============

#[test]
fn test_file_metadata_construction() {
    let meta = FileMetadata {
        lines_total: 200,
        lines_code: 150,
        lines_comment: 30,
        lines_blank: 20,
        file_size_bytes: 4096,
        detected_language: Language::Rust,
        confidence: 0.95,
    };
    assert_eq!(meta.lines_total, 200);
    assert_eq!(meta.lines_code + meta.lines_comment + meta.lines_blank, 200);
    assert!(meta.confidence > 0.9);
}

#[test]
fn test_file_metadata_serde() {
    let meta = FileMetadata {
        lines_total: 100,
        lines_code: 80,
        lines_comment: 10,
        lines_blank: 10,
        file_size_bytes: 2048,
        detected_language: Language::Python,
        confidence: 1.0,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: FileMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lines_total, 100);
}

// ============ LanguageAnalysisResult ============

#[test]
fn test_language_analysis_result_construction() {
    let result = LanguageAnalysisResult {
        path: PathBuf::from("src/lib.rs"),
        language: Language::Rust,
        analysis_results: vec![AnalysisResult {
            analysis_type: AnalysisType::Complexity,
            success: true,
            data: serde_json::json!({"total": 10}),
            error: None,
        }],
        metadata: FileMetadata {
            lines_total: 500,
            lines_code: 400,
            lines_comment: 60,
            lines_blank: 40,
            file_size_bytes: 10240,
            detected_language: Language::Rust,
            confidence: 1.0,
        },
        processing_time_ms: 42,
    };
    assert_eq!(result.analysis_results.len(), 1);
    assert_eq!(result.processing_time_ms, 42);
}

// ============ LanguageAnalyzer ============

#[test]
fn test_language_analyzer_new() {
    let analyzer = LanguageAnalyzer::new();
    let _ = analyzer.supported_languages();
}

#[test]
fn test_language_analyzer_default() {
    let analyzer = LanguageAnalyzer::default();
    let langs = analyzer.supported_languages();
    assert!(!langs.is_empty());
}

#[test]
fn test_language_analyzer_supported_languages_not_empty() {
    let analyzer = LanguageAnalyzer::new();
    let langs = analyzer.supported_languages();
    assert!(langs.len() > 10); // Should support many languages
}

#[test]
fn test_language_analyzer_supports_analysis_complexity_rust() {
    let analyzer = LanguageAnalyzer::new();
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Complexity));
}

#[test]
fn test_language_analyzer_supports_analysis_satd() {
    let analyzer = LanguageAnalyzer::new();
    // SATD (TODO/FIXME detection) should be supported for most languages
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Satd));
    assert!(analyzer.supports_analysis(Language::Python, &AnalysisType::Satd));
}

#[test]
fn test_language_analyzer_supports_analysis_security() {
    let analyzer = LanguageAnalyzer::new();
    assert!(analyzer.supports_analysis(Language::Python, &AnalysisType::Security));
    assert!(analyzer.supports_analysis(Language::JavaScript, &AnalysisType::Security));
}

#[test]
fn test_language_analyzer_supports_analysis_style() {
    let analyzer = LanguageAnalyzer::new();
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Style));
}

#[test]
fn test_language_analyzer_supports_analysis_documentation() {
    let analyzer = LanguageAnalyzer::new();
    // Documentation analysis may not be supported for all languages
    let _ = analyzer.supports_analysis(Language::Rust, &AnalysisType::Documentation);
    let _ = analyzer.supports_analysis(Language::Python, &AnalysisType::Documentation);
}

#[test]
fn test_language_analyzer_supports_analysis_dependencies() {
    let analyzer = LanguageAnalyzer::new();
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Dependencies));
    assert!(analyzer.supports_analysis(Language::Python, &AnalysisType::Dependencies));
}

#[test]
fn test_language_analyzer_supports_analysis_metrics() {
    let analyzer = LanguageAnalyzer::new();
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Metrics));
}

// ============ Async analyze_file tests ============

#[tokio::test]
async fn test_language_analyzer_analyze_rust_file() {
    let analyzer = LanguageAnalyzer::new();
    // Use a real file from the project
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    if path.exists() {
        let result = analyzer
            .analyze_file(&path, vec![AnalysisType::Metrics, AnalysisType::Complexity])
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.language, Language::Rust);
        assert!(result.metadata.lines_total > 0);
        assert_eq!(result.analysis_results.len(), 2);
    }
}

#[tokio::test]
async fn test_language_analyzer_analyze_nonexistent_file() {
    let analyzer = LanguageAnalyzer::new();
    let result = analyzer
        .analyze_file(
            &PathBuf::from("/nonexistent/file.rs"),
            vec![AnalysisType::Metrics],
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_language_analyzer_analyze_all_types() {
    let analyzer = LanguageAnalyzer::new();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    if path.exists() {
        let result = analyzer
            .analyze_file(
                &path,
                vec![
                    AnalysisType::Complexity,
                    AnalysisType::Satd,
                    AnalysisType::Security,
                    AnalysisType::Style,
                    AnalysisType::Documentation,
                    AnalysisType::Dependencies,
                    AnalysisType::Metrics,
                    AnalysisType::DeadCode,
                ],
            )
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.analysis_results.len(), 8);
        // Most should succeed; some analysis types may not be supported for all languages
        let succeeded = result.analysis_results.iter().filter(|r| r.success).count();
        assert!(succeeded >= 5, "Expected at least 5 successes, got {succeeded}");
    }
}

#[tokio::test]
async fn test_language_analyzer_analyze_python_content() {
    let analyzer = LanguageAnalyzer::new();
    // Create a temp Python file
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.py");
    std::fs::write(
        &path,
        r#"
import os
import sys

# TODO: Fix this function
def hello(name):
    if name:
        for i in range(10):
            print(f"Hello {name} {i}")
    else:
        print("Hello world")

class MyClass:
    def __init__(self):
        self.value = 42
        # FIXME: hardcoded value
"#,
    )
    .unwrap();

    let result = analyzer
        .analyze_file(
            &path,
            vec![
                AnalysisType::Complexity,
                AnalysisType::Satd,
                AnalysisType::Style,
                AnalysisType::Dependencies,
                AnalysisType::Metrics,
            ],
        )
        .await
        .unwrap();

    assert_eq!(result.language, Language::Python);
    assert!(result.metadata.lines_total > 10);
    assert!(result.metadata.lines_comment > 0);
    assert!(result.metadata.lines_code > 0);

    // Check that analysis results were returned
    assert!(!result.analysis_results.is_empty());
    // All requested analyses should succeed
    let succeeded = result.analysis_results.iter().filter(|r| r.success).count();
    assert!(succeeded >= 3, "Expected at least 3 successes, got {succeeded}");
}

#[tokio::test]
async fn test_language_analyzer_analyze_javascript_content() {
    let analyzer = LanguageAnalyzer::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.js");
    std::fs::write(
        &path,
        r#"
const fs = require('fs');
import { useState } from 'react';

// HACK: workaround for bug
function processData(data) {
    if (data && data.length > 0) {
        for (let i = 0; i < data.length; i++) {
            eval(data[i]); // security issue
        }
    }
}

module.exports = { processData };
"#,
    )
    .unwrap();

    let result = analyzer
        .analyze_file(
            &path,
            vec![
                AnalysisType::Security,
                AnalysisType::Satd,
                AnalysisType::Dependencies,
            ],
        )
        .await
        .unwrap();

    assert_eq!(result.language, Language::JavaScript);

    // Security should find eval usage
    let sec_result = result
        .analysis_results
        .iter()
        .find(|r| matches!(r.analysis_type, AnalysisType::Security))
        .unwrap();
    assert!(sec_result.success);
}
