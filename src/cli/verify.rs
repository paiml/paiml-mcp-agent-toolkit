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
use std::path::{Path, PathBuf};
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

/// What one stage produced.
///
/// `verify` used to have only two answers — pass or fail — and so had nowhere to
/// put "I could not run here". Both directions were wrong at once: in an empty
/// directory it printed `✓ pass complexity / ✓ pass satd / ✓ verify passed —
/// safe to commit` and exited 0, a green pre-commit verdict over zero files;
/// and in the same directory (and in any non-Cargo project) the format stage
/// reported `ok:false` whose evidence was **rustfmt's usage screen**, i.e. a
/// tool-invocation error surfaced as a code-quality violation.
///
/// `NotApplicable` is the third answer `enforce` and `cuda-tdg` already have. It
/// is not a pass: it does not fail the run either, but a run in which NOTHING
/// was measured cannot report "safe to commit".
#[derive(Debug)]
enum StageResult {
    Ran {
        ok: bool,
        violations: Vec<Violation>,
        detail: Option<String>,
    },
    /// The stage's tool had nothing here to check, and said so.
    NotApplicable(String),
}

impl StageResult {
    fn pass() -> Self {
        Self::Ran {
            ok: true,
            violations: Vec::new(),
            detail: None,
        }
    }
    fn ran(ok: bool, violations: Vec<Violation>, detail: Option<String>) -> Self {
        Self::Ran {
            ok,
            violations,
            detail,
        }
    }
    fn not_applicable(reason: impl Into<String>) -> Self {
        Self::NotApplicable(reason.into())
    }
}

#[derive(Debug, Serialize)]
struct StageReport {
    name: &'static str,
    /// `Some(true/false)` ran and passed/failed; `None` skipped or not applicable.
    ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skipped: Option<&'static str>,
    /// Why this stage measured nothing. Never a pass, never a failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    not_applicable: Option<String>,
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
    /// How many selected stages actually produced a measurement.
    stages_measured: usize,
    /// Set when `stages_measured == 0`: there is no verdict to give.
    #[serde(skip_serializing_if = "Option::is_none")]
    not_measured: Option<String>,
    duration_ms: u64,
    stages: Vec<StageReport>,
}

/// Run the CI-faithful gate set fail-fast; exit non-zero on any failure.
pub async fn handle_verify(args: VerifyArgs) -> Result<()> {
    let overall = Instant::now();
    let selected = select_stages(args.stage.as_deref(), &args.skip)?;

    let mut stages = Vec::new();
    let mut failed = false;
    let mut measured = 0usize;
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
        let duration_ms = |start: Instant| start.elapsed().as_millis() as u64;
        match run_stage(name, &args) {
            StageResult::Ran {
                ok,
                violations,
                detail,
            } => {
                measured += 1;
                failed |= !ok;
                stages.push(StageReport {
                    name,
                    ok: Some(ok),
                    skipped: None,
                    not_applicable: None,
                    duration_ms: duration_ms(start),
                    violations,
                    detail: if ok { None } else { detail },
                });
            }
            StageResult::NotApplicable(reason) => stages.push(StageReport {
                name,
                ok: None,
                skipped: None,
                not_applicable: Some(reason),
                duration_ms: duration_ms(start),
                violations: Vec::new(),
                detail: None,
            }),
        }
    }

    // A run that measured nothing has not passed. This is the half of the
    // contract `enforce` already had and `verify` did not: the two gave opposite
    // verdicts on an empty directory ("safe to commit" vs "Violating,
    // complexity/satd not measured").
    let not_measured = (measured == 0).then(|| {
        "no selected stage could measure anything here, so verify has no verdict to give"
            .to_string()
    });
    let report = VerifyReport {
        ok: !failed && measured > 0,
        stages_measured: measured,
        not_measured,
        duration_ms: overall.elapsed().as_millis() as u64,
        stages,
    };
    match args.format {
        VerifyFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        VerifyFormat::Text => print_text(&report),
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

/// Resolve which stages to run, rejecting a `--stage` that is not a stage.
///
/// An unrecognised name used to be taken at face value, so it matched no stage:
/// every stage recorded "not-selected", `failed` stayed false, and
/// `pmat verify --stage nonexistent` printed `ok:true` and exited 0 on a tree
/// whose format stage was red. A typo in the one gate agents are told to trust
/// read as a green tree.
fn select_stages(stage: Option<&str>, skip: &[String]) -> Result<Vec<&'static str>> {
    match stage {
        Some(s) => match STAGES.iter().find(|k| **k == s) {
            Some(&known) => Ok(vec![known]),
            None => anyhow::bail!("unknown stage `{s}`; valid stages: {}", STAGES.join(",")),
        },
        None => Ok(STAGES
            .iter()
            .copied()
            .filter(|s| !skip.iter().any(|k| k == s))
            .collect()),
    }
}

