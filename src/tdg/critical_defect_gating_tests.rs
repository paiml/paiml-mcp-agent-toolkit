//! #919: `critical_defects_count` and `has_critical_defects` must never contradict.
//!
//! The #279 exemption (a file with no git history must not be auto-failed by a
//! gate it cannot pass until committed) used to be expressed by clearing
//! `has_critical_defects` while leaving the count set. That produced a record
//! asserting "1 critical defect" and "no critical defects" at once, made the
//! same bytes score 0.0/F inside a repo and 99.5/A+ outside one — because the
//! git query is also false when there is no repository at all — and was written
//! into `.pmat/baseline.json`, so a baseline captured before `git add` recorded
//! the clean answer permanently.

use super::grade::Grade;
use super::score::TdgScore;

fn with_defects(count: usize) -> TdgScore {
    TdgScore {
        critical_defects_count: count,
        has_critical_defects: count > 0,
        ..Default::default()
    }
}

/// The invariant the defect broke.
#[test]
fn the_existence_flag_always_agrees_with_the_count() {
    for (count, suppressed) in [(0, None), (1, None), (3, Some("untracked".to_string()))] {
        let mut s = with_defects(count);
        s.critical_defects_suppressed = suppressed;
        s.calculate_total();
        assert_eq!(
            s.has_critical_defects,
            s.critical_defects_count > 0,
            "count {} and flag {} disagree",
            s.critical_defects_count,
            s.has_critical_defects
        );
    }
}

/// The penalty is graduated, monotone, and capped below B-, so an agent fixing
/// defects sees the number move. It used to be a flat 0.0 at every count.
#[test]
fn unsuppressed_critical_defects_degrade_the_score_monotonically() {
    let mut previous = f32::MAX;
    for count in 1..=6 {
        let mut s = with_defects(count);
        s.calculate_total();

        assert!(s.has_critical_defects);
        assert!(s.critical_defects_suppressed.is_none());
        assert!(
            s.total < 70.0,
            "count {count}: a critical defect must cap the score below B-, got {}",
            s.total
        );
        assert!(
            s.total < previous,
            "count {count}: more defects must score strictly worse than fewer \
             ({} is not below {previous})",
            s.total
        );
        previous = s.total;
    }
}

/// ...and one defect in an otherwise perfect file must still leave the other
/// measurements legible, which a flat zero does not.
#[test]
fn a_single_defect_does_not_erase_every_other_signal() {
    let mut s = with_defects(1);
    s.calculate_total();
    assert!(s.total > 0.0, "got {}", s.total);
    assert_eq!(s.grade, Grade::CPlus);
}

/// #279's intent survives: the gate does not fire for an untracked file.
#[test]
fn suppressed_critical_defects_do_not_zero_the_score() {
    let mut s = with_defects(1);
    s.critical_defects_suppressed = Some("file is not tracked by git".to_string());
    s.calculate_total();

    assert!(
        s.total > 0.0,
        "an exempted file must keep its quality score"
    );
    assert_ne!(s.grade, Grade::F);
    // ...and it still admits the defects exist, which is the whole point.
    assert!(s.has_critical_defects);
    assert_eq!(s.critical_defects_count, 1);
}

#[test]
fn a_clean_file_is_never_marked_suppressed() {
    let mut s = with_defects(0);
    s.calculate_total();
    assert!(!s.has_critical_defects);
    assert!(s.critical_defects_suppressed.is_none());
    assert_ne!(s.grade, Grade::F);
}

/// The suppression must survive a round-trip through a persisted baseline, and
/// an OLD baseline written before this field existed must still deserialize.
#[test]
fn suppression_round_trips_through_json_and_old_baselines_still_load() {
    let mut s = with_defects(2);
    s.critical_defects_suppressed = Some("file is not tracked by git".to_string());
    s.calculate_total();

    let json = serde_json::to_string(&s).expect("serialize");
    assert!(json.contains("critical_defects_suppressed"), "{json}");
    let back: TdgScore = serde_json::from_str(&json).expect("round trip");
    assert_eq!(
        back.critical_defects_suppressed,
        s.critical_defects_suppressed
    );
    assert_eq!(back.has_critical_defects, s.has_critical_defects);

    // A clean score omits the field entirely (skip_serializing_if), and a
    // baseline from before 3.30.1 has no such key at all.
    let clean = serde_json::to_string(&with_defects(0)).expect("serialize");
    assert!(!clean.contains("critical_defects_suppressed"), "{clean}");
    let legacy: TdgScore = serde_json::from_str(&clean).expect("legacy baseline must load");
    assert!(legacy.critical_defects_suppressed.is_none());
}

