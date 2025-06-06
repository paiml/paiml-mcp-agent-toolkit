//! Content-Addressable Artifact Storage System
//!
//! This module implements deterministic artifact storage with content-addressable
//! organization and atomic write operations.

use crate::models::error::TemplateError;
use crate::services::unified_ast_engine::{ArtifactTree, MermaidArtifacts, Template};
use blake3::Hash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Content-addressable artifact writer with atomic operations
pub struct ArtifactWriter {
    root: PathBuf,
    manifest: BTreeMap<String, ArtifactMetadata>,
}

/// Metadata for each artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub path: PathBuf,
    pub hash: String,
    pub size: usize,
    pub generated_at: DateTime<Utc>,
    pub artifact_type: ArtifactType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    DogfoodingMarkdown,
    DogfoodingJson,
    MermaidDiagram,
    Template,
    Manifest,
}

impl ArtifactWriter {
    /// Create a new artifact writer for the given root directory
    pub fn new(root: PathBuf) -> Result<Self, TemplateError> {
        // Ensure root directory exists
        fs::create_dir_all(&root).map_err(TemplateError::Io)?;

        // Load existing manifest if it exists
        let manifest_path = root.join("artifacts.json");
        let manifest = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).map_err(TemplateError::Io)?;
            serde_json::from_str(&content).map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?
        } else {
            BTreeMap::new()
        };

        Ok(Self { root, manifest })
    }

    /// Write complete artifact tree to storage
    pub fn write_artifacts(&mut self, tree: &ArtifactTree) -> Result<(), TemplateError> {
        // Ensure directory structure exists
        self.create_directory_structure()?;

        // Write dogfooding artifacts
        for (name, content) in &tree.dogfooding {
            let artifact_type = if name.ends_with(".md") {
                ArtifactType::DogfoodingMarkdown
            } else {
                ArtifactType::DogfoodingJson
            };

            let path = self.root.join("dogfooding").join(name);
            let hash = self.write_with_hash(&path, content, artifact_type.clone())?;

            self.manifest.insert(
                name.clone(),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: content.len(),
                    generated_at: Utc::now(),
                    artifact_type,
                },
            );
        }

        // Write Mermaid diagrams with directory structure
        self.write_mermaid_artifacts(&tree.mermaid)?;

        // Write templates
        self.write_template_artifacts(&tree.templates)?;

        // Write manifest for verification
        self.write_manifest()?;

        Ok(())
    }

    /// Create the canonical directory structure
    fn create_directory_structure(&self) -> Result<(), TemplateError> {
        let directories = [
            "dogfooding",
            "mermaid",
            "mermaid/ast-generated",
            "mermaid/ast-generated/simple",
            "mermaid/ast-generated/styled",
            "mermaid/non-code",
            "mermaid/non-code/simple",
            "mermaid/non-code/styled",
            "mermaid/fixtures",
            "templates",
        ];

        for dir in &directories {
            let path = self.root.join(dir);
            fs::create_dir_all(&path).map_err(TemplateError::Io)?;
        }

        Ok(())
    }

    /// Write Mermaid artifacts with proper directory organization
    fn write_mermaid_artifacts(
        &mut self,
        artifacts: &MermaidArtifacts,
    ) -> Result<(), TemplateError> {
        // Write AST-generated diagrams
        for (name, content) in &artifacts.ast_generated {
            let subdir = if name.contains("styled") {
                "styled"
            } else {
                "simple"
            };
            let path = self
                .root
                .join("mermaid")
                .join("ast-generated")
                .join(subdir)
                .join(name);

            let hash = self.write_with_hash(&path, content, ArtifactType::MermaidDiagram)?;

            self.manifest.insert(
                format!("mermaid/ast-generated/{subdir}/{name}"),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: content.len(),
                    generated_at: Utc::now(),
                    artifact_type: ArtifactType::MermaidDiagram,
                },
            );
        }

        // Write non-code diagrams
        for (name, content) in &artifacts.non_code {
            let subdir = if name.contains("styled") {
                "styled"
            } else {
                "simple"
            };
            let path = self
                .root
                .join("mermaid")
                .join("non-code")
                .join(subdir)
                .join(name);

            let hash = self.write_with_hash(&path, content, ArtifactType::MermaidDiagram)?;

            self.manifest.insert(
                format!("mermaid/non-code/{subdir}/{name}"),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: content.len(),
                    generated_at: Utc::now(),
                    artifact_type: ArtifactType::MermaidDiagram,
                },
            );
        }

        Ok(())
    }

    /// Write template artifacts
    fn write_template_artifacts(&mut self, templates: &[Template]) -> Result<(), TemplateError> {
        for template in templates {
            let filename = format!("{}.hbs", template.name);
            let path = self.root.join("templates").join(&filename);

            let hash = self.write_with_hash(&path, &template.content, ArtifactType::Template)?;

            self.manifest.insert(
                format!("templates/{filename}"),
                ArtifactMetadata {
                    path: path.clone(),
                    hash: format!("{hash}"),
                    size: template.content.len(),
                    generated_at: Utc::now(),
                    artifact_type: ArtifactType::Template,
                },
            );
        }

        Ok(())
    }

    /// Write content with atomic operation and return hash
    fn write_with_hash(
        &self,
        path: &Path,
        content: &str,
        _artifact_type: ArtifactType,
    ) -> Result<Hash, TemplateError> {
        // Compute hash first
        let hash = blake3::hash(content.as_bytes());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(TemplateError::Io)?;
        }

        // Atomic write with temp file
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content).map_err(TemplateError::Io)?;
        fs::rename(temp_path, path).map_err(TemplateError::Io)?;

        Ok(hash)
    }

    /// Write the manifest file
    fn write_manifest(&mut self) -> Result<(), TemplateError> {
        let manifest_path = self.root.join("artifacts.json");
        let manifest_content = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        // Compute hash and add manifest to itself
        let hash = blake3::hash(manifest_content.as_bytes());
        self.manifest.insert(
            "artifacts.json".to_string(),
            ArtifactMetadata {
                path: manifest_path.clone(),
                hash: format!("{hash}"),
                size: manifest_content.len(),
                generated_at: Utc::now(),
                artifact_type: ArtifactType::Manifest,
            },
        );

        // Re-serialize with updated manifest
        let final_content = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

        // Atomic write
        let temp_path = manifest_path.with_extension("tmp");
        {
            let file = File::create(&temp_path).map_err(TemplateError::Io)?;
            let mut writer = BufWriter::new(file);
            writer
                .write_all(final_content.as_bytes())
                .map_err(TemplateError::Io)?;
            writer.flush().map_err(TemplateError::Io)?;
        }
        fs::rename(temp_path, manifest_path).map_err(TemplateError::Io)?;

        Ok(())
    }

    /// Verify artifact integrity using stored hashes
    pub fn verify_integrity(&self) -> Result<VerificationReport, TemplateError> {
        let mut report = VerificationReport {
            total_artifacts: self.manifest.len(),
            verified: 0,
            failed: Vec::new(),
            missing: Vec::new(),
        };

        for (name, metadata) in &self.manifest {
            if !metadata.path.exists() {
                report.missing.push(name.clone());
                continue;
            }

            // Read file and compute hash
            let content = fs::read_to_string(&metadata.path).map_err(TemplateError::Io)?;
            let computed_hash = blake3::hash(content.as_bytes());

            if format!("{computed_hash}") == metadata.hash {
                report.verified += 1;
            } else {
                report.failed.push(IntegrityFailure {
                    artifact: name.clone(),
                    expected_hash: metadata.hash.clone(),
                    actual_hash: format!("{computed_hash}"),
                });
            }
        }

        Ok(report)
    }

    /// Get artifact statistics
    pub fn get_statistics(&self) -> ArtifactStatistics {
        let mut stats = ArtifactStatistics {
            total_artifacts: self.manifest.len(),
            total_size: 0,
            by_type: BTreeMap::new(),
            oldest: None,
            newest: None,
        };

        for metadata in self.manifest.values() {
            stats.total_size += metadata.size;

            let type_stats = stats
                .by_type
                .entry(format!("{:?}", metadata.artifact_type))
                .or_insert(TypeStatistics { count: 0, size: 0 });
            type_stats.count += 1;
            type_stats.size += metadata.size;

            if stats.oldest.is_none() || stats.oldest.as_ref().unwrap() > &metadata.generated_at {
                stats.oldest = Some(metadata.generated_at);
            }

            if stats.newest.is_none() || stats.newest.as_ref().unwrap() < &metadata.generated_at {
                stats.newest = Some(metadata.generated_at);
            }
        }

        stats
    }

    /// Clean up artifacts older than specified duration
    pub fn cleanup_old_artifacts(
        &mut self,
        max_age_days: u32,
    ) -> Result<CleanupReport, TemplateError> {
        let cutoff = Utc::now() - chrono::Duration::days(max_age_days as i64);
        let mut removed = Vec::new();
        let mut failed = Vec::new();

        let old_artifacts: Vec<_> = self
            .manifest
            .iter()
            .filter(|(_, metadata)| metadata.generated_at < cutoff)
            .map(|(name, _)| name.clone())
            .collect();

        for name in old_artifacts {
            if let Some(metadata) = self.manifest.remove(&name) {
                match fs::remove_file(&metadata.path) {
                    Ok(()) => removed.push(name),
                    Err(e) => {
                        failed.push((name.clone(), e.to_string()));
                        // Re-add to manifest if removal failed
                        self.manifest.insert(name, metadata);
                    }
                }
            }
        }

        // Update manifest if any files were removed
        if !removed.is_empty() {
            self.write_manifest()?;
        }

        Ok(CleanupReport { removed, failed })
    }
}

