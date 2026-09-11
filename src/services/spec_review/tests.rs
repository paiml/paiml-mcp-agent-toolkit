//! goal-mode.md §6.2: one test per finding class, each naming the defect it would
//! let through.
use super::*;

const SPEC: &str = "docs/specifications/goal-mode.md";
/// A plan hash in the form §6.1 names: 64 hex digits (the sha256 of "plan").
const PLAN: &str = "64879f7d6b960a01909762d911a32d4582c20010c5641ee90278b644a9e3b525";

const TEXT: &str = "---\nepic: null\nstatus: active\nvendors: []\n---\n\n# Goal mode\n";

fn artifact(text: &str, roles: &[(&str, &str)], plan_sha: Option<&str>, partial: bool) -> String {
    let lanes: Vec<String> = roles
        .iter()
        .map(|(r, v)| format!(r#"{{"role":"{r}","executor":"agy","verdict":"{v}","summary":"s"}}"#))
        .collect();
    let plan = match plan_sha {
        Some(s) => format!(r#","plan":{{"tool":"claude-plan","ref":"x","sha256":"{s}"}}"#),
        None => String::new(),
    };
    format!(
        r#"{{"spec":"{SPEC}","spec_sha256":"{}"{plan},"lanes":[{}],"agreed":true,"partial":{partial}}}"#,
        sha256_hex(text),
        lanes.join(",")
    )
}

fn all_pass() -> Vec<(&'static str, &'static str)> {
    BASE_ROLES.iter().map(|r| (*r, "PASS")).collect()
}

fn classes(f: &[ReviewFinding]) -> Vec<&'static str> {
    f.iter().map(ReviewFinding::class).collect()
}

/// The guard: a complete, current review holds.
#[test]
fn a_complete_review_passes() {
    let a = artifact(TEXT, &all_pass(), Some(PLAN), false);
    assert!(judge(SPEC, TEXT, &[], Some(&a)).is_empty());
}

#[test]
fn an_active_spec_with_no_review_is_refused() {
    assert_eq!(classes(&judge(SPEC, TEXT, &[], None)), vec!["NO-REVIEW"]);
}

/// The §7 falsifier: append one space to the spec, and the review is stale.
#[test]
fn appending_one_space_to_the_spec_stales_its_review() {
    let a = artifact(TEXT, &all_pass(), Some(PLAN), false);
    let edited = format!("{TEXT} ");
    assert_eq!(
        classes(&judge(SPEC, &edited, &[], Some(&a))),
        vec!["STALE-REVIEW"]
    );
}

#[test]
fn an_artifact_that_does_not_parse_is_refused() {
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some("{not json"))),
        vec!["BAD-REVIEW"]
    );
}

#[test]
fn a_review_of_another_spec_is_refused() {
    let a = artifact(TEXT, &all_pass(), Some(PLAN), false)
        .replace(SPEC, "docs/specifications/other.md");
    assert!(classes(&judge(SPEC, TEXT, &[], Some(&a))).contains(&"SPEC-MISMATCH"));
}

#[test]
fn a_review_without_a_plan_or_with_an_empty_plan_hash_is_refused() {
    for plan in [None, Some("")] {
        let a = artifact(TEXT, &all_pass(), plan, false);
        assert_eq!(
            classes(&judge(SPEC, TEXT, &[], Some(&a))),
            vec!["NO-PLAN"],
            "plan {plan:?}"
        );
    }
}

#[test]
fn every_required_role_must_be_present() {
    let mut roles = all_pass();
    roles.retain(|(r, _)| *r != "crux");
    let a = artifact(TEXT, &roles, Some(PLAN), false);
    assert_eq!(
        judge(SPEC, TEXT, &[], Some(&a)),
        vec![ReviewFinding::MissingRole {
            spec: SPEC.into(),
            role: "crux".into()
        }]
    );
}

/// §4.3: `vendors: [cuda]` adds `vendor:cuda` to the required set.
#[test]
fn a_front_matter_vendor_adds_a_required_role() {
    let vendors = vec!["cuda".to_string()];
    let a = artifact(TEXT, &all_pass(), Some(PLAN), false);
    assert_eq!(
        judge(SPEC, TEXT, &vendors, Some(&a)),
        vec![ReviewFinding::MissingRole {
            spec: SPEC.into(),
            role: "vendor:cuda".into()
        }]
    );
    let mut roles = all_pass();
    roles.push(("vendor:cuda", "PASS"));
    let a2 = artifact(TEXT, &roles, Some(PLAN), false);
    assert!(judge(SPEC, TEXT, &vendors, Some(&a2)).is_empty());
}

