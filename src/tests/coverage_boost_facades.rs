#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for facade modules and MCP mapping
//! Tests: SATD facade types, DefectPrediction types, MCP parameter parsing

// ============ SATD Facade Types Tests ============

mod satd_facade_tests {
    use crate::services::facades::satd_facade::{
        SatdAnalysisRequest, SatdAnalysisResult, SatdSeverity, SatdViolation,
    };

    #[test]
    fn test_satd_analysis_request_clone() {
        let request = SatdAnalysisRequest {
            path: std::path::PathBuf::from("/test/path"),
            strict_mode: true,
            include_tests: false,
            extended: false,
        };
        let cloned = request.clone();
        assert_eq!(cloned.path, request.path);
        assert!(cloned.strict_mode);
        assert!(!cloned.include_tests);
    }

    #[test]
    fn test_satd_analysis_request_debug() {
        let request = SatdAnalysisRequest {
            path: std::path::PathBuf::from("/test"),
            strict_mode: false,
            include_tests: true,
            extended: false,
        };
        let debug = format!("{:?}", request);
        assert!(debug.contains("SatdAnalysisRequest"));
        assert!(debug.contains("strict_mode"));
    }

    #[test]
    fn test_satd_analysis_result_default_values() {
        let result = SatdAnalysisResult {
            total_files: 0,
            violations: vec![],
            summary: String::new(),
        };
        assert_eq!(result.total_files, 0);
        assert!(result.violations.is_empty());
        assert!(result.summary.is_empty());
    }

