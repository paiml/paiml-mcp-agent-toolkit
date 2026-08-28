//! R2 stage 2 — what an annotation is *doing*.
//!
//! A number written next to another number is usually not a claim about it. It
//! quotes a superseded value, cites a spec, gives a hypothetical, itemises the
//! parts the value is built from, or approximates. Every false positive the
//! research pass hand-audited came from reading one of those as an assertion.
//!
//! So the gate is deliberately narrow: only a **same-line trailing comment**,
//! short, unquoted, unhedged and free of narration, is read as asserting
//! something about the value beside it. A preceding `///` or `#` block is
//! context — which is exactly why `.pmat-metrics.toml` at HEAD, where the
//! explanation of `max_unwrap_calls = 0` lives on the six lines above it, is
//! correctly silent.
//!
//! The 72-character bound is measured, not chosen. The longest true-positive
//! annotation across both audit repositories is 38 characters
//! (`Current: 570 (CRITICAL - must reduce!)`); the shortest narration false
//! positive is 74.

use regex::Regex;
use std::sync::LazyLock;

use super::extract::{close, NUM};

/// Longest annotation that can still be read as an assertion.
pub const MAX_ASSERTION_CHARS: usize = 72;

fn rx(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex must compile")
}

static NARRATION: LazyLock<Regex> = LazyLock::new(|| {
    rx(concat!(
        r"(?i)\b(old|older|previous(ly)?|was|were|used to|formerly|superseded|",
        r"e\.g\.|i\.e\.|if |until|unless|would|could|should we|instead of|rather than|",
        r"relax\w*|tighten\w*|loosen\w*|drift|spec calls for|calls for|baseline|observed|",
        r"aim\w*|hope|plan|eventually|later|when |once |assum\w*|estimat\w*|roughly|about|approx)\b"
    ))
});
static APPROX: LazyLock<Regex> = LazyLock::new(|| rx(r"[~≈]|\babout\b|\bapprox"));
static QUOTED: LazyLock<Regex> = LazyLock::new(|| rx("[\"\u{201c}\u{201d}\u{2018}\u{2019}`]"));

/// What an annotation is doing, and — when it is doing nothing assertive — why.
///
/// The reason is kept rather than collapsed to a boolean so the census can say
/// *how* an annotation was disqualified, and so a gate that rots can be
/// diagnosed instead of merely observed to be quiet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// No trailing comment at all.
    Empty,
    /// Longer than [`MAX_ASSERTION_CHARS`]. A paragraph explains; it does not assert.
    Context,
    /// Talks about another time or another world: `was`, `until`, `if`, `baseline`.
    Narration,
    /// Hedged: `~`, `≈`, `about`, `approx`. An approximation cannot contradict.
    Approximate,
    /// Contains a quoted span. A quoted number is someone else's.
    Quoted,
    /// A short, unhedged, unquoted statement about this value.
    Assertive,
}

/// Classify a same-line trailing comment.
pub fn classify(annot: &str) -> Role {
    let a = annot.trim();
    if a.is_empty() {
        return Role::Empty;
    }
    if a.chars().count() > MAX_ASSERTION_CHARS {
        return Role::Context;
    }
    if NARRATION.is_match(a) {
        return Role::Narration;
    }
    if APPROX.is_match(a) {
        return Role::Approximate;
    }
    if QUOTED.is_match(a) {
        return Role::Quoted;
    }
    Role::Assertive
}

/// Whether a trailing comment asserts something about the value beside it.
pub fn assertive(annot: &str) -> bool {
    classify(annot) == Role::Assertive
}

// ---------------------------------------------------------------- observations

static OBS: LazyLock<Regex> = LazyLock::new(|| {
    rx(&format!(
        concat!(
            r"(?i)\b(current(?:ly)?|actual(?:ly)?|real|measured|today|now|observed|",
            r"we (?:have|measure)|is at|at head|tree measures|shipped)\b\s*[:=]?\s*[~≈]?\s*",
            r"({})\s*([a-zA-Z%]{{0,8}})"
        ),
        NUM
    ))
});
static UNIT_NOISE: LazyLock<Regex> =
    LazyLock::new(|| rx(r"(?i)^(x|times|of|and|in|to|per|the|a)$"));