/// The tri-state that replaced the old two-valued predicate. #279 exempts a file
/// that is *about to* gain history; it says nothing about code that is not under
/// version control at all, where no commit can be blocked. Collapsing those two
/// into one `false` is what waived the gate for everything outside a repo.
mod git_tracking {
    // `analyzer_impl1_source_dispatch.rs` is `include!`d into `analyzer_ast`,
    // so its items live directly on that module rather than a submodule.
    use crate::tdg::analyzer_ast::{git_tracking_status, is_exempt_as_new_file, GitTracking};
    use std::process::Command;

    fn git(dir: &std::path::Path, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .output()
            .expect("git must be available for this test")
            .status
            .success();
        assert!(ok, "git {args:?} failed");
    }

    #[test]
    fn code_outside_any_repository_is_not_exempt() {
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("lib.rs");
        std::fs::write(&f, "pub fn f() {}\n").expect("write");

        assert_eq!(git_tracking_status(&f), GitTracking::NotVersioned);
        assert!(
            !is_exempt_as_new_file(&f),
            "no repository means no commit to be blocked, so nothing to exempt"
        );
    }

    #[test]
    fn an_uncommitted_file_inside_a_repository_is_exempt() {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "-q"]);
        let f = dir.path().join("lib.rs");
        std::fs::write(&f, "pub fn f() {}\n").expect("write");

        assert_eq!(git_tracking_status(&f), GitTracking::UntrackedInRepo);
        assert!(is_exempt_as_new_file(&f), "this is exactly the #279 case");
    }

    #[test]
    fn a_committed_file_is_never_exempt() {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "-q"]);
        let f = dir.path().join("lib.rs");
        std::fs::write(&f, "pub fn f() {}\n").expect("write");
        git(dir.path(), &["add", "-A"]);
        git(dir.path(), &["commit", "-qm", "init", "--no-verify"]);

        assert_eq!(git_tracking_status(&f), GitTracking::Tracked);
        assert!(!is_exempt_as_new_file(&f));
    }

    /// The bug in one assertion: identical bytes must not be exempt in one
    /// place and gated in another purely because of where they sit.
    #[test]
    fn committed_and_unversioned_copies_of_one_file_agree() {
        let repo = tempfile::tempdir().expect("tempdir");
        let plain = tempfile::tempdir().expect("tempdir");
        git(repo.path(), &["init", "-q"]);
        let src = "pub fn f(v: Vec<i32>) -> i32 { *v.first().unwrap() }\n";
        let a = repo.path().join("lib.rs");
        let b = plain.path().join("lib.rs");
        std::fs::write(&a, src).expect("write");
        std::fs::write(&b, src).expect("write");
        git(repo.path(), &["add", "-A"]);
        git(repo.path(), &["commit", "-qm", "init", "--no-verify"]);

        assert_eq!(
            is_exempt_as_new_file(&a),
            is_exempt_as_new_file(&b),
            "same bytes, same gating decision"
        );
    }
}

/// End-to-end pins, run through `analyze_source` — the path that actually
/// produced the contradiction. The struct-level invariant above cannot catch a
/// regression here, because the old code cleared the flag *after* the score was
/// built and before it was serialized.
mod through_the_analyzer {
    use crate::tdg::analyzer_ast::TdgAnalyzerAst;
    use crate::tdg::grade::Grade;
    use crate::tdg::language_simple::Language;
    use std::process::Command;

    /// Detected as a critical defect by `RustDefectDetector`.
    const WITH_CRITICAL_DEFECT: &str = "pub fn f(v: Vec<i32>) -> i32 { *v.first().unwrap() }\n";

    fn analyze_at_path(file: &std::path::Path) -> crate::tdg::score::TdgScore {
        TdgAnalyzerAst::new()
            .expect("analyzer")
            .analyze_source(
                WITH_CRITICAL_DEFECT,
                Language::Rust,
                Some(file.to_path_buf()),
            )
            .expect("analyze")
    }

    fn analyze_at(dir: &std::path::Path) -> crate::tdg::score::TdgScore {
        let file = dir.join("lib.rs");
        std::fs::write(&file, WITH_CRITICAL_DEFECT).expect("write");
        TdgAnalyzerAst::new()
            .expect("analyzer")
            .analyze_source(WITH_CRITICAL_DEFECT, Language::Rust, Some(file))
            .expect("analyze")
    }

