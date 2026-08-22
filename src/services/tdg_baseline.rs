//! CB-200's ratchet baseline, re-derived rather than transcribed.
//!
//! # The decision this implements
//!
//! CB-200 (`check_handlers/check_tdg_grade.rs`) fails if a definition in
//! `.pmat/context.db` grades below the floor. Roughly 1,900 do, across roughly
//! a thousand files, at most a dozen in any one — a flat distribution with no
//! hotspot and no bounded refactor. The floor STAYS `A` and no exclude glob is
//! added; both are threshold-lowering in disguise, and one of them
//! (187f506885) is how a past "pass" was manufactured. Instead the gate is a
//! RATCHET on its own measured baseline, recorded as `[tdg] baseline` in
//! `.pmat-gates.toml`: any NEW below-floor definition fails it, the baseline
//! may only decrease, and the absolute count is reported on every outcome so
//! "passing" can never be mistaken for "clean".
//!
//! # What this module adds
//!
//! The gate re-derives its count on every run, so the baseline cannot rot
//! unnoticed *while the gate runs*. This module is what makes that true at the
//! merge gate, where `pmat comply check` does not run:
//!
//! 1. it re-derives the count by RUNNING CB-200 and reading its verdict;
//! 2. it reads the recorded baseline out of `.pmat-gates.toml` INDEPENDENTLY of
//!    the gate, and asserts the two readings agree — so a baseline the gate
//!    stops honouring, or one this test misreads, is a failure rather than a
//!    quiet agreement between two copies of the same mistake;
//! 3. it asserts the recorded baseline EQUALS the measurement. Not `<=`: a
//!    baseline the tree has already beaten is headroom for new debt to hide in.
//!
//! # Why re-derived and never transcribed
//!
//! `.pmat-metrics.toml:45` carries `max_unwrap_calls = 100` annotated
//! `Current: 570` in a tree that measures 20,390. Three numbers, no two of
//! which agree, and a green build throughout, because nothing re-runs any of
//! them. `.pmat-ratchet.toml`'s header states the rule: a number in a config
//! file that nobody re-runs is not a gate; it is a wish with a colon after it.
//!
//! The same header states the subtler half, and this module exists because of
//! it: THE SCOPE PREDICATE IS THE METRIC. [`measure_below_floor`] therefore
//! calls `check_tdg_grade_gate` with the config `compute_compliance_report`
//! loads, and re-implements none of the passing-grade set, the test-path filter
//! or the exclude union. Two implementations of one scope is the defect this
//! module exists to prevent: CB-200 was BLIND until 2026-08-20 because a
//! five-letter reader `["A","B","C","D","F"]` met an eleven-letter writer
//! through SQL `IN`, so `A-`, `B+` and `C-` matched nothing. It saw 247
//! violations and could not see 1,719, and every historical "CB-200 passed"
//! came from that version.
//!
//! # Why a `--lib` test
//!
//! Merge CI runs `cargo test --lib` and nothing else; `tests/all.rs` is not
//! built there. A check living in `tests/` would not execute at the gate that
//! decides a merge.
//!
//! # What runs where
//!
//! `.pmat/context.db` is gitignored (`**/.pmat/`) and no CI leg builds one, so
//! the live count is measurable on developer machines and agent runs, and not
//! on a fresh checkout. The tests enumerate that rather than papering over it:
//!
//! - index present — the full ratchet runs, and a silent zero is refused.
//! - index absent — the ratchet cannot run, and what is asserted instead is the
//!   one thing that IS true there: CB-200 answers "Not measured", never a Pass
//!   (#939). `the_committed_baseline_is_the_measured_count_strict` is the
//!   `#[ignore]`d twin that refuses absence outright, matching the convention
//!   in `services/ttg/differential.rs`.
//! - the floor and the exclude set are committed facts checked on every
//!   machine, index or no index, so the cheapest way out of a red ratchet —
//!   lower the floor, or add one more glob — cannot be taken quietly.

use crate::cli::handlers::comply_handlers::check_handlers::check_tdg_grade::check_tdg_grade_gate;
use crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus;
use crate::models::comply_config::PmatYamlConfig;
use std::path::Path;

/// The floor CB-200 measures against. Not a knob: the ratchet exists so that
/// this stays where it is.
pub const FLOOR: &str = "A";

