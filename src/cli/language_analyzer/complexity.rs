#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared complexity calculation logic: BraceState, brace-balanced end finding,
//! and the generic ComplexityVisitor used by multiple language analyzers.

use crate::services::complexity::ComplexityMetrics;

/// State machine for string-aware brace counting.
/// Handles string literals, char literals, line/block comments, and Rust raw strings.
pub(crate) struct BraceState {
    brace_count: i32,
    found_first_brace: bool,
    in_string: bool,
    in_block_comment: bool,
    in_raw_string: bool,
    raw_hashes: usize,
    escape_next: bool,
}

impl BraceState {
    pub(crate) fn new() -> Self {
        Self {
            brace_count: 0,
            found_first_brace: false,
            in_string: false,
            in_block_comment: false,
            in_raw_string: false,
            raw_hashes: 0,
            escape_next: false,
        }
    }

    /// Process one line of source. Returns true when braces reach balance.
    pub(crate) fn process_line(&mut self, chars: &[char], handle_raw_strings: bool) -> bool {
        let len = chars.len();
        let mut j = 0;
        while j < len {
            if self.escape_next {
                self.escape_next = false;
                j += 1;
                continue;
            }
            if self.in_block_comment || self.in_string || self.in_raw_string {
                j = self.advance_literal(chars, j);
                continue;
            }
            j = self.advance_normal(chars, j, handle_raw_strings);
            if self.found_first_brace && self.brace_count == 0 {
                return true;
            }
        }
        false
    }

    /// Advance through one character inside a literal (string, raw string, block comment).
    fn advance_literal(&mut self, chars: &[char], j: usize) -> usize {
        let len = chars.len();
        let ch = chars[j];
        if self.in_block_comment {
            if ch == '*' && j + 1 < len && chars[j + 1] == '/' {
                self.in_block_comment = false;
                return j + 2;
            }
            return j + 1;
        }
        if self.in_string {
            if ch == '\\' {
                self.escape_next = true;
            } else if ch == '"' {
                self.in_string = false;
            }
            return j + 1;
        }
        // in_raw_string
        if ch == '"' {
            let h = chars[j + 1..].iter().take_while(|&&c| c == '#').count();
            if h >= self.raw_hashes {
                self.in_raw_string = false;
                return j + 1 + self.raw_hashes;
            }
        }
        j + 1
    }

    /// Process one character in normal state. Returns next position.
    fn advance_normal(&mut self, chars: &[char], j: usize, handle_raw_strings: bool) -> usize {
        let len = chars.len();
        let ch = chars[j];
        // Line comment: skip to end
        if ch == '/' && j + 1 < len && chars[j + 1] == '/' {
            return len;
        }
        // Block comment
        if ch == '/' && j + 1 < len && chars[j + 1] == '*' {
            self.in_block_comment = true;
            return j + 2;
        }
        // Char literal: 'x' or '\x'
        if ch == '\'' && j + 2 < len {
            if chars[j + 1] == '\\' && j + 3 < len && chars[j + 3] == '\'' {
                return j + 4;
            }
            if chars[j + 2] == '\'' {
                return j + 3;
            }
        }
        // Raw string
        if handle_raw_strings && ch == 'r' {
            let h = chars[j + 1..].iter().take_while(|&&c| c == '#').count();
            let qp = j + 1 + h;
            if qp < len && chars[qp] == '"' {
                self.in_raw_string = true;
                self.raw_hashes = h;
                return qp + 1;
            }
        }
        // Regular string
        if ch == '"' {
            self.in_string = true;
            return j + 1;
        }
        // Braces
        if ch == '{' {
            self.brace_count += 1;
            self.found_first_brace = true;
        } else if ch == '}' {
            self.brace_count -= 1;
        }
        j + 1
    }
}

/// String-aware brace counting for C-like languages.
/// Prevents false positive complexity violations from `{` inside string literals.
pub(crate) fn find_brace_balanced_end(
    lines: &[&str],
    start: usize,
    handle_raw_strings: bool,
) -> usize {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut state = BraceState::new();
    for (i, line) in lines.iter().enumerate().skip(start) {
        let chars: Vec<char> = line.chars().collect();
        if state.process_line(&chars, handle_raw_strings) {
            return i;
        }
    }
    lines.len() - 1
}

/// Complexity visitor for analyzing code metrics
pub(crate) struct ComplexityVisitor {
    cyclomatic: u16,
    cognitive: u16,
    nesting: u8,
    max_nesting: u8,
    lines: u16,
}

impl ComplexityVisitor {
    pub(crate) fn new() -> Self {
        Self {
            cyclomatic: 1, // Base complexity
            cognitive: 0,
            nesting: 0,
            max_nesting: 0,
            lines: 0,
        }
    }

    pub(crate) fn analyze_lines(&mut self, lines: &[&str]) {
        debug_assert!(!lines.is_empty(), "lines must not be empty");
        self.lines = lines.len() as u16;

        for line in lines {
            let trimmed = line.trim();

            // Count control flow keywords
            if self.is_control_flow(trimmed) {
                self.cyclomatic += 1;
                self.cognitive += 1 + u16::from(self.nesting);
            }

            if trimmed.contains("else") {
                self.cyclomatic += 1;
                self.cognitive += 1;
            }

            // Track nesting
            if trimmed.ends_with('{') || trimmed.ends_with(':') {
                self.nesting += 1;
                self.max_nesting = self.max_nesting.max(self.nesting);
            }
            if trimmed.starts_with('}') || (trimmed.is_empty() && self.nesting > 0) {
                self.nesting = self.nesting.saturating_sub(1);
            }
        }
    }

    fn is_control_flow(&self, line: &str) -> bool {
        debug_assert!(!line.is_empty(), "line must not be empty");
        line.contains("if ")
            || line.contains("while ")
            || line.contains("for ")
            || line.contains("match ")
            || line.contains("switch ")
            || line.contains("case ")
            || line.contains("elif ")
            || line.contains("except ")
            || line.contains("catch ")
    }

    pub(crate) fn into_metrics(self) -> ComplexityMetrics {
        ComplexityMetrics {
            cyclomatic: self.cyclomatic.min(255),
            cognitive: self.cognitive.min(255),
            nesting_max: self.max_nesting,
            lines: self.lines,
            halstead: None,
        }
    }
}
