use crate::cli::language_analyzer::Language;
use std::time::Duration;

/// How long a single-file `cargo clippy` may run before its child is killed.
///
/// The content being linted is caller-supplied and is compiled in a fresh temp
/// crate with no dependencies, so even a cold run is one rustc invocation over
/// one file — seconds, not minutes. Sixty leaves an order of magnitude of
/// headroom on a loaded box while still bounding the request. What it replaces
/// is far worse than a generous limit: `Command::output()` carried no deadline
/// at all, so content that made rustc spin held the child forever. Because the
/// spawn was a *blocking* one inside an `async fn` awaited from the tokio MCP
/// handler, it also pinned a tokio worker thread for that whole time, and
/// `mcp-http` is in the default feature set — an availability bug, not only a
/// resource one.
const CLIPPY_TIMEOUT: Duration = Duration::from_secs(60);

/// rustfmt over a single file is milliseconds; ten seconds is already
/// pathological. Reachable from AutoFix mode on the same caller-supplied
/// content, and unbounded before this for the same reason clippy was.
const RUSTFMT_TIMEOUT: Duration = Duration::from_secs(10);

/// Names published in `QualityReport::gates_run`. A gate missing from that
/// list did not run, and the corresponding zero in `metrics` is not a
/// measurement of zero.
const GATE_SATD: &str = "satd";
const GATE_COMPLEXITY: &str = "complexity";
const GATE_LINT: &str = "lint";
const GATE_DOCS: &str = "docs";

/// Maximum cyclomatic complexity over every function in the file, including
/// methods on impls/classes.
///
/// Deliberately unfiltered: this is the measurement the quality report
/// publishes as `metrics.max_complexity`, and it must not depend on the
/// threshold the caller happened to configure.
pub(crate) fn max_function_cyclomatic(
    file_metrics: &crate::services::complexity::FileComplexityMetrics,
) -> u16 {
    file_metrics
        .functions
        .iter()
        .map(|f| f.metrics.cyclomatic)
        .chain(
            file_metrics
                .classes
                .iter()
                .flat_map(|c| c.methods.iter().map(|m| m.metrics.cyclomatic)),
        )
        .max()
        .unwrap_or(0)
}

/// Severity for one line of `cargo clippy` output.
///
/// Every lint finding used to be filed as a `Warning`, and `passed` is "all
/// violations are warnings", so content that does not even compile
/// ("error: could not compile … due to 1 previous error") was *accepted* by
/// strict mode while a TODO comment rejected it. rustc's own level prefix
/// decides: `error:`/`error[E0433]:` is an error.
pub(crate) fn lint_line_severity(line: &str) -> ViolationSeverity {
    let trimmed = line.trim_start();
    if trimmed.starts_with("error:") || trimmed.starts_with("error[") {
        ViolationSeverity::Error
    } else {
        ViolationSeverity::Warning
    }
}

/// Stderr fragments that mean **clippy never ran**, as opposed to clippy
/// having run and disliked the code.
///
/// Each is emitted by cargo/rustup *before* any code is compiled.
const CLIPPY_UNAVAILABLE_MARKERS: &[&str] = &[
    // rustup: the shim exists, the component does not (rust:*-slim, and any
    // minimal-profile toolchain):
    //   error: 'cargo-clippy' is not installed for the toolchain '1.95.0-...'
    "is not installed for the toolchain",
    // rustup: the whole toolchain is missing:
    //   error: toolchain '1.95.0-x86_64-unknown-linux-gnu' is not installed
    // Anchored on rustup's closing quote so it cannot match a rustc diagnostic
    // that merely contains the words.
    "' is not installed",
    // cargo: no `cargo-clippy` on PATH at all.
    "no such command:",
    // cargo (pre-1.54 wording).
    "no such subcommand:",
];

/// Why `cargo clippy` could not deliver a verdict — `None` when it did.
///
/// This exists because a tool that did not run was being read as a judgement
/// about the code. `rust:1.95-slim` installs the MINIMAL rustup profile, so
/// `/usr/local/cargo/bin/cargo-clippy` is present as a *shim* while the clippy
/// component is not installed. `cargo clippy` then writes, to stderr, exit 1:
///
/// ```text
/// error: 'cargo-clippy' is not installed for the toolchain '1.95.0-x86_64-unknown-linux-gnu'.
/// help: run `rustup component add clippy` to install it
/// ```
///
/// Both lines begin with a rustc-shaped level prefix, so `lint_line_severity`
/// filed the first as [`ViolationSeverity::Error`], `passed` became false, and
/// `ProxyMode::Strict` answered `Rejected`. The caller cannot tell that
/// rejection apart from "your code does not compile" — and the property test
/// `test_high_quality_code_accepted` duly reported a minimal failing input
/// (`file_path = "a.rs"`, `fn_name = "a"`), i.e. a logic bug that does not
/// exist. Measured in paiml/infra run 33091353601, `clean-room (pmat)` GATE B2:
/// 21024 passed, 9 failed, every failure this string.
///
/// A missing verifier is a NO-GO, never a verdict. Returning the reason here
/// lets the lint stage fail loudly and name the tool.
pub(crate) fn clippy_unavailable_reason(stderr: &str) -> Option<String> {
    stderr.lines().find_map(|line| {
        let trimmed = line.trim();
        // Only cargo's/rustup's own diagnostics, not source spans that might
        // quote one of these phrases inside the code being linted.
        if !(trimmed.starts_with("error:") || trimmed.starts_with("error[")) {
            return None;
        }
        CLIPPY_UNAVAILABLE_MARKERS
            .iter()
            .any(|marker| trimmed.contains(marker))
            .then(|| trimmed.to_string())
    })
}

