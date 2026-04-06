#![cfg_attr(coverage_nightly, coverage(off))]
// BonusDetector - Detects advanced quality practices for bonus points
//
// Bonus Points (up to +10 max):
// - Property-based testing (proptest) → +3 points
// - Fuzzing (cargo-fuzz) → +2 points
// - Mutation testing (cargo-mutants) → +2 points
// - Living documentation (mdBook) → +3 points

use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use std::path::Path;
use walkdir::WalkDir;

pub struct BonusDetector;

impl BonusDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect property-based testing with proptest (+3 points)
    async fn detect_property_tests(&self, repo_path: &Path) -> Result<BonusItem> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let mut evidence = vec![];
        let mut detected = false;

        // Check Cargo.toml for proptest dependency
        let cargo_toml = repo_path.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = tokio::fs::read_to_string(&cargo_toml).await?;
            if content.contains("proptest") {
                evidence.push("Cargo.toml contains proptest dependency".to_string());
                detected = true;
            }
        }

        // Check for proptest usage in source files
        for entry in WalkDir::new(repo_path).max_depth(10).into_iter().flatten() {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        if let Ok(content) = tokio::fs::read_to_string(path).await {
                            if content.contains("proptest!") || content.contains("use proptest::") {
                                evidence
                                    .push(format!("Property tests found in {}", path.display()));
                                detected = true;
                                break; // Found evidence, no need to scan more
                            }
                        }
                    }
                }
            }
        }

        Ok(BonusItem {
            points: if detected { 3.0 } else { 0.0 },
            max_points: 3.0,
            detected,
            evidence,
        })
    }

    /// Detect fuzzing with cargo-fuzz (+2 points)
    async fn detect_fuzzing(&self, repo_path: &Path) -> Result<BonusItem> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let mut evidence = vec![];
        let mut detected = false;

        // Check for fuzz directory
        let fuzz_dir = repo_path.join("fuzz");
        if fuzz_dir.exists() && fuzz_dir.is_dir() {
            evidence.push("fuzz/ directory found".to_string());
            detected = true;

            // Check for fuzz targets
            let fuzz_targets = fuzz_dir.join("fuzz_targets");
            if fuzz_targets.exists() {
                evidence.push("fuzz/fuzz_targets/ directory found".to_string());
            }
        }

        // Check Cargo.toml for fuzzing dependencies
        let cargo_toml = repo_path.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = tokio::fs::read_to_string(&cargo_toml).await?;
            if content.contains("cargo-fuzz") || content.contains("libfuzzer-sys") {
                evidence.push("Fuzzing dependencies in Cargo.toml".to_string());
                detected = true;
            }
        }

        Ok(BonusItem {
            points: if detected { 2.0 } else { 0.0 },
            max_points: 2.0,
            detected,
            evidence,
        })
    }

    /// Detect mutation testing with cargo-mutants (+2 points)
    async fn detect_mutation_testing(&self, repo_path: &Path) -> Result<BonusItem> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let mut evidence = vec![];
        let mut detected = false;

        // Check for .cargo/mutants.toml or mutants.toml
        let mutants_toml_paths = vec![
            repo_path.join(".cargo/mutants.toml"),
            repo_path.join("mutants.toml"),
        ];

        for path in mutants_toml_paths {
            if path.exists() {
                evidence.push(format!(
                    "{} found",
                    path.file_name().expect("internal error").to_string_lossy()
                ));
                detected = true;
            }
        }

        // Check CI workflows for cargo-mutants
        let workflows_dir = repo_path.join(".github/workflows");
        if workflows_dir.exists() {
            for entry in WalkDir::new(&workflows_dir)
                .max_depth(1)
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "yml" || ext == "yaml" {
                            if let Ok(content) = tokio::fs::read_to_string(entry.path()).await {
                                if content.contains("cargo-mutants")
                                    || content.contains("cargo mutants")
                                {
                                    evidence.push("Mutation testing in CI workflow".to_string());
                                    detected = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check Makefile for mutation testing targets
        let makefile = repo_path.join("Makefile");
        if makefile.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&makefile).await {
                if content.contains("mutants") || content.contains("mutation") {
                    evidence.push("Mutation testing target in Makefile".to_string());
                    detected = true;
                }
            }
        }

        Ok(BonusItem {
            points: if detected { 2.0 } else { 0.0 },
            max_points: 2.0,
            detected,
            evidence,
        })
    }

    /// Detect living documentation with mdBook (+3 points)
    async fn detect_living_docs(&self, repo_path: &Path) -> Result<BonusItem> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let mut evidence = vec![];
        let mut detected = false;

        // Check for book.toml (mdBook configuration)
        let book_toml = repo_path.join("book.toml");
        if book_toml.exists() {
            evidence.push("book.toml found".to_string());
            detected = true;
        }

        // Check for src/SUMMARY.md (mdBook structure)
        let summary_md = repo_path.join("src/SUMMARY.md");
        if summary_md.exists() {
            evidence.push("src/SUMMARY.md found".to_string());
            detected = true;
        }

        // Check for common mdBook directories
        let common_book_dirs = vec![repo_path.join("book"), repo_path.join("docs")];

        for book_dir in common_book_dirs {
            if book_dir.exists() && book_dir.is_dir() {
                // Check if it contains mdBook structure
                let summary_path = book_dir.join("src/SUMMARY.md");
                if summary_path.exists() {
                    evidence.push(format!(
                        "mdBook structure in {}/",
                        book_dir
                            .file_name()
                            .expect("internal error")
                            .to_string_lossy()
                    ));
                    detected = true;
                }
            }
        }

        // Check Cargo.toml for mdbook dependency
        let cargo_toml = repo_path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&cargo_toml).await {
                if content.contains("mdbook") {
                    evidence.push("mdbook in dependencies".to_string());
                    detected = true;
                }
            }
        }

        Ok(BonusItem {
            points: if detected { 3.0 } else { 0.0 },
            max_points: 3.0,
            detected,
            evidence,
        })
    }

    /// Detect all bonus features
    pub async fn detect(&self, repo_path: &Path) -> Result<BonusScores> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let property_tests = self.detect_property_tests(repo_path).await?;
        let fuzzing = self.detect_fuzzing(repo_path).await?;
        let mutation_testing = self.detect_mutation_testing(repo_path).await?;
        let living_docs = self.detect_living_docs(repo_path).await?;

        Ok(BonusScores {
            property_tests,
            fuzzing,
            mutation_testing,
            living_docs,
        })
    }
}

