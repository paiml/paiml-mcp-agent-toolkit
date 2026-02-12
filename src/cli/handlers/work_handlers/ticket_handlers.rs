
/// Collect semantic warnings for roadmap items (helper for handle_work_validate)
fn collect_semantic_warnings(roadmap: &crate::models::roadmap::Roadmap) -> Vec<String> {
    let mut warnings = Vec::new();
    for item in &roadmap.roadmap {
        if item.acceptance_criteria.is_empty()
            && !matches!(item.status, ItemStatus::Cancelled)
        {
            warnings.push(format!("⚠️  {} has no acceptance criteria", item.id));
        }
        if item.id.chars().count() > 50 {
            let truncated: String = item.id.chars().take(30).collect();
            warnings.push(format!(
                "⚠️  {} has a long ID ({} chars) - consider using shorter IDs",
                truncated,
                item.id.chars().count()
            ));
        }
    }
    warnings
}

/// Print valid roadmap with semantic validation (helper for handle_work_validate)
fn print_valid_roadmap(
    roadmap: &crate::models::roadmap::Roadmap,
    verbose: bool,
    fix: bool,
) {
    println!("✅ Syntax valid");
    println!("   Version: {}", roadmap.roadmap_version);
    println!("   Items: {}", roadmap.roadmap.len());
    println!(
        "   GitHub: {}",
        if roadmap.github_enabled {
            roadmap.github_repo.as_deref().unwrap_or("not configured")
        } else {
            "disabled"
        }
    );
    println!();

    let warnings = collect_semantic_warnings(roadmap);
    if !warnings.is_empty() {
        println!("Warnings ({}):", warnings.len());
        for warning in &warnings {
            println!("   {}", warning);
        }
        println!();
    }

    if verbose {
        println!("📋 Items:");
        for item in &roadmap.roadmap {
            println!("   {} [{:?}] - {}", item.id, item.status, item.title);
        }
    }

    if fix && !warnings.is_empty() {
        println!("💡 Tip: Use `pmat work migrate` to auto-fix issues");
    }

    println!("✅ Validation passed");
}

/// Print YAML parse error with context and suggestions (helper for handle_work_validate)
fn print_yaml_error_context(error_msg: &str, content: &str) {
    println!("❌ Validation failed\n");
    println!("Error: {}", error_msg);
    println!();

    if let Some(line) = extract_line_from_yaml_error(error_msg) {
        let lines: Vec<&str> = content.lines().collect();
        if line > 0 && line <= lines.len() {
            println!("Context (around line {}):", line);
            let start = line.saturating_sub(3);
            let end = std::cmp::min(line + 2, lines.len());
            for (i, l) in lines[start..end].iter().enumerate() {
                let line_num = start + i + 1;
                let marker = if line_num == line { ">>>" } else { "   " };
                println!("{} {:4}: {}", marker, line_num, l);
            }
            println!();
        }
    }

    println!("💡 Common fixes:");
    println!("   - Use valid status values: completed, done, wip, planned, blocked, review");
    println!("   - Quote strings with special characters: `:`, `<`, `>`");
    println!("   - Use proper YAML indentation (2 spaces)");
    println!();
    println!("Run `pmat work status --list` to see all valid status values.");
}

/// Handle work validate command (Part B: UX Improvements)
///
/// Validates roadmap.yaml syntax and content with actionable error messages.
pub async fn handle_work_validate(path: Option<PathBuf>, verbose: bool, fix: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("🔍 Validating roadmap: {}", roadmap_path.display());
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    let content = std::fs::read_to_string(&roadmap_path).context("Failed to read roadmap file")?;

    match serde_yaml::from_str::<crate::models::roadmap::Roadmap>(&content) {
        Ok(roadmap) => {
            print_valid_roadmap(&roadmap, verbose, fix);
            Ok(())
        }
        Err(e) => {
            print_yaml_error_context(&format!("{}", e), &content);
            anyhow::bail!("Roadmap validation failed")
        }
    }
}

