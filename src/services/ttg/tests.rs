//! Unit tests for the TTG base measures.
//!
//! Every expectation here was computed from the reference implementation
//! (`spec/tok.py`) before it was written down, and every test was watched to
//! fail under a named mutation of this module before it was kept. The
//! mutation is named in each test's doc comment, together with a
//! counter-assertion that the lazy over-correction cannot satisfy.

use super::*;

fn d(src: &str) -> u32 {
    measure(src).decisions
}

fn t(src: &str) -> u32 {
    measure(src).tokens
}

fn n(src: &str) -> u32 {
    measure(src).max_nesting
}

/// A `match` is one dispatch, not one decision per arm.
///
/// RED: charge +1 per `=>` — the first assertion reads 5.
/// Counter (over-correction: stop charging `match` at all) — the second
/// assertion reads 1 instead of 2.
#[test]
fn match_charges_one_for_the_dispatch_not_one_per_arm() {
    let lookup = r#"fn f(s: &str) -> u8 { match s { "a" => 1, "b" => 2, "c" => 3, _ => 0 } }"#;
    assert_eq!(d(lookup), 1);
    assert_eq!(
        d("fn f() { match x { A => if y { 1 } else { 2 }, _ => 0 } }"),
        2
    );
}

/// A run of like `&&`/`||` at one delimiter depth charges +1 in total, and it
/// charges the same however the source is wrapped or commented.
///
/// RED: charge +1 per operator instead of per run — `bare run` reads 2.
/// Counter (over-correction: collapse every boolean operator in the
/// definition to a single charge) — `a && b || c && d` reads 1 instead of 3,
/// and the two-statement case reads 1 instead of 2.
#[test]
fn a_boolean_run_charges_once_however_it_is_wrapped() {
    assert_eq!(d("fn f() { let z = a && b && c; }"), 1);
    assert_eq!(
        d("fn f() {\n    let z = a\n        && b\n        && c;\n}"),
        1
    );
    assert_eq!(
        d("fn f() {\n    let z = a // one\n        && b /* two */\n        && c; // three\n}"),
        1
    );
    // A different operator opens a new run.
    assert_eq!(d("fn f() { let z = a && b || c && d; }"), 3);
    // A deeper delimiter is a different depth.
    assert_eq!(d("fn f() { let z = a && (b || c); }"), 2);
    // `;` and `,` close a run; the next operator opens a new one.
    assert_eq!(d("fn f() { let x = a || b; let y = b || c; }"), 2);
    assert_eq!(d("fn f() { g(a && b, c && d); }"), 2);
}

