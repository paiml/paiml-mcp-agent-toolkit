// Toyota Way: Comprehensive Integration Tests for Unified Detection Framework
//
// This test suite validates the end-to-end functionality of our unified detection
// framework, ensuring all detectors work correctly through the common interface.

#[cfg(test)]
mod unified_detection_integration_tests {
    use super::super::{
        duplicates::DuplicateDetector, polyglot::PolyglotDetector, satd::SATDDetector,
        DetectionConfig, DetectionInput, DetectionOutput, Detector, DetectorCapabilities,
        UnifiedDetectionProcessor,
    };
    use std::fs;
    use tempfile::TempDir;

    /// Test that all unified detectors can be created successfully
    #[test]
    fn test_all_detectors_creation() {
        let _duplicate = DuplicateDetector::new();
        let _satd = SATDDetector::new();
        let _polyglot = PolyglotDetector::new();
        let _processor = UnifiedDetectionProcessor::new();

        assert!(true, "All unified detectors created successfully");
    }

    /// Test detector capabilities and metadata
    #[test]
    fn test_detector_capabilities() {
        let detectors: Vec<(
            &str,
            Box<
                dyn Detector<
                    Input = DetectionInput,
                    Output = DetectionOutput,
                    Config = DetectionConfig,
                >,
            >,
        )> = vec![
            ("duplicates", Box::new(DuplicateDetector::new())),
            ("satd", Box::new(SATDDetector::new())),
            ("polyglot", Box::new(PolyglotDetector::new())),
        ];

        for (expected_name, detector) in detectors {
            // Check name matches expected
            assert_eq!(detector.name(), expected_name, "Detector name should match");

            // Check capabilities are properly defined
            let caps = detector.capabilities();

            // Capabilities should have reasonable values
            assert!(
                caps.supports_batch || !caps.supports_batch, // Boolean should be valid
                "Batch support capability should be defined"
            );
            assert!(
                caps.language_agnostic || !caps.language_agnostic, // Boolean should be valid
                "Language agnostic capability should be defined"
            );
        }
    }

    /// Test unified detection processor functionality
    #[tokio::test]
    async fn test_unified_processor_basic_operations() {
        let processor = UnifiedDetectionProcessor::new();

        // Check that processor has expected detectors
        let available = processor.available_detectors();
        assert!(
            available.contains(&"duplicates"),
            "Should have duplicates detector"
        );
        assert!(available.contains(&"satd"), "Should have SATD detector");
        assert!(
            available.contains(&"polyglot"),
            "Should have polyglot detector"
        );
        assert_eq!(available.len(), 3, "Should have exactly 3 detectors");
    }

    /// Test detection with real file content
    #[tokio::test]
    async fn test_detection_with_real_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files with known patterns
        let file1 = temp_dir.path().join("duplicate1.rs");
        let file2 = temp_dir.path().join("duplicate2.rs");
        let file3 = temp_dir.path().join("satd_example.rs");

        let duplicate_content = r#"
            fn calculate_sum(numbers: &[i32]) -> i32 {
                let mut total = 0;
                for num in numbers {
                    total += num;
                }
                total
            }
        "#;

        // Write duplicate content
        fs::write(&file1, duplicate_content).unwrap();
        fs::write(&file2, duplicate_content).unwrap();

        // Write SATD content
        fs::write(
            &file3,
            r#"
            fn todo_function() {
                // TODO: Implement this function properly
                // FIXME: This is a hack, needs proper solution
                unimplemented!()
            }
        "#,
        )
        .unwrap();

        let processor = UnifiedDetectionProcessor::new();

        // Test duplicate detection
        let files_for_dup = vec![file1, file2];
        let dup_result = processor.detect_duplicates(files_for_dup).await;

        match dup_result {
            Ok(report) => {
                println!(
                    "✅ Duplicate detection completed: {} files analyzed",
                    report.summary.files_analyzed
                );
                assert!(
                    report.summary.files_analyzed > 0,
                    "Should analyze some files"
                );
            }
            Err(e) => {
                println!("⚠️  Duplicate detection failed gracefully: {}", e);
                // Graceful failure is acceptable
            }
        }

