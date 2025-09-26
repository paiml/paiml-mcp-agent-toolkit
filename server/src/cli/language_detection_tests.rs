//! Property-based tests for language detection
//!
//! CRITICAL: This is the most important feature of the entire tool and "it must work".
//! These tests ensure ALL languages are detected correctly with extreme TDD.

use super::{detect_primary_language_with_confidence, count_files_by_extension};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use proptest::prelude::*;

/// Test fixture for creating temporary project structures
struct ProjectFixture {
    #[allow(dead_code)]
    temp_dir: TempDir,
    path: PathBuf,
}

impl ProjectFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();
        Self { temp_dir, path }
    }

    fn create_file(&self, relative_path: &str, content: &str) {
        let file_path = self.path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }
        fs::write(&file_path, content).expect("Failed to write file");
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: TypeScript files must be detected as "typescript", never "deno" or "rust"
    #[test]
    fn test_typescript_detection_consistency() {
        let fixture = ProjectFixture::new();

        // Create TypeScript files
        fixture.create_file("src/main.ts", "console.log('Hello, TypeScript!');");
        fixture.create_file("src/component.tsx", "export const Component = () => <div>Hello</div>;");
        fixture.create_file("package.json", r#"{"name": "test", "type": "module"}"#);

        let result = detect_primary_language_with_confidence(fixture.path());

        // CRITICAL: Must detect as typescript, not deno/rust
        assert!(result.is_some(), "TypeScript project must be detected");
        let (lang, confidence) = result.unwrap();
        assert_eq!(lang, "typescript", "TypeScript files must be detected as 'typescript', got '{}'", lang);
        assert!(confidence > 0.0, "Confidence must be positive");
    }

    /// Property: JavaScript files must be detected as "javascript", never "deno" or "rust"
    #[test]
    fn test_javascript_detection_consistency() {
        let fixture = ProjectFixture::new();

        // Create JavaScript files
        fixture.create_file("src/main.js", "console.log('Hello, JavaScript!');");
        fixture.create_file("src/component.jsx", "export const Component = () => <div>Hello</div>;");
        fixture.create_file("package.json", r#"{"name": "test", "type": "module"}"#);

        let result = detect_primary_language_with_confidence(fixture.path());

        // CRITICAL: Must detect as javascript, not deno/rust
        assert!(result.is_some(), "JavaScript project must be detected");
        let (lang, confidence) = result.unwrap();
        assert_eq!(lang, "javascript", "JavaScript files must be detected as 'javascript', got '{}'", lang);
        assert!(confidence > 0.0, "Confidence must be positive");
    }

    /// Property: Rust files must be detected as "rust"
    #[test]
    fn test_rust_detection_consistency() {
        let fixture = ProjectFixture::new();

        // Create Rust files
        fixture.create_file("src/main.rs", "fn main() { println!(\"Hello, Rust!\"); }");
        fixture.create_file("Cargo.toml", r#"[package]
name = "test"
version = "0.1.0""#);

        let result = detect_primary_language_with_confidence(fixture.path());

        assert!(result.is_some(), "Rust project must be detected");
        let (lang, confidence) = result.unwrap();
        assert_eq!(lang, "rust", "Rust files must be detected as 'rust', got '{}'", lang);
        assert!(confidence > 0.0, "Confidence must be positive");
    }

    /// Property: Deno projects with deno.json must be detected as "deno"
    #[test]
    fn test_deno_detection_with_deno_config() {
        let fixture = ProjectFixture::new();

        // Create Deno project with explicit deno.json
        fixture.create_file("src/main.ts", "console.log('Hello, Deno!');");
        fixture.create_file("package.json", r#"{"name": "test", "type": "module"}"#);
        fixture.create_file("deno.json", r#"{"imports": {"std/": "https://deno.land/std/"}}"#);

        let result = detect_primary_language_with_confidence(fixture.path());

        assert!(result.is_some(), "Deno project must be detected");
        let (lang, confidence) = result.unwrap();
        assert_eq!(lang, "deno", "Deno projects with deno.json must be detected as 'deno', got '{}'", lang);
        assert_eq!(confidence, 100.0, "Deno with deno.json should have 100% confidence");
    }

    /// Property: Language detection must be deterministic for identical file sets
    #[test]
    fn test_detection_determinism() {
        let fixture1 = ProjectFixture::new();
        let fixture2 = ProjectFixture::new();

        // Create identical project structures
        let files = vec![
            ("src/main.ts", "console.log('test');"),
            ("src/utils.ts", "export const helper = () => {};"),
            ("package.json", r#"{"name": "test"}"#),
        ];

        for (path, content) in &files {
            fixture1.create_file(path, content);
            fixture2.create_file(path, content);
        }

        let result1 = detect_primary_language_with_confidence(fixture1.path());
        let result2 = detect_primary_language_with_confidence(fixture2.path());

        assert_eq!(result1, result2, "Language detection must be deterministic for identical projects");
    }

    /// Property: File extension counting must be accurate
    #[test]
    fn test_file_extension_counting_accuracy() {
        let fixture = ProjectFixture::new();

        // Create files with known extensions
        fixture.create_file("file1.ts", "// TypeScript 1");
        fixture.create_file("file2.ts", "// TypeScript 2");
        fixture.create_file("file3.tsx", "// TypeScript JSX");
        fixture.create_file("file1.js", "// JavaScript 1");
        fixture.create_file("ignored.txt", "// Should be ignored");

        let result = count_files_by_extension(fixture.path());
        assert!(result.is_some(), "Should detect files");

        let (detected_lang, confidence) = result.unwrap();

        // Should detect typescript due to more .ts/.tsx files (3) vs .js files (1)
        assert_eq!(detected_lang, "typescript", "Should detect typescript as dominant language");
        assert!(confidence > 0.0, "Confidence should be positive");
    }

    /// Property: Empty projects should not crash
    #[test]
    fn test_empty_project_handling() {
        let fixture = ProjectFixture::new();

        let result = detect_primary_language_with_confidence(fixture.path());

        // Empty projects should return None, not crash
        assert!(result.is_none(), "Empty projects should return None");
    }

    /// Property: Projects with only configuration files should handle gracefully
    #[test]
    fn test_config_only_project_handling() {
        let fixture = ProjectFixture::new();

        fixture.create_file("package.json", r#"{"name": "test"}"#);
        fixture.create_file("README.md", "# Test Project");

        let result = detect_primary_language_with_confidence(fixture.path());

        // Should handle gracefully - either None or a reasonable default
        // The important thing is it shouldn't crash
        if let Some((lang, confidence)) = result {
            assert!(confidence > 0.0, "If detected, confidence should be positive");
            assert!(!lang.is_empty(), "If detected, language should not be empty");
        }
        // None is also acceptable for config-only projects
    }
}

#[cfg(test)]
mod integration_tests {

    /// Integration test: Verify cross-command consistency
    /// This test ensures that different analysis commands produce consistent results
    #[test]
    fn test_cross_command_language_consistency() {
        // This is a placeholder for the integration test that would verify
        // `pmat context` and `pmat analyze deep-context` detect the same language
        // for the same project.

        // TODO: Implement after build completes and we can test binary commands
        // This test should:
        // 1. Create a test project
        // 2. Run both `context` and `deep-context` commands
        // 3. Verify they report the same language
        // 4. Verify they report the same or similar function counts
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Regression test for the critical bug where .ts files were detected as "deno"
    #[test]
    fn test_typescript_not_detected_as_deno_regression() {
        let fixture = ProjectFixture::new();

        // Create a pure TypeScript project without deno.json
        fixture.create_file("src/main.ts", "const greeting: string = 'Hello, TypeScript!';");
        fixture.create_file("src/types.ts", "export interface User { id: number; name: string; }");
        fixture.create_file("package.json", r#"{"name": "typescript-project", "dependencies": {"typescript": "^5.0.0"}}"#);

        let result = detect_primary_language_with_confidence(fixture.path());

        assert!(result.is_some(), "TypeScript project must be detected");
        let (lang, _) = result.unwrap();

        // CRITICAL REGRESSION TEST: Must NOT be "deno"
        assert_ne!(lang, "deno", "REGRESSION: TypeScript project must not be detected as 'deno'");
        assert_ne!(lang, "rust", "REGRESSION: TypeScript project must not be detected as 'rust'");
        assert_eq!(lang, "typescript", "TypeScript project must be detected as 'typescript'");
    }

    /// Regression test for inconsistent function counts between commands
    #[test]
    fn test_function_count_consistency_regression() {
        // This test will be implemented after we can run the binary commands
        // It should verify that `context` and `deep-context` return the same function counts

        // TODO: Implement after build completes
    }
}

/// Property-based test generator for language detection
mod proptest_generators {
    use super::*;

    /// Generate valid file extensions and their expected languages
    fn file_extension_strategy() -> impl Strategy<Value = (&'static str, &'static str)> {
        prop_oneof![
            Just(("ts", "typescript")),
            Just(("tsx", "typescript")),
            Just(("js", "javascript")),
            Just(("jsx", "javascript")),
            Just(("rs", "rust")),
            Just(("py", "python-uv")),
            Just(("kt", "kotlin")),
            Just(("kts", "kotlin")),
        ]
    }

    proptest! {
        /// Property: All supported file extensions must map to correct languages
        #[test]
        fn test_extension_mapping_correctness((ext, expected_lang) in file_extension_strategy()) {
            let fixture = ProjectFixture::new();

            // Create a file with the given extension
            let filename = format!("test.{}", ext);
            fixture.create_file(&filename, "// Test content");

            let result = count_files_by_extension(fixture.path());

            if let Some((detected_lang, _)) = result {
                prop_assert_eq!(detected_lang.clone(), expected_lang,
                    "Extension '{}' must map to '{}', got '{}'", ext, expected_lang, detected_lang);
            } else {
                prop_assert!(false, "Must detect language for extension '{}'", ext);
            }
        }
    }
}