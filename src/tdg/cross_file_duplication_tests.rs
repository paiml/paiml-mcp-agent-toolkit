//! The TDG duplication component must measure duplication ACROSS the files it
//! grades, not only inside each one.
//!
//! Issue #1050. `impl Scorer for DuplicationDetector` and its live successor
//! `analyze_duplication_ast` both take a single file's source. Ten byte-identical
//! files each have 0% *internal* duplication, so each scored the full 20/20 and
//! the project aggregate — a mean of per-file components — scored 20/20 too.
//! `TdgScore::default()` seeds `duplication_ratio: 20.0`, and nothing at project
//! level ever lowered it, so the component awarded full marks for a tree it had
//! no way to measure.
//!
//! Measured on the fixtures below against the pre-fix binary:
//!
//! ```text
//!            analyze duplicates   tdg total              duplication component
//! clean10          0.0%           99.7727279663086       20.0 / 20
//! cloned20        88.2%           99.77272033691406      20.0 / 20
//! ```
//!
//! The two totals differ only in f32 summation noise at the seventh decimal.
//! `cloned20` is `clean10` plus a byte-identical copy of every one of its files,
//! so every OTHER per-file component has an identical mean across the two trees
//! by construction: any difference these tests observe is attributable to
//! duplication and to nothing else.

use crate::tdg::TdgAnalyzer;
use std::path::Path;

/// Ten structurally distinct Rust files. Distinct at the level the clone engine
/// actually measures: the detector normalises identifiers and literals, so a
/// templated corpus that merely renames variables is a Type-2 clone family and
/// scores 96.8% duplicated. These differ in control flow and construct.
const DISTINCT_FILES: [(&str, &str); 10] = [
    (
        "f0.rs",
        r#"use std::collections::HashMap;
pub struct Ledger { rows: HashMap<String, i64> }
impl Ledger {
    pub fn post(&mut self, key: &str, amount: i64) {
        *self.rows.entry(key.to_string()).or_insert(0) += amount;
    }
    pub fn balance(&self) -> i64 { self.rows.values().sum() }
}
"#,
    ),
    (
        "f1.rs",
        r#"pub enum Token { Word(String), Number(f64), Punct(char) }
pub fn classify(raw: &str) -> Token {
    match raw.parse::<f64>() {
        Ok(n) => Token::Number(n),
        Err(_) if raw.len() == 1 => Token::Punct(raw.chars().next().unwrap_or('?')),
        Err(_) => Token::Word(raw.to_owned()),
    }
}
"#,
    ),
    (
        "f2.rs",
        r#"pub trait Sink { fn accept(&mut self, line: &str); }
pub struct Counter { pub seen: usize }
impl Sink for Counter {
    fn accept(&mut self, line: &str) { if !line.is_empty() { self.seen += 1 } }
}
"#,
    ),
    (
        "f3.rs",
        r#"pub fn fib(n: u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n { let t = a + b; a = b; b = t; }
    a
}
"#,
    ),
    (
        "f4.rs",
        r#"use std::fmt;
#[derive(Debug)]
pub struct Rect { pub w: f32, pub h: f32 }
impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{} area={}", self.w, self.h, self.w * self.h)
    }
}
"#,
    ),
    (
        "f5.rs",
        r#"pub fn quicksort<T: Ord + Clone>(v: &[T]) -> Vec<T> {
    if v.len() <= 1 { return v.to_vec(); }
    let pivot = v[v.len() / 2].clone();
    let less: Vec<T> = v.iter().filter(|x| **x < pivot).cloned().collect();
    let more: Vec<T> = v.iter().filter(|x| **x > pivot).cloned().collect();
    let mut out = quicksort(&less);
    out.push(pivot);
    out.extend(quicksort(&more));
    out
}
"#,
    ),
    (
        "f6.rs",
        r#"pub const GREETINGS: [&str; 3] = ["hola", "salut", "ciao"];
pub fn greet(idx: usize) -> String {
    GREETINGS.get(idx % GREETINGS.len()).map(|g| format!("{g}!")).unwrap_or_default()
}
"#,
    ),
    (
        "f7.rs",
        r#"pub struct Retry { pub attempts: u8, pub backoff_ms: u64 }
impl Default for Retry {
    fn default() -> Self { Retry { attempts: 3, backoff_ms: 250 } }
}
impl Retry {
    pub fn delay_for(&self, attempt: u8) -> u64 { self.backoff_ms << attempt.min(6) }
}
"#,
    ),
    (
        "f8.rs",
        r#"pub fn hamming(a: &[u8], b: &[u8]) -> Option<u32> {
    if a.len() != b.len() { return None; }
    Some(a.iter().zip(b).map(|(x, y)| (x ^ y).count_ones()).sum())
}
"#,
    ),
    (
        "f9.rs",
        r#"use std::io::{self, BufRead};
pub fn longest_line<R: BufRead>(r: R) -> io::Result<String> {
    let mut best = String::new();
    for line in r.lines() {
        let line = line?;
        if line.len() > best.len() { best = line; }
    }
    Ok(best)
}
"#,
    ),
];