/// Every exclude glob CB-200 honours, as committed today — the union of
/// `.pmat.yaml`'s `tdg_exclude_paths` and `.pmat-gates.toml`'s `[tdg].exclude`.
///
/// A glob added to either file shrinks the population CB-200 measures, which
/// lowers the count without a single grade improving. Pinning the set here
/// makes that edit fail a test instead of arriving as a quiet improvement.
pub const COMMITTED_EXCLUDES: &[&str] = &[
    // .pmat.yaml -> comply.thresholds.tdg_exclude_paths
    "examples/*",
    "src/tdg/analyzer_ast/analyzer_impl2_heuristics_lean.rs",
    "tests/*",
    "benches/*",
    // .pmat-gates.toml -> [tdg].exclude
    "examples/**",
    "scripts/**",
    "benches/**",
    "src/cli/command_dispatcher/**",
    "src/cli/command_structure.rs",
];

/// A completed measurement.
pub struct Measurement {
    /// Definitions CB-200 counted below the floor, after its own filters.
    pub below_floor: usize,
    /// Definitions the index holds at all. A below-floor count of 0 means one
    /// thing against 20,000 indexed definitions and something else entirely
    /// against 0 — this is what tells them apart.
    pub indexed: usize,
    /// `[tdg] baseline` in `.pmat-gates.toml`, read here rather than taken from
    /// the gate. `None` when the project records none, which CB-200 treats as
    /// zero tolerance.
    pub recorded: Option<usize>,
    /// The baseline CB-200 says it used, recovered from its own verdict. Must
    /// equal [`Measurement::recorded`] wherever the verdict states one: two
    /// readers of one number that never disagree is the property being bought.
    pub reported: Option<usize>,
    /// CB-200's verdict text, which names the count and the first offenders.
    pub detail: String,
}

/// The outcome of asking CB-200 for its count.
///
/// `Unmeasurable` is deliberately not `Ok(0)`. A rotted query and a clean tree
/// are byte-identical at the count, and this repository has been bitten by
/// exactly that: a `git grep` pathspec that no longer matches anything prints
/// `0` just like a genuine zero does.
pub enum Measured {
    /// CB-200 answered, over an index that holds something.
    Ok(Measurement),
    /// Nothing was measured, and this is why.
    Unmeasurable(String),
}

/// The digits immediately preceding `head`'s end, if any.
fn trailing_count(head: &str) -> Option<usize> {
    head.rsplit(|c: char| !c.is_ascii_digit())
        .next()
        .filter(|digits| !digits.is_empty())
        .and_then(|digits| digits.parse().ok())
}

/// How many definitions CB-200 reports below the floor, from its verdict text.
///
/// `ComplianceCheck` carries no structured count, so the absolute number is
/// read back from the message CB-200 is required to lead with. A wording change
/// that drops the count fails loudly here rather than silently removing the
/// ratchet's subject — which is the correct outcome, because a gate that stops
/// printing its absolute count is how ~1,900 accumulated unseen in the first
/// place. It already earned its keep once: CB-200's ratchet rewrite renamed
/// `function(s)` to `definition(s)` mid-flight, and this reported
/// "CB-200's verdict carries no absolute count" on the next run rather than
/// defaulting to zero.
///
/// Both nouns are accepted because CB-200 emits both: the zero-tolerance path
/// (no recorded baseline — every other project using pmat) still says
/// `function(s)`.
fn below_floor_from_verdict(message: &str) -> Option<usize> {
    for marker in [" definition", " function"] {
        if let Some((head, _)) = message.split_once(marker) {
            if let Some(count) = trailing_count(head) {
                return Some(count);
            }
        }
    }
    if message.starts_with("All non-test functions meet minimum grade") {
        return Some(0);
    }
    None
}

