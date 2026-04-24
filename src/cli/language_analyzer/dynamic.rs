#![cfg_attr(coverage_nightly, coverage(off))]
//! Dynamic/other language analysis: Lua, SQL, Scala, and similar languages.

use super::complexity::find_brace_balanced_end;
use super::types::{FunctionInfo, LanguageAnalyzer};
use crate::services::complexity::ComplexityMetrics;

// ---------------------------------------------------------------------------
// Struct definitions
// ---------------------------------------------------------------------------

/// Lua language analyzer
///
/// Lua uses `function name() ... end` and `local function name() ... end` syntax.
/// Block termination is via `end` keyword matching.
pub struct LuaAnalyzer;

/// SQL language analyzer -- extracts CREATE FUNCTION/VIEW/TRIGGER/PROCEDURE and CTEs
pub struct SqlAnalyzer;

/// Scala language analyzer -- extracts def/val/class/object/trait
pub struct ScalaAnalyzer;

// ---------------------------------------------------------------------------
// Lua trait implementation (requires cfg feature gates with imports)
// ---------------------------------------------------------------------------

impl LanguageAnalyzer for LuaAnalyzer {
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        #[cfg(feature = "lua-ast")]
        {
            if let Some(fns) = self.extract_functions_treesitter(content) {
                return fns;
            }
        }
        self.extract_functions_heuristic(content)
    }

    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics {
        #[cfg(feature = "lua-ast")]
        {
            if let Some(m) = self.estimate_complexity_treesitter(content, function) {
                return m;
            }
        }
        self.estimate_complexity_heuristic(content, function)
    }
}

// ---------------------------------------------------------------------------
// Include implementation methods and remaining trait impls
// ---------------------------------------------------------------------------

include!("dynamic_lua.rs");
include!("dynamic_sql.rs");
include!("dynamic_scala.rs");

#[cfg(test)]
mod lua_tests {
    //! PMAT-646: cover dynamic_lua.rs pure fn + trait-dispatch paths.
    use super::*;

    // --- is_function_declaration / extract_function_name (heuristic helpers) ---

    #[test]
    fn test_is_function_declaration_plain() {
        let a = LuaAnalyzer;
        assert!(a.is_function_declaration("function foo()"));
        assert!(a.is_function_declaration("local function bar(x)"));
        assert!(!a.is_function_declaration("-- a comment"));
        assert!(!a.is_function_declaration("x = 1"));
        // Missing parenthesis → false.
        assert!(!a.is_function_declaration("function no_paren"));
    }

