use super::*;

const H: &str = "headsha1";
fn lane(name: &str, verdict: &str, model: &str, ts: &str) -> Lane {
    Lane {
        lane: name.into(),
        verdict: verdict.into(),
        model: model.into(),
        ts: ts.into(),
        by: "lane-1".into(),
        head: H.into(),
        control_caught: Some(true),
    }
}
fn good() -> Receipt {
    Receipt {
        patch_id: "p1".into(),
        head: H.into(),
        pr: 7,
        repo: "paiml/x".into(),
        lanes: vec![
            lane("q1", "PASS", "claude-opus-5-5", "2026-10-24T10:00:00Z"),
            lane("q2", "PASS", "gemini-3.1-pro", "2026-10-24T10:00:01Z"),
            lane("q3", "PASS", "gemini-3.1-pro", "2026-10-24T10:00:02Z"),
            lane(
                "seat:rust",
                "PASS",
                "gemini-3.8-flash",
                "2026-10-24T10:00:03Z",
            ),
        ],
    }
}
fn check(r: &Receipt) -> Result<(), Refusal> {
    verify_receipt(
        r,
        H,
        &["lane-1".into()],
        &["rust".into()],
        &["claude-haiku-5".into()],
    )
}

#[test]
fn good_receipt_passes() {
    assert_eq!(check(&good()), Ok(()));
}
#[test]
fn refuses_fewer_than_two_models() {
    let mut r = good();
    for l in &mut r.lanes {
        l.model = "gemini-3.1-pro".into();
    }
    assert_eq!(check(&r), Err(Refusal::FewerThanTwoModels(1)));
}
#[test]
fn refuses_seat_not_from_lane_identity() {
    let mut r = good();
    r.lanes[3].by = "coding-worker-5404".into();
    assert!(matches!(
        check(&r),
        Err(Refusal::SeatNotFromLaneIdentity(_))
    ));
}
#[test]
fn refuses_red_then_green_on_same_head() {
    let mut r = good();
    r.lanes.push(lane(
        "q1",
        "FAIL",
        "claude-opus-5-5",
        "2026-10-24T09:00:00Z",
    ));
    assert!(matches!(check(&r), Err(Refusal::RerunAsPass(_))));
}
#[test]
fn red_then_green_with_a_diff_is_allowed() {
    let mut r = good();
    let mut red = lane("q1", "FAIL", "claude-opus-5-5", "2026-10-24T09:00:00Z");
    red.head = "older".into();
    r.lanes.push(red);
    assert_eq!(check(&r), Ok(()));
}
#[test]
fn refuses_missed_planted_control() {
    let mut r = good();
    r.lanes[1].control_caught = Some(false);
    assert!(matches!(check(&r), Err(Refusal::MissedControl(_))));
}
#[test]
fn refuses_lane_model_equal_to_author() {
    let mut r = good();
    r.lanes[0].model = "Claude Haiku 5".into(); // normalises to the author id
    assert!(matches!(check(&r), Err(Refusal::AuthorModelInLane(_))));
}
#[test]
fn co_authored_by_trailer_names_an_author() {
    let m = co_authored_models(&["x\n\nCo-Authored-By: Claude Opus 5.5 <n@a.com>".into()]);
    assert_eq!(m, vec!["claude-opus-5-5"]);
    let r = good();
    let e = verify_receipt(&r, H, &["lane-1".into()], &["rust".into()], &m);
    assert!(matches!(e, Err(Refusal::AuthorModelInLane(_))));
}
#[test]
fn refuses_stale_head() {
    let e = verify_receipt(
        &good(),
        "newhead",
        &["lane-1".into()],
        &["rust".into()],
        &[],
    );
    assert!(matches!(e, Err(Refusal::StaleHead { .. })));
}
#[test]
fn refuses_missing_seat_lane() {
    let mut r = good();
    r.lanes.pop();
    assert!(matches!(check(&r), Err(Refusal::MissingLane(_))));
}

fn setup(r: &Receipt) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = receipt_dir(d.path(), "paiml/x", 7);
    std::fs::create_dir_all(&p).unwrap();
    std::fs::write(p.join("p1.json"), serde_json::to_vec(r).unwrap()).unwrap();
    d
}
fn go(d: &Path, head: &'static str, rearm: bool) -> Result<SignFile, SignError> {
    let lookup = move |_: &str, _: u64| Ok(head.to_string());
    sign(&Inputs {
        repo: "paiml/x",
        pr: 7,
        root: d,
        patch_ids: &["p1".to_string()],
        head_lookup: &lookup,
        lane_roster: &["lane-1".to_string()],
        required_seats: &["rust".to_string()],
        author_models: &["claude-haiku-5".to_string()],
        now: "2026-10-24T11:00:00Z",
        rearm,
    })
}