/// Turn one completed `cargo clippy` run into lint findings, or into a loud
/// error when the run produced no verdict to read.
///
/// Split out from [`QualityProxyService::run_lint_checks`] so the
/// did-the-tool-even-run decision is a pure function over the process output
/// and can be tested against the exact bytes the clean room saw.
pub(crate) fn interpret_clippy_output(
    output: &std::process::Output,
) -> Result<Vec<(usize, String)>> {
    let stderr = String::from_utf8_lossy(&output.stderr);

    if let Some(reason) = clippy_unavailable_reason(&stderr) {
        anyhow::bail!(
            "the quality proxy's lint stage did not run: {reason}\n\
             This is a missing tool, not a finding about the code under review. \
             Install it with `rustup component add clippy`. No lint verdict was \
             produced, so none is reported."
        );
    }

    let mut violations = Vec::new();
    // Warnings are reported on a *successful* run too, so the findings are
    // collected regardless of exit status.
    for line in stderr.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("warning:")
            || trimmed.starts_with("error:")
            || trimmed.starts_with("error[")
        {
            // Extract line number if possible
            let line_num = 1; // Default line number
            let message = line.to_string();
            violations.push((line_num, message));
        }
    }

    Ok(violations)
}

/// The language the quality gates are selected for.
///
/// Reading the raw extension got this wrong twice. `proxy_operation` ended its
/// extension lookup with `.unwrap_or("rs")`, so an EXTENSIONLESS file — a
/// `Makefile`, a `Dockerfile` — was written into a temp crate's `src/lib.rs`
/// and handed to `cargo clippy`, which rejected it with parse errors about
/// content that was never Rust. And the guard it fed, `extension != "rs"`, is
/// case-sensitive, so a `.RS` file took the non-Rust path and was passed with
/// no gate running at all. `Language::from_path` is itself case-sensitive
/// (`src/cli/language_analyzer/types.rs` matches lowercase literals), which is
/// why the extension is lower-cased here rather than the path handed over
/// directly; a missing extension is `Unknown`, never Rust.
pub(crate) fn proxy_language(file_path: &str) -> Language {
    match Path::new(file_path).extension().and_then(|e| e.to_str()) {
        Some(ext) => Language::from_path(Path::new(&format!("f.{}", ext.to_ascii_lowercase()))),
        None => Language::Unknown,
    }
}

/// The label a report publishes for the language it judged.
///
/// Derived from `Debug` on purpose: a variant added to `Language` cannot then
/// silently acquire the label `"unknown"` here while being analysed as
/// something else.
pub(crate) fn language_label(language: Language) -> String {
    format!("{language:?}").to_lowercase()
}

/// `passed` is, and has always been, "no violation is worse than a warning".
fn all_warnings(violations: &[QualityViolation]) -> bool {
    violations
        .iter()
        .all(|v| matches!(v.severity, ViolationSeverity::Warning))
}

/// What `analyze_content` measured — and, in `gates_run`, what it did not.
pub(crate) struct AnalysisOutcome {
    metrics: QualityMetrics,
    passed: bool,
    language: String,
    gates_run: Vec<String>,
    violations: Vec<QualityViolation>,
}

