// clippy_file_analysis.rs - File resolution, SLOC counting, and workspace detection
// Included by clippy.rs via include!()

/// Run clippy for the single-file mode.
///
/// Delegates to [`execute_clippy_command`] so there is exactly ONE place that
/// builds the cargo argv. Two builders is how the `-p` path ended up without the
/// `--` separator while the `--file` path had it (#679).
async fn run_clippy_command(
    project_path: &Path,
    clippy_flags: &str,
) -> Result<std::process::Output> {
    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();
    execute_clippy_command(project_path, &flags).await
}

fn resolve_absolute_path(project_path: &Path, file_path: &Path) -> PathBuf {
    if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    }
}

fn create_single_file_result(
    file_path: &Path,
    file_violations: Vec<ViolationDetail>,
    all_violations: Vec<ViolationDetail>,
    severity_dist: SeverityDistribution,
    sloc: usize,
) -> Result<LintHotspotResult> {
    // UNIT FIX: this used to be `(violations / sloc) * 100.0` while the project
    // path used `violations / sloc`, so the SAME command reported the SAME file
    // at two densities 100x apart depending on whether `--file` was given
    // (observed: 52.17 via --file vs 0.5217 via -p). Both paths now report
    // violations per SLOC, which is what `check_quality_gates` compares against.
    let total_violations = file_violations.len();
    let defect_density = calculate_defect_density(total_violations, sloc);

    let hotspot = LintHotspot {
        file: file_path.to_path_buf(),
        defect_density,
        total_violations,
        sloc,
        severity_distribution: severity_dist,
        top_lints: count_top_lints(&file_violations),
        detailed_violations: file_violations,
    };

    let mut summary_by_file = HashMap::new();
    summary_by_file.insert(
        file_path.to_path_buf(),
        FileSummary {
            total_violations,
            errors: hotspot.severity_distribution.error,
            warnings: hotspot.severity_distribution.warning,
            sloc,
            defect_density,
        },
    );

    Ok(LintHotspotResult {
        hotspot,
        all_violations,
        summary_by_file,
        total_project_violations: total_violations,
        enforcement: None,
        refactor_chain: None,
        // Placeholder: `build_final_result` unconditionally replaces this with
        // `check_quality_gates(&hotspot, params.max_density)`. The previous
        // value here hardcoded `defect_density <= 5.0`, which had nothing to do
        // with the user's `--max-density`.
        quality_gate: QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        },
    })
}

/// Count top lint types from violations
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "lint_valid")]
pub(crate) fn count_top_lints(violations: &[ViolationDetail]) -> Vec<(String, usize)> {
    let mut lint_counts: HashMap<String, usize> = HashMap::new();

    for violation in violations {
        *lint_counts.entry(violation.lint_name.clone()).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = lint_counts.into_iter().collect();
    // DETERMINISM: sorting on the count alone left equal-count lints in
    // `HashMap` order, so the rendered "Top Violations" list changed between
    // runs on unchanged input. Ties break on the lint name.
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    counts.truncate(10); // Top 10 lints
    counts
}

/// Count source lines in a file
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn count_source_lines(project_path: &Path, file_path: &Path) -> Result<usize> {
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    let content = tokio::fs::read_to_string(&full_path).await?;
    let non_empty_lines = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count();

    Ok(non_empty_lines.max(1)) // At least 1 to avoid division by zero
}

/// Build the cargo argv used for every lint-hotspot measurement and run it.
///
/// #679 FIX: `flags` are rustc/clippy lint flags and MUST follow the `--`
/// separator. The old project-path builder appended them directly:
///   cargo clippy --message-format=json -W warnings -W clippy::pedantic …
/// cargo rejected that argv outright ("error: unexpected argument '-W' found"),
/// exited 1 with an EMPTY stdout, and the caller then reported the project as
/// clean. Observed wrong value: 0 violations on every input, including a
/// fixture with 20 real findings and pmat's own 3800-file tree.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn execute_clippy_command(
    project_path: &Path,
    flags: &[&str],
) -> Result<std::process::Output> {
    let argv = build_clippy_argv(flags);
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(&argv)
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    cmd.output()
        .await
        .context("failed to spawn `cargo clippy` (is cargo on PATH?)")
}

/// Build the cargo argv. Kept pure so the `--` separator is under test — its
/// absence is what made `analyze lint-hotspot -p <path>` report every project
/// as clean (#679).
pub(crate) fn build_clippy_argv(flags: &[&str]) -> Vec<String> {
    let mut argv = vec![
        "clippy".to_string(),
        "--all-targets".to_string(),
        "--message-format=json".to_string(),
    ];
    if !flags.is_empty() {
        argv.push("--".to_string());
        argv.extend(flags.iter().map(|f| (*f).to_string()));
    }
    argv
}

