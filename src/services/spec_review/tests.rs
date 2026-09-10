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
