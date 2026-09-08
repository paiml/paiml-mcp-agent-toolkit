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

/// The adapter that the CLI entry point calls: everything above measures the
/// RULE against in-memory fixtures, and these measure the two READS the rule
/// needs — the staged blob and its HEAD counterpart — against a real repo.
///
/// A fixture repo, not a mock: `git show :<path>` (the INDEX, not the working
/// tree) is precisely the read a mock would have gotten wrong, and it is the
/// one that decides whether the hook judges what is being committed or what
/// happens to be on disk.
mod staged_repo {
    use super::super::{staged_verdict, staged_verdict_for_file, DebtThresholds};
    use super::{whole_file_verdict, GROWTH_IN_INNOCENT, HEAD_VERSION, ONE_LINE_IN_INNOCENT};
    use std::path::Path;

    const LIMITS: DebtThresholds = DebtThresholds {
        max_cyclomatic: 5,
        max_cognitive: 100,
    };

    fn git(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .current_dir(dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(args)
            .output()
            .expect("git must be on PATH");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// `--template=` keeps a developer's global hook template out of the
    /// fixture; the identity is pinned so the commit does not depend on the
    /// machine's git config.
    fn repo_with_committed_debt() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path();
        git(p, &["init", "-q", "--template=", "--initial-branch=main"]);
        git(p, &["config", "user.email", "fixture@example.com"]);
        git(p, &["config", "user.name", "Fixture"]);
        std::fs::create_dir_all(p.join("src")).expect("mkdir");
        std::fs::write(p.join("src/lib.rs"), HEAD_VERSION).expect("write");
        git(p, &["add", "-A"]);
        git(p, &["commit", "-q", "-m", "pre-existing debt"]);
        dir
    }

    fn stage(dir: &Path, source: &str) {
        std::fs::write(dir.join("src/lib.rs"), source).expect("write");
        git(dir, &["add", "src/lib.rs"]);
    }

    /// Report O11, end to end over git: a one-line fix in an undebted function
    /// of a file carrying pre-existing debt is ALLOWED.
    #[test]
    fn staged_verdict_allows_a_one_line_fix_beside_pre_existing_debt() {
        let repo = repo_with_committed_debt();

        let verdict =
            staged_verdict(repo.path(), Path::new("src/lib.rs"), LIMITS).expect("verdict");

        assert!(
            verdict.is_allowed(),
            "the O11 case must commit, got {:?}",
            verdict.rendered()
        );
        // The falsifier: the rule the hook used before this ticket refuses the
        // very same staged content, so the pass above is scoping and not an
        // absence of debt.
        assert!(
            !whole_file_verdict(ONE_LINE_IN_INNOCENT, LIMITS).is_allowed(),
            "the pre-BSE-12 whole-file rule must refuse this same file"
        );
    }

    /// Growth inside a touched function is refused, and the offender line names
    /// the function and both numbers.
    #[test]
    fn staged_verdict_refuses_growth_and_names_the_function() {
        let repo = repo_with_committed_debt();
        stage(repo.path(), GROWTH_IN_INNOCENT);

        let verdict =
            staged_verdict(repo.path(), Path::new("src/lib.rs"), LIMITS).expect("verdict");

        assert!(!verdict.is_allowed(), "growth must be refused");
        let rendered = verdict.rendered().join("\n");
        assert!(
            rendered.contains("innocent"),
            "the refusal names the function: {rendered}"
        );
        assert!(
            rendered.contains("6 > 5"),
            "the refusal carries measured and limit: {rendered}"
        );
        assert!(
            rendered.contains("(was 1)"),
            "the refusal carries the previous value: {rendered}"
        );
        assert!(
            !rendered.contains("debted"),
            "the untouched debted function is never an offender: {rendered}"
        );
    }

    /// The verdict is a function of the INDEX, not of the working tree: a
    /// further unstaged edit that would blow the limit cannot change it.
    #[test]
    fn staged_verdict_reads_the_index_not_the_working_tree() {
        let repo = repo_with_committed_debt();
        std::fs::write(repo.path().join("src/lib.rs"), GROWTH_IN_INNOCENT).expect("write");

        let verdict =
            staged_verdict(repo.path(), Path::new("src/lib.rs"), LIMITS).expect("verdict");

        assert!(
            verdict.is_allowed(),
            "only staged content is judged, got {:?}",
            verdict.rendered()
        );
    }

    /// A file that exists only in the index (added, never committed) has no
    /// HEAD counterpart: it is judged against the threshold alone rather than
    /// erroring out.
    #[test]
    fn staged_verdict_judges_a_file_absent_from_head() {
        let repo = repo_with_committed_debt();
        std::fs::write(repo.path().join("src/added.rs"), HEAD_VERSION).expect("write");
        git(repo.path(), &["add", "src/added.rs"]);

        let verdict =
            staged_verdict(repo.path(), Path::new("src/added.rs"), LIMITS).expect("verdict");

        assert!(!verdict.is_allowed(), "new debt in a new file is refused");
        assert!(
            verdict.rendered().iter().any(|l| l.contains("debted")),
            "the new file's offender is named: {:?}",
            verdict.rendered()
        );
    }

    /// The entry point the CLI flag uses takes the path the user typed and
    /// finds the repo itself.
    #[test]
    fn staged_verdict_for_file_resolves_the_repo_from_the_path() {
        let repo = repo_with_committed_debt();

        let verdict =
            staged_verdict_for_file(&repo.path().join("src/lib.rs"), LIMITS).expect("verdict");

        assert!(
            verdict.is_allowed(),
            "same O11 verdict through the path-taking entry point: {:?}",
            verdict.rendered()
        );
    }

    /// A path git does not have staged is NOT "no violations": say so.
    #[test]
    fn staged_verdict_refuses_to_grade_an_unstaged_path() {
        let repo = repo_with_committed_debt();
        std::fs::write(repo.path().join("src/loose.rs"), HEAD_VERSION).expect("write");

        let err = staged_verdict(repo.path(), Path::new("src/loose.rs"), LIMITS)
            .expect_err("an unstaged path has nothing to judge");

        assert!(
            err.to_string().contains("src/loose.rs"),
            "the error names the path: {err}"
        );
    }
    #[test]
    fn staged_verdict_allows_one_line_fix_after_git_mv() {
        let repo = repo_with_committed_debt();
        git(repo.path(), &["mv", "src/lib.rs", "src/renamed.rs"]);
        // Wait, stage() function writes to src/lib.rs hardcoded!
        // So I'll do it manually
        std::fs::write(repo.path().join("src/renamed.rs"), ONE_LINE_IN_INNOCENT).expect("write");
        git(repo.path(), &["add", "src/renamed.rs"]);

        let verdict =
            staged_verdict(repo.path(), Path::new("src/renamed.rs"), LIMITS).expect("verdict");
        assert!(
            verdict.is_allowed(),
            "file was renamed and edited, should keep its old baseline and be allowed: {:?}",
            verdict
        );
    }

    #[test]
    fn staged_verdict_mutant_no_rename_resolution_refuses() {
        let repo = repo_with_committed_debt();
        git(repo.path(), &["mv", "src/lib.rs", "src/renamed.rs"]);
        std::fs::write(repo.path().join("src/renamed.rs"), ONE_LINE_IN_INNOCENT).expect("write");
        git(repo.path(), &["add", "src/renamed.rs"]);

        let verdict =
            staged_verdict(repo.path(), Path::new("src/renamed.rs"), LIMITS).expect("verdict");
        assert!(verdict.is_allowed(), "real code allows");

        // Mutant logic (old behaviour without -M name status)
        let spec = "src/renamed.rs";
        let staged = format!(":{spec}");
        let new_source = super::super::git_read(repo.path(), &["show", &staged])
            .expect("git show of the staged blob must succeed")
            .expect("the staged blob must exist");
        let old_source = super::super::git_read(repo.path(), &["show", &format!("HEAD:{spec}")])
            .expect("git show of HEAD:<path> must succeed even when the path is absent"); // None: src/renamed.rs does not exist in HEAD
        let diff = super::super::git_read(repo.path(), &["diff", "--cached", "-U0", "--", spec])
            .expect("git diff --cached must succeed")
            .expect("a staged rename must produce a diff");

        let mutant_verdict =
            super::super::diff_scoped_verdict(old_source.as_deref(), &new_source, &diff, LIMITS)
                .expect("the mutant path must still return a verdict, not an error");
        assert!(
            !mutant_verdict.is_allowed(),
            "mutant without rename resolution loses baseline and refuses pre-existing debt"
        );
    }
}
const TWO_IMPLS_HEAD: &str = "\
struct A;
impl A {
    fn new() -> Self { 
        let mut x = 1;
        if x > 0 { x += 1; }
        if x > 1 { x += 1; }
        if x > 2 { x += 1; }
        if x > 3 { x += 1; }
        if x > 4 { x += 1; }
        if x > 5 { x += 1; }
        A
    }
}
struct B;
impl B {
    fn new() -> Self { B }
}
";

const TWO_IMPLS_SECOND_GROWS: &str = "\
struct A;
impl A {
    fn new() -> Self { 
        let mut x = 1;
        if x > 0 { x += 1; }
        if x > 1 { x += 1; }
        if x > 2 { x += 1; }
        if x > 3 { x += 1; }
        if x > 4 { x += 1; }
        if x > 5 { x += 1; }
        A
    }
}
struct B;
impl B {
    fn new() -> Self { 
        let mut x = 1;
        if x > 0 { x += 1; }
        if x > 1 { x += 1; }
        if x > 2 { x += 1; }
        if x > 3 { x += 1; }
        if x > 4 { x += 1; }
        B
    }
}
";

const TWO_IMPLS_SECOND_DEBTED_HEAD: &str = "\
struct A;
impl A {
    fn new() -> Self { A }
}
struct B;
impl B {
    fn new() -> Self { 
        let mut x = 1;
        if x > 0 { x += 1; }
        if x > 1 { x += 1; }
        if x > 2 { x += 1; }
        if x > 3 { x += 1; }
        if x > 4 { x += 1; }
        if x > 5 { x += 1; }
        B
    }
}
";

const TWO_IMPLS_SECOND_DEBTED_TOUCHED: &str = "\
struct A;
impl A {
    fn new() -> Self { A }
}
struct B;
impl B {
    fn new() -> Self { 
        let mut x = 2; // touched
        if x > 0 { x += 1; }
        if x > 1 { x += 1; }
        if x > 2 { x += 1; }
        if x > 3 { x += 1; }
        if x > 4 { x += 1; }
        if x > 5 { x += 1; }
        B
    }
}
";

const THREE_IMPLS_INSERTED_BEFORE: &str = "\
struct AA;
impl AA {
    fn new() -> Self { AA } // Inserted
}
struct A;
impl A {
    fn new() -> Self { A }
}
struct B;
impl B {
    fn new() -> Self { 
        let mut x = 2; // touched
        if x > 0 { x += 1; }
        if x > 1 { x += 1; }
        if x > 2 { x += 1; }
        if x > 3 { x += 1; }
        if x > 4 { x += 1; }
        if x > 5 { x += 1; }
        B
    }
}
";

#[test]
fn hook_debt_scope_pairing_two_impls_growth_in_second_refused() {
    let diff = "@@ -17 +17,7 @@\n";
    let verdict = diff_scoped_verdict(Some(TWO_IMPLS_HEAD), TWO_IMPLS_SECOND_GROWS, diff, LIMITS)
        .expect("parse");
    assert!(!verdict.is_allowed(), "growth must be refused");
    let rendered = verdict.rendered().join("\n");
    assert!(rendered.contains("new"), "must name 'new'");
    assert!(
        rendered.contains("6 > 5"),
        "must contain measured and limit"
    );
    assert!(rendered.contains("(was 1)"), "must contain previous");
}

#[test]
fn hook_debt_scope_pairing_two_impls_touched_second_debted_allowed() {
    let diff = "@@ -8 +8 @@\n-        let mut x = 1;\n+        let mut x = 2; // touched\n";
    let verdict = diff_scoped_verdict(
        Some(TWO_IMPLS_SECOND_DEBTED_HEAD),
        TWO_IMPLS_SECOND_DEBTED_TOUCHED,
        diff,
        LIMITS,
    )
    .expect("parse");
    assert!(verdict.is_allowed(), "touched without growing -> allowed");
}

#[test]
fn hook_debt_scope_pairing_inserted_before_differs_min_baseline_refused() {
    let diff = "@@ -0,0 +1,4 @@\n@@ -8 +12 @@\n-        let mut x = 1;\n+        let mut x = 2; // touched\n";
    let verdict = diff_scoped_verdict(
        Some(TWO_IMPLS_SECOND_DEBTED_HEAD),
        THREE_IMPLS_INSERTED_BEFORE,
        diff,
        LIMITS,
    )
    .expect("parse");
    assert!(
        !verdict.is_allowed(),
        "count differs, min baseline used, so debted is refused"
    );
    let rendered = verdict.rendered().join("\n");
    assert!(
        rendered.contains("new - Cyclomatic 7 > 5 (was 1)"),
        "must use the min baseline (1) from the first 'new'"
    );
}

#[test]
fn hook_debt_scope_pairing_mutant_bare_name_find() {
    let new_functions =
        measure_source(TWO_IMPLS_SECOND_GROWS).expect("the staged fixture must parse");
    let old_functions = measure_source(TWO_IMPLS_HEAD).expect("the HEAD fixture must parse");
    let ranges = parse_touched_ranges("@@ -17 +17,7 @@\n");

    let mut growths = Vec::new();
    for func in touched_functions(&new_functions, &ranges) {
        let previous = old_functions.iter().find(|old| old.name == func.name);
        growths.extend(super::growth_for(func, previous, LIMITS));
    }

    let mutant_verdict = if growths.is_empty() {
        DebtVerdict::Allowed
    } else {
        DebtVerdict::Refused(growths)
    };

    assert!(mutant_verdict.is_allowed(), "the mutant allows the growth because it matches against the first 'new' (cyc=7), hiding the growth of the second 'new' (cyc=6)");
}
#[test]
fn hook_debt_scope_rename_to_deleted_name_inherits_baseline_pin() {
    // PIN: This asserts the CURRENT (wrong) behaviour where a touched function renamed to the name of a
    // DELETED function inherits the deleted one's baseline. This is an accepted limitation of name-keyed pairing.

    let old_source = "\
fn deleted_debted(x: i32) -> i32 {
    let mut n = x;
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    if x > 5 { n += 1; }
    if x > 6 { n += 1; }
    if x > 7 { n += 1; }
    if x > 8 { n += 1; }
    n
}
fn will_rename(x: i32) -> i32 {
    x + 1
}
";

    let new_source = "\
fn deleted_debted(x: i32) -> i32 {
    let mut n = x + 1; // touched
    if x > 0 { n += 1; }
    if x > 1 { n += 1; }
    if x > 2 { n += 1; }
    if x > 3 { n += 1; }
    if x > 4 { n += 1; }
    if x > 5 { n += 1; }
    n
}
";
    let diff = "@@ -1,14 +1,9 @@\n";
    let verdict = diff_scoped_verdict(Some(old_source), new_source, diff, LIMITS).expect("parse");
    // Under correct identity tracking (it's actually `will_rename`), it grew from cyc=1 to cyc=7, limit=5. Should be REFUSED.
    // However, it pairs with `deleted_debted` (cyc=10), sees 7 <= 10, and ALLOWS.
    assert!(
        verdict.is_allowed(),
        "PIN: wrongly inherits deleted baseline and passes"
    );
}