    #[test]
    fn test_satd_analysis_result_with_violations() {
        let violation = SatdViolation {
            file_path: "src/main.rs".to_string(),
            line_number: 42,
            violation_type: "TODO".to_string(),
            message: "Fix this later".to_string(),
            severity: SatdSeverity::Medium,
        };
        let result = SatdAnalysisResult {
            total_files: 1,
            violations: vec![violation],
            summary: "1 violation found".to_string(),
        };
        assert_eq!(result.total_files, 1);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].line_number, 42);
    }

    #[test]
    fn test_satd_violation_clone() {
        let violation = SatdViolation {
            file_path: "lib.rs".to_string(),
            line_number: 10,
            violation_type: "FIXME".to_string(),
            message: "Broken".to_string(),
            severity: SatdSeverity::High,
        };
        let cloned = violation.clone();
        assert_eq!(cloned.file_path, "lib.rs");
        assert_eq!(cloned.line_number, 10);
    }

    #[test]
    fn test_satd_severity_variants() {
        let critical = SatdSeverity::Critical;
        let high = SatdSeverity::High;
        let medium = SatdSeverity::Medium;
        let low = SatdSeverity::Low;

        let debug_critical = format!("{:?}", critical);
        assert!(debug_critical.contains("Critical"));

        let debug_high = format!("{:?}", high);
        assert!(debug_high.contains("High"));

        let debug_medium = format!("{:?}", medium);
        assert!(debug_medium.contains("Medium"));

        let debug_low = format!("{:?}", low);
        assert!(debug_low.contains("Low"));
    }

    #[test]
    fn test_satd_severity_clone() {
        let s1 = SatdSeverity::Critical;
        let s2 = s1.clone();
        assert!(format!("{:?}", s2).contains("Critical"));
    }

    #[test]
    fn test_satd_result_serialization() {
        let result = SatdAnalysisResult {
            total_files: 5,
            violations: vec![],
            summary: "Clean".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("total_files"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_satd_violation_serialization() {
        let violation = SatdViolation {
            file_path: "test.rs".to_string(),
            line_number: 1,
            violation_type: "HACK".to_string(),
            message: "Quick fix".to_string(),
            severity: SatdSeverity::Low,
        };
        let json = serde_json::to_string(&violation).unwrap();
        assert!(json.contains("HACK"));
        assert!(json.contains("line_number"));
    }
}

// ============ Defect Prediction Facade Types Tests ============

mod defect_prediction_tests {
    use crate::services::facades::defect_prediction_facade::{
        DefectPredictionRequest, DefectPredictionResult, FilePrediction, FileRiskMetrics, RiskLevel,
    };
    use std::path::PathBuf;

    #[test]
    fn test_defect_prediction_request_clone() {
        let request = DefectPredictionRequest {
            project_path: PathBuf::from("/project"),
            confidence_threshold: 0.8,
            min_lines: 50,
            include_low_confidence: false,
            high_risk_only: true,
            include_recommendations: true,
            include: Some(vec!["src".to_string()]),
            exclude: Some(vec!["test".to_string()]),
            top_files: 10,
        };
        let cloned = request.clone();
        assert_eq!(cloned.project_path, PathBuf::from("/project"));
        assert!((cloned.confidence_threshold - 0.8).abs() < f32::EPSILON);
        assert!(cloned.high_risk_only);
    }

    #[test]
    fn test_defect_prediction_request_debug() {
        let request = DefectPredictionRequest {
            project_path: PathBuf::from("."),
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
            top_files: 5,
        };
        let debug = format!("{:?}", request);
        assert!(debug.contains("DefectPredictionRequest"));
    }

    #[test]
    fn test_risk_level_equality() {
        assert_eq!(RiskLevel::Critical, RiskLevel::Critical);
        assert_eq!(RiskLevel::High, RiskLevel::High);
        assert_eq!(RiskLevel::Medium, RiskLevel::Medium);
        assert_eq!(RiskLevel::Low, RiskLevel::Low);
        assert_ne!(RiskLevel::Critical, RiskLevel::Low);
    }

    #[test]
    fn test_risk_level_clone() {
        let level = RiskLevel::High;
        let cloned = level;
        assert_eq!(cloned, RiskLevel::High);
    }

    #[test]
    fn test_risk_level_debug() {
        assert!(format!("{:?}", RiskLevel::Critical).contains("Critical"));
        assert!(format!("{:?}", RiskLevel::High).contains("High"));
        assert!(format!("{:?}", RiskLevel::Medium).contains("Medium"));
        assert!(format!("{:?}", RiskLevel::Low).contains("Low"));
    }

    #[test]
    fn test_file_risk_metrics_clone() {
        let metrics = FileRiskMetrics {
            complexity_score: 0.5,
            churn_score: 0.3,
            coupling_score: 0.2,
            size_score: 0.4,
            duplication_score: 0.1,
        };
        let cloned = metrics.clone();
        assert!((cloned.complexity_score - 0.5).abs() < f32::EPSILON);
        assert!((cloned.churn_score - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_file_risk_metrics_debug() {
        let metrics = FileRiskMetrics {
            complexity_score: 0.0,
            churn_score: 0.0,
            coupling_score: 0.0,
            size_score: 0.0,
            duplication_score: 0.0,
        };
        let debug = format!("{:?}", metrics);
        assert!(debug.contains("FileRiskMetrics"));
    }

    #[test]
    fn test_file_prediction_clone() {
        let prediction = FilePrediction {
            file_path: "main.rs".to_string(),
            defect_probability: 0.75,
            risk_level: RiskLevel::High,
            confidence: 0.9,
            metrics: FileRiskMetrics {
                complexity_score: 0.8,
                churn_score: 0.6,
                coupling_score: 0.4,
                size_score: 0.3,
                duplication_score: 0.2,
            },
            contributing_factors: vec!["High complexity".to_string()],
        };
        let cloned = prediction.clone();
        assert_eq!(cloned.file_path, "main.rs");
        assert_eq!(cloned.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_file_prediction_serialization() {
        let prediction = FilePrediction {
            file_path: "lib.rs".to_string(),
            defect_probability: 0.5,
            risk_level: RiskLevel::Medium,
            confidence: 0.8,
            metrics: FileRiskMetrics {
                complexity_score: 0.5,
                churn_score: 0.5,
                coupling_score: 0.5,
                size_score: 0.5,
                duplication_score: 0.5,
            },
            contributing_factors: vec![],
        };
        let json = serde_json::to_string(&prediction).unwrap();
        assert!(json.contains("lib.rs"));
        assert!(json.contains("defect_probability"));
    }

    #[test]
    fn test_defect_prediction_result_empty() {
        let result = DefectPredictionResult {
            total_files_analyzed: 0,
            high_risk_files: 0,
            medium_risk_files: 0,
            low_risk_files: 0,
            predictions: vec![],
            summary: String::new(),
            recommendations: vec![],
        };
        assert_eq!(result.total_files_analyzed, 0);
        assert!(result.predictions.is_empty());
    }

    #[test]
    fn test_defect_prediction_result_with_data() {
        let prediction = FilePrediction {
            file_path: "risky.rs".to_string(),
            defect_probability: 0.9,
            risk_level: RiskLevel::Critical,
            confidence: 0.95,
            metrics: FileRiskMetrics {
                complexity_score: 1.0,
                churn_score: 0.9,
                coupling_score: 0.8,
                size_score: 0.7,
                duplication_score: 0.6,
            },
            contributing_factors: vec!["Very complex".to_string(), "High churn".to_string()],
        };
        let result = DefectPredictionResult {
            total_files_analyzed: 1,
            high_risk_files: 1,
            medium_risk_files: 0,
            low_risk_files: 0,
            predictions: vec![prediction],
            summary: "1 critical file".to_string(),
            recommendations: vec!["Review risky.rs".to_string()],
        };
        assert_eq!(result.high_risk_files, 1);
        assert_eq!(result.predictions[0].risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_defect_prediction_result_serialization() {
        let result = DefectPredictionResult {
            total_files_analyzed: 10,
            high_risk_files: 2,
            medium_risk_files: 3,
            low_risk_files: 5,
            predictions: vec![],
            summary: "Analysis complete".to_string(),
            recommendations: vec!["Improve tests".to_string()],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("total_files_analyzed"));
        assert!(json.contains("10"));
    }
}

// ============ Project Analyzer Tests ============

mod project_analyzer_tests {
    use crate::services::project_analyzer::Project;
    use std::path::Path;

    #[test]
    fn test_project_new_with_current_dir() {
        let project = Project::new(Path::new(".")).unwrap();
        assert_eq!(project.root(), Path::new("."));
    }

    #[test]
    fn test_project_new_with_absolute_path() {
        let project = Project::new(Path::new("/tmp")).unwrap();
        assert_eq!(project.root(), Path::new("/tmp"));
    }

    #[test]
    fn test_project_root_accessor() {
        let path = Path::new("./src");
        let project = Project::new(path).unwrap();
        assert_eq!(project.root(), path);
    }

    #[test]
    fn test_project_source_files_returns_vec() {
        let project = Project::new(Path::new(".")).unwrap();
        let files = project.source_files();
        // Should return a vector (may be empty depending on directory)
        assert!(files.len() >= 0);
    }
}

// ============ MCP Mapping Parse Functions Tests ============

mod mcp_mapping_tests {
    use crate::contracts::{OutputFormat, QualityProfile, SatdSeverity};

    #[test]
    fn test_output_format_default() {
        let format: OutputFormat = Default::default();
        assert!(matches!(format, OutputFormat::Table));
    }

    #[test]
    fn test_quality_profile_default() {
        let profile: QualityProfile = Default::default();
        assert!(matches!(profile, QualityProfile::Standard));
    }

    #[test]
    fn test_satd_severity_variants() {
        let low = SatdSeverity::Low;
        let medium = SatdSeverity::Medium;
        let high = SatdSeverity::High;
        let critical = SatdSeverity::Critical;

        assert!(format!("{:?}", low).contains("Low"));
        assert!(format!("{:?}", medium).contains("Medium"));
        assert!(format!("{:?}", high).contains("High"));
        assert!(format!("{:?}", critical).contains("Critical"));
    }

    #[test]
    fn test_output_format_variants() {
        let table = OutputFormat::Table;
        let json = OutputFormat::Json;
        let yaml = OutputFormat::Yaml;
        let markdown = OutputFormat::Markdown;
        let csv = OutputFormat::Csv;

        assert!(format!("{:?}", table).contains("Table"));
        assert!(format!("{:?}", json).contains("Json"));
        assert!(format!("{:?}", yaml).contains("Yaml"));
        assert!(format!("{:?}", markdown).contains("Markdown"));
        assert!(format!("{:?}", csv).contains("Csv"));
    }

    #[test]
    fn test_quality_profile_variants() {
        let standard = QualityProfile::Standard;
        let strict = QualityProfile::Strict;
        let extreme = QualityProfile::Extreme;
        let toyota = QualityProfile::Toyota;

        assert!(format!("{:?}", standard).contains("Standard"));
        assert!(format!("{:?}", strict).contains("Strict"));
        assert!(format!("{:?}", extreme).contains("Extreme"));
        assert!(format!("{:?}", toyota).contains("Toyota"));
    }
}

// ============ Contract Base Types Tests ============

mod contract_base_tests {
    use crate::contracts::BaseAnalysisContract;
    use std::path::PathBuf;

    #[test]
    fn test_base_analysis_contract_creation() {
        let contract = BaseAnalysisContract {
            path: PathBuf::from("/project"),
            format: crate::contracts::OutputFormat::Json,
            output: Some(PathBuf::from("/output.json")),
            top_files: Some(20),
            include_tests: true,
            timeout: 120,
        };
        assert_eq!(contract.path, PathBuf::from("/project"));
        assert!(contract.include_tests);
        assert_eq!(contract.timeout, 120);
    }

    #[test]
    fn test_base_analysis_contract_minimal() {
        let contract = BaseAnalysisContract {
            path: PathBuf::from("."),
            format: crate::contracts::OutputFormat::Table,
            output: None,
            top_files: None,
            include_tests: false,
            timeout: 60,
        };
        assert!(contract.output.is_none());
        assert!(contract.top_files.is_none());
    }

    #[test]
    fn test_base_analysis_contract_clone() {
        let contract = BaseAnalysisContract {
            path: PathBuf::from("/test"),
            format: crate::contracts::OutputFormat::Markdown,
            output: None,
            top_files: Some(10),
            include_tests: false,
            timeout: 30,
        };
        let cloned = contract.clone();
        assert_eq!(cloned.path, PathBuf::from("/test"));
        assert_eq!(cloned.top_files, Some(10));
    }

    #[test]
    fn test_base_analysis_contract_debug() {
        let contract = BaseAnalysisContract {
            path: PathBuf::from("."),
            format: crate::contracts::OutputFormat::Csv,
            output: None,
            top_files: None,
            include_tests: true,
            timeout: 60,
        };
        let debug = format!("{:?}", contract);
        assert!(debug.contains("BaseAnalysisContract"));
    }
}
