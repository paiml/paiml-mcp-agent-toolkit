//! `#[cfg(...)]` as a value, and the three-valued logic needed to decide it.
//!
//! Two-valued evaluation is the trap here. A predicate this module does not
//! understand is not `false` — `false` would silently declare the test unrun,
//! and `true` would silently declare it run. Either way a wrong answer is
//! indistinguishable from a right one. `Tri::Unknown` propagates instead, and
//! the caller reports it as a finding rather than folding it into a verdict.

use std::collections::BTreeSet;

/// A `cfg` predicate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CfgExpr {
    /// No `#[cfg]` at all — always compiled.
    True,
    /// `feature = "x"`.
    Feature(String),
    /// A bare identifier: `test`, `unix`, `kani`, …
    Flag(String),
    /// `key = "value"`: `target_os = "linux"`, …
    KeyValue(String, String),
    Not(Box<CfgExpr>),
    All(Vec<CfgExpr>),
    Any(Vec<CfgExpr>),
    /// Syntactically present but not understood by this parser. Never treated
    /// as either polarity.
    Unparsed(String),
}

/// Three-valued because "we could not tell" is a distinct, reportable answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    True,
    False,
    Unknown,
}

/// The compilation environment one CI leg presents.
///
/// `features` is the *resolved closure*, not the flags typed on the command
/// line: `--features full` enables `all-languages`, which enables `wasm-ast`,
/// and a test gated on `wasm-ast` is run by that leg.
#[derive(Debug, Clone)]
pub struct Env {
    pub features: BTreeSet<String>,
}

/// Non-feature predicates this analysis is willing to decide, and why.
///
/// Every CI leg that runs tests in this repository is `ubuntu-latest`,
/// x86-64, `cargo test` (so `cfg(test)` is on and the `test` profile keeps
/// `debug_assertions`). Anything outside this table is `Unknown` — the table
/// is an allowlist, not a default.
fn decide_flag(name: &str) -> Tri {
    match name {
        "test" | "unix" | "debug_assertions" => Tri::True,
        // Set by tooling that no test-running leg uses.
        "windows" | "doc" | "doctest" | "miri" | "kani" | "coverage" | "coverage_nightly"
        | "tarpaulin" | "llvm_cov" | "docsrs" | "cargo_publish" | "fuzzing" | "clippy"
        | "rustfmt" | "loom" | "madsim" => Tri::False,
        _ => Tri::Unknown,
    }
}

fn decide_kv(key: &str, value: &str) -> Tri {
    let matches = match key {
        "target_os" => value == "linux",
        "target_family" => value == "unix",
        "target_arch" => value == "x86_64",
        "target_pointer_width" => value == "64",
        "target_endian" => value == "little",
        "target_env" => value == "gnu",
        "panic" => value == "unwind",
        // Enabled only by an explicit `-C target-feature`, which no leg sets.
        "target_feature" => false,
        "target_vendor" => value == "unknown",
        _ => return Tri::Unknown,
    };
    if matches {
        Tri::True
    } else {
        Tri::False
    }
}

impl CfgExpr {
    /// Conjoin two predicates, flattening so the ledger's bucket labels stay
    /// stable under refactors that only move a `#[cfg]` up or down a level.
    #[must_use]
    pub fn and(self, other: CfgExpr) -> CfgExpr {
        match (self, other) {
            (CfgExpr::True, b) => b,
            (a, CfgExpr::True) => a,
            (CfgExpr::All(mut a), CfgExpr::All(b)) => {
                a.extend(b);
                CfgExpr::All(a)
            }
            (CfgExpr::All(mut a), b) => {
                a.push(b);
                CfgExpr::All(a)
            }
            (a, CfgExpr::All(b)) => {
                let mut v = vec![a];
                v.extend(b);
                CfgExpr::All(v)
            }
            (a, b) => CfgExpr::All(vec![a, b]),
        }
    }

