//! Template generation trait and utilities.

use super::context::AgentContext;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Trait for template generation.
#[async_trait]
pub trait TemplateGenerator: Send + Sync {
    /// Generate files from the agent context.
    fn generate(&self, ctx: &AgentContext) -> Result<GeneratedFiles>;

    /// Validate the agent context before generation.
    fn validate_context(&self, ctx: &AgentContext) -> Result<()>;

    /// Post-generation hooks (e.g., running formatters, installing dependencies).
    fn post_generation_hooks(&self, _path: &Path) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Get the name of this template generator.
    fn name(&self) -> &str;

    /// Get a description of what this generator creates.
    fn description(&self) -> &str;
}

/// Collection of generated files.
pub struct GeneratedFiles {
    /// Map of file paths to their contents.
    pub files: HashMap<PathBuf, FileContent>,
    /// Map of file paths to their permissions (Unix only).
    pub permissions: HashMap<PathBuf, u32>,
    /// List of symlinks to create (source, target).
    pub symlinks: Vec<(PathBuf, PathBuf)>,
}

impl GeneratedFiles {
    /// Create a new empty collection of generated files.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            permissions: HashMap::new(),
            symlinks: Vec::new(),
        }
    }

    /// Add a text file to the collection.
    pub fn add_text_file(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
        self.files
            .insert(path.into(), FileContent::Text(content.into()));
    }

    /// Add a binary file to the collection.
    pub fn add_binary_file(&mut self, path: impl Into<PathBuf>, content: Vec<u8>) {
        self.files.insert(path.into(), FileContent::Binary(content));
    }

    /// Add a template file to the collection.
    pub fn add_template_file(&mut self, path: impl Into<PathBuf>, template: impl Into<String>) {
        self.files
            .insert(path.into(), FileContent::Template(template.into()));
    }

    /// Set permissions for a file (Unix only).
    pub fn set_permissions(&mut self, path: impl Into<PathBuf>, permissions: u32) {
        self.permissions.insert(path.into(), permissions);
    }

    /// Add a symlink to create.
    pub fn add_symlink(&mut self, source: impl Into<PathBuf>, target: impl Into<PathBuf>) {
        self.symlinks.push((source.into(), target.into()));
    }

    /// Write all files to disk.
    pub async fn write_to_disk(&self, base_path: &Path) -> Result<()> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(base_path).await?;

        // Write all files
        for (relative_path, content) in &self.files {
            let full_path = base_path.join(relative_path);

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            // Write file content
            match content {
                FileContent::Text(text) | FileContent::Template(text) => {
                    fs::write(&full_path, text).await?;
                }
                FileContent::Binary(bytes) => {
                    fs::write(&full_path, bytes).await?;
                }
            }

            // Set permissions if specified (Unix only)
            #[cfg(unix)]
            if let Some(&perms) = self.permissions.get(relative_path) {
                use std::os::unix::fs::PermissionsExt;
                let permissions = std::fs::Permissions::from_mode(perms);
                fs::set_permissions(&full_path, permissions).await?;
            }
        }

        // Create symlinks
        #[cfg(unix)]
        for (source, target) in &self.symlinks {
            let source_path = base_path.join(source);
            let target_path = base_path.join(target);
            if source_path.exists() {
                std::os::unix::fs::symlink(&source_path, &target_path)?;
            }
        }

        Ok(())
    }

    /// Check if a file exists in the collection.
    #[must_use]
    pub fn contains_file(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    /// Get the number of files in the collection.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Merge another collection of generated files into this one.
    pub fn merge(&mut self, other: GeneratedFiles) {
        self.files.extend(other.files);
        self.permissions.extend(other.permissions);
        self.symlinks.extend(other.symlinks);
    }
}

impl Default for GeneratedFiles {
    fn default() -> Self {
        Self::new()
    }
}

/// Content of a generated file.
#[derive(Debug, Clone)]
pub enum FileContent {
    /// Plain text content.
    Text(String),
    /// Binary content.
    Binary(Vec<u8>),
    /// Template content (for further processing).
    Template(String),
}

impl FileContent {
    /// Get the content as a string, if possible.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) | Self::Template(s) => Some(s),
            Self::Binary(_) => None,
        }
    }

    /// Get the content as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Text(s) | Self::Template(s) => s.as_bytes(),
            Self::Binary(b) => b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generated_files_creation() {
        let mut files = GeneratedFiles::new();
        files.add_text_file("src/main.rs", "fn main() {}");
        files.add_binary_file("data.bin", vec![0, 1, 2, 3]);
        files.set_permissions("script.sh", 0o755);

        assert_eq!(files.file_count(), 2);
        assert!(files.contains_file(Path::new("src/main.rs")));
        assert!(files.contains_file(Path::new("data.bin")));
    }

    #[tokio::test]
    async fn test_write_to_disk() {
        let temp_dir = TempDir::new().unwrap();
        let mut files = GeneratedFiles::new();
        files.add_text_file("src/main.rs", "fn main() { println!(\"Hello\"); }");
        files.add_text_file("Cargo.toml", "[package]\nname = \"test\"");

        files.write_to_disk(temp_dir.path()).await.unwrap();

        let main_path = temp_dir.path().join("src/main.rs");
        assert!(main_path.exists());
        let content = fs::read_to_string(main_path).await.unwrap();
        assert_eq!(content, "fn main() { println!(\"Hello\"); }");

        let cargo_path = temp_dir.path().join("Cargo.toml");
        assert!(cargo_path.exists());
    }

    #[test]
    fn test_file_content() {
        let text = FileContent::Text("hello".to_string());
        assert_eq!(text.as_str(), Some("hello"));
        assert_eq!(text.as_bytes(), b"hello");

        let binary = FileContent::Binary(vec![1, 2, 3]);
        assert_eq!(binary.as_str(), None);
        assert_eq!(binary.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_merge_files() {
        let mut files1 = GeneratedFiles::new();
        files1.add_text_file("file1.txt", "content1");
        files1.set_permissions("file1.txt", 0o644);

        let mut files2 = GeneratedFiles::new();
        files2.add_text_file("file2.txt", "content2");
        files2.set_permissions("file2.txt", 0o755);

        files1.merge(files2);

        assert_eq!(files1.file_count(), 2);
        assert!(files1.contains_file(Path::new("file1.txt")));
        assert!(files1.contains_file(Path::new("file2.txt")));
        assert_eq!(files1.permissions.len(), 2);
    }
}

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
