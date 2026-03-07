#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests_part2 {
    use crate::cli::handlers::comprehensive_analysis_handler::output::{
        format_complexity_section, format_dead_code_section, format_satd_section,
    };
    use crate::cli::handlers::comprehensive_analysis_handler::types::ComprehensiveAnalysisConfig;
    use crate::cli::ComprehensiveOutputFormat;
    use crate::services::facades::complexity_facade::{
        ComplexityAnalysisResult, ComplexityViolation,
    };
    use crate::services::facades::dead_code_facade::{
        DeadCodeAnalysisResult, DeadCodeItem, DeadCodeType,
    };
    use crate::services::facades::satd_facade::{SatdAnalysisResult, SatdSeverity, SatdViolation};
    use std::path::PathBuf;

    fn make_default_config() -> ComprehensiveAnalysisConfig {
        ComprehensiveAnalysisConfig {
            project_path: PathBuf::from("/test/project"),
            file: None,
            files: Vec::new(),
            format: ComprehensiveOutputFormat::Json,
            include_duplicates: false,
            include_dead_code: true,
            include_defects: false,
            include_complexity: true,
            include_tdg: false,
            confidence_threshold: 0.7,
            min_lines: 50,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            executive_summary: true,
            top_files: 10,
        }
    }

    #[test]
    fn test_format_complexity_section() {
        let complexity = ComplexityAnalysisResult {
            total_files: 25,
            violations: vec![ComplexityViolation {
                file_path: "src/main.rs".to_string(),
                function_name: "complex_fn".to_string(),
                line_number: 10,
                complexity: 30,
                complexity_type: "cyclomatic".to_string(),
            }],
            average_complexity: 12.5,
            max_complexity: 30,
            summary: "Test summary".to_string(),
        };
        let mut output = String::new();
        format_complexity_section(&mut output, &complexity).unwrap();
        assert!(output.contains("## Complexity Analysis"));
        assert!(output.contains("**Files Analyzed**: 25"));
        assert!(output.contains("**Average Complexity**: 12.5"));
        assert!(output.contains("**Max Complexity**: 30"));
        assert!(output.contains("**Violations**: 1"));
        assert!(output.contains("### Top Complexity Violations"));
        assert!(output.contains("complex_fn"));
    }

    #[test]
    fn test_format_complexity_section_no_violations() {
        let complexity = ComplexityAnalysisResult {
            total_files: 10,
            violations: vec![],
            average_complexity: 5.0,
            max_complexity: 10,
            summary: "Clean".to_string(),
        };
        let mut output = String::new();
        format_complexity_section(&mut output, &complexity).unwrap();
        assert!(output.contains("**Violations**: 0"));
        assert!(!output.contains("### Top Complexity Violations"));
    }

    #[test]
    fn test_format_dead_code_section() {
        let dead_code = DeadCodeAnalysisResult {
            total_files: 15,
            dead_items: vec![DeadCodeItem {
                file_path: "src/old.rs".to_string(),
                item_name: "old_function".to_string(),
                item_type: DeadCodeType::Function,
                line_number: 50,
                reason: "Never referenced".to_string(),
            }],
            dead_percentage: 3.5,
            summary: "Found dead code".to_string(),
        };
        let mut output = String::new();
        format_dead_code_section(&mut output, &dead_code).unwrap();
        assert!(output.contains("## Dead Code Analysis"));
        assert!(output.contains("**Files Analyzed**: 15"));
        assert!(output.contains("**Dead Items**: 1"));
        assert!(output.contains("**Dead Code %**: 3.5%"));
        assert!(output.contains("old_function"));
    }

    #[test]
    fn test_format_dead_code_section_empty() {
        let dead_code = DeadCodeAnalysisResult {
            total_files: 10,
            dead_items: vec![],
            dead_percentage: 0.0,
            summary: "No dead code".to_string(),
        };
        let mut output = String::new();
        format_dead_code_section(&mut output, &dead_code).unwrap();
        assert!(output.contains("**Dead Items**: 0"));
        assert!(!output.contains("### Dead Code Items"));
    }

    #[test]
    fn test_format_satd_section() {
        let satd = SatdAnalysisResult {
            total_files: 8,
            violations: vec![SatdViolation {
                file_path: "src/hack.rs".to_string(),
                line_number: 15,
                violation_type: "HACK".to_string(),
                message: "Temporary workaround".to_string(),
                severity: SatdSeverity::High,
            }],
            summary: "Found SATD".to_string(),
        };
        let mut output = String::new();
        format_satd_section(&mut output, &satd).unwrap();
        assert!(output.contains("## Technical Debt (SATD) Analysis"));
        assert!(output.contains("**Files Analyzed**: 8"));
        assert!(output.contains("**Violations**: 1"));
        assert!(output.contains("HACK"));
        assert!(output.contains("High"));
    }

    #[test]
    fn test_format_satd_section_empty() {
        let satd = SatdAnalysisResult {
            total_files: 5,
            violations: vec![],
            summary: "No SATD".to_string(),
        };
        let mut output = String::new();
        format_satd_section(&mut output, &satd).unwrap();
        assert!(output.contains("**Violations**: 0"));
        assert!(!output.contains("### SATD Violations"));
    }

    #[test]
    fn test_format_complexity_section_limits_to_five() {
        let violations: Vec<ComplexityViolation> = (0..10)
            .map(|i| ComplexityViolation {
                file_path: format!("src/file{i}.rs"),
                function_name: format!("function{i}"),
                line_number: i,
                complexity: 25 + i as u32,
                complexity_type: "cyclomatic".to_string(),
            })
            .collect();
        let complexity = ComplexityAnalysisResult {
            total_files: 10,
            violations,
            average_complexity: 30.0,
            max_complexity: 34,
            summary: "Many violations".to_string(),
        };
        let mut output = String::new();
        format_complexity_section(&mut output, &complexity).unwrap();
        assert!(output.contains("5. "));
        assert!(!output.contains("6. "));
    }

    #[test]
    fn test_format_dead_code_section_limits_to_five() {
        let dead_items: Vec<DeadCodeItem> = (0..10)
            .map(|i| DeadCodeItem {
                file_path: format!("src/file{i}.rs"),
                item_name: format!("item{i}"),
                item_type: DeadCodeType::Function,
                line_number: i,
                reason: "Unused".to_string(),
            })
            .collect();
        let dead_code = DeadCodeAnalysisResult {
            total_files: 10,
            dead_items,
            dead_percentage: 10.0,
            summary: "Many dead items".to_string(),
        };
        let mut output = String::new();
        format_dead_code_section(&mut output, &dead_code).unwrap();
        assert!(output.contains("5. "));
        assert!(!output.contains("6. "));
    }

    #[test]
    fn test_format_satd_section_limits_to_five() {
        let violations: Vec<SatdViolation> = (0..10)
            .map(|i| SatdViolation {
                file_path: format!("src/file{i}.rs"),
                line_number: i,
                violation_type: "TODO".to_string(),
                message: format!("Message {i}"),
                severity: SatdSeverity::Low,
            })
            .collect();
        let satd = SatdAnalysisResult {
            total_files: 10,
            violations,
            summary: "Many SATD items".to_string(),
        };
        let mut output = String::new();
        format_satd_section(&mut output, &satd).unwrap();
        assert!(output.contains("5. "));
        assert!(!output.contains("6. "));
    }

    #[test]
    fn test_dead_code_type_variants_in_output() {
        let dead_items = vec![
            DeadCodeItem {
                file_path: "test.rs".to_string(),
                item_name: "unused_fn".to_string(),
                item_type: DeadCodeType::Function,
                line_number: 1,
                reason: "test".to_string(),
            },
            DeadCodeItem {
                file_path: "test.rs".to_string(),
                item_name: "UnusedClass".to_string(),
                item_type: DeadCodeType::Class,
                line_number: 10,
                reason: "test".to_string(),
            },
            DeadCodeItem {
                file_path: "test.rs".to_string(),
                item_name: "unused_var".to_string(),
                item_type: DeadCodeType::Variable,
                line_number: 20,
                reason: "test".to_string(),
            },
            DeadCodeItem {
                file_path: "test.rs".to_string(),
                item_name: "unused_import".to_string(),
                item_type: DeadCodeType::Import,
                line_number: 30,
                reason: "test".to_string(),
            },
            DeadCodeItem {
                file_path: "test.rs".to_string(),
                item_name: "unreachable".to_string(),
                item_type: DeadCodeType::UnreachableCode,
                line_number: 40,
                reason: "test".to_string(),
            },
        ];
        let dead_code = DeadCodeAnalysisResult {
            total_files: 1,
            dead_items,
            dead_percentage: 5.0,
            summary: "Test".to_string(),
        };
        let mut output = String::new();
        format_dead_code_section(&mut output, &dead_code).unwrap();
        assert!(output.contains("Function"));
        assert!(output.contains("Class"));
        assert!(output.contains("Variable"));
        assert!(output.contains("Import"));
        assert!(output.contains("UnreachableCode"));
    }

    #[test]
    fn test_satd_severity_variants_in_output() {
        let violations = vec![
            SatdViolation {
                file_path: "test.rs".to_string(),
                line_number: 1,
                violation_type: "FIXME".to_string(),
                message: "Critical".to_string(),
                severity: SatdSeverity::Critical,
            },
            SatdViolation {
                file_path: "test.rs".to_string(),
                line_number: 2,
                violation_type: "TODO".to_string(),
                message: "High".to_string(),
                severity: SatdSeverity::High,
            },
            SatdViolation {
                file_path: "test.rs".to_string(),
                line_number: 3,
                violation_type: "NOTE".to_string(),
                message: "Medium".to_string(),
                severity: SatdSeverity::Medium,
            },
            SatdViolation {
                file_path: "test.rs".to_string(),
                line_number: 4,
                violation_type: "XXX".to_string(),
                message: "Low".to_string(),
                severity: SatdSeverity::Low,
            },
        ];
        let satd = SatdAnalysisResult {
            total_files: 1,
            violations,
            summary: "Test".to_string(),
        };
        let mut output = String::new();
        format_satd_section(&mut output, &satd).unwrap();
        assert!(output.contains("Critical"));
        assert!(output.contains("High"));
        assert!(output.contains("Medium"));
        assert!(output.contains("Low"));
    }

    #[test]
    fn test_create_default_config_fields() {
        let config = make_default_config();
        assert_eq!(config.project_path.to_str().unwrap(), "/test/project");
        assert!(config.include_dead_code);
        assert!(config.include_complexity);
        assert!(!config.include_duplicates);
        assert!(!config.include_defects);
        assert!(!config.include_tdg);
        assert_eq!(config.top_files, 10);
    }
}
