//! Find `#[test]` functions that cannot fail.
//!
//! # Why this exists
//!
//! A stack-wide audit counted ~933 tests across the fleet whose bodies contain
//! nothing that can fail. 714 of forjar's 802 live in files named `tests_cov_*`,
//! and 584 of them use `let _ = <fallible call>;` — the minimum edit that
//! executes a line without checking anything.
//!
//! That is not carelessness, it is the predictable response to an incentive.
//! Line coverage is the only metric in the fleet with a hard floor (copia
//! `coverage_min: 95.0`, forjar `--fail-under-lines 95`, whisper
//! `COV_THRESHOLD ?= 95`). Line coverage measures **execution**, not
//! **verification**, so under a hard floor `let _ = call();` is the cheapest
//! way to comply. The metric that would oppose it — mutation kill rate — is
//! structurally unavailable, because `pmat mutate` cannot Kill.
//!
//! In the pass/fail vocabulary a test that executes code and discards the result
//! IS a passing test. Nothing distinguishes "the assertion held" from "there was
//! no assertion". This module makes that distinction expressible.
//!
//! # Why this parses Rust rather than grepping it
//!
//! The question "does this test body contain an assertion?" is not answerable by
//! line matching: bodies span lines, `assert` appears inside strings and
//! comments, and helper functions defined inside a test body have their own
//! bodies. Every text-based draft of a detector in this crate has needed several
//! rounds of false-positive removal for exactly those reasons. `syn` answers the
//! question directly, so the failure modes that remain are semantic ones worth
//! arguing about rather than lexical accidents.
//!
//! # What is deliberately NOT flagged
//!
//! - `#[should_panic]` tests — the attribute IS the assertion.
//! - Tests returning `Result` that use `?` — a propagated error fails the test.
//! - `.unwrap()` / `.expect()` — a panic on the unhappy path is a real failure
//!   mode, weaker than an assertion but not vacuous.
//! - Tests that call a helper which asserts. Helpers defined **in the same
//!   file** are resolved one level deep; cross-file helpers are not, and that
//!   residual is reported rather than hidden.

use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/// Why a test cannot fail.
///
/// # The objection worth answering
///
/// forjar's `test_stdout_writer_does_not_panic` calls five methods and asserts
/// nothing, and its name argues that not panicking IS the assertion. That is
/// true as far as it goes — but it is true of *every* test ever written, so if
/// incidental panic-catching counts as a failure mode then no test is vacuous
/// and the word means nothing.
///
/// The distinction this type draws is therefore narrower and stateable: a
/// `NoFailureMode` test can still catch a panic in the code it calls. What it
/// cannot do is notice a **wrong answer**. It is a smoke test, and reporting it
/// as one is the point — a suite of them satisfies a line-coverage floor while
/// leaving every computed value unchecked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Vacuity {
    /// The body contains no assertion, no panic, no `?` and no unwrap. It runs
    /// code and discards the outcome: it would catch a panic, but not a wrong
    /// answer.
    NoFailureMode,
    /// The body asserts something that is true for every input:
    /// `assert!(true)`, `assert!(r.is_ok() || r.is_err())`, `assert_eq!(x, x)`.
    Tautology,
}

