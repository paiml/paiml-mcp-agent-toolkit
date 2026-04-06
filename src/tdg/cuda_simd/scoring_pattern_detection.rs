impl CudaSimdAnalyzer {

    fn check_flash_attention_tile_size(
        &self,
        content: &str,
        path: &Path,
        analysis: &mut FileAnalysis,
    ) {
        let has_flash = content.contains("FlashAttention")
            || content.contains("flash_attention")
            || content.contains("tiled_attention");
        if !has_flash {
            return;
        }
        let tile_kv = match self.extract_value(content, "tile_kv") {
            Some(v) => v,
            None => return,
        };
        let head_dim = match self.extract_value(content, "head_dim") {
            Some(v) => v,
            None => return,
        };
        if tile_kv >= head_dim {
            return;
        }
        if let Some(defect_class) = self.taxonomy.get("PAR-041") {
            analysis.defects.push(DetectedDefect {
                defect_class: defect_class.clone(),
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("tile_kv ({}) < head_dim ({})", tile_kv, head_dim)),
                suggestion: Some(format!("Set tile_kv >= head_dim (at least {})", head_dim)),
            });
        }
    }

    fn check_missing_tensor_core(
        &self,
        content: &str,
        path: &Path,
        analysis: &mut FileAnalysis,
    ) {
        let has_matmul = content.contains("matmul") || content.contains("gemm");
        let has_tensor = content.contains("wmma")
            || content.contains("mma")
            || content.contains("tensor_core");
        if !has_matmul || has_tensor {
            return;
        }
        if let Some(defect_class) = self.taxonomy.get("PAR-034") {
            analysis.defects.push(DetectedDefect {
                defect_class: defect_class.clone(),
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some("Matrix multiplication without Tensor Core".to_string()),
                suggestion: Some(
                    "Consider using wmma or mma instructions for better performance".to_string(),
                ),
            });
        }
    }

    fn detect_known_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.check_flash_attention_tile_size(content, path, analysis);
        self.check_missing_tensor_core(content, path, analysis);
    }

    fn extract_value(&self, content: &str, name: &str) -> Option<usize> {
        // Simple pattern matching for variable assignments
        let patterns = [
            format!("{} = ", name),
            format!("{}=", name),
            format!("const {} = ", name),
            format!("let {} = ", name),
        ];

        for pattern in &patterns {
            if let Some(pos) = content.find(pattern) {
                let after = content.get(pos + pattern.len()..).unwrap_or_default();
                let value_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(value) = value_str.parse() {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Check if any .rs file in src/backends contains SAFETY comments
    fn has_safety_comments(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let backends = path.join("src/backends");
        let entries = match std::fs::read_dir(&backends) {
            Ok(e) => e,
            Err(_) => return false,
        };
        entries.filter_map(Result::ok).any(|entry| {
            entry.path().extension().is_some_and(|e| e == "rs")
                && std::fs::read_to_string(entry.path())
                    .map(|c| c.contains("// SAFETY:") || c.contains("/// SAFETY:"))
                    .unwrap_or(false)
        })
    }

    /// Detect Rust project quality patterns for enhanced scoring
    fn detect_rust_patterns(&self, path: &Path) -> RustProjectPatterns {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let mut patterns = RustProjectPatterns::default();

        patterns.has_cargo_lock = path.join("Cargo.lock").exists();
        patterns.has_rust_toolchain =
            path.join("rust-toolchain.toml").exists() || path.join("rust-toolchain").exists();
        patterns.has_criterion_benches = path.join("benches").exists()
            && std::fs::read_dir(path.join("benches"))
                .map(|d| {
                    d.filter_map(Result::ok)
                        .any(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                })
                .unwrap_or(false);
        patterns.has_github_ci = path.join(".github/workflows").exists();
        patterns.has_proptest_regressions = path.join("proptest-regressions").exists();
        patterns.has_changelog =
            path.join("CHANGELOG.md").exists() || path.join("CHANGELOG").exists();
        patterns.has_golden_traces = path.join("golden_traces").exists();
        patterns.has_safety_comments =
            path.join("src/backends").exists() && Self::has_safety_comments(path);
        patterns.has_miri_config = path.join(".cargo/config.toml").exists()
            && std::fs::read_to_string(path.join(".cargo/config.toml"))
                .map(|c| c.contains("miri"))
                .unwrap_or(false);

        patterns
    }
}
