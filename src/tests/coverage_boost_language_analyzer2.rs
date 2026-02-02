//! Coverage boost tests for language_analyzer module
//! Tests: LanguageAnalysisRequest, AnalysisType, AnalysisOptions, OutputFormat, FileMetadata

use crate::services::language_analyzer::{
    AnalysisOptions, AnalysisResult, AnalysisType, FileMetadata, LanguageAnalysisRequest,
    LanguageAnalysisResult, LanguageAnalyzer, OutputFormat,
};
use crate::services::language_registry::Language;
use std::path::PathBuf;

// ============ AnalysisType Tests ============

#[test]
fn test_analysis_type_complexity_debug() {
    let t = AnalysisType::Complexity;
    assert!(format!("{:?}", t).contains("Complexity"));
}

#[test]
fn test_analysis_type_satd_debug() {
    let t = AnalysisType::Satd;
    assert!(format!("{:?}", t).contains("Satd"));
}

#[test]
fn test_analysis_type_dead_code_debug() {
    let t = AnalysisType::DeadCode;
    assert!(format!("{:?}", t).contains("DeadCode"));
}

#[test]
fn test_analysis_type_security_debug() {
    let t = AnalysisType::Security;
    assert!(format!("{:?}", t).contains("Security"));
}

#[test]
fn test_analysis_type_style_debug() {
    let t = AnalysisType::Style;
    assert!(format!("{:?}", t).contains("Style"));
}

#[test]
fn test_analysis_type_documentation_debug() {
    let t = AnalysisType::Documentation;
    assert!(format!("{:?}", t).contains("Documentation"));
}

#[test]
fn test_analysis_type_dependencies_debug() {
    let t = AnalysisType::Dependencies;
    assert!(format!("{:?}", t).contains("Dependencies"));
}

#[test]
fn test_analysis_type_metrics_debug() {
    let t = AnalysisType::Metrics;
    assert!(format!("{:?}", t).contains("Metrics"));
}

#[test]
fn test_analysis_type_clone() {
    let t1 = AnalysisType::Complexity;
    let t2 = t1.clone();
    assert!(format!("{:?}", t2).contains("Complexity"));
}

#[test]
fn test_analysis_type_serialization() {
    let types = vec![
        AnalysisType::Complexity,
        AnalysisType::Satd,
        AnalysisType::Security,
    ];
    let json = serde_json::to_string(&types).unwrap();
    assert!(json.contains("complexity"));
    assert!(json.contains("satd"));
    assert!(json.contains("security"));
}

// ============ OutputFormat Tests ============

#[test]
fn test_output_format_json() {
    let f = OutputFormat::Json;
    assert!(format!("{:?}", f).contains("Json"));
}

#[test]
fn test_output_format_yaml() {
    let f = OutputFormat::Yaml;
    assert!(format!("{:?}", f).contains("Yaml"));
}

#[test]
fn test_output_format_plain() {
    let f = OutputFormat::Plain;
    assert!(format!("{:?}", f).contains("Plain"));
}

#[test]
fn test_output_format_markdown() {
    let f = OutputFormat::Markdown;
    assert!(format!("{:?}", f).contains("Markdown"));
}

#[test]
fn test_output_format_clone() {
    let f1 = OutputFormat::Json;
    let f2 = f1.clone();
    assert!(format!("{:?}", f2).contains("Json"));
}

#[test]
fn test_output_format_serialization() {
    let f = OutputFormat::Json;
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("json"));
}

// ============ AnalysisOptions Tests ============

#[test]
fn test_analysis_options_default() {
    let opts = AnalysisOptions::default();
    assert_eq!(opts.complexity_threshold, 20);
    assert!(opts.include_comments);
    assert!(!opts.include_tests);
    assert!(opts.parallel_analysis);
}

#[test]
fn test_analysis_options_custom() {
    let opts = AnalysisOptions {
        complexity_threshold: 15,
        include_comments: false,
        include_tests: true,
        parallel_analysis: false,
        output_format: OutputFormat::Yaml,
    };
    assert_eq!(opts.complexity_threshold, 15);
    assert!(!opts.include_comments);
    assert!(opts.include_tests);
    assert!(!opts.parallel_analysis);
}

#[test]
fn test_analysis_options_clone() {
    let opts1 = AnalysisOptions::default();
    let opts2 = opts1.clone();
    assert_eq!(opts2.complexity_threshold, 20);
}