#[test]
fn a_lane_that_is_not_pass_is_refused() {
    let mut roles = all_pass();
    roles[1] = ("architecture", "FAIL");
    let a = artifact(TEXT, &roles, Some(PLAN), false);
    assert_eq!(
        judge(SPEC, TEXT, &[], Some(&a)),
        vec![ReviewFinding::LaneNotPass {
            spec: SPEC.into(),
            role: "architecture".into(),
            verdict: "FAIL".into()
        }]
    );
}

#[test]
fn partial_true_is_red() {
    let a = artifact(TEXT, &all_pass(), Some(PLAN), true);
    assert_eq!(classes(&judge(SPEC, TEXT, &[], Some(&a))), vec!["PARTIAL"]);
}

/// §6.1: "six reviewers" must not decay into six copies of one.
#[test]
fn an_unrecognised_role_is_an_error_not_an_extra_lane() {
    let mut roles = all_pass();
    roles.push(("style", "PASS"));
    let a = artifact(TEXT, &roles, Some(PLAN), false);
    assert_eq!(
        judge(SPEC, TEXT, &[], Some(&a)),
        vec![ReviewFinding::UnknownRole {
            spec: SPEC.into(),
            role: "style".into()
        }]
    );
}

#[test]
fn the_slug_flattens_the_path_under_docs_specifications() {
    assert_eq!(slug(SPEC), "goal-mode");
    assert_eq!(
        slug("docs/specifications/components/cli-api.md"),
        "components-cli-api"
    );
    assert_eq!(
        artifact_path(SPEC),
        "docs/audits/spec-goal-mode-review.json"
    );
}

#[test]
fn the_closed_role_set_is_the_five_and_vendor_roles() {
    for r in BASE_ROLES {
        assert!(is_known_role(r), "{r}");
    }
    assert!(is_known_role("vendor:cuda"));
    assert!(!is_known_role("vendor:"));
    assert!(!is_known_role("style"));
    assert_eq!(required_roles(&[]).len(), 5);
    assert_eq!(
        required_roles(&["cuda".to_string()])
            .last()
            .map(String::as_str),
        Some("vendor:cuda")
    );
}

// ── pmat spec review --record (§6.3): validate, then stage; never produce ──

/// Host git configuration is kept out of the fixture.
fn git_in(dir: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=pmat1299",
            "-c",
            "user.email=pmat1299@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A git repository holding one active spec, and a complete review of it
/// written OUTSIDE docs/audits, where a quorum would leave it.
fn recordable() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    git_in(dir.path(), &["init", "-q"]);
    let specs = dir.path().join("docs/specifications");
    std::fs::create_dir_all(&specs).expect("mkdir specs");
    std::fs::write(specs.join("goal-mode.md"), TEXT).expect("write spec");
    let review = dir.path().join("review.json");
    std::fs::write(&review, artifact(TEXT, &all_pass(), Some(PLAN), false)).expect("write review");
    (dir, review)
}

fn staged(dir: &std::path::Path) -> String {
    git_in(dir, &["diff", "--cached", "--name-only"])
}

/// Mutant: record writes somewhere else, re-serialises the JSON, or never
/// stages.
#[test]
fn record_writes_a_valid_review_to_its_artifact_path_and_stages_it() {
    let (dir, review) = recordable();
    let got = record(dir.path(), &review).expect("a valid review records");
    assert_eq!(got.artifact, "docs/audits/spec-goal-mode-review.json");
    assert_eq!(got.spec, SPEC);
    assert_eq!(got.spec_sha256, sha256_hex(TEXT));
    let written = std::fs::read(dir.path().join(&got.artifact)).expect("artifact written");
    assert_eq!(
        written,
        std::fs::read(&review).expect("source"),
        "recorded byte for byte"
    );
    assert_eq!(staged(dir.path()).trim(), got.artifact);
}