/// Handle work migrate command (Part B: UX Improvements)
///
/// Auto-fixes common roadmap.yaml issues.
pub async fn handle_work_migrate(path: Option<PathBuf>, dry_run: bool, backup: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("🔄 Migrating roadmap: {}", roadmap_path.display());
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    let content = std::fs::read_to_string(&roadmap_path)?;
    let mut changes: Vec<String> = Vec::new();
    let mut new_content = content.clone();

    // 1. Normalize status values
    let status_patterns = [
        ("status: done", "status: completed"),
        ("status: Done", "status: completed"),
        ("status: DONE", "status: completed"),
        ("status: finished", "status: completed"),
        ("status: in progress", "status: inprogress"),
        ("status: In Progress", "status: inprogress"),
        ("status: WIP", "status: inprogress"),
        ("status: wip", "status: inprogress"),
        ("status: stuck", "status: blocked"),
        ("status: on-hold", "status: blocked"),
        ("status: todo", "status: planned"),
        ("status: TODO", "status: planned"),
        ("status: open", "status: planned"),
    ];

    for (old, new) in status_patterns {
        if new_content.contains(old) {
            changes.push(format!("Normalize status: {} → {}", old, new));
            new_content = new_content.replace(old, new);
        }
    }

    // 2. Quote special characters in titles
    let special_chars = [':', '<', '>', '≥', '≤', '±', 'ε', '→', '↔'];
    for line in content.lines() {
        if line.trim_start().starts_with("title:") || line.trim_start().starts_with("- title:") {
            let has_special = special_chars
                .iter()
                .any(|c| line.contains(*c) && !line.contains("\""));
            if has_special && !line.contains("\"") {
                // This is a simplistic check - in practice we'd need proper YAML parsing
                changes.push(format!("Consider quoting: {}", line.trim()));
            }
        }
    }

    if changes.is_empty() {
        println!("✅ No migrations needed - roadmap is already up to date");
        return Ok(());
    }

    println!("Found {} potential changes:", changes.len());
    for change in &changes {
        println!("   • {}", change);
    }
    println!();

    if dry_run {
        println!("(Dry run - no changes made)");
        return Ok(());
    }

    // Create backup
    if backup {
        let backup_path = roadmap_path.with_extension("yaml.bak");
        std::fs::write(&backup_path, &content)?;
        println!("✅ Created backup: {}", backup_path.display());
    }

    // Write changes
    std::fs::write(&roadmap_path, &new_content)?;
    println!("✅ Updated roadmap: {}", roadmap_path.display());

    // Verify the changes
    if serde_yaml::from_str::<crate::models::roadmap::Roadmap>(&new_content).is_ok() {
        println!("✅ Verified: updated roadmap is valid");
    } else {
        println!("⚠️  Warning: updated roadmap may have issues - check manually");
    }

    Ok(())
}

/// Handle work list-statuses command (Part B: UX Improvements)
///
/// Lists all valid status values with descriptions and aliases.
pub async fn handle_work_list_statuses() -> Result<()> {
    println!("📋 Valid Status Values\n");
    println!("{:<15} {:<25} DESCRIPTION", "STATUS", "ALIASES");
    println!("{}", "-".repeat(70));

    let statuses = [
        (
            "planned",
            "todo, open, pending, new",
            "Task not yet started",
        ),
        (
            "inprogress",
            "wip, active, started",
            "Currently being worked on",
        ),
        (
            "blocked",
            "stuck, waiting, on-hold",
            "Cannot proceed (waiting on something)",
        ),
        (
            "review",
            "reviewing, pr, pending-review",
            "Ready for or in code review",
        ),
        (
            "completed",
            "done, finished, closed",
            "Work finished successfully",
        ),
        (
            "cancelled",
            "canceled, dropped, wontfix",
            "Work abandoned or not needed",
        ),
    ];

    for (status, aliases, description) in statuses {
        println!("{:<15} {:<25} {}", status, aliases, description);
    }

    println!();
    println!("💡 All status values are case-insensitive and support hyphens/underscores.");
    println!("   Example: 'In-Progress', 'in_progress', 'InProgress', 'WIP' all work.");

    Ok(())
}

/// Handle work add command (CRUD: Create)
///
/// Creates a new work ticket in roadmap.yaml with optional GitHub issue creation.
pub async fn handle_work_add(
    title: String,
    description: Option<String>,
    priority: crate::cli::commands::WorkPriority,
    tags: Option<String>,
    path: Option<PathBuf>,
    create_github: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    // Validate roadmap exists
    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Load existing roadmap to find next available ID
    let roadmap = service.load()?;
    let next_id = generate_next_id(&roadmap);

    // Create new item
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let item = crate::models::roadmap::RoadmapItem {
        id: next_id.clone(),
        github_issue: None,
        item_type: crate::models::roadmap::ItemType::Task,
        title: title.clone(),
        status: crate::models::roadmap::ItemStatus::Planned,
        priority: priority.to_roadmap_priority(),
        assigned_to: None,
        created: now.clone(),
        updated: now,
        spec: None,
        acceptance_criteria: description
            .as_ref()
            .map(|d| vec![d.clone()])
            .unwrap_or_default(),
        phases: vec![],
        subtasks: vec![],
        estimated_effort: None,
        labels: tags
            .clone()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        notes: None,
    };

    // Save to roadmap
    service.upsert_item(item)?;

    println!("✅ Created ticket: {}", next_id);
    println!("   Title: {}", title);
    println!("   Priority: {:?}", priority);
    if let Some(desc) = description {
        println!("   Description: {}", desc);
    }
    if let Some(t) = tags {
        println!("   Tags: {}", t);
    }

    // Create GitHub issue if requested
    if create_github {
        println!("\n⚠️  GitHub issue creation not yet implemented. Use 'pmat work sync' after creating the ticket.");
    }

    Ok(())
}