#[test]
fn test_analysis_options_debug() {
    let opts = AnalysisOptions::default();
    let debug = format!("{:?}", opts);
    assert!(debug.contains("AnalysisOptions"));
    assert!(debug.contains("complexity_threshold"));
}

#[test]
fn test_analysis_options_serialization() {
    let opts = AnalysisOptions::default();
    let json = serde_json::to_string(&opts).unwrap();
    assert!(json.contains("complexity_threshold"));
    assert!(json.contains("20"));
}

// ============ LanguageAnalysisRequest Tests ============

#[test]
fn test_language_analysis_request_creation() {
    let request = LanguageAnalysisRequest {
        path: PathBuf::from("/project/src"),
        language: Some(Language::Rust),
        analysis_types: vec![AnalysisType::Complexity, AnalysisType::Satd],
        options: AnalysisOptions::default(),
    };
    assert_eq!(request.path, PathBuf::from("/project/src"));
    assert!(request.language.is_some());
    assert_eq!(request.analysis_types.len(), 2);
}

#[test]
fn test_language_analysis_request_no_language() {
    let request = LanguageAnalysisRequest {
        path: PathBuf::from("./file.py"),
        language: None,
        analysis_types: vec![AnalysisType::Metrics],
        options: AnalysisOptions::default(),
    };
    assert!(request.language.is_none());
}

#[test]
fn test_language_analysis_request_clone() {
    let request = LanguageAnalysisRequest {
        path: PathBuf::from("test"),
        language: Some(Language::Python),
        analysis_types: vec![],
        options: AnalysisOptions::default(),
    };
    let cloned = request.clone();
    assert_eq!(cloned.path, request.path);
}

#[test]
fn test_language_analysis_request_debug() {
    let request = LanguageAnalysisRequest {
        path: PathBuf::from("."),
        language: None,
        analysis_types: vec![],
        options: AnalysisOptions::default(),
    };
    let debug = format!("{:?}", request);
    assert!(debug.contains("LanguageAnalysisRequest"));
}

// ============ AnalysisResult Tests ============

#[test]
fn test_analysis_result_success() {
    let result = AnalysisResult {
        analysis_type: AnalysisType::Complexity,
        success: true,
        data: serde_json::json!({"score": 15}),
        error: None,
    };
    assert!(result.success);
    assert!(result.error.is_none());
}

#[test]
fn test_analysis_result_failure() {
    let result = AnalysisResult {
        analysis_type: AnalysisType::DeadCode,
        success: false,
        data: serde_json::Value::Null,
        error: Some("Parse error".to_string()),
    };
    assert!(!result.success);
    assert_eq!(result.error, Some("Parse error".to_string()));
}

#[test]
fn test_analysis_result_clone() {
    let result = AnalysisResult {
        analysis_type: AnalysisType::Satd,
        success: true,
        data: serde_json::json!({}),
        error: None,
    };
    let cloned = result.clone();
    assert!(cloned.success);
}

#[test]
fn test_analysis_result_serialization() {
    let result = AnalysisResult {
        analysis_type: AnalysisType::Metrics,
        success: true,
        data: serde_json::json!({"lines": 100}),
        error: None,
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("success"));
    assert!(json.contains("true"));
}

// ============ FileMetadata Tests ============

#[test]
fn test_file_metadata_creation() {
    let meta = FileMetadata {
        lines_total: 100,
        lines_code: 80,
        lines_comment: 15,
        lines_blank: 5,
        file_size_bytes: 2048,
        detected_language: Language::Rust,
        confidence: 0.95,
    };
    assert_eq!(meta.lines_total, 100);
    assert_eq!(meta.lines_code, 80);
    assert_eq!(meta.lines_comment, 15);
    assert_eq!(meta.lines_blank, 5);
}

#[test]
fn test_file_metadata_clone() {
    let meta = FileMetadata {
        lines_total: 50,
        lines_code: 40,
        lines_comment: 5,
        lines_blank: 5,
        file_size_bytes: 1024,
        detected_language: Language::Python,
        confidence: 0.9,
    };
    let cloned = meta.clone();
    assert_eq!(cloned.lines_total, 50);
}

#[test]
fn test_file_metadata_debug() {
    let meta = FileMetadata {
        lines_total: 10,
        lines_code: 8,
        lines_comment: 1,
        lines_blank: 1,
        file_size_bytes: 256,
        detected_language: Language::Unknown,
        confidence: 0.5,
    };
    let debug = format!("{:?}", meta);
    assert!(debug.contains("FileMetadata"));
    assert!(debug.contains("lines_total"));
}