/// Verify that clippy actually produced a measurement.
///
/// #679 FIX: this used to `Ok(())` unconditionally and only whisper to stderr
/// under `LINT_HOTSPOT_DEBUG`, so a cargo run that never linted anything was
/// rendered to the user as "project is clean" with rc=0. A clean bill of health
/// that was never established is worse than no linter at all, so an unusable
/// run is now a hard error.
///
/// `cargo --message-format=json` always terminates its stream with a
/// `{"reason":"build-finished", …}` record. Its absence means cargo never got
/// as far as building, which is exactly what the malformed argv produced.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn check_clippy_output(output: &std::process::Output) -> Result<()> {
    if clippy_stream_finished(&output.stdout) {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let excerpt: String = stderr.lines().take(10).collect::<Vec<_>>().join("\n");
    anyhow::bail!(
        "`cargo clippy` produced no build-finished record (exit status {:?}); \
         the lint measurement did NOT run, so no result can be reported.\n\
         cargo stderr:\n{}",
        output.status.code(),
        if excerpt.is_empty() {
            "<empty>"
        } else {
            excerpt.as_str()
        }
    )
}

/// True when cargo's JSON stream contains the terminating `build-finished`
/// record. Used as the "the measurement happened" predicate.
pub(crate) fn clippy_stream_finished(stdout: &[u8]) -> bool {
    let reader = BufReader::new(stdout);
    for line in std::io::BufRead::lines(reader) {
        let Ok(line) = line else { continue };
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            if value.get("reason").and_then(serde_json::Value::as_str) == Some("build-finished") {
                return true;
            }
        }
    }
    false
}

/// Calculate SLOC for each file in metrics
async fn calculate_sloc_for_files(
    file_metrics: &mut HashMap<PathBuf, FileMetrics>,
    project_path: &Path,
    workspace_root: Option<&PathBuf>,
) -> Result<()> {
    let manifest_dir = find_manifest_dir(project_path);
    let mut unresolved = Vec::new();

    for (file_path, metrics) in file_metrics.iter_mut() {
        let actual_path =
            resolve_file_path(file_path, project_path, workspace_root, manifest_dir.as_deref());

        if actual_path.exists() {
            let content = tokio::fs::read_to_string(&actual_path).await?;
            metrics.sloc = count_sloc(&content);
            log_sloc_debug(&actual_path, metrics.sloc);
        } else {
            unresolved.push(file_path.clone());
            log_file_not_found_debug(file_path, &actual_path, workspace_root);
        }
    }

    // A file whose SLOC we could not read has `sloc == 0`, which makes its
    // density unmeasurable and drops it out of the hotspot ranking. Say so
    // instead of letting it silently disappear from a "total".
    if !unresolved.is_empty() {
        unresolved.sort();
        eprintln!(
            "⚠️  SLOC not measured for {} file(s); their defect density is not \
             reported and they cannot be the hotspot: {}",
            unresolved.len(),
            unresolved
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

/// Nearest ancestor of `start` that holds a `Cargo.toml` — the directory cargo
/// itself resolves the manifest from, and therefore the base its relative span
/// paths are usually stated against.
fn find_manifest_dir(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

/// Resolve a diagnostic's file path to something on disk.
///
/// CWD INDEPENDENCE: this used to start with `if file_path.exists()`, which
/// resolves a RELATIVE path (cargo emits `src/lib.rs`) against the *process's*
/// current directory. Running `pmat analyze lint-hotspot -p /tmp/fixture` from
/// inside a Rust repo therefore measured the SLOC of the CALLER's `src/lib.rs`:
/// observed sloc 414 (pmat's own lib.rs) for a 23-line fixture file, which put
/// the reported defect density 18x too low. A bare relative path is now only
/// ever resolved against the analysed project.
fn resolve_file_path(
    file_path: &Path,
    project_path: &Path,
    workspace_root: Option<&PathBuf>,
    manifest_dir: Option<&Path>,
) -> PathBuf {
    if file_path.is_absolute() && file_path.exists() {
        return file_path.to_path_buf();
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = manifest_dir {
        candidates.push(dir.join(file_path));
    }
    if let Some(ws_root) = workspace_root {
        candidates.push(ws_root.join(file_path));
        candidates.push(ws_root.join("server").join(file_path));
    }
    candidates.push(project_path.join(file_path));

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    project_path.join(file_path)
}

/// Count source lines of code (non-empty, non-comment lines)
fn count_sloc(content: &str) -> usize {
    content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count()
}

/// Log SLOC debug info if enabled
fn log_sloc_debug(path: &Path, sloc: usize) {
    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() && sloc > 0 {
        eprintln!("✓ File {} has {} SLOC", path.display(), sloc);
    }
}

/// Log file not found debug info if enabled
fn log_file_not_found_debug(
    file_path: &Path,
    actual_path: &Path,
    workspace_root: Option<&PathBuf>,
) {
    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
        eprintln!("⚠️  Could not find file: {}", file_path.display());
        eprintln!("   Tried: {}", actual_path.display());
        if let Some(ws) = workspace_root {
            eprintln!("   Workspace root: {}", ws.display());
        }
    }
}

/// Find workspace root by looking for Cargo.toml with [workspace]
///
/// # Errors
///
/// Returns an error if the operation fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn find_workspace_root(start_path: &Path) -> Result<Option<PathBuf>> {
    let mut current = start_path;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if this Cargo.toml contains [workspace]
            let contents = std::fs::read_to_string(&cargo_toml)?;
            if contents.contains("[workspace]") {
                return Ok(Some(current.to_path_buf()));
            }
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Ok(None)
}