/// Verification report for artifact integrity
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub total_artifacts: usize,
    pub verified: usize,
    pub failed: Vec<IntegrityFailure>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IntegrityFailure {
    pub artifact: String,
    pub expected_hash: String,
    pub actual_hash: String,
}

/// Statistics about stored artifacts
#[derive(Debug, Clone)]
pub struct ArtifactStatistics {
    pub total_artifacts: usize,
    pub total_size: usize,
    pub by_type: BTreeMap<String, TypeStatistics>,
    pub oldest: Option<DateTime<Utc>>,
    pub newest: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct TypeStatistics {
    pub count: usize,
    pub size: usize,
}

/// Report from cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub removed: Vec<String>,
    pub failed: Vec<(String, String)>, // (artifact_name, error_message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    #[test]
    fn test_artifact_writer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(writer.manifest.len(), 0);
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_directory_structure_creation() {
        let temp_dir = TempDir::new().unwrap();
        let writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();
        writer.create_directory_structure().unwrap();

        // Check that all expected directories exist
        let expected_dirs = [
            "dogfooding",
            "mermaid/ast-generated/simple",
            "mermaid/ast-generated/styled",
            "mermaid/non-code/simple",
            "mermaid/non-code/styled",
            "templates",
        ];

        for dir in &expected_dirs {
            let path = temp_dir.path().join(dir);
            assert!(path.exists(), "Directory {dir} should exist");
            assert!(path.is_dir(), "Path {dir} should be a directory");
        }
    }