#[test]
fn test_file_metadata_serialization() {
    let meta = FileMetadata {
        lines_total: 200,
        lines_code: 150,
        lines_comment: 30,
        lines_blank: 20,
        file_size_bytes: 4096,
        detected_language: Language::JavaScript,
        confidence: 0.99,
    };
    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains("lines_total"));
    assert!(json.contains("200"));
}

// ============ LanguageAnalysisResult Tests ============

#[test]
fn test_language_analysis_result_creation() {
    let result = LanguageAnalysisResult {
        path: PathBuf::from("test.rs"),
        language: Language::Rust,
        analysis_results: vec![],
        metadata: FileMetadata {
            lines_total: 10,
            lines_code: 8,
            lines_comment: 1,
            lines_blank: 1,
            file_size_bytes: 256,
            detected_language: Language::Rust,
            confidence: 1.0,
        },
        processing_time_ms: 50,
    };
    assert_eq!(result.path, PathBuf::from("test.rs"));
    assert_eq!(result.processing_time_ms, 50);
}

#[test]
fn test_language_analysis_result_clone() {
    let result = LanguageAnalysisResult {
        path: PathBuf::from("lib.rs"),
        language: Language::Rust,
        analysis_results: vec![],
        metadata: FileMetadata {
            lines_total: 5,
            lines_code: 4,
            lines_comment: 0,
            lines_blank: 1,
            file_size_bytes: 128,
            detected_language: Language::Rust,
            confidence: 1.0,
        },
        processing_time_ms: 10,
    };
    let cloned = result.clone();
    assert_eq!(cloned.processing_time_ms, 10);
}

#[test]
fn test_language_analysis_result_serialization() {
    let result = LanguageAnalysisResult {
        path: PathBuf::from("main.py"),
        language: Language::Python,
        analysis_results: vec![],
        metadata: FileMetadata {
            lines_total: 20,
            lines_code: 15,
            lines_comment: 3,
            lines_blank: 2,
            file_size_bytes: 512,
            detected_language: Language::Python,
            confidence: 0.98,
        },
        processing_time_ms: 100,
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("processing_time_ms"));
}

// ============ LanguageAnalyzer Tests ============

#[test]
fn test_language_analyzer_new() {
    let analyzer = LanguageAnalyzer::new();
    // Just verify it creates successfully
    let languages = analyzer.supported_languages();
    assert!(!languages.is_empty());
}

#[test]
fn test_language_analyzer_default() {
    let analyzer = LanguageAnalyzer::default();
    assert!(!analyzer.supported_languages().is_empty());
}

#[test]
fn test_language_analyzer_supports_complexity() {
    let analyzer = LanguageAnalyzer::new();

    // Rust should support complexity
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Complexity));

    // Python should support complexity
    assert!(analyzer.supports_analysis(Language::Python, &AnalysisType::Complexity));
}

#[test]
fn test_language_analyzer_supports_satd() {
    let analyzer = LanguageAnalyzer::new();

    // All languages should support SATD (it's comment-based)
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Satd));
    assert!(analyzer.supports_analysis(Language::Python, &AnalysisType::Satd));
    assert!(analyzer.supports_analysis(Language::Unknown, &AnalysisType::Satd));
}

#[test]
fn test_language_analyzer_supports_metrics() {
    let analyzer = LanguageAnalyzer::new();

    // All languages should support basic metrics
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::Metrics));
    assert!(analyzer.supports_analysis(Language::Unknown, &AnalysisType::Metrics));
}

#[test]
fn test_language_analyzer_supports_dead_code() {
    let analyzer = LanguageAnalyzer::new();

    // Languages with AST support should support dead code
    assert!(analyzer.supports_analysis(Language::Rust, &AnalysisType::DeadCode));
}

#[test]
fn test_language_analyzer_supports_documentation() {
    let analyzer = LanguageAnalyzer::new();

    // Markdown should support documentation analysis
    assert!(analyzer.supports_analysis(Language::Markdown, &AnalysisType::Documentation));
}

#[test]
fn test_language_analyzer_supported_languages() {
    let analyzer = LanguageAnalyzer::new();
    let languages = analyzer.supported_languages();

    // Should have multiple languages
    assert!(languages.len() > 5);
}
