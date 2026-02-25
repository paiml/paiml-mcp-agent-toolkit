// Tests for Lean 4 language support
// Unit tests for AST extraction, sorry detection, theorem counting, and complexity analysis
// Property-based tests for namespace handling, sorry counting, and complexity bounds

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
        assert_eq!(
            functions.len(),
            4,
            "Should extract axiom, opaque, abbrev, instance"
        );

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

        assert!(
            cyclomatic >= 1,
            "Should have at least cyclomatic complexity of 1"
        );
        assert!(
            cognitive >= 1,
            "Should have at least cognitive complexity of 1"
        );
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
        assert_eq!(
            functions[0], "Foo::a",
            "First def should be in Foo namespace"
        );
        assert_eq!(
            functions[1], "Bar::b",
            "Second def should be in Bar namespace"
        );
    }

    #[test]
    fn test_sorry_in_identifier_not_counted() {
        let source = "def sorry_helper := 42\ndef no_sorry_here := 0";
        let count = count_sorry(source);
        assert_eq!(
            count, 0,
            "sorry as substring of identifier should not be counted"
        );
    }

    #[test]
    fn test_sorry_in_inline_block_comment_not_counted() {
        let source = "/- sorry -/ theorem real : True := by trivial";
        let count = count_sorry(source);
        assert_eq!(
            count, 0,
            "sorry inside inline block comment should not be counted"
        );
    }

    #[test]
    fn test_sorry_after_inline_block_comment_counted() {
        let source = "/- comment -/ sorry";
        let count = count_sorry(source);
        assert_eq!(
            count, 1,
            "sorry after inline block comment should be counted"
        );
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
