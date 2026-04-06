/// Handle `--score-diagnosis` mode: map composite sub-score breakdown to code locations.
///
/// Reads persisted composite from .pmat-metrics/commit-<sha>-meta.json,
/// then uses the function index to identify which functions drag each sub-score down.
pub(super) fn handle_score_diagnosis_mode(
    index: &crate::services::agent_context::AgentContextIndex,
    project_path: &std::path::Path,
    limit: usize,
) -> anyhow::Result<()> {
    use crate::cli::handlers::score_handler::CompositeScore;

    // 1. Read persisted composite score
    let sha = get_head_sha(project_path);
    let meta_path = project_path
        .join(".pmat-metrics")
        .join(format!("commit-{sha}-meta.json"));

    let score: CompositeScore = if meta_path.exists() {
        let content = std::fs::read_to_string(&meta_path)?;
        serde_json::from_str(&content)?
    } else {
        anyhow::bail!(
            "No composite score found. Run `pmat score` first to generate .pmat-metrics/commit-{sha}-meta.json"
        );
    };

    let limit = limit.min(10);

    // 2. Print composite header
    println!(
        "COMPOSITE: {:.1}/100  Grade: {}\n",
        score.composite, score.grade
    );

    // 3. Find weakest sub-scores (below 70)
    let sub = &score.sub_scores;
    let mut weak: Vec<(&str, f64)> = vec![
        ("RPS", sub.rps),
        ("Comply", sub.comply),
        ("Coverage", sub.coverage),
        ("Muda (inv)", sub.muda_inv),
        ("EvoScore", sub.evoscore),
        ("DBC", sub.dbc),
        ("File Health", sub.file_health),
    ];
    weak.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // 4. For each weak sub-score, map to code locations
    for (name, value) in &weak {
        if *value >= 80.0 {
            continue; // Only diagnose sub-scores below 80
        }
        println!("Dragging {} ({:.1}):", name, value);
        match *name {
            "RPS" => diagnose_rps(&score.rps_categories, limit),
            "Comply" => diagnose_comply(score.comply_errors, score.comply_warnings),
            "Muda (inv)" => diagnose_muda(project_path, limit),
            "File Health" => diagnose_file_health(project_path, limit),
            "Coverage" => diagnose_coverage(index, limit),
            "DBC" => println!("  Run `pmat work codebase-score` for DBC breakdown\n"),
            "EvoScore" => println!("  Run `pmat test --record` to build EvoScore history\n"),
            _ => {}
        }
    }

    Ok(())
}

fn diagnose_rps(categories: &std::collections::HashMap<String, f64>, limit: usize) {
    let mut cats: Vec<_> = categories.iter().collect();
    cats.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));
    for (name, pct) in cats.iter().take(limit) {
        if **pct < 70.0 {
            println!("  {}: {:.0}%", name, pct);
        }
    }
    println!();
}

fn diagnose_comply(errors: usize, warnings: usize) {
    println!("  {} errors, {} warnings", errors, warnings);
    println!("  Run `pmat comply check --failures-only` for details\n");
}

fn diagnose_muda(project_path: &std::path::Path, limit: usize) {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::handlers::comply_handlers::muda_handlers;
    let report = muda_handlers::calculate_muda_score(project_path);
    let categories = [
        ("Overproduction", report.overproduction),
        ("Waiting", report.waiting),
        ("Inventory", report.inventory),
        ("Over-processing", report.over_processing),
        ("Defects", report.defects),
    ];
    for (name, val) in &categories {
        if *val > 20.0 {
            println!("  {}: {:.0}", name, val);
        }
    }
    // Show top files from file_details
    for (category, files) in &report.file_details {
        if files.is_empty() {
            continue;
        }
        let top: Vec<&str> = files.iter().take(limit).map(|s| s.as_str()).collect();
        println!("  {} files: {}", category, top.join(", "));
    }
    println!();
}

fn diagnose_file_health(project_path: &std::path::Path, limit: usize) {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let check = crate::cli::handlers::comply_handlers::check_file_health(project_path);
    // Parse message for file count
    if let Some(pos) = check.message.find("files >") {
        println!("  {}", &check.message[..pos + 20].trim());
    }
    // List largest files from src/
    let src = project_path.join("src");
    if src.exists() {
        let mut big_files: Vec<(String, usize)> = Vec::new();
        for entry in walkdir::WalkDir::new(&src)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let lines = content.lines().count();
                if lines > 500 {
                    let rel = entry
                        .path()
                        .strip_prefix(project_path)
                        .unwrap_or(entry.path());
                    big_files.push((rel.display().to_string(), lines));
                }
            }
        }
        big_files.sort_by(|a, b| b.1.cmp(&a.1));
        for (path, lines) in big_files.iter().take(limit) {
            println!("  {}: {} lines", path, lines);
        }
    }
    println!();
}

fn diagnose_coverage(
    index: &crate::services::agent_context::AgentContextIndex,
    limit: usize,
) {
    // Show functions with lowest coverage from the index
    let funcs = &index.functions;
    let mut uncovered: Vec<_> = funcs
        .iter()
        .filter(|f| {
            f.quality.tdg_grade == "F" || f.quality.tdg_grade == "D"
        })
        .take(limit * 2)
        .collect();
    uncovered.truncate(limit);
    if uncovered.is_empty() {
        println!("  Run `pmat query --coverage-gaps` for detailed coverage analysis\n");
    } else {
        for f in &uncovered {
            println!(
                "  {}:{} {} (grade {})",
                f.file_path,
                f.start_line,
                f.function_name,
                f.quality.tdg_grade
            );
        }
        println!();
    }
}

fn get_head_sha(path: &std::path::Path) -> String {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
