// Score handler display, trend, and stack quality functions.
// Extracted for file health compliance (CB-040).

/// CB-150: Print sovereign stack dependency quality.
fn print_stack_quality(path: &Path) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let cargo_toml = path.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return,
    };

    let sovereign = [
        "aprender", "trueno", "trueno-graph", "trueno-db", "trueno-rag",
        "trueno-viz", "trueno-zram-core", "pmcp", "renacer", "certeza",
        "bashrs", "probar", "presentar-core", "ruchy",
    ];

    let mut found = Vec::new();
    for dep in &sovereign {
        if content.contains(&format!("{dep} ")) || content.contains(&format!("{dep}\"")) {
            let local_path = path.join(format!("../{dep}"));
            let has_score = local_path.join(".pmat-metrics").exists();
            found.push((*dep, local_path.exists(), has_score));
        }
    }

    if found.is_empty() {
        return;
    }

    println!("\nStack Quality (CB-150):");
    for (name, has_local, has_score) in &found {
        let status = if *has_score {
            let metrics = path.join(format!("../{name}/.pmat-metrics"));
            let score = read_latest_composite(&metrics);
            match score {
                Some(s) => format!("{:.0}/100 ({})", s.composite, s.grade),
                None => "no composite".to_string(),
            }
        } else if *has_local {
            "local (no score)".to_string()
        } else {
            "crates.io".to_string()
        };
        println!("  {name:20} {status}");
    }
}

fn read_latest_composite(metrics_dir: &Path) -> Option<CompositeScore> {
    debug_assert!(metrics_dir.exists(), "metrics_dir must exist: {}", metrics_dir.display());
    let mut latest: Option<CompositeScore> = None;
    if let Ok(entries) = std::fs::read_dir(metrics_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("commit-") && name_str.ends_with("-meta.json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(score) = serde_json::from_str::<CompositeScore>(&content) {
                        if score.composite > 0.0
                            && latest.as_ref().map_or(true, |l| score.timestamp > l.timestamp) {
                                latest = Some(score);
                            }
                    }
                }
            }
        }
    }
    latest
}

/// Load historical composite scores from .pmat-metrics/commit-*-meta.json.
fn load_score_history(path: &Path) -> Vec<CompositeScore> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let metrics_dir = path.join(".pmat-metrics");
    let mut scores = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&metrics_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("commit-") && name_str.ends_with("-meta.json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(score) = serde_json::from_str::<CompositeScore>(&content) {
                        if score.composite > 0.0 {
                            scores.push(score);
                        }
                    }
                }
            }
        }
    }
    scores.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    scores
}

/// Check regression against previous commit score. Returns delta (negative = worse).
fn check_regression(path: &Path, current: &CompositeScore) -> Option<f64> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let history = load_score_history(path);
    let previous = history.iter().rev().find(|s| s.sha != current.sha)?;
    Some(current.composite - previous.composite)
}

/// Print sparkline trend of composite scores (CB-145).
fn print_trend(path: &Path) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let history = load_score_history(path);
    if history.is_empty() {
        println!("No score history found. Run `pmat score` to generate data.");
        return;
    }

    println!("Score Trend ({} commits):\n", history.len());
    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let composites: Vec<f64> = history.iter().map(|s| s.composite).collect();
    let min = composites.iter().cloned().fold(f64::MAX, f64::min);
    let max = composites.iter().cloned().fold(f64::MIN, f64::max);
    let range = (max - min).max(1.0);

    print!("  ");
    for &c in &composites {
        let idx = (((c - min) / range) * 7.0) as usize;
        print!("{}", blocks[idx.min(7)]);
    }
    println!();

    println!(
        "  Range: {:.1} - {:.1}  Current: {:.1} ({})",
        min,
        max,
        composites.last().unwrap_or(&0.0),
        history.last().map(|s| s.grade.as_str()).unwrap_or("?")
    );
}

fn format_text(score: &CompositeScore) -> String {
    debug_assert!(true, "contract: format_text");
    let mut out = String::new();
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("PMAT Unified Score\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    out.push_str(&format!(
        "  Composite: {:.1}/100  Grade: {}\n\n",
        score.composite, score.grade
    ));
    out.push_str("Sub-Scores\n");
    out.push_str(&format!("  RPS:         {:.1}\n", score.sub_scores.rps));
    out.push_str(&format!("  Comply:      {:.1}  ({} errors, {} warnings)\n",
        score.sub_scores.comply, score.comply_errors, score.comply_warnings));
    out.push_str(&format!("  Coverage:    {:.1}\n", score.sub_scores.coverage));
    out.push_str(&format!("  Muda (inv):  {:.1}\n", score.sub_scores.muda_inv));
    out.push_str(&format!("  EvoScore:    {:.1}\n", score.sub_scores.evoscore));
    out.push_str(&format!("  DBC:         {:.1}\n", score.sub_scores.dbc));
    out.push_str(&format!("  File Health: {:.1}\n", score.sub_scores.file_health));
    out.push_str(&format!("  PV Lint:     {:.1}\n", score.sub_scores.pv_lint));
    out
}