impl QualityProxyService {
    /// Judge `content` against the gates that apply to its language.
    ///
    /// Everything whose extension was not the literal lowercase `rs` used to
    /// return from here immediately with `passed: true`, all-zero metrics and
    /// an empty violation list. A Python payload carrying a debt marker, a
    /// second debt marker and five levels of nesting came back
    /// `{"status":"accepted","quality_report":{"passed":true,"metrics":
    /// {"max_complexity":0,"satd_count":0,...},"violations":[]}}` in strict
    /// mode — and strict is the DEFAULT mode, not an edge case.
    ///
    /// The sharp part is that pmat already analyses those languages in the same
    /// binary: `pmat analyze satd` finds both markers in that exact file and
    /// `pmat analyze complexity` measures it at cyclomatic 6 / cognitive 20. So
    /// this was one tool declining to call the polyglot analysis its sibling
    /// tools use, and then publishing the absence as a measurement of zero —
    /// a surface contradiction, strictly worse than a missing capability.
    ///
    /// The gates are therefore selected per language rather than skipped
    /// wholesale. SATD runs for every language (`CommentScanner::for_path`
    /// already knows the `//`, `#` and `<!-- -->` comment families, so it was
    /// language-aware all along — it was simply never reached). Complexity runs
    /// through the analyzer that is canonical for the language. Only the two
    /// genuinely Rust-only stages stay behind a Rust check: `cargo clippy`,
    /// which on Python emits `error:` lines that `lint_line_severity` maps to
    /// Error and so would reject every non-Rust file for the wrong reason, and
    /// the documentation scan, which only matches `pub fn`/`pub struct`/
    /// `pub enum`. Whatever did not run is named by its absence from
    /// `gates_run`, so no zero here is readable as a measurement.
    async fn analyze_content(
        &self,
        content: &str,
        file_path: &str,
        language: Language,
        config: &QualityConfig,
    ) -> Result<AnalysisOutcome> {
        let is_rust = language == Language::Rust;
        let mut gates_run = vec![GATE_SATD.to_string()];

        // SATD first, and for every language. The block itself is unchanged;
        // only its position moved, out from under the guard that skipped it.
        let (satd_count, mut violations) = self.satd_stage(content, file_path, config)?;

        if language == Language::Unknown {
            // A `.rst`, `.txt` or `.proto` file: pmat has no complexity
            // analyzer for it and clippy would be nonsense, but its comments
            // were still scanned. `passed` therefore follows from the SATD
            // result alone, and `gates_run` names the one gate that ran — never
            // `passed: true` with nothing claimed behind it.
            debug!(
                "no complexity analyzer for {}; SATD is the only gate that ran",
                file_path
            );
            return Ok(AnalysisOutcome {
                metrics: QualityMetrics {
                    max_complexity: 0,
                    satd_count,
                    lint_violations: 0,
                    coverage_percentage: None,
                },
                passed: all_warnings(&violations),
                language: language_label(language),
                gates_run,
                violations,
            });
        }

        let (measured, complexity_violations) = self
            .complexity_stage(content, file_path, language, config)
            .await;
        violations.extend(complexity_violations);
        if measured.is_some() {
            gates_run.push(GATE_COMPLEXITY.to_string());
        }
        // `None` is "not measured", and it is disclosed by the missing gate
        // above; the 0 published beside it is a placeholder, not a reading.
        let max_complexity = measured.unwrap_or(0);

        // Run lint checks using cargo clippy directly — Rust only, see above.
        //
        // A lint stage that did not run is NOT "zero lint violations". This arm
        // used to `warn!` and substitute 0, publishing `metrics.lint_violations
        // = 0` — a measurement nobody took — into the same QualityReport that
        // callers read as evidence. The failure is propagated instead: the
        // report either carries a lint measurement or it does not exist. That
        // covers both halves of the split `run_lint_checks` makes — clippy
        // absent, and clippy killed on its deadline — neither of which is a
        // statement about the content being judged.
        let lint_violations = if is_rust {
            let violations_found = self
                .run_lint_checks(content)
                .await
                .context("quality proxy lint stage failed; no lint measurement was produced")?;
            gates_run.push(GATE_LINT.to_string());
            for (line, message) in &violations_found {
                violations.push(QualityViolation {
                    violation_type: ViolationType::Lint,
                    severity: lint_line_severity(message),
                    location: format!("{file_path}:{line}"),
                    message: message.clone(),
                    suggestion: Some("Fix lint issue".to_string()),
                });
            }
            violations_found.len()
        } else {
            0
        };

        // Check documentation
        if is_rust && config.require_docs {
            gates_run.push(GATE_DOCS.to_string());
            let doc_violations = self.check_documentation(content, file_path);
            violations.extend(doc_violations);
        }

        Ok(AnalysisOutcome {
            metrics: QualityMetrics {
                max_complexity,
                satd_count,
                lint_violations,
                coverage_percentage: None,
            },
            passed: all_warnings(&violations),
            language: language_label(language),
            gates_run,
            violations,
        })
    }

    /// Comment-marker debt, for every language.
    ///
    /// Lifted verbatim out of the Rust-only section of `analyze_content`; the
    /// only thing that changed about it is that it is now reached at all for a
    /// `.py`, `.sh` or `.md` file.
    fn satd_stage(
        &self,
        content: &str,
        file_path: &str,
        config: &QualityConfig,
    ) -> Result<(usize, Vec<QualityViolation>)> {
        let satd_instances = self
            .satd_detector
            .extract_from_content(content, Path::new(file_path))?;
        let satd_count = satd_instances.len();

        let mut violations = Vec::new();
        if !config.allow_satd && satd_count > 0 {
            for instance in &satd_instances {
                violations.push(QualityViolation {
                    violation_type: ViolationType::Satd,
                    severity: ViolationSeverity::Error,
                    location: format!("{}:{}", file_path, instance.line),
                    message: format!("SATD detected: {}", instance.text),
                    suggestion: Some(
                        "Remove TODO/FIXME comments and implement the functionality".to_string(),
                    ),
                });
            }
        }

        Ok((satd_count, violations))
    }

