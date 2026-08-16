/// Extract contract metadata from context around the function definition.
///
/// Scans the 5 lines BEFORE `start_line` in the full file content for
/// `#[provable_contracts_macros::contract("yaml", equation = "eq")]`.
/// O(1) per function — no file I/O, just line indexing of already-loaded content.
fn extract_contract_metadata_from_context(
    full_content: &str,
    start_line: usize,
) -> (Option<String>, Option<String>) {
    let lines: Vec<&str> = full_content.lines().collect();
    // Scan 5 lines before the function definition (attributes are above)
    let scan_start = start_line.saturating_sub(5).max(1);
    for line_num in scan_start..start_line {
        if line_num == 0 || line_num > lines.len() {
            continue;
        }
        let trimmed = lines[line_num - 1].trim(); // lines are 0-indexed, start_line is 1-indexed
        if trimmed.contains("contract(") && trimmed.contains("equation") {
            // Extract equation name: equation = "name"
            if let Some(eq_start) = trimmed.find("equation") {
                let after_eq = &trimmed[eq_start..];
                if let Some(q1) = after_eq.find('"') {
                    let after_q1 = &after_eq[q1 + 1..];
                    if let Some(q2) = after_q1.find('"') {
                        let equation = after_q1[..q2].to_string();
                        return (Some("L2".to_string()), Some(equation));
                    }
                }
            }
            return (Some("L2".to_string()), None);
        }
    }
    (None, None)
}

/// Check if directory should be ignored
pub(super) fn is_ignored_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    matches!(
        name,
        "target"
            | "node_modules"
            | ".git"
            | ".pmat"
            | "__pycache__"
            | "venv"
            | ".venv"
            | "dist"
            | "build"
            | ".next"
            | ".cache"
            | "vendor"
            | "third_party"
            | "third-party"
            | "external"
            | "deps"
            | "book"
            | "theme"
            | "fixtures"
            | ".cargo"
    )
}

/// Detect language from file extension
pub(super) fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" | "js" | "jsx" => Some(Language::TypeScript),
        "py" => Some(Language::Python),
        "c" => Some(Language::C),
        "h" => Some(Language::C), // Default; classify_header() upgrades to Cpp with content
        "cpp" | "cc" | "cxx" | "hpp" | "cu" | "cuh" => Some(Language::Cpp),
        "go" => Some(Language::Go),
        "lua" => Some(Language::Lua),
        "ptx" => Some(Language::Ptx),
        _ => None,
    }
}

/// Classify a .h header as C or C++ based on content heuristics
pub(super) fn classify_header_language(content: &str) -> Language {
    // C++ indicators in non-comment context
    const CPP_INDICATORS: &[&str] = &[
        "extern \"C\"",
        "class ",
        "namespace ",
        "template<",
        "template <",
        "virtual ",
        "constexpr ",
        "nullptr",
        "std::",
        "public:",
        "private:",
        "protected:",
    ];
    if CPP_INDICATORS.iter().any(|kw| content.contains(kw)) {
        return Language::Cpp;
    }
    Language::C
}

/// Extract quality metrics from a code chunk
#[allow(clippy::cast_possible_truncation)]
pub(super) fn extract_quality_metrics(chunk: &CodeChunk, _full_content: &str) -> QualityMetrics {
    let loc = chunk.content.lines().count() as u32;

    // Count control flow complexity (simple heuristic)
    let mut complexity = count_complexity(&chunk.content);

    // Add C++/CUDA-specific complexity penalties (Phase 4 + Phase 7)
    let lang = chunk.language.as_str();
    if lang == "cpp" || lang == "c" || lang == "cuda" {
        complexity += cpp_complexity_penalty(&chunk.content);
    }

    // Count SATD markers
    let satd_count = count_satd_markers(&chunk.content);

    // Estimate Big-O from control flow
    let big_o = estimate_big_o(&chunk.content);

    // Exempt enums/structs/traits from LOC penalty — they're declarations, not logic
    use crate::services::semantic::ChunkType;
    let effective_loc = match chunk.chunk_type {
        ChunkType::Enum | ChunkType::Struct | ChunkType::Trait | ChunkType::TypeAlias => 0,
        _ => loc,
    };
    let tdg_score = calculate_simple_tdg(complexity, satd_count, effective_loc);
    let tdg_grade = score_to_grade(tdg_score);

    // Extract contract annotation from the lines preceding the function in full file content
    let (contract_level, contract_equation) =
        extract_contract_metadata_from_context(_full_content, chunk.start_line);

    QualityMetrics {
        tdg_score,
        tdg_grade,
        complexity,
        cognitive_complexity: complexity, // Simplified: use same as cyclomatic
        big_o,
        satd_count,
        loc,
        commit_count: 0,  // Populated later by churn enrichment
        churn_score: 0.0, // Populated later by churn enrichment
        contract_level,
        contract_equation,
    }
}

