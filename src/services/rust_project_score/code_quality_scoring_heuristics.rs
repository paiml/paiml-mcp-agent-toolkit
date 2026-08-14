// Code quality scoring: heuristic checks (complexity, unsafe, dead code)
// Cache-aware: uses FileCache if available, falls back to filesystem

/// What the Complexity check measured across a project's `src/` tree.
///
/// Every field is a count of *functions*, measured by the AST visitor, so the
/// figures here are the same ones `pmat analyze complexity` and
/// `quality-gate --checks complexity` report for the same tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct CyclomaticProfile {
    /// Functions the parser could measure. Zero means nothing was measured.
    functions: usize,
    /// Functions above `ComplexityThresholds::cyclomatic_error` (20).
    over_error: usize,
    /// Highest cyclomatic complexity of any measured function.
    max: u32,
}

impl CyclomaticProfile {
    fn merge(mut self, other: Self) -> Self {
        self.functions += other.functions;
        self.over_error += other.over_error;
        self.max = self.max.max(other.max);
        self
    }
}

/// Cyclomatic complexity of every function in one Rust source file.
///
/// #937: the Complexity check used to count *lines indented more than 40
/// characters* and call the result complexity. It measured no decision point at
/// all, so the ranking it produced was inverted: a crate whose only function has
/// cyclomatic complexity 241 scored Code Quality 14.0/14.0 (100%) because its
/// lines are short, while a three-line crate of cyclomatic complexity 1 scored
/// 13.0/14.0 for one wide line — and the same binary's
/// `quality-gate --checks complexity` blocked the first crate with "Cyclomatic
/// complexity of 241 exceeds maximum allowed complexity of 30". The check now
/// routes through [`measure_block`], the visitor `pmat analyze complexity`,
/// `quality-gate` and `pmat context` all share, so those two surfaces cannot
/// disagree about the same function again.
///
/// `None` when the text is not standalone Rust the parser can measure — that is
/// "not measured", not "no complexity", and the caller drops the check's points
/// out of the denominator rather than awarding them.
fn cyclomatic_of_content(content: &str) -> Option<CyclomaticProfile> {
    use crate::services::accurate_complexity_analyzer::{collect_functions, measure_block};

    let ast = syn::parse_file(content).ok()?;
    let error_threshold =
        u32::from(crate::services::complexity::ComplexityThresholds::default().cyclomatic_error);

    let mut profile = CyclomaticProfile::default();
    for func in collect_functions(&ast.items) {
        let cyclomatic = measure_block(&func.name, func.block).cyclomatic;
        profile.functions += 1;
        profile.over_error += usize::from(cyclomatic > error_threshold);
        profile.max = profile.max.max(cyclomatic);
    }
    Some(profile)
}

/// Score the Complexity check (3pts) from measured cyclomatic complexity.
///
/// The thresholds are [`ComplexityThresholds`]'s own defaults — the ones
/// `analyze complexity` flags violations with — so there is one answer in the
/// binary to "is this function too complex", not two. The tiers are a gradient
/// an agent can climb: fix the single worst function to leave 0.0, hold every
/// function inside the error threshold for 2.0, inside the warn threshold for
/// full marks.
fn score_from_cyclomatic(profile: CyclomaticProfile) -> f64 {
    let thresholds = crate::services::complexity::ComplexityThresholds::default();
    let warn = u32::from(thresholds.cyclomatic_warn);
    let error = u32::from(thresholds.cyclomatic_error);

    if profile.functions == 0 {
        return 0.0;
    }
    if profile.max <= warn {
        return 3.0;
    }
    if profile.over_error == 0 {
        return 2.0;
    }
    let violation_ratio = profile.over_error as f64 / profile.functions as f64;
    if violation_ratio <= 0.01 && profile.max <= error * 2 {
        return 1.0;
    }
    0.0
}

/// Every `.rs` file under `dir`, recursively.
fn collect_rs_paths(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    let mut push = |p: std::path::PathBuf| paths.push(p);
    collect_rs_paths_into(dir, &mut push);
    paths
}

fn collect_rs_paths_into(dir: &Path, f: &mut impl FnMut(std::path::PathBuf)) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_paths_into(&path, f);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                f(path);
            }
        }
    }
}

/// Returns true if the trimmed line mentions "unsafe" in a non-code context
/// (comments, string literals, variable bindings, or string-matching expressions).
fn is_non_code_unsafe(trimmed: &str) -> bool {
    const SKIP_PREFIXES: &[&str] = &["//", "*", "\"", "r#", "r\"", "let "];
    const SKIP_CONTAINS: &[&str] = &[
        ".contains(\"unsafe",
        ".starts_with(\"unsafe",
        "\"unsafe {\"",
        "\"unsafe{\"",
    ];
    SKIP_PREFIXES.iter().any(|p| trimmed.starts_with(p))
        || SKIP_CONTAINS.iter().any(|s| trimmed.contains(s))
}

