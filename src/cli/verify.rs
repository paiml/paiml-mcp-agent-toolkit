#![cfg_attr(coverage_nightly, coverage(off))]
//! `pmat verify` — autonomous pre-flight verification.
//!
//! Runs the CI-faithful gate set (format, complexity, satd, clippy, tests)
//! fail-fast, with machine-readable output, so an autonomous agent gets
//! "green here ⇒ green in CI" before committing. The canonical agent loop is:
//! `edit → pmat verify --changed --format json → self-fix on red → commit on green`.
//!
//! Spec: `docs/specifications/pmat-verify-autonomous-preflight.md`.

use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Output format for `pmat verify`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum VerifyFormat {
    /// Human-readable summary.
    Text,
    /// Structured JSON for autonomous agents.
    Json,
}

/// `pmat verify` arguments.
#[derive(Debug, clap::Args)]
pub struct VerifyArgs {
    /// Output format (`json` is for autonomous agents).
    #[arg(long, value_enum, default_value = "text")]
    pub format: VerifyFormat,

    /// Auto-apply fixable issues (`cargo fmt`, `cargo clippy --fix`).
    #[arg(long)]
    pub fix: bool,

    /// Run every stage even after a failure (full report instead of fail-fast).
    #[arg(long)]
    pub no_fail_fast: bool,

    /// Stages to skip, comma-separated: format,complexity,satd,clippy,tests.
    #[arg(long, value_delimiter = ',')]
    pub skip: Vec<String>,

    /// Run only this single stage.
    #[arg(long)]
    pub stage: Option<String>,
}

/// CI-faithful stages, cheapest first (fail-fast catches common cases in seconds).
const STAGES: &[&str] = &["format", "complexity", "satd", "clippy", "tests"];

#[derive(Debug, Serialize)]
struct Violation {
    file: String,
    line: u64,
    rule: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct StageReport {
    name: &'static str,
    /// `Some(true/false)` ran and passed/failed; `None` skipped.
    ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skipped: Option<&'static str>,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    violations: Vec<Violation>,
    /// Tail of command output, for failed stages that have no parsed violations.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerifyReport {
    ok: bool,
    duration_ms: u64,
    stages: Vec<StageReport>,
}

/// Run the CI-faithful gate set fail-fast; exit non-zero on any failure.
pub async fn handle_verify(args: VerifyArgs) -> Result<()> {
    let overall = Instant::now();
    let selected: Vec<&str> = match args.stage.as_deref() {
        Some(s) => vec![s],
        None => STAGES
            .iter()
            .copied()
            .filter(|s| !args.skip.iter().any(|k| k == s))
            .collect(),
    };

    let mut stages = Vec::new();
    let mut failed = false;
    for &name in STAGES {
        if !selected.contains(&name) {
            stages.push(skipped(name, "not-selected"));
            continue;
        }
        if failed && !args.no_fail_fast {
            stages.push(skipped(name, "fail-fast"));
            continue;
        }
        let start = Instant::now();
        let (ok, violations, detail) = run_stage(name, &args);
        failed |= !ok;
        stages.push(StageReport {
            name,
            ok: Some(ok),
            skipped: None,
            duration_ms: start.elapsed().as_millis() as u64,
            violations,
            detail: if ok { None } else { detail },
        });
    }

    let report = VerifyReport {
        ok: !failed,
        duration_ms: overall.elapsed().as_millis() as u64,
        stages,
    };
    match args.format {
        VerifyFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        VerifyFormat::Text => print_text(&report),
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}

fn skipped(name: &'static str, why: &'static str) -> StageReport {
    StageReport {
        name,
        ok: None,
        skipped: Some(why),
        duration_ms: 0,
        violations: Vec::new(),
        detail: None,
    }
}

fn run_stage(name: &str, args: &VerifyArgs) -> (bool, Vec<Violation>, Option<String>) {
    match name {
        "format" => stage_format(args),
        "complexity" => stage_complexity(),
        "satd" => stage_satd(),
        "clippy" => stage_clippy(args),
        "tests" => stage_tests(),
        _ => (true, Vec::new(), None),
    }
}

fn cargo() -> Command {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
}

fn pmat_self() -> Command {
    Command::new(std::env::current_exe().unwrap_or_else(|_| PathBuf::from("pmat")))
}

/// Run a command capturing output; return (success, combined stdout+stderr).
fn run(cmd: &mut Command) -> (bool, String) {
    match cmd.output() {
        Ok(out) => {
            let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
            s.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), s)
        }
        Err(e) => (false, format!("failed to spawn command: {e}")),
    }
}

