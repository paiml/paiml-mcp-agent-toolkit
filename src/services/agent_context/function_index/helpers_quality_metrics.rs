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
        if trimmed.starts_with("#if") || trimmed.starts_with("#ifdef") || trimmed.starts_with("#ifndef") {
            ifdef_depth += 1;
            penalty += ifdef_depth; // +1 per nesting level
        } else if trimmed.starts_with("#endif") {
            ifdef_depth = ifdef_depth.saturating_sub(1);
        }

        // Count macro calls (UPPER_CASE identifiers with parens, common C/C++ convention)
        if trimmed.contains("GGML_") || trimmed.contains("TORCH_") || trimmed.contains("AT_")
            || trimmed.contains("CUDA_") || trimmed.contains("CHECK_") {
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
    if source.contains("__shfl_") || source.contains("__ballot_") || source.contains("__any_sync")
        || source.contains("__all_sync") {
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
#[allow(clippy::cast_possible_truncation)]
pub(super) fn count_satd_markers(source: &str) -> u32 {
    let mut count = 0u32;
    let mut in_block_comment = false;
    let mut in_raw_string = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip lines inside raw string literals
        if update_raw_string_state(trimmed, &mut in_raw_string) {
            continue;
        }

        // Track block comment state
        if in_block_comment {
            count += count_markers_in_line(trimmed);
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            in_block_comment = true;
            count += count_markers_in_line(trimmed);
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        // Skip doc comments (/// and //!) — these describe behavior, not debt
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            continue;
        }

        count += count_markers_in_comment(trimmed);
    }

    count
}

/// Count SATD markers in a single line (used for block comments).
fn count_markers_in_line(line: &str) -> u32 {
    let upper = line.to_uppercase();
    let mut count = 0u32;
    for marker in ["TODO", "FIXME", "HACK", "OPTIMIZE"] {
        count += upper.matches(marker).count() as u32;
    }
    count
}

/// Count SATD markers in inline comment portion of a line.
/// Skips if // is inside a string literal (odd quote count before //).
fn count_markers_in_comment(trimmed: &str) -> u32 {
    let Some(comment_start) = trimmed.find("//") else {
        return 0;
    };
    let before = &trimmed[..comment_start];
    if before.chars().filter(|&c| c == '"').count() % 2 != 0 {
        return 0;
    }
    count_markers_in_line(&trimmed[comment_start..])
}

/// Track raw string literal state. Returns true if line should be skipped.
fn update_raw_string_state(trimmed: &str, in_raw_string: &mut bool) -> bool {
    if *in_raw_string {
        if trimmed.contains("\"#") || trimmed.ends_with('"') {
            *in_raw_string = false;
        }
        return true;
    }
    if let Some(pos) = trimmed.find("r#\"") {
        let after_open = &trimmed[pos + 3..];
        if !after_open.contains("\"#") {
            *in_raw_string = true;
        }
        return true;
    }
    false
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

/// Calculate simplified TDG score
#[allow(clippy::cast_possible_truncation)]
pub(super) fn calculate_simple_tdg(complexity: u32, satd_count: u32, loc: u32) -> f32 {
    let mut score = 0.0f32;

    // Complexity penalty (0-4 points)
    // Divisor of 25: CC=50 -> 2.0 (B boundary). Functions at the pre-commit
    // CC<=30 gate get score=1.2 (safe A). Dispatchers (CC~45) score 1.8 (A).
    // CC=75 -> 3.0, CC=100 -> 4.0 (cap).
    score += (complexity as f32 / 25.0).min(4.0);

    // SATD penalty (0-2 points, first 2 markers free to reduce false positives)
    // Many functions reference SATD markers descriptively (detector code, enums).
    // 3 SATD -> 0.5, 4 -> 1.0, 5 -> 1.5, 6+ -> 2.0.
    score += (satd_count.saturating_sub(2) as f32 * 0.5).min(2.0);

    // LOC penalty (0-2 points for > 200 lines)
    // Threshold at 200: functions under 200 LOC are rarely problematic.
    // Divisor of 200: LOC=400 -> 1.0 penalty, LOC=600 -> 2.0 (capped).
    if loc > 200 {
        score += ((loc - 200) as f32 / 200.0).min(2.0);
    }

    score.min(10.0)
}

/// Convert TDG score to letter grade
pub(super) fn score_to_grade(score: f32) -> String {
    match score {
        s if s < 2.0 => "A".to_string(),
        s if s < 4.0 => "B".to_string(),
        s if s < 6.0 => "C".to_string(),
        s if s < 8.0 => "D".to_string(),
        _ => "F".to_string(),
    }
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

    let lines: Vec<&str> = content.lines().collect();
    let mut doc_lines = Vec::new();

    for i in (0..start_line - 1).rev() {
        let line = lines.get(i)?.trim();
        match classify_doc_line(line) {
            DocLineKind::DocComment(text) => doc_lines.push(text),
            DocLineKind::BlockCommentBody(text) => doc_lines.push(text),
            DocLineKind::BlockCommentStart | DocLineKind::Other => break,
            DocLineKind::SkipLine => continue,
        }
    }

    if doc_lines.is_empty() {
        return None;
    }
    doc_lines.reverse();
    Some(doc_lines.join(" "))
}
