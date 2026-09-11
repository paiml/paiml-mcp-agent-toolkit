//! goal-mode.md §6.2: one test per finding class, each naming the defect it would
//! let through.
use super::*;

const SPEC: &str = "docs/specifications/goal-mode.md";
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
    let a = artifact(TEXT, &all_pass(), Some("abc123"), false);
    assert!(judge(SPEC, TEXT, &[], Some(&a)).is_empty());
}

#[test]
fn an_active_spec_with_no_review_is_refused() {
    assert_eq!(classes(&judge(SPEC, TEXT, &[], None)), vec!["NO-REVIEW"]);
}

/// The §7 falsifier: append one space to the spec, and the review is stale.
#[test]
fn appending_one_space_to_the_spec_stales_its_review() {
    let a = artifact(TEXT, &all_pass(), Some("abc123"), false);
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
    let a = artifact(TEXT, &all_pass(), Some("abc123"), false)
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
    let a = artifact(TEXT, &roles, Some("abc123"), false);
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
    let a = artifact(TEXT, &all_pass(), Some("abc123"), false);
    assert_eq!(
        judge(SPEC, TEXT, &vendors, Some(&a)),
        vec![ReviewFinding::MissingRole {
            spec: SPEC.into(),
            role: "vendor:cuda".into()
        }]
    );
    let mut roles = all_pass();
    roles.push(("vendor:cuda", "PASS"));
    let a2 = artifact(TEXT, &roles, Some("abc123"), false);
    assert!(judge(SPEC, TEXT, &vendors, Some(&a2)).is_empty());
}

#[test]
fn a_lane_that_is_not_pass_is_refused() {
    let mut roles = all_pass();
    roles[1] = ("architecture", "FAIL");
    let a = artifact(TEXT, &roles, Some("abc123"), false);
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
    let a = artifact(TEXT, &all_pass(), Some("abc123"), true);
    assert_eq!(classes(&judge(SPEC, TEXT, &[], Some(&a))), vec!["PARTIAL"]);
}

/// §6.1: "six reviewers" must not decay into six copies of one.
#[test]
fn an_unrecognised_role_is_an_error_not_an_extra_lane() {
    let mut roles = all_pass();
    roles.push(("style", "PASS"));
    let a = artifact(TEXT, &roles, Some("abc123"), false);
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
    std::fs::write(&review, artifact(TEXT, &all_pass(), Some("abc123"), false))
        .expect("write review");
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
    ] {
        std::fs::write(
            &review,
            artifact(TEXT, &all_pass(), Some("abc123"), false).replace(SPEC, named),
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
    std::fs::write(&review, artifact(text, &all_pass(), Some("abc123"), false))
        .expect("write review");
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
    std::fs::write(&review, artifact(&text, &all_pass(), Some("abc123"), false))
        .expect("write review");
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
