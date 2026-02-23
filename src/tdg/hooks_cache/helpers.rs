#![cfg_attr(coverage_nightly, coverage(off))]
//! Private helper methods for the Hooks Cache Manager.
//!
//! Contains file hashing, git tree hash retrieval, config hashing,
//! metrics persistence, and cache size calculation.

use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use super::types::HooksCacheMetrics;
use super::HooksCacheManager;

impl HooksCacheManager {
    /// Hash a list of files for Level 1/2 caching
    pub(super) fn hash_files(&self, files: &[std::path::PathBuf]) -> Result<String> {
        let mut hasher = blake3::Hasher::new();

        for file in files {
            let path = if file.is_absolute() {
                file.clone()
            } else {
                self.project_path.join(file)
            };

            if path.exists() {
                let content = fs::read(&path)?;
                hasher.update(&content);
                // Also hash the path for uniqueness
                hasher.update(file.to_string_lossy().as_bytes());
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Get git tree hash for HEAD
    pub(super) fn get_tree_hash(&self) -> Result<String> {
        // Use HEAD^{tree} to get the tree hash of the root tree
        // This is more reliable than HEAD:. across different git versions
        let output = Command::new("git")
            .args(["rev-parse", "HEAD^{tree}"])
            .current_dir(&self.project_path)
            .output()
            .context("Failed to get git tree hash")?;

        if !output.status.success() {
            anyhow::bail!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get hash of config files
    pub(super) fn get_config_hash(&self) -> Result<String> {
        let mut hasher = blake3::Hasher::new();

        // Hash tdg-rules.toml if it exists
        let rules_path = self.project_path.join(".pmat/tdg-rules.toml");
        if rules_path.exists() {
            let content = fs::read(&rules_path)?;
            hasher.update(&content);
        }

        // Hash pmat.toml if it exists
        let pmat_path = self.project_path.join("pmat.toml");
        if pmat_path.exists() {
            let content = fs::read(&pmat_path)?;
            hasher.update(&content);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Save metrics to file
    pub(super) fn save_metrics(&self, metrics: &HooksCacheMetrics) -> Result<()> {
        let metrics_path = self.cache_dir.join("metrics.json");
        let content = serde_json::to_string_pretty(metrics)?;
        fs::write(metrics_path, content)?;
        Ok(())
    }

    /// Calculate total cache size
    pub(super) fn calculate_cache_size(&self) -> Result<u64> {
        let mut size = 0u64;
        if self.cache_dir.exists() {
            for entry in walkdir::WalkDir::new(&self.cache_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
        Ok(size)
    }
}
