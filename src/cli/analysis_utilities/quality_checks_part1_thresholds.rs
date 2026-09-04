// Threshold checks — file size, churn, lint (AD-05; spec §4.4–4.6).
//
// include!()d into `analysis_utilities` from mod.rs, so this file is a fragment
// of that module: it carries no `//!` docs and re-imports nothing mod.rs already
// has in scope (`Path`, `PathBuf`, `Result`, …).
//
// Each of the three is a NAMED finding type with its own counter and its own
// disclosure, never folded into an existing threshold's count (spec §10). The
// two that can be UNANSWERABLE — churn without a git repository, lint without a
// Cargo.toml — say so in an advisory `scope` row rather than returning the empty
// list a clean project returns, which is the rule `security_scope_disclosure`
// and `run_coverage_check` already apply: a check that did not run has not
// passed.

/// The thresholds the file-size and churn gates enforce, resolved once.
///
/// A struct rather than two more positional parameters on an 11-argument
/// function: the public `handle_quality_gate` signature does not move, and a
/// caller cannot transpose two same-typed numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityThresholds {
    /// Lines above which a source file is a `file-size` violation.
    pub max_file_lines: usize,
    /// Commits in the last 90 days above which a source file is a `churn`
    /// violation.
    pub max_churn_commits_90d: usize,
}

impl Default for QualityThresholds {
    /// The shipped defaults, taken FROM the config schema rather than restated:
    /// `[quality] max_file_lines = 500`, `max_churn_commits_90d = 20`.
    fn default() -> Self {
        let quality = crate::services::configuration_service::QualityConfig::default();
        Self {
            max_file_lines: quality.max_file_lines,
            max_churn_commits_90d: quality.max_churn_commits_90d,
        }
    }
}

impl QualityThresholds {
    /// Resolve the thresholds this run will apply: an explicit CLI value wins,
    /// then `pmat.toml [quality]`, then the built-in default.
    ///
    /// The CLI value outranks project config for the reason #683 records for
    /// `--min-entropy`: a number the user typed that the run silently replaces
    /// is a request with no effect.
    #[must_use]
    pub fn resolve(
        project_path: &Path,
        cli_max_file_lines: Option<usize>,
        cli_max_churn_commits: Option<usize>,
    ) -> Self {
        let defaults = Self::default();
        let quality = pmat_toml_quality_table(project_path);
        let from_config = |keys: &[&str]| -> Option<usize> {
            let table = quality.as_ref()?;
            keys.iter()
                .find_map(|k| table.get(*k))
                .and_then(toml::Value::as_integer)
                .and_then(|v| usize::try_from(v).ok())
        };
        Self {
            max_file_lines: cli_max_file_lines
                .or_else(|| from_config(&["max_file_lines"]))
                .unwrap_or(defaults.max_file_lines),
            max_churn_commits_90d: cli_max_churn_commits
                // The schema key carries the window; the CLI flag's spelling is
                // accepted too, exactly as `QualityConfig`'s serde alias does.
                .or_else(|| from_config(&["max_churn_commits_90d", "max_churn_commits"]))
                .unwrap_or(defaults.max_churn_commits_90d),
        }
    }
}

/// `pmat.toml`'s `[quality]` table, or `None` when the file is absent or does
/// not parse.
///
/// Unparsable is deliberately `None` and not an error here: `handle_project_quality_gate`
/// already BLOCKS on a config file that exists and does not parse
/// (`unparsable_gate_configs`), so re-reporting it from every reader would say
/// the same thing many times.
fn pmat_toml_quality_table(project_path: &Path) -> Option<toml::Table> {
    let content = std::fs::read_to_string(project_path.join("pmat.toml")).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table.get("quality")?.as_table().cloned()
}

/// Source files a threshold check measures, walked the way the rest of the gate
/// walks (gitignore honoured, hidden skipped, build artifacts dropped).
///
/// Deliberately the same `ignore` walk `count_examined_sources` uses, so the
/// population these checks measure over cannot drift from the population the
/// gate reports as `files_examined`.
fn threshold_source_files(project_path: &Path) -> Vec<PathBuf> {
    ignore::WalkBuilder::new(project_path)
        .hidden(true)
        .git_ignore(true)
        .build()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .map(ignore::DirEntry::into_path)
        .filter(|p| is_source_file(p) && !is_build_artifact(p))
        .collect()
}

/// Render a path for a finding: relative to the project when possible, so the
/// message does not carry a temp directory nobody else can resolve.
fn display_path(project_path: &Path, file: &Path) -> String {
    file.strip_prefix(project_path)
        .unwrap_or(file)
        .display()
        .to_string()
}