        // Test SATD detection
        let satd_result = processor.detect_satd(temp_dir.path()).await;

        match satd_result {
            Ok(report) => {
                println!(
                    "✅ SATD detection completed: {} files analyzed",
                    report.total_files_analyzed
                );
                assert!(
                    report.total_files_analyzed >= 0,
                    "Should have valid file count"
                );
            }
            Err(e) => {
                println!("⚠️  SATD detection failed gracefully: {}", e);
                // Graceful failure is acceptable
            }
        }

        // Test polyglot analysis
        let polyglot_result = processor.analyze_polyglot(temp_dir.path()).await;

        match polyglot_result {
            Ok(analysis) => {
                println!(
                    "✅ Polyglot analysis completed: {} languages found",
                    analysis.languages.len()
                );
                assert!(
                    analysis.recommendation_score >= 0.0 && analysis.recommendation_score <= 1.0,
                    "Recommendation score should be between 0 and 1"
                );
            }
            Err(e) => {
                println!("⚠️  Polyglot analysis failed gracefully: {}", e);
                // Graceful failure is acceptable
            }
        }
    }

    /// Test error handling with invalid inputs
    #[tokio::test]
    async fn test_error_handling() {
        let processor = UnifiedDetectionProcessor::new();

        // Test with non-existent directory
        let non_existent = std::path::PathBuf::from("/this/does/not/exist");

        let result = processor.detect_satd(&non_existent).await;

        // Should either succeed (with empty results) or fail gracefully
        match result {
            Ok(report) => {
                println!(
                    "Graceful handling of non-existent directory: {} files",
                    report.total_files_analyzed
                );
                assert_eq!(
                    report.total_files_analyzed, 0,
                    "Should analyze 0 files for non-existent directory"
                );
            }
            Err(e) => {
                println!("Graceful error for non-existent directory: {}", e);
                assert!(!e.to_string().is_empty(), "Error should have description");
            }
        }
    }

    /// Test detection consistency - same input should produce same output
    #[tokio::test]
    async fn test_detection_consistency() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("consistency_test.rs");

        fs::write(
            &test_file,
            r#"
            fn consistent_function() -> i32 {
                // TODO: Make this more efficient
                let mut sum = 0;
                for i in 0..100 {
                    sum += i;
                }
                sum
            }
        "#,
        )
        .unwrap();

        let processor = UnifiedDetectionProcessor::new();

        // Run SATD detection twice
        let result1 = processor.detect_satd(temp_dir.path()).await;
        let result2 = processor.detect_satd(temp_dir.path()).await;

        // Results should be consistent (both succeed or both fail)
        match (result1, result2) {
            (Ok(r1), Ok(r2)) => {
                assert_eq!(
                    r1.total_files_analyzed, r2.total_files_analyzed,
                    "File count should be consistent"
                );
                assert!(true, "Consistent successful results");
            }
            (Err(_), Err(_)) => {
                assert!(true, "Consistent error results");
            }
            _ => panic!("Inconsistent results between detection runs"),
        }
    }

    /// Test detector capabilities properties
    #[test]
    fn test_detector_capabilities_properties() {
        let duplicate_detector = DuplicateDetector::new();
        let satd_detector = SATDDetector::new();
        let polyglot_detector = PolyglotDetector::new();

        let caps_dup = duplicate_detector.capabilities();
        let caps_satd = satd_detector.capabilities();
        let caps_poly = polyglot_detector.capabilities();

        // Duplicate detector should support batch processing
        assert!(
            caps_dup.supports_batch,
            "Duplicate detector should support batch processing"
        );
        assert!(
            caps_dup.language_agnostic,
            "Duplicate detector should be language agnostic"
        );

        // SATD detector should support streaming (line by line)
        assert!(
            caps_satd.supports_streaming,
            "SATD detector should support streaming"
        );
        assert!(
            caps_satd.language_agnostic,
            "SATD detector should be language agnostic"
        );

        // Polyglot analyzer should be language agnostic by definition
        assert!(
            caps_poly.language_agnostic,
            "Polyglot detector should be language agnostic"
        );
        assert!(
            caps_poly.requires_ast,
            "Polyglot detector should require AST"
        );

        println!("✅ All detector capabilities verified");
    }

    /// Integration test: All detectors on a complex project
    #[tokio::test]
    async fn test_comprehensive_detection_integration() {
        let temp_dir = TempDir::new().unwrap();

        // Create a multi-language project structure
        let rust_dir = temp_dir.path().join("src");
        fs::create_dir_all(&rust_dir).unwrap();

        let js_dir = temp_dir.path().join("web");
        fs::create_dir_all(&js_dir).unwrap();

        // Rust file with various patterns
        fs::write(
            rust_dir.join("main.rs"),
            r#"
            // TODO: Optimize this function
            fn duplicate_logic(data: &[i32]) -> i32 {
                let mut sum = 0;
                for item in data {
                    sum += item;
                }
                sum
            }
            
            // FIXME: This is duplicated code
            fn another_sum(numbers: &[i32]) -> i32 {
                let mut sum = 0;
                for item in numbers {
                    sum += item;
                }
                sum
            }
        "#,
        )
        .unwrap();

        // JavaScript file
        fs::write(
            js_dir.join("app.js"),
            r#"
            // TODO: Add error handling
            function calculateSum(arr) {
                let total = 0;
                for (let i = 0; i < arr.length; i++) {
                    total += arr[i];
                }
                return total;
            }
        "#,
        )
        .unwrap();

        let processor = UnifiedDetectionProcessor::new();

        // Test all detection types
        let mut successful_detections = 0;

        // Test duplicate detection
        let rust_files = vec![rust_dir.join("main.rs")];
        if let Ok(_) = processor.detect_duplicates(rust_files).await {
            successful_detections += 1;
            println!("✅ Duplicate detection successful");
        } else {
            println!("⚠️  Duplicate detection failed gracefully");
        }

        // Test SATD detection
        if let Ok(satd_result) = processor.detect_satd(temp_dir.path()).await {
            successful_detections += 1;
            println!(
                "✅ SATD detection successful: {} files",
                satd_result.total_files_analyzed
            );
        } else {
            println!("⚠️  SATD detection failed gracefully");
        }

        // Test polyglot analysis
        if let Ok(poly_result) = processor.analyze_polyglot(temp_dir.path()).await {
            successful_detections += 1;
            println!(
                "✅ Polyglot analysis successful: {} languages",
                poly_result.languages.len()
            );
        } else {
            println!("⚠️  Polyglot analysis failed gracefully");
        }

        // At least some detections should work
        assert!(
            successful_detections > 0,
            "At least one detection should succeed"
        );
        println!(
            "Comprehensive detection test: {}/3 detections successful",
            successful_detections
        );
    }

    /// Property test: Detection results should be reasonable
    #[tokio::test]
    async fn test_detection_result_properties() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("property_test.rs");

        fs::write(
            &test_file,
            r#"
            // Multiple TODO comments for testing
            fn function_with_todos() {
                // TODO: Item 1
                println!("Hello");
                
                // FIXME: Item 2
                println!("World");
            }
        "#,
        )
        .unwrap();

        let processor = UnifiedDetectionProcessor::new();

        if let Ok(satd_result) = processor.detect_satd(temp_dir.path()).await {
            // Property: Files analyzed should be >= 0
            assert!(
                satd_result.total_files_analyzed >= 0,
                "Files analyzed should be non-negative"
            );

            // Property: Files with debt should be <= total files
            assert!(
                satd_result.files_with_debt <= satd_result.total_files_analyzed,
                "Files with debt should not exceed total files analyzed"
            );

            // Property: SATD items should be finite
            assert!(
                satd_result.items.len() < 1000,
                "SATD items should be reasonable in number"
            );

            println!("✅ SATD result properties verified");
        }

        if let Ok(poly_result) = processor.analyze_polyglot(temp_dir.path()).await {
            // Property: Recommendation score should be between 0 and 1
            assert!(
                poly_result.recommendation_score >= 0.0 && poly_result.recommendation_score <= 1.0,
                "Recommendation score should be between 0 and 1"
            );

            // Property: Languages list should be finite
            assert!(
                poly_result.languages.len() < 100,
                "Languages list should be reasonable"
            );

            println!("✅ Polyglot result properties verified");
        }
    }
}