/// Write the ten distinct files into `root`.
fn write_clean_tree(root: &Path) {
    std::fs::create_dir_all(root).expect("mkdir clean tree");
    for (name, body) in DISTINCT_FILES {
        std::fs::write(root.join(name), body).expect("write distinct file");
    }
}

/// Write the ten distinct files into `root` AND a byte-identical copy of each
/// into `root/copy`.
fn write_cloned_tree(root: &Path) {
    write_clean_tree(root);
    let copy = root.join("copy");
    std::fs::create_dir_all(&copy).expect("mkdir copy dir");
    for (name, body) in DISTINCT_FILES {
        std::fs::write(copy.join(name), body).expect("write cloned file");
    }
}

async fn score_tree(root: &Path) -> crate::tdg::ProjectScore {
    let analyzer = TdgAnalyzer::new().expect("analyzer");
    analyzer
        .analyze_project(root)
        .await
        .expect("analyze project")
}

/// RED: on the pre-fix code both trees report a duplication component of exactly
/// 20.0 — full marks for a tree that is 100% duplicated by the very engine
/// `analyze duplicates` reports 88.2% with.
#[tokio::test]
async fn a_cloned_tree_scores_worse_on_duplication_than_a_clean_one() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let clean_root = tmp.path().join("clean");
    let cloned_root = tmp.path().join("cloned");
    write_clean_tree(&clean_root);
    write_cloned_tree(&cloned_root);

    let clean = score_tree(&clean_root).await;
    let cloned = score_tree(&cloned_root).await;

    assert_eq!(clean.total_files, 10, "clean tree file count");
    assert_eq!(cloned.total_files, 20, "cloned tree file count");

    let clean_dup = clean.average().duplication_ratio;
    let cloned_dup = cloned.average().duplication_ratio;

    assert!(
        cloned_dup < clean_dup,
        "a 100%-duplicated tree must score WORSE on duplication than a clean one, \
         got cloned={cloned_dup} clean={clean_dup} (equal means the component was never measured)"
    );

    let clean_total = clean.average_score.expect("clean average");
    let cloned_total = cloned.average_score.expect("cloned average");
    assert!(
        cloned_total < clean_total,
        "the duplicated tree must score worse overall: cloned={cloned_total} clean={clean_total}"
    );
}

/// COUNTER-TEST: "always penalise" must not pass. A genuinely clean tree keeps
/// full marks on duplication, so the fix cannot be a blanket deduction.
#[tokio::test]
async fn a_clean_tree_keeps_full_duplication_marks() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("clean");
    write_clean_tree(&root);

    let project = score_tree(&root).await;
    let dup = project.average().duplication_ratio;

    assert!(
        (dup - 20.0).abs() < f32::EPSILON,
        "ten structurally distinct files are not duplicated; component must stay at 20.0, got {dup}"
    );
    let total = project.average_score.expect("average");
    assert!(
        total > 95.0,
        "a clean tree must still grade well, got {total}"
    );
}

/// COUNTER-TEST: the penalty is bounded by the component's own weight, so a
/// pathological tree cannot drive the component — or the total — negative.
#[tokio::test]
async fn the_duplication_penalty_is_bounded_at_the_component_weight() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("pathological");
    std::fs::create_dir_all(&root).expect("mkdir");
    // Forty byte-identical copies: the worst case the ratio can express.
    for i in 0..40 {
        std::fs::write(root.join(format!("f{i}.rs")), DISTINCT_FILES[5].1).expect("write clone");
    }

    let project = score_tree(&root).await;
    let dup = project.average().duplication_ratio;
    let total = project.average_score.expect("average");

    assert!(
        (0.0..=20.0).contains(&dup),
        "duplication component must stay within its 0..=20 weight, got {dup}"
    );
    assert!(
        dup < 1.0,
        "a 40x-duplicated tree must lose essentially the whole component, got {dup}"
    );
    assert!(total >= 0.0, "total must never go negative, got {total}");
}