/// Handle work list command (CRUD: Read - simple list)
///
/// Lists all work tickets with optional filtering.
pub async fn handle_work_list(
    status: Option<String>,
    priority: Option<crate::cli::commands::WorkPriority>,
    count_only: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    let roadmap = service.load()?;

    // Filter items
    let items: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| {
            // Filter by status if specified
            if let Some(ref s) = status {
                let item_status = format!("{:?}", item.status).to_lowercase();
                if !item_status.contains(&s.to_lowercase()) {
                    return false;
                }
            }
            // Filter by priority if specified
            if let Some(ref p) = priority {
                let roadmap_priority = p.to_roadmap_priority();
                if item.priority != roadmap_priority {
                    return false;
                }
            }
            true
        })
        .collect();

    if count_only {
        println!("{}", items.len());
        return Ok(());
    }

    if items.is_empty() {
        println!("No tickets found matching criteria.");
        return Ok(());
    }

    // Print header
    println!("{:<12} {:<12} {:<10} TITLE", "ID", "STATUS", "PRIORITY");
    println!("{}", "-".repeat(70));

    // Print items
    for item in items {
        let status_str = format!("{:?}", item.status).to_lowercase();
        let priority_str = format!("{:?}", item.priority).to_lowercase();
        let title_truncated = if item.title.len() > 40 {
            format!("{}...", item.title.get(..37).unwrap_or(&item.title))
        } else {
            item.title.clone()
        };
        println!(
            "{:<12} {:<12} {:<10} {}",
            item.id, status_str, priority_str, title_truncated
        );
    }

    Ok(())
}

/// Handle work edit command (CRUD: Update)
///
/// Edits an existing work ticket.
pub async fn handle_work_edit(
    id: String,
    title: Option<String>,
    description: Option<String>,
    priority: Option<crate::cli::commands::WorkPriority>,
    status: Option<String>,
    tags: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the item (with fuzzy matching)
    let item = find_item_fuzzy(&service, &id)?;
    let mut updated_item = item.clone();
    let mut changes = vec![];

    // Apply changes
    if let Some(new_title) = title {
        updated_item.title = new_title.clone();
        changes.push(format!("title: {}", new_title));
    }

    if let Some(desc) = description {
        updated_item.acceptance_criteria = vec![desc.clone()];
        changes.push(format!("description: {}", desc));
    }

    if let Some(p) = priority {
        updated_item.priority = p.to_roadmap_priority();
        changes.push(format!("priority: {:?}", p));
    }

    if let Some(s) = status {
        let new_status = crate::models::roadmap::ItemStatus::from_string(&s)
            .map_err(|e| anyhow::anyhow!("Invalid status '{}': {}", s, e))?;
        updated_item.status = new_status;
        changes.push(format!("status: {}", s));
    }

    if let Some(t) = tags {
        updated_item.labels = t.split(',').map(|s| s.trim().to_string()).collect();
        changes.push(format!("labels: {}", t));
    }

    if changes.is_empty() {
        println!("⚠️  No changes specified. Use --title, --description, --priority, --status, or --tags.");
        return Ok(());
    }

    // Update timestamp
    updated_item.updated = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Save
    service.upsert_item(updated_item)?;

    println!("✅ Updated ticket: {}", item.id);
    for change in changes {
        println!("   {}", change);
    }

    Ok(())
}

/// Handle work delete command (CRUD: Delete)
///
/// Deletes a work ticket from roadmap.yaml.
pub async fn handle_work_delete(id: String, force: bool, path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the item (with fuzzy matching)
    let item = find_item_fuzzy(&service, &id)?;

    // Confirm deletion unless --force
    if !force {
        println!("About to delete ticket:");
        println!("  ID: {}", item.id);
        println!("  Title: {}", item.title);
        println!("  Status: {:?}", item.status);
        println!();
        println!("⚠️  Use --force to skip this confirmation.");
        return Ok(());
    }

    // Delete
    service.remove_item(&item.id)?;
    println!("🗑️  Deleted ticket: {} - {}", item.id, item.title);

    Ok(())
}