/// Count cyclomatic complexity (simplified)
pub(super) fn count_complexity(source: &str) -> u32 {
    let mut complexity = 1u32; // Base complexity

    // Count decision points
    for line in source.lines() {
        let trimmed = line.trim();

        // Control flow keywords (Rust + C/C++)
        if trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.starts_with("else if ")
            || trimmed.starts_with("} else if ")
            || trimmed.contains(" if ")
            || trimmed.starts_with("match ")
            || trimmed.starts_with("switch ")
            || trimmed.starts_with("switch(")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("while(")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.starts_with("loop ")
            || trimmed.starts_with("do {")
            || trimmed.starts_with("do{")
            || trimmed.starts_with("catch ")
            || trimmed.starts_with("catch(")
            || trimmed.contains("&&")
            || trimmed.contains("||")
            || trimmed.contains("? ")
        {
            complexity += 1;
        }

        // C++ case labels: "case FOO:"
        if trimmed.starts_with("case ") && trimmed.contains(':') && !trimmed.starts_with("//") {
            complexity += 1;
        }

        // Match arms (Rust)
        if trimmed.contains("=>") && !trimmed.starts_with("//") {
            complexity += 1;
        }
    }

    complexity
}

/// C++/CUDA-specific complexity penalties (Phase 4 + Phase 7 of cpp-pmat-query spec).
///
/// Adds penalties for patterns that increase cognitive complexity beyond
/// standard control flow: preprocessor conditionals, macro-heavy code,
/// template nesting, SFINAE, and CUDA synchronization primitives.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn cpp_complexity_penalty(source: &str) -> u32 {
    let mut penalty = 0u32;

    // Preprocessor conditionals: +1 per nesting level
    let mut ifdef_depth = 0u32;
    let mut macro_call_count = 0u32;

    for line in source.lines() {
        let trimmed = line.trim();

        // Preprocessor conditional nesting
        if trimmed.starts_with("#if")
            || trimmed.starts_with("#ifdef")
            || trimmed.starts_with("#ifndef")
        {
            ifdef_depth += 1;
            penalty += ifdef_depth; // +1 per nesting level
        } else if trimmed.starts_with("#endif") {
            ifdef_depth = ifdef_depth.saturating_sub(1);
        }

        // Count macro calls (UPPER_CASE identifiers with parens, common C/C++ convention)
        if trimmed.contains("GGML_")
            || trimmed.contains("TORCH_")
            || trimmed.contains("AT_")
            || trimmed.contains("CUDA_")
            || trimmed.contains("CHECK_")
        {
            macro_call_count += 1;
        }
    }

    // Macro-heavy function: +3 for >5 macro calls
    if macro_call_count > 5 {
        penalty += 3;
    }

    // SFINAE / concepts: +3
    if source.contains("enable_if") || source.contains("requires ") || source.contains("SFINAE") {
        penalty += 3;
    }

    // Template nesting: +2 per nested template<>
    let template_depth = source.matches("template<").count() + source.matches("template <").count();
    if template_depth > 1 {
        penalty += (template_depth as u32 - 1) * 2;
    }

    // const_cast / reinterpret_cast: +2 each
    if source.contains("const_cast<") || source.contains("reinterpret_cast<") {
        penalty += 2;
    }

    // CUDA kernel penalties (Phase 7)
    // __shared__ memory: +2 (synchronization complexity)
    if source.contains("__shared__") {
        penalty += 2;
    }

    // __syncthreads(): +3 (barrier coordination)
    if source.contains("__syncthreads()") {
        penalty += 3;
    }

    // Warp primitives: +2
    if source.contains("__shfl_")
        || source.contains("__ballot_")
        || source.contains("__any_sync")
        || source.contains("__all_sync")
    {
        penalty += 2;
    }

    // Thread divergence in kernel (if inside __global__ function): +2
    if source.contains("__global__") && (source.contains("if (") || source.contains("if(")) {
        penalty += 2;
    }

    penalty
}

/// Count SATD markers in implementation comments only.
/// Excludes doc comments (/// and //!), string literals, and identifiers.
/// Only counts markers that represent genuine self-admitted technical debt.
/// SATD markers in one chunk of source, counted by the SAME detector
/// `pmat analyze satd` uses.
///
/// This was a hand-rolled scanner: it counted `TODO`/`FIXME`/`HACK`/`OPTIMIZE`
/// as raw substrings anywhere in a comment, with its own block-comment and
/// raw-string tracking, and it skipped doc comments on the reasoning that they
/// "describe behavior, not debt".
///
/// Every one of those rules disagreed with the real detector, in both
/// directions at once:
///
/// | input | this function | `analyze satd` |
/// |---|---|---|
/// | `/// TODO: implement X` | 0 (doc comments skipped) | 1 |
/// | `// the TODO list is empty` | 1 (substring anywhere) | 0 (marker must lead) |
///
/// So the agent-context index and the CLI reported different debt for the same
/// file, which is exactly the defect #998 fixed between the CLI and MCP SATD
/// surfaces, and #831 fixed for `five-whys` — where the same style of raw scan
/// reported 808 markers against `analyze satd`'s 39 for one repo, a 20x
/// disagreement presented as measured evidence.
///
/// Rather than fix a third copy of the rule to match, the rule now has one
/// implementation and this delegates to it.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn count_satd_markers(source: &str) -> u32 {
    use crate::services::satd_detector::SATDDetector;
    // A `.rs` path so the scanner selects Rust comment syntax. The path is not
    // opened — `extract_from_content` reads the string it is handed.
    SATDDetector::new()
        .extract_from_content(source, std::path::Path::new("chunk.rs"))
        .map_or(0, |debts| debts.len() as u32)
}


