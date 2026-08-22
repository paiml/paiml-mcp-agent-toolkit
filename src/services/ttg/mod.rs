//! TTG — the Token-Tree Grade base measures.
//!
//! The incumbent complexity scanner counts tokens per **line**, so
//! `rustfmt.toml` decides the grade: `if a && b && c` scores 1 and the same
//! expression wrapped over three lines scores 3. It also reads control-flow
//! keywords inside string literals, which is why `generate_trigram_index` in
//! `build.rs` — codegen whose body is one raw string containing Rust source —
//! is scored `cc = 12` by a scanner that is looking at a string.
//!
//! TTG walks tokens. It never sees a newline, a comment, or the inside of a
//! string. This module is phase one: the tokenizer and the two base measures
//! only. It does not touch the incumbent scorer.
//!
//! ```text
//!   T   tokens, after comments and attribute spans are removed
//!   D   decision points, Campbell-shaped: case-collapsed and
//!       boolean-run-collapsed
//!   N   max control-flow nesting depth (diagnostic; it does not score)
//! ```
//!
//! # Example
//!
//! ```
//! use pmat::services::ttg::measure;
//!
//! // One dispatch, not one per arm.
//! let m = measure(r#"fn f(s: &str) -> u8 { match s { "a" => 1, "b" => 2, _ => 0 } }"#);
//! assert_eq!(m.decisions, 1);
//!
//! // A run of like operators charges once, however it is wrapped: both of
//! // these are 2 — one for the `if`, one for the whole `&&` run — where the
//! // incumbent line scanner charged the wrapped form 3.
//! assert_eq!(measure("fn f() { if a && b && c {} }").decisions, 2);
//! assert_eq!(measure("fn f() {\n if a\n && b\n && c {}\n}").decisions, 2);
//! ```

pub mod lexer;
pub mod score;
mod walk;

pub use lexer::{lex, strip_attrs, Tok, TokKind, PUNCT};

/// The base measures of one definition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenMeasures {
    /// `T` — tokens, after comments and attribute spans are removed. Each
    /// string and char literal counts as exactly one.
    pub tokens: u32,
    /// `D` — decision points.
    pub decisions: u32,
    /// `N` — maximum control-flow nesting depth. Diagnostic only.
    pub max_nesting: u32,
}

/// Measure Rust source.
///
/// `source` is a definition's own text. This never fails: unbalanced
/// delimiters and truncated chunks yield whatever the scan reached, so the
/// caller decides what to do about truncation rather than losing the row.
pub fn measure(source: &str) -> TokenMeasures {
    let toks = strip_attrs(&lex(source));
    let (decisions, max_nesting) = walk::walk_rust(&toks);
    TokenMeasures {
        tokens: toks.len() as u32,
        decisions,
        max_nesting,
    }
}

/// Measure C-family source (C, C++, TypeScript, JavaScript, Python, Lua).
///
/// Same tokenizer, but the decision rule is the flat one: every branching
/// keyword and every short-circuit operator charges, with no run collapsing.
/// `max_nesting` is always 0 — this path does not compute it.
pub fn measure_c_family(source: &str) -> TokenMeasures {
    let toks = strip_attrs(&lex(source));
    TokenMeasures {
        tokens: toks.len() as u32,
        decisions: walk::walk_c_family(&toks),
        max_nesting: 0,
    }
}

#[cfg(test)]
mod differential;
#[cfg(test)]
mod tests;
