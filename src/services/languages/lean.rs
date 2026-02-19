#![cfg_attr(coverage_nightly, coverage(off))]
//! Lean 4 Language Support for PMAT
//!
//! This module provides Lean 4-specific analysis capabilities using pattern-based parsing
//! for AST extraction and proof quality metrics.
//!
//! Lean 4 constructs: def, theorem, lemma, structure, class, inductive, abbrev,
//! axiom, opaque, instance, namespace.

#[cfg(feature = "lean-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "lean-ast")]
use std::path::{Path, PathBuf};

/// Lean 4 AST visitor that extracts Lean-specific AST information
#[cfg(feature = "lean-ast")]
pub struct LeanAstVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    namespace: String,
}

#[cfg(feature = "lean-ast")]
impl LeanAstVisitor {
    /// Creates a new Lean AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            namespace: String::new(),
        }
    }

    /// Analyzes Lean 4 source code and extracts AST items (single-pass, complexity ≤10)
    pub fn analyze_lean_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Track namespace changes inline (single-pass)
            if trimmed.starts_with("namespace ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    self.namespace = parts[1].to_string();
                }
                continue;
            }

            self.extract_line(trimmed, line_num);
        }

        Ok(self.items)
    }

    /// Extracts AST items from a single line (complexity ≤10)
    fn extract_line(&mut self, trimmed: &str, line_num: usize) {
        // Try definitions first
        if let Some(item) = self.try_extract_definition(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try theorems/lemmas
        if let Some(item) = self.try_extract_theorem(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try types (structure, class, inductive)
        if let Some(item) = self.try_extract_type(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try instances
        if let Some(item) = self.try_extract_instance(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try axioms/opaque
        if let Some(item) = self.try_extract_axiom(trimmed, line_num) {
            self.items.push(item);
        }
    }

    /// Tries to extract a def/abbrev from a line (complexity ≤10)
    fn try_extract_definition(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("noncomputable def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("abbrev ") {
            Self::extract_name_after_keyword(trimmed, "abbrev")
        } else if trimmed.starts_with("partial def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("private def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("protected def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        let visibility = if trimmed.starts_with("private ") {
            "private"
        } else {
            "public"
        };
        Some(AstItem::Function {
            name: qualified,
            visibility: visibility.to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Tries to extract a theorem/lemma from a line (complexity ≤10)
    fn try_extract_theorem(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("theorem ") {
            Self::extract_name_after_keyword(trimmed, "theorem")
        } else if trimmed.starts_with("lemma ") {
            Self::extract_name_after_keyword(trimmed, "lemma")
        } else if trimmed.starts_with("private theorem ") {
            Self::extract_name_after_keyword(trimmed, "theorem")
        } else if trimmed.starts_with("private lemma ") {
            Self::extract_name_after_keyword(trimmed, "lemma")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Function {
            name: qualified,
            visibility: "public".to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Tries to extract a structure/class/inductive from a line (complexity ≤10)
    fn try_extract_type(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("structure ") {
            Self::extract_name_after_keyword(trimmed, "structure")
        } else if trimmed.starts_with("class ") {
            Self::extract_name_after_keyword(trimmed, "class")
        } else if trimmed.starts_with("inductive ") {
            Self::extract_name_after_keyword(trimmed, "inductive")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Struct {
            name: qualified,
            visibility: "public".to_string(),
            fields_count: 0,
            derives: vec![],
            line: line_num + 1,
        })
    }

    /// Tries to extract an instance from a line (complexity ≤10)
    fn try_extract_instance(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        if !trimmed.starts_with("instance ") {
            return None;
        }
        let name = Self::extract_name_after_keyword(trimmed, "instance")?;
        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Function {
            name: qualified,
            visibility: "public".to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Tries to extract an axiom/opaque from a line (complexity ≤10)
    fn try_extract_axiom(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("axiom ") {
            Self::extract_name_after_keyword(trimmed, "axiom")
        } else if trimmed.starts_with("opaque ") {
            Self::extract_name_after_keyword(trimmed, "opaque")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Function {
            name: qualified,
            visibility: "public".to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Extracts name after a keyword (complexity ≤10)
    fn extract_name_after_keyword(line: &str, keyword: &str) -> Option<String> {
        // Find the keyword position and get the word after it
        if let Some(pos) = line.find(keyword) {
            let after = &line[pos + keyword.len()..].trim_start();
            let name = after
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .next()
                .unwrap_or("");
            if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets qualified name for a symbol (complexity ≤10)
    fn get_qualified_name(&self, name: &str) -> String {
        if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.namespace, name)
        }
    }
}

/// Count sorry occurrences in Lean source (proof incompleteness metric)
///
/// Handles: line comments (--), nested block comments (/- ... -/),
/// and word-boundary checking to avoid false positives from identifiers
/// containing "sorry" as a substring.
#[cfg(feature = "lean-ast")]
pub fn count_sorry(source: &str) -> usize {
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
#[cfg(feature = "lean-ast")]
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
#[cfg(feature = "lean-ast")]
fn contains_sorry_word(line: &str) -> bool {
    let bytes = line.as_bytes();
    let sorry = b"sorry";

    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok =
                abs_idx == 0 || !is_ident_char(bytes[abs_idx - 1]);
            let after_ok =
                abs_idx + sorry.len() >= bytes.len() || !is_ident_char(bytes[abs_idx + sorry.len()]);
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
#[cfg(feature = "lean-ast")]
fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Count theorem and lemma declarations in Lean source
#[cfg(feature = "lean-ast")]
pub fn count_theorems(source: &str) -> usize {
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

/// Lean complexity analyzer for extracting Lean-specific metrics (complexity ≤10)
#[cfg(feature = "lean-ast")]
pub struct LeanComplexityAnalyzer {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
}

#[cfg(feature = "lean-ast")]
impl Default for LeanComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "lean-ast")]
impl LeanComplexityAnalyzer {
    /// Creates a new Lean complexity analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of Lean source code (complexity ≤10)
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
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
#[cfg(feature = "lean-ast")]
pub async fn analyze_lean_file(
    path: &Path,
) -> Result<crate::services::context::FileContext, crate::models::error::TemplateError> {
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

#[cfg(all(test, feature = "lean-ast"))]
mod tests {
    use super::*;
    use std::path::Path;

    const SIMPLE_LEAN_DEF: &str = r#"
namespace MyLib

def hello : String := "Hello, World!"

def add (x y : Nat) : Nat := x + y
"#;

    const LEAN_THEOREMS: &str = r#"
namespace Nat

theorem add_comm (a b : Nat) : a + b = b + a := by
  induction a with
  | zero => simp
  | succ n ih => simp [Nat.succ_add, ih]

lemma add_zero (n : Nat) : n + 0 = n := by
  rfl
"#;

    const LEAN_TYPES: &str = r#"
structure Point where
  x : Float
  y : Float

class HasSize (α : Type) where
  size : α → Nat

inductive Tree (α : Type) where
  | leaf : Tree α
  | node : Tree α → α → Tree α → Tree α
"#;

    const LEAN_SORRY: &str = r#"
theorem hard_theorem : 1 + 1 = 2 := by
  sorry

def unfinished : Nat := sorry

theorem incomplete_proof : True := by
  sorry
"#;

    const LEAN_MIXED: &str = r#"
namespace Algebra

axiom choice : ∀ (α : Type), Nonempty α → α

opaque secretImpl : Nat → Nat

abbrev MyNat := Nat

instance natToString : ToString Nat where
  toString := Nat.repr

structure Ring where
  carrier : Type
"#;

    #[test]
    fn test_simple_lean_definitions() {
        let visitor = LeanAstVisitor::new(Path::new("test.lean"));
        let items = visitor
            .analyze_lean_source(SIMPLE_LEAN_DEF)
            .expect("Should parse Lean defs");

        let functions: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(functions.len(), 2, "Should extract two definitions");

        if let AstItem::Function { name, .. } = &functions[0] {
            assert_eq!(name, "MyLib::hello");
        }
        if let AstItem::Function { name, .. } = &functions[1] {
            assert_eq!(name, "MyLib::add");
        }
    }

    #[test]
    fn test_lean_theorems_and_lemmas() {
        let visitor = LeanAstVisitor::new(Path::new("test.lean"));
        let items = visitor
            .analyze_lean_source(LEAN_THEOREMS)
            .expect("Should parse Lean theorems");

        let functions: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(functions.len(), 2, "Should extract theorem and lemma");

        if let AstItem::Function { name, .. } = &functions[0] {
            assert_eq!(name, "Nat::add_comm");
        }
        if let AstItem::Function { name, .. } = &functions[1] {
            assert_eq!(name, "Nat::add_zero");
        }
    }

    #[test]
    fn test_lean_types() {
        let visitor = LeanAstVisitor::new(Path::new("test.lean"));
        let items = visitor
            .analyze_lean_source(LEAN_TYPES)
            .expect("Should parse Lean types");

        let structs: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();

        assert_eq!(
            structs.len(),
            3,
            "Should extract structure, class, and inductive"
        );

        if let AstItem::Struct { name, .. } = &structs[0] {
            assert_eq!(name, "Point");
        }
        if let AstItem::Struct { name, .. } = &structs[1] {
            assert_eq!(name, "HasSize");
        }
        if let AstItem::Struct { name, .. } = &structs[2] {
            assert_eq!(name, "Tree");
        }
    }

    #[test]
    fn test_sorry_detection() {
        let count = count_sorry(LEAN_SORRY);
        assert_eq!(count, 3, "Should detect 3 sorry occurrences");
    }

    #[test]
    fn test_theorem_counting() {
        let count = count_theorems(LEAN_SORRY);
        assert_eq!(count, 2, "Should count 2 theorems");
    }

    #[test]
    fn test_lean_mixed_constructs() {
        let visitor = LeanAstVisitor::new(Path::new("test.lean"));
        let items = visitor
            .analyze_lean_source(LEAN_MIXED)
            .expect("Should parse mixed Lean constructs");

        // axiom, opaque, abbrev, instance = 4 functions
        let functions: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();
        assert_eq!(functions.len(), 4, "Should extract axiom, opaque, abbrev, instance");

        // structure = 1 struct
        let structs: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();
        assert_eq!(structs.len(), 1, "Should extract Ring structure");
    }

    #[test]
    fn test_lean_complexity_analysis() {
        let mut analyzer = LeanComplexityAnalyzer::new();
        let (cyclomatic, cognitive) = analyzer
            .analyze_complexity(LEAN_THEOREMS)
            .expect("Should analyze Lean complexity");

        assert!(cyclomatic >= 1, "Should have at least cyclomatic complexity of 1");
        assert!(cognitive >= 1, "Should have at least cognitive complexity of 1");
    }

    #[test]
    fn test_empty_lean_source() {
        let visitor = LeanAstVisitor::new(Path::new("empty.lean"));
        let items = visitor
            .analyze_lean_source("")
            .expect("Should handle empty source");
        assert!(items.is_empty(), "Empty source should produce no AST items");
    }

    #[test]
    fn test_sorry_in_comment_not_counted() {
        let source = "-- sorry this is a comment\ntheorem real : True := by trivial";
        let count = count_sorry(source);
        assert_eq!(count, 0, "sorry in line comments should not be counted");
    }

    #[test]
    fn test_sorry_in_block_comment_not_counted() {
        let source = "/-!\nThis has sorry in a doc block\n-/\ntheorem real : True := by trivial";
        let count = count_sorry(source);
        assert_eq!(count, 0, "sorry in block comments should not be counted");
    }

    #[test]
    fn test_namespace_scoping() {
        let source = "namespace Foo\ndef bar : Nat := 42";
        let visitor = LeanAstVisitor::new(Path::new("test.lean"));
        let items = visitor.analyze_lean_source(source).expect("Should parse");

        if let AstItem::Function { name, .. } = &items[0] {
            assert_eq!(name, "Foo::bar", "Should have namespace-qualified name");
        }
    }

    // --- Edge case tests (Falsification 5) ---

    #[test]
    fn test_multiple_namespaces() {
        let source = "namespace Foo\ndef a : Nat := 1\nnamespace Bar\ndef b : Nat := 2";
        let visitor = LeanAstVisitor::new(Path::new("test.lean"));
        let items = visitor.analyze_lean_source(source).expect("Should parse");

        let functions: Vec<_> = items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0], "Foo::a", "First def should be in Foo namespace");
        assert_eq!(functions[1], "Bar::b", "Second def should be in Bar namespace");
    }

    #[test]
    fn test_sorry_in_identifier_not_counted() {
        let source = "def sorry_helper := 42\ndef no_sorry_here := 0";
        let count = count_sorry(source);
        assert_eq!(count, 0, "sorry as substring of identifier should not be counted");
    }

    #[test]
    fn test_sorry_in_inline_block_comment_not_counted() {
        let source = "/- sorry -/ theorem real : True := by trivial";
        let count = count_sorry(source);
        assert_eq!(count, 0, "sorry inside inline block comment should not be counted");
    }

    #[test]
    fn test_sorry_after_inline_block_comment_counted() {
        let source = "/- comment -/ sorry";
        let count = count_sorry(source);
        assert_eq!(count, 1, "sorry after inline block comment should be counted");
    }

    #[test]
    fn test_sorry_standalone_word() {
        let source = "theorem t : True := by sorry";
        let count = count_sorry(source);
        assert_eq!(count, 1, "standalone sorry should be counted");
    }
}

#[cfg(all(test, feature = "lean-ast"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_lean_visitor_handles_any_namespace(
            ns_name in "[a-zA-Z_][a-zA-Z0-9_]*"
        ) {
            let source = format!("namespace {}\ndef myFunc : Nat := 42", ns_name);
            let visitor = LeanAstVisitor::new(Path::new("test.lean"));

            if let Ok(items) = visitor.analyze_lean_source(&source) {
                prop_assert!(!items.is_empty());
                let has_ns_prefix = items.iter().any(|item| match item {
                    AstItem::Function { name, .. } => name.starts_with(&format!("{}::", ns_name)),
                    _ => false,
                });
                prop_assert!(has_ns_prefix);
            }
        }

        #[test]
        fn test_lean_sorry_count_nonnegative(
            sorry_count in 0usize..5
        ) {
            let mut source = String::new();
            for i in 0..sorry_count {
                source.push_str(&format!("theorem t{} : True := by sorry\n", i));
            }

            let count = count_sorry(&source);
            prop_assert_eq!(count, sorry_count);
        }

        #[test]
        fn test_lean_complexity_stays_bounded(
            depth in 1u32..5
        ) {
            let mut source = String::from("def complexFunc : Nat := by\n");
            for _ in 0..depth {
                source.push_str("  if true then\n");
            }
            source.push_str("  0\n");

            let mut analyzer = LeanComplexityAnalyzer::new();
            if let Ok((cyclomatic, cognitive)) = analyzer.analyze_complexity(&source) {
                prop_assert!(cyclomatic >= depth);
                prop_assert!(cognitive >= depth);
                prop_assert!(cyclomatic <= depth * 2 + 5);
                prop_assert!(cognitive <= depth * 3 + 5);
            }
        }
    }
}
