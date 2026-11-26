//! #1528 EMR-001 — tests for `scripts/emr1-verify.sh`, the read-only EMR-1 verifier.
//!
//! EMR-1 (EMR-001 rev1 section 7): every landed ledger row's `-U0` patch-id must
//! equal its `key`, and its receipt must carry that key with `Verdict: PASS`.
//!
//! Each test builds a throwaway git repository and ledger in a temp dir, so the
//! verifier is exercised against real `git diff | git patch-id` output.
//! `cargo test --lib -- emr1_verify`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const HEADER: &str =
    "pr\tticket\tclass\tkey\treview\tprescreen\tint_sha\tcheckpoint\tdisposition\treason\tts";

fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/emr1-verify.sh")
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Build a repo with one base commit and one change commit. Returns the change
/// commit sha and the `-U0` patch-id of parent -> change, computed the contract's way.
fn make_repo(dir: &Path) -> TestResult<(String, String)> {
    git(dir, &["init", "-q"]);
    fs::write(dir.join("a.txt"), "one\ntwo\nthree\n")?;
    git(dir, &["add", "a.txt"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    fs::write(dir.join("a.txt"), "one\nTWO\nthree\nfour\n")?;
    git(dir, &["add", "a.txt"]);
    git(dir, &["commit", "-q", "-m", "EMR: 001 change"]);
    let sha = git(dir, &["rev-parse", "HEAD"]);
    let diff = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "diff",
            "-U0",
            "--no-color",
            "--no-ext-diff",
            "--no-renames",
            "--diff-algorithm=myers",
            "HEAD^",
            "HEAD",
        ])
        .output()
        .expect("git diff runs");
    let mut patch_id = Command::new("git")
        .args(["patch-id", "--verbatim"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("git patch-id runs");
    use std::io::Write;
    // Wait for the child before returning any error, so it is never left a zombie.
    let written = patch_id
        .stdin
        .take()
        .map(|mut stdin| stdin.write_all(&diff.stdout));
    let out = patch_id.wait_with_output()?;
    written.ok_or("git patch-id stdin is not piped")??;
    let key = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .ok_or("git patch-id printed no id")?
        .to_string();
    Ok((sha, key))
}

fn write_ledger(dir: &Path, rows: &[String]) -> TestResult<PathBuf> {
    let path = dir.join("ledger.tsv");
    let mut text = format!("{HEADER}\n");
    for r in rows {
        text.push_str(r);
        text.push('\n');
    }
    fs::write(&path, text)?;
    Ok(path)
}

fn row(ticket: &str, key: &str, review: &str, sha: &str, disposition: &str) -> String {
    format!(
        "1\t{ticket}\tA\t{key}\t{review}\tgreen\t{sha}\tcp1\t{disposition}\t-\t2026-10-24T16:00Z"
    )
}

fn write_receipt(dir: &Path, name: &str, key: &str, verdict: &str) -> TestResult<String> {
    fs::write(
        dir.join(name),
        format!("EMR-Key: {key}\nVerdict: {verdict}\n"),
    )?;
    Ok(name.to_string())
}

fn run(
    ledger: &Path,
    repo: &Path,
    receipts: &Path,
    path_prefix: Option<&Path>,
) -> TestResult<Output> {
    let mut cmd = Command::new("sh");
    cmd.arg(script()).arg(ledger).arg(repo).arg(receipts);
    if let Some(prefix) = path_prefix {
        let old = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![prefix.to_path_buf()];
        paths.extend(std::env::split_paths(&old));
        cmd.env("PATH", std::env::join_paths(paths)?);
    }
    Ok(cmd.output()?)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn tmp(name: &str) -> TestResult<tempfile::TempDir> {
    Ok(tempfile::Builder::new().prefix(name).tempdir()?)
}

#[test]
fn emr1_verify_passes_when_key_and_receipt_match() -> TestResult {
    let t = tmp("emr1-pass")?;
    let repo = t.path().join("repo");
    fs::create_dir(&repo)?;
    let (sha, key) = make_repo(&repo)?;
    let receipts = t.path().join("receipts");
    fs::create_dir(&receipts)?;
    let review = write_receipt(&receipts, "r1.txt", &key, "PASS")?;
    let ledger = write_ledger(t.path(), &[row("PMAT-1", &key, &review, &sha, "landed")])?;

    let out = run(&ledger, &repo, &receipts, None)?;
    assert!(out.status.success(), "stdout: {}", stdout(&out));
    assert!(
        stdout(&out).contains("PASS PMAT-1"),
        "stdout: {}",
        stdout(&out)
    );
    Ok(())
}

#[test]
fn emr1_verify_fails_on_key_mismatch() -> TestResult {
    let t = tmp("emr1-key")?;
    let repo = t.path().join("repo");
    fs::create_dir(&repo)?;
    let (sha, key) = make_repo(&repo)?;
    let receipts = t.path().join("receipts");
    fs::create_dir(&receipts)?;
    let bad = "0000000000000000000000000000000000000000";
    assert_ne!(bad, key);
    let review = write_receipt(&receipts, "r1.txt", bad, "PASS")?;
    let ledger = write_ledger(t.path(), &[row("PMAT-1", bad, &review, &sha, "landed")])?;

    let out = run(&ledger, &repo, &receipts, None)?;
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stdout(&out).contains("FAIL PMAT-1 key-mismatch"),
        "stdout: {}",
        stdout(&out)
    );
    Ok(())
}

#[test]
fn emr1_verify_fails_without_passing_receipt() -> TestResult {
    let t = tmp("emr1-receipt")?;
    let repo = t.path().join("repo");
    fs::create_dir(&repo)?;
    let (sha, key) = make_repo(&repo)?;
    let receipts = t.path().join("receipts");
    fs::create_dir(&receipts)?;
    // Receipt exists and carries the key, but its verdict is FAIL.
    let failing = write_receipt(&receipts, "r-fail.txt", &key, "FAIL")?;
    let ledger = write_ledger(t.path(), &[row("PMAT-1", &key, &failing, &sha, "landed")])?;
    let out = run(&ledger, &repo, &receipts, None)?;
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stdout(&out).contains("FAIL PMAT-1 receipt-not-passing"),
        "stdout: {}",
        stdout(&out)
    );

    // Receipt path that does not exist.
    let ledger = write_ledger(
        t.path(),
        &[row("PMAT-1", &key, "missing.txt", &sha, "landed")],
    )?;
    let out = run(&ledger, &repo, &receipts, None)?;
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stdout(&out).contains("FAIL PMAT-1 receipt-missing"),
        "stdout: {}",
        stdout(&out)
    );
    Ok(())
}