/// Estimate Big-O from control flow
pub(super) fn estimate_big_o(source: &str) -> String {
    let mut current_nesting = 0;
    let mut max_nesting = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("loop ")
        {
            current_nesting += 1;
            max_nesting = max_nesting.max(current_nesting);
        }

        if trimmed == "}" && current_nesting > 0 {
            current_nesting -= 1;
        }
    }

    match max_nesting {
        0 => "O(1)".to_string(),
        1 => "O(n)".to_string(),
        2 => "O(n^2)".to_string(),
        3 => "O(n^3)".to_string(),
        n => format!("O(n^{n})"),
    }
}

/// Maximum points the cyclomatic-complexity penalty can remove.
const COMPLEXITY_PENALTY_CAP: f32 = 50.0;
/// Points removed per unit of cyclomatic complexity above 1 (no branches).
const COMPLEXITY_PENALTY_PER_BRANCH: f32 = 1.5;
/// Maximum points the SATD penalty can remove.
const SATD_PENALTY_CAP: f32 = 20.0;
/// Points removed per SATD marker past the first two.
const SATD_PENALTY_PER_MARKER: f32 = 5.0;
/// Number of SATD markers that cost nothing (see `calculate_simple_tdg`).
const SATD_FREE_MARKERS: u32 = 2;
/// Maximum points the function-length penalty can remove.
const LOC_PENALTY_CAP: f32 = 30.0;
/// Function length at which the length penalty starts.
const LOC_PENALTY_THRESHOLD: u32 = 50;
/// Lines per point of length penalty past the threshold.
const LOC_LINES_PER_POINT: f32 = 15.0;

/// Per-function TDG score, on the SAME scale `pmat tdg` reports:
/// **0-100, higher is better**.
///
/// Before v3.30.0 this returned a 0-10 LOWER-is-better *debt* number and had a
/// private five-letter `score_to_grade` beside it, so `pmat query` / `pmat sql`
/// and `pmat tdg` disagreed about the direction of the axis AND about what the
/// grade letters meant: `pmat sql grade-dist` reported `A=21748, B=5` over the
/// whole repository (99.98% A, i.e. no resolution at all) while `pmat tdg` on
/// the same tree reported `99.77 / A+`. A number that is "good" when small in
/// one command and "good" when large in another is not a measurement.
///
/// The score is now `100 - penalties`, with the penalty budget mirroring the
/// weighting `pmat tdg` uses (structure dominates, then size, then
/// self-admitted debt), and grading goes through `crate::tdg::Grade` — the one
/// score→grade mapping in the codebase.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn calculate_simple_tdg(complexity: u32, satd_count: u32, loc: u32) -> f32 {
    let mut score = 100.0f32;

    // Structural complexity (0-50 points). 1.5 points per branch past the
    // first, so the pre-commit CC<=30 gate lands at 56.5 (C-) and CC>=35
    // exhausts the budget. CC=1 (no branches) costs nothing.
    score -= (complexity.saturating_sub(1) as f32 * COMPLEXITY_PENALTY_PER_BRANCH)
        .min(COMPLEXITY_PENALTY_CAP);

    // Self-admitted technical debt (0-20 points). The first two markers are
    // free: many functions reference TODO/FIXME descriptively (detector code,
    // marker enums) and would otherwise be penalised for naming the thing.
    score -= (satd_count.saturating_sub(SATD_FREE_MARKERS) as f32 * SATD_PENALTY_PER_MARKER)
        .min(SATD_PENALTY_CAP);

    // Size (0-30 points), starting at 50 lines: one point per 15 lines, so a
    // 200-line function loses 10 and a 500-line one exhausts the budget.
    if loc > LOC_PENALTY_THRESHOLD {
        score -= ((loc - LOC_PENALTY_THRESHOLD) as f32 / LOC_LINES_PER_POINT).min(LOC_PENALTY_CAP);
    }

    // GH-272: cyclomatic complexity 1 means no branches — the simplest
    // possible control flow. Such functions (large data tables, long trivial
    // constructors) are floored at grade A regardless of the size/SATD
    // penalties. On the old lower-is-better scale this read `score.min(1.99)`;
    // the polarity flip makes it a floor, not a ceiling.
    if complexity <= 1 {
        score = score.max(GH272_TRIVIAL_FLOOR);
    }

    score.clamp(0.0, 100.0)
}

/// Score floor for zero-branch definitions (GH-272). Equal to the `A` band
/// floor in `crate::tdg::Grade`, so the rule reads "at least an A".
const GH272_TRIVIAL_FLOOR: f32 = 90.0;

/// Convert a 0-100 TDG score to a letter grade.
///
/// Delegates to `crate::tdg::Grade::from_score` — the ONE score→grade mapping.
/// This used to be a second, private five-band table on a different scale,
/// which is why `pmat query` could call a function "A" that `pmat tdg` graded
/// `C+`.
pub(super) fn score_to_grade(score: f32) -> String {
    crate::tdg::Grade::from_score(score).to_string()
}

/// Extract doc comment from source
/// Classify a line above a function definition for doc comment extraction.
enum DocLineKind<'a> {
    DocComment(&'a str),
    BlockCommentStart,
    BlockCommentBody(&'a str),
    SkipLine, // empty, attribute, annotation
    Other,
}

fn classify_doc_line(line: &str) -> DocLineKind<'_> {
    if line.starts_with("///") || line.starts_with("//!") {
        DocLineKind::DocComment(
            line.trim_start_matches("///")
                .trim_start_matches("//!")
                .trim(),
        )
    } else if line.starts_with("/**") || line.starts_with("/*") {
        DocLineKind::BlockCommentStart
    } else if line.starts_with('*') {
        DocLineKind::BlockCommentBody(line.trim_start_matches('*').trim())
    } else if line.is_empty() || line.starts_with("#[") || line.starts_with('@') {
        DocLineKind::SkipLine
    } else {
        DocLineKind::Other
    }
}