/// §6.3 "validates": mutant — record stages without judging.
#[test]
fn record_refuses_a_stale_review_and_writes_nothing() {
    let (dir, review) = recordable();
    std::fs::write(dir.path().join(SPEC), format!("{TEXT} ")).expect("append a space");
    let refusal = record(dir.path(), &review).expect_err("a stale review is refused");
    assert!(
        matches!(&refusal, RecordRefusal::Findings(f) if classes(f) == vec!["STALE-REVIEW"]),
        "{refusal:?}"
    );
    assert!(!dir.path().join("docs/audits").exists(), "nothing written");
    assert_eq!(staged(dir.path()), "", "nothing staged");
}

/// Mutant: the spec path is trusted, so a review can point the hash at any
/// file, or at a file outside the specifications.
#[test]
fn record_refuses_a_review_that_names_a_file_outside_docs_specifications() {
    let (dir, review) = recordable();
    for named in [
        "README.md",
        "docs/specifications/../../README.md",
        "docs/specifications/goal-mode.txt",
        "docs/specifications//goal-mode.md",
        "docs/specifications/./goal-mode.md",
    ] {
        std::fs::write(
            &review,
            artifact(TEXT, &all_pass(), Some(PLAN), false).replace(SPEC, named),
        )
        .expect("write review");
        let refusal = record(dir.path(), &review).expect_err("refused");
        assert_eq!(
            refusal,
            RecordRefusal::NotASpec {
                named: named.to_string()
            }
        );
    }
    assert!(!dir.path().join("docs/audits").exists(), "nothing written");
}

#[test]
fn record_refuses_json_that_does_not_parse() {
    let (dir, review) = recordable();
    std::fs::write(&review, "{not json").expect("write review");
    let refusal = record(dir.path(), &review).expect_err("refused");
    assert!(
        matches!(refusal, RecordRefusal::BadReview { .. }),
        "{refusal:?}"
    );
}

/// Mutant: an unparseable front-matter read as "no vendors", so a review
/// missing its vendor lane records.
#[test]
fn record_refuses_a_spec_whose_front_matter_does_not_parse() {
    let (dir, review) = recordable();
    let text = "# no front-matter\n";
    std::fs::write(dir.path().join(SPEC), text).expect("write spec");
    std::fs::write(&review, artifact(text, &all_pass(), Some(PLAN), false)).expect("write review");
    let refusal = record(dir.path(), &review).expect_err("refused");
    assert!(
        matches!(refusal, RecordRefusal::Unjudgeable { .. }),
        "{refusal:?}"
    );
}

/// Mutant: record judges with an empty vendor list instead of the spec's.
#[test]
fn record_reads_the_vendor_roles_from_the_specs_front_matter() {
    let (dir, review) = recordable();
    let text = TEXT.replace("vendors: []", "vendors: [cuda]");
    std::fs::write(dir.path().join(SPEC), &text).expect("write spec");
    std::fs::write(&review, artifact(&text, &all_pass(), Some(PLAN), false)).expect("write review");
    let refusal = record(dir.path(), &review).expect_err("refused");
    assert_eq!(
        refusal,
        RecordRefusal::Findings(vec![ReviewFinding::MissingRole {
            spec: SPEC.into(),
            role: "vendor:cuda".into()
        }])
    );
}

/// A review already at its artifact path records in place: staged, not
/// rewritten.
#[test]
fn record_in_place_stages_the_artifact() {
    let (dir, review) = recordable();
    let dest = dir.path().join(artifact_path(SPEC));
    std::fs::create_dir_all(dest.parent().expect("parent")).expect("mkdir audits");
    std::fs::rename(&review, &dest).expect("move into place");
    let got = record(dir.path(), &dest).expect("records in place");
    assert_eq!(staged(dir.path()).trim(), got.artifact);
}

/// Mutant: `partial` defaults again (`#[serde(default)]`), so a review whose
/// `partial` key is misspelt reads as complete.
#[test]
fn a_misspelt_partial_key_is_refused_never_defaulted_to_complete() {
    let a = artifact(TEXT, &all_pass(), Some(PLAN), false)
        .replace("\"partial\":false", "\"partail\":true");
    assert!(
        a.contains("partail"),
        "the fixture must carry the misspelling"
    );
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some(&a))),
        vec!["BAD-REVIEW"]
    );
}

// ── the quorum on PMAT-1299: each test names the lane that found the hole ──