/// Last `n` non-empty lines of output, for actionable failure detail.
fn tail(output: &str, n: usize) -> Option<String> {
    let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return None;
    }
    let start = lines.len().saturating_sub(n);
    Some(lines[start..].join("\n"))
}

fn stage_format(args: &VerifyArgs) -> (bool, Vec<Violation>, Option<String>) {
    if args.fix {
        let _ = run(cargo().args(["fmt", "--all"]));
        return (true, Vec::new(), None);
    }
    let (ok, out) = run(cargo().args(["fmt", "--all", "--", "--check"]));
    (ok, Vec::new(), tail(&out, 20))
}

fn stage_complexity() -> (bool, Vec<Violation>, Option<String>) {
    // The complexity gate is incremental — the pre-commit hook checks staged
    // files. So scope to files changed vs HEAD: a whole-project scan would flag
    // pre-existing high-complexity test files that the gate never gates. No
    // changes ⇒ nothing new to gate ⇒ pass.
    let files = changed_rust_files();
    if files.is_empty() {
        return (true, Vec::new(), None);
    }
    let mut cmd = pmat_self();
    cmd.args([
        "analyze",
        "complexity",
        "--max-cyclomatic",
        "30",
        "--max-cognitive",
        "25",
        "--fail-on-violation",
        "--files",
    ]);
    cmd.arg(files.join(","));
    let (ok, out) = run(&mut cmd);
    (ok, Vec::new(), tail(&out, 25))
}

fn stage_satd() -> (bool, Vec<Violation>, Option<String>) {
    let (ok, out) = run(pmat_self().args(["analyze", "satd", "--strict"]));
    (ok, Vec::new(), tail(&out, 25))
}

fn stage_tests() -> (bool, Vec<Violation>, Option<String>) {
    // Lib tests; clap tests need an 8MB stack or they SIGABRT.
    let (ok, out) = run(cargo()
        .args(["test", "--lib"])
        .env("RUST_MIN_STACK", "8388608"));
    (ok, Vec::new(), tail(&out, 30))
}

/// Selector and lint flags for the clippy stage, matching `ci / lint` exactly.
///
/// The whole promise of `pmat verify` is "green here ⇒ green in CI", so this
/// must track the reusable sovereign-ci workflow, which runs
/// `cargo clippy --all-targets -- -D warnings -A unused-variables`.
///
/// It previously ran `--lib --bins` and omitted `-A unused-variables`, so it
/// diverged in *both* directions: it never linted test/bench/example targets
/// (v3.25.0 shipped seven `clippy::empty_line_after_outer_attr` violations in
/// test files that CI then rejected), and it was stricter than CI on unused
/// variables, which can block work CI would accept.
///
/// Deliberately NOT `--all-features`: that pulls optional batuta-stack feature
/// combos CI never builds and that fail to compile. (PMAT_FAST_BUILD is also
/// deliberately unset — it stubs build.rs codegen and conflicts with a normal
/// build's target state.)
const CLIPPY_TARGETS: &str = "--all-targets";
const CLIPPY_LINTS: &[&str] = &["-D", "warnings", "-A", "unused-variables"];

fn stage_clippy(args: &VerifyArgs) -> (bool, Vec<Violation>, Option<String>) {
    if args.fix {
        let mut fix = cargo();
        fix.args([
            "clippy",
            CLIPPY_TARGETS,
            "--fix",
            "--allow-dirty",
            "--allow-staged",
            "--",
        ])
        .args(CLIPPY_LINTS);
        let _ = run(&mut fix);
    }
    let mut check = cargo();
    check
        .args(["clippy", CLIPPY_TARGETS, "--message-format=json", "--"])
        .args(CLIPPY_LINTS);
    let (ok, out) = run(&mut check);
    let violations = parse_clippy_violations(&out);
    let detail = if violations.is_empty() {
        first_error(&out).or_else(|| tail(&out, 10))
    } else {
        None
    };
    (ok, violations, detail)
}