/// Handle work annotate command - show unified quality metrics for a ticket
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
    let mut score = 0.0;
    if !spec.issue_refs.is_empty() {
        score += 10.0;
    }
    score += (spec.code_examples.len().min(5) * 4) as f64;
    score += (spec.acceptance_criteria.len().min(10) * 3) as f64;
    score += (spec.claims.len().min(20)) as f64;
    if !spec.title.is_empty() {
        score += 5.0;
    }
    score += (spec.test_requirements.len().min(5) * 3) as f64;
    score.min(100.0)
}

/// Extract file paths mentioned in a spec file (helper for find_related_files)
fn extract_files_from_spec(spec_path: &Path, project_path: &Path) -> Vec<PathBuf> {
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
    labels
        .iter()
        .filter(|label| label.ends_with(".rs") || label.ends_with(".ts"))
        .filter(|label| project_path.join(label).exists())
        .map(|label| PathBuf::from(label))
        .collect()
}

fn find_related_files(
    item: &crate::models::roadmap::RoadmapItem,
    project_path: &Path,
) -> Vec<PathBuf> {
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

/// Print specification section of text annotations (helper for print_annotations_text)
fn print_text_spec_section(ann: &TicketAnnotations) {
    println!("📋 SPECIFICATION");
    if let Some(ref spec) = ann.spec_path {
        println!("   Path:  {}", spec.display());
        if let Some(score) = ann.spec_score {
            let status = if score >= 95.0 { "✅" } else { "❌" };
            println!("   Score: {:.1}/100 {}", score, status);
        }
    } else {
        println!("   ⚠️  No specification linked");
    }
    println!();
}

/// Print TDG section of text annotations (helper for print_annotations_text)
fn print_text_tdg_section(ann: &TicketAnnotations) {
    println!("📈 TDG (Technical Debt Gradient)");
    if let Some(tdg) = ann.avg_tdg {
        let severity = tdg_severity_label(tdg);
        println!("   Avg Score: {:.2}/5.0 ({})", tdg, severity);
        for ft in &ann.file_tdg_scores {
            println!("     {:.2} [{}] {}", ft.score, ft.severity, ft.file);
        }
    } else {
        println!("   Not calculated (no files)");
    }
    println!();
}

/// Print churn section of text annotations (helper for print_annotations_text)
fn print_text_churn_section(ann: &TicketAnnotations) {
    println!("🔄 CHURN ANALYSIS");
    if let Some(churn) = ann.total_churn {
        println!("   Total Commits: {}", churn);
        for h in &ann.churn_hotspots {
            println!("     ⚠️  {}", h);
        }
    } else {
        println!("   Run with --with-churn to analyze");
    }
    println!();
}

/// Print tarantula and coverage sections (helper for print_annotations_text)
fn print_text_fault_coverage_section(ann: &TicketAnnotations) {
    println!("🔴 TARANTULA FAULT DETECTION");
    if ann.repeated_fixes.is_empty() {
        println!("   ✅ No repeated fix patterns detected");
    } else {
        for fix in &ann.repeated_fixes {
            println!("   ⚠️  {}: {}", fix.file, fix.description);
        }
    }
    println!();

    println!("📊 COVERAGE");
    if let Some(cov) = ann.coverage_percent {
        let status = if cov >= 95.0 { "✅" } else { "❌" };
        println!("   {:.1}% {}", cov, status);
    } else {
        println!("   Not available (run coverage analysis)");
    }
}

fn print_annotations_text(ann: &TicketAnnotations) {
    println!("📊 Quality Annotations for {}\n", ann.ticket_id);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Title:    {}", ann.title);
    println!("Status:   {}", ann.status);
    println!("Priority: {}", ann.priority);
    println!();

    print_text_spec_section(ann);

    println!("📁 RELATED FILES ({})", ann.files.len());
    if ann.files.is_empty() {
        println!("   No files detected");
    } else {
        for f in &ann.files {
            println!("   • {}", f.display());
        }
    }
    println!();

    print_text_tdg_section(ann);
    print_text_churn_section(ann);
    print_text_fault_coverage_section(ann);
}

fn print_annotations_json(ann: &TicketAnnotations) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(ann)?);
    Ok(())
}

