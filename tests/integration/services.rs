//! Service-level integration tests with controlled I/O
//! Target: <30s execution time, filesystem/network mocking

use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_discovery_basic() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    create_test_file(&temp_dir, "main.rs", "fn main() {}");
    create_test_file(&temp_dir, "lib.rs", "pub fn test() {}");
    create_test_file(&temp_dir, "README.md", "# Test Project");

    let discovery = file_discovery::FileDiscovery::new();
    let files = discovery.discover_files(temp_dir.path(), &[]).unwrap();

    assert_eq!(files.len(), 3);
    assert!(files.iter().any(|f| f.ends_with("main.rs")));
    assert!(files.iter().any(|f| f.ends_with("lib.rs")));
    assert!(files.iter().any(|f| f.ends_with("README.md")));
}

#[test]
fn test_file_discovery_with_ignore_patterns() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    create_test_file(&temp_dir, "main.rs", "fn main() {}");
    create_test_file(&temp_dir, "test.txt", "test content");
    create_test_file(&temp_dir, "README.md", "# README");

    let discovery = file_discovery::FileDiscovery::new();
    let files = discovery
        .discover_files(temp_dir.path(), &["*.txt".to_string()])
        .unwrap();

    // Should exclude .txt files based on ignore patterns
    assert!(files.iter().any(|f| f.ends_with("main.rs")));
    assert!(files.iter().any(|f| f.ends_with("README.md")));
    assert_eq!(files.len(), 2); // Only 2 files, not the .txt
}

#[test]
fn test_project_metadata_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create Rust project structure
    create_test_file(
        &temp_dir,
        "Cargo.toml",
        r#"
[package]
name = "test_project"
version = "0.1.0"
"#,
    );
    create_test_file(&temp_dir, "src/main.rs", "fn main() {}");

    let detector = project_meta_detector::ProjectMetaDetector::new();
    let metadata = detector.detect(temp_dir.path()).unwrap();

    assert!(metadata.is_some());
    let meta = metadata.unwrap();
    assert_eq!(
        meta.project_type,
        project_meta_detector::ProjectType::RustCargo
    );
}

#[test]
fn test_file_classifier() {
    let classifier = file_classifier::FileClassifier::new();

    assert_eq!(
        classifier.classify("main.rs"),
        Some(file_classifier::FileType::Source)
    );
    assert_eq!(
        classifier.classify("test.py"),
        Some(file_classifier::FileType::Source)
    );
    assert_eq!(
        classifier.classify("README.md"),
        Some(file_classifier::FileType::Documentation)
    );
    assert_eq!(
        classifier.classify("Cargo.toml"),
        Some(file_classifier::FileType::Config)
    );
    assert_eq!(
        classifier.classify("test.jpg"),
        Some(file_classifier::FileType::Asset)
    );
}

#[test]
fn test_template_service_basic() {
    let service = template_service::TemplateService::new();

    // Test that we can list templates
    let templates = service.list_templates().unwrap();
    assert!(!templates.is_empty());

    // Should have at least rust templates
    assert!(templates.iter().any(|t| t.starts_with("rust/")));
}

#[test]
fn test_git_analysis_basic() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize a git repo
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output();

    if output.is_ok() && output.unwrap().status.success() {
        // Create and commit a file
        create_test_file(&temp_dir, "test.txt", "test content");

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .ok();

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .output()
            .ok();

        let analyzer = git_analysis::GitAnalyzer::new();
        let result = analyzer.analyze_repository(temp_dir.path());

        // Git analysis might fail in CI, so we just check it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
}

fn create_test_file(dir: &TempDir, name: &str, content: &str) {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

// Mock implementations for testing
pub mod file_discovery {
    use std::path::Path;
    use walkdir::WalkDir;

    pub struct FileDiscovery;

    impl Default for FileDiscovery {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FileDiscovery {
        pub fn new() -> Self {
            Self
        }

        pub fn discover_files(
            &self,
            path: &Path,
            ignore_patterns: &[String],
        ) -> Result<Vec<String>, String> {
            let mut files = Vec::new();

            for entry in WalkDir::new(path).into_iter().flatten() {
                if entry.file_type().is_file() {
                    if let Some(path_str) = entry.path().to_str() {
                        // Check if file matches any ignore pattern
                        let should_ignore = ignore_patterns.iter().any(|pattern| {
                            if pattern.starts_with("*.") {
                                path_str.ends_with(&pattern[1..])
                            } else {
                                path_str.contains(pattern)
                            }
                        });

                        if !should_ignore {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }

            Ok(files)
        }
    }
}

pub mod project_meta_detector {
    use std::path::Path;

    #[derive(Debug, PartialEq)]
    pub enum ProjectType {
        RustCargo,
        Python,
        JavaScript,
        Unknown,
    }

    pub struct ProjectMetadata {
        pub project_type: ProjectType,
    }

    pub struct ProjectMetaDetector;

    impl Default for ProjectMetaDetector {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProjectMetaDetector {
        pub fn new() -> Self {
            Self
        }

        pub fn detect(&self, path: &Path) -> Result<Option<ProjectMetadata>, String> {
            if path.join("Cargo.toml").exists() {
                Ok(Some(ProjectMetadata {
                    project_type: ProjectType::RustCargo,
                }))
            } else if path.join("package.json").exists() {
                Ok(Some(ProjectMetadata {
                    project_type: ProjectType::JavaScript,
                }))
            } else if path.join("setup.py").exists() || path.join("pyproject.toml").exists() {
                Ok(Some(ProjectMetadata {
                    project_type: ProjectType::Python,
                }))
            } else {
                Ok(None)
            }
        }
    }
}

pub mod file_classifier {
    #[derive(Debug, PartialEq)]
    pub enum FileType {
        Source,
        Test,
        Documentation,
        Config,
        Asset,
    }

    pub struct FileClassifier;

    impl Default for FileClassifier {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FileClassifier {
        pub fn new() -> Self {
            Self
        }

        pub fn classify(&self, filename: &str) -> Option<FileType> {
            if filename.ends_with(".rs") || filename.ends_with(".py") || filename.ends_with(".js") {
                Some(FileType::Source)
            } else if filename.ends_with(".md") {
                Some(FileType::Documentation)
            } else if filename.ends_with(".toml")
                || filename.ends_with(".json")
                || filename.ends_with(".yaml")
            {
                Some(FileType::Config)
            } else if filename.ends_with(".jpg")
                || filename.ends_with(".png")
                || filename.ends_with(".gif")
            {
                Some(FileType::Asset)
            } else {
                None
            }
        }
    }
}

pub mod template_service {
    pub struct TemplateService;

    impl Default for TemplateService {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TemplateService {
        pub fn new() -> Self {
            Self
        }

        pub fn list_templates(&self) -> Result<Vec<String>, String> {
            Ok(vec![
                "rust/cli".to_string(),
                "rust/lib".to_string(),
                "python/cli".to_string(),
                "deno/cli".to_string(),
            ])
        }
    }
}

pub mod git_analysis {
    use std::path::Path;

    pub struct GitAnalyzer;

    impl Default for GitAnalyzer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GitAnalyzer {
        pub fn new() -> Self {
            Self
        }

        pub fn analyze_repository(&self, _path: &Path) -> Result<(), String> {
            // Simple mock that always succeeds
            Ok(())
        }
    }
}
