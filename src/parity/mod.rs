#![cfg_attr(coverage_nightly, coverage(off))]
//! Parity recording: capture the ORIGINAL program's observable behaviour as cases.
//!
//! `pmat parity record` executes the original once per case, twice, and keeps a
//! case only when both runs agree. A case whose two runs differ (a clock, a PID,
//! a random name) is `flaky` and is excluded from the corpus with a count.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

/// One case to record: the arguments appended to the base command, and stdin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseSpec {
    pub argv: Vec<String>,
    pub stdin: String,
}

/// One observed run of the original, with the files it left under its sandbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    pub argv: Vec<String>,
    pub stdin: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub files: BTreeMap<String, String>,
}

/// Result of recording one case after two runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Recorded(Case),
    Flaky,
}

/// Summary of recording a list of cases.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Summary {
    pub recorded: Vec<Case>,
    pub flaky: usize,
}

static SANDBOX_SEQ: AtomicUsize = AtomicUsize::new(0);

fn fresh_sandbox() -> io::Result<PathBuf> {
    let n = SANDBOX_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("pmat-parity-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn collect_files(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else if let Ok(rel) = path.strip_prefix(root) {
            let body = String::from_utf8_lossy(&std::fs::read(&path)?).into_owned();
            out.insert(rel.to_string_lossy().into_owned(), body);
        }
    }
    Ok(())
}

/// Run the original once in a fresh sandbox. The environment is cleared and only
/// `PATH` plus the `env` allowlist is passed through.
fn run_once(
    bin: &str,
    base: &[String],
    spec: &CaseSpec,
    env: &BTreeMap<String, String>,
) -> io::Result<Case> {
    let sandbox = fresh_sandbox()?;
    let mut cmd = Command::new(bin);
    cmd.args(base)
        .args(&spec.argv)
        .current_dir(&sandbox)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }
    cmd.envs(env);
    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(spec.stdin.as_bytes())?;
    }
    let output = child.wait_with_output()?;
    let mut files = BTreeMap::new();
    collect_files(&sandbox, &sandbox, &mut files)?;
    let _ = std::fs::remove_dir_all(&sandbox);
    Ok(Case {
        argv: spec.argv.clone(),
        stdin: spec.stdin.clone(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code(),
        files,
    })
}

/// Record one case: run twice; equal observations are recorded, unequal are flaky.
pub fn record_case(
    bin: &str,
    base: &[String],
    spec: &CaseSpec,
    env: &BTreeMap<String, String>,
) -> io::Result<Verdict> {
    let first = run_once(bin, base, spec, env)?;
    let second = run_once(bin, base, spec, env)?;
    if first == second {
        Ok(Verdict::Recorded(first))
    } else {
        Ok(Verdict::Flaky)
    }
}

