// Lean 4 analysis methods for FormalVerificationScorer
// Included by formal_verification_scorer.rs — shares parent module scope

impl FormalVerificationScorer {
    /// Check if project is a Lean 4 project (lakefile.lean or lean-toolchain at root or lean/)
    fn is_lean_project(&self, project_path: &Path) -> bool {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        project_path.join("lakefile.lean").exists()
            || project_path.join("lean-toolchain").exists()
            || project_path.join("lean").join("lakefile.lean").exists()
            || project_path.join("lean").join("lean-toolchain").exists()
    }

    /// Score Lean 4 proofs. For Lean-only projects (no Cargo.toml), Lean proofs ARE
    /// the formal verification mechanism — scale to fill available points (13pts).
    /// For mixed Rust+Lean projects, use the standard 3-point allocation.
    ///
    /// Also gives partial credit to consumer repos that reference Lean theorems
    /// via `lean_theorem:` in their contract YAML files (even without lean/ subdir).
    fn score_lean(&self, project_path: &Path) -> f64 {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        if self.is_lean_project(project_path) {
            let is_rust = project_path.join("Cargo.toml").exists();
            let lean_max = if is_rust {
                LEAN_POINTS
            } else {
                MAX_POINTS - MIRI_POINTS
            };

            let theorems = self.count_lean_theorems(project_path);
            let sorrys = self.count_lean_sorrys(project_path);

            return if theorems > 0 {
                let proven = theorems.saturating_sub(sorrys);
                let ratio = proven as f64 / theorems as f64;
                lean_max * ratio
            } else {
                lean_max * 0.1
            };
        }

        // Consumer repos: check for lean_theorem references in contracts/ YAML
        let contracts_dir = project_path.join("contracts");
        if contracts_dir.exists() {
            let mut lean_refs = 0usize;
            if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().is_some_and(|e| e == "yaml") {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            lean_refs += content.matches("lean_theorem:").count();
                        }
                    }
                }
            }
            if lean_refs > 0 {
                // Partial credit: contracts reference Lean proofs (proven in provable-contracts)
                let ratio = (lean_refs as f64 / 10.0).min(1.0);
                return LEAN_POINTS * ratio * 0.8; // 80% of max (proofs are external)
            }
        }

        0.0
    }

    /// Count theorems and lemmas in .lean files
    fn count_lean_theorems(&self, project_path: &Path) -> usize {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let theorem_pattern = Regex::new(r"^\s*(theorem|lemma|private theorem|private lemma)\s+")
            .expect("internal error");
        let mut count = 0;

        for entry in walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_some_and(|ext| ext == "lean") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    count += content
                        .lines()
                        .filter(|line| theorem_pattern.is_match(line))
                        .count();
                }
            }
        }

        count
    }

    /// Count sorry occurrences in .lean files (incomplete proofs)
    /// Respects block comments (/- ... -/) and line comments (--)
    fn count_lean_sorrys(&self, project_path: &Path) -> usize {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut total = 0;

        for entry in walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_some_and(|ext| ext == "lean") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    total += Self::count_sorrys_in_content(&content);
                }
            }
        }

        total
    }

    /// Count sorry occurrences in a single file's content, respecting comments
    fn count_sorrys_in_content(content: &str) -> usize {
        let mut count = 0;
        let mut in_block_comment = 0i32;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("--") {
                continue;
            }

            // Strip block comments inline for same-line handling
            let cleaned =
                Self::strip_lean_block_comments_inline(trimmed, &mut in_block_comment);

            if in_block_comment > 0 {
                continue;
            }

            if Self::contains_sorry_word(&cleaned) {
                count += 1;
            }
        }

        count
    }

    /// Strips block comment content from a line, updating nesting depth.
    fn strip_lean_block_comments_inline(line: &str, depth: &mut i32) -> String {
        let bytes = line.as_bytes();
        let mut result = String::with_capacity(line.len());
        let mut i = 0;

        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
                *depth += 1;
                i += 2;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
                *depth -= 1;
                i += 2;
                continue;
            }
            if *depth == 0 {
                result.push(bytes[i] as char);
            }
            i += 1;
        }

        result
    }

    /// Checks if line contains "sorry" as a standalone word.
    fn contains_sorry_word(line: &str) -> bool {
        let bytes = line.as_bytes();
        let sorry = b"sorry";
        let mut pos = 0;
        while pos + sorry.len() <= bytes.len() {
            if let Some(idx) = line[pos..].find("sorry") {
                let abs_idx = pos + idx;
                let before_ok = abs_idx == 0
                    || !(bytes[abs_idx - 1].is_ascii_alphanumeric() || bytes[abs_idx - 1] == b'_');
                let after_ok = abs_idx + sorry.len() >= bytes.len()
                    || !(bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                        || bytes[abs_idx + sorry.len()] == b'_');
                if before_ok && after_ok {
                    return true;
                }
                pos = abs_idx + 1;
            } else {
                break;
            }
        }
        false
    }
}