    /// Evaluate under one leg's environment.
    #[must_use]
    pub fn eval(&self, env: &Env) -> Tri {
        match self {
            CfgExpr::True => Tri::True,
            CfgExpr::Feature(f) => {
                if env.features.contains(f) {
                    Tri::True
                } else {
                    Tri::False
                }
            }
            CfgExpr::Flag(n) => decide_flag(n),
            CfgExpr::KeyValue(k, v) => decide_kv(k, v),
            CfgExpr::Unparsed(_) => Tri::Unknown,
            CfgExpr::Not(inner) => match inner.eval(env) {
                Tri::True => Tri::False,
                Tri::False => Tri::True,
                Tri::Unknown => Tri::Unknown,
            },
            CfgExpr::All(v) => fold_all(v, env),
            CfgExpr::Any(v) => fold_any(v, env),
        }
    }

    /// Every `feature = "…"` named anywhere in the predicate, in any polarity.
    pub fn features(&self, out: &mut BTreeSet<String>) {
        match self {
            CfgExpr::Feature(f) => {
                out.insert(f.clone());
            }
            CfgExpr::Not(i) => i.features(out),
            CfgExpr::All(v) | CfgExpr::Any(v) => {
                for e in v {
                    e.features(out);
                }
            }
            _ => {}
        }
    }

    /// Every `feature = "…"` named under an EVEN number of `not(…)`.
    ///
    /// Polarity matters for the ledger label: a test gated on
    /// `not(feature = "x")` is not unrun *because of* `x` — the absence of `x`
    /// is what compiles it. Only positively-required features can be the reason
    /// nothing built the test.
    pub fn positive_features(&self, negated: bool, out: &mut BTreeSet<String>) {
        match self {
            CfgExpr::Feature(f) if !negated => {
                out.insert(f.clone());
            }
            CfgExpr::Not(i) => i.positive_features(!negated, out),
            CfgExpr::All(v) | CfgExpr::Any(v) => {
                for e in v {
                    e.positive_features(negated, out);
                }
            }
            _ => {}
        }
    }

    /// Every `feature = "…"` named under an ODD number of `not(…)`.
    ///
    /// A test gated on `not(feature = "viz")` compiles only where `viz` is OFF.
    /// Every leg here has it ON, so the reason nothing runs the test is the
    /// PRESENCE of a feature, not the absence of one — a distinct answer that
    /// `<environment>` would have hidden.
    pub fn negated_features(&self, negated: bool, out: &mut BTreeSet<String>) {
        match self {
            CfgExpr::Feature(f) if negated => {
                out.insert(f.clone());
            }
            CfgExpr::Not(i) => i.negated_features(!negated, out),
            CfgExpr::All(v) | CfgExpr::Any(v) => {
                for e in v {
                    e.negated_features(negated, out);
                }
            }
            _ => {}
        }
    }

    /// Render for a ledger row or an error message.
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            CfgExpr::True => "true".to_string(),
            CfgExpr::Feature(f) => format!("feature = \"{f}\""),
            CfgExpr::Flag(n) => n.clone(),
            CfgExpr::KeyValue(k, v) => format!("{k} = \"{v}\""),
            CfgExpr::Unparsed(s) => format!("<unparsed: {s}>"),
            CfgExpr::Not(i) => format!("not({})", i.render()),
            CfgExpr::All(v) => format!("all({})", render_list(v)),
            CfgExpr::Any(v) => format!("any({})", render_list(v)),
        }
    }
}

fn render_list(v: &[CfgExpr]) -> String {
    v.iter().map(CfgExpr::render).collect::<Vec<_>>().join(", ")
}

fn fold_all(v: &[CfgExpr], env: &Env) -> Tri {
    let mut unknown = false;
    for e in v {
        match e.eval(env) {
            Tri::False => return Tri::False,
            Tri::Unknown => unknown = true,
            Tri::True => {}
        }
    }
    if unknown {
        Tri::Unknown
    } else {
        Tri::True
    }
}

fn fold_any(v: &[CfgExpr], env: &Env) -> Tri {
    let mut unknown = false;
    for e in v {
        match e.eval(env) {
            Tri::True => return Tri::True,
            Tri::Unknown => unknown = true,
            Tri::False => {}
        }
    }
    if unknown {
        Tri::Unknown
    } else {
        Tri::False
    }
}