    /// Measure the file's worst function — and say so when it could not be.
    ///
    /// The failure arm used to be `Err(e) => { warn!(...); 0 }`. Content syn
    /// could not parse was published as `max_complexity: 0` with no violation
    /// recorded, so it never reached the `passed` computation and a consumer
    /// could not tell trivial code from code that was never read. The failure
    /// is now an Error violation — `passed` is "every violation is a warning",
    /// so a single Error correctly flips strict mode to Rejected — and the
    /// `None` returned here keeps `complexity` out of `gates_run`, so the 0 the
    /// caller publishes can never be read as a measurement that was taken.
    async fn complexity_stage(
        &self,
        content: &str,
        file_path: &str,
        language: Language,
        config: &QualityConfig,
    ) -> (Option<u32>, Vec<QualityViolation>) {
        let mut violations = Vec::new();

        let file_metrics = match self.measure_complexity(content, file_path, language).await {
            Ok(file_metrics) => file_metrics,
            Err(e) => {
                warn!("Failed to analyze complexity: {:#}", e);
                violations.push(QualityViolation {
                    violation_type: ViolationType::Complexity,
                    severity: ViolationSeverity::Error,
                    location: file_path.to_string(),
                    message: format!("complexity was not measured: {e:#}"),
                    suggestion: Some(
                        "The content could not be parsed; fix the syntax error so it can be \
                         analysed"
                            .to_string(),
                    ),
                });
                return (None, violations);
            }
        };

        // `report.hotspots` is threshold-filtered, so reading the maximum
        // from it published a 0 meaning "nothing exceeded the threshold"
        // as if it were the measured maximum: the same content reported
        // max_complexity 0 under the default threshold and 9 under
        // max_complexity=1. The measurement comes from the unfiltered
        // function list; the threshold only decides the violation.
        let max_comp = u32::from(max_function_cyclomatic(&file_metrics));

        // The result is already FileComplexityMetrics, use it directly
        let report = aggregate_results_with_thresholds(
            vec![file_metrics],
            Some(config.max_complexity as u16),
            Some(config.max_complexity as u16 + 5),
        );

        if max_comp > config.max_complexity {
            if let Some(hotspot) = report.hotspots.first() {
                violations.push(QualityViolation {
                    violation_type: ViolationType::Complexity,
                    severity: ViolationSeverity::Error,
                    location: format!("{}:{}", file_path, hotspot.line),
                    message: format!(
                        "Function '{}' complexity {} exceeds maximum {}",
                        hotspot.function.as_ref().unwrap_or(&"unknown".to_string()),
                        hotspot.complexity,
                        config.max_complexity
                    ),
                    suggestion: Some(
                        "Consider splitting this function into smaller functions".to_string(),
                    ),
                });
            }
        }

        (Some(max_comp), violations)
    }

    /// Per-function complexity from the analyzer that is canonical for the
    /// language.
    ///
    /// Rust keeps the syn AST path deliberately: `src/services/complexity/
    /// uncached.rs` records that the heuristic counter and the AST analyzer
    /// disagree on the same function — 10/18 against 6/9 — and that the AST is
    /// the number `pmat analyze complexity` publishes, so routing Rust through
    /// heuristics here would reintroduce exactly the tool-versus-tool
    /// contradiction this change exists to remove. Every other language has no
    /// AST complexity analyzer in this build, so the heuristic counter is not a
    /// degradation there: it is the only analyzer that has ever existed for it,
    /// and it is the one the CLI uses too.
    ///
    /// The `ast_python_compat` / `ast_c_compat` / `ast_cpp_compat` shims are
    /// deliberately NOT wired in. They emit `cyclomatic: 1` placeholders under
    /// `function_{i}` names, which would trade a silent zero for a fabricated
    /// number — worse than the defect.
    async fn measure_complexity(
        &self,
        content: &str,
        file_path: &str,
        language: Language,
    ) -> Result<crate::services::complexity::FileComplexityMetrics> {
        if language == Language::Rust {
            let temp_file = self.create_temp_file(content, "rs")?;
            return analyze_rust_file_with_complexity(temp_file.path())
                .await
                .context("the content could not be parsed as Rust");
        }

        crate::cli::language_analyzer::analyze_with_heuristics(
            Path::new(file_path),
            content,
            language,
        )
    }