impl Vacuity {
    pub fn as_str(self) -> &'static str {
        match self {
            Vacuity::NoFailureMode => "no-failure-mode",
            Vacuity::Tautology => "tautology",
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Vacuity::NoFailureMode => "smoke test: catches a panic, not a wrong answer",
            Vacuity::Tautology => "asserts something true for every input",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VacuousTest {
    pub file: String,
    pub line: usize,
    pub name: String,
    pub kind: Vacuity,
    /// For a tautology, the offending assertion as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A test that returns early when its fixture is missing. Not vacuous by
/// construction — it may assert plenty when the fixture is present — but it
/// reports PASS on a machine where it checked nothing, and unlike `#[ignore]`
/// that is invisible in the test output.
#[derive(Debug, Clone, Serialize)]
pub struct ConditionalSkip {
    pub file: String,
    pub line: usize,
    pub name: String,
    /// The guard that triggers the early return, as written.
    pub guard: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Skipped {
    pub unreadable: Vec<String>,
    /// Files `syn::parse_file` rejects. In this tree they are all legitimate:
    /// `include!` fragments, which are deliberately not balanced because their
    /// closing brace lives in the includer (`work_tests_part1.rs` has 56 `{`
    /// and 53 `}`), and non-Rust content carrying a `.rs` extension (the demo
    /// HTML templates). Neither is a defect — but the tests inside them are
    /// real and go unjudged, so they are counted, not merely listed.
    pub unparseable: Vec<String>,
    /// `#[test]` markers found textually in files that could not be parsed.
    /// This is what makes the FLOOR a magnitude instead of an apology.
    pub unmeasured_tests: usize,
}

impl Skipped {
    pub fn total(&self) -> usize {
        self.unreadable.len() + self.unparseable.len()
    }
}

/// Count `#[test]`-ish attributes in text, for files that will not parse.
fn count_test_markers(src: &str) -> usize {
    src.lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//")
                && (t.starts_with("#[test]")
                    || t.starts_with("#[tokio::test")
                    || t.starts_with("#[rstest")
                    || t.starts_with("#[test_case"))
        })
        .count()
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Report {
    pub vacuous: Vec<VacuousTest>,
    pub conditional_skips: Vec<ConditionalSkip>,
    /// Denominator: `#[test]` functions actually examined.
    pub tests_examined: usize,
    /// Denominator: files actually parsed.
    pub files_parsed: usize,
    pub skipped: Skipped,
}

impl Report {
    pub fn by_kind(&self) -> BTreeMap<&'static str, usize> {
        let mut m = BTreeMap::new();
        for v in &self.vacuous {
            *m.entry(v.kind.as_str()).or_insert(0) += 1;
        }
        m
    }

    /// The rate, which is the number that actually means something: 800 vacuous
    /// tests out of 17,000 is a different problem from 800 out of 900.
    pub fn rate(&self) -> f64 {
        if self.tests_examined == 0 {
            return 0.0;
        }
        self.vacuous.len() as f64 / self.tests_examined as f64 * 100.0
    }

    pub fn summary(&self) -> String {
        let mut s = format!(
            "{} of {} #[test] fns cannot fail ({:.1}%) across {} parsed file(s); \
             {} more skip silently when a fixture is missing",
            self.vacuous.len(),
            self.tests_examined,
            self.rate(),
            self.files_parsed,
            self.conditional_skips.len(),
        );
        if self.skipped.total() > 0 {
            s.push_str(&format!(
                " — FLOOR ONLY: {} file(s) not analysed ({} unreadable, {} unparseable), \
                 holding {} unjudged #[test] fn(s)",
                self.skipped.total(),
                self.skipped.unreadable.len(),
                self.skipped.unparseable.len(),
                self.skipped.unmeasured_tests,
            ));
        }
        s
    }
}

/// What a function body does that could make a test fail.
#[derive(Debug, Default, Clone, Copy)]
struct FailureModes {
    asserts: usize,
    /// Assertions that hold for every input.
    tautologies: usize,
    panics: usize,
    /// `?`, `.unwrap()`, `.expect()`
    propagates: usize,
}

impl FailureModes {
    /// An assertion that can discriminate, or any other way to fail.
    fn can_fail(self) -> bool {
        self.asserts > self.tautologies || self.panics > 0 || self.propagates > 0
    }
    fn merge(&mut self, other: Self) {
        self.asserts += other.asserts;
        self.tautologies += other.tautologies;
        self.panics += other.panics;
        self.propagates += other.propagates;
    }
}

/// Walks a function body counting the ways it could fail, and records the
/// same-file functions it calls so helpers can be resolved one level deep.
#[derive(Default)]
struct BodyVisitor {
    modes: FailureModes,
    called: HashSet<String>,
    /// The first tautological assertion seen, for the report.
    first_tautology: Option<String>,
    /// A guard that returns early, e.g. `if !path.exists() { return; }`.
    skip_guard: Option<String>,
}

fn macro_name(mac: &syn::Macro) -> String {
    mac.path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default()
}

/// Normalise a token stream to a comparable string: no whitespace differences.
fn toks(mac: &syn::Macro) -> String {
    mac.tokens
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Is this assertion true for every input?
///
/// Only syntactic tautologies are recognised. Proving an arbitrary assertion
/// vacuous is undecidable; these three forms cover what the audit actually
/// found, and anything less certain is left alone rather than guessed at.
fn tautology_of(name: &str, args: &str) -> Option<String> {
    let a = args.trim();
    match name {
        // `assert!(true)`
        "assert" if a == "true" => Some(a.to_string()),
        // `assert!(x.is_ok() || x.is_err())` — the wos `poppian_*` form. Any
        // `Result` satisfies exactly one, so the disjunction is always true.
        "assert" if is_ok_or_is_err(a) => Some(a.to_string()),
        // `assert_eq!(x, x)` / `assert_ne!` of a value with itself.
        "assert_eq" | "assert_ne" => {
            let (l, r) = split_top_comma(a)?;
            (l == r && !l.is_empty()).then(|| format!("{l}, {r}"))
        }
        _ => None,
    }
}

/// `<e>.is_ok() || <e>.is_err()` in either order, for the same receiver.
/// Compared after removing ALL whitespace, because the input is a `syn` token
/// stream rendered back to text: `assert!(r.is_ok() || r.is_err())` arrives as
/// `r . is_ok () || r . is_err ()`. Matching the source spelling instead silently
/// matched nothing.
fn is_ok_or_is_err(a: &str) -> bool {
    let dense: String = a.chars().filter(|c| !c.is_whitespace()).collect();
    let Some((l, r)) = dense.split_once("||") else {
        return false;
    };
    let strip = |s: &str| -> Option<(String, bool)> {
        s.strip_suffix(".is_ok()")
            .map(|b| (b.to_string(), true))
            .or_else(|| s.strip_suffix(".is_err()").map(|b| (b.to_string(), false)))
    };
    match (strip(l), strip(r)) {
        (Some((lb, l_ok)), Some((rb, r_ok))) => lb == rb && !lb.is_empty() && l_ok != r_ok,
        _ => false,
    }
}

/// Split `a, b` at the top-level comma, respecting nesting and strings.
fn split_top_comma(s: &str) -> Option<(String, String)> {
    let (mut depth, mut in_str, mut esc) = (0i32, false, false);
    for (i, c) in s.char_indices() {
        if esc {
            esc = false;
            continue;
        }
        match c {
            '\\' if in_str => esc = true,
            '"' => in_str = !in_str,
            _ if in_str => {}
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                return Some((s[..i].trim().to_string(), s[i + 1..].trim().to_string()))
            }
            _ => {}
        }
    }
    None
}

impl<'ast> Visit<'ast> for BodyVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let name = macro_name(mac);
        let args = toks(mac);
        match name.as_str() {
            n if n.starts_with("assert") || n == "debug_assert" => {
                self.modes.asserts += 1;
                // `assert_eq!(a, b, "msg")` — only the compared operands matter.
                let cmp = args.rsplit_once(" , \"").map_or(args.as_str(), |(l, _)| l);
                if let Some(t) = tautology_of(&name, cmp) {
                    self.modes.tautologies += 1;
                    self.first_tautology.get_or_insert(format!("{name}!({t})"));
                }
            }
            "panic" | "unreachable" | "todo" | "unimplemented" => self.modes.panics += 1,
            // A macro we do not understand may contain assertions — most
            // commonly a project's own `assert_matches!`-style helper. Count it
            // as a failure mode rather than call the test vacuous on a guess.
            n if n.contains("assert") || n.contains("expect") || n.contains("verify") => {
                self.modes.asserts += 1;
            }
            _ => {}
        }
        syn::visit::visit_macro(self, mac);
    }

