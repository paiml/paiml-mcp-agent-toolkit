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
        debug_assert!(!content.is_empty(), "content must not be empty");
        #[cfg(feature = "lua-ast")]
        {
            if let Some(fns) = self.extract_functions_treesitter(content) {
                return fns;
            }
        }
        self.extract_functions_heuristic(content)
    }

    fn estimate_complexity(&self, content: &str, function: &FunctionInfo) -> ComplexityMetrics {
        debug_assert!(!content.is_empty(), "content must not be empty");
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