    fn init_repo(dir: &std::path::Path) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["init", "-q"])
            .output()
            .expect("git must be available")
            .status
            .success();
        assert!(ok, "git init failed");
    }

    /// The exact record that was reported: count 1, flag false.
    #[test]
    fn a_score_never_reports_defects_and_no_defects_at_once() {
        for in_repo in [false, true] {
            let dir = tempfile::tempdir().expect("tempdir");
            if in_repo {
                init_repo(dir.path());
            }
            let score = analyze_at(dir.path());

            assert!(
                score.critical_defects_count > 0,
                "fixture must have a defect"
            );
            assert!(
                score.has_critical_defects,
                "in_repo={in_repo}: count is {} but has_critical_defects is false",
                score.critical_defects_count
            );
        }
    }

    /// Code outside version control is treated exactly like committed code.
    #[test]
    fn defects_outside_a_repository_are_not_waived() {
        let plain = tempfile::tempdir().expect("tempdir");
        let repo = tempfile::tempdir().expect("tempdir");
        init_repo(repo.path());
        let committed = {
            let f = repo.path().join("lib.rs");
            std::fs::write(&f, WITH_CRITICAL_DEFECT).expect("write");
            let ok = Command::new("git")
                .arg("-C")
                .arg(repo.path())
                .args(["add", "-A"])
                .output()
                .expect("git")
                .status
                .success();
            assert!(ok);
            let ok = Command::new("git")
                .arg("-C")
                .arg(repo.path())
                .args(["-c", "user.email=t@t", "-c", "user.name=t"])
                .args(["commit", "-qm", "init", "--no-verify"])
                .output()
                .expect("git")
                .status
                .success();
            assert!(ok);
            analyze_at_path(&f)
        };
        let unversioned = analyze_at(plain.path());

        assert!(unversioned.critical_defects_suppressed.is_none());
        // The point of the tri-state: same bytes, same verdict, same number.
        assert_eq!(unversioned.total, committed.total);
        assert_eq!(unversioned.grade, committed.grade);
    }

    /// #279 still holds for the case it was written for, and now says so.
    #[test]
    fn defects_in_an_uncommitted_file_are_waived_with_a_stated_reason() {
        let dir = tempfile::tempdir().expect("tempdir");
        init_repo(dir.path());
        let score = analyze_at(dir.path());

        let reason = score
            .critical_defects_suppressed
            .as_deref()
            .expect("the waiver must record why");
        assert!(
            reason.contains("#279"),
            "reason should cite the rule: {reason}"
        );
        assert_ne!(
            score.grade,
            Grade::F,
            "an uncommitted file is not auto-failed"
        );
        assert!(
            score.has_critical_defects,
            "...but the defects are still reported"
        );
    }
}

/// The score must not depend on git status — the invariant #919 was filed for,
/// and which the #279 waiver briefly broke again from the other direction.
///
/// The waiver skipped the PENALTY rather than the gate, so an uncommitted file
/// with five `.unwrap()` calls scored 100.0/A+ while the byte-identical
/// committed file scored 9.1/F, and `check-quality --min-grade A` exited 0 on
/// the first and 3 on the second.
mod the_waiver_touches_the_gate_not_the_score {
    use super::*;

    fn scored(count: usize, suppressed: Option<&str>) -> TdgScore {
        let mut s = TdgScore {
            critical_defects_count: count,
            has_critical_defects: count > 0,
            critical_defects_suppressed: suppressed.map(str::to_string),
            ..Default::default()
        };
        s.calculate_total();
        s
    }

    #[test]
    fn a_waived_file_carries_the_same_score_as_an_unwaived_one() {
        for count in 1..=5 {
            let waived = scored(count, Some("no commits yet (#279)"));
            let plain = scored(count, None);

            assert!(
                (waived.total - plain.total).abs() < f32::EPSILON,
                "count {count}: waiving the gate must not change the score \
                 ({} waived vs {} unwaived)",
                waived.total,
                plain.total
            );
            assert_eq!(waived.grade, plain.grade, "count {count}");
        }
    }

    /// ...and specifically, a waiver never restores full marks.
    #[test]
    fn a_waived_file_with_defects_is_never_a_plus() {
        let waived = scored(5, Some("no commits yet (#279)"));
        assert!(
            waived.total < 70.0,
            "five critical defects cannot score {} merely because the file is \
             uncommitted",
            waived.total
        );
        assert_ne!(waived.grade, Grade::APlus);
    }
}