    fn visit_expr_try(&mut self, node: &'ast syn::ExprTry) {
        self.modes.propagates += 1;
        syn::visit::visit_expr_try(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let m = node.method.to_string();
        if m == "unwrap" || m == "expect" || m == "unwrap_err" || m == "expect_err" {
            self.modes.propagates += 1;
        }
        // A method call is not a same-file free function, so nothing is
        // recorded for helper resolution here.
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*node.func {
            if let Some(last) = p.path.segments.last() {
                self.called.insert(last.ident.to_string());
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        // `if <guard> { return; }` — a silent skip. Only a bare `return` with no
        // value counts; `return Ok(())` in a Result test is ordinary control
        // flow, and an early return that yields a value is not a skip.
        let bare_return = node.then_branch.stmts.len() == 1
            && matches!(
                node.then_branch.stmts.first(),
                Some(syn::Stmt::Expr(
                    syn::Expr::Return(syn::ExprReturn { expr: None, .. }),
                    _
                ))
            );
        if bare_return {
            use quote::ToTokens;
            let cond = node.cond.to_token_stream().to_string();
            if looks_environmental(&cond) {
                self.skip_guard.get_or_insert(cond);
            }
        }
        syn::visit::visit_expr_if(self, node);
    }
}

/// Does this guard depend on the machine rather than on the code under test?
///
/// The distinction matters: `if v.is_empty() { return; }` is a legitimate early
/// exit from a loop body, while `if !Path::new(p).exists() { return; }` means
/// the test silently did nothing on this host. Only the latter is reported.
fn looks_environmental(cond: &str) -> bool {
    const MARKERS: &[&str] = &[
        "exists",
        "env :: var",
        "env::var",
        "var_os",
        "which",
        "is_err",
        "is_none",
        "metadata",
        "Command :: new",
        "Command::new",
    ];
    MARKERS.iter().any(|m| cond.contains(m))
}

/// `#[test]`, `#[tokio::test]`, `#[test_case(..)]`, `#[rstest]` — anything whose
/// attribute path ends in a test marker. Matching on the last segment rather
/// than a per-framework list means a new harness is picked up without an edit
/// here; the cost is that an unrelated attribute named `test` would match, which
/// has no realistic instance.
fn is_test_fn(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "test" || s.ident == "test_case" || s.ident == "rstest")
    })
}