/// A number the annotation presents as an observation of the current world.
#[derive(Debug, Clone)]
pub struct Observation {
    /// The observed value as written.
    pub value: f64,
    /// The unit token following it, blank when it was a stray English word.
    pub unit: String,
    /// The matched span, for the finding's evidence line.
    pub text: String,
}

/// Every `current: N`, `measured N`, `today N` in an annotation.
pub fn observations(annot: &str) -> Vec<Observation> {
    let mut out = Vec::new();
    for c in OBS.captures_iter(annot) {
        let Some(v) = c
            .get(2)
            .and_then(|m| super::extract::parse_literal(m.as_str()))
        else {
            continue;
        };
        let raw_unit = c.get(3).map_or("", |m| m.as_str());
        let unit = if UNIT_NOISE.is_match(raw_unit) {
            String::new()
        } else {
            raw_unit.to_string()
        };
        out.push(Observation {
            value: v,
            unit,
            text: c.get(0).map_or("", |m| m.as_str()).trim().to_string(),
        });
    }
    out
}

// ---------------------------------------------------------------- restatements

static RESTATE: LazyLock<Regex> = LazyLock::new(|| {
    rx(&format!(
        concat!(
            r"(?i)(?:^|[\s(=])[~≈]?({})\s*",
            r"(ms|s|sec|secs|seconds|min|mins|minutes|h|hours|kb|kib|mb|mib|gb|gib|bytes|%)\b"
        ),
        NUM
    ))
});
static OBS_CLAUSE: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"(?i)\b(current|actual|was|previously|target|expect\w*|real|measured|observed|today|now)\b[^,;)]*",
    )
});
static COMPOUND_DURATION: LazyLock<Regex> = LazyLock::new(|| {
    rx(&format!(
        r"(?i){NUM}\s*(h|hr|m|min|mins)\s+{NUM}\s*(m|min|mins|s|sec|secs)\b"
    ))
});
static HEADROOM: LazyLock<Regex> = LazyLock::new(|| rx(&format!(r"(?i)({NUM})\s*%\s*headroom")));

/// A number the annotation restates the declared value as.
#[derive(Debug, Clone)]
pub struct Restatement {
    /// The restated value as written.
    pub value: f64,
    /// Its unit token.
    pub unit: String,
    /// The matched span.
    pub text: String,
}

/// Drop the observation clauses — they are C1's business, not C2's.
///
/// `50 MB (current: 42 MB, 16% headroom)` restates the limit as `50 MB` and
/// *observes* 42 MB. Without this, C2 would read the observation as a
/// disagreeing restatement and fire on every healthy budget line in the tree.
pub fn strip_observation_clauses(annot: &str) -> String {
    OBS_CLAUSE.replace_all(annot, "").into_owned()
}

/// Drop `11m 57s`-style compound durations, which are one value in two tokens.
pub fn strip_compound_durations(s: &str) -> String {
    COMPOUND_DURATION.replace_all(s, "").into_owned()
}

/// Every `N unit` the cleaned annotation offers as a restatement.
pub fn restatements(head: &str) -> Vec<Restatement> {
    let mut out = Vec::new();
    for c in RESTATE.captures_iter(head) {
        let Some(v) = c
            .get(1)
            .and_then(|m| super::extract::parse_literal(m.as_str()))
        else {
            continue;
        };
        out.push(Restatement {
            value: v,
            unit: c.get(2).map_or("", |m| m.as_str()).to_string(),
            text: c.get(0).map_or("", |m| m.as_str()).trim().to_string(),
        });
    }
    out
}

/// The percentage an annotation claims is left under the limit.
pub fn headroom(annot: &str) -> Option<(f64, String)> {
    let c = HEADROOM.captures(annot)?;
    let v = super::extract::parse_literal(c.get(1)?.as_str())?;
    Some((v, c.get(0)?.as_str().trim().to_string()))
}

// ---------------------------------------------------------------- derivation

/// One lexical token of an annotation, for derivation analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Num(f64),
    Op(char),
    Open,
    Close,
    Word,
    Break,
}