impl Default for BonusDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().expect("internal error")
    }

    fn create_file(repo_path: &Path, relative_path: &str, content: &str) {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let file_path = repo_path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("internal error");
        }
        fs::write(file_path, content).expect("internal error");
    }

    #[tokio::test]
    async fn test_bonus_detector_no_features() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert_eq!(result.total(), 0.0);
        assert!(!result.property_tests.detected);
        assert!(!result.fuzzing.detected);
        assert!(!result.mutation_testing.detected);
        assert!(!result.living_docs.detected);
    }

    #[tokio::test]
    async fn test_bonus_detector_property_tests() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let cargo_toml = r#"
[dependencies]
proptest = "1.0"
"#;
        create_file(repo_path, "Cargo.toml", cargo_toml);

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert_eq!(result.property_tests.points, 3.0);
        assert!(result.property_tests.detected);
    }

    #[tokio::test]
    async fn test_bonus_detector_fuzzing() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create fuzz directory
        fs::create_dir_all(repo_path.join("fuzz/fuzz_targets")).expect("internal error");

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert_eq!(result.fuzzing.points, 2.0);
        assert!(result.fuzzing.detected);
    }

    #[tokio::test]
    async fn test_bonus_detector_mutation_testing() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "mutants.toml", "[mutants]");

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert_eq!(result.mutation_testing.points, 2.0);
        assert!(result.mutation_testing.detected);
    }

    #[tokio::test]
    async fn test_bonus_detector_living_docs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "book.toml", "[book]");
        create_file(repo_path, "src/SUMMARY.md", "# Summary");

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert_eq!(result.living_docs.points, 3.0);
        assert!(result.living_docs.detected);
    }

    #[tokio::test]
    async fn test_bonus_detector_all_features() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Add all bonus features
        create_file(
            repo_path,
            "Cargo.toml",
            "[dependencies]\nproptest = \"1.0\"",
        );
        fs::create_dir_all(repo_path.join("fuzz")).expect("internal error");
        create_file(repo_path, "mutants.toml", "[mutants]");
        create_file(repo_path, "book.toml", "[book]");

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert_eq!(result.total(), 10.0);
        assert!(result.property_tests.detected);
        assert!(result.fuzzing.detected);
        assert!(result.mutation_testing.detected);
        assert!(result.living_docs.detected);
    }

    #[tokio::test]
    async fn test_bonus_detector_partial_features() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Add only property tests and fuzzing
        create_file(
            repo_path,
            "Cargo.toml",
            "[dependencies]\nproptest = \"1.0\"",
        );
        fs::create_dir_all(repo_path.join("fuzz")).expect("internal error");

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert_eq!(result.total(), 5.0); // 3 + 2
        assert!(result.property_tests.detected);
        assert!(result.fuzzing.detected);
        assert!(!result.mutation_testing.detected);
        assert!(!result.living_docs.detected);
    }

    #[tokio::test]
    async fn test_bonus_detector_evidence_tracking() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "book.toml", "[book]");

        let detector = BonusDetector::new();
        let result = detector.detect(repo_path).await.expect("internal error");

        assert!(!result.living_docs.evidence.is_empty());
        assert!(result
            .living_docs
            .evidence
            .iter()
            .any(|e| e.contains("book.toml")));
    }

    #[tokio::test]
    async fn test_bonus_max_points() {
        let detector = BonusDetector::new();
        let temp_dir = create_temp_repo();
        let result = detector
            .detect(temp_dir.path())
            .await
            .expect("internal error");

        // Verify max points sum to 10
        assert_eq!(
            result.property_tests.max_points
                + result.fuzzing.max_points
                + result.mutation_testing.max_points
                + result.living_docs.max_points,
            10.0
        );
    }
}
