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

    /// Check if Verus is available (reserved for future use)
    #[allow(dead_code)]
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

    /// Count Verus specification attributes in the project.
    ///
    /// Only counts REAL Verus specs (`decreases`, `recommends`, `proof fn`,
    /// `spec fn`). Does NOT count `#[requires]`/`#[ensures]` — those are
    /// provable-contracts macros, scored separately under contract coverage.
    fn count_verus_specs(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        // Only count Verus-specific constructs, NOT provable-contracts macros
        let spec_pattern = Regex::new(r"#\[(decreases|recommends)\s*\(|(?:proof|spec)\s+fn\s")
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

    /// Count `#[contract("yaml", equation = "eq")]` annotations.
    /// These represent provable-contracts enforcement — YAML-driven
    /// assertions backed by Lean proofs and Kani harnesses.
    fn count_contract_macros(&self, project_path: &Path, cache: Option<&FileCache>) -> usize {
        let pattern = Regex::new(r#"#\[(?:provable_contracts_macros::)?contract\("#)
            .expect("internal error");
        let mut count = 0;
        let src_path = project_path.join("src");

        if let Some(file_cache) = cache {
            for (_path, content) in file_cache.get_rust_files_in_dir(&src_path) {
                count += pattern.find_iter(content).count();
            }
        } else {
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        count += pattern.find_iter(&content).count();
                    }
                }
            }
        }

        count
    }

    /// Check for vstd dependency in Cargo.toml (indicates Verus project)
    #[allow(dead_code)]
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