/// A run of word characters collapses to one `Word`, so `bytes payload` is one
/// token and not two: the derivation guard cares only whether a number sits
/// next to prose, never what the prose says.
fn push_word(toks: &mut Vec<Tok>) {
    if toks.last() != Some(&Tok::Word) {
        toks.push(Tok::Word);
    }
}

fn tokenize(annot: &str) -> Vec<Tok> {
    let mut toks = Vec::new();
    let mut rest = annot;
    while !rest.is_empty() {
        if let Some(m) = super::extract::num_prefix(rest) {
            if let Some(v) = super::extract::parse_literal(m) {
                toks.push(Tok::Num(v));
            } else {
                toks.push(Tok::Word);
            }
            rest = &rest[m.len()..];
            continue;
        }
        let c = rest.chars().next().unwrap_or(' ');
        let width = c.len_utf8();
        match c {
            '+' | '-' | '*' | '/' => toks.push(Tok::Op(c)),
            // A repository writes arithmetic in prose, not in Rust:
            // `intermediate_dim × 4 bytes (18944 × 4 for 7B)` in aprender's
            // kernel-fusion contract is a correct derivation of 75,776, and
            // reading U+00D7 as punctuation makes it look like a contradiction.
            '\u{00d7}' | '\u{22c5}' | '\u{00b7}' => toks.push(Tok::Op('*')),
            '\u{00f7}' => toks.push(Tok::Op('/')),
            '\u{2212}' => toks.push(Tok::Op('-')),
            '(' | '[' => toks.push(Tok::Open),
            ')' | ']' => toks.push(Tok::Close),
            ',' | ';' | ':' => toks.push(Tok::Break),
            c if c.is_alphanumeric() || c == '_' => push_word(&mut toks),
            _ => {}
        }
        rest = &rest[width..];
    }
    toks
}

fn numbers_of(toks: &[Tok]) -> Vec<f64> {
    toks.iter()
        .filter_map(|t| match t {
            Tok::Num(v) => Some(*v),
            _ => None,
        })
        .collect()
}

/// Numbers with a word immediately before or after them — `16 bytes`, `4 workers`.
fn anchored_numbers(toks: &[Tok]) -> Vec<f64> {
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let Tok::Num(v) = t else { continue };
        let before = i > 0 && toks[i - 1] == Tok::Word;
        let after = toks.get(i + 1) == Some(&Tok::Word);
        if before || after {
            out.push(*v);
        }
    }
    out
}

/// Evaluate a `Num`/`Op`/paren run with normal precedence.
///
/// Returns `None` for a malformed run rather than guessing — a guess here
/// silently suppresses a genuine contradiction.
fn eval_run(toks: &[Tok]) -> Option<f64> {
    let mut pos = 0usize;
    let v = eval_sum(toks, &mut pos)?;
    if pos == toks.len() {
        Some(v)
    } else {
        None
    }
}

fn eval_sum(toks: &[Tok], pos: &mut usize) -> Option<f64> {
    let mut acc = eval_product(toks, pos)?;
    while let Some(Tok::Op(op)) = toks.get(*pos) {
        let op = *op;
        if op != '+' && op != '-' {
            break;
        }
        *pos += 1;
        let rhs = eval_product(toks, pos)?;
        acc = if op == '+' { acc + rhs } else { acc - rhs };
    }
    Some(acc)
}

fn eval_product(toks: &[Tok], pos: &mut usize) -> Option<f64> {
    let mut acc = eval_atom(toks, pos)?;
    while let Some(Tok::Op(op)) = toks.get(*pos) {
        let op = *op;
        if op != '*' && op != '/' {
            break;
        }
        *pos += 1;
        let rhs = eval_atom(toks, pos)?;
        if op == '/' && rhs == 0.0 {
            return None;
        }
        acc = if op == '*' { acc * rhs } else { acc / rhs };
    }
    Some(acc)
}