fn skipped(name: &'static str, why: &'static str) -> StageReport {
    StageReport {
        name,
        ok: None,
        skipped: Some(why),
        not_applicable: None,
        duration_ms: 0,
        violations: Vec::new(),
        detail: None,
    }
}

fn run_stage(name: &str, args: &VerifyArgs) -> StageResult {
    match name {
        "format" => stage_format(args),
        "complexity" => stage_complexity(),
        "satd" => stage_satd(),
        "clippy" => stage_clippy(args),
        "tests" => stage_tests(),
        // A name that is not a stage must never read as a pass. `select_stages`
        // rejects unknown names up front, so getting here means this match and
        // `STAGES` have drifted apart — report that, don't return green.
        other => StageResult::ran(
            false,
            Vec::new(),
            Some(format!("no such verify stage: `{other}`")),
        ),
    }
}

/// Is the working directory inside a Cargo project?
///
/// `cargo fmt`, `cargo clippy` and `cargo test` all refuse outside one, and
/// rustfmt answers with "could not find Cargo.toml" followed by its whole usage
/// screen. `verify` recorded that screen as the failure detail, telling an agent
/// the code was misformatted when the real fact is that the directory is not a
/// Cargo project — a fact about applicability, not about quality.
fn in_cargo_project(dir: &Path) -> bool {
    dir.canonicalize()
        .as_deref()
        .unwrap_or(dir)
        .ancestors()
        .any(|a| a.join("Cargo.toml").is_file())
}

/// Extensions the pmat-native stages (complexity, satd) can read.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "ts", "tsx", "js", "jsx", "go", "c", "h", "cc", "cpp", "hpp", "java", "kt", "rb",
    "php", "swift", "sh", "bash", "lua", "sql", "scala",
];

/// Does `dir` contain any file matching `wanted`, honouring `.gitignore`?
fn any_source_file(dir: &Path, wanted: &[&str]) -> bool {
    ignore::WalkBuilder::new(dir)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build()
        .filter_map(std::result::Result::ok)
        .any(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| wanted.contains(&x))
        })
}

fn cargo() -> Command {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
}

fn pmat_self() -> Command {
    Command::new(std::env::current_exe().unwrap_or_else(|_| PathBuf::from("pmat")))
}

/// Strip ANSI CSI/OSC escape sequences from captured child output.
///
/// `cargo fmt --check` colours its diff, and `verify` captured that verbatim
/// into `StageReport::detail`. So `pmat verify --format json` emitted raw
/// `\x1b[…m` bytes inside a JSON string — unreadable for the autonomous agent
/// the JSON format exists for — and `--color never` changed nothing, because
/// the escapes came from the child, not from us.
pub(crate) fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.next() {
            // CSI: `ESC [` … final byte in 0x40..=0x7E.
            Some('[') => {
                for f in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&f) {
                        break;
                    }
                }
            }
            // OSC: `ESC ]` … terminated by BEL or `ESC \`.
            Some(']') => {
                while let Some(f) = chars.next() {
                    if f == '\x07' {
                        break;
                    }
                    if f == '\x1b' {
                        let _ = chars.next();
                        break;
                    }
                }
            }
            // Any other two-byte escape: drop both bytes.
            Some(_) | None => {}
        }
    }
    out
}

