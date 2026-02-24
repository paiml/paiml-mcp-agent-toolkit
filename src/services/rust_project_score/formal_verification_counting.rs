// Counting and detection methods for FormalVerificationScorer
// Included by formal_verification_scorer.rs — shares parent module scope

impl FormalVerificationScorer {
    /// Check if Miri is available
    fn is_miri_available(&self) -> bool {
        Command::new("cargo")
            .args(["miri", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if Kani is available
    fn is_kani_available(&self) -> bool {
        Command::new("cargo")
            .args(["kani", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if Verus is available
    fn is_verus_available(&self) -> bool {
        Command::new("verus")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Count unsafe blocks in the project
    fn count_unsafe_blocks(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        let unsafe_pattern = Regex::new(r"\bunsafe\s*\{").expect("internal error");
        let mut count = 0;
        let src_path = project_path.join("src");

        if let Some(file_cache) = cache {
            // Use cache for efficiency (Kaizen Round 4)
            for (_path, content) in file_cache.get_rust_files_in_dir(&src_path) {
                count += unsafe_pattern.find_iter(content).count();
            }
        } else {
            // Fallback: walk directory
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        count += unsafe_pattern.find_iter(&content).count();
                    }
                }
            }
        }

        count
    }

    /// Check for Kani proofs in the project
    fn count_kani_proofs(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        // Look for #[kani::proof] attributes
        let proof_pattern = Regex::new(r"#\[kani::proof\]").expect("internal error");
        let mut count = 0;
        let src_path = project_path.join("src");

        if let Some(file_cache) = cache {
            for (_path, content) in file_cache.get_rust_files_in_dir(&src_path) {
                count += proof_pattern.find_iter(content).count();
            }
        } else {
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        count += proof_pattern.find_iter(&content).count();
                    }
                }
            }
        }

        count
    }

    /// Count Verus specification attributes in the project
    ///
    /// Looks for #[requires(...)], #[ensures(...)], #[invariant(...)] attributes
    /// which are the core Verus specification constructs.
    fn count_verus_specs(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        // Verus uses requires/ensures/invariant attributes for specifications
        // Also check for proof blocks and spec functions
        let spec_pattern = Regex::new(r"#\[(requires|ensures|invariant|decreases|recommends)\s*\(")
            .expect("internal error");
        let mut count = 0;
        let src_path = project_path.join("src");

        if let Some(file_cache) = cache {
            for (_path, content) in file_cache.get_rust_files_in_dir(&src_path) {
                count += spec_pattern.find_iter(content).count();
            }
        } else {
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        count += spec_pattern.find_iter(&content).count();
                    }
                }
            }
        }

        count
    }

    /// Check for vstd dependency in Cargo.toml (indicates Verus project)
    fn has_vstd_dependency(&self, project_path: &Path) -> bool {
        let cargo_toml = project_path.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(cargo_toml) {
            // Check for vstd or builtin dependencies (Verus standard library)
            content.contains("vstd") || content.contains("builtin")
        } else {
            false
        }
    }
}