pub(super) fn extract_doc_comment(content: &str, start_line: usize) -> Option<String> {
    if start_line <= 1 {
        return None;
    }

    // Find byte offset of start_line by counting newlines (0 alloc)
    let bytes = content.as_bytes();
    let mut line_num = 1usize;
    let mut def_line_start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        if line_num >= start_line {
            def_line_start = i;
            break;
        }
        if b == b'\n' {
            line_num += 1;
            if line_num >= start_line {
                def_line_start = i + 1;
                break;
            }
        }
    }
    if line_num < start_line {
        return None;
    }

    // Scan backward line-by-line from (start_line - 1) without allocating a line-offset Vec
    let mut doc_lines = Vec::new();
    let mut end = def_line_start; // exclusive end of current line (before its \n)
                                  // Skip trailing \n before def line
    if end > 0 && bytes[end.saturating_sub(1)] == b'\n' {
        end = end.saturating_sub(1);
    }
    let mut pos = end;
    loop {
        // Find start of this line
        let line_start = if pos == 0 {
            0
        } else {
            match content[..pos].rfind('\n') {
                Some(nl) => nl + 1,
                None => 0,
            }
        };
        let line = content.get(line_start..pos).unwrap_or("").trim();
        match classify_doc_line(line) {
            DocLineKind::DocComment(text) => doc_lines.push(text),
            DocLineKind::BlockCommentBody(text) => doc_lines.push(text),
            DocLineKind::BlockCommentStart | DocLineKind::Other => break,
            DocLineKind::SkipLine => {
                if line_start == 0 {
                    break;
                }
                pos = line_start.saturating_sub(1);
                continue;
            }
        }
        if line_start == 0 {
            break;
        }
        pos = line_start.saturating_sub(1);
    }

    if doc_lines.is_empty() {
        return None;
    }
    doc_lines.reverse();
    Some(doc_lines.join(" "))
}

#[cfg(test)]
mod quality_metrics_tests {
    use super::*;
    use std::path::PathBuf;

    // ── is_ignored_dir ──────────────────────────────────────────────────────

    #[test]
    fn test_is_ignored_dir_target() {
        assert!(is_ignored_dir(&PathBuf::from("/foo/target")));
    }

    #[test]
    fn test_is_ignored_dir_node_modules() {
        assert!(is_ignored_dir(&PathBuf::from("node_modules")));
    }

    #[test]
    fn test_is_ignored_dir_python_cache() {
        assert!(is_ignored_dir(&PathBuf::from("__pycache__")));
        assert!(is_ignored_dir(&PathBuf::from(".venv")));
        assert!(is_ignored_dir(&PathBuf::from("venv")));
    }

    #[test]
    fn test_is_ignored_dir_build_artifacts() {
        assert!(is_ignored_dir(&PathBuf::from("dist")));
        assert!(is_ignored_dir(&PathBuf::from("build")));
        assert!(is_ignored_dir(&PathBuf::from(".next")));
        assert!(is_ignored_dir(&PathBuf::from(".cache")));
    }

    #[test]
    fn test_is_ignored_dir_vendor_third_party() {
        assert!(is_ignored_dir(&PathBuf::from("vendor")));
        assert!(is_ignored_dir(&PathBuf::from("third_party")));
        assert!(is_ignored_dir(&PathBuf::from("third-party")));
        assert!(is_ignored_dir(&PathBuf::from("external")));
    }

    #[test]
    fn test_is_ignored_dir_normal_dirs() {
        assert!(!is_ignored_dir(&PathBuf::from("src")));
        assert!(!is_ignored_dir(&PathBuf::from("docs")));
        assert!(!is_ignored_dir(&PathBuf::from("lib")));
    }

    // ── detect_language ─────────────────────────────────────────────────────

    #[test]
    fn test_detect_language_rust() {
        assert!(matches!(
            detect_language(&PathBuf::from("foo.rs")),
            Some(Language::Rust)
        ));
    }

    #[test]
    fn test_detect_language_typescript_variants() {
        for ext in &["ts", "tsx", "js", "jsx"] {
            let p = PathBuf::from(format!("foo.{ext}"));
            assert!(matches!(detect_language(&p), Some(Language::TypeScript)));
        }
    }

    #[test]
    fn test_detect_language_python() {
        assert!(matches!(
            detect_language(&PathBuf::from("foo.py")),
            Some(Language::Python)
        ));
    }

    #[test]
    fn test_detect_language_c_and_h_default_c() {
        assert!(matches!(
            detect_language(&PathBuf::from("foo.c")),
            Some(Language::C)
        ));
        assert!(matches!(
            detect_language(&PathBuf::from("foo.h")),
            Some(Language::C)
        ));
    }

    #[test]
    fn test_detect_language_cpp_variants() {
        for ext in &["cpp", "cc", "cxx", "hpp", "cu", "cuh"] {
            let p = PathBuf::from(format!("foo.{ext}"));
            assert!(matches!(detect_language(&p), Some(Language::Cpp)));
        }
    }