/// The baseline CB-200 says it compared against, from its own verdict.
fn reported_baseline_from_verdict(message: &str) -> Option<usize> {
    let (_, tail) = message.split_once("recorded baseline of ")?;
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// `[tdg] baseline` from `.pmat-gates.toml`, read independently of the gate.
///
/// Deliberately a second reader of one number, and deliberately NOT a second
/// implementation of the scope predicate. The number is the thing a
/// transcription can corrupt; the scope is the thing a re-implementation can
/// corrupt. This guards the first, and calling CB-200 guards the second.
pub fn recorded_baseline(project_path: &Path) -> Option<usize> {
    let text = std::fs::read_to_string(project_path.join(".pmat-gates.toml")).ok()?;
    let table: toml::Table = text.parse().ok()?;
    let raw = table.get("tdg")?.get("baseline")?.as_integer()?;
    usize::try_from(raw).ok()
}

/// Definitions in the index, or `None` when it cannot be read.
///
/// Not a second copy of CB-200's scope predicate — no grade set, no path
/// filter, no globs. It answers only "does this index hold anything", the
/// question that separates a genuine improvement from a rotted measurement.
fn indexed_definitions(db_path: &Path) -> Option<usize> {
    let conn = rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()?;
    conn.query_row("SELECT COUNT(*) FROM functions", [], |row| {
        row.get::<_, i64>(0)
    })
    .ok()
    .map(|n| n as usize)
}

/// Re-derive the below-floor count by RUNNING CB-200 over `project_path`.
///
/// The scope — passing spellings, test-path filter, the union of `.pmat.yaml`'s
/// `tdg_exclude_paths` with `.pmat-gates.toml`'s `[tdg].exclude` — is CB-200's,
/// because this calls CB-200. The config is loaded exactly as
/// `compute_compliance_report` loads it, which is the only config the gate is
/// ever run under in production. Measuring under `ComplyConfig::default()`
/// instead silently drops `.pmat.yaml`'s excludes and answers a different
/// question.
pub fn measure_below_floor(project_path: &Path) -> Measured {
    let db_path = project_path.join(".pmat").join("context.db");
    let yaml_config = PmatYamlConfig::load(project_path).unwrap_or_default();
    let verdict = check_tdg_grade_gate(project_path, &yaml_config.comply);
    if verdict.status == CheckStatus::Skip {
        return Measured::Unmeasurable(verdict.message);
    }
    let Some(indexed) = indexed_definitions(&db_path) else {
        return Measured::Unmeasurable(format!(
            "cannot count definitions in {} — an unreadable index is not the same as a \
             tree with nothing to report",
            db_path.display()
        ));
    };
    if indexed == 0 {
        return Measured::Unmeasurable(format!(
            "{} holds 0 definitions — an empty index reports 0 below the floor for the \
             same reason a clean tree does. Rebuild it with `pmat query \"x\"`.",
            db_path.display()
        ));
    }
    match below_floor_from_verdict(&verdict.message) {
        Some(below_floor) => Measured::Ok(Measurement {
            below_floor,
            indexed,
            recorded: recorded_baseline(project_path),
            reported: reported_baseline_from_verdict(&verdict.message),
            detail: verdict.message,
        }),
        None => Measured::Unmeasurable(format!(
            "CB-200's verdict carries no absolute count, so the ratchet has no subject. \
             The count must lead the sentence on every outcome, passing or failing. \
             Verdict was: {}",
            verdict.message
        )),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// The six columns CB-200's SELECT names. Deliberately the minimum: a
    /// fixture mirroring the whole 22-column production schema would keep
    /// passing while the query drifted off it.
    const FIXTURE_SCHEMA: &str = "CREATE TABLE functions (id INTEGER PRIMARY KEY, \
         file_path TEXT NOT NULL, function_name TEXT NOT NULL, \
         tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, \
         start_line INTEGER NOT NULL DEFAULT 0)";

    /// A project whose index holds exactly `rows` of `(path, name, grade)`.
    fn index_with(rows: &[(&str, &str, &str)]) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let conn = rusqlite::Connection::open(pmat_dir.join("context.db")).expect("open db");
        conn.execute_batch(FIXTURE_SCHEMA).expect("schema");
        for (i, (path, name, grade)) in rows.iter().enumerate() {
            conn.execute(
                "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, \
                 start_line) VALUES (?1, ?2, ?3, 40, ?4)",
                rusqlite::params![path, name, grade, i as i64 + 1],
            )
            .expect("insert row");
        }
        drop(conn);
        tmp
    }

    /// `false`, named for the one thing it is used to say: this branch reached
    /// a state in which nothing was measured, so the assertion must fail
    /// carrying its reason. A literal `assert!(false, ..)` trips
    /// `clippy::assertions_on_constants`.
    fn nothing_was_measured() -> bool {
        false
    }

    /// The count this module reads back is the count CB-200 arrived at.
    ///
    /// Fixture-checked rather than assumed, because the number crosses a
    /// string. It also pins the two scope decisions most easily lost: a
    /// MODIFIED grade counts (`B+` is below a floor of `A`, and matching it is
    /// exactly what the five-letter reader could not do), and a test path does
    /// not.
    ///
    /// RED, mutating `below_floor_from_verdict` to `return Some(0)` before it
    /// reads the digits:
    /// ```text
    /// assertion `left == right` failed: CB-200 counted 3 offenders
    ///   left: 0
    ///  right: 3
    /// ```
    /// Note which test does NOT catch that mutation:
    /// `a_clean_index_measures_zero_rather_than_refusing` stays green, because
    /// 0 is the right answer there. A fixture with real offenders in it is the
    /// only thing that pins the number to something.
    #[test]
    fn the_count_read_back_is_the_count_cb200_counted() {
        let project = index_with(&[
            ("src/good.rs", "fine", "A"),
            ("src/legacy.rs", "bad", "D"),
            ("src/awful.rs", "terrible", "F"),
            ("src/modified.rs", "nearly", "B+"),
            ("src/tests/helpers.rs", "helper", "D"),
        ]);
        match measure_below_floor(project.path()) {
            Measured::Ok(m) => {
                assert_eq!(m.below_floor, 3, "CB-200 counted 3 offenders");
                assert_eq!(m.indexed, 5, "the index holds 5 definitions");
                assert!(
                    m.detail.contains("bad") && m.detail.contains("nearly"),
                    "the verdict must name its offenders: {}",
                    m.detail
                );
                assert!(
                    !m.detail.contains("helper"),
                    "a test path is not a violation: {}",
                    m.detail
                );
            }
            Measured::Unmeasurable(why) => assert!(
                nothing_was_measured(),
                "a populated index measured nothing: {why}"
            ),
        }
    }

    /// The recorded baseline is read the same way CB-200 reads it, on a value
    /// that is nobody's default.
    ///
    /// Two readers of one number are only worth having if they are checked
    /// against each other; otherwise they are two places for the same
    /// transcription to live. `5` is chosen because it is neither `0` nor the
    /// count, so a reader that quietly returns either is caught.
    ///
    /// RED, mutating `recorded_baseline` to `Some(0)`:
    /// ```text
    /// assertion `left == right` failed: the two readers of `[tdg] baseline` disagree
    ///   left: Some(0)
    ///  right: Some(5)
    /// ```
    #[test]
    fn the_recorded_baseline_is_read_the_same_way_cb200_reads_it() {
        let project = index_with(&[
            ("src/a.rs", "one", "D"),
            ("src/b.rs", "two", "F"),
            ("src/c.rs", "three", "C-"),
        ]);
        std::fs::write(
            project.path().join(".pmat-gates.toml"),
            "[tdg]\nbaseline = 5\n",
        )
        .expect("write gates toml");

        match measure_below_floor(project.path()) {
            Measured::Ok(m) => {
                assert_eq!(m.below_floor, 3);
                assert_eq!(
                    m.recorded, m.reported,
                    "the two readers of `[tdg] baseline` disagree"
                );
                assert_eq!(m.recorded, Some(5), "the recorded baseline is 5");
                assert!(
                    m.detail.contains("lower `[tdg] baseline` to 3"),
                    "a beaten baseline must name the number to write: {}",
                    m.detail
                );
            }
            Measured::Unmeasurable(why) => assert!(
                nothing_was_measured(),
                "a populated index measured nothing: {why}"
            ),
        }
    }

    /// A baseline that is not a count is not a baseline, and CB-200 refuses it
    /// rather than reading it as a bound nothing exceeds. There is no count in
    /// that verdict, so there is nothing here to ratchet either.
    #[test]
    fn an_unreadable_baseline_leaves_nothing_to_measure() {
        let project = index_with(&[("src/a.rs", "one", "D")]);
        std::fs::write(
            project.path().join(".pmat-gates.toml"),
            "[tdg]\nbaseline = \"lots\"\n",
        )
        .expect("write gates toml");
        assert!(
            recorded_baseline(project.path()).is_none(),
            "\"lots\" is not a recorded baseline"
        );
        match measure_below_floor(project.path()) {
            Measured::Unmeasurable(why) => assert!(
                why.contains("not a count"),
                "the refusal must carry CB-200's reason: {why}"
            ),
            Measured::Ok(m) => assert!(
                nothing_was_measured(),
                "an unreadable baseline yielded a measurement of {}",
                m.below_floor
            ),
        }
    }

    /// Zero is reachable when it is honest — the counter-test to the refusals
    /// below, so that "0 is suspicious" did not quietly become "0 is
    /// impossible".
    #[test]
    fn a_clean_index_measures_zero_rather_than_refusing() {
        let project = index_with(&[
            ("src/one.rs", "a", "A"),
            ("src/two.rs", "b", "A+"),
            ("src/three.rs", "c", "A"),
        ]);
        match measure_below_floor(project.path()) {
            Measured::Ok(m) => {
                assert_eq!(m.below_floor, 0, "nothing in this index is below an A");
                assert_eq!(m.indexed, 3);
            }
            Measured::Unmeasurable(why) => assert!(
                nothing_was_measured(),
                "a genuine zero over a populated index must measure, not refuse: {why}"
            ),
        }
    }

    /// Silence is not a pass. An index that holds nothing, and one that is not
    /// there at all, both report 0 below the floor for the same reason a clean
    /// tree does.
    ///
    /// RED, removing the `indexed == 0` guard: the empty-index case returns
    /// `Ok(0)` and this fails with
    /// `an empty index must be UNMEASURABLE, not a pass at 0`.
    #[test]
    fn an_empty_or_absent_index_is_unmeasurable_not_zero() {
        let empty = index_with(&[]);
        assert!(
            matches!(measure_below_floor(empty.path()), Measured::Unmeasurable(_)),
            "an empty index must be UNMEASURABLE, not a pass at 0"
        );

        let nothing = tempfile::tempdir().expect("create tempdir");
        match measure_below_floor(nothing.path()) {
            Measured::Unmeasurable(why) => assert!(
                why.contains("Not measured"),
                "an absent index must say so in CB-200's own words: {why}"
            ),
            Measured::Ok(m) => assert!(
                nothing_was_measured(),
                "a project with no index reported {} below the floor",
                m.below_floor
            ),
        }
    }

    /// Every shape of verdict CB-200 can emit, and what each yields.
    ///
    /// The two nouns are not belt-and-braces: `definition(s)` is the ratchet
    /// path and `function(s)` is the zero-tolerance path that every other
    /// project using pmat still takes.
    #[test]
    fn a_verdict_without_a_count_is_not_a_count() {
        // Ratchet paths.
        assert_eq!(
            below_floor_from_verdict(
                "1904 definition(s) below minimum grade A across 1052 file(s), at the \
                 recorded baseline of 1905 — this is debt held flat, not a clean tree."
            ),
            Some(1904)
        );
        assert_eq!(
            below_floor_from_verdict(
                "0 definitions below minimum grade A, against a recorded baseline of 12."
            ),
            Some(0)
        );
        assert_eq!(
            below_floor_from_verdict(
                "1910 definition(s) below minimum grade A — 5 OVER the recorded baseline of 1905."
            ),
            Some(1910)
        );
        // Zero-tolerance paths, unchanged for every project that records no
        // baseline.
        assert_eq!(
            below_floor_from_verdict("3 function(s) below minimum grade A"),
            Some(3)
        );
        assert_eq!(
            below_floor_from_verdict(
                "All non-test functions meet minimum grade A (7 test/excluded functions skipped)"
            ),
            Some(0)
        );
        // No count in the sentence: refused, never defaulted to zero.
        //
        // The vacuous floor is the one that matters. `min_grade = F` admits
        // every spelling, so CB-200 returns Pass without counting anything;
        // reading that as 0 would let a one-line config edit retire the ratchet.
        assert_eq!(
            below_floor_from_verdict("Minimum grade F \u{2014} no grades below threshold"),
            None
        );
        assert_eq!(
            below_floor_from_verdict("Failed to open context.db: disk I/O error"),
            None
        );

        // And the baseline recovered from the verdict, which must not confuse
        // itself with the count that precedes it.
        assert_eq!(
            reported_baseline_from_verdict(
                "1904 definition(s) below minimum grade A across 1052 file(s), at the \
                 recorded baseline of 1905 — this is debt held flat."
            ),
            Some(1905)
        );
        assert_eq!(
            reported_baseline_from_verdict("3 function(s) below minimum grade A"),
            None
        );
    }

    /// The floor and the exclude set are committed facts, checked wherever the
    /// tests run — index or no index.
    ///
    /// This is the half of the ratchet a fresh checkout CAN enforce, and it
    /// guards the cheapest ways out of a red gate: lower `min_tdg_grade`, or
    /// add one more glob until the count falls. Commit 187f506885 is how a past
    /// "CB-200 passed" was manufactured.
    ///
    /// RED, appending `"vendor/**"` to `COMMITTED_EXCLUDES` — which is the same
    /// diff, seen from the other side, as adding one glob to `.pmat-gates.toml`
    /// and saying nothing:
    /// ```text
    /// assertion `left == right` failed: the exclude set has changed. ...
    ///   left: [..., "src/cli/command_structure.rs"]
    ///  right: [..., "src/cli/command_structure.rs", "vendor/**"]
    /// ```
    #[test]
    fn the_floor_and_the_exclude_set_are_the_committed_ones() {
        let root = repo_root();
        let yaml = PmatYamlConfig::load(&root).expect(".pmat.yaml must parse");
        assert_eq!(
            yaml.comply.thresholds.min_tdg_grade, FLOOR,
            "the TDG floor moved. The ratchet exists so that it does not: a lower floor \
             retires every violation at once without fixing one of them."
        );

        let gates = std::fs::read_to_string(root.join(".pmat-gates.toml"))
            .expect(".pmat-gates.toml must exist");
        let gates: toml::Table = gates.parse().expect(".pmat-gates.toml must parse");
        let tdg = gates.get("tdg");
        assert!(
            tdg.and_then(|t| t.get("min_grade")).is_none(),
            ".pmat-gates.toml now overrides the TDG floor, and that override silently wins \
             over .pmat.yaml"
        );

        let mut on_disk: Vec<String> = yaml.comply.thresholds.tdg_exclude_paths.clone();
        on_disk.extend(
            tdg.and_then(|t| t.get("exclude"))
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        );
        let committed: Vec<String> = COMMITTED_EXCLUDES.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            on_disk, committed,
            "the exclude set has changed. An exclude shrinks the population CB-200 measures, \
             so the count falls without a single grade improving. If the new glob is \
             justified, say why in the commit and update COMMITTED_EXCLUDES in the same \
             change — then re-derive `[tdg] baseline`, because it no longer means what it \
             meant."
        );
    }

    /// CB-200's first line — the count, and any staleness note attached to it.
    ///
    /// Staleness decides what a number MEANS: the count describes the tree the
    /// index was built from, not necessarily the one on disk. CB-200 reports it
    /// rather than refusing (#939), and so does this — refusing would make the
    /// ratchet unrunnable on any machine where somebody has edited a file since
    /// the last `pmat query`.
    fn headline(detail: &str) -> &str {
        detail.lines().next().unwrap_or(detail)
    }

    /// Assert the ratchet over a measurement, saying what to do in either
    /// direction.
    fn assert_ratchet_holds(m: &Measurement) {
        assert!(
            m.indexed > 1000,
            "the index holds only {} definitions and this repository has tens of thousands. \
             It is truncated or half-built, so its count means nothing. Rebuild it with \
             `pmat query \"x\"` and re-run.",
            m.indexed
        );
        assert_eq!(
            m.recorded, m.reported,
            "two readers of `[tdg] baseline` disagree: this test read {:?} from \
             .pmat-gates.toml, CB-200 says it compared against {:?}. One of them is reading \
             a number nothing honours.",
            m.recorded, m.reported
        );
        let Some(recorded) = m.recorded else {
            assert!(
                nothing_was_measured(),
                "this repository records no `[tdg] baseline` in .pmat-gates.toml, so CB-200 \
                 has no ratchet to hold and {} definitions below grade {} are failing it \
                 outright. CB-200 said: {}",
                m.below_floor,
                FLOOR,
                headline(&m.detail)
            );
            return;
        };
        assert!(
            m.below_floor <= recorded,
            "CB-200 REGRESSED: {} definitions below grade {}, recorded baseline {}. \
             {} new one(s) since the baseline was taken. Fix them, or revert what added \
             them. Raising `[tdg] baseline` is not the fix — a baseline may only go down, \
             and a raise needs a written justification reviewed against the previous \
             committed version of .pmat-gates.toml. CB-200 said:\n{}",
            m.below_floor,
            FLOOR,
            recorded,
            m.below_floor - recorded,
            m.detail
        );
        assert!(
            m.below_floor >= recorded,
            "the tree has already BEATEN the recorded baseline and the number was never \
             updated: measured {}, recorded {}. Slack in a ratchet is headroom for a \
             regression to hide in. Set `[tdg] baseline = {}` in .pmat-gates.toml and \
             commit it. CB-200 said: {}",
            m.below_floor,
            recorded,
            m.below_floor,
            headline(&m.detail)
        );
    }

    /// The live assertion: the recorded baseline IS the count CB-200 takes at
    /// HEAD, over this repository's own index.
    ///
    /// Two honest outcomes are enumerated, and only one of them exercises the
    /// ratchet:
    ///
    /// - index present — the ratchet runs at full strength, in both directions,
    ///   and a truncated index is refused rather than read as an improvement.
    /// - index absent — `.pmat/` is gitignored and no CI leg builds one, so
    ///   there is genuinely nothing to count. What is asserted instead is the
    ///   thing that is true there and load-bearing: CB-200 answers "Not
    ///   measured", never Pass (#939). `..._strict` below refuses this case
    ///   outright for anyone who wants the unconditional check.
    ///
    /// RED at the commit that added this file, and not by contrivance: the
    /// recorded baseline was 1905, taken under `ComplyConfig::default()`, while
    /// the config `pmat comply check` actually loads excludes one further file
    /// (`.pmat.yaml` names `analyzer_impl2_heuristics_lean.rs`, which holds
    /// exactly one below-A definition). Measured under the production config it
    /// is 1904, and this printed:
    /// ```text
    /// the tree has already BEATEN the recorded baseline and the number was
    /// never updated: measured 1904, recorded 1905. ... Set
    /// `[tdg] baseline = 1904` in .pmat-gates.toml and commit it.
    /// ```
    /// The other direction was proved by pinning BOTH baseline readers to 1903
    /// (a mutation, rather than editing `.pmat-gates.toml`, so the two-reader
    /// check stays satisfied and the ratchet branch is what fires):
    /// ```text
    /// CB-200 REGRESSED: 1904 definitions below grade A, recorded baseline
    /// 1903. 1 new one(s) since the baseline was taken. ...
    /// ```
    /// A baseline derived under a different config than the gate runs under is
    /// precisely the failure `.pmat-ratchet.toml`'s header warns about: the
    /// scope predicate IS the metric.
    #[test]
    fn the_committed_baseline_is_the_measured_count() {
        let root = repo_root();
        match measure_below_floor(&root) {
            Measured::Ok(m) => assert_ratchet_holds(&m),
            Measured::Unmeasurable(why) => {
                let db_path = root.join(".pmat").join("context.db");
                assert!(
                    !db_path.exists(),
                    "the index at {} exists and the ratchet still could not be measured. \
                     That is a broken measurement, not a clean tree: {why}",
                    db_path.display()
                );
                assert!(
                    why.contains("Not measured"),
                    "an unbuilt index must be reported as unmeasured, in CB-200's own words. \
                     Build one with `pmat query \"x\"` to run the ratchet here. CB-200 \
                     said: {why}"
                );
                let yaml = PmatYamlConfig::load(&root).unwrap_or_default();
                let verdict = check_tdg_grade_gate(&root, &yaml.comply);
                assert!(
                    verdict.status != CheckStatus::Pass,
                    "CB-200 reported a PASS with no index to read. An audit that measured \
                     nothing must never read as one that found nothing: {}",
                    verdict.message
                );
            }
        }
    }

    /// The unconditional twin. Fails rather than tolerating an absent index,
    /// which is the convention `services/ttg/differential.rs` established for
    /// checks that need a built `.pmat/context.db`.
    ///
    /// ```text
    /// env -u RUST_MIN_STACK cargo test --lib \
    ///     tdg_baseline::tests::the_committed_baseline_is_the_measured_count_strict \
    ///     -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "needs a built .pmat/context.db; see the module docs"]
    fn the_committed_baseline_is_the_measured_count_strict() {
        let root = repo_root();
        match measure_below_floor(&root) {
            Measured::Ok(m) => {
                eprintln!(
                    "CB-200: {} of {} indexed definitions are below grade {} (recorded \
                     baseline {:?})",
                    m.below_floor, m.indexed, FLOOR, m.recorded
                );
                assert_ratchet_holds(&m);
            }
            Measured::Unmeasurable(why) => assert!(
                nothing_was_measured(),
                "nothing was measured, so nothing passed. Build the index with \
                 `pmat query \"x\"` and re-run. Reason: {why}"
            ),
        }
    }
}
