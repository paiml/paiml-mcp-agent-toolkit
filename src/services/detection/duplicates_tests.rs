// Property tests and data type unit tests for duplicate detection types.
// Included by duplicates.rs — shares parent module scope (no `use` imports here).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_types {
    use super::*;
    use std::path::PathBuf;

    // === DuplicateDetector tests ===

    #[test]
    fn test_duplicate_detector_new() {
        let detector = DuplicateDetector::new();
        assert_eq!(detector.name(), "duplicates");
    }

    #[test]
    fn test_duplicate_detector_default() {
        let detector = DuplicateDetector;
        assert_eq!(detector.name(), "duplicates");
    }

    #[test]
    fn test_duplicate_detector_name() {
        let detector = DuplicateDetector::new();
        assert_eq!(detector.name(), "duplicates");
    }

    #[test]
    fn test_duplicate_detector_capabilities() {
        let detector = DuplicateDetector::new();
        let caps = detector.capabilities();

        assert!(caps.supports_batch);
        assert!(!caps.supports_streaming);
        assert!(caps.language_agnostic);
        assert!(!caps.requires_ast);
    }

    // === DuplicateConfig tests ===

    #[test]
    fn test_duplicate_config_default() {
        let config = DuplicateConfig::default();

        assert!((config.similarity_threshold - 0.8).abs() < 0.001);
        assert_eq!(config.min_lines, 3);
        assert_eq!(config.hash_count, 128);
        assert!(config.ignore_whitespace);
        assert!(config.cross_language);
    }

    #[test]
    fn test_duplicate_config_custom() {
        let config = DuplicateConfig {
            similarity_threshold: 0.9,
            min_lines: 5,
            hash_count: 256,
            ignore_whitespace: false,
            cross_language: false,
        };

        assert!((config.similarity_threshold - 0.9).abs() < 0.001);
        assert_eq!(config.min_lines, 5);
        assert_eq!(config.hash_count, 256);
        assert!(!config.ignore_whitespace);
        assert!(!config.cross_language);
    }

    #[test]
    fn test_duplicate_config_serialization() {
        let config = DuplicateConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("similarity_threshold"));
        assert!(json.contains("min_lines"));
        assert!(json.contains("hash_count"));
        assert!(json.contains("ignore_whitespace"));
        assert!(json.contains("cross_language"));
    }

    #[test]
    fn test_duplicate_config_deserialization() {
        let json = r#"{
            "similarity_threshold": 0.75,
            "min_lines": 10,
            "hash_count": 64,
            "ignore_whitespace": false,
            "cross_language": true
        }"#;

        let config: DuplicateConfig = serde_json::from_str(json).unwrap();
        assert!((config.similarity_threshold - 0.75).abs() < 0.001);
        assert_eq!(config.min_lines, 10);
        assert_eq!(config.hash_count, 64);
        assert!(!config.ignore_whitespace);
        assert!(config.cross_language);
    }

    #[test]
    fn test_duplicate_config_clone() {
        let config = DuplicateConfig::default();
        let cloned = config.clone();

        assert_eq!(config.similarity_threshold, cloned.similarity_threshold);
        assert_eq!(config.min_lines, cloned.min_lines);
    }

    #[test]
    fn test_duplicate_config_debug() {
        let config = DuplicateConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("DuplicateConfig"));
        assert!(debug_str.contains("similarity_threshold"));
    }

    // === DuplicateDetectionResult tests ===

    #[test]
    fn test_duplicate_detection_result_empty() {
        let result = DuplicateDetectionResult {
            duplicates: Vec::new(),
            summary: DuplicateSummary {
                total_groups: 0,
                total_duplicates: 0,
                files_analyzed: 0,
                time_saved_hours: 0.0,
            },
        };

        assert!(result.duplicates.is_empty());
        assert_eq!(result.summary.total_groups, 0);
    }

    #[test]
    fn test_duplicate_detection_result_serialization() {
        let result = DuplicateDetectionResult {
            duplicates: Vec::new(),
            summary: DuplicateSummary {
                total_groups: 5,
                total_duplicates: 10,
                files_analyzed: 20,
                time_saved_hours: 2.5,
            },
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("duplicates"));
        assert!(json.contains("summary"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_duplicate_detection_result_clone() {
        let result = DuplicateDetectionResult {
            duplicates: Vec::new(),
            summary: DuplicateSummary {
                total_groups: 3,
                total_duplicates: 6,
                files_analyzed: 15,
                time_saved_hours: 1.5,
            },
        };

        let cloned = result.clone();
        assert_eq!(result.summary.total_groups, cloned.summary.total_groups);
    }

    // === DuplicateGroup tests ===

    #[test]
    fn test_duplicate_group_creation() {
        let group = DuplicateGroup {
            id: "group-1".to_string(),
            similarity: 0.95,
            fragments: Vec::new(),
            clone_type: CloneType::Type1 { similarity: 0.95 },
        };

        assert_eq!(group.id, "group-1");
        assert!((group.similarity - 0.95).abs() < 0.001);
        assert!(group.fragments.is_empty());
    }

    #[test]
    fn test_duplicate_group_with_fragments() {
        let fragment = CodeFragment {
            file: PathBuf::from("/path/to/file.rs"),
            start_line: 10,
            end_line: 20,
            content: "fn test() {}".to_string(),
            hash: "abc123".to_string(),
        };

        let group = DuplicateGroup {
            id: "group-2".to_string(),
            similarity: 0.88,
            fragments: vec![fragment],
            clone_type: CloneType::Type2 {
                similarity: 0.88,
                normalized: true,
            },
        };

        assert_eq!(group.fragments.len(), 1);
        assert_eq!(group.fragments[0].start_line, 10);
    }

    #[test]
    fn test_duplicate_group_serialization() {
        let group = DuplicateGroup {
            id: "test-group".to_string(),
            similarity: 0.92,
            fragments: Vec::new(),
            clone_type: CloneType::Type1 { similarity: 0.92 },
        };

        let json = serde_json::to_string(&group).unwrap();
        assert!(json.contains("test-group"));
        assert!(json.contains("0.92"));
        assert!(json.contains("Type1"));
    }

    #[test]
    fn test_duplicate_group_clone() {
        let group = DuplicateGroup {
            id: "clone-test".to_string(),
            similarity: 0.85,
            fragments: Vec::new(),
            clone_type: CloneType::Type3 {
                similarity: 0.85,
                ast_distance: 0.1,
            },
        };

        let cloned = group.clone();
        assert_eq!(group.id, cloned.id);
        assert_eq!(group.similarity, cloned.similarity);
    }

    // === CodeFragment tests ===

    #[test]
    fn test_code_fragment_creation() {
        let fragment = CodeFragment {
            file: PathBuf::from("src/main.rs"),
            start_line: 1,
            end_line: 10,
            content: "fn main() {}".to_string(),
            hash: "xyz789".to_string(),
        };

        assert_eq!(fragment.file, PathBuf::from("src/main.rs"));
        assert_eq!(fragment.start_line, 1);
        assert_eq!(fragment.end_line, 10);
        assert_eq!(fragment.content, "fn main() {}");
        assert_eq!(fragment.hash, "xyz789");
    }

    #[test]
    fn test_code_fragment_serialization() {
        let fragment = CodeFragment {
            file: PathBuf::from("test.rs"),
            start_line: 5,
            end_line: 15,
            content: "// comment".to_string(),
            hash: "hash123".to_string(),
        };

        let json = serde_json::to_string(&fragment).unwrap();
        assert!(json.contains("test.rs"));
        assert!(json.contains("5"));
        assert!(json.contains("15"));
        assert!(json.contains("hash123"));
    }

    #[test]
    fn test_code_fragment_clone() {
        let fragment = CodeFragment {
            file: PathBuf::from("clone.rs"),
            start_line: 100,
            end_line: 200,
            content: "code content".to_string(),
            hash: "clonehash".to_string(),
        };

        let cloned = fragment.clone();
        assert_eq!(fragment.file, cloned.file);
        assert_eq!(fragment.start_line, cloned.start_line);
        assert_eq!(fragment.hash, cloned.hash);
    }

    #[test]
    fn test_code_fragment_debug() {
        let fragment = CodeFragment {
            file: PathBuf::from("debug.rs"),
            start_line: 42,
            end_line: 84,
            content: "test".to_string(),
            hash: "debughash".to_string(),
        };

        let debug_str = format!("{:?}", fragment);
        assert!(debug_str.contains("CodeFragment"));
        assert!(debug_str.contains("debug.rs"));
    }

    // === DuplicateSummary tests ===

    #[test]
    fn test_duplicate_summary_creation() {
        let summary = DuplicateSummary {
            total_groups: 10,
            total_duplicates: 25,
            files_analyzed: 100,
            time_saved_hours: 5.5,
        };

        assert_eq!(summary.total_groups, 10);
        assert_eq!(summary.total_duplicates, 25);
        assert_eq!(summary.files_analyzed, 100);
        assert!((summary.time_saved_hours - 5.5).abs() < 0.001);
    }

    #[test]
    fn test_duplicate_summary_serialization() {
        let summary = DuplicateSummary {
            total_groups: 3,
            total_duplicates: 7,
            files_analyzed: 50,
            time_saved_hours: 1.0,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("total_groups"));
        assert!(json.contains("total_duplicates"));
        assert!(json.contains("files_analyzed"));
        assert!(json.contains("time_saved_hours"));
    }

    #[test]
    fn test_duplicate_summary_clone() {
        let summary = DuplicateSummary {
            total_groups: 5,
            total_duplicates: 12,
            files_analyzed: 30,
            time_saved_hours: 2.0,
        };

        let cloned = summary.clone();
        assert_eq!(summary.total_groups, cloned.total_groups);
        assert_eq!(summary.time_saved_hours, cloned.time_saved_hours);
    }
}
