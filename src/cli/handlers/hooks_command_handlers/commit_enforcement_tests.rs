//! AD-03 (docs/specifications/agentic-delivery-pmat.md §4.7 / §9.3, #1126):
//! the generated hooks must be able to REFUSE a commit, not only warn.
//!
//! Before this: the pre-commit SATD and task-ID checks printed a warning and
//! reached `echo "✅ All quality gates passed!"`; no commit-msg hook existed,
//! so "link work to a ticket" was a habit. These tests install the hooks
//! into a throwaway repository and drive `git commit` itself — the only
//! oracle that can tell a warning from a refusal.
use super::hooks_command::HooksCommand;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("git")
}

/// A throwaway git repository with one commit and pmat's hooks installed in
/// strict mode; `.git/hooks` is the hooks dir the installer writes to.
fn strict_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let d = tmp.path();
    assert!(git(d, &["init", "-q"]).status.success());
    assert!(git(d, &["config", "user.email", "t@t"]).status.success());
    assert!(git(d, &["config", "user.name", "t"]).status.success());
    std::fs::write(d.join("README.md"), "one\n").expect("write");
    assert!(git(d, &["add", "."]).status.success());
    assert!(git(d, &["commit", "-q", "-m", "init"]).status.success());
    let cmd = HooksCommand::new(d.join(".git").join("hooks"), d.join("pmat.toml"));
    cmd.install_commit_msg_hook(true)
        .expect("install commit-msg hook");
    tmp
}

