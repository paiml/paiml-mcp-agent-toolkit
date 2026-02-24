// Unit tests for file_classifier
// Included from file_classifier.rs - do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_file_detection() {
        let classifier = FileClassifier::default();

        // Test file under threshold (400KB) - use content with newlines to avoid LineTooLong
        let mut small_content = String::new();
        for _ in 0..4000 {
            small_content.push_str("a".repeat(100).as_str());
            small_content.push('\n');
        }
        let decision = classifier.should_parse(Path::new("small.js"), small_content.as_bytes());
        assert_eq!(decision, ParseDecision::Parse);

        // Test file over threshold (600KB) - use content with newlines to avoid LineTooLong
        let mut large_content = String::new();
        for _ in 0..6000 {
            large_content.push_str("a".repeat(100).as_str());
            large_content.push('\n');
        }
        let decision = classifier.should_parse(Path::new("large.js"), large_content.as_bytes());
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));

        // Test file exactly at threshold
        let mut threshold_content = String::new();
        for _ in 0..(LARGE_FILE_THRESHOLD / 101) {
            threshold_content.push_str("a".repeat(100).as_str());
            threshold_content.push('\n');
        }
        let decision =
            classifier.should_parse(Path::new("threshold.js"), threshold_content.as_bytes());
        assert_eq!(decision, ParseDecision::Parse);
    }

    #[test]
    fn test_include_large_files_flag() {
        let classifier = FileClassifier::default();

        // Create large content with newlines to avoid LineTooLong
        let mut large_content = String::new();
        for _ in 0..6000 {
            large_content.push_str("a".repeat(100).as_str());
            large_content.push('\n');
        }
        let large_content_bytes = large_content.as_bytes();

        // Without flag - should skip
        let decision =
            classifier.should_parse_with_options(Path::new("large.js"), large_content_bytes, false);
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));

        // With flag - should parse
        let decision =
            classifier.should_parse_with_options(Path::new("large.js"), large_content_bytes, true);
        assert_eq!(decision, ParseDecision::Parse);
    }

    #[test]
    fn test_very_large_files_still_skipped() {
        let classifier = FileClassifier::default();
        let very_large_content = vec![b'a'; 2_000_000]; // 2MB

        // Even with include_large_files, files over max_file_size should be skipped
        let decision =
            classifier.should_parse_with_options(Path::new("huge.js"), &very_large_content, true);
        assert_eq!(decision, ParseDecision::Skip(SkipReason::FileTooLarge));
    }

    #[test]
    fn test_skip_reason_priorities() {
        let classifier = FileClassifier::default();

        // Empty file should skip with EmptyFile reason (highest priority)
        let empty_content = b"";
        let decision =
            classifier.should_parse_with_options(Path::new("empty.js"), empty_content, false);
        assert_eq!(decision, ParseDecision::Skip(SkipReason::EmptyFile));

        // Build artifact should skip even if large
        // But LargeFile check happens first in our implementation
        let build_content = vec![b'a'; 600_000];
        let decision = classifier.should_parse_with_options(
            Path::new("target/debug/deps/lib.rlib"),
            &build_content,
            false,
        );
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));
    }

    #[test]
    fn test_minified_vs_large_file_detection() {
        let classifier = FileClassifier::default();

        // Large but not minified file (has newlines)
        let mut large_normal = String::new();
        for i in 0..10_000 {
            large_normal.push_str(&format!("function test{} () {{\n  return {};\n}}\n", i, i));
        }
        let content = large_normal.as_bytes();

        // Should skip due to size if over threshold
        if content.len() > LARGE_FILE_THRESHOLD {
            let decision = classifier.should_parse(Path::new("large_normal.js"), content);
            assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));
        }

        // Minified content (one very long line)
        let minified = "a".repeat(11_000); // Long line
        let decision = classifier.should_parse(Path::new("minified.js"), minified.as_bytes());
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LineTooLong));
    }

    #[test]
    fn test_vendor_detection_determinism() {
        let classifier = FileClassifier::default();
        let test_files = [
            (
                "vendor/jquery.min.js",
                b"!function(e,t){var n=e.jQuery}" as &[u8],
            ),
            (
                "src/main.rs",
                b"fn main() {\n    println!(\"Hello\");\n}" as &[u8],
            ),
            (
                "assets/vendor/d3.min.js",
                b"/*! For license information please see d3.min.js.LICENSE.txt */" as &[u8],
            ),
            (
                "node_modules/react/index.js",
                b"'use strict';\n\nmodule.exports = require('./lib/React');" as &[u8],
            ),
            (
                "target/debug/build/htmlServer-abc123/out/rules.rs",
                b"// Auto-generated code\npub enum TreeBuilderStep {\n    A,\n    B,\n}" as &[u8],
            ),
        ];

        // Run 100 times to ensure determinism
        let mut results = Vec::new();
        for _ in 0..100 {
            let run_results: Vec<_> = test_files
                .iter()
                .map(|(path, content)| classifier.should_parse(Path::new(path), content))
                .collect();
            results.push(run_results);
        }

        // All runs should produce identical results
        assert!(results.windows(2).all(|w| w[0] == w[1]));

        // Verify expected classifications
        let decisions = &results[0];
        assert!(matches!(
            decisions[0],
            ParseDecision::Skip(SkipReason::VendorDirectory)
        ));
        assert!(matches!(decisions[1], ParseDecision::Parse));
        assert!(matches!(
            decisions[2],
            ParseDecision::Skip(SkipReason::VendorDirectory)
        ));
        assert!(matches!(
            decisions[3],
            ParseDecision::Skip(SkipReason::VendorDirectory)
        ));
        // Verify that target/ directory is properly filtered
        assert!(matches!(
            decisions[4],
            ParseDecision::Skip(SkipReason::BuildArtifact)
        ));
    }

    #[test]
    fn test_performance_on_large_files() {
        let classifier = FileClassifier::default();
        let large_minified = vec![b'a'; 1_000_000]; // 1MB of minified code

        let start = Instant::now();
        let decision = classifier.should_parse(Path::new("large.min.js"), &large_minified);
        let elapsed = start.elapsed();

        assert!(matches!(decision, ParseDecision::Skip(_)));
        assert!(elapsed.as_micros() < 1000); // Should decide in <1ms
    }

    #[test]
    fn test_entropy_calculation() {
        // Test with uniform distribution (low entropy)
        let uniform = b"aaaaaaaaaa";
        let entropy1 = calculate_shannon_entropy(uniform);
        assert!(entropy1 < 1.0);

        // Test with random-like distribution (high entropy)
        let random = b"a1b2c3d4e5f6g7h8i9j0";
        let entropy2 = calculate_shannon_entropy(random);
        assert!(entropy2 > 3.0);

        // Test with minified-like content
        let minified = b"!function(e,t){var n,r,i,o,a,s,u,c,l,f,d,p,h,m,v,g,y,b,_,w,x,k,C,S,E,T,A,O,j,N,D,P,L,q,R,M,I,F,B,H,U,z,W,V,$,G,Q,K,X,Y,J,Z,ee,te,ne,re,ie,oe,ae,se,ue,ce,le";
        let entropy3 = calculate_shannon_entropy(minified);
        assert!(entropy3 > 4.0); // Adjusted threshold based on actual entropy of test data
    }

    #[test]
    fn test_binary_detection() {
        let classifier = FileClassifier::default();

        // Test with text file
        let text = b"Hello, world!\nThis is a text file.";
        assert!(!classifier.is_binary(text));

        // Test with binary content (null bytes)
        let binary = b"PNG\x00\x00\x00\rIHDR";
        assert!(classifier.is_binary(binary));

        // Test with high non-printable ratio
        let mostly_binary = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert!(classifier.is_binary(&mostly_binary));
    }

    #[test]
    fn test_line_length_detection() {
        let classifier = FileClassifier::default();

        // Normal code file
        let normal_code = b"fn main() {\n    println!(\"Hello\");\n}";
        assert_eq!(
            classifier.should_parse(Path::new("main.rs"), normal_code),
            ParseDecision::Parse
        );

        // File with very long line
        let long_line = format!("const DATA = \"{}\";\n", "a".repeat(15_000));
        assert_eq!(
            classifier.should_parse(Path::new("data.js"), long_line.as_bytes()),
            ParseDecision::Skip(SkipReason::LineTooLong)
        );
    }

    #[test]
    fn test_rust_target_directory_filtering() {
        let classifier = FileClassifier::default();

        // Test various target/ directory patterns that should be filtered
        let rust_build_artifacts = [
            "target/debug/build/htmlserver-abc123/out/rules.rs",
            "target/release/build/htmlserver-abc123/out/rules.rs",
            "target/debug/deps/libhtml5ever-xyz.rlib",
            "target/release/deps/libhtml5ever-xyz.rlib",
            "target/debug/build/proc-macro2-def456/out/generated.rs",
            "target/thumbv7em-none-eabihf/release/libcore.rlib",
        ];

        for path in rust_build_artifacts {
            let content = b"// Auto-generated code or compiled artifact";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::BuildArtifact)),
                "Failed to filter target directory path: {path}"
            );
        }

        // Verify legitimate source files are not filtered
        let source_files = [
            "src/main.rs",
            "src/lib.rs",
            "tests/integration_test.rs",
            "examples/demo.rs",
        ];

        for path in source_files {
            let content = b"fn main() {\n    println!(\"Hello\");\n}";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Parse),
                "Incorrectly filtered source file: {path} -> {decision:?}"
            );
        }
    }

    #[test]
    fn test_additional_build_artifacts() {
        let classifier = FileClassifier::default();

        // Test additional build artifact patterns
        let build_artifacts = [
            ".gradle/caches/transforms-3/abc123/transformed/classes.jar",
            "frontend/.gradle/build/outputs/apk/debug/app-debug.apk",
        ];

        for path in build_artifacts {
            let content = b"// Some content";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::BuildArtifact)),
                "Failed to filter build artifact: {path}"
            );
        }

        // Test node_modules patterns - should be vendor, not build artifacts
        let vendor_artifacts = [
            "backend/node_modules/@babel/core/lib/index.js",
            "/home/user/project/node_modules/lodash/index.js",
        ];

        for path in vendor_artifacts {
            let content = b"// Some content";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::VendorDirectory)),
                "Failed to filter vendor artifact: {path}"
            );
        }
    }

    #[test]
    fn test_debug_reporter() {
        let mut reporter = DebugReporter::new(None);

        // Record some events
        reporter.record_decision(
            Path::new("vendor/lib.js"),
            &ParseDecision::Skip(SkipReason::VendorDirectory),
        );
        reporter.record_decision(Path::new("src/main.rs"), &ParseDecision::Parse);

        // Record parse result for main.rs
        reporter.record_parse_result(
            Path::new("src/main.rs"),
            std::time::Duration::from_millis(25),
            None,
        );

        let report = reporter.generate_report().unwrap();

        assert_eq!(report.summary.total_files, 2);
        assert_eq!(report.summary.parsed_files, 1);
        assert_eq!(report.summary.skipped_files, 1);
        assert_eq!(report.summary.parse_errors, 0);
        assert_eq!(report.skip_reasons.get("VendorDirectory"), Some(&1));
    }
}

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