/// Is there ANY feature assignment that compiles this item?
///
/// Two shapes in this tree answer no, and both are real defects a
/// feature-set-difference check would have labelled as merely "needs a flag":
///
/// * `all(feature = "demo", not(feature = "demo"))` — 6 tests, contradictory;
/// * `all(feature = "mutation-testing", any())` — 4 tests, and `any()` with no
///   arguments is `false` by definition.
///
/// Features are treated as free variables, which over-approximates: this
/// returns `false` only when no assignment at all works, never merely because
/// the feature graph makes a combination unreachable. Over-approximating is the
/// safe direction — it can miss an unsatisfiable predicate, it cannot invent
/// one.
#[must_use]
pub fn satisfiable(e: &CfgExpr) -> bool {
    let mut names = BTreeSet::new();
    e.features(&mut names);
    let names: Vec<String> = names.into_iter().collect();
    // 2^12 assignments is 4096 evaluations of a tiny tree; beyond that, decline
    // to answer rather than spend the time, and call it satisfiable.
    if names.len() > 12 {
        return true;
    }
    (0u32..(1u32 << names.len())).any(|mask| {
        let features = names
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, n)| n.clone())
            .collect();
        e.eval(&Env { features }) == Tri::True
    })
}

/// Build a `CfgExpr` from the arguments of one `#[cfg(...)]` attribute.
#[must_use]
pub fn from_meta(meta: &syn::Meta) -> CfgExpr {
    match meta {
        syn::Meta::Path(p) => match p.get_ident() {
            Some(i) => CfgExpr::Flag(i.to_string()),
            None => CfgExpr::Unparsed(quote_path(p)),
        },
        syn::Meta::NameValue(nv) => from_name_value(nv),
        syn::Meta::List(list) => from_list(list),
    }
}

fn from_name_value(nv: &syn::MetaNameValue) -> CfgExpr {
    let Some(key) = nv.path.get_ident().map(|i| i.to_string()) else {
        return CfgExpr::Unparsed(quote_path(&nv.path));
    };
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.value
    else {
        return CfgExpr::Unparsed(key);
    };
    if key == "feature" {
        CfgExpr::Feature(s.value())
    } else {
        CfgExpr::KeyValue(key, s.value())
    }
}

fn from_list(list: &syn::MetaList) -> CfgExpr {
    let Some(op) = list.path.get_ident().map(|i| i.to_string()) else {
        return CfgExpr::Unparsed(quote_path(&list.path));
    };
    let parsed = list.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    );
    let args: Vec<CfgExpr> = match parsed {
        // `any()` and `all()` are legal and empty; an empty token stream is not
        // a parse failure.
        Ok(items) => items.iter().map(from_meta).collect(),
        Err(_) if list.tokens.is_empty() => Vec::new(),
        Err(e) => return CfgExpr::Unparsed(format!("{op}(…): {e}")),
    };
    match op.as_str() {
        "all" => CfgExpr::All(args),
        "any" => CfgExpr::Any(args),
        // A one-element array conversion binds the sole argument directly.
        // `not` with any other arity is unparseable, which is the same verdict
        // the catch-all arm gives it.
        "not" => match <[CfgExpr; 1]>::try_from(args) {
            Ok([inner]) => CfgExpr::Not(Box::new(inner)),
            Err(_) => CfgExpr::Unparsed(op),
        },
        _ => CfgExpr::Unparsed(op),
    }
}

fn quote_path(p: &syn::Path) -> String {
    p.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Extract the conjunction of every `#[cfg(...)]` on one item.
///
/// `#[cfg_attr(...)]` is deliberately ignored: it applies an *attribute*
/// conditionally, it does not remove the item from the build.
#[must_use]
pub fn of_attrs(attrs: &[syn::Attribute]) -> CfgExpr {
    let mut acc = CfgExpr::True;
    for a in attrs {
        if !a.path().is_ident("cfg") {
            continue;
        }
        let e = match a.parse_args::<syn::Meta>() {
            Ok(m) => from_meta(&m),
            Err(err) => CfgExpr::Unparsed(err.to_string()),
        };
        acc = acc.and(e);
    }
    acc
}