    #[test]
    fn test_extract_function_name_local_function() {
        let a = LuaAnalyzer;
        assert_eq!(
            a.extract_function_name("local function my_fn(a, b)"),
            Some("my_fn".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_plain_function() {
        let a = LuaAnalyzer;
        assert_eq!(
            a.extract_function_name("function top_level()"),
            Some("top_level".to_string())
        );
    }

    #[test]
    fn test_extract_function_name_returns_none_for_non_function_line() {
        let a = LuaAnalyzer;
        assert_eq!(a.extract_function_name("return 42"), None);
    }

    #[test]
    fn test_extract_function_name_returns_none_for_empty_name() {
        let a = LuaAnalyzer;
        // "function ()" would be an anonymous function; the helper returns None
        // when the name between `function ` and `(` is empty.
        assert_eq!(a.extract_function_name("function ()"), None);
    }

    // --- find_function_end (heuristic) ---

    #[test]
    fn test_find_function_end_simple_flat_function() {
        let a = LuaAnalyzer;
        let src = "function foo()\n  return 1\nend\n";
        let lines: Vec<&str> = src.lines().collect();
        // "function foo()" is at line 0; matching `end` is at line 2.
        assert_eq!(a.find_function_end(&lines, 0), 2);
    }

    #[test]
    fn test_find_function_end_with_nested_if() {
        let a = LuaAnalyzer;
        let src = "function foo()\n  if x then\n    return 1\n  end\nend\n";
        let lines: Vec<&str> = src.lines().collect();
        // Last `end` (outer) at line 4 matches.
        assert_eq!(a.find_function_end(&lines, 0), 4);
    }

    #[test]
    fn test_find_function_end_repeat_until_closes() {
        let a = LuaAnalyzer;
        let src = "function f()\n  repeat\n    x = x + 1\n  until x > 5\nend\n";
        let lines: Vec<&str> = src.lines().collect();
        // `until x > 5` closes the repeat depth, final `end` closes function.
        let end = a.find_function_end(&lines, 0);
        // End is last `end` at line 4.
        assert_eq!(end, 4);
    }

    #[test]
    fn test_find_function_end_skips_comment_only_lines() {
        let a = LuaAnalyzer;
        let src = "function f()\n  -- comment\n  return 1\nend\n";
        let lines: Vec<&str> = src.lines().collect();
        assert_eq!(a.find_function_end(&lines, 0), 3);
    }

    #[test]
    fn test_find_function_end_missing_end_falls_back_to_last_line() {
        let a = LuaAnalyzer;
        // Function with no matching `end` (source truncated).
        let src = "function f()\n  return 1\n";
        let lines: Vec<&str> = src.lines().collect();
        // Fallback: lines.len() - 1 == 1.
        assert_eq!(a.find_function_end(&lines, 0), lines.len() - 1);
    }

    // --- extract_functions_heuristic ---

    #[test]
    fn test_extract_functions_heuristic_empty_source() {
        let a = LuaAnalyzer;
        assert!(a.extract_functions_heuristic("").is_empty());
    }

    #[test]
    fn test_extract_functions_heuristic_single_function() {
        let a = LuaAnalyzer;
        let src = "function lonely()\n  return 0\nend\n";
        let fns = a.extract_functions_heuristic(src);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "lonely");
        assert_eq!(fns[0].line_start, 0);
    }

    #[test]
    fn test_extract_functions_heuristic_multiple_functions_mixed_styles() {
        let a = LuaAnalyzer;
        let src = "local function helper(x)\n  return x * 2\nend\n\nfunction top()\n  return helper(3)\nend\n";
        let fns = a.extract_functions_heuristic(src);
        assert_eq!(fns.len(), 2);
        assert!(fns.iter().any(|f| f.name == "helper"));
        assert!(fns.iter().any(|f| f.name == "top"));
    }

    // --- estimate_complexity_heuristic ---

    #[test]
    fn test_estimate_complexity_heuristic_flat_function_is_1() {
        let a = LuaAnalyzer;
        let src = "function flat()\n  return 1\nend\n";
        let fns = a.extract_functions_heuristic(src);
        let m = a.estimate_complexity_heuristic(src, &fns[0]);
        assert_eq!(m.cyclomatic, 1);
        assert_eq!(m.cognitive, 0);
        // nesting_max: the "function" line itself increments nesting, so max is 1.
        assert!(m.nesting_max >= 1);
    }

    #[test]
    fn test_estimate_complexity_heuristic_if_branch_raises_cyclomatic() {
        let a = LuaAnalyzer;
        let src = "function with_if(x)\n  if x > 0 then\n    return 1\n  end\nend\n";
        let fns = a.extract_functions_heuristic(src);
        let m = a.estimate_complexity_heuristic(src, &fns[0]);
        // +1 for `if`, baseline 1 → cyclomatic ≥ 2
        assert!(m.cyclomatic >= 2, "got {}", m.cyclomatic);
    }

    #[test]
    fn test_estimate_complexity_heuristic_and_or_add_to_cyclomatic() {
        let a = LuaAnalyzer;
        let src = "function combined(x, y)\n  if x and y then\n    return 1\n  end\nend\n";
        let fns = a.extract_functions_heuristic(src);
        let m = a.estimate_complexity_heuristic(src, &fns[0]);
        // `if` +1 and `and` +1 → cyclomatic ≥ 3
        assert!(m.cyclomatic >= 3, "got {}", m.cyclomatic);
    }

    #[test]
    fn test_estimate_complexity_heuristic_nested_for_raises_nesting() {
        let a = LuaAnalyzer;
        let src = "function nested()\n  for i = 1, 10 do\n    for j = 1, 10 do\n      x = i + j\n    end\n  end\nend\n";
        let fns = a.extract_functions_heuristic(src);
        let m = a.estimate_complexity_heuristic(src, &fns[0]);
        // Nesting: function (1), outer for (2), inner for (3)
        assert!(m.nesting_max >= 3, "got {}", m.nesting_max);
    }

    // --- LanguageAnalyzer trait dispatch ---

    #[test]
    fn test_lua_trait_extract_functions_finds_functions() {
        let a = LuaAnalyzer;
        let src = "function foo()\n  return 1\nend\n\nlocal function bar()\n  return 2\nend\n";
        let fns = a.extract_functions(src);
        assert!(fns.iter().any(|f| f.name == "foo"));
        assert!(fns.iter().any(|f| f.name == "bar"));
    }

    #[test]
    fn test_lua_trait_estimate_complexity_returns_nonzero() {
        let a = LuaAnalyzer;
        let src = "function complex(x)\n  if x > 0 then\n    return x\n  else\n    return -x\n  end\nend\n";
        let fns = a.extract_functions(src);
        assert!(!fns.is_empty());
        let m = a.estimate_complexity(src, &fns[0]);
        // `if` + `else` may or may not count depending on path — cyclomatic should be >= 2.
        assert!(m.cyclomatic >= 2, "got {}", m.cyclomatic);
        assert!(m.lines > 0);
    }

    #[test]
    fn test_lua_trait_handles_empty_input_gracefully() {
        let a = LuaAnalyzer;
        assert!(a.extract_functions("").is_empty());
    }

    #[test]
    fn test_lua_trait_handles_non_function_input() {
        let a = LuaAnalyzer;
        // Valid Lua with no function definitions.
        let src = "x = 1\ny = 2\nprint(x + y)\n";
        assert!(a.extract_functions(src).is_empty());
    }

    #[test]
    fn test_estimate_complexity_heuristic_while_and_repeat_increase_cyclomatic() {
        let a = LuaAnalyzer;
        let src = "function loops()\n  while x > 0 do\n    x = x - 1\n  end\n  repeat\n    y = y + 1\n  until y > 10\nend\n";
        let fns = a.extract_functions_heuristic(src);
        let m = a.estimate_complexity_heuristic(src, &fns[0]);
        // baseline 1 + while +1 + repeat +1 = 3
        assert!(m.cyclomatic >= 3, "got {}", m.cyclomatic);
    }

    #[test]
    fn test_estimate_complexity_heuristic_elseif_bumps_cyclomatic() {
        let a = LuaAnalyzer;
        let src = "function ladder(x)\n  if x == 1 then\n    return 1\n  elseif x == 2 then\n    return 2\n  end\nend\n";
        let fns = a.extract_functions_heuristic(src);
        let m = a.estimate_complexity_heuristic(src, &fns[0]);
        // if +1, elseif +1 → cyclomatic ≥ 3.
        assert!(m.cyclomatic >= 3, "got {}", m.cyclomatic);
    }
}