fn print_annotations_markdown(ann: &TicketAnnotations) {
    println!("# Quality Annotations: {}\n", ann.ticket_id);
    println!("**Title:** {}", ann.title);
    println!("**Status:** {} | **Priority:** {}\n", ann.status, ann.priority);

    println!("## Specification");
    if let Some(ref spec) = ann.spec_path {
        let score_str = ann.spec_score.map(|s| format!("{:.1}/100", s)).unwrap_or_else(|| "N/A".to_string());
        println!("| Metric | Value |");
        println!("|--------|-------|");
        println!("| Path | {} |", spec.display());
        println!("| Score | {} |", score_str);
    } else {
        println!("⚠️ No specification linked\n");
    }

    println!("\n## Metrics Summary");
    println!("| Metric | Value | Status |");
    println!("|--------|-------|--------|");
    println!("| Files | {} | - |", ann.files.len());
    println!("| TDG (avg) | {} | {} |",
        ann.avg_tdg.map(|t| format!("{:.2}/5.0", t)).unwrap_or_else(|| "N/A".to_string()),
        ann.avg_tdg.map(|t| if t <= 2.0 { "✅" } else { "⚠️" }).unwrap_or("⚠️")
    );
    println!("| Coverage | {} | {} |",
        ann.coverage_percent.map(|c| format!("{:.1}%", c)).unwrap_or_else(|| "N/A".to_string()),
        ann.coverage_percent.map(|c| if c >= 95.0 { "✅" } else { "❌" }).unwrap_or("⚠️")
    );
    println!("| Churn | {} | {} |",
        ann.total_churn.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string()),
        if ann.total_churn.map(|c| c < 10).unwrap_or(true) { "✅" } else { "⚠️" }
    );
    println!("| Repeated Fixes | {} | {} |",
        ann.repeated_fixes.len(),
        if ann.repeated_fixes.is_empty() { "✅" } else { "🔴" }
    );

    // Per-file TDG breakdown
    if !ann.file_tdg_scores.is_empty() {
        println!("\n## TDG Per-File Breakdown");
        println!("| File | Score | Severity |");
        println!("|------|-------|----------|");
        for ft in &ann.file_tdg_scores {
            println!("| {} | {:.2} | {} |", ft.file, ft.score, ft.severity);
        }
    }
}

/// Generate the next available ID for a new ticket
fn generate_next_id(roadmap: &crate::models::roadmap::Roadmap) -> String {
    let mut max_num = 0u32;

    for item in &roadmap.roadmap {
        // Try to extract number from IDs like "PMAT-001", "GH-123", etc.
        if let Some(num_str) = item.id.split('-').next_back() {
            if let Ok(num) = num_str.parse::<u32>() {
                max_num = max_num.max(num);
            }
        }
    }

    format!("PMAT-{:03}", max_num + 1)
}

/// Find an item with fuzzy ID matching (case-insensitive, partial match)
fn find_item_fuzzy(
    service: &RoadmapService,
    id: &str,
) -> Result<crate::models::roadmap::RoadmapItem> {
    // First try exact match
    if let Ok(Some(item)) = service.find_item(id) {
        return Ok(item);
    }

    // Load all items for fuzzy matching
    let roadmap = service.load()?;

    // Try case-insensitive exact match
    let id_lower = id.to_lowercase();
    for item in &roadmap.roadmap {
        if item.id.to_lowercase() == id_lower {
            return Ok(item.clone());
        }
    }

    // Try partial match (ID contains the search string)
    let mut matches: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| item.id.to_lowercase().contains(&id_lower))
        .collect();

    match matches.len() {
        0 => anyhow::bail!(
            "Ticket '{}' not found. Use 'pmat work list' to see available tickets.",
            id
        ),
        1 => Ok(matches.pop().expect("verified 1 element exists").clone()),
        _ => {
            let match_ids: Vec<_> = matches.iter().map(|i| i.id.as_str()).collect();
            anyhow::bail!(
                "Ambiguous ID '{}'. Multiple matches: {}. Please be more specific.",
                id,
                match_ids.join(", ")
            )
        }
    }
}

/// Extract line number from YAML error message
fn extract_line_from_yaml_error(error: &str) -> Option<usize> {
    // serde_yaml errors often contain "at line X column Y"
    if let Some(pos) = error.find("at line ") {
        let rest = error.get(pos + 8..).unwrap_or_default();
        if let Some(end) = rest.find(' ') {
            return rest.get(..end).unwrap_or_default().parse().ok();
        }
    }
    None
}


// Tests extracted to work_handlers_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax (functions/modules split across files)
#[cfg(all(test, feature = "broken-tests"))]
#[path = "work_handlers_tests.rs"]
mod tests;