// ── file-size ────────────────────────────────────────────────────────────────

/// Files longer than `max_lines`, one violation each.
///
/// §4.6: `max_file_lines` existed only as a `pmat work` contract CLAIM — nothing
/// refused a file that grew past it. This is the gate half of the same number.
///
/// # Errors
/// Propagates a read error on a file the walk offered.
pub async fn check_file_size(
    project_path: &Path,
    max_lines: usize,
) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    for file in threshold_source_files(project_path) {
        // A file that cannot be read as UTF-8 is not a file with zero lines;
        // it is one this check did not measure, and the walk's own extension
        // filter already restricts us to text source. Skip rather than guess.
        let Ok(content) = tokio::fs::read_to_string(&file).await else {
            continue;
        };
        let lines = content.lines().count();
        if lines > max_lines {
            let shown = display_path(project_path, &file);
            violations.push(QualityViolation {
                check_type: "file-size".to_string(),
                severity: "error".to_string(),
                file: shown.clone(),
                line: None,
                message: format!(
                    "{shown} is {lines} lines, over the {max_lines}-line maximum"
                ),
                details: Some(ViolationDetails {
                    affected_files: vec![shown],
                    example_code: None,
                    fix_suggestion: Some(format!(
                        "Split the file, or raise the cap with --max-file-lines \
                         or `[quality] max_file_lines` in pmat.toml (currently {max_lines})."
                    )),
                    score_factors: vec![format!("lines: {lines} > {max_lines}")],
                }),
            });
        }
    }
    Ok(violations)
}

// ── churn ────────────────────────────────────────────────────────────────────

/// The window the churn gate asks about, in days.
pub const CHURN_WINDOW_DAYS: u32 = 90;

/// Files touched by more than `max_commits` commits in the last 90 days.
///
/// §4.5: churn was measured (`pmat analyze churn`) and gated nowhere. One
/// `git log` pass, not one per file — N subprocesses over a real repository is
/// the difference between a gate and a coffee break.
///
/// A directory that is not a git repository yields NO violations and one
/// advisory `scope` row: "no history" and "quiet history" are different claims,
/// and this is the same disclosure the security check makes about its reach.
///
/// # Errors
/// Never fails on a missing or broken git; that is disclosed, not raised.
pub async fn check_churn(
    project_path: &Path,
    max_commits: usize,
) -> Result<Vec<QualityViolation>> {
    let Some(commits_by_file) = churn_by_file(project_path).await else {
        return Ok(vec![churn_scope_disclosure(project_path)]);
    };

    let mut violations = Vec::new();
    for file in threshold_source_files(project_path) {
        let relative = display_path(project_path, &file);
        let commits = commits_by_file.get(&relative).copied().unwrap_or(0);
        if commits > max_commits {
            violations.push(QualityViolation {
                check_type: "churn".to_string(),
                severity: "error".to_string(),
                file: relative.clone(),
                line: None,
                message: format!(
                    "{relative} was touched by {commits} commits in the last \
                     {CHURN_WINDOW_DAYS} days, over the maximum of {max_commits}"
                ),
                details: Some(ViolationDetails {
                    affected_files: vec![relative],
                    example_code: None,
                    fix_suggestion: Some(format!(
                        "Stabilise the file, or raise the cap with \
                         --max-churn-commits or `[quality] max_churn_commits_90d` \
                         in pmat.toml (currently {max_commits})."
                    )),
                    score_factors: vec![format!(
                        "commits in {CHURN_WINDOW_DAYS} days: {commits} > {max_commits}"
                    )],
                }),
            });
        }
    }
    // Findings are ordered by the walk, which is filesystem order; sort by the
    // measurement so two runs over one tree report the same list.
    violations.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(violations)
}

/// Commits per path in the last 90 days, or `None` when there is no history to
/// read (not a repository, or git is unavailable or failed).
async fn churn_by_file(project_path: &Path) -> Option<std::collections::HashMap<String, usize>> {
    if !project_path.join(".git").exists() {
        return None;
    }
    // ONE subprocess: `%H` marks a commit, every other non-empty line is a path
    // that commit touched.
    let output = run_with_timeout(
        tokio::process::Command::new("git")
            .arg("-C")
            .arg(project_path)
            .args([
                "log",
                &format!("--since={CHURN_WINDOW_DAYS} days ago"),
                "--format=%H",
                "--name-only",
            ]),
        CHURN_TIMEOUT,
    )
    .await?;
    if !output.ok {
        return None;
    }
    Some(parse_churn_log(&output.text))
}

