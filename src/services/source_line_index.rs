//! Measured line spans for Rust function definitions, derived from source text.
//!
//! This exists because two production paths used to *invent* function extents
//! instead of measuring them:
//!
//! * `ast_rust_compat` gave the last function in every file
//!   `line_end = line_start + 50` and `metrics.lines = 50` — a constant dressed
//!   up as a measurement, frequently past EOF (#652, #656).
//! * `unified_rust_analyzer` (the analyzer behind `pmat context`) emitted
//!   `line_start: 0`, `line_end: 0` and a hard-coded `lines: 10` (#686).
//!
//! Every span here is read off the source text; when a definition cannot be
//! located the span is [`LineSpan::UNKNOWN`] (zeroes) rather than a plausible
//! guess, so callers can tell "not measured" from "measured as N".

use std::collections::{HashMap, VecDeque};

use crate::cli::language_analyzer::find_brace_balanced_end;

/// A 1-based, inclusive line span in a source file.
///
/// `start == 0` means the definition could not be located in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineSpan {
    pub start: u32,
    pub end: u32,
}

impl LineSpan {
    /// Sentinel for "this definition was not located in the source text".
    pub const UNKNOWN: Self = Self { start: 0, end: 0 };

    /// Number of source lines the definition covers, or 0 when unknown.
    #[must_use]
    pub fn line_count(&self) -> u32 {
        if self.start == 0 || self.end < self.start {
            0
        } else {
            self.end - self.start + 1
        }
    }

    /// True when the span was actually measured.
    #[must_use]
    pub fn is_known(&self) -> bool {
        self.start > 0
    }
}

/// Index of function definition spans in one Rust source file, in textual order.
#[derive(Debug, Clone, Default)]
pub struct FunctionSpans {
    by_name: HashMap<String, VecDeque<LineSpan>>,
    total_lines: u32,
}

impl FunctionSpans {
    /// Scan `content` and record the span of every function definition.
    ///
    /// Duplicate names (an impl method and a free function, or the same method
    /// name in two impl blocks) are kept as a queue in textual order so callers
    /// walking the AST in source order get the right one.
    #[must_use]
    pub fn from_source(content: &str) -> Self {
        let lines: Vec<&str> = content.lines().collect();
        let mut by_name: HashMap<String, VecDeque<LineSpan>> = HashMap::new();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let Some(name) = extract_fn_name(trimmed) else {
                continue;
            };
            let end = definition_end(&lines, idx, trimmed);
            by_name.entry(name).or_default().push_back(LineSpan {
                start: idx as u32 + 1,
                end: end as u32 + 1,
            });
        }

        Self {
            by_name,
            total_lines: lines.len() as u32,
        }
    }

    /// Real number of lines in the file (what `total_complexity.lines` must report).
    #[must_use]
    pub fn total_lines(&self) -> u32 {
        self.total_lines
    }

    /// Consume and return the next span recorded for `name`, in textual order.
    pub fn take(&mut self, name: &str) -> Option<LineSpan> {
        self.by_name.get_mut(name)?.pop_front()
    }

    /// Look at the first remaining span for `name` without consuming it.
    #[must_use]
    pub fn peek(&self, name: &str) -> Option<LineSpan> {
        self.by_name.get(name)?.front().copied()
    }
}

/// Last line of the definition beginning at `start`.
///
/// A body-less declaration (a trait method signature, `fn f(&self);`) ends on
/// its own line — without this check the brace scan would run to EOF and report
/// the whole rest of the file as the function's extent.
fn definition_end(lines: &[&str], start: usize, trimmed: &str) -> usize {
    if is_bodyless_declaration(trimmed) {
        return start;
    }
    find_brace_balanced_end(lines, start, true)
}

fn is_bodyless_declaration(trimmed: &str) -> bool {
    match (trimmed.find('{'), trimmed.find(';')) {
        (Some(brace), Some(semi)) => semi < brace,
        (None, Some(_)) => true,
        _ => false,
    }
}