    #[test]
    fn test_detect_language_other() {
        assert!(matches!(
            detect_language(&PathBuf::from("foo.go")),
            Some(Language::Go)
        ));
        assert!(matches!(
            detect_language(&PathBuf::from("foo.lua")),
            Some(Language::Lua)
        ));
        assert!(matches!(
            detect_language(&PathBuf::from("foo.ptx")),
            Some(Language::Ptx)
        ));
    }

    #[test]
    fn test_detect_language_unknown_extension_returns_none() {
        assert!(detect_language(&PathBuf::from("foo.txt")).is_none());
        assert!(detect_language(&PathBuf::from("Cargo.toml")).is_none());
        assert!(detect_language(&PathBuf::from("Makefile")).is_none());
    }

    // ── classify_header_language ────────────────────────────────────────────

    #[test]
    fn test_classify_header_language_no_indicators_is_c() {
        assert!(matches!(classify_header_language(""), Language::C));
        assert!(matches!(
            classify_header_language("int foo(int x);"),
            Language::C
        ));
    }

    #[test]
    fn test_classify_header_language_extern_c_is_cpp() {
        assert!(matches!(
            classify_header_language("extern \"C\" {}"),
            Language::Cpp
        ));
    }

    #[test]
    fn test_classify_header_language_class_keyword_is_cpp() {
        assert!(matches!(
            classify_header_language("class Foo { };"),
            Language::Cpp
        ));
    }

    #[test]
    fn test_classify_header_language_namespace_is_cpp() {
        assert!(matches!(
            classify_header_language("namespace foo { }"),
            Language::Cpp
        ));
    }

    #[test]
    fn test_classify_header_language_template_is_cpp() {
        assert!(matches!(
            classify_header_language("template<typename T>"),
            Language::Cpp
        ));
        assert!(matches!(
            classify_header_language("template <class T>"),
            Language::Cpp
        ));
    }

    #[test]
    fn test_classify_header_language_visibility_modifiers_are_cpp() {
        assert!(matches!(classify_header_language("public:"), Language::Cpp));
        assert!(matches!(
            classify_header_language("private:"),
            Language::Cpp
        ));
        assert!(matches!(
            classify_header_language("protected:"),
            Language::Cpp
        ));
    }

    // ── count_complexity ────────────────────────────────────────────────────

    #[test]
    fn test_count_complexity_empty_source_base_one() {
        assert_eq!(count_complexity(""), 1);
    }

    #[test]
    fn test_count_complexity_single_if() {
        let src = "if x { y }";
        assert_eq!(count_complexity(src), 2);
    }

    #[test]
    fn test_count_complexity_match_arms() {
        // match + 3 arms (=>) → 4 over the base 1 = 5
        let src = "match x {\n    1 => a,\n    2 => b,\n    _ => c,\n}";
        assert_eq!(count_complexity(src), 5);
    }

    #[test]
    fn test_count_complexity_loops_count() {
        let src = "for i in v { }\nwhile cond { }\nloop { break; }";
        // 3 loops + base 1 = 4
        assert_eq!(count_complexity(src), 4);
    }

    #[test]
    fn test_count_complexity_short_circuit_operators_per_line() {
        // Pinned behavior: predicate is OR-chained so a single line matching
        // ANY arm increments by 1 (not per operator). Putting && / || on
        // separate lines counts each.
        let src = "if cond_a {\n    a && b\n    a || b\n}";
        // line 1: "if cond_a {" matches `if ` → +1
        // line 2: "a && b" matches `&&` → +1
        // line 3: "a || b" matches `||` → +1
        // base 1 + 3 = 4
        assert_eq!(count_complexity(src), 4);
    }

    #[test]
    fn test_count_complexity_single_line_with_multiple_ops_only_counts_once() {
        // PIN: the OR-chained predicate matches once per line; multiple ops
        // on the same line don't sum.
        let src = "if a && b || c { }";
        // single line matching `if ` → +1; base 1 = 2
        assert_eq!(count_complexity(src), 2);
    }

    #[test]
    fn test_count_complexity_c_switch_case() {
        let src = "switch (x) {\ncase 1: foo;\ncase 2: bar;\n}";
        // 1 switch + 2 cases + base = 4
        assert_eq!(count_complexity(src), 4);
    }

    #[test]
    fn test_count_complexity_skips_comment_arrows_and_cases() {
        let src = "// case 1:\n// =>\nlet x = 1;";
        // Should NOT count the comment arrows/cases
        assert_eq!(count_complexity(src), 1);
    }

    // ── cpp_complexity_penalty ──────────────────────────────────────────────

    #[test]
    fn test_cpp_complexity_penalty_no_patterns_is_zero() {
        assert_eq!(cpp_complexity_penalty(""), 0);
        assert_eq!(cpp_complexity_penalty("int x = 1;"), 0);
    }

    #[test]
    fn test_cpp_complexity_penalty_ifdef_nesting() {
        let src = "#ifdef A\n#ifdef B\nfoo\n#endif\n#endif";
        // depth 1 + depth 2 = 3
        assert_eq!(cpp_complexity_penalty(src), 3);
    }

    #[test]
    fn test_cpp_complexity_penalty_macro_heavy_above_5_adds_3() {
        // PIN: macro_call_count increments per LINE that contains a macro
        // pattern, not per occurrence. 6 lines each containing GGML_ → +3.
        let src = "GGML_X();\nGGML_Y();\nGGML_Z();\nGGML_W();\nGGML_V();\nGGML_U();";
        assert_eq!(cpp_complexity_penalty(src), 3);
    }