/// Five PASS lanes, each with every §6.1 lane field.
fn five() -> serde_json::Value {
    serde_json::Value::Array(
        BASE_ROLES
            .iter()
            .map(|r| serde_json::json!({"role": r, "executor": "agy", "verdict": "PASS", "summary": "s"}))
            .collect(),
    )
}

/// A complete review of TEXT, built by serde so any value is escaped.
fn review_with(plan_sha: &str, lanes: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "spec": SPEC,
        "spec_sha256": sha256_hex(TEXT),
        "plan": {"tool": "claude-plan", "ref": "x", "sha256": plan_sha},
        "lanes": lanes,
        "agreed": true,
        "partial": false
    })
}

/// Remove `a.b.0.c` from a JSON value.
fn remove_at(v: &mut serde_json::Value, path: &str) {
    let mut parts: Vec<&str> = path.split('.').collect();
    let last = parts.pop().expect("a non-empty path");
    let mut cur = v;
    for p in parts {
        cur = match p.parse::<usize>() {
            Ok(i) => &mut cur[i],
            Err(_) => &mut cur[p],
        };
    }
    if let serde_json::Value::Object(m) = cur {
        m.remove(last);
    }
}

/// Conformance lane: §6.1 lists every field, so a review missing any of them
/// does not parse. Mutant: any one of them given a serde default again.
#[test]
fn a_review_missing_a_section_6_1_field_is_bad_review() {
    let full = review_with(PLAN, five());
    assert!(
        judge(SPEC, TEXT, &[], Some(&full.to_string())).is_empty(),
        "the complete fixture passes"
    );
    for path in [
        "spec",
        "spec_sha256",
        "lanes",
        "agreed",
        "partial",
        "plan.tool",
        "plan.ref",
        "lanes.0.role",
        "lanes.0.executor",
        "lanes.0.verdict",
        "lanes.0.summary",
    ] {
        let mut v = full.clone();
        remove_at(&mut v, path);
        assert_eq!(
            classes(&judge(SPEC, TEXT, &[], Some(&v.to_string()))),
            vec!["BAD-REVIEW"],
            "without {path}"
        );
    }
}

/// Adversarial lane: "non-empty" let a plan hash that is not one pass. The
/// field is a sha256: 64 hex digits, either case. Mutant: the format check dropped.
#[test]
fn a_plan_sha256_that_is_not_64_hex_digits_is_no_plan() {
    let long_g = "g".repeat(64);
    let short = "a".repeat(63);
    for sha in [
        "abc123",
        "\u{0}",
        "not-a-hash",
        long_g.as_str(),
        short.as_str(),
    ] {
        let v = review_with(sha, five());
        assert_eq!(
            classes(&judge(SPEC, TEXT, &[], Some(&v.to_string()))),
            vec!["NO-PLAN"],
            "plan sha256 {sha:?}"
        );
    }
    let upper = review_with(&PLAN.to_uppercase(), five());
    assert!(
        judge(SPEC, TEXT, &[], Some(&upper.to_string())).is_empty(),
        "hex digits in either case"
    );
}

/// Adversarial lane: `vendors: [nvidia cuda]` required `vendor:nvidia cuda`,
/// which the closed set then refused as unknown, so the spec could never pass.
/// Mutant: the vendor-name check rejects inner whitespace again.
#[test]
fn a_vendor_named_with_a_space_can_be_reviewed() {
    let vendors = vec!["nvidia cuda".to_string()];
    let mut lanes = all_pass();
    lanes.push(("vendor:nvidia cuda", "PASS"));
    let a = artifact(TEXT, &lanes, Some(PLAN), false);
    assert!(judge(SPEC, TEXT, &vendors, Some(&a)).is_empty());
    assert!(
        !is_known_role("vendor:  "),
        "a blank vendor name is not a role"
    );
}

/// Conformance lane, §6.1: "six reviewers" must not decay into six copies of
/// one. A closed-set role this spec does not require is an extra lane, and a
/// role given twice is a copy. Mutants: either check dropped.
#[test]
fn an_extra_or_a_duplicate_lane_is_refused() {
    let mut extra = all_pass();
    extra.push(("vendor:made-up", "PASS"));
    let a = artifact(TEXT, &extra, Some(PLAN), false);
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some(&a))),
        vec!["EXTRA-LANE"]
    );
    let mut dup = all_pass();
    dup.push(("quality", "PASS"));
    let d = artifact(TEXT, &dup, Some(PLAN), false);
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some(&d))),
        vec!["DUPLICATE-LANE"]
    );
}

