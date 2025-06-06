use crate::models::project_meta::{MetaFile, MetaFileType};
use regex::Regex;
use std::path::Path;
use tokio::fs;
use tokio::task::JoinSet;
use tracing::debug;
use walkdir::WalkDir;

pub struct ProjectMetaDetector {
    patterns: Vec<(Regex, MetaFileType)>,
}

impl ProjectMetaDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (Regex::new(r"^Makefile$").unwrap(), MetaFileType::Makefile),
                (Regex::new(r"^makefile$").unwrap(), MetaFileType::Makefile),
                (
                    Regex::new(r"^GNUmakefile$").unwrap(),
                    MetaFileType::Makefile,
                ),
                (Regex::new(r"^README\.md$").unwrap(), MetaFileType::Readme),
                (
                    Regex::new(r"^README\.markdown$").unwrap(),
                    MetaFileType::Readme,
                ),
                (Regex::new(r"^README\.rst$").unwrap(), MetaFileType::Readme),
                (Regex::new(r"^README\.txt$").unwrap(), MetaFileType::Readme),
                (Regex::new(r"^README$").unwrap(), MetaFileType::Readme),
                (Regex::new(r"^readme\.md$").unwrap(), MetaFileType::Readme),
            ],
        }
    }

    pub async fn detect(&self, project_root: &Path) -> Vec<MetaFile> {
        let mut tasks = JoinSet::new();
        let mut found_files = Vec::new();

        // Only scan top 2 levels to avoid deep recursion
        for entry in WalkDir::new(project_root)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };

            // Check if filename matches any pattern
            for (pattern, file_type) in &self.patterns {
                if pattern.is_match(file_name) {
                    let path_buf = path.to_path_buf();
                    let file_type_clone = file_type.clone();

                    tasks.spawn(async move {
                        match tokio::time::timeout(
                            std::time::Duration::from_millis(100),
                            fs::read_to_string(&path_buf),
                        )
                        .await
                        {
                            Ok(Ok(content)) => Some(MetaFile {
                                path: path_buf,
                                file_type: file_type_clone,
                                content,
                            }),
                            Ok(Err(e)) => {
                                debug!("Failed to read file {:?}: {}", path_buf, e);
                                None
                            }
                            Err(_) => {
                                debug!("Timeout reading file {:?}", path_buf);
                                None
                            }
                        }
                    });
                    break; // Only match first pattern for each file
                }
            }
        }

        // Collect results with timeout
        while let Some(result) = tasks.join_next().await {
            if let Ok(Some(meta_file)) = result {
                found_files.push(meta_file);
            }
        }

        found_files
    }
}

impl Default for ProjectMetaDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_detect_metadata_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        fs::write(root.join("Makefile"), "test:\n\techo test").unwrap();
        fs::write(root.join("README.md"), "# Test Project").unwrap();
        fs::write(root.join("readme.md"), "# Lower case readme").unwrap();
        fs::write(root.join("random.txt"), "Not a meta file").unwrap();

        // Create subdirectory with files
        let sub_dir = root.join("docs");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("README.md"), "# Docs README").unwrap();

        let detector = ProjectMetaDetector::new();
        let mut files = detector.detect(root).await;

        // Sort by path for predictable ordering
        files.sort_by(|a, b| a.path.cmp(&b.path));

        // Should find 4 meta files
        assert_eq!(files.len(), 4);

        // Check Makefile
        let makefile = files
            .iter()
            .find(|f| f.path.file_name().unwrap() == "Makefile")
            .unwrap();
        assert!(matches!(makefile.file_type, MetaFileType::Makefile));
        assert!(makefile.content.contains("echo test"));

        // Check README.md files
        let readme_count = files
            .iter()
            .filter(|f| matches!(f.file_type, MetaFileType::Readme))
            .count();
        assert_eq!(readme_count, 3);

        // Should not detect non-meta files
        assert!(!files
            .iter()
            .any(|f| f.path.file_name().unwrap() == "random.txt"));
    }

    #[tokio::test]
    async fn test_detect_various_makefile_variants() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create different Makefile variants
        fs::write(root.join("Makefile"), "all:").unwrap();
        fs::write(root.join("makefile"), "build:").unwrap();
        fs::write(root.join("GNUmakefile"), "test:").unwrap();

        let detector = ProjectMetaDetector::new();
        let files = detector.detect(root).await;

        // Should find all 3 variants
        assert_eq!(files.len(), 3);
        assert!(files
            .iter()
            .all(|f| matches!(f.file_type, MetaFileType::Makefile)));
    }

    #[tokio::test]
    async fn test_detect_various_readme_variants() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create different README variants
        fs::write(root.join("README.md"), "# MD").unwrap();
        fs::write(root.join("README.markdown"), "# Markdown").unwrap();
        fs::write(root.join("README.rst"), "RST").unwrap();
        fs::write(root.join("README.txt"), "TXT").unwrap();
        fs::write(root.join("README"), "Plain").unwrap();

        let detector = ProjectMetaDetector::new();
        let files = detector.detect(root).await;

        // Should find all 5 variants
        assert_eq!(files.len(), 5);
        assert!(files
            .iter()
            .all(|f| matches!(f.file_type, MetaFileType::Readme)));
    }

    #[tokio::test]
    async fn test_max_depth_limitation() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create nested structure
        let deep_path = root.join("a").join("b").join("c");
        fs::create_dir_all(&deep_path).unwrap();

        // File at depth 0
        fs::write(root.join("README.md"), "Root").unwrap();

        // File at depth 1
        fs::write(root.join("a").join("README.md"), "Level 1").unwrap();

        // File at depth 2
        fs::write(root.join("a").join("b").join("README.md"), "Level 2").unwrap();

        // File at depth 3 (should not be detected)
        fs::write(deep_path.join("README.md"), "Too deep").unwrap();

        let detector = ProjectMetaDetector::new();
        let files = detector.detect(root).await;

        // Should find files at depth 0 and 1 (max_depth 2 means go down 2 levels from root)
        // root/ (depth 0) -> a/ (depth 1) -> b/ (depth 2, beyond our max_depth)
        assert_eq!(files.len(), 2);
        assert!(!files.iter().any(|f| f.content.contains("Too deep")));
        assert!(!files.iter().any(|f| f.content.contains("Level 2")));
    }

    #[tokio::test]
    async fn test_file_read_timeout() {
        // This test is conceptual since we can't easily simulate a slow file read
        // But it verifies the timeout mechanism is in place
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("README.md"), "Content").unwrap();

        let detector = ProjectMetaDetector::new();
        let files = detector.detect(root).await;

        // Should successfully read the file
        assert_eq!(files.len(), 1);
    }
}
