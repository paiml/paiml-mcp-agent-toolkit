// CI/CD integration and build automation scoring
// Included into rust_tooling_scorer.rs

/// Check if any workflow file's name matches one of the given patterns
fn workflow_has_name(files: &[std::fs::DirEntry], patterns: &[&str]) -> bool {
    files.iter().any(|entry| {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        patterns.iter().any(|p| name.contains(p))
    })
}

/// Score GitHub Actions workflow quality (28pts)
fn score_workflows(workflows_dir: &Path) -> ScorerResult<f64> {
    let workflow_files: Vec<_> = std::fs::read_dir(workflows_dir)
        .map_err(|e| ScorerError::IoError(e.to_string()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
        })
        .collect();

    let mut all_content = String::new();
    for file in &workflow_files {
        if let Ok(content) = std::fs::read_to_string(file.path()) {
            all_content.push_str(&content);
            all_content.push('\n');
        }
    }

    let uses_sovereign_ci = all_content.contains("sovereign-ci");

    let mut score = 0.0;

    // sovereign-ci.yml provides multi-platform, audit, lint, and feature matrix
    let has_multi_platform = uses_sovereign_ci
        || (all_content.contains("ubuntu-")
            && all_content.contains("windows-")
            && all_content.contains("macos-"));
    if has_multi_platform {
        score += 6.0;
    }

    let has_feature_matrix = uses_sovereign_ci
        || (all_content.contains("features:")
            && (all_content.contains("minimal")
                || all_content.contains("default")
                || all_content.contains("full")));
    if has_feature_matrix {
        score += 4.0;
    }
    if workflow_files.len() >= 3 {
        score += 6.0;
    }
    if uses_sovereign_ci
        || workflow_has_name(&workflow_files, &["audit", "security"])
        || all_content.contains("cargo audit")
    {
        score += 4.0;
    }
    if workflow_has_name(&workflow_files, &["bench", "benchmark"])
        || all_content.contains("cargo bench")
    {
        score += 3.0;
    }
    if uses_sovereign_ci
        || workflow_has_name(&workflow_files, &["lint", "clippy", "spell"])
    {
        score += 2.0;
    }
    if workflow_has_name(&workflow_files, &["stress", "loom"]) {
        score += 3.0;
    }
    Ok(score)
}

/// Score build automation (justfile/Makefile/xtask, 8pts)
///
/// #244: Makefile given equal base score as justfile (5.0). The Windows argument
/// is weak for Rust projects that already require Unix-like toolchains.
fn score_build_automation(project_path: &Path) -> f64 {
    let justfile = project_path.join("justfile");
    let makefile = project_path.join("Makefile");
    let xtask = project_path.join("xtask");

    let (base_score, content) = if justfile.exists() {
        (5.0, std::fs::read_to_string(&justfile).unwrap_or_default())
    } else if xtask.exists() {
        (5.0, String::new())
    } else if makefile.exists() {
        (5.0, std::fs::read_to_string(&makefile).unwrap_or_default())
    } else {
        return 0.0;
    };

    let has_all_targets = content.contains("build:")
        && content.contains("test:")
        && (content.contains("lint:") || content.contains("clippy:"))
        && content.contains("bench:");
    base_score + if has_all_targets { 3.0 } else { 0.0 }
}

impl RustToolingScorer {
    /// Score CI/CD integration and build automation (v2.0 Phase 2)
    ///
    /// Based on "Learn from Rust Giants" specification (TPS-reviewed):
    ///
    /// **Multi-Platform CI** (13pts):
    /// - +6pts: CI tests on Linux + Windows + Mac
    /// - +4pts: Feature matrix testing (minimal, default, full)
    /// - +3pts: Separate workflows for stress tests, loom, audit
    ///
    /// **CI Workflow Diversity** (15pts):
    /// - +6pts: ≥3 separate GitHub Actions workflows
    /// - +4pts: Dedicated security audit workflow
    /// - +3pts: Dedicated benchmark workflow
    /// - +2pts: Dedicated spell-check or linting workflow
    ///
    /// **Build Automation** (9pts):
    /// - +5pts: justfile, cargo-xtask, or Makefile exists
    /// - +3pts: Common targets (build, test, lint, bench)
    /// - +2pts: CI uses automation targets (consistency)
    ///
    /// Total possible: 37 points
    pub(super) fn score_ci_cd_integration(
        &self,
        project_path: &Path,
        _cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let mut score = 0.0;

        let workflows_dir = project_path.join(".github").join("workflows");
        if workflows_dir.exists() && workflows_dir.is_dir() {
            score += score_workflows(&workflows_dir)?;
        }

        score += score_build_automation(project_path);
        Ok(score)
    }
}
