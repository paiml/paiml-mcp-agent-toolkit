// include!()'d from executor.rs
// Helper methods: backup, test execution, atomic write, parsing

impl MutantExecutor {
    /// These functions are deprecated - use MutantGuard instead
    /// Kept for backward compatibility
    /// Create backup of original file (deprecated)
    #[deprecated(since = "2.171.0", note = "Use MutantGuard instead")]
    #[allow(dead_code)]
    async fn create_backup(&self, original_path: &Path) -> Result<PathBuf> {
        let backup_path = original_path.with_extension("pmat_backup");
        fs::copy(original_path, &backup_path)
            .await
            .context("Failed to create backup")?;
        Ok(backup_path)
    }

    /// Restore original file from backup (deprecated)
    #[deprecated(since = "2.171.0", note = "Use MutantGuard instead")]
    #[allow(dead_code)]
    async fn restore_backup(&self, original_path: &Path, backup_path: &Path) -> Result<()> {
        fs::copy(backup_path, original_path)
            .await
            .context("Failed to restore backup")?;
        fs::remove_file(backup_path)
            .await
            .context("Failed to remove backup")?;
        Ok(())
    }

    /// Run cargo test in working directory with smart test filtering
    async fn run_cargo_test_for_mutant(&self, mutant: &Mutant) -> Result<String> {
        // Extract module path for test filtering
        let module_filter = self.extract_module_path(&mutant.original_file);

        let mut cmd = Command::new("cargo");
        cmd.arg("test").arg("--lib");

        // Add module filter if present
        if !module_filter.is_empty() {
            cmd.arg("--").arg(&module_filter);
        }

        let output = cmd
            .current_dir(&self.work_dir)
            .output()
            .context("Failed to run cargo test")?;

        // Combine stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);

        Ok(combined)
    }

    /// Atomically write content to a file (BUG-064 FIX)
    ///
    /// Uses write-to-temp-then-rename pattern to ensure atomic file operations.
    /// This prevents file corruption if the process is interrupted during write.
    ///
    /// # Arguments
    ///
    /// * `path` - Target file path
    /// * `content` - Content to write
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error
    ///
    /// # Implementation
    ///
    /// 1. Create temp file in same directory (ensures atomic rename works)
    /// 2. Write content to temp file
    /// 3. Flush and sync to ensure data is on disk
    /// 4. Atomically rename temp file to target (Unix atomic operation)
    /// 5. On error, clean up temp file
    ///
    /// This ensures the target file is either:
    /// - Fully updated with new content, OR
    /// - Completely unchanged (original content preserved)
    ///
    /// Never partially written (which causes "491 lines -> 5 lines" bug)
    async fn atomic_write(&self, path: &Path, content: &str) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        // Create scratch file in same directory as target
        // (required for atomic rename on same filesystem)
        let temp_path = path.with_extension("pmat_tmp");

        // Write to scratch file
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .with_context(|| format!("Failed to create scratch file: {}", temp_path.display()))?;

        file.write_all(content.as_bytes())
            .await
            .context("Failed to write to scratch file")?;

        // Flush and sync to ensure data is on disk
        file.flush().await.context("Failed to flush scratch file")?;
        file.sync_all()
            .await
            .context("Failed to sync scratch file")?;

        // Close the file explicitly
        drop(file);

        // Atomically rename scratch file to target
        // This is atomic on Unix - either succeeds completely or fails completely
        tokio::fs::rename(&temp_path, path).await.with_context(|| {
            format!(
                "Failed to atomically rename {} to {}",
                temp_path.display(),
                path.display()
            )
        })?;

        Ok(())
    }

    /// Extract module path from file path for smart test filtering
    ///
    /// This implements the Toyota Way fix: only run tests relevant to the mutation
    /// instead of the entire test suite.
    fn extract_module_path(&self, file_path: &Path) -> String {
        let path_str = file_path.to_str().unwrap_or("");

        // Handle external crates (paths starting with ../)
        if path_str.starts_with("../") || path_str.starts_with("..\\") {
            return String::new(); // Use package-level testing
        }

        // Handle workspace crates (crates/foo/src/bar.rs)
        // Extract just the module path from the crate
        let relative = if let Some(after_crates) = path_str.strip_prefix("crates/") {
            // Find the crate name boundary (e.g., "pforge-config/src/validator.rs")
            if let Some(src_index) = after_crates.find("/src/") {
                // Get everything after "/src/" (e.g., "validator.rs")
                after_crates.get(src_index + 5..).unwrap_or_default()
            } else {
                after_crates
            }
        } else {
            // Remove "server/src/" or "src/" prefix for regular paths
            path_str
                .strip_prefix("server/src/")
                .or_else(|| path_str.strip_prefix("src/"))
                .unwrap_or(path_str)
        };

        // Remove ".rs" suffix
        let without_ext = relative.strip_suffix(".rs").unwrap_or(relative);

        // Handle lib.rs and main.rs - run all tests
        if without_ext == "lib" || without_ext == "main" {
            return String::new();
        }

        // Check if this is a mod.rs file
        let is_mod_file = without_ext.ends_with("/mod");

        // Remove "/mod" at end for processing
        let without_mod = without_ext.strip_suffix("/mod").unwrap_or(without_ext);

        // Split into parts
        let parts: Vec<&str> = without_mod.split('/').collect();

        // Determine which parts to use
        let module_parts = if is_mod_file {
            // For mod.rs files, keep full path
            &parts[..]
        } else if parts.len() > 3 {
            // For deep paths, use parent module for broader coverage
            &parts[..parts.len() - 1]
        } else if parts.len() > 1 {
            // For 2-3 levels, use parent module
            &parts[..parts.len() - 1]
        } else {
            // Single level, use as-is
            &parts[..]
        };

        // Join with "::"
        module_parts.join("::")
    }

    /// Parse test output to determine mutant status
    fn parse_test_output(&self, output: &str) -> (MutantStatus, Vec<String>, Option<String>) {
        // Check for compilation errors
        if output.contains("error: could not compile") || output.contains("error[E") {
            return (
                MutantStatus::CompileError,
                vec![],
                Some("Compilation failed".to_string()),
            );
        }

        // Extract test failures
        let test_failures = self.extract_test_failures(output);

        // Determine status based on test results
        let status = if !test_failures.is_empty() {
            // At least one test failed -> mutant was killed
            MutantStatus::Killed
        } else if output.contains("test result: ok") {
            // All tests passed -> mutant survived
            MutantStatus::Survived
        } else {
            // Unclear status, default to survived
            MutantStatus::Survived
        };

        (status, test_failures, None)
    }

    /// Extract failed test names from output
    fn extract_test_failures(&self, output: &str) -> Vec<String> {
        let mut failures = Vec::new();

        for line in output.lines() {
            // Look for "test <name> ... FAILED" pattern (not "test result:")
            if line.starts_with("test ")
                && line.contains("... FAILED")
                && !line.starts_with("test result")
            {
                if let Some(test_name) = line.split_whitespace().nth(1) {
                    failures.push(test_name.to_string());
                }
            }
        }

        failures
    }
}
