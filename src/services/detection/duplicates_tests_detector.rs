// CloneType tests, Detector trait integration tests, and DetectorCapabilities tests.
// Included by duplicates.rs — shares parent module scope (no `use` imports here).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // === CloneType tests ===

    #[test]
    fn test_clone_type_type1() {
        let clone_type = CloneType::Type1 { similarity: 1.0 };

        match clone_type {
            CloneType::Type1 { similarity } => {
                assert!((similarity - 1.0).abs() < 0.001);
            }
            _ => panic!("Expected Type1"),
        }
    }

    #[test]
    fn test_clone_type_type2() {
        let clone_type = CloneType::Type2 {
            similarity: 0.9,
            normalized: true,
        };

        match clone_type {
            CloneType::Type2 {
                similarity,
                normalized,
            } => {
                assert!((similarity - 0.9).abs() < 0.001);
                assert!(normalized);
            }
            _ => panic!("Expected Type2"),
        }
    }

    #[test]
    fn test_clone_type_type3() {
        let clone_type = CloneType::Type3 {
            similarity: 0.8,
            ast_distance: 0.15,
        };

        match clone_type {
            CloneType::Type3 {
                similarity,
                ast_distance,
            } => {
                assert!((similarity - 0.8).abs() < 0.001);
                assert!((ast_distance - 0.15).abs() < 0.001);
            }
            _ => panic!("Expected Type3"),
        }
    }

    #[test]
    fn test_clone_type_serialization_type1() {
        let clone_type = CloneType::Type1 { similarity: 0.99 };
        let json = serde_json::to_string(&clone_type).unwrap();
        assert!(json.contains("Type1"));
        assert!(json.contains("0.99"));
    }

    #[test]
    fn test_clone_type_serialization_type2() {
        let clone_type = CloneType::Type2 {
            similarity: 0.85,
            normalized: false,
        };
        let json = serde_json::to_string(&clone_type).unwrap();
        assert!(json.contains("Type2"));
        assert!(json.contains("normalized"));
    }

    #[test]
    fn test_clone_type_serialization_type3() {
        let clone_type = CloneType::Type3 {
            similarity: 0.75,
            ast_distance: 0.25,
        };
        let json = serde_json::to_string(&clone_type).unwrap();
        assert!(json.contains("Type3"));
        assert!(json.contains("ast_distance"));
    }

    #[test]
    fn test_clone_type_clone() {
        let clone_type = CloneType::Type1 { similarity: 0.95 };
        let cloned = clone_type.clone();

        match (clone_type, cloned) {
            (CloneType::Type1 { similarity: s1 }, CloneType::Type1 { similarity: s2 }) => {
                assert_eq!(s1, s2);
            }
            _ => panic!("Clone type mismatch"),
        }
    }

    #[test]
    fn test_clone_type_debug() {
        let clone_type = CloneType::Type1 { similarity: 0.88 };
        let debug_str = format!("{:?}", clone_type);
        assert!(debug_str.contains("Type1"));
    }

    // === Detector trait tests ===

    #[tokio::test]
    async fn test_detector_detect_content_input() {
        let detector = DuplicateDetector::new();
        let input = DetectionInput::Content("fn test() { let x = 1; }".to_string());
        let config = DetectionConfig::default();

        let result = detector.detect(input, config).await;
        assert!(result.is_ok());

        match result.unwrap() {
            DetectionOutput::Duplicates(dup_result) => {
                assert!(dup_result.duplicates.is_empty());
            }
            _ => panic!("Expected Duplicates output"),
        }
    }

    #[tokio::test]
    async fn test_detector_detect_single_file() {
        let detector = DuplicateDetector::new();
        let input = DetectionInput::SingleFile(PathBuf::from("/nonexistent/file.rs"));
        let config = DetectionConfig::default();

        let result = detector.detect(input, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_detector_detect_multiple_files() {
        let detector = DuplicateDetector::new();
        let input = DetectionInput::MultipleFiles(vec![
            PathBuf::from("/nonexistent/file1.rs"),
            PathBuf::from("/nonexistent/file2.rs"),
        ]);
        let config = DetectionConfig::default();

        let result = detector.detect(input, config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_scan_directory_nonexistent() {
        let detector = DuplicateDetector::new();
        let result = detector.scan_directory_for_files(Path::new("/nonexistent/directory"));
        // Should return empty vec or error depending on implementation
        // For a nonexistent directory, it won't be is_dir() so returns empty
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // === DetectorCapabilities tests ===

    #[test]
    fn test_detector_capabilities_all_fields() {
        let caps = DetectorCapabilities {
            supports_batch: true,
            supports_streaming: true,
            language_agnostic: true,
            requires_ast: true,
        };

        assert!(caps.supports_batch);
        assert!(caps.supports_streaming);
        assert!(caps.language_agnostic);
        assert!(caps.requires_ast);
    }
}
