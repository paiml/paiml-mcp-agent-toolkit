#![cfg_attr(coverage_nightly, coverage(off))]
//! Python language analysis.

use super::complexity::ComplexityVisitor;
use super::types::{FunctionInfo, LanguageAnalyzer};
use crate::services::complexity::ComplexityMetrics;

/// The synthetic unit that carries a Python file's MODULE-LEVEL control flow.
///
/// `pmat analyze complexity` measures a file by summing the complexity of the
/// functions it found. For Python that silently dropped everything outside a
/// `def`: a file whose branching lives at module level — a config script, a
/// `setup.py`, a notebook export, anything guarded by `if __name__` — reported
/// `cyclomatic: 1, cognitive: 1, functions: []`, byte-identical to a file
/// holding three assignments. A metric that cannot tell those two apart
/// measures nothing.
///
/// Module-level code is therefore reported as one more unit in `functions`,
/// which is the unit the whole pipeline already speaks (totals, hotspots,
/// `--max-cyclomatic` violations and Big-O all read that list) rather than a
/// new file-level channel only the JSON emitter would know about.
///
/// `<module>` is CPython's own name for the top-level frame — it is what a
/// traceback prints for a line outside any `def` — and it cannot collide with
/// a real function: `<` is not legal in a Python identifier.
pub(crate) const MODULE_UNIT: &str = "<module>";

/// Python analyzer
pub struct PythonAnalyzer {
    /// Report module-level control flow as a synthetic [`MODULE_UNIT`].
    ///
    /// True for Python. False for the other languages that borrow this
    /// analyzer for its `def`-shaped function extraction (Ruby, Lean, and the
    /// structural-only YAML/Markdown passes): masking a Python `def` body by
    /// indentation is not a statement about any of them, and counting `if ` in
    /// Markdown prose would be a regression, not a fix.
    module_unit: bool,
}

impl Default for PythonAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonAnalyzer {
    /// The Python analyzer, counting module-level control flow.
    #[must_use]
    pub fn new() -> Self {
        Self { module_unit: true }
    }

    /// The `def`-extraction half only, for the languages that reuse it.
    #[must_use]
    pub fn without_module_unit() -> Self {
        Self { module_unit: false }
    }
}

impl LanguageAnalyzer for PythonAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                if let Some(name) = self.extract_function_name(trimmed) {
                    let line_end = self.find_function_end(&lines, line_num);
                    functions.push(FunctionInfo {
                        name,
                        line_start: line_num,
                        line_end,
                    });
                }
            }
        }

        // A module unit is emitted only when module level actually branches.
        // A module whose top level is imports and assignments has base
        // complexity and nothing to say; giving every Python file a `<module>`
        // row would move the summary's medians without adding information.
        if let Some(last) = self.module_unit_span(&lines) {
            functions.insert(
                0,
                FunctionInfo {
                    name: MODULE_UNIT.to_string(),
                    line_start: 0,
                    line_end: last,
                },
            );
        }

        functions
    }

    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics {
        let lines: Vec<&str> = content.lines().collect();

        // The module unit spans the whole file, so it cannot be measured by
        // slicing: that would re-count every `def` body the sum already holds.
        if function.name == MODULE_UNIT {
            return Self::module_metrics(&lines);
        }

        let function_lines = &lines[function.line_start..=function.line_end];

        let mut visitor = ComplexityVisitor::new();
        visitor.analyze_lines(function_lines);
        visitor.into_metrics()
    }
}