/// `#[should_panic]` is itself the assertion.
fn has_should_panic(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("should_panic"))
}

fn is_ignored(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("ignore"))
}

/// Collect every free function in the file, so a helper's failure modes can be
/// credited to the test that calls it.
#[derive(Default)]
struct FnCollector {
    bodies: HashMap<String, FailureModes>,
}

impl<'ast> Visit<'ast> for FnCollector {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        let mut v = BodyVisitor::default();
        v.visit_block(&f.block);
        self.bodies.insert(f.sig.ident.to_string(), v.modes);
        syn::visit::visit_item_fn(self, f);
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let mut v = BodyVisitor::default();
        v.visit_block(&f.block);
        self.bodies.insert(f.sig.ident.to_string(), v.modes);
        syn::visit::visit_impl_item_fn(self, f);
    }
}

/// Walks a parsed file and judges every `#[test]` function in it.
struct TestVisitor<'a> {
    rel: &'a str,
    helpers: &'a HashMap<String, FailureModes>,
    lines: &'a mut HashMap<String, VecDeque<usize>>,
    vacuous: Vec<VacuousTest>,
    skips: Vec<ConditionalSkip>,
    examined: usize,
}

impl TestVisitor<'_> {
    fn judge(&mut self, sig: &syn::Signature, attrs: &[syn::Attribute], block: &syn::Block) {
        if !is_test_fn(attrs) {
            return;
        }
        self.examined += 1;
        let name = sig.ident.to_string();
        let line = self
            .lines
            .get_mut(&name)
            .and_then(std::collections::VecDeque::pop_front)
            .unwrap_or(0);

        let mut v = BodyVisitor::default();
        v.visit_block(block);

        if let Some(guard) = v.skip_guard.clone() {
            self.skips.push(ConditionalSkip {
                file: self.rel.to_string(),
                line,
                name: name.clone(),
                guard,
            });
        }

        // `#[should_panic]` and `#[ignore]` are both explicit: the first IS an
        // assertion, the second is an honestly-declared non-run. Neither is the
        // silent pass this command hunts.
        if has_should_panic(attrs) || is_ignored(attrs) {
            return;
        }

        let mut modes = v.modes;
        // Credit same-file helpers, one level deep. A test whose whole body is
        // `check_roundtrip(x)` is not vacuous when `check_roundtrip` asserts.
        for callee in &v.called {
            if let Some(h) = self.helpers.get(callee) {
                modes.merge(*h);
            }
        }

        if !modes.can_fail() {
            let (kind, detail) = if modes.asserts > 0 {
                (Vacuity::Tautology, v.first_tautology.clone())
            } else {
                (Vacuity::NoFailureMode, None)
            };
            self.vacuous.push(VacuousTest {
                file: self.rel.to_string(),
                line,
                name,
                kind,
                detail,
            });
        }
    }
}

