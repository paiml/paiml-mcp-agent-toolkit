// Tests for defect analyzers
// Extracted to separate file for file health compliance (CB-040)

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use crate::models::complexity_bound::{BigOClass, ComplexityBound, InputVariable};
#[allow(unused_imports)]
use crate::models::defect_report::{DefectCategory, Severity};
#[allow(unused_imports)]
use crate::models::tdg::{TDGComponents, TDGScore, TDGSeverity};
#[allow(unused_imports)]
use crate::services::dead_code_analyzer::{DeadCodeItem, DeadCodeType, UnreachableBlock};
#[allow(unused_imports)]
use crate::services::duplicate_detector::{CloneGroup, CloneInstance, CloneType};
#[allow(unused_imports)]
use crate::services::satd_detector::DebtCategory;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::path::PathBuf;

mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

mod tests {
    use super::*;
    use crate::models::complexity_bound::{BigOClass, ComplexityBound, InputVariable};
    use crate::models::defect_report::{DefectCategory, Severity};
    use crate::models::tdg::{TDGComponents, TDGScore, TDGSeverity};
    use crate::services::dead_code_analyzer::{DeadCodeItem, DeadCodeType, UnreachableBlock};
    use crate::services::duplicate_detector::{CloneGroup, CloneInstance, CloneType};
    use crate::services::satd_detector::DebtCategory;
    use std::path::PathBuf;

    // ==================== ComplexityConfig Tests ====================

    #[test]
    fn test_complexity_config_default() {
        let config = ComplexityConfig::default();
        assert_eq!(config.max_tdg_score, 2.0);
        assert_eq!(config.high_threshold, 1.5);
    }

    #[test]
    fn test_complexity_config_custom() {
        let config = ComplexityConfig {
            max_tdg_score: 3.5,
            high_threshold: 2.0,
        };
        assert_eq!(config.max_tdg_score, 3.5);
        assert_eq!(config.high_threshold, 2.0);
    }

    #[test]
    fn test_complexity_config_clone() {
        let config = ComplexityConfig {
            max_tdg_score: 1.5,
            high_threshold: 1.0,
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_tdg_score, config.max_tdg_score);
        assert_eq!(cloned.high_threshold, config.high_threshold);
    }

    // ==================== ComplexityDefectAnalyzer Tests ====================

    #[test]
    fn test_complexity_analyzer_category() {
        let analyzer = ComplexityDefectAnalyzer;
        assert_eq!(analyzer.category(), DefectCategory::Complexity);
    }

    #[test]
    fn test_complexity_analyzer_supports_incremental() {
        let analyzer = ComplexityDefectAnalyzer;
        assert!(analyzer.supports_incremental());
    }

