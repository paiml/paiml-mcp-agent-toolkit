#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Type definitions: structs, enums, and internal types
include!("polyglot_analyzer_types.rs");

// Language detection, scanning, framework detection, and stats
include!("polyglot_analyzer_detection.rs");

// Cross-language dependency analysis
include!("polyglot_analyzer_dependencies.rs");

// Architecture detection, confidence scoring, and insights
include!("polyglot_analyzer_architecture.rs");

/// Toyota Way: Extract Method - Check if directory should be skipped (complexity <= 3)
fn should_skip_directory(path: &Path) -> bool {
    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
        matches!(
            dir_name,
            "node_modules" | "target" | "build" | ".git" | "__pycache__" | ".venv" | "venv"
        )
    } else {
        false
    }
}

impl Default for PolyglotAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// Tests extracted to polyglot_analyzer_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "polyglot_analyzer_tests.rs"]
mod tests;

#[cfg(test)]
mod dependencies_tests {
    //! PMAT-654: cover polyglot_analyzer_dependencies.rs pure sync helpers.
    use super::*;
    use tempfile::TempDir;

    fn write(p: &std::path::Path, content: &str) {
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, content).unwrap();
    }

    // --- is_skipped_dir ---

    #[test]
    fn test_is_skipped_dir_all_allowlisted_names() {
        for name in [
            "node_modules",
            "target",
            "build",
            ".git",
            "__pycache__",
            ".venv",
            "venv",
        ] {
            let p = std::path::PathBuf::from("/root").join(name);
            assert!(is_skipped_dir(&p), "expected {name} to be skipped");
        }
    }

    #[test]
    fn test_is_skipped_dir_non_match_returns_false() {
        for name in ["src", "tests", "docs", "my_module", "node_modules_bak"] {
            let p = std::path::PathBuf::from("/root").join(name);
            assert!(!is_skipped_dir(&p), "expected {name} to NOT be skipped");
        }
    }

    #[test]
    fn test_is_skipped_dir_root_path_not_skipped() {
        assert!(!is_skipped_dir(std::path::Path::new("/")));
    }

    // --- build_config_dependency_pairs ---

    #[test]
    fn test_build_config_dependency_pairs_empty_is_empty() {
        let deps = build_config_dependency_pairs(&[], "pkg.json");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_build_config_dependency_pairs_single_lang_is_empty() {
        let rust = "rust".to_string();
        let deps = build_config_dependency_pairs(&[&rust], "Cargo.toml");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_build_config_dependency_pairs_two_langs_one_pair() {
        let rust = "rust".to_string();
        let python = "python".to_string();
        let deps = build_config_dependency_pairs(&[&rust, &python], "pyproject.toml");
        assert_eq!(deps.len(), 1);
        let dep = &deps[0];
        assert_eq!(dep.from_language, "rust");
        assert_eq!(dep.to_language, "python");
        assert!(matches!(
            dep.dependency_type,
            DependencyType::ConfigurationFile
        ));
        assert!((dep.coupling_strength - 0.4).abs() < 1e-9);
        assert_eq!(dep.files_involved, vec!["pyproject.toml".to_string()]);
    }

    #[test]
    fn test_build_config_dependency_pairs_three_langs_three_pairs() {
        let rust = "rust".to_string();
        let python = "python".to_string();
        let js = "javascript".to_string();
        let deps = build_config_dependency_pairs(&[&rust, &python, &js], "config.yaml");
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].from_language, "rust");
        assert_eq!(deps[0].to_language, "python");
        assert_eq!(deps[1].from_language, "rust");
        assert_eq!(deps[1].to_language, "javascript");
        assert_eq!(deps[2].from_language, "python");
        assert_eq!(deps[2].to_language, "javascript");
    }

    // --- count_files_recursive (impl method) ---

    fn analyzer() -> PolyglotAnalyzer {
        PolyglotAnalyzer::new()
    }

    #[test]
    fn test_count_files_recursive_missing_dir_is_no_op() {
        let a = analyzer();
        let mut count = 0usize;
        let _ = a.count_files_recursive(
            std::path::Path::new("/tmp/absolutely-does-not-exist-qwerty"),
            &["rs".to_string()],
            &mut count,
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_files_recursive_matching_extension_counted() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.rs"), "");
        write(&tmp.path().join("b.rs"), "");
        write(&tmp.path().join("c.py"), "");
        let a = analyzer();
        let mut count = 0usize;
        a.count_files_recursive(tmp.path(), &["rs".to_string()], &mut count)
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_files_recursive_nested_dirs_traversed() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.rs"), "");
        write(&tmp.path().join("sub/b.rs"), "");
        write(&tmp.path().join("sub/deep/c.rs"), "");
        let a = analyzer();
        let mut count = 0usize;
        a.count_files_recursive(tmp.path(), &["rs".to_string()], &mut count)
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_files_recursive_skips_excluded_dirs() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("src/main.rs"), "");
        write(&tmp.path().join("target/build/artifact.rs"), "");
        write(&tmp.path().join("node_modules/pkg/file.rs"), "");
        let a = analyzer();
        let mut count = 0usize;
        a.count_files_recursive(tmp.path(), &["rs".to_string()], &mut count)
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_files_recursive_multi_extension() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.ts"), "");
        write(&tmp.path().join("b.tsx"), "");
        write(&tmp.path().join("c.rs"), "");
        let a = analyzer();
        let mut count = 0usize;
        a.count_files_recursive(
            tmp.path(),
            &["ts".to_string(), "tsx".to_string()],
            &mut count,
        )
        .unwrap();
        assert_eq!(count, 2);
    }

    // --- has_potential_integration ---

    #[test]
    fn test_has_potential_integration_rust_python_both_directions() {
        let a = analyzer();
        assert!(a.has_potential_integration("rust", "python"));
        assert!(a.has_potential_integration("python", "rust"));
    }

    #[test]
    fn test_has_potential_integration_js_python() {
        let a = analyzer();
        assert!(a.has_potential_integration("javascript", "python"));
        assert!(a.has_potential_integration("python", "javascript"));
    }

    #[test]
    fn test_has_potential_integration_ts_rust() {
        let a = analyzer();
        assert!(a.has_potential_integration("typescript", "rust"));
        assert!(a.has_potential_integration("rust", "typescript"));
    }

    #[test]
    fn test_has_potential_integration_non_match_pairs_false() {
        let a = analyzer();
        assert!(!a.has_potential_integration("rust", "c"));
        assert!(!a.has_potential_integration("python", "c"));
        assert!(!a.has_potential_integration("rust", "rust"));
        assert!(!a.has_potential_integration("go", "python"));
    }

    // --- infer_dependency_type ---

    #[test]
    fn test_infer_dependency_type_rust_python_is_ffi() {
        let a = analyzer();
        assert!(matches!(
            a.infer_dependency_type("rust", "python"),
            DependencyType::FFI
        ));
        assert!(matches!(
            a.infer_dependency_type("python", "rust"),
            DependencyType::FFI
        ));
    }

    #[test]
    fn test_infer_dependency_type_ts_js_is_shared_data() {
        let a = analyzer();
        assert!(matches!(
            a.infer_dependency_type("typescript", "javascript"),
            DependencyType::SharedDataStructure
        ));
        assert!(matches!(
            a.infer_dependency_type("javascript", "typescript"),
            DependencyType::SharedDataStructure
        ));
    }

    #[test]
    fn test_infer_dependency_type_fallback_process_communication() {
        let a = analyzer();
        assert!(matches!(
            a.infer_dependency_type("rust", "go"),
            DependencyType::ProcessCommunication
        ));
        assert!(matches!(
            a.infer_dependency_type("c", "python"),
            DependencyType::ProcessCommunication
        ));
        assert!(matches!(
            a.infer_dependency_type("unknown1", "unknown2"),
            DependencyType::ProcessCommunication
        ));
    }
}