/// First error-shaped line of a failed command's output.
///
/// Used instead of `tail` when a stage fails with no structured violations. The
/// tail of a cargo build is the *end* of the progress stream ("Compiling pmat
/// v3.28.2 …", "warning: build failed, waiting for other jobs to finish") — never
/// the cause (#640). Cargo's own failures ("error: no such command: `clippy`")
/// arrive as plain text on stderr, which `run` concatenates onto stdout, so they
/// are findable here; `--message-format=json` diagnostics start with `{` and are
/// skipped by the prefix test.
fn first_error(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("error"))
        .map(str::to_string)
}

/// Parse `cargo clippy --message-format=json` output into structured violations.
///
/// Keeps **rustc errors as well as clippy lints**. The filter used to drop every
/// diagnostic whose code did not start with `clippy::`, so a plain compile error
/// — `error[E0063]: missing field ...` — left `violations` empty and pushed
/// `detail` onto its last-10-lines fallback, which for a cargo build is progress
/// noise ("Compiling pmat v3.28.2 …"). The one gate agents are told to trust
/// reported strictly less than it already knew (#640).
///
/// rustc *warnings* stay filtered out: the clippy stage runs with `-D warnings`,
/// so anything that actually fails the build arrives at `level == "error"`, and
/// keeping warnings would bury the cause again under lints CI does not enforce.
fn parse_clippy_violations(json_stream: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    for line in json_stream.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-message") {
            continue;
        }
        let msg = &v["message"];
        let rule = msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let level = msg
            .get("level")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let text = msg
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        // rustc closes a failed build with "aborting due to N previous errors",
        // which carries no code and no span. It is a count, not a cause.
        let is_abort_summary = text.starts_with("aborting due to");
        let keep = rule.starts_with("clippy::") || (level == "error" && !is_abort_summary);
        if !keep {
            continue;
        }
        let span = msg
            .get("spans")
            .and_then(serde_json::Value::as_array)
            .and_then(|a| {
                a.iter()
                    .find(|s| {
                        s.get("is_primary")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false)
                    })
                    .or_else(|| a.first())
            });
        let (file, line) = span
            .map(|s| {
                (
                    s.get("file_name")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    s.get("line_start")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or_default(),
                )
            })
            .unwrap_or_default();
        out.push(Violation {
            file,
            line,
            // A codeless rustc error ("could not compile `cc` (lib) due to 11
            // previous errors") still needs a rule slug the JSON consumer can
            // switch on; `rustc` says which tool spoke.
            rule: if rule.is_empty() {
                "rustc".to_string()
            } else {
                rule.to_string()
            },
            message: text.to_string(),
        });
    }
    out
}

fn changed_rust_files() -> Vec<String> {
    let (ok, out) = run(Command::new("git").args(["diff", "--name-only", "HEAD"]));
    if !ok {
        return Vec::new();
    }
    out.lines()
        .filter(|l| l.ends_with(".rs"))
        .map(str::to_string)
        .collect()
}

