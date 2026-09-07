//! BSE-12 / PMAT-707: the pre-commit hook's verdict is a function of the
//! diff's touched functions only.
//!
//! Every fixture here is an in-memory source pair (HEAD version, staged
//! version) plus the `git diff --cached -U0` hunk headers that connect them,
//! so the tests measure the VERDICT rule and not git.

use super::{
    diff_scoped_verdict, measure_source, parse_touched_ranges, touched_functions, DebtThresholds,
    DebtVerdict, MeasuredFn,
};

/// Only cyclomatic drives these fixtures: `max_cognitive` is set out of reach
/// so an assertion on a printed number is about one metric, not two.
const LIMITS: DebtThresholds = DebtThresholds {
    max_cyclomatic: 5,
    max_cognitive: 100,
};

/// Lines 1-10: `debted`, cyclomatic 7 (1 + six `if`s) — over the limit of 5
/// and already committed. Lines 12-14: `innocent`, cyclomatic 1.
const HEAD_VERSION: &str = "\
fn debted(x: i32) -> i32 {
    let mut n = 0;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    if x > 5 { n += 1; }
    n
}

fn innocent(x: i32) -> i32 {
    x + 1
}
";

/// The same file with ONE line changed, inside `innocent` (line 13).
const ONE_LINE_IN_INNOCENT: &str = "\
fn debted(x: i32) -> i32 {
    let mut n = 0;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    if x > 5 { n += 1; }
    n
}

fn innocent(x: i32) -> i32 {
    x + 2
}
";

/// `innocent` grows to cyclomatic 6 — debt written by THIS commit.
const GROWTH_IN_INNOCENT: &str = "\
fn debted(x: i32) -> i32 {
    let mut n = 0;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    if x > 5 { n += 1; }
    n
}

fn innocent(x: i32) -> i32 {
    let mut n = x;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    n
}
";

/// A brand-new function, over the limit, appended at line 16.
const NEW_OVER_THRESHOLD_FN: &str = "\
fn debted(x: i32) -> i32 {
    let mut n = 0;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    if x > 5 { n += 1; }
    n
}

fn innocent(x: i32) -> i32 {
    x + 1
}

fn fresh(x: i32) -> i32 {
    let mut n = x;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    n
}
";

/// A brand-new, innocent function appended at line 16, beside the untouched
/// `debted`.
const NEW_INNOCENT_FN: &str = "\
fn debted(x: i32) -> i32 {
    let mut n = 0;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    if x > 5 { n += 1; }
    n
}

fn innocent(x: i32) -> i32 {
    x + 1
}

fn also_innocent(x: i32) -> i32 {
    x * 2
}
";

fn measured(source: &str, name: &str) -> MeasuredFn {
    measure_source(source)
        .expect("fixture must parse")
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("fixture has no function named {name}"))
}

fn touched_names(source: &str, diff: &str) -> Vec<String> {
    let functions = measure_source(source).expect("fixture must parse");
    let ranges = parse_touched_ranges(diff);
    touched_functions(&functions, &ranges)
        .into_iter()
        .map(|f| f.name.clone())
        .collect()
}

/// The rule this ticket REPLACES, kept as an executable falsifier.
///
/// Pre-BSE-12 the hook ran `pmat analyze complexity --file "$SRC_FILE"` and
/// refused when ANY function in the file was over threshold — the diff was not
/// consulted at all. `hook_debt_scope_the_old_whole_file_rule_refuses_case_a`
/// asserts this rule still refuses the very commit the new rule allows, so the
/// scoping test cannot pass by accident.
fn whole_file_verdict(new_source: &str, thresholds: DebtThresholds) -> DebtVerdict {
    let offenders: Vec<_> = measure_source(new_source)
        .expect("fixture must parse")
        .into_iter()
        .filter(|f| {
            f.cyclomatic > thresholds.max_cyclomatic || f.cognitive > thresholds.max_cognitive
        })
        .map(|f| super::DebtGrowth {
            function: f.name,
            metric: "Cyclomatic",
            measured: f.cyclomatic,
            limit: thresholds.max_cyclomatic,
            previous: None,
        })
        .collect();
    if offenders.is_empty() {
        DebtVerdict::Allowed
    } else {
        DebtVerdict::Refused(offenders)
    }
}

/// (a) A one-line fix in an undebted function of a file that already carries
/// debt COMMITS.
#[test]
fn hook_debt_scope_one_line_fix_beside_pre_existing_debt_is_allowed() {
    // `git diff --cached -U0` for a single changed line inside `innocent`.
    let diff = "@@ -13 +13 @@\n-    x + 1\n+    x + 2\n";
    assert!(
        measured(ONE_LINE_IN_INNOCENT, "debted").cyclomatic > LIMITS.max_cyclomatic,
        "the fixture must actually carry pre-existing debt"
    );
    assert_eq!(
        touched_names(ONE_LINE_IN_INNOCENT, diff),
        vec!["innocent".to_string()],
        "only the edited function is touched"
    );

    let verdict = diff_scoped_verdict(Some(HEAD_VERSION), ONE_LINE_IN_INNOCENT, diff, LIMITS)
        .expect("fixtures parse");

    assert!(
        verdict.is_allowed(),
        "a one-line change in an undebted function must commit, got {:?}",
        verdict.rendered()
    );
}