/// Record every case, counting the flaky ones instead of keeping them.
pub fn record_all(
    bin: &str,
    base: &[String],
    specs: &[CaseSpec],
    env: &BTreeMap<String, String>,
) -> io::Result<Summary> {
    let mut summary = Summary::default();
    for spec in specs {
        match record_case(bin, base, spec, env)? {
            Verdict::Recorded(case) => summary.recorded.push(case),
            Verdict::Flaky => summary.flaky += 1,
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn parity_record_flaky_time_printing_fixture_is_not_recorded() {
        let spec = CaseSpec {
            argv: vec![],
            stdin: String::new(),
        };
        let base = vec!["-c".to_string(), "date +%s%N".to_string()];
        let verdict = record_case("/bin/sh", &base, &spec, &BTreeMap::new()).expect("sh must run");
        assert!(
            matches!(verdict, Verdict::Flaky),
            "a fixture that prints the time must be flaky, got {verdict:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn parity_record_stable_case_keeps_stdout_stderr_exit_stdin_and_files() {
        let spec = CaseSpec {
            argv: vec![],
            stdin: "abc".to_string(),
        };
        let base = vec![
            "-c".to_string(),
            "cat; echo err >&2; echo x > out.txt; exit 3".to_string(),
        ];
        let verdict = record_case("/bin/sh", &base, &spec, &BTreeMap::new()).expect("sh must run");
        let Verdict::Recorded(case) = verdict else {
            panic!("a stable fixture must be recorded");
        };
        assert_eq!(case.stdout, "abc");
        assert_eq!(case.stderr, "err\n");
        assert_eq!(case.exit_code, Some(3));
        assert_eq!(case.files.get("out.txt").map(String::as_str), Some("x\n"));
        assert_eq!(case.stdin, "abc");
    }

    #[cfg(unix)]
    #[test]
    fn parity_record_all_counts_flaky_cases_and_keeps_stable_ones() {
        let stable = CaseSpec {
            argv: vec!["hello".to_string()],
            stdin: String::new(),
        };
        let flaky = CaseSpec {
            argv: vec![],
            stdin: String::new(),
        };
        let base = vec![
            "-c".to_string(),
            "if [ \"$1\" = hello ]; then echo hello; else date +%s%N; fi".to_string(),
            "sh".to_string(),
        ];
        let summary =
            record_all("/bin/sh", &base, &[stable, flaky], &BTreeMap::new()).expect("sh must run");
        assert_eq!(summary.recorded.len(), 1);
        assert_eq!(summary.flaky, 1);
        assert_eq!(summary.recorded[0].argv, vec!["hello".to_string()]);
    }
}

/// Outcome of replaying recorded cases against an implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    pub matched: usize,
    pub total: usize,
    pub first_diff: Option<String>,
}

impl ReplayReport {
    /// `matched N/M` as printed by `pmat parity replay`.
    pub fn line(&self) -> String {
        format!("matched {}/{}", self.matched, self.total)
    }
}

/// First differing byte of one stream, as a human-readable line.
fn diff_stream(name: &str, want: &str, got: &str) -> Option<String> {
    if want == got {
        return None;
    }
    let at = want
        .bytes()
        .zip(got.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| want.len().min(got.len()));
    Some(format!(
        "{name} differs at byte {at}: recorded {:?}, got {:?}",
        want.as_bytes().get(at),
        got.as_bytes().get(at)
    ))
}

/// Replay recorded cases against an implementation and compare every stream.
pub fn replay(
    bin: &str,
    base: &[String],
    cases: &[Case],
    env: &BTreeMap<String, String>,
) -> io::Result<ReplayReport> {
    let mut report = ReplayReport {
        matched: 0,
        total: cases.len(),
        first_diff: None,
    };
    for case in cases {
        let spec = CaseSpec {
            argv: case.argv.clone(),
            stdin: case.stdin.clone(),
        };
        let got = run_once(bin, base, &spec, env)?;
        let diffs = [
            diff_stream("stdout", &case.stdout, &got.stdout),
            diff_stream("stderr", &case.stderr, &got.stderr),
            (case.exit_code != got.exit_code).then(|| {
                format!(
                    "exit code differs: recorded {:?}, got {:?}",
                    case.exit_code, got.exit_code
                )
            }),
            (case.files != got.files).then(|| "files written differ".to_string()),
        ];
        match diffs.into_iter().flatten().next() {
            None => report.matched += 1,
            Some(d) => {
                if report.first_diff.is_none() {
                    report.first_diff = Some(format!("argv {:?}: {d}", case.argv));
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod replay_tests {
    use super::*;

    fn recorded(script: &str) -> Vec<Case> {
        let base = vec!["-c".to_string(), script.to_string()];
        let spec = CaseSpec {
            argv: vec![],
            stdin: String::new(),
        };
        match record_case("/bin/sh", &base, &spec, &BTreeMap::new()).expect("sh must run") {
            Verdict::Recorded(c) => vec![c],
            Verdict::Flaky => panic!("fixture must be stable"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn parity_replay_identical_implementation_matches_all() {
        let cases = recorded("echo hi");
        let base = vec!["-c".to_string(), "echo hi".to_string()];
        let report = replay("/bin/sh", &base, &cases, &BTreeMap::new()).expect("sh must run");
        assert_eq!(report.line(), "matched 1/1");
        assert!(report.first_diff.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn parity_replay_one_trailing_newline_fails() {
        let cases = recorded("printf 'x\\n'");
        let base = vec!["-c".to_string(), "printf x".to_string()];
        let report = replay("/bin/sh", &base, &cases, &BTreeMap::new()).expect("sh must run");
        assert_eq!(report.line(), "matched 0/1");
        let diff = report
            .first_diff
            .expect("a one-byte difference must be reported");
        assert!(diff.contains("stdout differs at byte 1"), "{diff}");
    }
}

/// Committed record of what the corpus must still contain.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Manifest {
    pub count: usize,
    pub hashes: std::collections::BTreeSet<String>,
}

/// One reason the gate refuses: the cheat guard, not a behaviour mismatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    LiteralInSource { argv: Vec<String>, len: usize },
    MissingOrEdited { hash: String },
    CountDropped { committed: usize, now: usize },
}

/// Stable FNV-1a over a case's observable fields; unlike `DefaultHasher` it does
/// not change between Rust releases, so a committed manifest keeps meaning.
pub fn case_hash(case: &Case) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |bytes: &[u8]| {
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    for a in &case.argv {
        feed(a.as_bytes());
    }
    feed(case.stdin.as_bytes());
    feed(case.stdout.as_bytes());
    feed(case.stderr.as_bytes());
    feed(
        case.exit_code
            .map_or(String::new(), |c| c.to_string())
            .as_bytes(),
    );
    for (k, v) in &case.files {
        feed(k.as_bytes());
        feed(v.as_bytes());
    }
    format!("{h:016x}")
}

/// Check the corpus against the implementation source and the committed manifest.
pub fn gate(cases: &[Case], source: &str, committed: &Manifest) -> Vec<Violation> {
    const MIN_LITERAL: usize = 24;
    let mut out = Vec::new();
    for case in cases {
        if case.stdout.len() >= MIN_LITERAL && source.contains(case.stdout.as_str()) {
            out.push(Violation::LiteralInSource {
                argv: case.argv.clone(),
                len: case.stdout.len(),
            });
        }
    }
    let now: std::collections::BTreeSet<String> = cases.iter().map(case_hash).collect();
    for hash in &committed.hashes {
        if !now.contains(hash) {
            out.push(Violation::MissingOrEdited { hash: hash.clone() });
        }
    }
    if cases.len() < committed.count {
        out.push(Violation::CountDropped {
            committed: committed.count,
            now: cases.len(),
        });
    }
    out
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    fn case(stdout: &str) -> Case {
        Case {
            argv: vec!["a".to_string()],
            stdin: String::new(),
            stdout: stdout.to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            files: BTreeMap::new(),
        }
    }

    fn manifest_of(cases: &[Case]) -> Manifest {
        Manifest {
            count: cases.len(),
            hashes: cases.iter().map(case_hash).collect(),
        }
    }

    #[test]
    fn parity_gate_clean_corpus_passes() {
        let cases = vec![case("short")];
        let v = gate(&cases, "fn main() {}", &manifest_of(&cases));
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn parity_gate_cheat_literal_in_source_fails() {
        let lit = "this literal is at least twenty-four bytes";
        let cases = vec![case(lit)];
        let source = format!("fn out() -> &'static str {{ \"{lit}\" }}");
        let v = gate(&cases, &source, &manifest_of(&cases));
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::LiteralInSource { .. })),
            "{v:?}"
        );
    }

    #[test]
    fn parity_gate_cheat_edited_case_fails() {
        let committed_cases = vec![case("original output")];
        let manifest = manifest_of(&committed_cases);
        let edited = vec![case("edited output")];
        let v = gate(&edited, "", &manifest);
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::MissingOrEdited { .. })),
            "{v:?}"
        );
    }

    #[test]
    fn parity_gate_cheat_dropped_case_fails() {
        let all = vec![case("one"), case("two")];
        let manifest = manifest_of(&all);
        let dropped = vec![case("one")];
        let v = gate(&dropped, "", &manifest);
        assert!(
            v.iter().any(|x| matches!(
                x,
                Violation::CountDropped {
                    committed: 2,
                    now: 1
                }
            )),
            "{v:?}"
        );
    }
}