    #[test]
    fn test_atomic_write_with_hash() {
        let temp_dir = TempDir::new().unwrap();
        let writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();

        let content = "Hello, World!";
        let file_path = temp_dir.path().join("test.txt");

        let hash = writer
            .write_with_hash(&file_path, content, ArtifactType::DogfoodingMarkdown)
            .unwrap();

        // Verify file exists and content is correct
        assert!(file_path.exists());
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);

        // Verify hash is correct
        let expected_hash = blake3::hash(content.as_bytes());
        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_artifact_tree_writing() {
        let temp_dir = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();

        // Create test artifact tree
        let mut dogfooding = BTreeMap::new();
        dogfooding.insert("test.md".to_string(), "# Test Markdown".to_string());
        dogfooding.insert(
            "metrics.json".to_string(),
            r#"{"test": "data"}"#.to_string(),
        );

        let mut ast_generated = BTreeMap::new();
        ast_generated.insert(
            "simple-diagram.mmd".to_string(),
            "graph TD\n  A --> B".to_string(),
        );

        let mermaid = MermaidArtifacts {
            ast_generated,
            non_code: BTreeMap::new(),
        };

        let templates = vec![Template {
            name: "test_template".to_string(),
            content: "Hello {{name}}!".to_string(),
            hash: blake3::hash(b"Hello {{name}}!"),
            source_location: PathBuf::from("test.rs"),
        }];

        let tree = ArtifactTree {
            dogfooding,
            mermaid,
            templates,
        };

        // Write artifacts
        writer.write_artifacts(&tree).unwrap();

        // Verify files exist
        assert!(temp_dir.path().join("dogfooding/test.md").exists());
        assert!(temp_dir.path().join("dogfooding/metrics.json").exists());
        assert!(temp_dir
            .path()
            .join("mermaid/ast-generated/simple/simple-diagram.mmd")
            .exists());
        assert!(temp_dir.path().join("templates/test_template.hbs").exists());
        assert!(temp_dir.path().join("artifacts.json").exists());

        // Verify manifest contains all artifacts
        assert!(writer.manifest.len() >= 4); // At least the files we created
    }

