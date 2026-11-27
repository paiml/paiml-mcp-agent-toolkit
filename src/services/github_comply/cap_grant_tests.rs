//! The case table of the fleet hook's `gh-comply-cap-grant-test.sh`, against the port.

use super::*;

/// 2026-11-27T07:00:00Z.
const NOW: i64 = 1_795_762_800;
const OK: &str = "2026-11-27T06:50Z";
const BAD: &str = "2026-99-99T99:99Z";

/// The exact line shape the grant watcher writes.
fn line(ts: &str, repo: &str, count: &str, reason: &str) -> String {
    format!("{ts} CAP-GRANT repo={repo} count={count} cop=show-cop reason={reason} | req: 06:59Z show-cop | CAP-ESCALATE {repo} {count} {reason}")
}

fn good() -> String {
    line(OK, "infra", "9/4", "PR: nightly fix")
}

fn on(text: &str, open: usize) -> Grant {
    judge(text, "paiml/infra", open, NOW)
}

fn covers(text: &str) -> bool {
    matches!(on(text, 9), Grant::Covers(_))
}

#[test]
fn fresh_pr_grant_covers_and_prints_the_line_without_its_request() {
    let want = format!("{OK} CAP-GRANT repo=infra count=9/4 cop=show-cop reason=PR: nightly fix");
    assert_eq!(on(&good(), 9), Grant::Covers(want));
}

#[test]
fn accepted_forms() {
    for text in [
        line(OK, "paiml/infra", "9/4", "PR: x"),
        line(OK, "PAIML/Infra", "9/4", "prs: x"),
        line(OK, "infra", "9/4", "PRs x"),
        line("2026-11-27T06:50:30Z", "infra", "12/4", "PR x"),
        line(OK, "infra", "9/4:", "PR x"),
        line("2026-11-27T05:00Z", "infra", "9/4", "PR x"),
        line("2026-11-27T07:02Z", "infra", "9/4", "PR x"),
        format!("{}\n", good()),
        format!(
            "# comment\n\n{OK} NOT-GRANTED rust-mcp-sdk 9/4 external repo | req: x\n{}\n{}\n",
            line("2026-11-26T06:50Z", "infra", "9/4", "PR old"),
            good()
        ),
    ] {
        assert!(covers(&text), "{text}");
    }
}

#[test]
fn the_last_covering_grant_is_named() {
    let text = format!("{}\n{}", good(), line(OK, "infra", "9/4", "PR: later"));
    assert!(matches!(on(&text, 9), Grant::Covers(l) if l.ends_with("PR: later")));
}

#[test]
fn well_formed_lines_that_grant_nothing() {
    let tail = format!("PR x | req: {OK} CAP-GRANT repo=infra count=99/4 cop=x reason=PR y");
    for text in [
        String::new(),
        line(OK, "rmedia", "9/4", "PR x"),
        line(OK, "otherorg/infra", "9/4", "PR x"),
        line(OK, "infra-tools", "9/4", "PR x"),
        line("2026-11-27T04:59Z", "infra", "9/4", "PR x"),
        line("2026-11-26T06:50Z", "infra", "9/4", "PR x"),
        line("2026-11-27T07:03Z", "infra", "9/4", "PR x"),
        line(OK, "infra", "8/4", "PR x"),
        line(OK, "infra", "61/50", "issues, 5/10 PRs: blocked = filing new tickets"),
        line(OK, "infra", "9/4", "preflight x"),
        line(OK, "infra", "9/4", "PR"),
        line(OK, "infra", "9/4", "none"),
        format!("{OK} NOT-GRANTED infra 9/4 external repo | req: CAP-GRANT repo=infra count=9/4 cop=x reason=PR x"),
        line(OK, "rmedia", "9/4", &tail),
    ] {
        assert_eq!(on(&text, 9), Grant::NoGrant, "{text}");
    }
}

#[test]
fn a_bare_name_never_covers_another_owner() {
    assert_eq!(judge(&good(), "otherorg/infra", 9, NOW), Grant::NoGrant);
}

#[test]
fn one_malformed_line_anywhere_voids_the_file() {
    let g = good();
    for bad in [
        "garbage".to_string(),
        " ".to_string(),
        format!("{OK} CAP-GRANT repo=infra count=lots cop=x reason=PR x"),
        format!("{OK} CAP-GRANT repo=infra count=1234567/4 cop=x reason=PR x"),
        line(BAD, "infra", "9/4", "PR x"),
        line(BAD, "rmedia", "9/4", "PR x"),
        line(BAD, "infra", "9/4", "issues x"),
        line(BAD, "infra", "5/4", "PR x"),
        format!("{BAD} NOT-GRANTED infra 9/4 x"),
        line("2026-02-29T06:50Z", "rmedia", "9/4", "PR x"),
        line("2026-11-27T06:50:60Z", "rmedia", "9/4", "PR x"),
        line("2026-11-27T24:00Z", "rmedia", "9/4", "PR x"),
    ] {
        for text in [format!("{g}\n{bad}"), format!("{bad}\n{g}")] {
            assert_eq!(on(&text, 9), Grant::Unusable, "{text}");
        }
    }
}

#[test]
fn only_a_readable_regular_utf8_file_grants() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("grants");
    std::fs::write(&file, good())?;
    assert!(matches!(
        read(&file, "paiml/infra", 9, NOW),
        Grant::Covers(_)
    ));
    for missing in [dir.path().join("nosuch"), dir.path().to_path_buf()] {
        assert_eq!(read(&missing, "paiml/infra", 9, NOW), Grant::Unusable);
    }
    std::fs::write(&file, [good().as_bytes(), &b"\n\xff"[..]].concat())?;
    assert_eq!(read(&file, "paiml/infra", 9, NOW), Grant::Unusable);
    Ok(())
}

#[test]
fn escalation_names_the_inbox_line_the_file_and_why() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("grants");
    let none = escalation(&file, &Grant::NoGrant, "paiml/infra", 9, 4).unwrap_or_default();
    for want in [
        "| CAP-ESCALATE paiml/infra 9/4 PR: <what is blocked>",
        INBOX,
        "no fresh PR grant covers it",
        "Never bypass",
    ] {
        assert!(none.contains(want), "{want}: {none}");
    }
    assert!(none.contains(&file.display().to_string()), "{none}");
    let bad = escalation(&file, &Grant::Unusable, "paiml/infra", 9, 4).unwrap_or_default();
    assert!(bad.contains("unreadable or malformed"), "{bad}");
    Ok(())
}

#[test]
fn a_host_without_the_valve_gets_no_escalation_text() {
    let file = Path::new("/nonexistent-valve-dir/grants");
    assert_eq!(escalation(file, &Grant::Unusable, "o/r", 4, 4), None);
}
