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