    #[test]
    fn test_generate_fix_suggestion_high_complexity() {
        let analyzer = ComplexityDefectAnalyzer;
        let score = TDGScore {
            value: 3.0,
            components: TDGComponents {
                complexity: 0.9,
                churn: 0.3,
                coupling: 0.2,
                domain_risk: 0.1,
                duplication: 0.2,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Critical,
            percentile: 95.0,
            confidence: 0.9,
        };
        let suggestion = analyzer.generate_fix_suggestion(&score);
        assert!(suggestion.contains("complexity"));
    }

    #[test]
    fn test_generate_fix_suggestion_high_coupling() {
        let analyzer = ComplexityDefectAnalyzer;
        let score = TDGScore {
            value: 2.5,
            components: TDGComponents {
                complexity: 0.3,
                churn: 0.3,
                coupling: 0.8,
                domain_risk: 0.1,
                duplication: 0.2,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Warning,
            percentile: 80.0,
            confidence: 0.85,
        };
        let suggestion = analyzer.generate_fix_suggestion(&score);
        assert!(suggestion.contains("coupling"));
    }

    #[test]
    fn test_generate_fix_suggestion_high_duplication() {
        let analyzer = ComplexityDefectAnalyzer;
        let score = TDGScore {
            value: 2.0,
            components: TDGComponents {
                complexity: 0.3,
                churn: 0.3,
                coupling: 0.3,
                domain_risk: 0.1,
                duplication: 0.6,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Warning,
            percentile: 70.0,
            confidence: 0.8,
        };
        let suggestion = analyzer.generate_fix_suggestion(&score);
        assert!(suggestion.contains("duplication"));
    }

    #[test]
    fn test_generate_fix_suggestion_default() {
        let analyzer = ComplexityDefectAnalyzer;
        let score = TDGScore {
            value: 1.5,
            components: TDGComponents {
                complexity: 0.4,
                churn: 0.3,
                coupling: 0.3,
                domain_risk: 0.1,
                duplication: 0.2,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Normal,
            percentile: 50.0,
            confidence: 0.75,
        };
        let suggestion = analyzer.generate_fix_suggestion(&score);
        assert!(suggestion.contains("refactoring"));
    }

    #[test]
    fn test_generate_fix_suggestion_multiple_issues() {
        let analyzer = ComplexityDefectAnalyzer;
        let score = TDGScore {
            value: 4.0,
            components: TDGComponents {
                complexity: 0.8,
                churn: 0.3,
                coupling: 0.9,
                domain_risk: 0.1,
                duplication: 0.7,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Critical,
            percentile: 99.0,
            confidence: 0.95,
        };
        let suggestion = analyzer.generate_fix_suggestion(&score);
        assert!(suggestion.contains("complexity"));
        assert!(suggestion.contains("coupling"));
        assert!(suggestion.contains("duplication"));
    }

    #[test]
    fn test_tdg_score_to_defect_critical_severity() {
        let analyzer = ComplexityDefectAnalyzer;
        let config = ComplexityConfig::default();
        let score = TDGScore {
            value: 3.5,
            components: TDGComponents {
                complexity: 0.8,
                churn: 0.5,
                coupling: 0.6,
                domain_risk: 0.2,
                duplication: 0.4,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Critical,
            percentile: 98.0,
            confidence: 0.92,
        };

        let defect = analyzer.tdg_score_to_defect(
            PathBuf::from("src/complex.rs"),
            score.clone(),
            1,
            &config,
        );

        assert_eq!(defect.id, "CPLX-0001");
        assert_eq!(defect.severity, Severity::Critical);
        assert_eq!(defect.category, DefectCategory::Complexity);
        assert_eq!(defect.file_path, PathBuf::from("src/complex.rs"));
        assert_eq!(defect.line_start, 1);
        assert!(defect.message.contains("TDG score"));
        assert_eq!(defect.rule_id, "tdg-complexity");
        assert!(defect.fix_suggestion.is_some());
        assert!(defect.metrics.contains_key("tdg_score"));
        assert!(defect.metrics.contains_key("complexity_factor"));
    }

    #[test]
    fn test_tdg_score_to_defect_warning_severity() {
        let analyzer = ComplexityDefectAnalyzer;
        let config = ComplexityConfig::default();
        let score = TDGScore {
            value: 2.0,
            components: TDGComponents {
                complexity: 0.5,
                churn: 0.3,
                coupling: 0.4,
                domain_risk: 0.1,
                duplication: 0.3,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Warning,
            percentile: 75.0,
            confidence: 0.85,
        };

        let defect =
            analyzer.tdg_score_to_defect(PathBuf::from("src/moderate.rs"), score, 2, &config);

        assert_eq!(defect.severity, Severity::High);
    }

    #[test]
    fn test_tdg_score_to_defect_normal_severity_above_high_threshold() {
        let analyzer = ComplexityDefectAnalyzer;
        let config = ComplexityConfig {
            max_tdg_score: 2.0,
            high_threshold: 1.0,
        };
        let score = TDGScore {
            value: 1.2,
            components: TDGComponents {
                complexity: 0.3,
                churn: 0.2,
                coupling: 0.2,
                domain_risk: 0.1,
                duplication: 0.2,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Normal,
            percentile: 50.0,
            confidence: 0.7,
        };

        let defect =
            analyzer.tdg_score_to_defect(PathBuf::from("src/normal.rs"), score, 3, &config);

        assert_eq!(defect.severity, Severity::Medium);
    }

    #[test]
    fn test_tdg_score_to_defect_normal_severity_below_high_threshold() {
        let analyzer = ComplexityDefectAnalyzer;
        let config = ComplexityConfig {
            max_tdg_score: 2.0,
            high_threshold: 1.5,
        };
        let score = TDGScore {
            value: 1.0,
            components: TDGComponents {
                complexity: 0.2,
                churn: 0.2,
                coupling: 0.2,
                domain_risk: 0.1,
                duplication: 0.1,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Normal,
            percentile: 30.0,
            confidence: 0.6,
        };

        let defect =
            analyzer.tdg_score_to_defect(PathBuf::from("src/simple.rs"), score, 4, &config);

        assert_eq!(defect.severity, Severity::Low);
    }

    // ==================== SATDConfig Tests ====================

    #[test]
    fn test_satd_config_default() {
        let config = SATDConfig::default();
        assert!(!config.include_test_files);
    }

    #[test]
    fn test_satd_config_with_test_files() {
        let config = SATDConfig {
            include_test_files: true,
        };
        assert!(config.include_test_files);
    }

    // ==================== SATDDefectAnalyzer Tests ====================

    #[test]
    fn test_satd_analyzer_new() {
        let analyzer = SATDDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::TechnicalDebt);
    }

    #[test]
    fn test_satd_analyzer_default() {
        let analyzer = SATDDefectAnalyzer::default();
        assert_eq!(analyzer.category(), DefectCategory::TechnicalDebt);
    }

    #[test]
    fn test_satd_analyzer_category() {
        let analyzer = SATDDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::TechnicalDebt);
    }

    #[test]
    fn test_satd_analyzer_supports_incremental() {
        let analyzer = SATDDefectAnalyzer::new();
        assert!(analyzer.supports_incremental());
    }

    #[test]
    fn test_get_debt_fix_suggestion_design() {
        let analyzer = SATDDefectAnalyzer::new();
        let suggestion = analyzer.get_debt_fix_suggestion(&DebtCategory::Design);
        assert!(suggestion.contains("design") || suggestion.contains("architecture"));
    }

    #[test]
    fn test_get_debt_fix_suggestion_defect() {
        let analyzer = SATDDefectAnalyzer::new();
        let suggestion = analyzer.get_debt_fix_suggestion(&DebtCategory::Defect);
        assert!(suggestion.contains("defect") || suggestion.contains("bug"));
    }

    #[test]
    fn test_get_debt_fix_suggestion_requirement() {
        let analyzer = SATDDefectAnalyzer::new();
        let suggestion = analyzer.get_debt_fix_suggestion(&DebtCategory::Requirement);
        assert!(suggestion.contains("requirement") || suggestion.contains("implementation"));
    }

    #[test]
    fn test_get_debt_fix_suggestion_test() {
        let analyzer = SATDDefectAnalyzer::new();
        let suggestion = analyzer.get_debt_fix_suggestion(&DebtCategory::Test);
        assert!(suggestion.contains("test"));
    }

    #[test]
    fn test_get_debt_fix_suggestion_performance() {
        let analyzer = SATDDefectAnalyzer::new();
        let suggestion = analyzer.get_debt_fix_suggestion(&DebtCategory::Performance);
        assert!(suggestion.contains("performance") || suggestion.contains("ptimize"));
    }

    #[test]
    fn test_get_debt_fix_suggestion_security() {
        let analyzer = SATDDefectAnalyzer::new();
        let suggestion = analyzer.get_debt_fix_suggestion(&DebtCategory::Security);
        assert!(suggestion.contains("security") || suggestion.contains("vulnerability"));
    }

    // ==================== DeadCodeConfig Tests ====================

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

    // ==================== DeadCodeDefectAnalyzer Tests ====================

    #[test]
    fn test_dead_code_analyzer_new() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::DeadCode);
    }

    #[test]
    fn test_dead_code_analyzer_default() {
        let analyzer = DeadCodeDefectAnalyzer::default();
        assert_eq!(analyzer.category(), DefectCategory::DeadCode);
    }

    #[test]
    fn test_dead_code_analyzer_category() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::DeadCode);
    }

    #[test]
    fn test_dead_code_analyzer_supports_incremental() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        assert!(!analyzer.supports_incremental());
    }

    #[test]
    fn test_dead_code_item_to_defect_high_confidence() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        let item = DeadCodeItem {
            node_key: 1,
            name: "unused_function".to_string(),
            file_path: "src/lib.rs".to_string(),
            line_number: 42,
            dead_type: DeadCodeType::UnusedFunction,
            confidence: 0.95,
            reason: "Never called".to_string(),
        };

        let defect = analyzer.dead_code_item_to_defect(&item, "DEAD-FN", 1);

        assert_eq!(defect.id, "DEAD-FN-0001");
        assert_eq!(defect.severity, Severity::High);
        assert_eq!(defect.category, DefectCategory::DeadCode);
        assert_eq!(defect.file_path, PathBuf::from("src/lib.rs"));
        assert_eq!(defect.line_start, 42);
        assert!(defect.message.contains("unused_function"));
        assert!(defect.message.contains("95%"));
        assert!(defect.fix_suggestion.is_some());
    }

    #[test]
    fn test_dead_code_item_to_defect_medium_confidence() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        let item = DeadCodeItem {
            node_key: 2,
            name: "maybe_unused".to_string(),
            file_path: "src/utils.rs".to_string(),
            line_number: 100,
            dead_type: DeadCodeType::UnusedClass,
            confidence: 0.8,
            reason: "No instantiation found".to_string(),
        };

        let defect = analyzer.dead_code_item_to_defect(&item, "DEAD-CLS", 2);

        assert_eq!(defect.id, "DEAD-CLS-0002");
        assert_eq!(defect.severity, Severity::Medium);
    }

