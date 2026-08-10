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

    /// Get the tree hash of the content the gates are about to check.
    ///
    /// This used to be `git rev-parse HEAD^{tree}` — the tree of the *last
    /// commit*. A pre-commit run keyed that way is invariant to exactly the
    /// change it is gating: staging a new FIXME and re-running produced the same
    /// key, so the hook reported "All quality gates passed (cached)" without
    /// looking at the staged code. `git write-tree` hashes the index — the
    /// content a commit would actually contain — which is what must key the
    /// cache. (It writes the tree objects into the object store, the same
    /// harmless side effect `git commit` has.)
    pub(super) fn get_tree_hash(&self) -> Result<String> {
        if let Some(index_tree) = self.git_stdout(&["write-tree"]) {
            return Ok(index_tree);
        }

        // No usable index (unmerged state, bare repo): fall back to the last
        // commit's tree so whole-tree/CI style runs still get a key.
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

    /// Run a git command in the project, returning trimmed stdout on success
    fn git_stdout(&self, args: &[&str]) -> Option<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.project_path)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            None
        } else {
            Some(stdout)
        }
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

#[cfg(test)]
mod tree_hash_tests {
    use super::HooksCacheManager;
    use std::process::Command;

    fn git(dir: &std::path::Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?} failed to spawn: {e}"));
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn repo_with_one_commit() -> tempfile::TempDir {
        let temp = tempfile::TempDir::new().unwrap();
        git(temp.path(), &["init"]);
        git(temp.path(), &["config", "user.email", "test@test.com"]);
        git(temp.path(), &["config", "user.name", "Test User"]);
        std::fs::write(temp.path().join("lib.rs"), "pub fn ok() {}\n").unwrap();
        git(temp.path(), &["add", "-A"]);
        git(temp.path(), &["commit", "-m", "initial"]);
        temp
    }

    #[test]
    fn test_tree_hash_follows_the_staged_index_not_the_last_commit() {
        // Regression: keyed on HEAD^{tree}, a pre-commit run was invariant to
        // the very change it was gating — stage a FIXME, get a cached pass.
        let temp = repo_with_one_commit();
        let manager = HooksCacheManager::new(temp.path());

        let before = manager.get_tree_hash().unwrap();

        std::fs::write(
            temp.path().join("lib.rs"),
            "pub fn ok() {}\n// FIXME: broken\n",
        )
        .unwrap();
        git(temp.path(), &["add", "-A"]);

        let after = manager.get_tree_hash().unwrap();
        assert_ne!(
            before, after,
            "staged content must change the hooks cache key"
        );
        assert_eq!(after, git(temp.path(), &["write-tree"]));
        assert_ne!(after, git(temp.path(), &["rev-parse", "HEAD^{tree}"]));
    }
}