    #[test]
    fn test_cpp_complexity_penalty_macros_on_single_line_only_counts_once() {
        // PIN: 6 macros on a single line → macro_call_count = 1 (not 6) →
        // below 5 threshold → no penalty
        let src = "GGML_X(); GGML_Y(); GGML_Z(); GGML_W(); GGML_V(); GGML_U();";
        assert_eq!(cpp_complexity_penalty(src), 0);
    }

    #[test]
    fn test_cpp_complexity_penalty_sfinae_adds_3() {
        assert_eq!(cpp_complexity_penalty("std::enable_if<>"), 3);
        assert_eq!(cpp_complexity_penalty("requires (T x)"), 3);
        assert_eq!(cpp_complexity_penalty("// SFINAE pattern"), 3);
    }

    #[test]
    fn test_cpp_complexity_penalty_template_nesting() {
        let src = "template<class A>\ntemplate<class B>\ntemplate<class C>";
        // 3 templates → (3-1)*2 = 4
        assert_eq!(cpp_complexity_penalty(src), 4);
    }

    #[test]
    fn test_cpp_complexity_penalty_unsafe_casts_add_2() {
        assert_eq!(cpp_complexity_penalty("const_cast<int>(x);"), 2);
        assert_eq!(cpp_complexity_penalty("reinterpret_cast<u8*>(p);"), 2);
    }

    #[test]
    fn test_cpp_complexity_penalty_cuda_shared_memory_adds_2() {
        assert_eq!(cpp_complexity_penalty("__shared__ int x[64];"), 2);
    }

    #[test]
    fn test_cpp_complexity_penalty_cuda_syncthreads_adds_3() {
        assert_eq!(cpp_complexity_penalty("__syncthreads();"), 3);
    }

    #[test]
    fn test_cpp_complexity_penalty_cuda_warp_primitives_add_2() {
        assert_eq!(cpp_complexity_penalty("__shfl_sync(...)"), 2);
        assert_eq!(cpp_complexity_penalty("__ballot_sync(...)"), 2);
        assert_eq!(cpp_complexity_penalty("__any_sync(...)"), 2);
        assert_eq!(cpp_complexity_penalty("__all_sync(...)"), 2);
    }

    #[test]
    fn test_cpp_complexity_penalty_global_kernel_with_branch_adds_2() {
        let src = "__global__ void k() { if (x) { } }";
        // template/macro none, but __global__ + if → +2
        assert_eq!(cpp_complexity_penalty(src), 2);
    }

    // ── count_satd_markers ──────────────────────────────────────────────────

    #[test]
    fn test_count_satd_markers_no_markers() {
        assert_eq!(count_satd_markers(""), 0);
        assert_eq!(count_satd_markers("// regular comment"), 0);
    }

    #[test]
    fn test_count_satd_markers_inline_todo() {
        assert_eq!(count_satd_markers("let x = 1; // TODO: rename"), 1);
    }

    /// One comment admitting debt is ONE debt item, not one per marker word.
    /// The old hand-rolled counter returned 2 here by counting substrings;
    /// `pmat analyze satd` reports 1 for the same line, and the whole point of
    /// delegating is that these two numbers agree.
    #[test]
    fn test_one_comment_is_one_debt_item_however_many_marker_words() {
        assert_eq!(count_satd_markers("// TODO: x and FIXME: y"), 1);
    }

    /// Doc comments are scanned, and judged by the same marker rule as any
    /// other comment: a marker counts, prose does not.
    ///
    /// This used to assert 0 for both lines on the grounds that "/// and //!
    /// are doc comments, not implementation comments". They still come back 0,
    /// but for a completely different and correct reason — neither line carries
    /// a marker in the canonical `MARKER:` form; they are sentences that happen
    /// to contain the word. A doc comment that DOES admit debt now counts.
    #[test]
    fn test_doc_comments_are_judged_by_the_same_marker_rule() {
        // Prose that merely contains the word: not debt.
        assert_eq!(count_satd_markers("/// TODO is in doc"), 0);
        assert_eq!(count_satd_markers("//! FIXME doc"), 0);
        // A real admission in the public API docs: debt.
        assert_eq!(count_satd_markers("/// TODO: implement the fast path"), 1);
        assert_eq!(count_satd_markers("//! FIXME: this module leaks"), 1);
    }

    #[test]
    fn test_count_satd_markers_block_comment() {
        let src = "/* TODO: something */";
        assert_eq!(count_satd_markers(src), 1);
    }

    #[test]
    fn test_count_satd_markers_in_string_literal_skipped() {
        // A `// FIXME` inside a string literal (odd quote count before //)
        // is not counted because count_markers_in_comment() bails on odd quotes.
        let src = "let s = \"hello // FIXME ignored\";";
        assert_eq!(count_satd_markers(src), 0);
    }

    /// A sentence that STARTS with a marker word is not an admission.
    ///
    /// This asserted 4 for `// TODO and FIXME and HACK and OPTIMIZE` by counting
    /// substrings. The shared detector requires the canonical `MARKER:` form,
    /// because #925 measured a 92% false-positive rate from exactly this kind of
    /// match — `TODO the same analysis reports under -p` and `WIP patterns -
    /// work in progress` were both reported as debt. Whitespace is deliberately
    /// not a separator.
    #[test]
    fn test_marker_word_without_punctuation_is_prose_not_debt() {
        assert_eq!(count_satd_markers("// TODO and FIXME and HACK and OPTIMIZE"), 0);
        // …and the canonical form still counts.
        assert_eq!(count_satd_markers("// TODO: and the rest"), 1);
    }

