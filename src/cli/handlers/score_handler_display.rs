// Score handler display, trend, and stack quality functions.
// Extracted for file health compliance (CB-040).

/// CB-150: Print sovereign stack dependency quality.
fn print_stack_quality(path: &Path) {
    let cargo_toml = path.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return,
    };

    let sovereign = [
        "aprender",
        "trueno",
        "trueno-graph",
        "trueno-db",
        "trueno-rag",
        "trueno-viz",
        "trueno-zram-core",
        "pmcp",
        "renacer",
        "certeza",
        "bashrs",
        "probar",
        "presentar-core",
        "ruchy",
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
            match score.and_then(|s| s.composite.map(|c| (c, s.grade))) {
                Some((composite, grade)) => format!("{composite:.0}/100 ({grade})"),
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
    let mut latest: Option<CompositeScore> = None;
    if let Ok(entries) = std::fs::read_dir(metrics_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("commit-") && name_str.ends_with("-meta.json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(score) = serde_json::from_str::<CompositeScore>(&content) {
                        if score.composite.is_some_and(|c| c > 0.0)
                            && latest
                                .as_ref()
                                .is_none_or(|l| score.timestamp > l.timestamp)
                        {
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
    let metrics_dir = path.join(".pmat-metrics");
    let mut scores = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&metrics_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("commit-") && name_str.ends_with("-meta.json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(score) = serde_json::from_str::<CompositeScore>(&content) {
                        if score.composite.is_some_and(|c| c > 0.0) {
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
    let current_composite = current.composite?;
    let history = load_score_history(path);
    let previous = history.iter().rev().find(|s| s.sha != current.sha)?;
    Some(current_composite - previous.composite?)
}

/// Print sparkline trend of composite scores (CB-145).
fn print_trend(path: &Path) {
    let history = load_score_history(path);
    if history.is_empty() {
        println!("No score history found. Run `pmat score` to generate data.");
        return;
    }

    println!("Score Trend ({} commits):\n", history.len());
    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let composites: Vec<f64> = history.iter().filter_map(|s| s.composite).collect();
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

/// `--format text` is documented as "Text format with colored output
/// (default)", but this renderer emitted no escape at all: `pmat score --color
/// always` was byte-identical to `--color never`, so the flag named a
/// behaviour the printer did not have. Colour now comes from the shared
/// helpers, which means `always` forces it through a pipe and `never` (and a
/// redirected stdout) still produce plain, diffable text.
fn format_text(score: &CompositeScore) -> String {
    use crate::cli::colors as c;

    let mut out = String::new();
    out.push_str(&format!("{}\n", c::rule()));
    out.push_str(&format!("{}\n", c::subheader("PMAT Unified Score")));
    out.push_str(&format!("{}\n\n", c::rule()));
    let composite = match score.composite {
        Some(v) => format!("{}/100", tinted_score(v)),
        None => c::dim("not measured"),
    };
    out.push_str(&format!(
        "  {} {}  {} {}  {} {}/{}\n\n",
        c::label("Composite:"),
        composite,
        c::label("Grade:"),
        c::grade(&score.grade),
        c::label("Dimensions:"),
        score.dimensions_measured,
        score.dimensions_total
    ));
    out.push_str(&format!("{}\n", c::subheader("Sub-Scores")));
    out.push_str(&sub_score_line("RPS:", score.sub_scores.rps));
    out.push_str(&format!(
        "  {:<12} {}{}\n",
        "Comply:",
        optional_score(score.sub_scores.comply),
        match (score.comply_errors, score.comply_warnings) {
            // Counts only when they were actually counted. This used to print
            // "not measured  (0 errors, 0 warnings)" whenever `pmat` was not on
            // PATH — a clean bill of health for a check that never ran.
            (Some(e), Some(w)) => format!("  ({e} errors, {w} warnings)"),
            _ => String::new(),
        }
    ));
    out.push_str(&sub_score_line("Coverage:", score.sub_scores.coverage));
    out.push_str(&sub_score_line("Muda (inv):", score.sub_scores.muda_inv));
    out.push_str(&sub_score_line("EvoScore:", score.sub_scores.evoscore));
    out.push_str(&sub_score_line("DBC:", score.sub_scores.dbc));
    out.push_str(&sub_score_line(
        "File Health:",
        score.sub_scores.file_health,
    ));
    out.push_str(&sub_score_line("PV Lint:", score.sub_scores.pv_lint));

    // A red gate changes what the composite MEANS: it is then the mean over the
    // dimensions that are not gating, so the number describes the project while
    // the verdict describes the gate. Printing the number without saying so
    // would be a third meaning for one field (#983).
    if !score.gated_by.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "{}\n",
            c::subheader("Gated (verdict, not average)")
        ));
        for dim in &score.gated_by {
            out.push_str(&format!(
                "  {:<12} {}\n",
                dim,
                c::dim("measured 0.0 — a red gate, excluded from the composite below")
            ));
        }
        out.push_str(&format!(
            "  {}\n",
            c::dim("Composite above is the mean of the non-gating dimensions.")
        ));
    }

    // Disclose what the composite does not cover. Without this the reader sees
    // a single number and cannot tell which dimensions went into it.
    if !score.not_measured.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "{}\n",
            c::subheader("Not Measured (excluded from the composite)")
        ));
        for n in &score.not_measured {
            out.push_str(&format!("  {:<12} {}\n", n.dimension, c::dim(&n.reason)));
        }
    }
    out
}

/// A 0-100 score, green ≥90 / yellow ≥70 / red below — plain text when colour
/// is off.
fn tinted_score(value: f64) -> String {
    use crate::cli::colors as c;
    c::colored(
        c::threshold_color(value, 90.0, 70.0),
        &format!("{value:.1}"),
    )
}

/// A sub-score that may not have been measured. "not measured" is never
/// tinted like a score, because it is not one.
fn optional_score(value: Option<f64>) -> String {
    use crate::cli::colors as c;
    match value {
        Some(v) => tinted_score(v),
        None => c::dim("not measured"),
    }
}

fn sub_score_line(label: &str, value: Option<f64>) -> String {
    format!("  {:<12} {}\n", label, optional_score(value))
}