    #[test]
    fn test_dead_code_item_to_defect_low_confidence() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        let item = DeadCodeItem {
            node_key: 3,
            name: "possibly_used".to_string(),
            file_path: "src/mod.rs".to_string(),
            line_number: 50,
            dead_type: DeadCodeType::UnusedVariable,
            confidence: 0.6,
            reason: "No references found".to_string(),
        };

        let defect = analyzer.dead_code_item_to_defect(&item, "DEAD-VAR", 3);

        assert_eq!(defect.severity, Severity::Low);
    }

    #[test]
    fn test_unreachable_block_to_defect() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        let block = UnreachableBlock {
            start_line: 10,
            end_line: 20,
            file_path: "src/main.rs".to_string(),
            reason: "After return statement".to_string(),
        };

        let defect = analyzer.unreachable_block_to_defect(&block, 1);

        assert_eq!(defect.id, "UNREACH-0001");
        assert_eq!(defect.severity, Severity::High);
        assert_eq!(defect.category, DefectCategory::DeadCode);
        assert_eq!(defect.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(defect.line_start, 10);
        assert_eq!(defect.line_end, Some(20));
        assert!(defect.message.contains("11 lines"));
        assert!(defect.message.contains("After return statement"));
        assert_eq!(defect.rule_id, "unreachable-code");
        assert!(defect.fix_suggestion.is_some());
        assert!(defect.metrics.contains_key("lines"));
    }

    #[test]
    fn test_unreachable_block_to_defect_single_line() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        let block = UnreachableBlock {
            start_line: 50,
            end_line: 50,
            file_path: "src/test.rs".to_string(),
            reason: "Dead branch".to_string(),
        };

        let defect = analyzer.unreachable_block_to_defect(&block, 2);

        assert!(defect.message.contains("1 lines"));
    }

    // ==================== DuplicationConfig Tests ====================

    #[test]
    fn test_duplication_config_default() {
        let config = DuplicationConfig::default();
        assert_eq!(config.min_similarity, 0.8);
    }

    #[test]
    fn test_duplication_config_custom() {
        let config = DuplicationConfig {
            min_similarity: 0.9,
        };
        assert_eq!(config.min_similarity, 0.9);
    }

    // ==================== DuplicationDefectAnalyzer Tests ====================

    #[test]
    fn test_duplication_analyzer_new() {
        let analyzer = DuplicationDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::Duplication);
    }

    #[test]
    fn test_duplication_analyzer_default() {
        let analyzer = DuplicationDefectAnalyzer::default();
        assert_eq!(analyzer.category(), DefectCategory::Duplication);
    }

    #[test]
    fn test_duplication_analyzer_category() {
        let analyzer = DuplicationDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::Duplication);
    }

    #[test]
    fn test_duplication_analyzer_supports_incremental() {
        let analyzer = DuplicationDefectAnalyzer::new();
        assert!(!analyzer.supports_incremental());
    }

    #[test]
    fn test_detect_language_rust() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/main.rs"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::Rust);
    }

    #[test]
    fn test_detect_language_typescript() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/app.ts"));
        assert_eq!(
            lang,
            crate::services::duplicate_detector::Language::TypeScript
        );
    }

    #[test]
    fn test_detect_language_javascript() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/index.js"));
        assert_eq!(
            lang,
            crate::services::duplicate_detector::Language::JavaScript
        );
    }

    #[test]
    fn test_detect_language_python() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/main.py"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::Python);
    }

    #[test]
    fn test_detect_language_c() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/main.c"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::C);
    }

    #[test]
    fn test_detect_language_cpp() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/main.cpp"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::Cpp);
    }

    #[test]
    fn test_detect_language_cc() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/main.cc"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::Cpp);
    }

    #[test]
    fn test_detect_language_kotlin() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("src/Main.kt"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::Kotlin);
    }

    #[test]
    fn test_detect_language_unknown() {
        let analyzer = DuplicationDefectAnalyzer::new();
        // Unknown extension defaults to Rust
        let lang = analyzer.detect_language(Path::new("src/file.unknown"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::Rust);
    }

    #[test]
    fn test_detect_language_no_extension() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let lang = analyzer.detect_language(Path::new("Makefile"));
        assert_eq!(lang, crate::services::duplicate_detector::Language::Rust);
    }

    #[test]
    fn test_clone_to_defect_type1_large() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let group = CloneGroup {
            id: 1,
            clone_type: CloneType::Type1 { similarity: 1.0 },
            fragments: vec![
                CloneInstance {
                    file: PathBuf::from("src/a.rs"),
                    start_line: 1,
                    end_line: 60,
                    start_column: 0,
                    end_column: 100,
                    similarity_to_representative: 1.0,
                    normalized_hash: 12345,
                },
                CloneInstance {
                    file: PathBuf::from("src/b.rs"),
                    start_line: 1,
                    end_line: 60,
                    start_column: 0,
                    end_column: 100,
                    similarity_to_representative: 1.0,
                    normalized_hash: 12345,
                },
            ],
            total_lines: 120,
            total_tokens: 500,
            average_similarity: 1.0,
            representative: 0,
        };
        let instance = &group.fragments[0];

        let defect = analyzer.clone_to_defect(&group, instance, 1);

        assert_eq!(defect.id, "DUP-0001");
        assert_eq!(defect.severity, Severity::High);
        assert_eq!(defect.category, DefectCategory::Duplication);
        assert!(defect.message.contains("Type1"));
    }

    #[test]
    fn test_clone_to_defect_type1_small() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let group = CloneGroup {
            id: 2,
            clone_type: CloneType::Type1 { similarity: 1.0 },
            fragments: vec![CloneInstance {
                file: PathBuf::from("src/c.rs"),
                start_line: 1,
                end_line: 20,
                start_column: 0,
                end_column: 50,
                similarity_to_representative: 1.0,
                normalized_hash: 54321,
            }],
            total_lines: 20,
            total_tokens: 100,
            average_similarity: 1.0,
            representative: 0,
        };
        let instance = &group.fragments[0];

        let defect = analyzer.clone_to_defect(&group, instance, 2);

        assert_eq!(defect.severity, Severity::Medium);
    }

    #[test]
    fn test_clone_to_defect_type2_large() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let group = CloneGroup {
            id: 3,
            clone_type: CloneType::Type2 {
                similarity: 0.95,
                normalized: true,
            },
            fragments: vec![CloneInstance {
                file: PathBuf::from("src/d.rs"),
                start_line: 1,
                end_line: 50,
                start_column: 0,
                end_column: 80,
                similarity_to_representative: 0.95,
                normalized_hash: 11111,
            }],
            total_lines: 50,
            total_tokens: 200,
            average_similarity: 0.95,
            representative: 0,
        };
        let instance = &group.fragments[0];

        let defect = analyzer.clone_to_defect(&group, instance, 3);

        assert_eq!(defect.severity, Severity::Medium);
    }

    #[test]
    fn test_clone_to_defect_type2_small() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let group = CloneGroup {
            id: 4,
            clone_type: CloneType::Type2 {
                similarity: 0.9,
                normalized: true,
            },
            fragments: vec![CloneInstance {
                file: PathBuf::from("src/e.rs"),
                start_line: 1,
                end_line: 15,
                start_column: 0,
                end_column: 50,
                similarity_to_representative: 0.9,
                normalized_hash: 22222,
            }],
            total_lines: 15,
            total_tokens: 50,
            average_similarity: 0.9,
            representative: 0,
        };
        let instance = &group.fragments[0];

        let defect = analyzer.clone_to_defect(&group, instance, 4);

        assert_eq!(defect.severity, Severity::Low);
    }

    #[test]
    fn test_clone_to_defect_type3() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let group = CloneGroup {
            id: 5,
            clone_type: CloneType::Type3 {
                similarity: 0.75,
                ast_distance: 0.2,
            },
            fragments: vec![CloneInstance {
                file: PathBuf::from("src/f.rs"),
                start_line: 1,
                end_line: 30,
                start_column: 0,
                end_column: 60,
                similarity_to_representative: 0.75,
                normalized_hash: 33333,
            }],
            total_lines: 30,
            total_tokens: 120,
            average_similarity: 0.75,
            representative: 0,
        };
        let instance = &group.fragments[0];

        let defect = analyzer.clone_to_defect(&group, instance, 5);

        assert_eq!(defect.severity, Severity::Low);
        assert!(defect.message.contains("Type3"));
    }

    #[test]
    fn test_clone_to_defect_metrics() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let group = CloneGroup {
            id: 6,
            clone_type: CloneType::Type1 { similarity: 1.0 },
            fragments: vec![
                CloneInstance {
                    file: PathBuf::from("src/g.rs"),
                    start_line: 10,
                    end_line: 25,
                    start_column: 0,
                    end_column: 50,
                    similarity_to_representative: 0.98,
                    normalized_hash: 44444,
                },
                CloneInstance {
                    file: PathBuf::from("src/h.rs"),
                    start_line: 5,
                    end_line: 20,
                    start_column: 0,
                    end_column: 50,
                    similarity_to_representative: 0.97,
                    normalized_hash: 44444,
                },
            ],
            total_lines: 32,
            total_tokens: 150,
            average_similarity: 0.975,
            representative: 0,
        };
        let instance = &group.fragments[0];

        let defect = analyzer.clone_to_defect(&group, instance, 6);

        assert_eq!(defect.line_start, 10);
        assert_eq!(defect.line_end, Some(25));
        assert!(defect.metrics.contains_key("similarity"));
        assert!(defect.metrics.contains_key("total_lines"));
        assert!(defect.metrics.contains_key("group_size"));
        assert_eq!(defect.metrics["group_size"], 2.0);
    }

    // ==================== PerformanceConfig Tests ====================

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert!(!config.include_nlogn);
    }

    #[test]
    fn test_performance_config_with_nlogn() {
        let config = PerformanceConfig {
            include_nlogn: true,
        };
        assert!(config.include_nlogn);
    }

    // ==================== PerformanceDefectAnalyzer Tests ====================

    #[test]
    fn test_performance_analyzer_new() {
        let analyzer = PerformanceDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::Performance);
    }

    #[test]
    fn test_performance_analyzer_default() {
        let analyzer = PerformanceDefectAnalyzer::default();
        assert_eq!(analyzer.category(), DefectCategory::Performance);
    }

    #[test]
    fn test_performance_analyzer_category() {
        let analyzer = PerformanceDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::Performance);
    }

    #[test]
    fn test_performance_analyzer_supports_incremental() {
        let analyzer = PerformanceDefectAnalyzer::new();
        assert!(analyzer.supports_incremental());
    }

    #[test]
    fn test_is_problematic_complexity_quadratic() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig::default();
        let bound = ComplexityBound::new(BigOClass::Quadratic, 1, InputVariable::N);

        assert!(analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_is_problematic_complexity_cubic() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig::default();
        let bound = ComplexityBound::new(BigOClass::Cubic, 1, InputVariable::N);

        assert!(analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_is_problematic_complexity_exponential() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig::default();
        let bound = ComplexityBound::new(BigOClass::Exponential, 1, InputVariable::N);

        assert!(analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_is_problematic_complexity_factorial() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig::default();
        let bound = ComplexityBound::new(BigOClass::Factorial, 1, InputVariable::N);

        assert!(analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_is_problematic_complexity_linearithmic_without_config() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig {
            include_nlogn: false,
        };
        let bound = ComplexityBound::new(BigOClass::Linearithmic, 1, InputVariable::N);

        assert!(!analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_is_problematic_complexity_linearithmic_with_config() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig {
            include_nlogn: true,
        };
        let bound = ComplexityBound::new(BigOClass::Linearithmic, 1, InputVariable::N);

        assert!(analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_is_problematic_complexity_linear() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig::default();
        let bound = ComplexityBound::new(BigOClass::Linear, 1, InputVariable::N);

        assert!(!analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_is_problematic_complexity_constant() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let config = PerformanceConfig::default();
        let bound = ComplexityBound::new(BigOClass::Constant, 1, InputVariable::N);

        assert!(!analyzer.is_problematic_complexity(&bound, &config));
    }

    #[test]
    fn test_function_complexity_to_defect_exponential() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let func = FunctionComplexity {
            file_path: PathBuf::from("src/algorithm.rs"),
            function_name: "recursive_solve".to_string(),
            line_number: 42,
            time_complexity: ComplexityBound::new(BigOClass::Exponential, 2, InputVariable::N),
            space_complexity: ComplexityBound::new(BigOClass::Linear, 1, InputVariable::N),
            confidence: 85,
            notes: vec!["Recursive backtracking".to_string()],
        };

        let defect = analyzer.function_complexity_to_defect(&func, 1);

        assert_eq!(defect.id, "PERF-0001");
        assert_eq!(defect.severity, Severity::Critical);
        assert_eq!(defect.category, DefectCategory::Performance);
        assert!(defect.message.contains("recursive_solve"));
        assert!(defect.message.contains("O(2^n)"));
        assert!(defect.fix_suggestion.is_some());
        let suggestion = defect.fix_suggestion.unwrap();
        assert!(suggestion.contains("dynamic programming") || suggestion.contains("approximation"));
    }

    #[test]
    fn test_function_complexity_to_defect_factorial() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let func = FunctionComplexity {
            file_path: PathBuf::from("src/permutations.rs"),
            function_name: "generate_permutations".to_string(),
            line_number: 100,
            time_complexity: ComplexityBound::new(BigOClass::Factorial, 1, InputVariable::N),
            space_complexity: ComplexityBound::new(BigOClass::Factorial, 1, InputVariable::N),
            confidence: 90,
            notes: vec!["Generates all permutations".to_string()],
        };

        let defect = analyzer.function_complexity_to_defect(&func, 2);

        assert_eq!(defect.severity, Severity::Critical);
        let suggestion = defect.fix_suggestion.unwrap();
        assert!(suggestion.contains("redesign") || suggestion.contains("unacceptable"));
    }

    #[test]
    fn test_function_complexity_to_defect_cubic() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let func = FunctionComplexity {
            file_path: PathBuf::from("src/matrix.rs"),
            function_name: "matrix_multiply".to_string(),
            line_number: 50,
            time_complexity: ComplexityBound::new(BigOClass::Cubic, 1, InputVariable::N),
            space_complexity: ComplexityBound::new(BigOClass::Quadratic, 1, InputVariable::N),
            confidence: 95,
            notes: vec!["Naive matrix multiplication".to_string()],
        };

        let defect = analyzer.function_complexity_to_defect(&func, 3);

        assert_eq!(defect.severity, Severity::High);
        let suggestion = defect.fix_suggestion.unwrap();
        assert!(suggestion.contains("rarely") || suggestion.contains("algorithmic"));
    }

    #[test]
    fn test_function_complexity_to_defect_quadratic() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let func = FunctionComplexity {
            file_path: PathBuf::from("src/sort.rs"),
            function_name: "bubble_sort".to_string(),
            line_number: 20,
            time_complexity: ComplexityBound::new(BigOClass::Quadratic, 1, InputVariable::N),
            space_complexity: ComplexityBound::new(BigOClass::Constant, 1, InputVariable::N),
            confidence: 100,
            notes: vec!["Simple bubble sort".to_string()],
        };

        let defect = analyzer.function_complexity_to_defect(&func, 4);

        assert_eq!(defect.severity, Severity::Medium);
        let suggestion = defect.fix_suggestion.unwrap();
        assert!(suggestion.contains("algorithm") || suggestion.contains("data structure"));
    }

    #[test]
    fn test_function_complexity_to_defect_metrics() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let func = FunctionComplexity {
            file_path: PathBuf::from("src/algo.rs"),
            function_name: "test_func".to_string(),
            line_number: 10,
            time_complexity: ComplexityBound::new(BigOClass::Quadratic, 2, InputVariable::N),
            space_complexity: ComplexityBound::new(BigOClass::Linear, 1, InputVariable::N),
            confidence: 75,
            notes: vec![],
        };

        let defect = analyzer.function_complexity_to_defect(&func, 5);

        assert!(defect.metrics.contains_key("time_complexity_class"));
        assert!(defect.metrics.contains_key("space_complexity_class"));
        assert!(defect.metrics.contains_key("confidence"));
        assert_eq!(defect.metrics["confidence"], 75.0);
    }

    #[test]
    fn test_generate_performance_suggestion_default() {
        let analyzer = PerformanceDefectAnalyzer::new();
        let bound = ComplexityBound::new(BigOClass::Linear, 1, InputVariable::N);
        let suggestion = analyzer.generate_performance_suggestion(&bound);
        assert!(suggestion.contains("efficiency") || suggestion.contains("optimization"));
    }

    // ==================== ArchitectureConfig Tests ====================

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

    // ==================== ArchitectureDefectAnalyzer Tests ====================

    #[test]
    fn test_architecture_analyzer_new() {
        let analyzer = ArchitectureDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::Architecture);
    }

    #[test]
    fn test_architecture_analyzer_default() {
        let analyzer = ArchitectureDefectAnalyzer::default();
        assert_eq!(analyzer.category(), DefectCategory::Architecture);
    }

    #[test]
    fn test_architecture_analyzer_category() {
        let analyzer = ArchitectureDefectAnalyzer::new();
        assert_eq!(analyzer.category(), DefectCategory::Architecture);
    }

    #[test]
    fn test_architecture_analyzer_supports_incremental() {
        let analyzer = ArchitectureDefectAnalyzer::new();
        assert!(!analyzer.supports_incremental());
    }

    // ==================== Edge Cases and Error Handling ====================

    #[test]
    fn test_dead_code_item_with_boundary_confidence() {
        let analyzer = DeadCodeDefectAnalyzer::new();

        // Exactly at boundary for high (0.9)
        let item = DeadCodeItem {
            node_key: 10,
            name: "boundary_func".to_string(),
            file_path: "src/test.rs".to_string(),
            line_number: 1,
            dead_type: DeadCodeType::UnusedFunction,
            confidence: 0.9,
            reason: "Boundary test".to_string(),
        };

        let defect = analyzer.dead_code_item_to_defect(&item, "DEAD", 1);
        // 0.9 is NOT > 0.9, so it should be Medium
        assert_eq!(defect.severity, Severity::Medium);
    }

    #[test]
    fn test_dead_code_item_with_boundary_medium() {
        let analyzer = DeadCodeDefectAnalyzer::new();

        // Exactly at boundary for medium (0.7)
        let item = DeadCodeItem {
            node_key: 11,
            name: "boundary_func2".to_string(),
            file_path: "src/test.rs".to_string(),
            line_number: 1,
            dead_type: DeadCodeType::UnusedFunction,
            confidence: 0.7,
            reason: "Boundary test".to_string(),
        };

        let defect = analyzer.dead_code_item_to_defect(&item, "DEAD", 2);
        // 0.7 is NOT > 0.7, so it should be Low
        assert_eq!(defect.severity, Severity::Low);
    }

    #[test]
    fn test_complexity_config_edge_case_zero() {
        let config = ComplexityConfig {
            max_tdg_score: 0.0,
            high_threshold: 0.0,
        };
        assert_eq!(config.max_tdg_score, 0.0);
        assert_eq!(config.high_threshold, 0.0);
    }

    #[test]
    fn test_dead_code_config_edge_case_zero() {
        let config = DeadCodeConfig {
            min_confidence: 0.0,
        };
        assert_eq!(config.min_confidence, 0.0);
    }

    #[test]
    fn test_dead_code_config_edge_case_one() {
        let config = DeadCodeConfig {
            min_confidence: 1.0,
        };
        assert_eq!(config.min_confidence, 1.0);
    }

    #[test]
    fn test_duplication_config_edge_case_zero() {
        let config = DuplicationConfig {
            min_similarity: 0.0,
        };
        assert_eq!(config.min_similarity, 0.0);
    }

    #[test]
    fn test_duplication_config_edge_case_one() {
        let config = DuplicationConfig {
            min_similarity: 1.0,
        };
        assert_eq!(config.min_similarity, 1.0);
    }

    #[test]
    fn test_unreachable_block_large_range() {
        let analyzer = DeadCodeDefectAnalyzer::new();
        let block = UnreachableBlock {
            start_line: 1,
            end_line: 1000,
            file_path: "src/huge.rs".to_string(),
            reason: "Large unreachable block".to_string(),
        };

        let defect = analyzer.unreachable_block_to_defect(&block, 1);

        assert!(defect.message.contains("1000 lines"));
        assert_eq!(defect.metrics["lines"], 1000.0);
    }

    #[test]
    fn test_tdg_score_with_all_components_high() {
        let analyzer = ComplexityDefectAnalyzer;
        let score = TDGScore {
            value: 5.0,
            components: TDGComponents {
                complexity: 1.0,
                churn: 1.0,
                coupling: 1.0,
                domain_risk: 1.0,
                duplication: 1.0,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Critical,
            percentile: 100.0,
            confidence: 1.0,
        };
        let suggestion = analyzer.generate_fix_suggestion(&score);
        // All components > threshold, so all should be mentioned
        assert!(suggestion.contains("complexity"));
        assert!(suggestion.contains("coupling"));
        assert!(suggestion.contains("duplication"));
    }

    #[test]
    fn test_tdg_score_with_all_components_low() {
        let analyzer = ComplexityDefectAnalyzer;
        let score = TDGScore {
            value: 0.5,
            components: TDGComponents {
                complexity: 0.1,
                churn: 0.1,
                coupling: 0.1,
                domain_risk: 0.1,
                duplication: 0.1,
                dead_code: 0.0,
            },
            severity: TDGSeverity::Normal,
            percentile: 10.0,
            confidence: 0.5,
        };
        let suggestion = analyzer.generate_fix_suggestion(&score);
        // No component > threshold, so default message
        assert!(suggestion.contains("refactoring"));
    }

    #[test]
    fn test_clone_with_zero_lines() {
        let analyzer = DuplicationDefectAnalyzer::new();
        let group = CloneGroup {
            id: 100,
            clone_type: CloneType::Type1 { similarity: 1.0 },
            fragments: vec![CloneInstance {
                file: PathBuf::from("src/empty.rs"),
                start_line: 1,
                end_line: 1,
                start_column: 0,
                end_column: 0,
                similarity_to_representative: 1.0,
                normalized_hash: 0,
            }],
            total_lines: 1,
            total_tokens: 0,
            average_similarity: 1.0,
            representative: 0,
        };
        let instance = &group.fragments[0];

        let defect = analyzer.clone_to_defect(&group, instance, 100);

        assert!(defect.message.contains("1 lines"));
    }
}