/// Test-adequacy lane: the verdict is the exact word PASS. Mutant: a
/// case-insensitive comparison.
#[test]
fn a_lowercase_pass_is_not_pass() {
    let mut lanes = all_pass();
    lanes[0] = ("quality", "pass");
    let a = artifact(TEXT, &lanes, Some(PLAN), false);
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some(&a))),
        vec!["LANE-NOT-PASS"]
    );
}

/// Adversarial lane: `--record` wrote through a symlink planted at the
/// artifact path, to a file outside the project. Mutant: the symlink check dropped.
#[cfg(unix)]
#[test]
fn record_refuses_to_write_through_a_symlinked_artifact_path() {
    let (dir, review) = recordable();
    let outside = tempfile::tempdir().expect("tempdir");
    let victim = outside.path().join("victim.txt");
    std::fs::write(&victim, "ORIGINAL").expect("write victim");
    let dest = dir.path().join(artifact_path(SPEC));
    std::fs::create_dir_all(dest.parent().expect("parent")).expect("mkdir audits");
    std::os::unix::fs::symlink(&victim, &dest).expect("plant the symlink");
    let refusal = record(dir.path(), &review).expect_err("refused");
    assert!(
        matches!(refusal, RecordRefusal::Unwritable { .. }),
        "{refusal:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&victim).expect("victim"),
        "ORIGINAL"
    );
    assert_eq!(staged(dir.path()), "");
}

/// Adversarial lane: a review recorded where git ignores it passes CB-2111
/// locally and never reaches a commit. It is refused before anything is
/// written. Mutant: the ignore check dropped.
#[test]
fn record_refuses_an_artifact_path_git_ignores_and_writes_nothing() {
    let (dir, review) = recordable();
    std::fs::write(dir.path().join(".gitignore"), "docs/audits\n").expect("ignore the audits");
    let refusal = record(dir.path(), &review).expect_err("refused");
    assert!(
        matches!(refusal, RecordRefusal::NotStageable { .. }),
        "{refusal:?}"
    );
    assert!(!dir.path().join("docs/audits").exists(), "nothing written");
}

/// Test-adequacy lane: a failed `git add` must be reported, not swallowed. The
/// index is locked, so the write happens and the stage does not. Mutant:
/// stage's result ignored.
#[test]
fn record_reports_a_failed_stage_and_says_the_file_is_written() {
    let (dir, review) = recordable();
    std::fs::write(dir.path().join(".git/index.lock"), "").expect("lock the index");
    let refusal = record(dir.path(), &review).expect_err("a locked index cannot stage");
    assert!(
        matches!(refusal, RecordRefusal::NotStaged { .. }),
        "{refusal:?}"
    );
    assert!(
        refusal.render().contains("written but NOT staged"),
        "{}",
        refusal.render()
    );
    assert!(
        dir.path().join(artifact_path(SPEC)).exists(),
        "the write happened"
    );
}

// ── the quorum re-run on PMAT-1299 ──

/// Adversarial lane (re-run): a review that records `agreed: false` says its
/// quorum did not agree. With every lane PASS that is a contradiction, and it
/// is red. Mutant: agreed ignored.
#[test]
fn a_review_that_records_agreed_false_is_red() {
    let mut v = review_with(PLAN, five());
    v["agreed"] = serde_json::json!(false);
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some(&v.to_string()))),
        vec!["NOT-AGREED"]
    );
}

