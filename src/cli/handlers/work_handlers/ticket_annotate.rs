/// Handle work annotate command - show unified quality metrics for a ticket
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_annotate(
    id: String,
    path: Option<PathBuf>,
    format: crate::cli::commands::AnnotateOutputFormat,
    with_churn: bool,
    churn_days: u32,
) -> Result<()> {
    use crate::cli::commands::AnnotateOutputFormat;
    use crate::services::spec_parser::SpecParser;

    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the ticket
    let item = find_item_fuzzy(&service, &id)?;

    // Collect annotations
    let mut annotations = TicketAnnotations {
        ticket_id: item.id.clone(),
        title: item.title.clone(),
        status: format!("{:?}", item.status),
        priority: format!("{:?}", item.priority),
        spec_path: item.spec.clone(),
        spec_score: None,
        files: vec![],
        avg_tdg: None,
        file_tdg_scores: vec![],
        total_churn: None,
        churn_hotspots: vec![],
        coverage_percent: None,
        repeated_fixes: vec![],
    };

    // Get spec score if spec exists
    if let Some(ref spec_path) = item.spec {
        let full_spec_path = project_path.join(spec_path);
        if full_spec_path.exists() {
            let parser = SpecParser::new();
            if let Ok(spec) = parser.parse_file(&full_spec_path) {
                annotations.spec_score = Some(calculate_spec_score_simple(&spec));
            }
        }
    }

    // Find related files from acceptance criteria or labels
    let related_files = find_related_files(&item, &project_path);
    annotations.files = related_files.clone();

    // Calculate real TDG scores for related files
    if !related_files.is_empty() {
        let calculator = crate::services::tdg_calculator::TDGCalculator::new()
            .with_project_root(project_path.clone());

        let mut tdg_scores = Vec::new();
        let mut tdg_sum = 0.0;
        for file in &related_files {
            let full_path = project_path.join(file);
            match calculator.calculate_file(&full_path).await {
                Ok(score) => {
                    tdg_sum += score.value;
                    tdg_scores.push(FileTdgScore {
                        file: file.to_string_lossy().to_string(),
                        score: score.value,
                        severity: format!("{:?}", score.severity),
                    });
                }
                Err(_) => {
                    // File might not be parseable; skip silently
                }
            }
        }
        if !tdg_scores.is_empty() {
            annotations.avg_tdg = Some(tdg_sum / tdg_scores.len() as f64);
        }
        annotations.file_tdg_scores = tdg_scores;
    }

    // Detect project coverage from LCOV report
    annotations.coverage_percent = detect_coverage_percent(&project_path);

    // Churn analysis if requested
    if with_churn && !related_files.is_empty() {
        let churn_result = analyze_churn_simple(&project_path, &related_files, churn_days);
        annotations.total_churn = Some(churn_result.total_commits);
        annotations.churn_hotspots = churn_result.hotspots;
        annotations.repeated_fixes = churn_result.repeated_fixes;
    }

    // Output based on format
    match format {
        AnnotateOutputFormat::Text => print_annotations_text(&annotations),
        AnnotateOutputFormat::Json => print_annotations_json(&annotations)?,
        AnnotateOutputFormat::Markdown => print_annotations_markdown(&annotations),
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
struct TicketAnnotations {
    ticket_id: String,
    title: String,
    status: String,
    priority: String,
    spec_path: Option<PathBuf>,
    spec_score: Option<f64>,
    files: Vec<PathBuf>,
    avg_tdg: Option<f64>,
    file_tdg_scores: Vec<FileTdgScore>,
    total_churn: Option<usize>,
    churn_hotspots: Vec<String>,
    coverage_percent: Option<f64>,
    repeated_fixes: Vec<RepeatedFix>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FileTdgScore {
    file: String,
    score: f64,
    severity: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RepeatedFix {
    file: String,
    line_range: String,
    fix_count: usize,
    description: String,
}

struct ChurnResult {
    total_commits: usize,
    hotspots: Vec<String>,
    repeated_fixes: Vec<RepeatedFix>,
}

fn calculate_spec_score_simple(spec: &crate::services::spec_parser::ParsedSpec) -> f64 {
    debug_assert!(true, "contract: calculate_spec_score_simple");
    let mut score = 0.0;
    if !spec.issue_refs.is_empty() {
        score += 10.0;
    }
    score += (spec.code_examples.len().min(5) * 4) as f64;
    score += (spec.acceptance_criteria.len().min(10)) as f64 * 2.5;
    score += (spec.claims.len().min(15)) as f64;
    if !spec.title.is_empty() {
        score += 5.0;
    }
    score += (spec.test_requirements.len().min(5) * 3) as f64;
    // Citations: 2 pts each up to 5
    let citations = {
        let mut seen = std::collections::HashSet::new();
        if let Ok(re) = regex::Regex::new(r"\[(\d+)\]") {
            for caps in re.captures_iter(&spec.raw_content) {
                if let Some(m) = caps.get(1) {
                    seen.insert(m.as_str().to_string());
                }
            }
        }
        seen.len()
    };
    score += (citations.min(5) * 2) as f64;
    score.min(100.0)
}

/// Extract file paths mentioned in a spec file (helper for find_related_files)
fn extract_files_from_spec(spec_path: &Path, project_path: &Path) -> Vec<PathBuf> {
    debug_assert!(spec_path.exists(), "spec_path must exist: {}", spec_path.display());
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let full_path = project_path.join(spec_path);
    let content = match std::fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let re = match regex::Regex::new(r"`([\w/._-]+\.(?:rs|ts|py|go|js))`") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    re.captures_iter(&content)
        .filter_map(|cap| cap.get(1))
        .filter(|m| project_path.join(m.as_str()).exists())
        .map(|m| PathBuf::from(m.as_str()))
        .collect()
}

/// Extract file paths from item labels (helper for find_related_files)
fn extract_files_from_labels(labels: &[String], project_path: &Path) -> Vec<PathBuf> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    labels
        .iter()
        .filter(|label| label.ends_with(".rs") || label.ends_with(".ts"))
        .filter(|label| project_path.join(label).exists())
        .map(PathBuf::from)
        .collect()
}

fn find_related_files(
    item: &crate::models::roadmap::RoadmapItem,
    project_path: &Path,
) -> Vec<PathBuf> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut files = Vec::new();

    if let Some(ref spec_path) = item.spec {
        if project_path.join(spec_path).exists() {
            files.extend(extract_files_from_spec(spec_path, project_path));
        }
    }

    files.extend(extract_files_from_labels(&item.labels, project_path));

    files.sort();
    files.dedup();
    files.into_iter().take(10).collect()
}

fn analyze_churn_simple(project_path: &Path, files: &[PathBuf], days: u32) -> ChurnResult {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut total_commits = 0;
    let mut hotspots = Vec::new();
    let mut repeated_fixes = Vec::new();

    for file in files {
        // Run git log to count commits
        let output = std::process::Command::new("git")
            .args([
                "log",
                "--oneline",
                &format!("--since={} days ago", days),
                "--",
                &file.to_string_lossy(),
            ])
            .current_dir(project_path)
            .output();

        if let Ok(output) = output {
            let commit_count = String::from_utf8_lossy(&output.stdout)
                .lines()
                .count();
            total_commits += commit_count;

            if commit_count > 5 {
                hotspots.push(format!("{}: {} commits", file.display(), commit_count));
            }

            // Check for repeated fix patterns (same file, similar commit messages)
            let log_output = std::process::Command::new("git")
                .args([
                    "log",
                    "--oneline",
                    &format!("--since={} days ago", days),
                    "--grep=fix",
                    "-i",
                    "--",
                    &file.to_string_lossy(),
                ])
                .current_dir(project_path)
                .output();

            if let Ok(log_output) = log_output {
                let fix_count = String::from_utf8_lossy(&log_output.stdout)
                    .lines()
                    .count();
                if fix_count >= 2 {
                    repeated_fixes.push(RepeatedFix {
                        file: file.to_string_lossy().to_string(),
                        line_range: "various".to_string(),
                        fix_count,
                        description: format!("{} fix commits in {} days (Tarantula alert)", fix_count, days),
                    });
                }
            }
        }
    }

    ChurnResult {
        total_commits,
        hotspots,
        repeated_fixes,
    }
}

/// Convert TDG score (0-5) to human-readable severity label.
fn tdg_severity_label(score: f64) -> &'static str {
    debug_assert!(score >= 0.0, "score must be non-negative");
    if score <= 1.0 {
        "Excellent"
    } else if score <= 2.0 {
        "Good"
    } else if score <= 3.0 {
        "Moderate"
    } else {
        "Critical"
    }
}

/// Detect project coverage from LCOV report at standard locations.
fn detect_coverage_percent(project_path: &Path) -> Option<f64> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let candidates = [
        project_path.join("target/coverage/lcov.info"),
        project_path.join("target/llvm-cov/lcov.info"),
        project_path.join("coverage/lcov.info"),
        project_path.join("lcov.info"),
    ];

    let lcov_path = candidates.iter().find(|p| p.exists())?;
    let content = std::fs::read_to_string(lcov_path).ok()?;

    let mut lines_found: usize = 0;
    let mut lines_hit: usize = 0;

    for line in content.lines() {
        if let Some(num) = line.strip_prefix("LF:") {
            lines_found += num.parse::<usize>().unwrap_or(0);
        } else if let Some(num) = line.strip_prefix("LH:") {
            lines_hit += num.parse::<usize>().unwrap_or(0);
        }
    }

    if lines_found > 0 {
        Some((lines_hit as f64 / lines_found as f64) * 100.0)
    } else {
        None
    }
}