fn eval_atom(toks: &[Tok], pos: &mut usize) -> Option<f64> {
    match toks.get(*pos) {
        Some(Tok::Num(v)) => {
            *pos += 1;
            Some(*v)
        }
        Some(Tok::Open) => {
            *pos += 1;
            let v = eval_sum(toks, pos)?;
            if toks.get(*pos) == Some(&Tok::Close) {
                *pos += 1;
                Some(v)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Split into arithmetic runs, dropping the English words between the operands.
///
/// `64 rows * 17 bytes` is the expression `64 * 17` with two nouns in it. A
/// comma or a colon ends a run: `500 ms budget, 3 phases` is two statements,
/// not one product.
fn arithmetic_runs(toks: &[Tok]) -> Vec<Vec<Tok>> {
    let mut runs = Vec::new();
    let mut cur: Vec<Tok> = Vec::new();
    for t in toks {
        match t {
            Tok::Word => {}
            Tok::Break => {
                runs.push(std::mem::take(&mut cur));
            }
            other => cur.push(*other),
        }
    }
    runs.push(cur);
    runs.into_iter().filter(|r| r.len() >= 3).collect()
}

/// The word-stripped stream with every bracket and separator flattened to a
/// break, for the chain pass.
fn flattened(toks: &[Tok]) -> Vec<Tok> {
    toks.iter()
        .filter(|t| **t != Tok::Word)
        .map(|t| match t {
            Tok::Open | Tok::Close => Tok::Break,
            other => *other,
        })
        .collect()
}

/// Every maximal `Num (Op Num)+` chain in the stream.
///
/// The run pass above has to consume its whole run, which a parenthesised aside
/// can defeat: in `(18944 × 4 for 7B)` the trailing `7` leaves the run
/// unparseable, and the multiplication that explains the value goes unseen.
/// A chain stops at the first numeral that no operator connects, so it finds
/// `18944 × 4` and ignores the `7`.
fn operator_chains(toks: &[Tok]) -> Vec<f64> {
    let s = flattened(toks);
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < s.len() {
        let Tok::Num(v) = s[i] else {
            i += 1;
            continue;
        };
        let mut chain = vec![Tok::Num(v)];
        let mut j = i + 1;
        while j + 1 < s.len() {
            let (Tok::Op(op), Tok::Num(n)) = (s[j], s[j + 1]) else {
                break;
            };
            chain.push(Tok::Op(op));
            chain.push(Tok::Num(n));
            j += 2;
        }
        if chain.len() >= 3 {
            out.extend(eval_run(&chain));
        }
        i = j.max(i + 1);
    }
    out
}

/// Every value the annotation can be read as computing.
fn derivation_candidates(toks: &[Tok]) -> Vec<f64> {
    let mut out: Vec<f64> = arithmetic_runs(toks)
        .iter()
        .filter_map(|r| eval_run(r))
        .collect();
    out.extend(operator_chains(toks));
    out
}

/// Sums and products of every 2- and 3-element subset of `pool`.
///
/// Bounded on purpose. `pool` holds only the numbers adjacent to a word, and
/// annotations are capped at 72 characters, so this is a handful of terms — but
/// the bound is what keeps the guard from finding a "derivation" by chance.
fn small_subset_values(pool: &[f64]) -> Vec<f64> {
    let mut out = Vec::new();
    for (i, a) in pool.iter().enumerate() {
        for b in &pool[i + 1..] {
            out.push(a + b);
            out.push(a * b);
        }
    }
    for (i, a) in pool.iter().enumerate() {
        for (j, b) in pool.iter().enumerate().skip(i + 1) {
            for c in &pool[j + 1..] {
                out.push(a + b + c);
                out.push(a * b * c);
            }
        }
    }
    out
}

fn subset_hits(pool: &[f64], target: &[f64]) -> bool {
    small_subset_values(pool)
        .into_iter()
        .any(|v| hits(v, target))
}

fn hits(candidate: f64, target: &[f64]) -> bool {
    target.iter().any(|t| close(candidate, *t))
}

/// Whether the annotation *computes* the declared value out of its parts.
///
/// A derivation is not a restatement: `= 18; // (2 bytes scale + 16 bytes
/// quants)` itemises 18, it does not contradict it. Three forms are accepted,
/// and only three:
///
/// ```text
///   (a) the sum, or the product, of EVERY number in the annotation
///   (b) an arithmetic run, read with the English words removed
///   (c) a sum or product of at most 3 numbers, each adjacent to a word
/// ```
///
/// The prototype instead searched every pairwise combination of `+ - * /`,
/// `/1000` and `/100` for a 2%-tolerant hit. Measured against a real annotation
/// distribution that declared an *unrelated* target a "derivation" 21.6% of the
/// time at two numbers and 84.4% at ten — so on the commonest annotated shape
/// it silently suppressed about one in five genuine contradictions. The forms
/// above are bounded, and every suppression is counted in the census.
pub fn is_derivation(annot: &str, target: &[f64]) -> bool {
    let toks = tokenize(annot);
    let ns = numbers_of(&toks);
    if ns.len() < 2 {
        return false;
    }
    if hits(ns.iter().sum::<f64>(), target) || hits(ns.iter().product::<f64>(), target) {
        return true;
    }
    if derivation_candidates(&toks)
        .into_iter()
        .any(|v| hits(v, target))
    {
        return true;
    }
    let anchored = anchored_numbers(&toks);
    anchored.len() >= 2 && subset_hits(&anchored, target)
}

// ---------------------------------------------------------------- references

static XREF: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r#"(?i)(?:aligned with|must match|should match|must equal|matches|same as|identical to|mirrors|mirrored in|consistent with|kept in sync with|in sync with|duplicated from|copy of|equals)\s+(?:the\s+)?([^\s,;)`'"]*\.(?:toml|yaml|yml|json|rs|md))?[\s`'"]*([A-Za-z_][A-Za-z0-9_.]{4,})?"#,
    )
});

/// A claim that this value equals a named value elsewhere.
#[derive(Debug, Clone)]
pub struct Xref {
    /// The file named, if one was.
    pub file: Option<String>,
    /// The key named, if one was.
    pub key: Option<String>,
    /// The matched span, for the finding's evidence line.
    pub text: String,
}

/// Every equality claim in a blob of annotation text.
pub fn xrefs(text: &str) -> Vec<Xref> {
    XREF.captures_iter(text)
        .map(|c| Xref {
            file: c.get(1).map(|m| m.as_str().to_string()),
            key: c.get(2).map(|m| m.as_str().to_string()),
            text: c.get(0).map_or("", |m| m.as_str()).trim().to_string(),
        })
        .collect()
}

static AUTHORITY: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"(?i)\b(PMAT-\d+|CB-\d+|[A-Z]{2,6}-\d{2,5}|NASA|CLAUDE\.md|AGENTS?\.md|README\.md|ISO ?\d+|MISRA|per spec|the spec|docs/[\w./-]+)\b",
    )
});
static RESTATE_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"(?i)^[\s\W]*(target|current|limit|max|min|minimum|maximum|value|budget|threshold|prefer|per|of|the|a|is|to|and|for|line|function|coverage|score|bytes?|ms|s|mb|kb|gb|%|<=|>=|≤|≥|~|\d|\.|,|\(|\)|-)*$",
    )
});
static WORDS: LazyLock<Regex> = LazyLock::new(|| rx(r"[A-Za-z][A-Za-z_-]{2,}"));