    fn check_documentation(&self, content: &str, file_path: &str) -> Vec<QualityViolation> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub fn")
                || trimmed.starts_with("pub struct")
                || trimmed.starts_with("pub enum")
            {
                // Check previous lines for documentation
                let has_doc = if line_num > 0 {
                    // Check up to 5 lines before for doc comments
                    let start = line_num.saturating_sub(5);
                    lines[start..line_num]
                        .iter()
                        .any(|l| l.trim().starts_with("///"))
                } else {
                    false
                };

                if !has_doc {
                    violations.push(QualityViolation {
                        violation_type: ViolationType::Docs,
                        severity: ViolationSeverity::Warning,
                        location: format!("{}:{}", file_path, line_num + 1),
                        message: "Public item missing documentation".to_string(),
                        suggestion: Some("Add /// documentation comment".to_string()),
                    });
                }
            }
        }

        violations
    }

    fn create_temp_file(&self, content: &str, extension: &str) -> Result<tempfile::NamedTempFile> {
        use std::io::Write;

        let mut temp_file = tempfile::Builder::new()
            .suffix(&format!(".{extension}"))
            .tempfile()?;

        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;

        Ok(temp_file)
    }

    async fn run_lint_checks(&self, content: &str) -> Result<Vec<(usize, String)>> {
        use std::fs;
        use std::io::Write;
        use std::process::Command;

        // Create a temporary Rust project
        let temp_dir = tempfile::TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)?;

        let lib_path = src_dir.join("lib.rs");
        let mut lib_file = fs::File::create(&lib_path)?;
        lib_file.write_all(content.as_bytes())?;
        lib_file.flush()?;

        let cargo_toml = r#"[package]