/// Leaving and re-entering a delimiter must NOT close the run at the outer
/// depth — that is what keeps a chain of calls at one charge.
///
/// RED: clear the outer depth's run slot on `(` or on `)` — the chain reads 3
/// instead of 1.
#[test]
fn a_run_survives_a_call_in_the_middle_of_it() {
    assert_eq!(
        d(r#"fn f() { let z = p.contains("a") || p.contains("b") || p.contains("c"); }"#),
        1
    );
    // `?` and `.` are inside one expression and do not close a run either.
    assert_eq!(d("fn f() -> R { if g()? && h()? && i()? { } Ok(()) }"), 2);
}

/// `||` is boolean-or only when the previous token can end an expression.
/// Everywhere else it is a zero-argument closure and charges nothing.
///
/// RED: treat every `||` as boolean — each of the first three reads 1.
/// Counter (over-correction: treat every `||` as a closure) — each of the last
/// three reads 1 instead of 2.
#[test]
fn a_zero_argument_closure_charges_nothing() {
    assert_eq!(d("fn f() { let g = || 1; g(); }"), 0);
    assert_eq!(d("fn f() { v.unwrap_or_else(|| 0); }"), 0);
    assert_eq!(d("fn f() { std::thread::spawn(move || {}); }"), 0);
    // Real boolean-or after a closing delimiter, a method call and a `?`.
    assert_eq!(d("fn f() { if a() || b() { } }"), 2);
    assert_eq!(d("fn f() { if x.is_some() || y { } }"), 2);
    assert_eq!(d("fn f() -> R { let z = g()? || h(); Ok(()) }"), 1);
}

/// A match-arm guard is an `if` and charges. This is what makes the
/// `if` → `match` conversion cost +1 instead of paying out.
///
/// RED: skip an `if` that appears inside a `match` block — the guarded match
/// reads 1 instead of 3.
/// Counter (over-correction: charge every arm) — the unguarded match reads 4
/// instead of 1.
#[test]
fn a_match_arm_guard_charges_one() {
    assert_eq!(
        d("fn f() { match () { _ if a => 1, _ if b => 2, _ => 3 } }"),
        3
    );
    assert_eq!(d("fn f() { match () { A => 1, B => 2, _ => 3 } }"), 1);
    // The conversion the guard rule closes: the `if` form is cheaper.
    assert_eq!(d("fn f() { if a { 1 } else if b { 2 } else { 3 } }"), 2);
}

/// `let PAT = EXPR else { … }` is a divergence and charges 1. Plain `else`
/// charges nothing — the `if` it belongs to was already charged.
///
/// RED: stop looking at the statement head, so no `else` charges — the first
/// assertion reads 0.
/// Counter (over-correction: charge every `else`) — the second reads 2 and the
/// third reads 4.
#[test]
fn let_else_charges_one_and_plain_else_charges_nothing() {
    assert_eq!(d("fn f() { let Some(x) = y else { return; }; }"), 1);
    assert_eq!(d("fn f() { if a { } else { } }"), 1);
    assert_eq!(d("fn f() { if a { } else if b { } else { } }"), 2);
}

/// `?` is shorthand for an early `Err` return and charges nothing.
///
/// RED: charge `?` — the first assertion reads 2.
/// Counter (over-correction: also stop charging what surrounds it) — the
/// second reads 0 or 1 instead of 2.
#[test]
fn the_question_mark_operator_is_free() {
    assert_eq!(
        d("fn f() -> R { let a = g()?; let b = h()?; Ok(a + b) }"),
        0
    );
    assert_eq!(d("fn f() -> R { if g()? && h()? { } Ok(()) }"), 2);
}

/// A string literal is one opaque token. Control flow written INSIDE a codegen
/// string is not control flow — this is the `generate_trigram_index` false
/// positive that the incumbent line scanner scores `cc = 12`.
///
/// RED: lex a raw string as ordinary source — the codegen body reads 4.
/// Counter (over-correction: drop literal tokens entirely, or return 0
/// unconditionally) — the `T` assertions and the last `D` assertion fail.
#[test]
fn code_inside_a_string_literal_charges_nothing() {
    let codegen = r##"fn gen() -> String { let s = r#"if a && b { for x in y { match z { _ => 0 } } }"#; s.to_string() }"##;
    assert_eq!(d(codegen), 0);
    // The literal is still exactly ONE token, whatever it contains.
    assert_eq!(t(r#"fn f() { let s = "a b c d e"; }"#), 11);
    assert_eq!(t(r#"fn f() { let s = "z"; }"#), 11);
    assert_eq!(t(r##"fn f() { let b = b"abc"; let r = br#"x"#; }"##), 16);
    // Code OUTSIDE the string is still measured.
    assert_eq!(d(r#"fn f() { let s = "if a && b"; if c { } }"#), 1);
}

/// Rust block comments nest, so the scanner must track depth rather than stop
/// at the first `*/`.
///
/// RED: end a block comment at the first `*/` — the first assertion reads 1,
/// because `if a { }` falls out of the comment.
/// Counter (over-correction: run a block comment to the end of input) — the
/// second reads 0 instead of 1.
#[test]
fn a_nested_block_comment_terminates_at_the_matching_close() {
    assert_eq!(d("fn f() { /* /* x */ if a { } */ }"), 0);
    assert_eq!(d("fn f() { /* /* x */ */ if a { } }"), 1);
    assert_eq!(t("fn f() { /* /* x */ */ }"), t("fn f() { }"));
}

/// Comments and doc comments cost nothing and earn nothing.
///
/// RED: stop skipping `///` — `T` rises above the comment-free form.
#[test]
fn comments_and_doc_comments_are_not_tokens() {
    let documented = "fn f() {\n    //! inner\n    /// doc if a && b\n    let x = 1;\n}";
    assert_eq!(t(documented), t("fn f() {\n    let x = 1;\n}"));
    assert_eq!(d(documented), 0);
}

/// Attribute spans are not code the reader executes.
///
/// RED: stop stripping attribute spans — the first two assertions diverge.
/// Counter (over-correction: drop every `#`) — a raw identifier collapses and
/// the last assertion fails.
#[test]
fn attribute_spans_are_removed_but_a_bare_hash_is_kept() {
    assert_eq!(
        t("#[derive(Debug, Clone)] struct S { a: u8 }"),
        t("struct S { a: u8 }")
    );
    assert_eq!(d("#[cfg(all(a, b))] fn f() { }"), 0);
    assert_eq!(t("let r#type = 1;"), 7);
    assert_eq!(t("let type_ = 1;"), 5);
}

/// `for` is a loop unless it is the HRTB `for<'a>` or the `for` of an
/// `impl Trait for Type` header.
///
/// RED: charge every `for` — the HRTB and the impl header read 1.
/// Counter (over-correction: never charge `for`) — the loop cases read 0.
#[test]
fn for_is_a_loop_unless_it_is_hrtb_or_an_impl_header() {
    assert_eq!(d("fn f() { for x in y { } }"), 1);
    assert_eq!(d("fn f() { for i in 0..10 { } }"), 1);
    assert_eq!(d("fn f<F>(g: F) where F: for<'a> Fn(&'a str) { }"), 0);
    assert_eq!(d("impl Display for Foo { fn fmt(&self) { } }"), 0);
    // A real loop inside an impl block is still charged: the statement head
    // resets on entering the block.
    assert_eq!(d("impl Foo { fn f(&self) { for x in y { } } }"), 1);
}

/// `while`, `loop` and labelled loops each charge one.
#[test]
fn loops_charge_one_each() {
    assert_eq!(d("fn f() { while let Some(x) = it.next() { } }"), 1);
    assert_eq!(d("fn f() { 'outer: loop { break 'outer; } }"), 1);
    // `matches!` and iterator combinators are not decisions.
    assert_eq!(d("fn f() { let b = matches!(x, A | B); }"), 0);
    assert_eq!(
        d("fn f() { v.iter().filter(|x| x.a).map(|x| x.b).collect() }"),
        0
    );
}

/// `N` counts control-flow blocks that are open simultaneously, not every
/// brace.
///
/// RED: count every `{` — the flat case reads 3 and the struct literal reads 1.
#[test]
fn nesting_counts_simultaneously_open_control_blocks() {
    assert_eq!(n("fn f() { if a { if b { if c { } } } }"), 3);
    assert_eq!(n("fn f() { if a { } if b { } if c { } }"), 1);
    assert_eq!(n("fn f() { let v = S { a: 1 }; }"), 0);
    assert_eq!(n("fn f() { }"), 0);
}

/// Char literals are one token; lifetimes are identifiers. Telling them apart
/// is what keeps `'a'` and `&'a str` from derailing the scan.
///
/// RED: treat every `'` as opening a char literal — the lifetime cases lose
/// tokens and `T` drops.
#[test]
fn char_literals_are_told_apart_from_lifetimes() {
    assert_eq!(
        t("fn f() { let c = 'a'; let d = '\\n'; let e = '\\''; }"),
        21
    );
    assert_eq!(t("fn f<'a>(x: &'a str) -> &'a str { x }"), 19);
    assert_eq!(t("fn f() { let c = 'é'; }"), 11);
}

/// Punctuation is matched greedily and in table order, so `>>` is one token
/// and `..` inside a numeric span is swallowed by the number rule.
#[test]
fn punctuation_is_matched_greedily() {
    assert_eq!(t("fn f() { let v: Vec<Vec<u8>> = x >> 2; }"), 20);
    assert_eq!(
        lex("a >>= b")
            .iter()
            .filter(|k| k.kind == TokKind::Punct)
            .map(|k| k.text)
            .collect::<Vec<_>>(),
        vec![">>="]
    );
    assert_eq!(lex("x[0..10]").len(), 4);
}

/// The scan must terminate and stay sane on the truncated chunks the indexer
/// already emits — 0.75% of stored definitions have unbalanced braces.
#[test]
fn truncated_and_malformed_input_still_terminates() {
    let truncated = "fn f() { if a { while b { let s = \"unterminated";
    let m = measure(truncated);
    assert_eq!(m.decisions, 2);
    assert!(m.tokens > 0);
    // Stray closers, an unterminated block comment and a lone quote.
    assert_eq!(measure(")}]").decisions, 0);
    assert_eq!(measure("fn f() { /* never closed").tokens, 5);
    assert_eq!(measure("fn f() { let c = ' }").decisions, 0);
    assert_eq!(measure("").tokens, 0);
}

/// The C-family rule set is the flat one: every branching keyword and every
/// short-circuit operator charges, with no run collapsing and no nesting.
///
/// RED: point `measure_c_family` at the Rust walk — the count reads 2, not 6.
#[test]
fn the_c_family_rule_set_does_not_collapse_runs_or_cases() {
    let src = "int main() { if (a && b && c) { switch (x) { case 1: case 2: break; } } }";
    assert_eq!(measure_c_family(src).decisions, 6);
    assert_eq!(measure_c_family(src).max_nesting, 0);
    // Same tokens either way; only the decision rule differs.
    assert_eq!(measure_c_family(src).tokens, measure(src).tokens);
    assert_eq!(measure(src).decisions, 2);
}

/// The headline invariance property, on one definition: reformatting and
/// deleting comments move neither measure.
#[test]
fn both_measures_are_invariant_to_formatting_and_comments() {
    let wide = "fn f(a: bool, b: bool, c: bool) -> u8 { if a && b && c { 1 } else if a || c { 2 } else { 0 } }";
    let tall = "fn f(\n    a: bool,\n    b: bool,\n    c: bool\n) -> u8 {\n    // decide\n    if a\n        && b\n        && c\n    {\n        1\n    } else if a\n        || c\n    {\n        2 // two\n    } else {\n        /* nothing */\n        0\n    }\n}";
    assert_eq!(measure(wide), measure(tall));
    assert_eq!(measure(wide).decisions, 4);
}