    // ── estimate_big_o ──────────────────────────────────────────────────────

    #[test]
    fn test_estimate_big_o_no_loops_is_o1() {
        assert_eq!(estimate_big_o("let x = 1;"), "O(1)");
        assert_eq!(estimate_big_o(""), "O(1)");
    }

    #[test]
    fn test_estimate_big_o_single_loop_is_on() {
        assert_eq!(estimate_big_o("for i in v { print(i); }\n}"), "O(n)");
    }

    #[test]
    fn test_estimate_big_o_nested_loops_is_n2() {
        let src = "for i in v {\nfor j in v {\nfoo();\n}\n}\n";
        assert_eq!(estimate_big_o(src), "O(n^2)");
    }

    #[test]
    fn test_estimate_big_o_triple_nested_is_n3() {
        let src = "for a in v {\nfor b in v {\nfor c in v {\nfoo();\n}\n}\n}\n";
        assert_eq!(estimate_big_o(src), "O(n^3)");
    }

    #[test]
    fn test_estimate_big_o_while_and_loop_count_too() {
        assert_eq!(estimate_big_o("while x {}\n}"), "O(n)");
        assert_eq!(estimate_big_o("loop {}\n}"), "O(n)");
    }

    // ── calculate_simple_tdg ────────────────────────────────────────────────

    #[test]
    fn test_calculate_simple_tdg_trivial_function_is_top_of_scale() {
        // CC=1, no SATD, 50 lines → nothing penalised → 100/100.
        let s = calculate_simple_tdg(1, 0, 50);
        assert!((s - 100.0).abs() < 1e-3, "expected 100.0, got {s}");
    }

    #[test]
    fn test_calculate_simple_tdg_high_complexity_lowers_score() {
        // Higher is better now: more branches must SUBTRACT.
        let simple = calculate_simple_tdg(5, 0, 0);
        let hairy = calculate_simple_tdg(40, 0, 0);
        assert!(
            hairy < simple,
            "CC=40 ({hairy}) must score worse than CC=5 ({simple})"
        );
    }

    #[test]
    fn test_calculate_simple_tdg_complexity_penalty_capped_at_50() {
        // CC=200 would remove 298.5 points unclamped; the budget is 50.
        let s = calculate_simple_tdg(200, 0, 0);
        assert!((s - 50.0).abs() < 1e-3, "expected 50.0, got {s}");
    }

    #[test]
    fn test_calculate_simple_tdg_satd_penalty_starts_at_3() {
        let s_no_satd = calculate_simple_tdg(2, 0, 0);
        let s_with_satd = calculate_simple_tdg(2, 3, 0);
        assert!(s_with_satd < s_no_satd);
        assert!((s_no_satd - s_with_satd - 5.0).abs() < 1e-3);
    }

    #[test]
    fn test_calculate_simple_tdg_loc_penalty_above_threshold() {
        let s_low = calculate_simple_tdg(2, 0, 50);
        let s_high = calculate_simple_tdg(2, 0, 400);
        assert!(s_high < s_low, "a 400-line function must score worse");
    }

    #[test]
    fn test_calculate_simple_tdg_never_leaves_0_100() {
        // Every penalty maxed out at once must stay inside the scale.
        let s = calculate_simple_tdg(500, 100, 5000);
        assert!((0.0..=100.0).contains(&s), "score {s} escaped 0-100");
    }

    #[test]
    fn test_calculate_simple_tdg_cc_one_floored_at_grade_a() {
        // GH-272: CC=1 with massive LOC + SATD is still at least an A.
        let s = calculate_simple_tdg(1, 100, 1000);
        assert!(s >= 90.0, "expected >= 90.0 (A floor), got {s}");
        assert_eq!(score_to_grade(s), "A");
    }

    /// R30: the index scale and polarity must be the ones `pmat tdg` uses.
    /// A known-bad function grades poorly, a known-good one well, and BOTH
    /// grades come out of `crate::tdg::Grade::from_score`.
    #[test]
    fn test_calculate_simple_tdg_agrees_with_pmat_tdg_scale() {
        // Known-good: a 12-line, 3-branch helper.
        let good = calculate_simple_tdg(3, 0, 12);
        // Known-bad: a 420-line, CC=45 god function with 8 SATD markers.
        let bad = calculate_simple_tdg(45, 8, 420);

        assert!(good > bad, "good {good} must outrank bad {bad}");
        assert!(good >= 90.0, "known-good scored {good}, expected >= 90");
        assert!(bad < 50.0, "known-bad scored {bad}, expected < 50 (F)");

        // Same mapping `pmat tdg` uses — not a private second table.
        assert_eq!(
            score_to_grade(good),
            crate::tdg::Grade::from_score(good).to_string()
        );
        assert_eq!(score_to_grade(bad), "F");
    }

    // ── score_to_grade ──────────────────────────────────────────────────────

    #[test]
    fn test_score_to_grade_matches_tdg_grade_bands() {
        // The band edges are crate::tdg::Grade's, not a local copy.
        for (score, expected) in [
            (100.0f32, "A+"),
            (95.0, "A+"),
            (94.9, "A"),
            (90.0, "A"),
            (85.0, "A-"),
            (80.0, "B+"),
            (75.0, "B"),
            (70.0, "B-"),
            (65.0, "C+"),
            (60.0, "C"),
            (55.0, "C-"),
            (50.0, "D"),
            (49.9, "F"),
            (0.0, "F"),
        ] {
            assert_eq!(score_to_grade(score), expected, "score {score}");
        }
    }

