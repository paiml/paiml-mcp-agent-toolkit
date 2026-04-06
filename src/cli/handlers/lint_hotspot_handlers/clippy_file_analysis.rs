// clippy_file_analysis.rs - File resolution, SLOC counting, and workspace detection
// Included by clippy.rs via include!()

async fn run_clippy_command(
    project_path: &Path,
    clippy_flags: &str,
) -> Result<std::process::Output> {
    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();
    let mut cmd = Command::new("cargo");

    cmd.current_dir(project_path)
        .arg("clippy")
        .arg("--all-targets")
        .arg("--message-format=json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if !flags.is_empty() {
        cmd.arg("--").args(&flags);
    }

    cmd.output().await.context("Failed to run cargo clippy")
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
    let total_violations = file_violations.len();
    let defect_density = (total_violations as f64 / sloc as f64) * 100.0;

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
        quality_gate: QualityGateStatus {
            passed: defect_density <= 5.0,
            violations: vec![],
            blocking: false,
        },
    })
}

/// Count top lint types from violations
pub(crate) fn count_top_lints(violations: &[ViolationDetail]) -> Vec<(String, usize)> {
    let mut lint_counts: HashMap<String, usize> = HashMap::new();

    for violation in violations {
        *lint_counts.entry(violation.lint_name.clone()).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = lint_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts.truncate(10); // Top 10 lints
    counts
}

/// Count source lines in a file
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

/// Execute clippy command with given flags (cognitive complexity <=3)
pub(crate) async fn execute_clippy_command(
    project_path: &Path,
    flags: &[&str],
) -> Result<std::process::Output> {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("clippy")
        .arg("--message-format=json")
        .args(flags)
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    Ok(cmd.output().await?)
}

/// Check clippy output status (cognitive complexity <=5)
pub(crate) fn check_clippy_output(output: &std::process::Output) -> Result<()> {
    if !output.status.success()
        && output.status.code() != Some(101)
        && std::env::var("LINT_HOTSPOT_DEBUG").is_ok()
    {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("⚠️  Clippy exited with status: {:?}", output.status);
        eprintln!("Stderr: {stderr}");
    }
    Ok(())
}

/// Calculate SLOC for each file in metrics
async fn calculate_sloc_for_files(
    file_metrics: &mut HashMap<PathBuf, FileMetrics>,
    project_path: &Path,
    workspace_root: Option<&PathBuf>,
) -> Result<()> {
    for (file_path, metrics) in file_metrics.iter_mut() {
        let actual_path = resolve_file_path(file_path, project_path, workspace_root);

        if actual_path.exists() {
            let content = tokio::fs::read_to_string(&actual_path).await?;
            metrics.sloc = count_sloc(&content);
            log_sloc_debug(&actual_path, metrics.sloc);
        } else {
            log_file_not_found_debug(file_path, &actual_path, workspace_root);
        }
    }
    Ok(())
}

/// Resolve actual file path trying various locations
fn resolve_file_path(
    file_path: &Path,
    project_path: &Path,
    workspace_root: Option<&PathBuf>,
) -> PathBuf {
    if file_path.exists() {
        return file_path.to_path_buf();
    }

    if let Some(ws_root) = workspace_root {
        let ws_relative = ws_root.join(file_path);
        if ws_relative.exists() {
            return ws_relative;
        }

        let with_server = ws_root.join("server").join(file_path);
        if with_server.exists() {
            return with_server;
        }
    }

    let project_relative = project_path.join(file_path);
    if project_relative.exists() {
        project_relative
    } else {
        file_path.to_path_buf()
    }
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
pub(crate) fn find_workspace_root(start_path: &Path) -> Result<Option<PathBuf>> {
    debug_assert!(start_path.exists(), "start_path must exist: {}", start_path.display());
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