/// (b) Debt that GROWS inside a touched function is refused, and the message
/// names the function and both numbers.
#[test]
fn hook_debt_scope_growth_in_a_touched_function_is_refused_with_both_numbers() {
    let diff = "@@ -13 +13,7 @@\n";
    assert_eq!(
        touched_names(GROWTH_IN_INNOCENT, diff),
        vec!["innocent".to_string()],
        "the hunk must land inside `innocent`"
    );

    let verdict = diff_scoped_verdict(Some(HEAD_VERSION), GROWTH_IN_INNOCENT, diff, LIMITS)
        .expect("fixtures parse");

    let lines = verdict.rendered();
    assert!(!verdict.is_allowed(), "growth must be refused");
    let now = measured(GROWTH_IN_INNOCENT, "innocent").cyclomatic;
    let before = measured(HEAD_VERSION, "innocent").cyclomatic;
    let offender = lines
        .iter()
        .find(|l| l.contains("innocent"))
        .unwrap_or_else(|| panic!("no line names the offender: {lines:?}"));
    assert!(
        offender.contains(&now.to_string()) && offender.contains(&before.to_string()),
        "the offender line must carry the measured value ({now}) and the previous \
         value ({before}), got {offender:?}"
    );
    assert!(
        offender.contains(&LIMITS.max_cyclomatic.to_string()),
        "the offender line must carry the limit, got {offender:?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("debted")),
        "the untouched pre-existing offender must not be reported: {lines:?}"
    );
}

/// (c) A NEW function over the threshold is refused on the threshold alone —
/// there is nothing for it to have grown from.
#[test]
fn hook_debt_scope_a_new_over_threshold_function_is_refused() {
    let diff = "@@ -14,0 +16,9 @@\n";
    assert_eq!(
        touched_names(NEW_OVER_THRESHOLD_FN, diff),
        vec!["fresh".to_string()],
        "the hunk must land on the added function"
    );

    let verdict = diff_scoped_verdict(Some(HEAD_VERSION), NEW_OVER_THRESHOLD_FN, diff, LIMITS)
        .expect("fixtures parse");

    let lines = verdict.rendered();
    assert!(
        !verdict.is_allowed(),
        "a new over-threshold function is debt this commit writes"
    );
    assert!(
        lines.iter().any(|l| l.contains("fresh")),
        "the message must name the new function: {lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.contains("no previous measurement")),
        "a function with no old counterpart must not be reported as growth: {lines:?}"
    );
}

/// (d) An untouched over-threshold function beside an innocent addition never
/// affects the verdict.
#[test]
fn hook_debt_scope_an_untouched_over_threshold_function_is_ignored() {
    let diff = "@@ -14,0 +16,4 @@\n";
    assert_eq!(
        touched_names(NEW_INNOCENT_FN, diff),
        vec!["also_innocent".to_string()],
        "the hunk must land on the added function only"
    );

    let verdict = diff_scoped_verdict(Some(HEAD_VERSION), NEW_INNOCENT_FN, diff, LIMITS)
        .expect("fixtures parse");

    assert!(
        verdict.is_allowed(),
        "`debted` is over threshold and untouched; it must not refuse this commit: {:?}",
        verdict.rendered()
    );
}

/// (e) MUTATION: with the scoping removed — the whole-file rule the hook used
/// before BSE-12 — case (a) is REFUSED.
///
/// Observed on the fixtures above:
///   diff-scoped rule : ALLOWED
///   whole-file rule  : REFUSED  ["debted - Cyclomatic 7 > 5 (new function, no previous measurement)"]
///
/// A falsifier that cannot go red is not evidence, so the old rule stays here
/// and is executed.
#[test]
fn hook_debt_scope_the_old_whole_file_rule_refuses_case_a() {
    let diff = "@@ -13 +13 @@\n";
    let scoped = diff_scoped_verdict(Some(HEAD_VERSION), ONE_LINE_IN_INNOCENT, diff, LIMITS)
        .expect("fixtures parse");
    let whole_file = whole_file_verdict(ONE_LINE_IN_INNOCENT, LIMITS);

    assert!(
        scoped.is_allowed(),
        "the shipped rule allows the one-line fix"
    );
    assert!(
        !whole_file.is_allowed(),
        "the pre-BSE-12 rule must refuse it — otherwise this fixture proves nothing"
    );
    assert!(
        whole_file.rendered().iter().any(|l| l.contains("debted")),
        "the old rule refuses because of the untouched function: {:?}",
        whole_file.rendered()
    );
}

/// Hunk headers come in three shapes and all three must be read.
#[test]
fn hook_debt_scope_reads_every_hunk_header_shape() {
    let ranges = parse_touched_ranges(
        "@@ -1,6 +1,7 @@ fn debted\n\
         @@ -30 +31 @@\n\
         @@ -10,3 +9,0 @@\n\
         not a hunk header\n\
         +@@ -1 +1 @@\n",
    );
    let pairs: Vec<(u32, u32)> = ranges.iter().map(|r| (r.start, r.end)).collect();
    assert_eq!(
        pairs,
        vec![(1, 7), (31, 31), (9, 10)],
        "counted hunk, count-less hunk, and a deletion-only hunk (which still \
         modifies the enclosing function)"
    );
}

/// A file with no counterpart at HEAD (a newly added file) is judged against
/// the threshold alone, and its untouched functions are still out of scope.
#[test]
fn hook_debt_scope_a_file_absent_from_head_is_judged_on_the_threshold() {
    let diff = "@@ -0,0 +1,10 @@\n";
    let verdict = diff_scoped_verdict(None, HEAD_VERSION, diff, LIMITS).expect("fixtures parse");
    assert!(!verdict.is_allowed(), "a new file's new debt is refused");
    assert!(
        verdict.rendered().iter().any(|l| l.contains("debted")),
        "the added function must be named: {:?}",
        verdict.rendered()
    );
    assert!(
        !verdict.rendered().iter().any(|l| l.contains("innocent")),
        "lines 1-10 do not reach `innocent`: {:?}",
        verdict.rendered()
    );
}