name = "temp_quality_check"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let cargo_path = temp_dir.path().join("Cargo.toml");
        let mut cargo_file = fs::File::create(&cargo_path)?;
        cargo_file.write_all(cargo_toml.as_bytes())?;
        cargo_file.flush()?;

        // Run cargo clippy.
        //
        // `-D warnings` used to be passed here, which relabelled every lint as
        // "error:" and made rustc's own level prefix useless for telling a
        // style lint apart from content that does not compile. Levels are left
        // as rustc reports them so `lint_line_severity` can be trusted; the
        // caller decides what fails the gate.
        //
        // The spawn is bounded and taken off the async task. It was a blocking
        // `Command::output()` with no deadline, no memory bound and no kill
        // path, run on content the caller supplies verbatim, from an `async fn`
        // awaited by the tokio MCP handler: one request could pin a worker
        // thread for as long as rustc chose to take, and nothing caps
        // concurrent requests. `run_with_timeout` kills the child on the
        // deadline and drains both pipes on their own threads — its doc comment
        // explains why polling `try_wait` while the child fills a pipe buffer
        // is the classic way a hand-rolled timeout makes hangs *more* likely —
        // and `spawn_blocking` keeps the wait off the runtime, so the same
        // change fixes the DoS and the tokio-worker starvation.
        //
        // The did-the-tool-even-run question is answered here now rather than
        // deferred. #1088 landed `clippy_unavailable_reason` and
        // `interpret_clippy_output` in this file, so the case a bound could
        // never have caught — a `cargo` that spawns perfectly and reports a
        // missing `clippy` subcommand, or a toolchain installed on rustup's
        // minimal profile, which is what `rust:1.95-slim` gives you — leaves
        // this function as an `Err` naming the tool, instead of as `error:`
        // lines filed against the caller's content. A `cargo` that is not on
        // PATH at all never reaches that classifier and is named in the spawn
        // arm below.
        //
        // What a timeout does to the verdict: nothing, because there is no
        // longer a verdict to distort. Every outcome above, the deadline
        // included, leaves an `Err` here, and that `Err` is no longer
        // swallowed — the lint stage in `analyze_content` propagates it with
        // `?` rather than logging and substituting 0, and `proxy_operation`
        // propagates it again from its own `analyze_content` call, so no
        // `ProxyResponse` and no `QualityReport` are constructed and both MCP
        // handlers surface an error. An earlier revision of this comment said
        // content that kept clippy past the deadline was "still accepted, with
        // an unmeasured zero beside it". That described the log-and-substitute
        // arm #1088 deleted; it is false of this code. Nor is the answer
        // `Rejected`: a killed child, a panicked worker thread and a temp-file
        // failure are not statements about the content, and the absence of a
        // report is the only answer that invents neither a pass nor a finding.
        let project_dir = temp_dir.path().to_path_buf();
        let spawned = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new("cargo");
            cmd.arg("clippy").current_dir(&project_dir);
            crate::cli::handlers::work_falsification::deny_refresh::run_with_timeout(
                &mut cmd,
                CLIPPY_TIMEOUT,
            )
        })
        .await
        .context("the cargo clippy worker thread panicked")?;

        let output = match spawned {
            Ok(Some(output)) => output,
            // The deadline passed and the child was killed, so whatever it had
            // written so far is a partial run, not a verdict to classify.
            Ok(None) => anyhow::bail!(
                "the quality proxy's lint stage did not run: `cargo clippy` did not finish \
                 within {}s and was killed. No lint verdict was produced, so none is \
                 reported.",
                CLIPPY_TIMEOUT.as_secs()
            ),
            // `cargo` itself is not on PATH. Spawning failed, so there is no
            // stderr to classify — say so here rather than let a caller read
            // an io error as a lint finding.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => anyhow::bail!(
                "the quality proxy's lint stage did not run: `cargo` is not on PATH ({e}). \
                 This is a missing tool, not a finding about the code under review."
            ),
            Err(e) => {
                return Err(e).context("failed to spawn `cargo clippy` for the quality proxy")
            }
        };

        interpret_clippy_output(&output)
    }

    async fn format_rust_code(&self, content: &str) -> Result<String> {
        use std::process::Command;

        let temp_file = self.create_temp_file(content, "rs")?;
        let target = temp_file.path().to_path_buf();

        // Bounded and off-runtime for the same reason as the clippy spawn: this
        // is the second unbounded blocking child in this file, reachable from
        // AutoFix mode on caller-supplied content, and AutoFix runs the whole
        // analysis twice per request. `temp_file` is deliberately kept alive in
        // this scope — only its path is moved into the worker — because rustfmt
        // formats it in place and it is read back below.
        let output = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new("rustfmt");
            cmd.arg("--edition").arg("2021").arg(&target);
            crate::cli::handlers::work_falsification::deny_refresh::run_with_timeout(
                &mut cmd,
                RUSTFMT_TIMEOUT,
            )
        })
        .await
        .context("the rustfmt worker thread panicked")?
        .context("rustfmt could not be spawned")?
        .with_context(|| {
            format!(
                "rustfmt did not finish within {}s",
                RUSTFMT_TIMEOUT.as_secs()
            )
        })?;

        if output.status.success() {
            std::fs::read_to_string(temp_file.path()).context("Failed to read formatted file")
        } else {
            Err(anyhow::anyhow!(
                "rustfmt failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

#[cfg(test)]
mod analysis_measurement_tests {
    //! Regressions for the fabricated values in the quality_proxy report: a
    //! threshold-filtered `max_complexity`, lint findings that were all filed
    //! as warnings so non-compiling content passed strict mode, a gate
    //! selection that read a raw extension (case-sensitively, and defaulting a
    //! missing one to Rust), and a parse failure published as `0`.
    use super::*;
    use crate::services::complexity::{
        ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
    };

    fn func(name: &str, cyclomatic: u16) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start: 1,
            line_end: 10,
            metrics: ComplexityMetrics {
                cyclomatic,
                cognitive: cyclomatic,
                nesting_max: 1,
                lines: 10,
                halstead: None,
            },
        }
    }

    fn file_with(functions: Vec<FunctionComplexity>) -> FileComplexityMetrics {
        FileComplexityMetrics {
            path: "qpa.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 0,
                cognitive: 0,
                nesting_max: 0,
                lines: 0,
                halstead: None,
            },
            functions,
            classes: vec![],
        }
    }

    #[test]
    fn test_max_complexity_is_measured_not_threshold_filtered() {
        // The reported maximum must not depend on the configured threshold:
        // under the default 20 the hotspot list is empty, and reading the max
        // from it published 0 for a function measured at 9.
        let metrics = file_with(vec![func("nasty", 9), func("tidy", 2)]);

        let report = aggregate_results_with_thresholds(vec![metrics.clone()], Some(20), Some(25));
        assert!(
            report.hotspots.iter().all(|h| h.complexity < 9),
            "the 9-complexity function is filtered out at threshold 20; \
             that filtered list must not be the source of the measurement"
        );

        assert_eq!(max_function_cyclomatic(&metrics), 9);
    }

    #[test]
    fn test_max_complexity_includes_methods() {
        let mut metrics = file_with(vec![func("plain", 3)]);
        metrics.classes.push(crate::services::complexity::ClassComplexity {
            name: "Impl".to_string(),
            line_start: 1,
            line_end: 20,
            metrics: ComplexityMetrics {
                cyclomatic: 0,
                cognitive: 0,
                nesting_max: 0,
                lines: 0,
                halstead: None,
            },
            methods: vec![func("method", 12)],
        });
        assert_eq!(max_function_cyclomatic(&metrics), 12);
    }

    #[test]
    fn test_max_complexity_of_empty_file_is_zero() {
        assert_eq!(max_function_cyclomatic(&file_with(vec![])), 0);
    }

    #[test]
    fn test_compile_errors_are_errors_not_warnings() {
        // These two lines are exactly what the shipped binary reported (as
        // severity "warning") while ACCEPTING "this is not rust at all !!!".
        for line in [
            "error: expected one of `!` or `::`, found `is`",
            "error: could not compile `temp_quality_check` (lib) due to 1 previous error",
            "error[E0433]: failed to resolve: use of undeclared crate or module `foo`",
        ] {
            assert!(
                matches!(lint_line_severity(line), ViolationSeverity::Error),
                "{line} must fail a strict gate"
            );
        }
    }

    #[test]
    fn test_style_lints_stay_warnings() {
        for line in [
            "warning: function `simple` is never used",
            "warning: `temp_quality_check` (lib) generated 1 warning",
        ] {
            assert!(matches!(
                lint_line_severity(line),
                ViolationSeverity::Warning
            ));
        }
    }

    /// The gate selection used to be `extension != "rs"`, which is
    /// case-sensitive: a `.RS` file took the non-Rust path and was passed
    /// without a single gate running. `Language::from_path` is case-sensitive
    /// too, so lower-casing has to happen here.
    #[test]
    fn test_proxy_language_ignores_extension_case() {
        assert_eq!(proxy_language("Shouty.RS"), Language::Rust);
        assert_eq!(proxy_language("Shouty.rs"), Language::Rust);
        assert_eq!(proxy_language("script.PY"), Language::Python);
    }

    /// An extensionless file is unknown, never Rust. `.unwrap_or("rs")` meant
    /// a `Makefile` was compiled as Rust in a temp crate and rejected with
    /// parse errors about content that was never Rust.
    #[test]
    fn test_proxy_language_of_extensionless_file_is_unknown() {
        assert_eq!(proxy_language("Makefile"), Language::Unknown);
        assert_eq!(proxy_language("Dockerfile"), Language::Unknown);
        assert_eq!(proxy_language("docs/notes.txt"), Language::Unknown);
    }

    #[test]
    fn test_language_label_is_lowercase_debug() {
        assert_eq!(language_label(Language::Rust), "rust");
        assert_eq!(language_label(Language::Python), "python");
        assert_eq!(language_label(Language::Bash), "bash");
        assert_eq!(language_label(Language::Unknown), "unknown");
    }

    /// The complexity stage must publish its own failure rather than a zero.
    /// `None` is what keeps `complexity` out of `gates_run`, and the Error
    /// violation is what keeps the content out of a strict-mode pass.
    #[tokio::test]
    async fn test_complexity_stage_records_a_parse_failure() {
        let service = QualityProxyService::new();
        let config = QualityConfig::default();

        let (measured, violations) = service
            .complexity_stage("this is not rust at all !!!", "broken.rs", Language::Rust, &config)
            .await;

        assert!(measured.is_none(), "a parse failure is not a 0: {measured:?}");
        assert_eq!(violations.len(), 1, "{violations:?}");
        assert!(matches!(violations[0].severity, ViolationSeverity::Error));
        assert!(
            violations[0].message.contains("not measured"),
            "{}",
            violations[0].message
        );
        assert!(!all_warnings(&violations));
    }

    /// Python complexity comes from the heuristic analyzer the CLI uses, not
    /// from a zero and not from the `ast_python_compat` shim's `cyclomatic: 1`
    /// placeholders.
    #[tokio::test]
    async fn test_complexity_stage_measures_python() {
        const NESTED_PY: &str = r#"def nested(a, b, c):
    if a:
        if b:
            if c:
                return 1
    return 0
"#;

        let service = QualityProxyService::new();
        let config = QualityConfig::default();

        let (measured, violations) = service
            .complexity_stage(NESTED_PY, "nested.py", Language::Python, &config)
            .await;

        let measured = measured.expect("Python complexity is measurable, not skipped");
        assert!(
            measured > 1,
            "three nested branches are not complexity {measured}"
        );
        assert!(violations.is_empty(), "{violations:?}");
    }
}

