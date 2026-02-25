// CiScorer workflow scoring methods (E1 + E2)
// Included from ci_scorer.rs - do NOT add `use` imports or `#!` attributes here.

fn find_yaml_workflow_files(workflows_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = vec![];
    for entry in WalkDir::new(workflows_dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            let extension = path.extension().and_then(|s| s.to_str());
            if extension == Some("yml") || extension == Some("yaml") {
                files.push(path.to_path_buf());
            }
        }
    }
    files
}

async fn collect_workflow_content(workflows_dir: &Path) -> String {
    let mut all_content = String::new();
    for entry in WalkDir::new(workflows_dir)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            let extension = path.extension().and_then(|s| s.to_str());
            if extension == Some("yml") || extension == Some("yaml") {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    all_content.push_str(&content.to_lowercase());
                }
            }
        }
    }
    all_content
}

fn check_workflow_structure(content: &str, workflow_name: &str, workflow_path: &Path) -> (f64, Finding) {
    let has_name = content.contains("name:");
    let has_on = content.contains("on:");
    let has_jobs = content.contains("jobs:");

    if has_name && has_on && has_jobs {
        (2.0, Finding {
            severity: Severity::Success,
            category: "CI".to_string(),
            message: format!("\u{2713} Valid workflow structure: {} (+2 pts)", workflow_name),
            location: Some(workflow_path.display().to_string()),
            impact_points: 2.0,
        })
    } else {
        let mut missing = vec![];
        if !has_name { missing.push("name"); }
        if !has_on { missing.push("on"); }
        if !has_jobs { missing.push("jobs"); }
        (0.0, Finding {
            severity: Severity::Warning,
            category: "CI".to_string(),
            message: format!("Incomplete: {} missing {} (+2 pts if fixed)", workflow_name, missing.join(", ")),
            location: Some(workflow_path.display().to_string()),
            impact_points: 0.0,
        })
    }
}

fn ci_pattern_finding(detected: bool, success_msg: &str, missing_msg: &str) -> (f64, Finding) {
    if detected {
        (1.0, Finding {
            severity: Severity::Success,
            category: "CI".to_string(),
            message: success_msg.to_string(),
            location: None,
            impact_points: 1.0,
        })
    } else {
        (0.0, Finding {
            severity: Severity::Info,
            category: "CI".to_string(),
            message: missing_msg.to_string(),
            location: None,
            impact_points: 0.0,
        })
    }
}

impl CiScorer {
    /// Score CI workflows presence (E1: 6 points)
    async fn score_workflows_present(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let workflows_dir = repo_path.join(".github/workflows");

        if !workflows_dir.exists() {
            return Ok(SubcategoryScore {
                id: "E1".to_string(),
                name: "CI Workflows Present".to_string(),
                score: 0.0,
                max_score: 6.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "CI".to_string(),
                    message:
                        "Missing: Create .github/workflows/ directory with CI workflow (+6 pts)"
                            .to_string(),
                    location: Some(workflows_dir.display().to_string()),
                    impact_points: -6.0,
                }],
            });
        }

        let workflow_files = find_yaml_workflow_files(&workflows_dir);

        if workflow_files.is_empty() {
            return Ok(SubcategoryScore {
                id: "E1".to_string(),
                name: "CI Workflows Present".to_string(),
                score: 1.0,
                max_score: 6.0,
                findings: vec![Finding {
                    severity: Severity::Warning,
                    category: "CI".to_string(),
                    message: "Missing: Add workflow files to .github/workflows/ (+5 pts)"
                        .to_string(),
                    location: Some(workflows_dir.display().to_string()),
                    impact_points: -5.0,
                }],
            });
        }

        let score = match workflow_files.len() {
            1 => 2.0,
            2 => 4.0,
            _ => 6.0,
        };

        let findings = workflow_files.iter().map(|wf| Finding {
            severity: Severity::Success,
            category: "CI".to_string(),
            message: format!(
                "\u{2713} Workflow: {} (+2 pts)",
                wf.file_name().expect("internal error").to_string_lossy()
            ),
            location: Some(wf.display().to_string()),
            impact_points: 2.0,
        }).collect();

        Ok(SubcategoryScore {
            id: "E1".to_string(),
            name: "CI Workflows Present".to_string(),
            score,
            max_score: 6.0,
            findings,
        })
    }

    /// Score workflow configuration (E2: 6 points)
    async fn score_workflows_configured(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let workflows_dir = repo_path.join(".github/workflows");
        let empty_result = SubcategoryScore {
            id: "E2".to_string(),
            name: "Workflows Configured Properly".to_string(),
            score: 0.0,
            max_score: 6.0,
            findings: vec![],
        };

        if !workflows_dir.exists() {
            return Ok(empty_result);
        }

        let workflow_files = find_yaml_workflow_files(&workflows_dir);
        if workflow_files.is_empty() {
            return Ok(empty_result);
        }

        let mut total_score: f64 = 0.0;
        let mut findings = vec![];
        let mut has_testing = false;
        let mut has_linting = false;

        for workflow_path in &workflow_files {
            let content = match tokio::fs::read_to_string(workflow_path).await {
                Ok(c) => c,
                Err(_) => continue,
            };

            let workflow_name = workflow_path.file_name().expect("internal error").to_string_lossy();
            let (pts, finding) = check_workflow_structure(&content, &workflow_name, workflow_path);
            total_score += pts;
            findings.push(finding);

            let content_lower = content.to_lowercase();
            has_testing = has_testing || content_lower.contains("test") || content_lower.contains("cargo test") || content_lower.contains("npm test");
            has_linting = has_linting || content_lower.contains("lint") || content_lower.contains("clippy") || content_lower.contains("eslint");
        }

        let (pts, finding) = ci_pattern_finding(has_testing, "\u{2713} Testing step detected (+1 pt)", "Missing: Add testing step (cargo test, npm test) (+1 pt)");
        total_score += pts;
        findings.push(finding);

        let (pts, finding) = ci_pattern_finding(has_linting, "\u{2713} Linting step detected (+1 pt)", "Missing: Add linting step (clippy, eslint) (+1 pt)");
        total_score += pts;
        findings.push(finding);

        Ok(SubcategoryScore {
            id: "E2".to_string(),
            name: "Workflows Configured Properly".to_string(),
            score: total_score.min(6.0),
            max_score: 6.0,
            findings,
        })
    }
}