const RATIONALE_STOP: [&str; 26] = [
    "target",
    "current",
    "limit",
    "max",
    "min",
    "minimum",
    "maximum",
    "value",
    "budget",
    "threshold",
    "prefer",
    "per",
    "the",
    "and",
    "for",
    "line",
    "function",
    "coverage",
    "score",
    "bytes",
    "byte",
    "achieved",
    "upgraded",
    "from",
    "required",
    "require",
];

/// Named authorities the annotation cites for its value.
pub fn authorities(annot: &str) -> Vec<String> {
    AUTHORITY
        .captures_iter(annot)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_ascii_lowercase()))
        .collect()
}

/// Whether a divergent value carries a stated reason.
///
/// A number that differs from its siblings *and gives a reason* is policy. One
/// that differs with no reason — or with a comment that only restates it — is
/// anchored to nothing, which is the whole of C4's claim.
pub fn has_rationale(annot: &str, key: &str) -> bool {
    if annot.trim().is_empty() {
        return false;
    }
    if RESTATE_ONLY.is_match(annot) {
        return false;
    }
    let key_tokens = super::extract::norm_key(key);
    WORDS
        .find_iter(annot)
        .map(|m| m.as_str().to_ascii_lowercase())
        .any(|w| !key_tokens.contains(&w) && !RATIONALE_STOP.contains(&w.as_str()))
}
