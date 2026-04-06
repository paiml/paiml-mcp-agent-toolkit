// Lean 4 analysis functions: sorry counting, theorem counting, complexity analysis, file analysis
// Free functions and LeanComplexityAnalyzer impl for Lean-specific metrics

/// Count sorry occurrences in Lean source (proof incompleteness metric)
///
/// Handles: line comments (--), nested block comments (/- ... -/),
/// and word-boundary checking to avoid false positives from identifiers
/// containing "sorry" as a substring.
pub fn count_sorry(source: &str) -> usize {
    debug_assert!(!source.is_empty(), "source must not be empty");
    let mut count = 0;
    let mut in_block_comment = 0i32;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip line comments
        if trimmed.starts_with("--") {
            continue;
        }

        // Strip block comments and check remaining text
        let cleaned = strip_block_comments(trimmed, &mut in_block_comment);

        // If we're still inside a block comment after processing, skip
        if in_block_comment > 0 {
            continue;
        }

        // Word-boundary check: sorry must be a standalone word
        if contains_sorry_word(&cleaned) {
            count += 1;
        }
    }

    count
}

/// Strips block comment content from a line, updating nesting depth.
/// Returns the text outside block comments.
fn strip_block_comments(line: &str, depth: &mut i32) -> String {
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

/// Checks if a line contains "sorry" as a standalone word (not part of an identifier).
fn contains_sorry_word(line: &str) -> bool {
    let bytes = line.as_bytes();
    let sorry = b"sorry";

    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok = abs_idx == 0 || !is_ident_char(bytes[abs_idx - 1]);
            let after_ok = abs_idx + sorry.len() >= bytes.len()
                || !is_ident_char(bytes[abs_idx + sorry.len()]);
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

/// Returns true if the byte is a valid Lean identifier character.
fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Count theorem and lemma declarations in Lean source
pub fn count_theorems(source: &str) -> usize {
    debug_assert!(!source.is_empty(), "source must not be empty");
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("theorem ")
                || trimmed.starts_with("lemma ")
                || trimmed.starts_with("private theorem ")
                || trimmed.starts_with("private lemma ")
        })
        .count()
}

impl Default for LeanComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LeanComplexityAnalyzer {
    /// Creates a new Lean complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Lean source code (complexity <=10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 1;

        for line in source.lines() {
            let trimmed = line.trim();

            // Control flow and branching
            if trimmed.contains("if ")
                || trimmed.contains("match ")
                || trimmed.contains("| ")
                || trimmed.starts_with("by ")
                || trimmed.contains("sorry")
            {
                self.cyclomatic_complexity += 1;
                self.cognitive_complexity += 1;
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
}

/// Public async function to analyze a Lean file and return FileContext
pub async fn analyze_lean_file(
    path: &Path,
) -> Result<crate::services::context::FileContext, crate::models::error::TemplateError> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::models::error::TemplateError;
    use crate::services::context::FileContext;

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    let visitor = LeanAstVisitor::new(path);
    let items = visitor
        .analyze_lean_source(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    Ok(FileContext {
        path: path.display().to_string(),
        language: "lean".to_string(),
        items,
        complexity_metrics: None,
    })
}