#[cfg(test)]
mod lint_stage_availability_tests {
    //! A missing tool must never be reported as a verdict about the code.
    //!
    //! These are not synthetic strings. `MISSING_COMPONENT_STDERR` is what
    //! `cargo clippy` printed inside the `rust:1.95-slim` clean-room container
    //! in paiml/infra run 33091353601, `clean-room (pmat)` GATE B2, where it
    //! cost nine tests — including a proptest that reported a "minimal failing
    //! input" for a logic bug that does not exist.
    use super::*;

    /// Verbatim from the clean-room job log (2026-08-27T17:07:27Z).
    const MISSING_COMPONENT_LINE: &str =
        "error: 'cargo-clippy' is not installed for the toolchain '1.95.0-x86_64-unknown-linux-gnu'.";
    const MISSING_COMPONENT_STDERR: &str = concat!(
        "error: 'cargo-clippy' is not installed for the toolchain '1.95.0-x86_64-unknown-linux-gnu'.\n",
        "help: run `rustup component add clippy` to install it\n",
    );

    /// What clippy prints when it HAS run and found something.
    const REAL_FINDINGS_STDERR: &str = "\
    Checking temp_quality_check v0.1.0 (/tmp/.tmpXYZ)
warning: function `simple` is never used
 --> src/lib.rs:1:4
warning: `temp_quality_check` (lib) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
";