/// Extract a function name from a trimmed source line, if it defines one.
///
/// Returns `None` for comments, so doc text such as `/// fn 2: does a thing`
/// can never be mistaken for a function (the failure mode in #655).
#[must_use]
pub fn extract_fn_name(trimmed: &str) -> Option<String> {
    let fn_pos = trimmed.find("fn ")?;
    let before = trimmed.get(..fn_pos).unwrap_or_default();
    if before.contains("//") || before.contains("/*") {
        return None;
    }
    let after = trimmed.get(fn_pos + 3..).unwrap_or_default();
    let name_end = after
        .find(|c: char| c == '(' || c == '<' || c.is_whitespace())
        .unwrap_or(after.len());
    let name = after.get(..name_end).unwrap_or_default().trim();
    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_are_real_extents_not_padded_guesses() {
        // #656: the last function used to get line_start + 50, past EOF.
        let src = "fn only_one() -> i32 { 42 }\n";
        let spans = FunctionSpans::from_source(src);
        assert_eq!(spans.total_lines(), 1);
        assert_eq!(spans.peek("only_one"), Some(LineSpan { start: 1, end: 1 }));
    }

    #[test]
    fn last_function_ends_at_its_closing_brace() {
        // #652 fixture: cc_six 1-9, cc_one 11-13, file 13 lines.
        let src = concat!(
            "pub fn cc_six(a: i32) -> i32 {\n",
            "    let mut t = 0;\n",
            "    if a > 1 { t += 1; }\n",
            "    if a > 2 { t += 1; }\n",
            "    if a > 3 { t += 1; }\n",
            "    if a > 4 { t += 1; }\n",
            "    if a > 5 { t += 1; }\n",
            "    t\n",
            "}\n",
            "\n",
            "pub fn cc_one(a: i32) -> i32 {\n",
            "    a + 1\n",
            "}\n",
        );
        let spans = FunctionSpans::from_source(src);
        assert_eq!(spans.total_lines(), 13);
        assert_eq!(spans.peek("cc_six"), Some(LineSpan { start: 1, end: 9 }));
        assert_eq!(spans.peek("cc_one"), Some(LineSpan { start: 11, end: 13 }));
    }

    #[test]
    fn duplicate_names_are_returned_in_textual_order() {
        let src = "impl A {\n    fn go(&self) {}\n}\nfn go() {\n    ()\n}\n";
        let mut spans = FunctionSpans::from_source(src);
        assert_eq!(spans.take("go"), Some(LineSpan { start: 2, end: 2 }));
        assert_eq!(spans.take("go"), Some(LineSpan { start: 4, end: 6 }));
        assert_eq!(spans.take("go"), None);
    }

    #[test]
    fn bodyless_trait_method_does_not_swallow_the_file() {
        let src = "trait T {\n    fn required(&self) -> u8;\n}\n\nfn after() {}\n";
        let spans = FunctionSpans::from_source(src);
        assert_eq!(spans.peek("required"), Some(LineSpan { start: 2, end: 2 }));
        assert_eq!(spans.peek("after"), Some(LineSpan { start: 5, end: 5 }));
    }

    #[test]
    fn doc_comments_are_not_functions() {
        // #655: `/// fn 2: ...` used to be parsed as a function named "2".
        let src = "/// fn 2: helper\n/// fn 3: other\nfn real() {}\n";
        let spans = FunctionSpans::from_source(src);
        assert_eq!(spans.peek("2"), None);
        assert_eq!(spans.peek("3"), None);
        assert_eq!(spans.peek("real"), Some(LineSpan { start: 3, end: 3 }));
    }

    #[test]
    fn unknown_span_reports_zero_lines_not_one() {
        assert_eq!(LineSpan::UNKNOWN.line_count(), 0);
        assert!(!LineSpan::UNKNOWN.is_known());
        assert!(LineSpan { start: 4, end: 4 }.is_known());
        assert_eq!(LineSpan { start: 4, end: 4 }.line_count(), 1);
    }

    #[test]
    fn empty_source_yields_empty_index() {
        let spans = FunctionSpans::from_source("");
        assert_eq!(spans.total_lines(), 0);
        assert_eq!(spans.peek("anything"), None);
    }

    #[test]
    fn extract_fn_name_handles_qualifiers_and_rejects_non_functions() {
        assert_eq!(extract_fn_name("fn foo() {}"), Some("foo".to_string()));
        assert_eq!(extract_fn_name("pub fn bar() {}"), Some("bar".to_string()));
        assert_eq!(
            extract_fn_name("async fn baz() {}"),
            Some("baz".to_string())
        );
        assert_eq!(
            extract_fn_name("pub async fn qux() {}"),
            Some("qux".to_string())
        );
        assert_eq!(
            extract_fn_name("fn generic<T>() {}"),
            Some("generic".to_string())
        );
        assert_eq!(extract_fn_name("let x = 42;"), None);
        assert_eq!(extract_fn_name("// fn commented() {}"), None);
    }
}
