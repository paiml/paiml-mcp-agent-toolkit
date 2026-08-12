// analyzer_simple_helpers.rs — Estimation and file discovery helpers for TdgAnalyzer
// Included by analyzer_simple.rs — shares parent module scope

impl TdgAnalyzer {
    fn estimate_cyclomatic_complexity(&self, lines: &[&str]) -> u32 {
        let mut complexity = 1;

        for line in lines {
            let trimmed = line.trim();
            complexity += count_control_flow_keywords(trimmed);
            complexity += count_logical_operators(trimmed);
        }

        complexity
    }

    fn estimate_nesting_depth(&self, source: &str) -> usize {
        let mut max_depth = 0;
        let mut current_depth = 0;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.contains('{') {
                current_depth += trimmed.matches('{').count();
                max_depth = max_depth.max(current_depth);
            }
            if trimmed.contains('}') {
                current_depth = current_depth.saturating_sub(trimmed.matches('}').count());
            }
        }

        max_depth
    }

    fn estimate_duplication_ratio(&self, source: &str) -> f32 {
        let lines: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
            .collect();

        if lines.len() < 3 {
            return 0.0;
        }

        let mut duplicates = 0;
        for i in 0..lines.len() {
            for j in i + 1..lines.len() {
                if lines[i] == lines[j] && lines[i].len() > 10 {
                    duplicates += 1;
                }
            }
        }

        duplicates as f32 / lines.len() as f32
    }

    /// Walk `dir`, keeping a record of the files this build cannot grade.
    ///
    /// This used to be a hand-rolled `read_dir` recursion over a hardcoded
    /// extension whitelist that dropped everything else without a trace — the
    /// walk behind `quality_gate`'s `not_measured: []` for a tree it graded half
    /// of. The walk, the skip list and the classification are now
    /// `crate::tdg::file_discovery`, shared with the AST analyzer.
    fn discover_files(&self, dir: &Path) -> Result<crate::tdg::file_discovery::Discovery> {
        crate::tdg::file_discovery::discover(dir, crate::tdg::file_discovery::Policy::heuristic())
    }

    #[cfg(test)]
    fn should_skip_directory(&self, path: &Path) -> bool {
        crate::tdg::file_discovery::is_skipped_directory(path, false)
    }

    #[cfg(test)]
    fn should_analyze_file(&self, path: &Path) -> bool {
        crate::tdg::file_discovery::is_gradable_path(
            path,
            crate::tdg::file_discovery::Policy::heuristic(),
        )
    }
}

// The Lean `sorry` counter that lived here was a byte-for-byte copy of the one
// in `analyzer_ast/analyzer_impl2_heuristics_lean.rs`. Both are gone: the rule
// is `crate::tdg::critical_defect_gate::count_lean_sorry`, applied to BOTH
// analyzers by the shared gate.

/// Count control flow keywords in a single trimmed line.
fn count_control_flow_keywords(trimmed: &str) -> u32 {
    let mut count = 0;
    if trimmed.starts_with("if ") || trimmed.contains(" if ") {
        count += 1;
    }
    if trimmed.starts_with("for ") || trimmed.contains(" for ") {
        count += 1;
    }
    if trimmed.starts_with("while ") || trimmed.contains(" while ") {
        count += 1;
    }
    if trimmed.starts_with("match ") || trimmed.contains(" match ") {
        count += 1;
    }
    count
}

/// Count logical operators (&& and ||) in a single trimmed line.
fn count_logical_operators(trimmed: &str) -> u32 {
    if trimmed.contains(" && ") || trimmed.contains(" || ") {
        trimmed.matches(" && ").count() as u32 + trimmed.matches(" || ").count() as u32
    } else {
        0
    }
}