/// Parse `git log --format=%H --name-only` into commits-per-path.
///
/// A path touched twice by one commit (a rename shows both sides) counts once
/// for that commit: the question is "how many commits touched this file".
fn parse_churn_log(log: &str) -> std::collections::HashMap<String, usize> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut seen_in_commit: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for line in log.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if is_commit_hash(line) {
            seen_in_commit.clear();
            continue;
        }
        if seen_in_commit.insert(line) {
            *counts.entry(line.to_string()).or_insert(0) += 1;
        }
    }
    counts
}

/// A `%H` line: 40 hex digits and nothing else. A path can never look like one
/// (it would have to be a 40-character hex filename at the repository root),
/// and the alternative — trusting blank-line framing — breaks on the first
/// commit git formats differently.
fn is_commit_hash(line: &str) -> bool {
    line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit())
}

/// The churn check's reach, stated when there is no history to read.
fn churn_scope_disclosure(project_path: &Path) -> QualityViolation {
    QualityViolation {
        check_type: "scope".to_string(),
        severity: ADVISORY_SEVERITY.to_string(),
        file: project_path.display().to_string(),
        line: None,
        message: format!(
            "churn was NOT measured: {} is not a git repository (or its history could \
             not be read), so no file's commit count over the last {CHURN_WINDOW_DAYS} \
             days is known — this is an unmeasured check, not a quiet one",
            project_path.display()
        ),
        details: None,
    }
}

// ── lint ─────────────────────────────────────────────────────────────────────

/// How long the churn `git log` may take before it is abandoned.
const CHURN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// How long `cargo clippy` may take before it is abandoned. Clippy compiles, so
/// this is minutes rather than seconds; it exists so a gate can never hang.
const LINT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

/// `cargo clippy --all-targets -- -D warnings -A unused-variables`, as ONE
/// finding when the tree does not lint clean.
///
/// §4.4: lint lived only in `pmat verify`, so the MCP `quality_gate` could not
/// report a warning. The flags are not restated here — they are
/// [`crate::cli::verify::CLIPPY_TARGETS`] and [`crate::cli::verify::CLIPPY_LINTS`],
/// the same constants the verify stage passes, so the two surfaces cannot come
/// to ask clippy different questions.
///
/// A tree with no `Cargo.toml` yields NO violations and one advisory `scope`
/// row, for the same reason the churn check does.
///
/// # Errors
/// Never fails on a missing cargo or a timeout; both are disclosed as findings.
pub async fn check_lint(project_path: &Path) -> Result<Vec<QualityViolation>> {
    if !project_path.join("Cargo.toml").is_file() {
        return Ok(vec![lint_scope_disclosure(project_path)]);
    }

    let mut cmd = tokio::process::Command::new(
        std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()),
    );
    cmd.current_dir(project_path)
        .args([
            "clippy",
            crate::cli::verify::CLIPPY_TARGETS,
            "--message-format=json",
            "--",
        ])
        .args(crate::cli::verify::CLIPPY_LINTS);
    scrub_parent_cargo_env(&mut cmd);

    let Some(output) = run_with_timeout(&mut cmd, LINT_TIMEOUT).await else {
        return Ok(vec![lint_not_run(
            project_path,
            &format!(
                "cargo clippy did not finish within {}s, or could not be started",
                LINT_TIMEOUT.as_secs()
            ),
        )]);
    };
    if output.ok {
        return Ok(Vec::new());
    }
    Ok(vec![lint_violation(project_path, &output.text)])
}

/// A nested cargo invocation must not inherit the OUTER build's environment.
///
/// When the gate itself runs under `cargo test`, `CARGO_TARGET_DIR`, `RUSTFLAGS`
/// and the make jobserver in `CARGO_MAKEFLAGS` are all set for a different
/// crate, and the inner build either fails or writes into the outer tree. The
/// measured version of this trap is recorded in this repository's own history:
/// a nested `cargo check` under `cargo test` made a check read 0 where a shell
/// read 195, so the gate manufactured the defect it was hunting.
fn scrub_parent_cargo_env(cmd: &mut tokio::process::Command) {
    for var in [
        "CARGO_TARGET_DIR",
        "CARGO_MAKEFLAGS",
        "MAKEFLAGS",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_PRIMARY_PACKAGE",
        "CARGO_MANIFEST_DIR",
    ] {
        cmd.env_remove(var);
    }
    cmd.env("CARGO_TERM_COLOR", "never");
}