#[test]
fn sign_writes_sign_json_with_receipt_hash() {
    let d = setup(&good());
    let sf = go(d.path(), H, false).unwrap();
    let on_disk: SignFile = serde_json::from_slice(
        &std::fs::read(receipt_dir(d.path(), "paiml/x", 7).join("sign.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(on_disk, sf);
    assert_eq!(sf.receipt_sha256["p1"].len(), 64);
}
#[test]
fn sign_refuses_when_pr_head_moved_and_writes_nothing() {
    let d = setup(&good());
    assert!(matches!(
        go(d.path(), "moved", false),
        Err(SignError::Refused(Refusal::StaleHead { .. }))
    ));
    assert!(!receipt_dir(d.path(), "paiml/x", 7)
        .join("sign.json")
        .exists());
}
#[test]
fn head_lookup_failure_is_not_measured() {
    let d = setup(&good());
    let lookup = |_: &str, _: u64| Err("rate limited".to_string());
    let e = sign(&Inputs {
        repo: "paiml/x",
        pr: 7,
        root: d.path(),
        patch_ids: &["p1".to_string()],
        head_lookup: &lookup,
        lane_roster: &[],
        required_seats: &[],
        author_models: &[],
        now: "t",
        rearm: false,
    });
    assert!(matches!(e, Err(SignError::NotMeasured(_))));
}
#[test]
fn second_sign_needs_rearm_and_keeps_the_old_file() {
    let d = setup(&good());
    go(d.path(), H, false).unwrap();
    assert_eq!(
        go(d.path(), H, false),
        Err(SignError::Refused(Refusal::AlreadySigned))
    );
    go(d.path(), H, true).unwrap();
    let dir = receipt_dir(d.path(), "paiml/x", 7);
    let kept = std::fs::read_dir(dir).unwrap().flatten().any(|e| {
        e.file_name()
            .to_string_lossy()
            .starts_with("sign.json.disarmed-")
    });
    assert!(kept);
}
#[test]
fn missing_receipt_is_refused() {
    let d = setup(&good());
    std::fs::remove_file(receipt_dir(d.path(), "paiml/x", 7).join("p1.json")).unwrap();
    assert!(matches!(
        go(d.path(), H, false),
        Err(SignError::Refused(Refusal::ReceiptMissing(_)))
    ));
}

fn rulings(text: &str) -> (tempfile::TempDir, PathBuf) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("noah-rulings.md");
    std::fs::write(&p, text).unwrap();
    (d, p)
}
#[test]
fn rearm_needs_alfredo_line_naming_the_pr() {
    let (_d, p) = rulings("ALFREDO 2026-10-24T11:00Z — rearm PR #7 after head move\n");
    assert!(rearm_authorized(&p, 7));
    assert!(!rearm_authorized(&p, 70), "#70 must not match #7");
    assert!(!rearm_authorized(&p, 8));
}
#[test]
fn rearm_rejects_missing_prefix_and_irreversible_verbs() {
    let (_d, p) = rulings("note 2026-10-24 — rearm #7\n");
    assert!(!rearm_authorized(&p, 7));
    let (_d, p) = rulings("ALFREDO 2026-10-24T11:00Z — rearm and merge #7\n");
    assert!(!rearm_authorized(&p, 7));
    assert!(!rearm_authorized(Path::new("/nonexistent/rulings.md"), 7));
}
#[test]
fn git_version_gate_for_verbatim() {
    assert!(!git_at_least("git version 2.34.1", (2, 39)));
    assert!(git_at_least("git version 2.39.0", (2, 39)));
    assert!(git_at_least("git version 2.45.2", (2, 39)));
}
#[test]
fn old_git_is_not_measured_never_a_different_key() {
    // Only meaningful on a git < 2.39 box (this host: 2.34.1); skipped elsewhere.
    let v = run(Path::new("."), "git", &["--version"]).unwrap_or_default();
    if !git_at_least(&v, (2, 39)) {
        assert!(matches!(
            patch_id_u0(Path::new("."), "HEAD"),
            Err(SignError::NotMeasured(_))
        ));
    }
}
#[test]
fn default_patch_id_command_is_u0_diff_tree() {
    assert!(PATCH_ID_CMD.starts_with("git diff-tree -p -U0 "));
    assert!(PATCH_ID_CMD.contains("--diff-algorithm=myers"));
    assert!(PATCH_ID_CMD.ends_with("git patch-id --verbatim"));
}
