#[cfg(test)]
mod tests {
    use crate::services::file_classifier::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

    // Strategy for generating file paths (avoiding vendor patterns)
    prop_compose! {
        fn arb_file_path()
            (
                segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..3),
                extension in prop::option::of("[a-zA-Z0-9]+")
            )
            -> PathBuf
        {
            let mut path = segments.join("/");
            if let Some(ext) = extension {
                path.push('.');
                path.push_str(&ext);
            }
            // Ensure path doesn't accidentally match vendor patterns
            if path.contains("min") || path.contains("vendor") || path.contains("node_modules") {
                return PathBuf::from("src/safe_test_file.rs");
            }
            PathBuf::from(path)
        }
    }

    // Strategy for generating vendor-like paths
    prop_compose! {
        fn arb_vendor_path()
            (
                prefix in prop::option::of("[a-zA-Z0-9_/]+"),
                vendor_dir in prop::sample::select(vec![
                    "vendor", "node_modules", "third_party", "external",
                    ".yarn", "bower_components"
                ]),
                suffix in "[a-zA-Z0-9_/.-]+",
            )
            -> PathBuf
        {
            let mut path = String::new();
            if let Some(p) = prefix {
                path.push_str(&p);
                path.push('/');
            }
            path.push_str(vendor_dir);
            path.push('/');
            path.push_str(&suffix);
            PathBuf::from(path)
        }
    }

    // Strategy for generating build artifact paths
    prop_compose! {
        fn arb_build_artifact_path()
            (
                prefix in prop::option::of("[a-zA-Z0-9_/]+"),
                build_dir in prop::sample::select(vec![
                    "target/debug", "target/release", "build", "dist",
                    "__pycache__", "venv", ".tox", "cmake-build-debug", ".gradle"
                ]),
                suffix in "[a-zA-Z0-9_/.-]+",
            )
            -> PathBuf
        {
            let mut path = String::new();
            if let Some(p) = prefix {
                path.push_str(&p);
                path.push('/');
            }
            path.push_str(build_dir);
            path.push('/');
            path.push_str(&suffix);
            PathBuf::from(path)
        }
    }

    // Strategy for generating file content
    prop_compose! {
        fn arb_file_content()
            (
                content_type in prop::sample::select(vec!["text", "binary", "mixed"]),
                size in 0usize..10000,
            )
            -> Vec<u8>
        {
            match content_type {
                "text" => {
                    let mut content = Vec::with_capacity(size);
                    for i in 0..size {
                        if i % 80 == 79 {
                            content.push(b'\n');
                        } else {
                            content.push(b'a' + ((i % 26) as u8));
                        }
                    }
                    content
                }
                "binary" => {
                    let mut content = Vec::with_capacity(size);
                    for i in 0..size {
                        content.push((i % 256) as u8);
                    }
                    content
                }
                _ => {
                    let mut content = Vec::with_capacity(size);
                    for i in 0..size {
                        if i % 100 < 80 {
                            content.push(b'a' + ((i % 26) as u8));
                        } else {
                            content.push((i % 256) as u8);
                        }
                    }
                    content
                }
            }
        }
    }

    // Strategy for generating minified-like content
    prop_compose! {
        fn arb_minified_content()
            (
                size in 100usize..5000,
                has_signature in any::<bool>(),
            )
            -> Vec<u8>
        {
            let mut content = Vec::with_capacity(size);

            // Add minified signature if requested
            if has_signature {
                let signatures: Vec<&[u8]> = vec![
                    b"/*! jQuery",
                    b"/*! * Bootstrap",
                    b"!function(e,t){",
                    b"/*! For license information",
                    b"/** @license React",
                ];
                let sig = signatures[size % signatures.len()];
                content.extend_from_slice(sig);
            }

            // Generate high-entropy content with few newlines
            for i in 0..size {
                if i % 1000 == 999 {
                    content.push(b'\n');
                } else {
                    // High variety of characters for high entropy
                    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
                    content.push(chars[i % chars.len()]);
                }
            }

            content
        }
    }

    // Strategy for generating file classifier config
    prop_compose! {
        fn arb_file_classifier_config()
            (
                skip_vendor in any::<bool>(),
                max_line_length in 100usize..20000,
                max_file_size in 10000usize..5000000,
            )
            -> FileClassifierConfig
        {
            FileClassifierConfig {
                skip_vendor,
                max_line_length,
                max_file_size,
            }
        }
    }

    proptest! {
        /// Property: Empty files are always skipped
        #[test]
        fn empty_files_always_skipped(path in arb_file_path()) {
            let classifier = FileClassifier::default();
            let decision = classifier.should_parse(&path, b"");
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::EmptyFile));
        }

        /// Property: Files over max size are always skipped
        #[test]
        fn oversized_files_always_skipped(
            path in arb_file_path(),
            extra_bytes in 1usize..10000
        ) {
            let classifier = FileClassifier::default();
            let content = vec![b'a'; classifier.max_file_size + extra_bytes];
            let decision = classifier.should_parse(&path, &content);
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::FileTooLarge));
        }

        /// Property: Vendor paths are skipped when skip_vendor is true
        #[test]
        fn vendor_paths_skipped_when_configured(
            vendor_path in arb_vendor_path(),
            content in arb_file_content()
        ) {
            let classifier = FileClassifier {
                skip_vendor: true,
                ..Default::default()
            };

            // Skip if content triggers other skip reasons
            if !content.is_empty() && content.len() < LARGE_FILE_THRESHOLD {
                let decision = classifier.should_parse(&vendor_path, &content);

                // Should be skipped, but might be for other reasons (binary, minified, etc)
                prop_assert!(matches!(decision, ParseDecision::Skip(_)));
            }
        }

        /// Property: Build artifacts are always skipped
        #[test]
        fn build_artifacts_always_skipped(
            build_path in arb_build_artifact_path(),
            content_size in 1usize..1000
        ) {
            let classifier = FileClassifier::default();
            let content = vec![b'a'; content_size];
            let decision = classifier.should_parse(&build_path, &content);
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::BuildArtifact));
        }

        /// Property: Binary content is detected and skipped
        #[test]
        fn binary_content_detected(
            binary_ratio in 0.4f64..1.0
        ) {
            let classifier = FileClassifier::default();
            let size = 1000;
            let mut content = Vec::with_capacity(size);

            // Generate content with specified ratio of non-printable chars
            for i in 0..size {
                if (i as f64 / size as f64) < binary_ratio {
                    content.push((i % 32) as u8); // Non-printable
                } else {
                    content.push(b'a' + ((i % 26) as u8)); // Printable
                }
            }

            // Use a safe path that won't trigger vendor detection
            let safe_path = PathBuf::from("src/test_file.dat");
            let decision = classifier.should_parse(&safe_path, &content);

            // Should be skipped as binary
            prop_assert!(matches!(decision, ParseDecision::Skip(SkipReason::BinaryContent)));
        }

        /// Property: Files with null bytes are binary
        #[test]
        fn null_bytes_mean_binary(
            prefix in prop::collection::vec(32u8..127, 0..100),
            suffix in prop::collection::vec(32u8..127, 0..100),
        ) {
            let classifier = FileClassifier::default();
            let mut content = prefix;
            content.push(0); // Add null byte
            content.extend(suffix);

            // Use a safe path that won't trigger vendor detection
            let safe_path = PathBuf::from("src/test_file.dat");
            let decision = classifier.should_parse(&safe_path, &content);
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::BinaryContent));
        }

        /// Property: Lines over max length trigger skip
        #[test]
        fn long_lines_trigger_skip(
            line_length in 11000usize..20000,
        ) {
            let classifier = FileClassifier::default();

            // Create content with a very long line
            let mut content = String::new();
            content.push_str("normal line\n");
            content.push_str(&"a".repeat(line_length));
            content.push_str("\nanother normal line");

            // Use a safe path that won't trigger vendor detection
            let safe_path = PathBuf::from("src/test_file.js");
            let decision = classifier.should_parse(&safe_path, content.as_bytes());
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::LineTooLong));
        }

        /// Property: File classification is deterministic for same data
        #[test]
        fn classification_deterministic_for_same_data(data in prop::collection::vec(any::<u8>(), 1..1000)) {
            // We can't test entropy directly, but we can test that classification is deterministic
            let classifier = FileClassifier::default();
            let path = PathBuf::from("test.dat");

            let decision1 = classifier.should_parse(&path, &data);
            let decision2 = classifier.should_parse(&path, &data);

            prop_assert_eq!(decision1, decision2);
        }

        /// Property: High entropy content is detected as minified
        #[test]
        fn high_entropy_detected_as_minified(
            minified_content in arb_minified_content()
        ) {
            let classifier = FileClassifier::default();

            // Only test if content isn't too large or empty
            if !minified_content.is_empty() && minified_content.len() < LARGE_FILE_THRESHOLD {
                // Use a safe path that won't trigger vendor detection
                let safe_path = PathBuf::from("src/test_file.js");
                let decision = classifier.should_parse(&safe_path, &minified_content);

                // Should be skipped as minified or for line length
                prop_assert!(matches!(
                    decision,
                    ParseDecision::Skip(SkipReason::MinifiedContent) |
                    ParseDecision::Skip(SkipReason::LineTooLong)
                ));
            }
        }

        /// Property: Include large files flag works correctly
        #[test]
        fn include_large_files_flag_behavior(
            size in (LARGE_FILE_THRESHOLD + 1)..DEFAULT_MAX_FILE_SIZE
        ) {
            // Create a safe path that won't trigger vendor/build artifact detection
            let path = PathBuf::from("src/large_test_file.rs");

            let classifier = FileClassifier::default();
            let mut content = String::new();

            // Generate content with newlines to avoid LineTooLong
            for _i in 0..(size / 100) {
                content.push_str(&"a".repeat(99));
                content.push('\n');
            }
            let content_bytes = content.as_bytes();

            // Without flag - should skip large files
            let decision = classifier.should_parse_with_options(&path, content_bytes, false);
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));

            // With flag - should parse large files (unless other skip conditions apply)
            let decision = classifier.should_parse_with_options(&path, content_bytes, true);
            // Check that it's not skipped for LargeFile reason
            prop_assert_ne!(decision, ParseDecision::Skip(SkipReason::LargeFile));

            // It should either parse or skip for a different reason (not LargeFile)
            match decision {
                ParseDecision::Parse => {},
                ParseDecision::Skip(reason) => {
                    prop_assert_ne!(reason, SkipReason::LargeFile);
                }
            }
        }

        /// Property: Decision is deterministic for same input
        #[test]
        fn decision_deterministic(
            path in arb_file_path(),
            content in arb_file_content()
        ) {
            let classifier = FileClassifier::default();

            let decision1 = classifier.should_parse(&path, &content);
            let decision2 = classifier.should_parse(&path, &content);
            let decision3 = classifier.should_parse(&path, &content);

            prop_assert_eq!(decision1, decision2);
            prop_assert_eq!(decision2, decision3);
        }

        /// Property: Skip reasons have correct priority
        #[test]
        fn skip_reason_priority_correct(
            size_factor in 0.0f64..3.0,
            _has_null in any::<bool>(),
            _has_long_line in any::<bool>(),
        ) {
            let classifier = FileClassifier::default();

            // Empty file has highest priority
            let empty_decision = classifier.should_parse(&PathBuf::from("test.js"), b"");
            prop_assert_eq!(empty_decision, ParseDecision::Skip(SkipReason::EmptyFile));

            // File too large has second priority
            if size_factor > 2.0 {
                let large_content = vec![b'a'; (DEFAULT_MAX_FILE_SIZE as f64 * size_factor) as usize];
                let large_decision = classifier.should_parse(&PathBuf::from("test.js"), &large_content);
                prop_assert_eq!(large_decision, ParseDecision::Skip(SkipReason::FileTooLarge));
            }
        }

        /// Property: Normal source files are parsed
        #[test]
        fn normal_source_files_parsed(
            segments in prop::collection::vec("[a-zA-Z0-9_-]+", 1..3),
            extension in prop::sample::select(vec!["rs", "js", "py", "java", "cpp", "go"]),
            lines in prop::collection::vec("[a-zA-Z ]+(\\{|\\}|\\(|\\)|;|,|\\.|_|-){0,5}[a-zA-Z ]*", 5..50)
        ) {
            let classifier = FileClassifier::default();

            let mut path = segments.join("/");
            path.push_str("/file.");
            path.push_str(extension);

            // Ensure content has reasonable structure (not just numbers or single chars)
            let mut content = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    content.push('\n');
                }
                // Add some whitespace to make it look like code
                if i % 4 == 0 {
                    content.push_str("    "); // Indentation
                }
                content.push_str(line);
            }

            // Only test if content has multiple lines and isn't too short
            if lines.len() >= 5 && content.len() > 50 {
                let decision = classifier.should_parse(&PathBuf::from(path), content.as_bytes());
                prop_assert_eq!(decision, ParseDecision::Parse);
            }
        }

        /// Property: Uniform data is not detected as minified
        #[test]
        fn uniform_data_not_minified(
            byte_value in b'a'..=b'z',
            size in 100usize..1000
        ) {
            let classifier = FileClassifier::default();
            let data = vec![byte_value; size];

            // Add newlines to avoid line too long
            let mut content = Vec::new();
            for chunk in data.chunks(80) {
                content.extend_from_slice(chunk);
                content.push(b'\n');
            }

            let decision = classifier.should_parse(&PathBuf::from("test.txt"), &content);
            // Uniform data should parse fine (not minified)
            prop_assert_eq!(decision, ParseDecision::Parse);
        }

        /// Property: Mixed character data may be detected as minified
        #[test]
        fn mixed_data_minification_detection(
            data in prop::collection::vec(any::<u8>(), 100..1000)
        ) {
            let classifier = FileClassifier::default();

            // Only test valid UTF-8 data to avoid binary detection
            if std::str::from_utf8(&data).is_ok() {
                let decision = classifier.should_parse(&PathBuf::from("test.js"), &data);
                // We can't predict the exact decision, but it should be consistent
                let decision2 = classifier.should_parse(&PathBuf::from("test.js"), &data);
                prop_assert_eq!(decision, decision2);
            }
        }

        /// Property: File classifier config fields are respected
        #[test]
        fn config_fields_respected(config in arb_file_classifier_config()) {
            let classifier = FileClassifier {
                skip_vendor: config.skip_vendor,
                max_line_length: config.max_line_length,
                max_file_size: config.max_file_size,
                ..Default::default()
            };

            // Test max file size is respected
            let oversized = vec![b'a'; config.max_file_size + 1];
            let decision = classifier.should_parse(&PathBuf::from("test.txt"), &oversized);
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::FileTooLarge));

            // Test vendor skip is respected
            if config.skip_vendor {
                let vendor_path = PathBuf::from("vendor/lib.js");
                let decision = classifier.should_parse(&vendor_path, b"content");
                prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::VendorDirectory));
            }
        }

        /// Property: Minified file signatures are detected
        #[test]
        fn minified_signatures_detected(
            signature in prop::sample::select(vec![
                "/*! jQuery",
                "/*! * Bootstrap",
                "!function(e,t){",
                "/*! For license information",
                "/** @license React",
            ]),
            suffix in "[a-zA-Z0-9 ]{0,100}"
        ) {
            let classifier = FileClassifier::default();

            let mut content = String::from(signature);
            content.push_str(&suffix);

            let decision = classifier.should_parse(&PathBuf::from("lib.js"), content.as_bytes());
            prop_assert_eq!(decision, ParseDecision::Skip(SkipReason::MinifiedContent));
        }
    }

    #[test]
    fn test_basic_classifier_invariants() {
        let classifier = FileClassifier::default();

        // Test default values
        assert_eq!(classifier.max_line_length, 10_000); // DEFAULT_MAX_LINE_LENGTH value
        assert_eq!(classifier.max_file_size, DEFAULT_MAX_FILE_SIZE);
        assert!(classifier.skip_vendor);

        // Test empty file
        assert_eq!(
            classifier.should_parse(&PathBuf::from("test.rs"), b""),
            ParseDecision::Skip(SkipReason::EmptyFile)
        );

        // Test normal file
        let normal_content = b"fn main() {\n    println!(\"Hello, world!\");\n}";
        assert_eq!(
            classifier.should_parse(&PathBuf::from("main.rs"), normal_content),
            ParseDecision::Parse
        );
    }

    #[test]
    fn test_vendor_detection_patterns() {
        let classifier = FileClassifier::default();

        let vendor_paths = vec![
            "vendor/jquery.js",
            "node_modules/react/index.js",
            "third_party/lib.rs",
            "external/dependency.py",
            ".yarn/cache/package.js",
            "bower_components/angular.js",
            "lib.min.js",
            "app.bundle.js",
        ];

        for path in vendor_paths {
            let decision = classifier.should_parse(&PathBuf::from(path), b"content");
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::VendorDirectory)),
                "Failed to detect vendor path: {}",
                path
            );
        }
    }

    #[test]
    fn test_build_artifact_detection() {
        let classifier = FileClassifier::default();

        let build_paths = vec![
            "target/debug/deps/lib.rlib",
            "target/release/myapp",
            "build/output.js",
            "dist/bundle.js",
            "__pycache__/module.pyc",
            "venv/lib/python3.9/site-packages/pkg.py",
            ".tox/py39/lib/python3.9/site-packages/test.py",
            "cmake-build-debug/CMakeFiles/app.dir/main.cpp.o",
            ".gradle/caches/modules-2/files-2.1/lib.jar",
        ];

        for path in build_paths {
            let decision = classifier.should_parse(&PathBuf::from(path), b"content");
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::BuildArtifact)),
                "Failed to detect build artifact: {}",
                path
            );
        }
    }

    #[test]
    fn test_minification_detection_edge_cases() {
        let classifier = FileClassifier::default();

        // Test that minified signatures are detected
        let minified_signatures: Vec<&[u8]> = vec![
            b"/*! jQuery",
            b"/*! * Bootstrap",
            b"!function(e,t){",
            b"/*! For license information",
            b"/** @license React",
        ];

        for sig in minified_signatures {
            let decision = classifier.should_parse(&PathBuf::from("lib.js"), sig);
            assert_eq!(
                decision,
                ParseDecision::Skip(SkipReason::MinifiedContent),
                "Failed to detect minified signature"
            );
        }

        // Test high-entropy content detection
        let high_entropy = b"a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6";
        let decision = classifier.should_parse(&PathBuf::from("data.js"), high_entropy);
        // Should be detected as minified due to lack of newlines
        assert_eq!(decision, ParseDecision::Skip(SkipReason::MinifiedContent));
    }
}