/// Run a command capturing output; return (success, combined stdout+stderr).
///
/// Output is de-ANSI'd here rather than at each call site: every consumer of it
/// is a machine-readable `detail` field or a line `print_text` re-colours
/// itself, so a child's colours can only corrupt them.
fn run(cmd: &mut Command) -> (bool, String) {
    // Belt and braces with `strip_ansi`: ask cargo not to colour in the first
    // place, so the common case needs no stripping at all.
    cmd.env("CARGO_TERM_COLOR", "never");
    match cmd.output() {
        Ok(out) => {
            let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
            s.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), strip_ansi(&s))
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

fn stage_format(args: &VerifyArgs) -> StageResult {
    if !in_cargo_project(Path::new(".")) {
        return StageResult::not_applicable(
            "cargo fmt needs a Cargo project; no Cargo.toml in this directory or any parent",
        );
    }
    if args.fix {
        let _ = run(cargo().args(["fmt", "--all"]));
        return StageResult::pass();
    }
    let (ok, out) = run(cargo().args(["fmt", "--all", "--", "--check"]));
    StageResult::ran(ok, Vec::new(), format_detail(&out))
}

/// Failure detail for the format stage.
///
/// Same reason as the clippy stage: when `cargo fmt` fails to even start — run
/// outside a crate, it exits with "error: could not find Cargo.toml" followed
/// by its whole rustfmt usage screen — `tail` returned the usage screen and
/// buried the one actionable line, so `verify --format json` told an agent the
/// code was misformatted when the real problem was that the directory is not a
/// cargo project (#640, same family).
fn format_detail(output: &str) -> Option<String> {
    first_error(output).or_else(|| tail(output, 20))
}

fn stage_complexity() -> StageResult {
    // The complexity gate is incremental — the pre-commit hook checks staged
    // files. So scope to files changed vs HEAD: a whole-project scan would flag
    // pre-existing high-complexity test files that the gate never gates.
    //
    // "No changes" used to return a PASS. It is not one: nothing was measured,
    // and in an empty directory that green tick was the whole of verify's
    // verdict.
    let files = changed_rust_files();
    if files.is_empty() {
        return if any_source_file(Path::new("."), &["rs"]) {
            StageResult::not_applicable("no Rust files changed vs HEAD, so nothing was measured")
        } else {
            StageResult::not_applicable("no Rust source files here, so nothing was measured")
        };
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
    StageResult::ran(ok, Vec::new(), tail(&out, 25))
}

fn stage_satd() -> StageResult {
    // A tree with no source files gives the detector nothing to read, and
    // "0 SATD violations over 0 files" is the absence of a measurement, not a
    // clean one.
    if !any_source_file(Path::new("."), SOURCE_EXTENSIONS) {
        return StageResult::not_applicable("no source files here, so nothing was measured");
    }
    // This stage used to shell out to `analyze satd --strict` with no
    // `--fail-on-violation` and take the child's exit status as its verdict, so
    // it stayed green on a tree carrying two fresh debt markers: the subcommand
    // prints "Found 3 SATD violations" and still exits 0. Ask for the JSON
    // report and read the count ourselves — the gate's verdict must come from
    // the measurement, not from an exit code that never moves.
    //
    // The sentence above used to wrap so that a line began with a debt marker,
    // and the detector flagged this comment as debt. It was right by its own
    // rule, which is line-oriented; prose that narrates a marker must not start
    // a line with one. Noted rather than silenced, because the same shape will
    // reach anyone documenting a marker they removed.
    let (_, out) = run(pmat_self().args([
        "analyze",
        "satd",
        "--strict",
        "--format",
        "json",
        "--fail-on-violation",
    ]));
    let (ok, detail) = satd_verdict(&out);
    StageResult::ran(ok, Vec::new(), detail)
}

/// Decide the satd stage from `analyze satd --format json` output.
///
/// No parseable count means the subcommand produced no report; an unmeasured
/// gate must not read as a pass.
fn satd_verdict(output: &str) -> (bool, Option<String>) {
    match parse_satd_violation_count(output) {
        Some(0) => (true, None),
        Some(n) => (
            false,
            Some(format!(
                "{n} strict-mode SATD violation(s) (TODO/FIXME/HACK/BUG)\n{}",
                tail(output, 25).unwrap_or_default()
            )),
        ),
        None => (
            false,
            first_error(output)
                .or_else(|| tail(output, 25))
                .or_else(|| Some("analyze satd produced no violation count".to_string())),
        ),
    }
}

/// Violation count out of `analyze satd --format json` (`"total_violations": N`).
fn parse_satd_violation_count(output: &str) -> Option<u64> {
    const KEY: &str = "\"total_violations\"";
    let after = output.lines().find_map(|l| l.split_once(KEY))?.1;
    let digits: String = after
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn stage_tests() -> StageResult {
    if !in_cargo_project(Path::new(".")) {
        return StageResult::not_applicable(
            "cargo test needs a Cargo project; no Cargo.toml in this directory or any parent",
        );
    }
    // Lib tests; clap tests need an 8MB stack or they SIGABRT.
    let (ok, out) = run(cargo()
        .args(["test", "--lib"])
        .env("RUST_MIN_STACK", "8388608"));
    StageResult::ran(ok, Vec::new(), tail(&out, 30))
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

fn stage_clippy(args: &VerifyArgs) -> StageResult {
    if !in_cargo_project(Path::new(".")) {
        return StageResult::not_applicable(
            "cargo clippy needs a Cargo project; no Cargo.toml in this directory or any parent",
        );
    }
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
    StageResult::ran(ok, violations, detail)
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
    changed_rust_files_in(Path::new("."))
}

/// Rust files a commit here would introduce or change.
///
/// `git diff --name-only HEAD` lists tracked modifications *only*: a brand-new
/// file is invisible to it until it is `git add`ed, so a freshly written module
/// with a cyclomatic-46 function passed the complexity stage with `ok:true` and
/// only went red after staging. Union the working-tree diff with the index and
/// with untracked-but-not-ignored files, so the gate sees exactly what a commit
/// would carry.
fn changed_rust_files_in(dir: &Path) -> Vec<String> {
    const SOURCES: [&[&str]; 3] = [
        &["diff", "--name-only", "HEAD"],
        &["diff", "--name-only", "--cached"],
        &["ls-files", "--others", "--exclude-standard"],
    ];
    let mut files: Vec<String> = Vec::new();
    for args in SOURCES {
        let (ok, out) = run(Command::new("git").args(args).current_dir(dir));
        if !ok {
            continue;
        }
        for line in out.lines().filter(|l| l.ends_with(".rs")) {
            if !files.iter().any(|f| f == line) {
                files.push(line.to_string());
            }
        }
    }
    files
}

/// Render the text report.
///
/// The escape sequences go through `colors::seq`, which is `""` when colour is
/// off. They used to be interpolated as bare literals, so `pmat verify --color
/// never > verify.txt` wrote `^[[32m✓ pass^[[0m` into the file exactly like
/// `--color auto` did — the flag parsed and changed nothing (GH #684, same
/// family).
fn print_text(report: &VerifyReport) {
    use crate::cli::colors as c;
    // Gate explicitly and take the bytes with `Sgr::raw`, which is documented
    // as ungated: `c::seq` is now an identity on `Sgr`, so interpolating its
    // result emits the escape whatever `--color` says.
    let on = c::colors_enabled();
    let sgr = |s: c::Sgr| if on { s.raw() } else { "" };
    let (green, red, dim, reset) = (sgr(c::GREEN), sgr(c::RED), sgr(c::DIM), sgr(c::RESET));
    for s in &report.stages {
        let status = match (s.ok, s.not_applicable.is_some()) {
            (Some(true), _) => format!("{green}✓ pass{reset}"),
            (Some(false), _) => format!("{red}✗ FAIL{reset}"),
            // A stage that could not measure is neither a pass nor a skip the
            // user asked for; it gets its own mark and its reason.
            (None, true) => format!("{dim}~ n/a {reset}"),
            (None, false) => format!("{dim}- skip{reset}"),
        };
        println!("  {status}  {:<11} {}ms", s.name, s.duration_ms);
        if let Some(reason) = &s.not_applicable {
            println!("       {dim}{reason}{reset}");
        }
        for v in &s.violations {
            println!(
                "       {red}{}{reset} {}:{}  {}",
                v.rule, v.file, v.line, v.message
            );
        }
        if let Some(d) = &s.detail {
            for l in d.lines() {
                println!("       {dim}{l}{reset}");
            }
        }
    }
    if let Some(reason) = &report.not_measured {
        // "safe to commit" over zero measured stages was the defect: an empty
        // directory got a green pre-commit verdict.
        println!(
            "\n{red}✗ verify measured nothing{reset} ({}ms) — {reason}",
            report.duration_ms
        );
    } else if report.ok {
        println!(
            "\n{green}✓ verify passed{reset} ({}ms) — safe to commit",
            report.duration_ms
        );
    } else {
        println!(
            "\n{red}✗ verify failed{reset} ({}ms) — fix before committing",
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

    /// The format stage run outside a cargo project must report the cargo
    /// error, not rustfmt's usage screen. This is the verbatim shape 3.29.0
    /// emitted from a directory with no Cargo.toml.
    #[test]
    fn test_format_detail_keeps_the_cargo_error_over_the_usage_screen() {
        let out = "error: could not find `Cargo.toml` in `/tmp/empty` or any parent directory\n\
                   Usage: cargo fmt [OPTIONS] [-- <rustfmt_options>...]\n\
                   Arguments:\n\
                   \x20 [rustfmt_options]...  Options passed to rustfmt\n\
                   Options:\n\
                   \x20 -q, --quiet\n\
                   \x20 -h, --help\n\
                   \x20         Print help";
        let detail = format_detail(out).expect("a failing format stage must say why");
        assert!(
            detail.contains("could not find `Cargo.toml`"),
            "the cause was dropped: {detail}"
        );
        assert!(
            !detail.contains("Print help"),
            "the usage screen is not a failure detail: {detail}"
        );
    }

    /// A genuine formatting diff has no `error:` line, so the tail is still the
    /// right detail there.
    #[test]
    fn test_format_detail_falls_back_to_the_diff_tail() {
        let out = "Diff in /x/src/lib.rs at line 3:\n-fn a(){}\n+fn a() {}\n";
        let detail = format_detail(out).expect("detail");
        assert!(detail.contains("Diff in /x/src/lib.rs"), "{detail}");
    }

    #[test]
    fn test_first_error_none_when_output_has_no_error_line() {
        assert_eq!(first_error("   Compiling pmat v3.29.1\n    Finished"), None);
        assert_eq!(first_error(""), None);
    }

    /// `--stage nonexistent` used to select nothing and report ok:true/exit 0.
    #[test]
    fn test_select_stages_rejects_an_unknown_stage_name() {
        let err = select_stages(Some("nonexistent"), &[])
            .unwrap_err()
            .to_string();
        assert!(err.contains("nonexistent"), "{err}");
        // The message has to say what a valid stage is.
        for s in STAGES {
            assert!(err.contains(s), "{err} is missing {s}");
        }
    }

    #[test]
    fn test_select_stages_known_name_and_skip_list() {
        assert_eq!(select_stages(Some("satd"), &[]).unwrap(), vec!["satd"]);
        let skip = vec!["format".to_string(), "tests".to_string()];
        assert_eq!(
            select_stages(None, &skip).unwrap(),
            vec!["complexity", "satd", "clippy"]
        );
    }

    /// A name that is not a stage must never come back as a pass.
    #[test]
    fn test_run_stage_unknown_name_is_not_a_pass() {
        let args = VerifyArgs {
            format: VerifyFormat::Json,
            fix: false,
            no_fail_fast: false,
            skip: Vec::new(),
            stage: None,
        };
        match run_stage("nonexistent", &args) {
            StageResult::Ran { ok, detail, .. } => {
                assert!(!ok);
                assert!(detail.unwrap_or_default().contains("nonexistent"));
            }
            other => panic!("a name that is not a stage must not be not-applicable: {other:?}"),
        }
    }

    /// The satd stage must fail on debt the subcommand reports while exiting 0.
    #[test]
    fn test_satd_verdict_fails_on_reported_violations() {
        let report = r#"{
  "total_files": 1,
  "total_violations": 3,
  "summary": "Found 3 SATD violations in 1 files"
}"#;
        let (ok, detail) = satd_verdict(report);
        assert!(!ok, "3 reported violations must fail the stage");
        assert!(detail.unwrap_or_default().contains('3'));
        assert_eq!(parse_satd_violation_count(report), Some(3));
    }

    #[test]
    fn test_satd_verdict_passes_only_on_a_measured_zero() {
        let (ok, _) = satd_verdict("{\n  \"total_violations\": 0\n}");
        assert!(ok);
        // No count in the output ⇒ nothing was measured ⇒ not a pass.
        let (ok, detail) = satd_verdict("error: no such subcommand: `satd`");
        assert!(!ok);
        assert!(detail.unwrap_or_default().contains("no such subcommand"));
        assert_eq!(parse_satd_violation_count(""), None);
    }

    /// A new, not-yet-staged .rs file must be visible to the complexity gate:
    /// it used to be invisible until `git add`, so a cyclomatic-46 module
    /// passed verify and only failed after staging.
    #[test]
    fn test_changed_rust_files_sees_untracked_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path();
        let git = |args: &[&str]| {
            let ok = Command::new("git")
                .args(args)
                .current_dir(p)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "t@example.com"]);
        git(&["config", "user.name", "t"]);
        std::fs::write(p.join("tracked.rs"), "pub fn a() {}\n").expect("write");
        git(&["add", "tracked.rs"]);
        git(&["commit", "-qm", "init"]);

        // Brand-new file, never staged, plus an ignored one and a non-Rust one.
        std::fs::write(p.join("untracked.rs"), "pub fn b() {}\n").expect("write");
        std::fs::write(p.join("notes.md"), "hi\n").expect("write");
        std::fs::write(p.join(".gitignore"), "ignored.rs\n").expect("write");
        std::fs::write(p.join("ignored.rs"), "pub fn c() {}\n").expect("write");

        let files = changed_rust_files_in(p);
        assert!(
            files.iter().any(|f| f == "untracked.rs"),
            "untracked .rs must be gated: {files:?}"
        );
        assert!(!files.iter().any(|f| f == "ignored.rs"), "{files:?}");
        assert!(!files.iter().any(|f| f.ends_with(".md")), "{files:?}");

        // Staged and modified files stay in, exactly once each.
        std::fs::write(p.join("tracked.rs"), "pub fn a() -> u8 { 1 }\n").expect("write");
        git(&["add", "untracked.rs", "tracked.rs"]);
        let files = changed_rust_files_in(p);
        assert_eq!(
            files.iter().filter(|f| *f == "untracked.rs").count(),
            1,
            "{files:?}"
        );
        assert!(files.iter().any(|f| f == "tracked.rs"), "{files:?}");
    }
}

#[cfg(test)]
mod ansi_tests {
    use super::*;

    /// `cargo fmt --check` colours its diff, and that colour used to land
    /// verbatim inside `--format json` `detail` strings. A machine-readable
    /// field must not carry terminal escapes.
    #[test]
    fn strip_ansi_removes_csi_sequences() {
        let coloured = "Diff in \u{1b}[1msrc/lib.rs\u{1b}[0m at line \u{1b}[31m3\u{1b}[0m:";
        let plain = strip_ansi(coloured);
        assert!(!plain.contains('\u{1b}'), "got: {plain:?}");
        assert_eq!(plain, "Diff in src/lib.rs at line 3:");
    }

    #[test]
    fn strip_ansi_removes_osc_and_leaves_plain_text_alone() {
        assert_eq!(strip_ansi("\u{1b}]0;title\u{7}body"), "body");
        assert_eq!(strip_ansi("no escapes here"), "no escapes here");
        // A lone ESC at the very end must not spin or panic.
        assert_eq!(strip_ansi("tail\u{1b}"), "tail");
    }

    /// The detail helpers feed off `run`'s output, so once that is stripped the
    /// tail/first-error extraction is escape-free too.
    #[test]
    fn detail_helpers_carry_no_escapes_once_stripped() {
        let raw = "\u{1b}[1mDiff in a.rs\u{1b}[0m\n\u{1b}[31merror: something broke\u{1b}[0m\n";
        let cleaned = strip_ansi(raw);
        assert_eq!(
            first_error(&cleaned).as_deref(),
            Some("error: something broke")
        );
        assert!(!tail(&cleaned, 5).unwrap().contains('\u{1b}'));
    }

    // ── R06: verify needs a measured/unmeasured contract ────────────────────

    /// The stage a Cargo tool cannot run in must report not-applicable, not a
    /// failure whose evidence is rustfmt's usage screen.
    #[test]
    fn cargo_stages_are_not_applicable_outside_a_cargo_project() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(
            !in_cargo_project(dir.path()),
            "a bare tempdir has no Cargo.toml in any ancestor"
        );
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").expect("write");
        assert!(in_cargo_project(dir.path()));
        assert!(
            in_cargo_project(&dir.path().join("src")),
            "a subdirectory of a Cargo project is still in it"
        );
    }

    /// The "nothing to measure" detector: an empty tree has no source at all.
    #[test]
    fn source_file_detection_distinguishes_empty_from_populated() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(!any_source_file(dir.path(), SOURCE_EXTENSIONS));
        assert!(!any_source_file(dir.path(), &["rs"]));

        std::fs::write(dir.path().join("main.go"), "package main\n").expect("write");
        assert!(
            any_source_file(dir.path(), SOURCE_EXTENSIONS),
            "a Go file is source the pmat-native stages can read"
        );
        assert!(
            !any_source_file(dir.path(), &["rs"]),
            "...but it is not Rust, so the complexity stage still has nothing"
        );

        std::fs::write(dir.path().join("lib.rs"), "pub fn a() {}\n").expect("write");
        assert!(any_source_file(dir.path(), &["rs"]));
    }

    /// The report verdict: measuring nothing is not passing.
    ///
    /// `pmat verify --skip format,clippy,tests` in an empty directory printed
    /// "\u{2713} pass complexity / \u{2713} pass satd / \u{2713} verify passed \u{2014} safe to commit"
    /// and exited 0 — a green pre-commit verdict over zero files.
    #[test]
    fn a_run_that_measured_nothing_is_not_a_pass() {
        let stage = |name: &'static str, ok: Option<bool>, na: Option<&str>| StageReport {
            name,
            ok,
            skipped: None,
            not_applicable: na.map(str::to_string),
            duration_ms: 0,
            violations: Vec::new(),
            detail: None,
        };

        let measured = |stages: Vec<StageReport>| -> (bool, usize) {
            let n = stages.iter().filter(|s| s.ok.is_some()).count();
            let failed = stages.iter().any(|s| s.ok == Some(false));
            (!failed && n > 0, n)
        };

        let (ok, n) = measured(vec![
            stage("complexity", None, Some("no Rust source files here")),
            stage("satd", None, Some("no source files here")),
        ]);
        assert_eq!(n, 0);
        assert!(!ok, "zero measured stages must not report a pass");

        let (ok, n) = measured(vec![
            stage("format", Some(true), None),
            stage("complexity", None, Some("no Rust files changed vs HEAD")),
        ]);
        assert_eq!(n, 1);
        assert!(
            ok,
            "one real pass alongside a not-applicable stage is green"
        );

        let (ok, _) = measured(vec![
            stage("format", Some(false), None),
            stage("complexity", None, Some("no Rust files changed vs HEAD")),
        ]);
        assert!(!ok);
    }
}