impl<'ast> Visit<'ast> for TestVisitor<'_> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.judge(&f.sig, &f.attrs, &f.block);
        syn::visit::visit_item_fn(self, f);
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.judge(&f.sig, &f.attrs, &f.block);
        syn::visit::visit_impl_item_fn(self, f);
    }
}

/// Map every `fn NAME` declaration to the 1-based line it is written on, in
/// source order.
///
/// `syn` spans cannot supply this: `Span::byte_range()` does not exist on the
/// non-proc-macro `proc_macro2::Span`, and `Span::start()` needs proc-macro2's
/// `span-locations` feature, which this crate does not enable and which would
/// impose a location-tracking cost on every other AST consumer in pmat.
///
/// Declarations are stored per name in a queue and consumed in visit order,
/// which `syn` performs in source order — so two same-named tests in different
/// modules get their own lines rather than both pointing at the first.
fn declaration_lines(src: &str) -> HashMap<String, VecDeque<usize>> {
    let mut m: HashMap<String, VecDeque<usize>> = HashMap::new();
    for (idx, line) in src.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        let Some(at) = t.find("fn ") else { continue };
        // Require a word boundary before `fn` so `async fn` counts and
        // identifiers ending in "fn" do not.
        if at > 0 && !t.as_bytes()[at - 1].is_ascii_whitespace() {
            continue;
        }
        let rest = &t[at + 3..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            m.entry(name).or_default().push_back(idx + 1);
        }
    }
    m
}

/// Analyse one file's source. Public so the judgement can be tested directly on
/// a string without a filesystem.
pub fn scan_source(
    rel: &str,
    src: &str,
) -> Option<(Vec<VacuousTest>, Vec<ConditionalSkip>, usize)> {
    let file = syn::parse_file(src).ok()?;

    let mut helpers = FnCollector::default();
    helpers.visit_file(&file);

    let mut lines = declaration_lines(src);
    let mut tv = TestVisitor {
        rel,
        helpers: &helpers.bodies,
        lines: &mut lines,
        vacuous: Vec::new(),
        skips: Vec::new(),
        examined: 0,
    };
    tv.visit_file(&file);
    Some((tv.vacuous, tv.skips, tv.examined))
}

/// Tracked `.rs` files, so the scan matches what is actually in the repo.
pub fn tracked_rust_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", "*.rs"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git ls-files failed in {} — cannot enumerate tracked files, so the scan would \
             have no denominator",
            root.display()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect())
}