/// COUNTER-TEST: single-file analysis still produces a score, and cross-file
/// duplication must NOT leak into it.
///
/// The fixture puts TWO BYTE-IDENTICAL FILES SIDE BY SIDE and grades one of
/// them. That placement is the whole test: an earlier version of it graded a
/// file whose clones lived in a subdirectory, and the mutation this is meant to
/// catch — "analyze_file runs the project detector over its parent directory" —
/// passed against it, because the parent directory held no clones. A guard that
/// its own named mutation walks straight through is not a guard.
///
/// Cross-file duplication is undefined for a population of one, so the honest
/// answer here is the within-file measurement: `dupe_a.rs` repeats nothing
/// inside itself, so it keeps the full component even though `dupe_b.rs` next to
/// it is the same bytes.
#[tokio::test]
async fn a_single_file_still_scores_and_is_unaffected_by_cross_file_duplication() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("pair");
    std::fs::create_dir_all(&root).expect("mkdir");
    let body = DISTINCT_FILES[5].1;
    std::fs::write(root.join("dupe_a.rs"), body).expect("write a");
    std::fs::write(root.join("dupe_b.rs"), body).expect("write b");

    // The PROJECT verdict over this pair is heavily duplicated ...
    let project = score_tree(&root).await;
    assert!(
        project.average().duplication_ratio < 20.0,
        "the pair IS duplicated at project scope, got {}",
        project.average().duplication_ratio
    );

    // ... while the SINGLE FILE keeps its within-file measurement.
    let analyzer = TdgAnalyzer::new().expect("analyzer");
    let one = analyzer
        .analyze_file(&root.join("dupe_a.rs"))
        .await
        .expect("single file score");

    assert!(
        one.total > 0.0,
        "single file must still score, got {}",
        one.total
    );
    assert!(
        (one.duplication_ratio - 20.0).abs() < f32::EPSILON,
        "dupe_a.rs repeats nothing WITHIN itself, so single-file mode reports the full \
         component; it must not borrow the project's cross-file verdict. got {}",
        one.duplication_ratio
    );
}

/// A tree TDG grades but the clone engine has no tokenizer for must report that
/// the duplication component was NOT MEASURED. "We could not look" must never be
/// rendered as the clean 20/20 that "we looked and found nothing" earns — that
/// is the #1050 defect itself, one level up.
#[tokio::test]
async fn a_language_the_clone_engine_cannot_read_is_disclosed_as_unmeasured() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("goproj");
    std::fs::create_dir_all(&root).expect("mkdir");
    // Go is graded by TDG and has no clone tokenizer. Two byte-identical files,
    // so a detector that COULD read them would report heavy duplication.
    let body = "package main\n\nfunc Add(a int, b int) int {\n\treturn a + b\n}\n\nfunc Mul(a int, b int) int {\n\treturn a * b\n}\n";
    std::fs::write(root.join("a.go"), body).expect("write");
    std::fs::write(root.join("b.go"), body).expect("write");

    let project = score_tree(&root).await;
    assert!(project.total_files >= 2, "Go files should still be graded");

    assert!(
        project.cross_file_duplication_ratio.is_none(),
        "an unreadable population must not produce a ratio, got {:?}",
        project.cross_file_duplication_ratio
    );
    assert!(
        project
            .not_measured
            .iter()
            .any(|f| f == "cross_file_duplication_ratio"),
        "the gap must be NAMED in not_measured, got {:?}",
        project.not_measured
    );
    let reason = project
        .cross_file_duplication_unmeasured
        .expect("a reason must accompany the gap");
    assert!(
        reason.contains("clone tokenizer"),
        "the reason must say why, got {reason}"
    );
}

/// The measured ratio is recorded on the project score even when it is clean, so
/// a reader can tell "measured, and it was 0%" from "never measured".
#[tokio::test]
async fn a_measured_clean_tree_records_its_ratio_rather_than_staying_silent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("clean");
    write_clean_tree(&root);

    let project = score_tree(&root).await;
    assert_eq!(
        project.cross_file_duplication_ratio,
        Some(0.0),
        "a clean tree was MEASURED at 0.0, which is not the same as unmeasured"
    );
    assert!(
        project.not_measured.is_empty(),
        "nothing is unmeasured here, got {:?}",
        project.not_measured
    );
    assert!(project.cross_file_duplication_unmeasured.is_none());
}

/// A tree the clone engine can read only PART of must publish how much of it
/// the duplication verdict actually covers. A ratio over the Rust half of a
/// Rust+Go repo is not that repo's duplication, and a reader who cannot see the
/// covered population will take it for the whole one.
#[tokio::test]
async fn a_partly_readable_tree_discloses_how_much_it_covered() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("mixed");
    write_clean_tree(&root); // 10 Rust files, all readable
                             // Two Go files TDG grades and the clone engine cannot tokenize.
    let go = "package main\n\nfunc Add(a int, b int) int {\n\treturn a + b\n}\n";
    std::fs::write(root.join("a.go"), go).expect("write a.go");
    std::fs::write(root.join("b.go"), go).expect("write b.go");

    let project = score_tree(&root).await;
    let coverage = project
        .cross_file_duplication_coverage
        .expect("coverage must be recorded");
    let (measured, total) = (coverage.measured, coverage.total);

    assert_eq!(
        total, project.total_files,
        "total must be the graded population"
    );
    assert!(
        measured < total,
        "the Go files are not readable by the clone engine, so coverage must be \
         partial: measured={measured} total={total}"
    );
    assert_eq!(measured, 10, "the ten Rust files are the readable subset");
    // The ratio IS measured -- over that subset -- so it is not null, and the
    // coverage numbers are what keep it from being read as whole-tree.
    assert!(project.cross_file_duplication_ratio.is_some());
}