/// Returns true if the trimmed line introduces an `unsafe` block.
fn is_unsafe_block(trimmed: &str) -> bool {
    const PATTERNS: &[&str] = &["unsafe {", "unsafe{", "= unsafe {", "} unsafe {"];
    PATTERNS.iter().any(|p| {
        trimmed.starts_with(p) || trimmed.contains(p)
    })
}

/// Returns true if any line in the slice contains a SAFETY documentation comment.
fn has_safety_comment(lines: &[&str]) -> bool {
    lines.iter().any(|l| l.contains("SAFETY:") || l.contains("Safety:"))
}

fn analyze_unsafe_in_content(content: &str) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| !is_non_code_unsafe(line.trim()))
        .filter(|(_, line)| is_unsafe_block(line.trim()))
        .fold((0, 0), |(blocks, docs), (i, _)| {
            let start = i.saturating_sub(10);
            let is_documented = has_safety_comment(&lines[start..=i]);
            (blocks + 1, docs + usize::from(is_documented))
        })
}

fn count_dead_code_attrs(content: &str) -> usize {
    // Build patterns at runtime to avoid self-detection when scoring our own codebase
    let outer = format!("#[allow({})]", "dead_code");
    let inner = format!("#![allow({})]", "dead_code");
    content.matches(outer.as_str()).count() + content.matches(inner.as_str()).count()
}

fn score_from_unsafe(unsafe_blocks: usize, documented: usize) -> f64 {
    if unsafe_blocks == 0 {
        return 9.0;
    }
    let doc_ratio = documented as f64 / unsafe_blocks as f64;
    if doc_ratio >= 0.9 { 9.0 }
    else if doc_ratio >= 0.7 { 7.0 }
    else if doc_ratio >= 0.5 { 5.0 }
    else if doc_ratio >= 0.3 { 3.0 }
    else { 1.0 }
}

fn for_each_rs_content_cached(cache: &FileCache, src_path: &Path, mut f: impl FnMut(&str)) {
    for (_path, content) in cache.get_rust_files_in_dir(src_path) {
        f(content);
    }
}

fn for_each_rs_content_fs(src_path: &Path, f: &mut impl FnMut(&str)) {
    // #237 bug 3: Must be recursive — read_dir is non-recursive and missed 98%+ of code
    if let Ok(entries) = std::fs::read_dir(src_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                for_each_rs_content_fs(&path, f);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    f(&content);
                }
            }
        }
    }
}

impl CodeQualityScorer {
    /// Measure cyclomatic complexity across the project's `src/` tree.
    ///
    /// `None` means *nothing was measured* — no `src/`, or not one file in it
    /// parsed as Rust. There is no score for that, so the check's points leave
    /// the denominator (see `score_internal`); the previous code returned a
    /// full 3.0 for a project with no source at all, which is absence rendered
    /// as success.
    fn measure_cyclomatic(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> Option<CyclomaticProfile> {
        use rayon::prelude::*;

        let src_path = project_path.join("src");
        if !src_path.exists() {
            return None;
        }

        // Parsing is the expensive part of the whole scorer, and every file is
        // independent, so it is measured in parallel.
        let profile = if let Some(cache) = cache {
            cache
                .get_rust_files_in_dir(&src_path)
                .par_iter()
                .filter_map(|(_path, content)| cyclomatic_of_content(content))
                .reduce(CyclomaticProfile::default, CyclomaticProfile::merge)
        } else {
            collect_rs_paths(&src_path)
                .par_iter()
                .filter_map(|path| std::fs::read_to_string(path).ok())
                .filter_map(|content| cyclomatic_of_content(&content))
                .reduce(CyclomaticProfile::default, CyclomaticProfile::merge)
        };

        (profile.functions > 0).then_some(profile)
    }

    fn score_unsafe(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(9.0);
        }

        let mut total_unsafe = 0;
        let mut total_documented = 0;
        let mut accumulate = |content: &str| {
            let (ub, doc) = analyze_unsafe_in_content(content);
            total_unsafe += ub;
            total_documented += doc;
        };

        if let Some(cache) = cache {
            for_each_rs_content_cached(cache, &src_path, accumulate);
        } else {
            for_each_rs_content_fs(&src_path, &mut accumulate);
        }

        Ok(score_from_unsafe(total_unsafe, total_documented))
    }

    fn score_dead_code(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(2.0);
        }

        let mut dead_code_count = 0;
        let mut accumulate = |content: &str| {
            dead_code_count += count_dead_code_attrs(content);
        };

        if let Some(cache) = cache {
            for_each_rs_content_cached(cache, &src_path, accumulate);
        } else {
            for_each_rs_content_fs(&src_path, &mut accumulate);
        }

        if dead_code_count == 0 { Ok(2.0) }
        else if dead_code_count <= 3 { Ok(1.0) }
        else { Ok(0.0) }
    }
}