pub fn analyze(root: &Path, files: &[PathBuf]) -> Report {
    let mut r = Report::default();
    for rel in files {
        let rel_s = rel.to_string_lossy().to_string();
        let Ok(src) = std::fs::read_to_string(root.join(rel)) else {
            r.skipped.unreadable.push(rel_s);
            continue;
        };
        match scan_source(&rel_s, &src) {
            Some((mut v, mut s, n)) => {
                r.files_parsed += 1;
                r.tests_examined += n;
                r.vacuous.append(&mut v);
                r.conditional_skips.append(&mut s);
            }
            // A file syn cannot parse is reported, never silently treated as
            // containing no tests — that would be the exact "absence rendered
            // as success" this analyzer exists to expose.
            None => {
                r.skipped.unmeasured_tests += count_test_markers(&src);
                r.skipped.unparseable.push(rel_s);
            }
        }
    }
    r.vacuous
        .sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    r.conditional_skips
        .sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(src: &str) -> (Vec<VacuousTest>, Vec<ConditionalSkip>, usize) {
        scan_source("src/lib.rs", src).expect("parses")
    }

    /// The `let _ = call();` form — 584 of forjar's 802.
    #[test]
    fn a_test_that_discards_its_result_cannot_fail() {
        let (v, _, n) = scan(
            r#"
#[test]
fn t() {
    let _ = compute(1, 2);
}
"#,
        );
        assert_eq!(n, 1);
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].kind, Vacuity::NoFailureMode);
        assert_eq!(v[0].name, "t");
    }

    /// wos's `poppian_41_path_traversal`, verbatim in shape. Every `Result`
    /// satisfies exactly one arm, so the disjunction holds for all inputs.
    #[test]
    fn is_ok_or_is_err_is_a_tautology() {
        let (v, _, _) = scan(
            r#"
#[test]
fn poppian_41() {
    let result = dispatch_syscall("/home/../../../etc/shadow");
    assert!(result.is_ok() || result.is_err());
}
"#,
        );
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].kind, Vacuity::Tautology);
        assert!(v[0].detail.as_deref().unwrap_or("").contains("is_ok"));
    }

    #[test]
    fn assert_true_and_self_comparison_are_tautologies() {
        for body in ["assert!(true);", "assert_eq!(x, x);", "assert_ne!(y, y);"] {
            let (v, _, _) = scan(&format!(
                "#[test]\nfn t() {{ let x = 1; let y = 2; {body} }}"
            ));
            assert_eq!(v.len(), 1, "{body} was not caught: {v:?}");
            assert_eq!(v[0].kind, Vacuity::Tautology, "{body}");
        }
    }

    /// The false-negative floor: a real assertion must never be called vacuous.
    #[test]
    fn a_real_assertion_is_not_vacuous() {
        for body in [
            "assert_eq!(add(2, 2), 4);",
            "assert!(v.len() > 3);",
            "assert!(r.is_ok());",
            "assert_ne!(a, b);",
        ] {
            let (v, _, _) = scan(&format!("#[test]\nfn t() {{ {body} }}"));
            assert!(v.is_empty(), "{body} was wrongly called vacuous: {v:?}");
        }
    }

    /// Panicking IS failing. These tests can fail, so they are not vacuous.
    #[test]
    fn unwrap_expect_try_and_panic_are_failure_modes() {
        for body in [
            "let _ = f().unwrap();",
            "let _ = f().expect(\"boom\");",
            "if !ok() { panic!(\"no\"); }",
            "unreachable!();",
        ] {
            let (v, _, _) = scan(&format!("#[test]\nfn t() {{ {body} }}"));
            assert!(v.is_empty(), "{body} was wrongly called vacuous: {v:?}");
        }
        let (v, _, _) = scan("#[test]\nfn t() -> Result<(), E> { f()?; Ok(()) }");
        assert!(v.is_empty(), "`?` is a failure mode: {v:?}");
    }

    /// `#[should_panic]` is the assertion. An empty-looking body under it is
    /// checking something real.
    #[test]
    fn should_panic_is_an_assertion() {
        let (v, _, _) = scan("#[test]\n#[should_panic]\nfn t() { let _ = boom(); }");
        assert!(v.is_empty(), "{v:?}");
    }

    /// `#[ignore]` is an honest declaration that the test does not run. It is a
    /// different problem (an unmeasured test) and is not silently green, so it
    /// is not reported here.
    #[test]
    fn an_ignored_test_is_not_reported_as_vacuous() {
        let (v, _, _) = scan("#[test]\n#[ignore]\nfn t() { let _ = f(); }");
        assert!(v.is_empty(), "{v:?}");
    }

    /// The single most likely false positive: a test whose whole body is a call
    /// to a helper that does the asserting.
    #[test]
    fn a_helper_that_asserts_is_credited_to_its_caller() {
        let (v, _, _) = scan(
            r#"
fn check_roundtrip(x: u32) {
    assert_eq!(decode(encode(x)), x);
}

#[test]
fn t() {
    check_roundtrip(7);
}
"#,
        );
        assert!(
            v.is_empty(),
            "a test delegating to an asserting helper was called vacuous: {v:?}"
        );
    }

    /// …but a helper that itself checks nothing must not launder the test.
    #[test]
    fn a_helper_that_asserts_nothing_does_not_launder_the_test() {
        let (v, _, _) = scan(
            r#"
fn just_runs(x: u32) -> u32 { x + 1 }

#[test]
fn t() {
    just_runs(7);
}
"#,
        );
        assert_eq!(v.len(), 1, "{v:?}");
    }

    /// The environment-dependent variant: ~97 in aprender, 95 in whisper.apr,
    /// none of them `#[ignore]`d. It reports PASS having checked nothing, and
    /// unlike `#[ignore]` that is invisible in the output.
    #[test]
    fn a_fixture_guard_that_returns_early_is_reported_as_a_silent_skip() {
        let (_, s, _) = scan(
            r#"
#[test]
fn t() {
    if !std::path::Path::new("/opt/model.gguf").exists() { return; }
    assert_eq!(load("/opt/model.gguf").len(), 42);
}
"#,
        );
        assert_eq!(s.len(), 1, "{s:?}");
        assert!(s[0].guard.contains("exists"), "{:?}", s[0].guard);
    }

    /// An ordinary early return on the data under test is control flow, not a
    /// silent skip. Without this distinction every `if x.is_empty() { return; }`
    /// would be reported.
    #[test]
    fn an_ordinary_early_return_is_not_a_silent_skip() {
        let (_, s, _) = scan(
            r#"
#[test]
fn t() {
    let v = build();
    if v.len() == 0 { return; }
    assert!(v[0] > 0);
}
"#,
        );
        assert!(
            s.is_empty(),
            "ordinary control flow reported as skip: {s:?}"
        );
    }

    /// The report must never let an unparsed file look like a file with no
    /// vacuous tests (#1015).
    #[test]
    fn an_unparseable_file_downgrades_the_claim_to_a_floor() {
        assert!(scan_source("x.rs", "fn broken( {").is_none());

        let mut r = Report {
            tests_examined: 10,
            files_parsed: 3,
            ..Default::default()
        };
        assert!(!r.summary().contains("FLOOR"));
        r.skipped.unparseable.push("x.rs".into());
        r.skipped.unmeasured_tests = 7;
        let s = r.summary();
        assert!(
            s.contains("FLOOR ONLY"),
            "partial scan reported as complete: {s}"
        );
        // The magnitude, not just the fact: "some files were skipped" is the
        // kind of caveat a reader walks past.
        assert!(s.contains("7 unjudged"), "{s}");
    }

    /// A rate needs a denominator: 800 of 17,000 is a different problem from
    /// 800 of 900, and the bare count cannot tell them apart.
    #[test]
    fn the_summary_carries_the_rate_and_its_denominator() {
        let r = Report {
            vacuous: vec![VacuousTest {
                file: "a.rs".into(),
                line: 1,
                name: "t".into(),
                kind: Vacuity::NoFailureMode,
                detail: None,
            }],
            tests_examined: 4,
            files_parsed: 1,
            ..Default::default()
        };
        let s = r.summary();
        assert!(s.contains("1 of 4"), "{s}");
        assert!(s.contains("25.0%"), "{s}");
    }

    /// Line numbers come from `syn` spans, which silently degrade to 0 when
    /// proc-macro2's span-locations are unavailable. A report that points every
    /// finding at line 1 is worse than useless, so the behaviour is asserted
    /// rather than assumed.
    #[test]
    fn test_markers_are_counted_in_files_that_will_not_parse() {
        // An `include!` fragment: unbalanced on purpose, so it cannot parse.
        let frag = "use super::*;\nmod t {\n#[test]\nfn a() { assert!(x); }\n#[test]\nfn b() {}\n";
        assert!(
            scan_source("frag.rs", frag).is_none(),
            "fragment should not parse"
        );
        assert_eq!(count_test_markers(frag), 2);
        // Commented-out attributes are not tests.
        assert_eq!(count_test_markers("// #[test]\n#[test]\n"), 1);
    }

    #[test]
    fn reported_line_numbers_are_real() {
        let src = "\n\n\n\n#[test]\nfn t() { let _ = f(); }\n";
        let (v, _, _) = scan(src);
        assert_eq!(v.len(), 1);
        assert_eq!(
            v[0].line, 6,
            "span-derived line numbers are not resolving (got {})",
            v[0].line
        );
    }

    #[test]
    fn tokio_tests_are_examined_too() {
        let (v, _, n) = scan("#[tokio::test]\nasync fn t() { let _ = f().await; }");
        assert_eq!(n, 1, "a #[tokio::test] was not examined");
        assert_eq!(v.len(), 1, "{v:?}");
    }
}
