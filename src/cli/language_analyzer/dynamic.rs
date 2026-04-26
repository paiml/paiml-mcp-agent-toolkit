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

    // ── SqlAnalyzer (Wave 39 PR13 — dynamic_sql.rs) ─────────────────────────
    //
    // Targets dynamic_sql.rs (122 missed, 0% pre-wave). SqlAnalyzer extracts
    // CREATE FUNCTION / PROCEDURE / VIEW / etc., plus CTEs from WITH ... AS.

    #[test]
    fn test_sql_extract_create_function() {
        let a = SqlAnalyzer;
        let src = "CREATE FUNCTION add(a int, b int) RETURNS int AS $$\nBEGIN\n  RETURN a + b;\nEND;\n$$ LANGUAGE plpgsql;\n";
        let fns = a.extract_functions(src);
        assert!(!fns.is_empty(), "expected to find CREATE FUNCTION");
        assert!(fns.iter().any(|f| f.name.to_lowercase().contains("add")));
    }

    #[test]
    fn test_sql_extract_create_procedure() {
        let a = SqlAnalyzer;
        let src = "CREATE PROCEDURE update_users()\nLANGUAGE plpgsql\nAS $$\nBEGIN\n  UPDATE users SET active = true;\nEND;\n$$;\n";
        let fns = a.extract_functions(src);
        assert!(!fns.is_empty());
    }

    #[test]
    fn test_sql_extract_create_view() {
        let a = SqlAnalyzer;
        let src = "CREATE VIEW active_users AS\nSELECT * FROM users WHERE active = true;\n";
        let fns = a.extract_functions(src);
        assert!(!fns.is_empty());
    }

    #[test]
    fn test_sql_extract_create_trigger() {
        let a = SqlAnalyzer;
        let src = "CREATE TRIGGER audit_trigger BEFORE UPDATE ON users\nFOR EACH ROW EXECUTE FUNCTION audit_log();\n";
        let fns = a.extract_functions(src);
        assert!(!fns.is_empty());
    }

    #[test]
    fn test_sql_extract_with_cte_simple() {
        let a = SqlAnalyzer;
        let src = "WITH active_users AS (\n  SELECT * FROM users WHERE active = true\n)\nSELECT * FROM active_users;\n";
        let fns = a.extract_functions(src);
        // PIN: WITH-clause CTE names are extracted as functions.
        assert!(
            fns.iter().any(|f| f.name.contains("active_users")),
            "expected CTE 'active_users' in extracted functions, got: {:?}",
            fns.iter().map(|f| &f.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_sql_extract_pure_select_no_function_definitions() {
        let a = SqlAnalyzer;
        let src = "SELECT id, name FROM users WHERE active = 1;\n";
        let fns = a.extract_functions(src);
        // PIN: a plain SELECT defines no function/procedure/view/trigger/CTE.
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert!(
            fns.is_empty(),
            "plain SELECT should yield no functions, got names: {names:?}"
        );
    }

    #[test]
    fn test_sql_estimate_complexity_simple() {
        let a = SqlAnalyzer;
        let src = "CREATE FUNCTION simple() RETURNS int AS $$\nBEGIN\n  RETURN 1;\nEND;\n$$ LANGUAGE plpgsql;\n";
        let fns = a.extract_functions(src);
        if !fns.is_empty() {
            let m = a.estimate_complexity(src, &fns[0]);
            assert!(m.cyclomatic >= 1);
        }
    }

    #[test]
    fn test_sql_estimate_complexity_with_branches() {
        let a = SqlAnalyzer;
        let src = r#"
CREATE FUNCTION branchy(x int) RETURNS int AS $$
BEGIN
  IF x > 0 THEN
    RETURN 1;
  ELSIF x = 0 THEN
    RETURN 0;
  ELSE
    RETURN -1;
  END IF;
END;
$$ LANGUAGE plpgsql;
"#;
        let fns = a.extract_functions(src);
        if !fns.is_empty() {
            let m = a.estimate_complexity(src, &fns[0]);
            assert!(m.cyclomatic > 1, "branchy SQL should have cyclomatic > 1");
        }
    }

    #[test]
    fn test_sql_extract_empty_source() {
        let a = SqlAnalyzer;
        let fns = a.extract_functions("");
        assert!(fns.is_empty());
    }

    // ── ScalaAnalyzer (Wave 39 PR16 — dynamic_scala.rs) ─────────────────────
    //
    // Targets dynamic_scala.rs (94 missed, 0% pre-wave). ScalaAnalyzer
    // extract_scala_name handles 9 prefixes (def, override def, private def,
    // protected def, class, case class, abstract class, object, trait).

    #[test]
    fn test_scala_extract_def() {
        let a = ScalaAnalyzer;
        let src = "def add(a: Int, b: Int): Int = a + b\n";
        let fns = a.extract_functions(src);
        assert!(!fns.is_empty());
        assert_eq!(fns[0].name, "add");
    }

    #[test]
    fn test_scala_extract_override_def() {
        let a = ScalaAnalyzer;
        let src = "override def speak(): String = \"woof\"\n";
        let fns = a.extract_functions(src);
        assert!(!fns.is_empty());
        assert_eq!(fns[0].name, "speak");
    }

    #[test]
    fn test_scala_extract_private_protected_def() {
        let a = ScalaAnalyzer;
        let src = "private def secret(): Unit = ()\nprotected def helper(x: Int): Int = x\n";
        let fns = a.extract_functions(src);
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"secret"), "got: {names:?}");
        assert!(names.contains(&"helper"), "got: {names:?}");
    }

    #[test]
    fn test_scala_extract_class_and_case_class() {
        let a = ScalaAnalyzer;
        let src = "class User(name: String) { }\ncase class Item(id: Int)\n";
        let fns = a.extract_functions(src);
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"User"));
        assert!(names.contains(&"Item"));
    }

    #[test]
    fn test_scala_extract_abstract_class() {
        let a = ScalaAnalyzer;
        let src = "abstract class Shape { def area(): Double }\n";
        let fns = a.extract_functions(src);
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"Shape"));
    }

    #[test]
    fn test_scala_extract_object_and_trait() {
        let a = ScalaAnalyzer;
        let src = "object Greeter { }\ntrait Animal { def speak(): String }\n";
        let fns = a.extract_functions(src);
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"Greeter"));
        assert!(names.contains(&"Animal"));
    }

    #[test]
    fn test_scala_extract_empty_source() {
        let a = ScalaAnalyzer;
        let fns = a.extract_functions("");
        assert!(fns.is_empty());
    }

    #[test]
    fn test_scala_extract_no_definitions() {
        // PIN: a simple expression with no def/class/object/trait yields no functions.
        let a = ScalaAnalyzer;
        let src = "val x = 42\nval y = x + 1\n";
        let fns = a.extract_functions(src);
        assert!(fns.is_empty(), "val-only code should have no functions");
    }

    #[test]
    fn test_scala_estimate_complexity_simple_def() {
        let a = ScalaAnalyzer;
        let src = "def simple(): Int = 42\n";
        let fns = a.extract_functions(src);
        if !fns.is_empty() {
            let m = a.estimate_complexity(src, &fns[0]);
            assert_eq!(m.cyclomatic, 1, "simple def should be cyclomatic 1");
        }
    }

    #[test]
    fn test_scala_estimate_complexity_with_if_match() {
        // PIN: cyclomatic increments for `if `, `case `, `while `, `for `, `catch `.
        let a = ScalaAnalyzer;
        let src = r#"def classify(x: Int): String = {
  if (x > 0) {
    x match {
      case 1 => "one"
      case _ => "other"
    }
  } else {
    "non-positive"
  }
}
"#;
        let fns = a.extract_functions(src);
        if !fns.is_empty() {
            let m = a.estimate_complexity(src, &fns[0]);
            // base 1 + if + 2 case = 4 minimum
            assert!(m.cyclomatic >= 4, "branchy got {}", m.cyclomatic);
        }
    }

    #[test]
    fn test_scala_estimate_complexity_with_logical_ops() {
        // PIN: a line containing `&&` OR `||` (or BOTH) increments cyclomatic
        // by ONE total — the check is `if a.contains("&&") || a.contains("||")`,
        // not "+1 per occurrence". This understates Boolean-chain complexity
        // but is pinned to prevent silent regressions.
        let a = ScalaAnalyzer;
        let src = "def check(x: Int, y: Int): Boolean = x > 0 && y > 0 || x == y\n";
        let fns = a.extract_functions(src);
        if !fns.is_empty() {
            let m = a.estimate_complexity(src, &fns[0]);
            // base 1 + (&& or ||) +1 = 2 (NOT 3)
            assert_eq!(m.cyclomatic, 2, "PIN: && and || on one line counted as +1");
        }
    }

    #[test]
    fn test_scala_estimate_complexity_logical_ops_per_line_increment() {
        // Multiple lines each with logical ops DO increment per line.
        let a = ScalaAnalyzer;
        let src = "def check(x: Int): Boolean = {\n  val a = x > 0 && x < 10\n  val b = x == 0 || x == 100\n  a && b\n}\n";
        let fns = a.extract_functions(src);
        if !fns.is_empty() {
            let m = a.estimate_complexity(src, &fns[0]);
            // 3 lines with logical ops → +3 cyclomatic, base 1 = 4
            assert!(m.cyclomatic >= 4, "got {}", m.cyclomatic);
        }
    }

    #[test]
    fn test_scala_estimate_complexity_nesting_tracked() {
        let a = ScalaAnalyzer;
        let src = r#"def deeply(x: Int): Int = {
  if (x > 0) {
    if (x > 10) {
      if (x > 100) {
        100
      } else {
        x
      }
    } else {
      0
    }
  } else {
    -1
  }
}
"#;
        let fns = a.extract_functions(src);
        if !fns.is_empty() {
            let m = a.estimate_complexity(src, &fns[0]);
            assert!(
                m.nesting_max >= 3,
                "expected nesting >= 3, got {}",
                m.nesting_max
            );
        }
    }
}