/// The refusal: a commit whose message carries no `Pmat-Ticket:` trailer
/// (and no `#NNN`) must not be created. The shipped hook warned and exited 0
/// — a gate that cannot fail (#1126).
#[test]
fn a_commit_without_a_ticket_trailer_is_refused_in_strict_mode() {
    let tmp = strict_repo();
    let d = tmp.path();
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(d, &["commit", "-qam", "no trailer here"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "a commit without Pmat-Ticket must be refused; git said: {stderr}"
    );
    assert!(
        stderr.contains("Pmat-Ticket"),
        "the refusal must name the trailer it wants: {stderr}"
    );
    let log = git(d, &["log", "--oneline"]);
    let head = String::from_utf8_lossy(&log.stdout);
    assert_eq!(head.lines().count(), 1, "the refused commit must not exist");
}

/// The control: the same change with the trailer commits, and the trailer
/// is readable by git itself — the record the comply check (AD-07) reads.
#[test]
fn a_commit_with_the_trailer_is_accepted_and_the_trailer_is_git_readable() {
    let tmp = strict_repo();
    let d = tmp.path();
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(
        d,
        &[
            "commit",
            "-qam",
            "with trailer",
            "-m",
            "Pmat-Ticket: PMAT-655",
        ],
    );
    assert!(
        out.status.success(),
        "a commit with the trailer must be accepted: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let t = git(
        d,
        &[
            "log",
            "-1",
            "--format=%(trailers:key=Pmat-Ticket,valueonly)",
        ],
    );
    assert_eq!(String::from_utf8_lossy(&t.stdout).trim(), "PMAT-655");
}

/// An issue reference is the accepted form for repositories without pmat
/// work: `#1126` in the message satisfies the hook.
#[test]
fn an_issue_reference_satisfies_the_ticket_rule() {
    let tmp = strict_repo();
    let d = tmp.path();
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(d, &["commit", "-qam", "fix the thing (#1126)"]);
    assert!(
        out.status.success(),
        "#NNN must satisfy the rule: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Non-strict keeps the old behaviour: warn, do not refuse — so an
/// unconfigured repository is not locked out by an upgrade.
#[test]
fn without_strict_the_hook_warns_and_lets_the_commit_through() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let d = tmp.path();
    assert!(git(d, &["init", "-q"]).status.success());
    assert!(git(d, &["config", "user.email", "t@t"]).status.success());
    assert!(git(d, &["config", "user.name", "t"]).status.success());
    std::fs::write(d.join("README.md"), "one\n").expect("write");
    assert!(git(d, &["add", "."]).status.success());
    assert!(git(d, &["commit", "-q", "-m", "init"]).status.success());
    HooksCommand::new(d.join(".git").join("hooks"), d.join("pmat.toml"))
        .install_commit_msg_hook(false)
        .expect("install");
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let out = git(d, &["commit", "-qam", "no trailer"]);
    assert!(out.status.success(), "non-strict must not refuse");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Pmat-Ticket"),
        "non-strict must still warn, naming the trailer"
    );
}

/// The pre-commit SATD block blocks under strict: the generated text must
/// carry an `exit 1` on the over-threshold branch when strict is on, and the
/// success banner must not be reachable from it.
#[test]
fn the_pre_commit_satd_block_exits_1_under_strict() {
    let cmd = HooksCommand::new(
        std::path::PathBuf::from("/tmp"),
        std::path::PathBuf::from("/tmp"),
    );
    let hook = cmd.generate_quality_checks();
    assert!(
        hook.contains("PMAT_HOOKS_STRICT"),
        "the SATD block must consult the strict switch"
    );
    assert!(
        hook.contains("SATD comments exceed the threshold and [hooks] strict is on"),
        "the strict branch must refuse with a reason, not a warning glyph"
    );
}

/// Found by the AD-04 quorum review of this branch (three lanes, all FAIL): the
/// `--strict` flag reached only the commit-msg hook — the pre-commit generator
/// read `[hooks] strict` from the configuration and ignored the flag, so
/// `pmat hooks install --strict` in a repository without `pmat.toml` produced a
/// pre-commit hook with `PMAT_HOOKS_STRICT=0`. Both hooks must agree.
#[test]
fn the_strict_flag_reaches_the_pre_commit_hook_without_a_config() {
    let tmp = strict_repo();
    let d = tmp.path();
    assert!(
        !d.join("pmat.toml").exists(),
        "the control: no configuration file"
    );
    let cmd = HooksCommand::new(d.join(".git").join("hooks"), d.join("pmat.toml"));
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let result = rt
        .block_on(cmd.install(true, false, false, true))
        .expect("install --strict");
    assert!(result.success, "{}", result.message);
    let pre_commit = std::fs::read_to_string(d.join(".git/hooks/pre-commit")).expect("hook");
    assert!(
        pre_commit.contains("export PMAT_HOOKS_STRICT=1"),
        "--strict must reach the pre-commit hook, not only commit-msg:\n{pre_commit}"
    );
    assert!(
        cmd.installed_strict(),
        "installed_strict() must read the flag back"
    );
}

/// Second finding of the same review: every automatic rewrite (`hooks verify --fix`,
/// `hooks update`, the comply auto-install) re-generated the hook with `strict=false`,
/// silently disarming a `--strict` install at the next refresh. The rewrite must
/// keep the strictness of the hook it replaces.
#[test]
fn hooks_verify_fix_keeps_a_strict_install_strict() {
    let tmp = strict_repo();
    let d = tmp.path();
    let cmd = HooksCommand::new(d.join(".git").join("hooks"), d.join("pmat.toml"));
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(cmd.install(true, false, false, true))
        .expect("install --strict");
    let hook_path = d.join(".git/hooks/pre-commit");
    // Drift the installed hook so `verify --fix` takes its rewrite path.
    let mut drifted = std::fs::read_to_string(&hook_path).expect("hook");
    drifted.push_str("\n# drift\n");
    std::fs::write(&hook_path, drifted).expect("write");
    let verified = rt.block_on(cmd.verify(true)).expect("verify --fix");
    assert!(
        verified.fixes_applied.iter().any(|f| f.contains("Updated")),
        "the control: verify --fix must have rewritten the hook: {:?}",
        verified
    );
    let rewritten = std::fs::read_to_string(&hook_path).expect("hook");
    assert!(
        rewritten.contains("export PMAT_HOOKS_STRICT=1"),
        "an automatic rewrite dropped --strict:\n{rewritten}"
    );
}

/// Third finding of the AD-04 quorum on the PR head: `--strict --stack` silently
/// dropped strict — the stack installer writes its own lightweight pre-commit hook
/// and no commit-msg hook. A flag that changes nothing must refuse, not pass.
#[test]
fn strict_with_stack_is_refused_rather_than_silently_dropped() {
    use crate::cli::commands::HooksCommands;
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let err = rt
        .block_on(super::command_dispatch::handle_hooks_command(
            &HooksCommands::Install {
                interactive: false,
                force: false,
                backup: false,
                tdg_enforcement: false,
                strict: true,
                stack: true,
                update: false,
            },
        ))
        .expect_err("--strict --stack must be refused");
    let msg = err.to_string();
    assert!(msg.contains("--stack") && msg.contains("--strict"), "{msg}");
    assert!(
        msg.contains("PMAT-659"),
        "the refusal names the ticket that closes the gap: {msg}"
    );
}

// ── PMAT-727: one predicate, two callers ─────────────────────────────────────
//
// The commit-msg hook and CB-2113 must give the same verdict on the same
// message. #1250 went red in CI on eleven commits the hook had accepted: their
// `Pmat-Ticket:` sat in the paragraph ABOVE the Co-Authored-By block, git reads
// trailers from the last paragraph only, so CB-2113 saw none — while the hook's
// fallback matched `PMAT-NNN` in the subject. And #1252 went red on a trailer
// naming the ticket its own commits marked completed, which the hook never
// looked at.

/// A repository whose `master` carries a roadmap with `items` (id, status) and
/// pmat's strict commit-msg hook; the working branch is `feature`.
fn roadmap_repo(items: &[(&str, &str)]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let d = tmp.path();
    assert!(git(d, &["init", "-q", "-b", "master"]).status.success());
    assert!(git(d, &["config", "user.email", "t@t"]).status.success());
    assert!(git(d, &["config", "user.name", "t"]).status.success());
    let mut yaml = String::from("roadmap_version: '1.0'\ngithub_enabled: false\nroadmap:\n");
    for (id, status) in items {
        yaml.push_str(&format!(
            "- id: {id}\n  github_issue: null\n  item_type: task\n  title: {id} work\n  status: {status}\n"
        ));
    }
    std::fs::create_dir_all(d.join("docs/roadmaps")).expect("mkdir docs/roadmaps");
    std::fs::write(d.join("docs/roadmaps/roadmap.yaml"), yaml).expect("write roadmap");
    std::fs::write(d.join("README.md"), "one\n").expect("write");
    assert!(git(d, &["add", "."]).status.success());
    assert!(git(d, &["commit", "-q", "-m", "init"]).status.success());
    assert!(git(d, &["switch", "-q", "-c", "feature"]).status.success());
    HooksCommand::new(d.join(".git").join("hooks"), d.join("pmat.toml"))
        .install_commit_msg_hook(true)
        .expect("install commit-msg hook");
    tmp
}

/// The hook's verdict on `msg`: run `.git/hooks/commit-msg` on a message file
/// from the repository root, exactly as git does.
fn hook_refuses(d: &Path, msg: &str) -> bool {
    let f = d.join(".git").join("PMAT727_HOOK_MSG");
    std::fs::write(&f, msg).expect("write message");
    let out = Command::new("bash")
        .current_dir(d)
        .arg(d.join(".git").join("hooks").join("commit-msg"))
        .arg(&f)
        .output()
        .expect("run the commit-msg hook");
    !out.status.success()
}

/// CB-2113's verdict on `msg`: commit it on `feature` with the hook bypassed,
/// then measure `master..HEAD` the way the rule does on a pull request.
fn rule_refuses(d: &Path, msg: &str) -> bool {
    std::fs::write(d.join("README.md"), "one\ntwo\n").expect("write");
    let f = d.join(".git").join("PMAT727_RULE_MSG");
    std::fs::write(&f, msg).expect("write message");
    let path = f.to_str().expect("utf-8 temp path");
    assert!(git(d, &["commit", "-q", "--no-verify", "-a", "-F", path])
        .status
        .success());
    let m = crate::services::commit_traceability::measure_against(d, Some("master"))
        .expect("CB-2113 measures the fixture");
    !m.findings.is_empty()
}

/// One agreement case: its name, the commit message, and the roadmap items
/// (id, status) the fixture repository carries.
type AgreementCase<'a> = (&'a str, String, &'a [(&'a str, &'a str)]);

#[test]
fn the_hook_and_cb2113_give_the_same_verdict_on_the_same_message() {
    let co = "Co-Authored-By: A <a@example.com>";
    let open: &[(&str, &str)] = &[("PMAT-001", "planned")];
    let done: &[(&str, &str)] = &[("PMAT-001", "completed")];
    let cases: Vec<AgreementCase> = vec![
        (
            "the trailer in the last paragraph names an open item",
            format!("feat: a\n\nPmat-Ticket: PMAT-001\n{co}\n"),
            open,
        ),
        (
            "the trailer sits in the paragraph above Co-Authored-By (#1250)",
            format!("feat(PMAT-001): a\n\nPmat-Ticket: PMAT-001\n\n{co}\n"),
            open,
        ),
        (
            "no trailer; the id appears only in the subject",
            format!("feat(PMAT-001): a\n\n{co}\n"),
            open,
        ),
        (
            "the trailer names a completed item (#1252)",
            format!("feat: a\n\nPmat-Ticket: PMAT-001\n{co}\n"),
            done,
        ),
        (
            "the trailer names an id the roadmap does not have",
            format!("feat: a\n\nPmat-Ticket: PMAT-999\n{co}\n"),
            open,
        ),
    ];
    for (name, msg, items) in &cases {
        let hook = hook_refuses(roadmap_repo(items).path(), msg);
        let rule = rule_refuses(roadmap_repo(items).path(), msg);
        assert_eq!(
            hook,
            rule,
            "{name}: the hook {} while CB-2113 {} — one predicate, two callers (PMAT-727)",
            if hook { "refuses" } else { "accepts" },
            if rule { "refuses" } else { "accepts" }
        );
    }
}

/// Without a roadmap CB-2113 does not judge (Skip), so the hook keeps the
/// configured ticket pattern as the repository's own convention. This is the
/// one place the two callers differ, and it is written down here.
#[test]
fn without_a_roadmap_the_hook_keeps_the_ticket_pattern_and_cb2113_skips() {
    let tmp = strict_repo();
    let d = tmp.path();
    assert!(matches!(
        crate::services::commit_traceability::inputs(d),
        crate::services::commit_traceability::Inputs::NoRoadmap
    ));
    assert!(
        !hook_refuses(d, "fix: PMAT-5 names the ticket\n"),
        "without a roadmap a ticket reference matching the pattern is accepted"
    );
    assert!(
        hook_refuses(d, "fix: names nothing\n"),
        "strict still refuses a message that names no ticket"
    );
}
