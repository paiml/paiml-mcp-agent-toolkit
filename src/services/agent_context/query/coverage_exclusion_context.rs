// ── ExclusionContext implementation + helpers ────────────────────────────────
// Included from coverage_exclusion.rs — do NOT add `use` imports or `#!` attributes here.

impl ExclusionContext {
    /// Build exclusion context from project state.
    ///
    /// When `cached_coverage_off` is provided (from SQLite index), skips all file I/O
    /// for coverage(off) detection — O(1) per file instead of O(file_size).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn build(
        project_path: &Path,
        cached_coverage_off: Option<&HashSet<String>>,
    ) -> Self {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let makefile_regex = parse_makefile_coverage_exclude(project_path);
        let dead_functions = load_dead_code_functions(project_path);
        let coverage_off_files = cached_coverage_off.cloned().unwrap_or_default();
        let use_cached = cached_coverage_off.is_some();
        Self {
            coverage_off_files,
            checked_files: HashSet::new(),
            makefile_regex,
            dead_functions,
            use_cached,
        }
    }

    /// Classify a single result's exclusion reason.
    ///
    /// Checks in priority order: dead code > coverage(off) > Makefile pattern.
    /// With cached data, this is pure HashSet lookups — no file I/O.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn classify(
        &mut self,
        result: &QueryResult,
        project_path: &Path,
    ) -> CoverageExclusion {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        // 1. Dead code check (function-level, highest signal)
        let dead_key = format!("{}::{}", result.file_path, result.function_name);
        if self.dead_functions.contains(&dead_key) {
            return CoverageExclusion::DeadCode;
        }

        // 2. Module-level coverage(off) check
        if self.is_coverage_off_file(&result.file_path, project_path) {
            return CoverageExclusion::CoverageOff;
        }

        // 3. Makefile COVERAGE_EXCLUDE pattern
        if let Some(ref re) = self.makefile_regex {
            if re.is_match(&result.file_path) {
                return CoverageExclusion::MakefileExcluded;
            }
        }

        CoverageExclusion::None
    }

    /// Check if a file has module-level `cfg_attr(coverage_nightly, coverage(off))`.
    ///
    /// With cached data (from index build), this is a pure HashSet lookup.
    /// Falls back to lazy file I/O only when no cached data is available.
    fn is_coverage_off_file(&mut self, file_path: &str, project_path: &Path) -> bool {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        if self.coverage_off_files.contains(file_path) {
            return true;
        }
        // If we have cached data, trust it — no need for file I/O fallback
        if self.use_cached {
            return false;
        }
        // Negative cache: already checked this file and it didn't have coverage(off)
        if self.checked_files.contains(file_path) {
            return false;
        }

        self.checked_files.insert(file_path.to_string());

        let full_path = project_path.join(file_path);
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            // Check first 50 lines for module-level coverage(off)
            let has_coverage_off = content.lines().take(50).any(|line| {
                let trimmed = line.trim();
                trimmed.contains("cfg_attr(coverage_nightly, coverage(off))")
                    || trimmed.contains("cfg_attr(coverage_nightly,coverage(off))")
            });
            if has_coverage_off {
                self.coverage_off_files.insert(file_path.to_string());
                return true;
            }
        }

        false
    }
}

/// Classify coverage exclusions for a batch of results.
///
/// When `cached_coverage_off` is provided (from index build), coverage(off)
/// detection is O(1) HashSet lookup with zero file I/O.
/// Mutates results in-place, setting `coverage_exclusion` and `coverage_excluded`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn classify_exclusions(
    results: &mut [QueryResult],
    project_path: &Path,
    cached_coverage_off: Option<&HashSet<String>>,
) {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut ctx = ExclusionContext::build(project_path, cached_coverage_off);
    for result in results.iter_mut() {
        let exclusion = ctx.classify(result, project_path);
        result.coverage_excluded = !exclusion.is_none();
        result.coverage_exclusion = exclusion;
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Parse the `COVERAGE_EXCLUDE` regex from the project Makefile.
///
/// Looks for `--ignore-filename-regex='...'` pattern and extracts the
/// inner regex, converting it from a filename regex to a path-matching regex.
fn parse_makefile_coverage_exclude(project_path: &Path) -> Option<regex::Regex> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let makefile_path = project_path.join("Makefile");
    let content = std::fs::read_to_string(makefile_path).ok()?;

    for line in content.lines() {
        if !line.contains("COVERAGE_EXCLUDE") || !line.contains("--ignore-filename-regex") {
            continue;
        }
        // Extract regex between single quotes after --ignore-filename-regex=
        if let Some(start) = line.find("--ignore-filename-regex='") {
            let after = line
                .get(start + "--ignore-filename-regex='".len()..)
                .unwrap_or_default();
            if let Some(end) = after.find('\'') {
                let raw_pattern = after.get(..end).unwrap_or_default();
                // Normalize escaping: Makefile uses `\\.` (backslash-backslash-dot) which
                // cargo-llvm-cov interprets as literal dot. But Rust regex sees `\\` as
                // literal backslash + `.` as any char. Replace `\\.` with `\.` so Rust
                // regex correctly matches literal dots in file paths.
                let pattern = raw_pattern.replace("\\\\.", "\\.");
                return regex::Regex::new(&pattern).ok();
            }
        }
    }
    None
}

/// Extract dead item keys from a single file entry in the dead-code cache.
fn collect_dead_items(file_entry: &serde_json::Value, dead: &mut HashSet<String>) {
    debug_assert!(true, "contract: collect_dead_items");
    let file_path = match file_entry.get("file_path").and_then(|p| p.as_str()) {
        Some(p) => p,
        None => return,
    };
    let items = file_entry.get("dead_items").and_then(|d| d.as_array());
    for item in items.into_iter().flatten() {
        if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
            dead.insert(format!("{}::{}", file_path, name));
        }
    }
}

/// Load dead function names from `.pmat/dead-code-cache.json`.
///
/// Returns a set of "file_path::function_name" keys for O(1) lookup.
fn load_dead_code_functions(project_path: &Path) -> HashSet<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cache_path = project_path.join(".pmat/dead-code-cache.json");
    let mut dead = HashSet::new();

    let data = match std::fs::read_to_string(cache_path) {
        Ok(d) => d,
        Err(_) => return dead,
    };
    let value: serde_json::Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return dead,
    };

    let files = value
        .get("report")
        .and_then(|r| r.get("files_with_dead_code"))
        .and_then(|f| f.as_array());

    for file_entry in files.into_iter().flatten() {
        collect_dead_items(file_entry, &mut dead);
    }

    dead
}