/// The one lint finding, carrying clippy's first diagnostic.
fn lint_violation(project_path: &Path, output: &str) -> QualityViolation {
    let (file, line, detail) = first_clippy_diagnostic(output).unwrap_or_else(|| {
        (
            "project".to_string(),
            None,
            first_error_line(output)
                .unwrap_or_else(|| "cargo clippy exited non-zero".to_string()),
        )
    });
    let file = if file == "project" {
        project_path.display().to_string()
    } else {
        file
    };
    QualityViolation {
        check_type: "lint".to_string(),
        severity: "error".to_string(),
        file: file.clone(),
        line,
        message: format!("cargo clippy reported: {detail}"),
        details: Some(ViolationDetails {
            affected_files: vec![file],
            example_code: None,
            fix_suggestion: Some(
                "Run `cargo clippy --all-targets --fix` (or `pmat verify --fix`), then \
                 re-run the gate."
                    .to_string(),
            ),
            score_factors: vec!["clippy: -D warnings".to_string()],
        }),
    }
}

/// First clippy lint or rustc error in a `--message-format=json` stream, as
/// (file, line, message).
///
/// The same keep-rule `pmat verify`'s parser applies: clippy lints AND rustc
/// errors, because a tree that does not compile does not lint clean either;
/// rustc's "aborting due to N previous errors" is a count, not a cause.
fn first_clippy_diagnostic(json_stream: &str) -> Option<(String, Option<usize>, String)> {
    json_stream.lines().find_map(|line| {
        let value: serde_json::Value = serde_json::from_str(line).ok()?;
        if value.get("reason")?.as_str()? != "compiler-message" {
            return None;
        }
        let message = value.get("message")?;
        let rule = message
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let level = message.get("level")?.as_str().unwrap_or_default();
        let text = message.get("message")?.as_str().unwrap_or_default();
        if text.starts_with("aborting due to") {
            return None;
        }
        if !rule.starts_with("clippy::") && level != "error" {
            return None;
        }
        let span = message
            .get("spans")
            .and_then(serde_json::Value::as_array)
            .and_then(|spans| {
                spans
                    .iter()
                    .find(|s| {
                        s.get("is_primary")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false)
                    })
                    .or_else(|| spans.first())
            });
        let file = span
            .and_then(|s| s.get("file_name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("project")
            .to_string();
        let line = span
            .and_then(|s| s.get("line_start"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|l| usize::try_from(l).ok());
        let named = if rule.is_empty() {
            text.to_string()
        } else {
            format!("{rule}: {text}")
        };
        Some((file, line, named))
    })
}

/// First error-shaped plain-text line, for cargo's own failures ("error: no
/// such command: `clippy`") which never arrive as JSON diagnostics.
fn first_error_line(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("error"))
        .map(str::to_string)
}

/// The lint check's reach, stated when there is nothing for clippy to read.
fn lint_scope_disclosure(project_path: &Path) -> QualityViolation {
    lint_not_run(
        project_path,
        "there is no Cargo.toml here, so cargo clippy has nothing to lint",
    )
}

/// An advisory row saying the lint question was not answered, and why.
///
/// Advisory, not blocking: it describes a limit of this run, not a defect in
/// the tree — the rule `security_scope_disclosure` follows. It is still a row,
/// because `lint_violations: 0` must not mean "clean" and "never asked" at once.
fn lint_not_run(project_path: &Path, why: &str) -> QualityViolation {
    QualityViolation {
        check_type: "scope".to_string(),
        severity: ADVISORY_SEVERITY.to_string(),
        file: project_path.display().to_string(),
        line: None,
        message: format!("lint was NOT measured: {why}"),
        details: None,
    }
}

// ── subprocess plumbing ──────────────────────────────────────────────────────

/// A finished subprocess: its verdict and its combined output.
struct CommandOutput {
    ok: bool,
    text: String,
}

/// Run a command with a wall-clock limit, returning `None` if it could not be
/// started or did not finish in time.
///
/// stderr is CONCATENATED onto stdout rather than discarded: cargo's own
/// failures arrive there, and a harness that throws a subprocess's stderr away
/// reports "it failed" with no cause.
async fn run_with_timeout(
    cmd: &mut tokio::process::Command,
    limit: std::time::Duration,
) -> Option<CommandOutput> {
    let output = tokio::time::timeout(limit, cmd.output()).await.ok()?.ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Some(CommandOutput {
        ok: output.status.success(),
        text: crate::cli::verify::strip_ansi(&text),
    })
}