    #[test]
    fn test_integrity_verification() {
        let temp_dir = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();

        // Write a test file
        let content = "Test content";
        let file_path = temp_dir.path().join("test.txt");
        let hash = writer
            .write_with_hash(&file_path, content, ArtifactType::DogfoodingMarkdown)
            .unwrap();

        // Add to manifest
        writer.manifest.insert(
            "test.txt".to_string(),
            ArtifactMetadata {
                path: file_path.clone(),
                hash: format!("{hash}"),
                size: content.len(),
                generated_at: Utc::now(),
                artifact_type: ArtifactType::DogfoodingMarkdown,
            },
        );

        // Verify integrity - should pass
        let report = writer.verify_integrity().unwrap();
        assert_eq!(report.verified, 1);
        assert_eq!(report.failed.len(), 0);
        assert_eq!(report.missing.len(), 0);

        // Corrupt the file
        fs::write(&file_path, "Corrupted content").unwrap();

        // Verify integrity - should fail
        let report = writer.verify_integrity().unwrap();
        assert_eq!(report.verified, 0);
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.missing.len(), 0);
    }

    #[test]
    fn test_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let mut writer = ArtifactWriter::new(temp_dir.path().to_path_buf()).unwrap();

        // Add some test metadata
        writer.manifest.insert(
            "test1.md".to_string(),
            ArtifactMetadata {
                path: temp_dir.path().join("test1.md"),
                hash: "hash1".to_string(),
                size: 100,
                generated_at: Utc::now(),
                artifact_type: ArtifactType::DogfoodingMarkdown,
            },
        );

        writer.manifest.insert(
            "test2.json".to_string(),
            ArtifactMetadata {
                path: temp_dir.path().join("test2.json"),
                hash: "hash2".to_string(),
                size: 200,
                generated_at: Utc::now(),
                artifact_type: ArtifactType::DogfoodingJson,
            },
        );

        let stats = writer.get_statistics();

        assert_eq!(stats.total_artifacts, 2);
        assert_eq!(stats.total_size, 300);
        assert!(stats.by_type.contains_key("DogfoodingMarkdown"));
        assert!(stats.by_type.contains_key("DogfoodingJson"));
        assert!(stats.oldest.is_some());
        assert!(stats.newest.is_some());
    }
}