impl PythonAnalyzer {
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let line = line.trim();
        if let Some(pos) = line.find("def ") {
            let after = line.get(pos + 4..).unwrap_or_default();
            if let Some(paren_pos) = after.find('(') {
                let name = after.get(..paren_pos).unwrap_or_default().trim();
                return Some(name.to_string());
            }
        }
        None
    }

    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        if lines.is_empty() || start >= lines.len() {
            return start;
        }

        // Get indentation level of function definition
        let def_indent = lines[start].len() - lines[start].trim_start().len();

        // Find next line with same or lower indentation
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            if !line.trim().is_empty() {
                let indent = line.len() - line.trim_start().len();
                if indent <= def_indent {
                    return i - 1;
                }
            }
        }

        lines.len() - 1
    }

    /// Last line of the synthetic module unit, or `None` when there is no
    /// module-level control flow worth reporting.
    fn module_unit_span(&self, lines: &[&str]) -> Option<usize> {
        if !self.module_unit || lines.is_empty() {
            return None;
        }
        let metrics = Self::module_metrics(lines);
        if metrics.cyclomatic > 1 || metrics.cognitive > 0 {
            Some(lines.len() - 1)
        } else {
            None
        }
    }

    /// Complexity of everything outside any top-level `def`/`class`.
    fn module_metrics(lines: &[&str]) -> ComplexityMetrics {
        let masked = Self::module_level_lines(lines);
        let refs: Vec<&str> = masked.iter().map(String::as_str).collect();
        let mut visitor = ComplexityVisitor::new();
        visitor.analyze_lines(&refs);
        visitor.into_metrics()
    }

    /// One entry per input line, positions preserved, holding only the code
    /// that runs at module level.
    ///
    /// Blanked out: the body of every top-level `def`/`class` (already measured
    /// as that function, and double-counting it would be its own defect),
    /// comments, and the inside of triple-quoted strings — a module docstring
    /// that says "if the caller passes None" is prose, not a branch.
    fn module_level_lines(lines: &[&str]) -> Vec<String> {
        let mut out = vec![String::new(); lines.len()];
        let mut fence: Option<char> = None;
        let mut i = 0;

        while i < lines.len() {
            if fence.is_some() {
                out[i] = Self::live_code(lines[i], &mut fence);
                i += 1;
                continue;
            }
            let trimmed = lines[i].trim();
            // Leading indent only: trailing whitespace on a `def` line must not
            // hide the block from the mask and leak its body into module scope.
            let indented = lines[i].len() != lines[i].trim_start().len();
            if !indented && Self::is_definition_header(trimmed) {
                // A decorator owns only its own line; the `def`/`class` that
                // follows is handled on the next pass round the loop.
                i = if trimmed.starts_with('@') {
                    i + 1
                } else {
                    Self::block_end(lines, i) + 1
                };
                continue;
            }
            out[i] = Self::live_code(lines[i], &mut fence);
            i += 1;
        }

        out
    }

    fn is_definition_header(trimmed: &str) -> bool {
        trimmed.starts_with("def ")
            || trimmed.starts_with("async def ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with('@')
    }

    /// Last line of the top-level block opened at `start` (indent 0).
    fn block_end(lines: &[&str], start: usize) -> usize {
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            if !line.trim().is_empty() && line.len() == line.trim_start().len() {
                return i - 1;
            }
        }
        lines.len().saturating_sub(1)
    }

    /// The part of `line` that is executable code: comments and the contents of
    /// triple-quoted strings removed, `fence` carrying the open-string state to
    /// the next line.
    fn live_code(line: &str, fence: &mut Option<char>) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::new();
        let mut i = 0;

        while i < chars.len() {
            if let Some(open) = *fence {
                if Self::triple_at(&chars, i) == Some(open) {
                    *fence = None;
                    i += 3;
                } else {
                    i += 1;
                }
                continue;
            }
            if let Some(quote) = Self::triple_at(&chars, i) {
                *fence = Some(quote);
                i += 3;
                continue;
            }
            if chars[i] == '#' {
                break;
            }
            out.push(chars[i]);
            i += 1;
        }

        out
    }

    /// The quote character of a triple-quote token starting at `i`, if any.
    fn triple_at(chars: &[char], i: usize) -> Option<char> {
        let c = *chars.get(i)?;
        if c != '"' && c != '\'' {
            return None;
        }
        if chars.get(i + 1) == Some(&c) && chars.get(i + 2) == Some(&c) {
            Some(c)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod module_level_tests {
    use super::*;
    use crate::cli::language_analyzer::{analyze_with_heuristics, Language};
    use std::path::Path;

    /// Module-level control flow, five levels deep, and not one `def`.
    const NESTED_MODULE_CODE: &str = r#"
import os
import sys

CONFIG = {}

if os.environ.get("MODE") == "prod":
    if sys.platform == "linux":
        for path in sys.path:
            if path.startswith("/usr"):
                while len(path) > 3:
                    path = path[:-1]
            elif path.startswith("/opt"):
                CONFIG["opt"] = True
            else:
                CONFIG["other"] = True
    elif sys.platform == "darwin":
        try:
            CONFIG["mac"] = True
        except KeyError:
            CONFIG["mac"] = False
    else:
        CONFIG["unknown"] = True
else:
    for i in range(10):
        if i % 2 == 0:
            CONFIG[i] = "even"

SELECTED = [x for x in CONFIG if x]
"#;

    /// Imports and assignments. Nothing branches.
    const TRIVIAL_MODULE_CODE: &str = r#"
import os

NAME = "trivial"
VALUE = 42
PATH = os.path.join("a", "b")
print(NAME, VALUE, PATH)
"#;

    fn analyze(content: &str) -> crate::services::complexity::FileComplexityMetrics {
        analyze_with_heuristics(Path::new("mod_under_test.py"), content, Language::Python)
            .expect("python heuristic analysis must succeed")
    }

    /// THE DEFECT. A file with five levels of module-level branching reported
    /// `cyclomatic: 1, cognitive: 1, functions: []` — identical to a file of
    /// three assignments.
    #[test]
    fn module_level_control_flow_is_counted() {
        let metrics = analyze(NESTED_MODULE_CODE);

        assert!(
            metrics.total_complexity.cyclomatic > 1,
            "module-level branching must raise cyclomatic above the base of 1, got {}",
            metrics.total_complexity.cyclomatic
        );
        assert!(
            metrics.total_complexity.cognitive > 1,
            "module-level branching must raise cognitive above 1, got {}",
            metrics.total_complexity.cognitive
        );
        assert!(
            metrics.functions.iter().any(|f| f.name == MODULE_UNIT),
            "module-level code must be reported as a `{MODULE_UNIT}` unit, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// COUNTER-TEST. "Always report high" must not pass: a genuinely trivial
    /// Python file still scores the base complexity, and gains no unit.
    #[test]
    fn a_trivial_module_still_scores_one() {
        let metrics = analyze(TRIVIAL_MODULE_CODE);

        assert_eq!(
            metrics.total_complexity.cyclomatic, 1,
            "a module with no branching must stay at base complexity"
        );
        assert!(
            metrics.functions.is_empty(),
            "a module with no branching gains no `{MODULE_UNIT}` unit, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// COUNTER-TEST. The two files must be DISTINGUISHABLE — the property the
    /// defect destroyed. Equal scores are the bug however high they are.
    #[test]
    fn the_nested_module_outscores_the_trivial_one() {
        let nested = analyze(NESTED_MODULE_CODE).total_complexity.cyclomatic;
        let trivial = analyze(TRIVIAL_MODULE_CODE).total_complexity.cyclomatic;
        assert!(
            nested > trivial,
            "nested ({nested}) must outscore trivial ({trivial})"
        );
    }

    /// COUNTER-TEST. Function bodies are measured as functions. Folding them
    /// into the module unit as well would double-count every Python file.
    #[test]
    fn a_def_body_is_not_counted_as_module_level() {
        let content = "\
def branchy(x):
    if x:
        for i in x:
            if i:
                return i
    return None
";
        let metrics = analyze(content);
        assert_eq!(
            metrics.functions.len(),
            1,
            "only `branchy` is a unit here, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
        assert_eq!(metrics.functions[0].name, "branchy");
    }

    /// A class body is a top-level block too: its methods are their own units.
    #[test]
    fn a_class_body_is_not_counted_as_module_level() {
        let content = "\
class Thing:
    def go(self, x):
        if x:
            return 1
        return 0
";
        let metrics = analyze(content);
        assert!(
            !metrics.functions.iter().any(|f| f.name == MODULE_UNIT),
            "a file that is one class has no module-level branching, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// Both halves of a mixed file are counted, and the total holds both.
    #[test]
    fn a_mixed_file_sums_both_halves() {
        let content = "\
import sys

def helper(x):
    if x:
        return 1
    return 0

if sys.argv:
    for a in sys.argv:
        if a.startswith('-'):
            helper(a)
";
        let metrics = analyze(content);
        let names: Vec<&str> = metrics.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(
            names.contains(&MODULE_UNIT),
            "expected module unit in {names:?}"
        );
        assert!(names.contains(&"helper"), "expected helper in {names:?}");

        let module = metrics
            .functions
            .iter()
            .find(|f| f.name == MODULE_UNIT)
            .expect("module unit present");
        let helper = metrics
            .functions
            .iter()
            .find(|f| f.name == "helper")
            .expect("helper present");
        assert_eq!(
            metrics.total_complexity.cyclomatic,
            module.metrics.cyclomatic + helper.metrics.cyclomatic,
            "the file total is the sum of its units"
        );
    }

    /// Prose is not control flow. A docstring full of the word `if` must not
    /// invent a module unit.
    #[test]
    fn a_docstring_is_not_control_flow() {
        let content = "\
\"\"\"Utilities.

Use this if you want a thing. Prefer it while you can, and
for each caller, catch what it raises.
\"\"\"
VALUE = 1
";
        let metrics = analyze(content);
        assert!(
            metrics.functions.is_empty(),
            "a docstring is prose, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
        assert_eq!(metrics.total_complexity.cyclomatic, 1);
    }

    /// Same for comments.
    #[test]
    fn a_comment_is_not_control_flow() {
        let content = "\
# if this were code it would branch
# for every item, while it lasts
VALUE = 1
";
        let metrics = analyze(content);
        assert!(
            metrics.functions.is_empty(),
            "a comment is not a branch, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// A decorated function is still a function: the decorator line must not
    /// drag the `def` below it back into module scope.
    #[test]
    fn a_decorated_def_is_not_module_level() {
        let content = "\
import functools

@functools.cache
def cached(x):
    if x:
        return 1
    return 0
";
        let metrics = analyze(content);
        assert!(
            !metrics.functions.iter().any(|f| f.name == MODULE_UNIT),
            "a decorated def is not module-level code, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// The languages that borrow this analyzer for `def`-shaped extraction do
    /// not get Python's indentation rules applied to them.
    #[test]
    fn markdown_gains_no_module_unit() {
        let content = "\
# Notes

Run it if you like, for as long as you like, while it works.
";
        let metrics = analyze_with_heuristics(
            Path::new("notes.md"),
            content,
            crate::cli::language_analyzer::Language::Markdown,
        )
        .expect("markdown heuristic analysis must succeed");
        assert!(
            metrics.functions.is_empty(),
            "markdown prose is not module-level Python, got {:?}",
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// THE UNIT IS THE ONE THE CODEBASE ALREADY USES. Take the module-level
    /// file and indent the whole of it into a `def`: the branches are the same
    /// branches, so the count must be the same count. This is the property that
    /// makes `<module>` a measurement rather than a second, private scale — and
    /// it is measured against the number pmat produced for the wrapped form
    /// before this fix existed (cyclomatic 22 for the fixture used here).
    #[test]
    fn module_level_scores_the_same_as_the_same_code_inside_a_def() {
        let wrapped: String = std::iter::once("def everything():".to_string())
            .chain(NESTED_MODULE_CODE.lines().map(|l| {
                if l.trim().is_empty() {
                    String::new()
                } else {
                    format!("    {l}")
                }
            }))
            .collect::<Vec<_>>()
            .join("\n");

        let at_module = analyze(NESTED_MODULE_CODE).total_complexity.cyclomatic;
        let in_def = analyze(&wrapped).total_complexity.cyclomatic;

        assert!(at_module > 1, "the fixture must branch, got {at_module}");
        assert_eq!(
            at_module, in_def,
            "the same branches must count the same at module level ({at_module}) \
             as inside a def ({in_def})"
        );
    }

    #[test]
    fn empty_content_is_not_a_module_unit() {
        let metrics = analyze("");
        assert!(metrics.functions.is_empty());
        assert_eq!(metrics.total_complexity.cyclomatic, 1);
    }
}