/// Test-adequacy lane (re-run): the edges the first tests left open. A
/// duplicated vendor requires its role once; a second lane is DUPLICATE-LANE
/// whatever its verdict; a hyphen is not a hex digit; a duplicated known key
/// does not parse. Mutants: the vendor dedup dropped; the duplicate check
/// moved after the verdict; hyphens accepted; serde's duplicate-field check
/// bypassed.
#[test]
fn the_edges_the_rerun_named_are_pinned() {
    let twice = vec!["cuda".to_string(), "cuda".to_string()];
    let a = artifact(TEXT, &all_pass(), Some(PLAN), false);
    assert_eq!(
        judge(SPEC, TEXT, &twice, Some(&a)),
        vec![ReviewFinding::MissingRole {
            spec: SPEC.into(),
            role: "vendor:cuda".into()
        }]
    );
    let mut dup = all_pass();
    dup.push(("quality", "FAIL"));
    let d = artifact(TEXT, &dup, Some(PLAN), false);
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some(&d))),
        vec!["DUPLICATE-LANE"]
    );
    let hyphen = format!("{}-", &PLAN[..63]);
    assert_eq!(
        classes(&judge(
            SPEC,
            TEXT,
            &[],
            Some(&review_with(&hyphen, five()).to_string())
        )),
        vec!["NO-PLAN"]
    );
    let doubled = review_with(PLAN, five()).to_string().replacen(
        "\"partial\":false",
        "\"partial\":false,\"partial\":false",
        1,
    );
    assert!(
        doubled.matches("\"partial\"").count() == 2,
        "the fixture must carry the key twice"
    );
    assert_eq!(
        classes(&judge(SPEC, TEXT, &[], Some(&doubled))),
        vec!["BAD-REVIEW"]
    );
}

/// Adversarial lane (re-run): a hard link planted at the artifact path shares
/// its inode with a file outside the project, and a direct write would change
/// that file. record writes a temporary file beside the artifact and renames
/// it into place, so the link is replaced, never written through. Mutant: a
/// direct write.
#[test]
fn record_replaces_a_hard_linked_artifact_path_without_writing_through_it() {
    let (dir, review) = recordable();
    let outside = tempfile::tempdir().expect("tempdir");
    let victim = outside.path().join("victim.txt");
    std::fs::write(&victim, "ORIGINAL").expect("write victim");
    let dest = dir.path().join(artifact_path(SPEC));
    std::fs::create_dir_all(dest.parent().expect("parent")).expect("mkdir audits");
    std::fs::hard_link(&victim, &dest).expect("plant the hard link");
    let got = record(dir.path(), &review).expect("a hard link is replaced, not refused");
    assert_eq!(
        std::fs::read_to_string(&victim).expect("victim"),
        "ORIGINAL",
        "the outside file must not change"
    );
    assert_eq!(
        std::fs::read(dir.path().join(&got.artifact)).expect("artifact"),
        std::fs::read(&review).expect("review")
    );
}

/// Adversarial lane (re-run): a spec path holding a control character (here a
/// carriage return) is not a spec. Mutant: control characters accepted.
#[test]
fn record_refuses_a_spec_path_with_a_control_character() {
    let (dir, review) = recordable();
    let named = "docs/specifications/a\rb.md";
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&review).expect("read review"))
            .expect("the review parses");
    v["spec"] = serde_json::json!(named);
    std::fs::write(&review, v.to_string()).expect("write review");
    assert_eq!(
        record(dir.path(), &review).expect_err("refused"),
        RecordRefusal::NotASpec {
            named: named.to_string()
        }
    );
}

/// Test-adequacy lane (re-run): a symlinked docs/audits (an intermediate
/// component) is refused with nothing written through it, and git's own
/// exclude file counts as an ignore, not only a .gitignore. Mutant: the ignore
/// check reduced to "is there a .gitignore". An equivalent mutant noted: the
/// symlink walk reduced to the final component is masked here, because git
/// check-ignore refuses a path beyond a symlink first.
#[cfg(unix)]
#[test]
fn record_refuses_a_symlinked_audits_directory_and_an_excluded_path() {
    let (dir, review) = recordable();
    let outside = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("docs")).expect("mkdir docs");
    std::os::unix::fs::symlink(outside.path(), dir.path().join("docs/audits"))
        .expect("symlink audits");
    let refusal = record(dir.path(), &review).expect_err("refused");
    assert!(
        matches!(
            refusal,
            RecordRefusal::Unwritable { .. } | RecordRefusal::NotStageable { .. }
        ),
        "{refusal:?}"
    );
    assert_eq!(
        std::fs::read_dir(outside.path()).expect("outside").count(),
        0,
        "nothing written through the link"
    );
    std::fs::remove_file(dir.path().join("docs/audits")).expect("unlink audits");
    std::fs::write(dir.path().join(".git/info/exclude"), "docs/audits\n")
        .expect("exclude the audits");
    let refusal = record(dir.path(), &review).expect_err("an excluded path is refused");
    assert!(
        matches!(refusal, RecordRefusal::NotStageable { .. }),
        "{refusal:?}"
    );
}