fn print_text(report: &VerifyReport) {
    for s in &report.stages {
        let status = match s.ok {
            Some(true) => "\x1b[32m✓ pass\x1b[0m",
            Some(false) => "\x1b[31m✗ FAIL\x1b[0m",
            None => "\x1b[2m- skip\x1b[0m",
        };
        println!("  {status}  {:<11} {}ms", s.name, s.duration_ms);
        for v in &s.violations {
            println!(
                "       \x1b[31m{}\x1b[0m {}:{}  {}",
                v.rule, v.file, v.line, v.message
            );
        }
        if let Some(d) = &s.detail {
            for l in d.lines() {
                println!("       \x1b[2m{l}\x1b[0m");
            }
        }
    }
    if report.ok {
        println!(
            "\n\x1b[32m✓ verify passed\x1b[0m ({}ms) — safe to commit",
            report.duration_ms
        );
    } else {
        println!(
            "\n\x1b[31m✗ verify failed\x1b[0m ({}ms) — fix before committing",
            report.duration_ms
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clippy_violations() {
        let stream = r#"{"reason":"compiler-message","message":{"level":"error","code":{"code":"clippy::nonminimal_bool"},"message":"this boolean expression can be simplified","spans":[{"file_name":"src/x.rs","line_start":230,"is_primary":true}]}}
{"reason":"compiler-artifact","package_id":"x"}
{"reason":"compiler-message","message":{"level":"warning","code":{"code":"dead_code"},"message":"never used","spans":[{"file_name":"src/y.rs","line_start":5,"is_primary":true}]}}"#;
        let v = parse_clippy_violations(stream);
        // Only the clippy:: lint is kept (dead_code is rustc, not clippy).
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, "clippy::nonminimal_bool");
        assert_eq!(v[0].file, "src/x.rs");
        assert_eq!(v[0].line, 230);
    }

    /// A rustc compile error must reach the report (#640).
    ///
    /// This is the exact shape that used to render as
    /// `{"name":"clippy","ok":false,"detail":"{\"reason\":\"build-finished\"…"}`
    /// — a red gate whose output did not contain the error.
    #[test]
    fn test_parse_clippy_violations_keeps_rustc_errors() {
        let stream = r#"{"reason":"compiler-message","message":{"level":"error","code":{"code":"E0063"},"message":"missing field `tdg_measured` in initializer of `EvidenceSummary`","spans":[{"file_name":"src/services/evidence.rs","line_start":412,"is_primary":true}]}}
{"reason":"compiler-message","message":{"level":"error","code":null,"message":"aborting due to 1 previous error","spans":[]}}
{"reason":"compiler-message","message":{"level":"error","code":null,"message":"could not compile `cc` (lib) due to 11 previous errors","spans":[]}}
{"reason":"build-finished","success":false}"#;
        let v = parse_clippy_violations(stream);

        assert_eq!(v.len(), 2, "got {v:?}");
        assert_eq!(v[0].rule, "E0063");
        assert_eq!(v[0].file, "src/services/evidence.rs");
        assert_eq!(v[0].line, 412);
        assert!(v[0].message.contains("tdg_measured"));
        // Codeless rustc errors still carry a slug and their text.
        assert_eq!(v[1].rule, "rustc");
        assert!(v[1].message.contains("could not compile `cc`"));
        // "aborting due to N previous errors" is a count, not a cause.
        assert!(!v.iter().any(|x| x.message.starts_with("aborting due to")));
    }

    /// rustc *warnings* must stay out — `-D warnings` promotes the ones that
    /// matter to errors, and keeping the rest re-buries the cause.
    #[test]
    fn test_parse_clippy_violations_drops_rustc_warnings() {
        let stream = r#"{"reason":"compiler-message","message":{"level":"warning","code":{"code":"unused_imports"},"message":"unused import: `std::fmt`","spans":[{"file_name":"src/a.rs","line_start":3,"is_primary":true}]}}"#;
        assert!(parse_clippy_violations(stream).is_empty());
    }

    #[test]
    fn test_tail() {
        assert_eq!(tail("a\n\nb\nc", 2).as_deref(), Some("b\nc"));
        assert_eq!(tail("", 5), None);
    }

    /// The failure detail must be the first error, not the tail of the progress
    /// stream (#640).
    #[test]
    fn test_first_error_prefers_the_cause_over_progress_noise() {
        let out = "   Compiling pmat v3.29.1\n\
                   error: no such command: `clippy`\n\
                   \n\
                   	Did you mean `check`?\n\
                      Compiling serde v1.0.0\n\
                   warning: build failed, waiting for other jobs to finish";
        assert_eq!(
            first_error(out).as_deref(),
            Some("error: no such command: `clippy`")
        );
        // tail(10) would have returned the trailing progress lines instead.
        assert!(tail(out, 10).unwrap().contains("waiting for other jobs"));
    }

    #[test]
    fn test_first_error_none_when_output_has_no_error_line() {
        assert_eq!(first_error("   Compiling pmat v3.29.1\n    Finished"), None);
        assert_eq!(first_error(""), None);
    }
}