    #[test]
    fn test_score_to_grade_low_score_is_not_an_a() {
        // The inversion in one line: 0.12 used to be an A (best possible on
        // the old 0-10 debt scale). On the 0-100 scale it is an F.
        assert_eq!(score_to_grade(0.12), "F");
    }

    // ── extract_contract_metadata_from_context ──────────────────────────────

    #[test]
    fn test_extract_contract_metadata_finds_l2_with_equation() {
        let src = "use foo;\n#[provable_contracts_macros::contract(\"yaml\", equation = \"my_eq\")]\npub fn target() {}";
        // start_line = 3 (the `pub fn` line)
        let (level, eq) = extract_contract_metadata_from_context(src, 3);
        assert_eq!(level, Some("L2".to_string()));
        assert_eq!(eq, Some("my_eq".to_string()));
    }

    #[test]
    fn test_extract_contract_metadata_no_contract_returns_none() {
        let src = "use foo;\nfn target() {}";
        let (level, eq) = extract_contract_metadata_from_context(src, 2);
        assert!(level.is_none());
        assert!(eq.is_none());
    }

    #[test]
    fn test_extract_contract_metadata_contract_without_equation() {
        let src = "#[contract(\"y\")]\nfn t() {}";
        // no `equation` keyword → returns None for both
        let (level, eq) = extract_contract_metadata_from_context(src, 2);
        assert!(level.is_none());
        assert!(eq.is_none());
    }

    #[test]
    fn test_extract_contract_metadata_only_scans_5_lines_back() {
        let mut src = String::new();
        for _ in 0..10 {
            src.push_str("// fill\n");
        }
        src.push_str("#[contract(\"y\", equation = \"e\")]\n");
        for _ in 0..6 {
            src.push_str("// more fill\n");
        }
        src.push_str("fn t() {}\n");
        // contract is line 11; fn at line 18; gap > 5 → not found
        let (level, _) = extract_contract_metadata_from_context(&src, 18);
        assert!(level.is_none());
    }

    // ── classify_doc_line ───────────────────────────────────────────────────

    #[test]
    fn test_classify_doc_line_triple_slash() {
        match classify_doc_line("/// hello docs") {
            DocLineKind::DocComment(text) => assert_eq!(text, "hello docs"),
            _ => panic!("expected DocComment"),
        }
    }

    #[test]
    fn test_classify_doc_line_inner_doc() {
        match classify_doc_line("//! module docs") {
            DocLineKind::DocComment(text) => assert_eq!(text, "module docs"),
            _ => panic!("expected DocComment"),
        }
    }

    #[test]
    fn test_classify_doc_line_block_comment_start() {
        assert!(matches!(
            classify_doc_line("/* block */"),
            DocLineKind::BlockCommentStart
        ));
        assert!(matches!(
            classify_doc_line("/** doc block */"),
            DocLineKind::BlockCommentStart
        ));
    }

    #[test]
    fn test_classify_doc_line_block_body() {
        match classify_doc_line("* body of block") {
            DocLineKind::BlockCommentBody(text) => assert_eq!(text, "body of block"),
            _ => panic!("expected BlockCommentBody"),
        }
    }

    #[test]
    fn test_classify_doc_line_skip_kinds() {
        assert!(matches!(classify_doc_line(""), DocLineKind::SkipLine));
        assert!(matches!(
            classify_doc_line("#[derive(Debug)]"),
            DocLineKind::SkipLine
        ));
        assert!(matches!(
            classify_doc_line("@override"),
            DocLineKind::SkipLine
        ));
    }

    #[test]
    fn test_classify_doc_line_other() {
        assert!(matches!(classify_doc_line("fn foo()"), DocLineKind::Other));
        assert!(matches!(
            classify_doc_line("let x = 1;"),
            DocLineKind::Other
        ));
    }

    // ── extract_doc_comment ─────────────────────────────────────────────────

    #[test]
    fn test_extract_doc_comment_returns_doc_above_function() {
        let src = "/// This is the doc.\nfn target() {}\n";
        let doc = extract_doc_comment(src, 2);
        assert_eq!(doc, Some("This is the doc.".to_string()));
    }

    #[test]
    fn test_extract_doc_comment_multiline_doc() {
        let src = "/// Line one.\n/// Line two.\nfn t() {}\n";
        let doc = extract_doc_comment(src, 3);
        assert_eq!(doc, Some("Line one. Line two.".to_string()));
    }

    #[test]
    fn test_extract_doc_comment_no_doc_returns_none() {
        let src = "fn t() {}\n";
        let doc = extract_doc_comment(src, 1);
        assert!(doc.is_none());
    }

    #[test]
    fn test_extract_doc_comment_attribute_skipped() {
        // #[derive] line is SkipLine; doc above is captured
        let src = "/// Doc.\n#[derive(Debug)]\nfn t() {}\n";
        let doc = extract_doc_comment(src, 3);
        assert_eq!(doc, Some("Doc.".to_string()));
    }

    #[test]
    fn test_extract_doc_comment_start_line_one_returns_none() {
        let doc = extract_doc_comment("fn t() {}", 1);
        assert!(doc.is_none());
    }
}