    /// What clippy prints when it HAS run and the content does not compile.
    const REAL_COMPILE_ERROR_STDERR: &str = "\
    Checking temp_quality_check v0.1.0 (/tmp/.tmpXYZ)
error: expected one of `!` or `::`, found `is`
 --> src/lib.rs:1:6
error: could not compile `temp_quality_check` (lib) due to 1 previous error
";

    fn output(stderr: &str) -> std::process::Output {
        std::process::Output {
            // Exit status is deliberately not consulted: the missing-component
            // run and a genuine compile error both exit non-zero.
            status: Default::default(),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    /// THE regression. Falsification first: the same bytes, run through the
    /// severity classifier the proxy actually uses, are an Error — which is how
    /// "clippy is not installed" became `ProxyStatus::Rejected`. The guard is
    /// therefore load-bearing, not decoration.
    #[test]
    fn a_missing_clippy_component_is_not_a_code_verdict() {
        assert!(
            MISSING_COMPONENT_STDERR.starts_with(MISSING_COMPONENT_LINE),
            "the two constants must describe the same captured stderr"
        );
        assert!(
            matches!(
                lint_line_severity(MISSING_COMPONENT_LINE),
                ViolationSeverity::Error
            ),
            "precondition: this line is what the proxy used to file as an Error \
             about the code under review"
        );

        let reason = clippy_unavailable_reason(MISSING_COMPONENT_STDERR)
            .expect("a toolchain without the clippy component ran nothing");
        assert!(reason.contains("cargo-clippy"), "{reason}");

        let err = interpret_clippy_output(&output(MISSING_COMPONENT_STDERR))
            .expect_err("no verdict was produced, so none may be returned");
        let msg = err.to_string();
        assert!(msg.contains("did not run"), "{msg}");
        assert!(
            msg.contains("rustup component add clippy"),
            "the failure has to name the tool and how to get it; got: {msg}"
        );
    }

    #[test]
    fn a_missing_toolchain_is_not_a_code_verdict() {
        let stderr = "error: toolchain '1.95.0-x86_64-unknown-linux-gnu' is not installed\n";
        assert!(clippy_unavailable_reason(stderr).is_some(), "{stderr}");
        assert!(interpret_clippy_output(&output(stderr)).is_err());
    }

    #[test]
    fn a_cargo_without_the_clippy_subcommand_is_not_a_code_verdict() {
        for stderr in [
            "error: no such command: `clippy`\n",
            "error: no such subcommand: `clippy`\n",
        ] {
            assert!(clippy_unavailable_reason(stderr).is_some(), "{stderr}");
            assert!(
                interpret_clippy_output(&output(stderr)).is_err(),
                "{stderr}"
            );
        }
    }

    /// The counter-test. A guard that refuses every run would hide real lint
    /// findings, which is the same defect pointed the other way.
    #[test]
    fn a_clippy_run_that_actually_happened_still_reports_its_findings() {
        assert!(clippy_unavailable_reason(REAL_FINDINGS_STDERR).is_none());
        let found = interpret_clippy_output(&output(REAL_FINDINGS_STDERR))
            .expect("clippy ran; its findings are a verdict");
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found
            .iter()
            .all(|(_, m)| m.trim_start().starts_with("warning:")));
    }

    /// Content that does not compile is a verdict about the content, and must
    /// keep reaching strict mode as an Error.
    #[test]
    fn a_compile_error_is_still_a_verdict_about_the_code() {
        assert!(clippy_unavailable_reason(REAL_COMPILE_ERROR_STDERR).is_none());
        let found = interpret_clippy_output(&output(REAL_COMPILE_ERROR_STDERR))
            .expect("clippy ran; a compile error is its answer");
        assert!(
            found
                .iter()
                .any(|(_, m)| matches!(lint_line_severity(m), ViolationSeverity::Error)),
            "{found:?}"
        );
    }

    /// The markers are matched only on cargo's own diagnostic lines, so a
    /// source span that happens to quote one is still a finding.
    #[test]
    fn a_source_span_quoting_the_marker_is_not_mistaken_for_a_missing_tool() {
        let stderr = "\
warning: unused variable: `x`
 --> src/lib.rs:2:9
  |
2 |     let x = \"no such command: nope\";
";
        assert!(clippy_unavailable_reason(stderr).is_none(), "{stderr}");
    }
}