#[test]
fn emr1_verify_ignores_rows_that_did_not_land() -> TestResult {
    let t = tmp("emr1-skip")?;
    let repo = t.path().join("repo");
    fs::create_dir(&repo)?;
    let (sha, _key) = make_repo(&repo)?;
    let receipts = t.path().join("receipts");
    fs::create_dir(&receipts)?;
    let ledger = write_ledger(
        t.path(),
        &[row("PMAT-2", "bogus", "none", &sha, "superseded")],
    )?;
    let out = run(&ledger, &repo, &receipts, None)?;
    assert!(out.status.success(), "stdout: {}", stdout(&out));
    assert!(
        stdout(&out).contains("0 landed"),
        "stdout: {}",
        stdout(&out)
    );
    Ok(())
}

/// Read-only proof: every git call the verifier makes is a read verb, it makes no
/// GitHub call, and the repo, ledger and receipts are byte-identical afterwards.
#[test]
fn emr1_verify_is_read_only() -> TestResult {
    let t = tmp("emr1-readonly")?;
    let repo = t.path().join("repo");
    fs::create_dir(&repo)?;
    let (sha, key) = make_repo(&repo)?;
    let receipts = t.path().join("receipts");
    fs::create_dir(&receipts)?;
    let review = write_receipt(&receipts, "r1.txt", &key, "PASS")?;
    let ledger = write_ledger(t.path(), &[row("PMAT-1", &key, &review, &sha, "landed")])?;

    // Shims for git and gh on PATH. The git shim logs its arguments, then runs the real git.
    let shims = t.path().join("shims");
    fs::create_dir(&shims)?;
    let real_git = find_on_path("git");
    let git_log = t.path().join("git.log");
    let gh_log = t.path().join("gh.log");
    write_shim(
        &shims.join("git"),
        &format!(
            "printf '%s\\n' \"$*\" >> '{}'\nexec '{}' \"$@\"\n",
            git_log.display(),
            real_git.display()
        ),
    )?;
    write_shim(
        &shims.join("gh"),
        &format!("printf '%s\\n' \"$*\" >> '{}'\nexit 99\n", gh_log.display()),
    )?;

    let before = snapshot(t.path())?;
    let out = run(&ledger, &repo, &receipts, Some(&shims))?;
    let after = snapshot(t.path())?;

    assert!(out.status.success(), "stdout: {}", stdout(&out));
    assert_eq!(before, after, "verifier changed files on disk");

    let log = fs::read_to_string(&git_log).expect("verifier ran git through the shim");
    assert!(!log.trim().is_empty());
    for line in log.lines() {
        let verb = git_verb(line);
        assert!(
            matches!(verb.as_str(), "rev-parse" | "diff" | "patch-id"),
            "verifier ran a non-read git verb: {line}"
        );
    }
    assert!(!gh_log.exists(), "verifier called gh");
    Ok(())
}

fn git_verb(line: &str) -> String {
    let mut words = line.split_whitespace();
    while let Some(w) = words.next() {
        if w == "-C" {
            words.next();
            continue;
        }
        return w.to_string();
    }
    String::new()
}

fn find_on_path(bin: &str) -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|d| d.join(bin))
        .find(|p| p.is_file())
        .expect("binary on PATH")
}

fn write_shim(path: &Path, body: &str) -> TestResult {
    fs::write(path, format!("#!/bin/sh\n{body}"))?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

/// Every file under `root` (path and bytes), sorted. Covers `.git`, so an index or
/// ref write would show up as a difference.
fn snapshot(root: &Path) -> TestResult<Vec<(PathBuf, Vec<u8>)>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let p = entry?.path();
            if p.is_dir() {
                stack.push(p);
            } else if p
                .file_name()
                .is_some_and(|n| n != "git.log" && n != "gh.log")
            {
                out.push((p.clone(), fs::read(&p)?));
            }
        }
    }
    out.sort();
    Ok(out)
}
