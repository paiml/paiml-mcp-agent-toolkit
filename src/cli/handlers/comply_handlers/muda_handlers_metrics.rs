/// Transport waste: excessive data copying (.clone() density)
fn measure_transport(project_path: &Path) -> f64 {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return 0.0;
    }

    let mut total_lines = 0usize;
    let mut clone_calls = 0usize;

    if let Ok(entries) = walkdir::WalkDir::new(&src_dir)
        .max_depth(5)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries.iter().filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        }) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
                clone_calls += content.matches(".clone()").count();
            }
        }
    }

    if total_lines == 0 {
        return 0.0;
    }

    // Clone density: clones per 100 lines
    let density = (clone_calls as f64 / total_lines as f64) * 100.0;
    // Scale: <0.5 clones/100 lines = 0 waste, >5 = 100 waste
    ((density - 0.5) / 4.5 * 100.0).clamp(0.0, 100.0)
}

/// Over-processing waste: cyclomatic complexity
fn measure_over_processing(project_path: &Path) -> f64 {
    // Check cached complexity metrics
    let metrics_path = project_path.join(".pmat/hooks-cache/metrics.json");
    if let Ok(content) = std::fs::read_to_string(&metrics_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(max_cc) = json
                .get("complexity")
                .and_then(|v| v.get("max_cyclomatic"))
                .and_then(|v| v.as_f64())
            {
                // Scale: <10 = 0 waste, >50 = 100 waste
                return ((max_cc - 10.0) / 40.0 * 100.0).clamp(0.0, 100.0);
            }
        }
    }
    20.0 // Default: assume moderate complexity
}

/// Motion waste: dependency sprawl
fn measure_motion(project_path: &Path) -> f64 {
    let cargo_lock = project_path.join("Cargo.lock");
    if !cargo_lock.exists() {
        return 0.0;
    }

    // Count dependency packages in Cargo.lock
    if let Ok(content) = std::fs::read_to_string(&cargo_lock) {
        let dep_count = content.matches("[[package]]").count();
        // Scale: <50 deps = 0 waste, >500 deps = 100 waste
        return ((dep_count as f64 - 50.0) / 450.0 * 100.0).clamp(0.0, 100.0);
    }
    0.0
}

/// Defects waste: stub/panic indicators
fn measure_defects(project_path: &Path) -> f64 {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return 0.0;
    }

    let mut stub_count = 0usize;
    let mut unwrap_count = 0usize;
    let mut total_lines = 0usize;

    if let Ok(entries) = walkdir::WalkDir::new(&src_dir)
        .max_depth(5)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries.iter().filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        }) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
                stub_count += content.matches("todo!()").count();
                stub_count += content.matches("unimplemented!()").count();
                unwrap_count += content.matches(".unwrap()").count();
            }
        }
    }

    // Size-normalized defect density (per 1000 lines)
    let kloc = (total_lines as f64 / 1000.0).max(1.0);
    let stub_density = stub_count as f64 / kloc;
    let unwrap_density = unwrap_count as f64 / kloc;
    let stub_score = (stub_density * 5.0).min(50.0);
    let unwrap_score = (unwrap_density * 2.0).min(50.0);
    (stub_score + unwrap_score).clamp(0.0, 100.0)
}

/// Collect top files with high complexity (cyclomatic > 15).
/// Scans source files and estimates per-file max complexity using heuristics.
/// Returns up to 5 file paths sorted by estimated complexity descending.
fn collect_over_processing_files(project_path: &Path) -> Vec<String> {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return Vec::new();
    }

    let entries = match walkdir::WalkDir::new(&src_dir)
        .max_depth(5)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let complexity_threshold = 15u32;
    let mut file_complexities: Vec<(String, u32)> = entries
        .iter()
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        })
        .filter_map(|entry| {
            let content = std::fs::read_to_string(entry.path()).ok()?;
            let max_cc = estimate_max_complexity(&content);
            if max_cc > complexity_threshold {
                let rel = entry
                    .path()
                    .strip_prefix(project_path)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                Some((rel, max_cc))
            } else {
                None
            }
        })
        .collect();

    file_complexities.sort_by_key(|b| std::cmp::Reverse(b.1));
    file_complexities
        .into_iter()
        .take(5)
        .map(|(p, cc)| format!("{} (cc={})", p, cc))
        .collect()
}

/// Estimate the maximum cyclomatic complexity of any function in the content.
/// Uses a lightweight heuristic: counts branching keywords per function.
fn estimate_max_complexity(content: &str) -> u32 {
    let mut max_cc = 1u32;
    let mut current_cc = 1u32;
    let mut in_function = false;
    let mut brace_depth = 0u32;
    let mut fn_brace_depth = 0u32;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect function start
        if (trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
            || trimmed.starts_with("pub(crate) fn ")
            || trimmed.starts_with("pub(crate) async fn "))
            && !in_function
        {
            in_function = true;
            current_cc = 1;
            fn_brace_depth = brace_depth;
        }

        // Track brace depth
        for ch in trimmed.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth = brace_depth.saturating_sub(1);
                    if in_function && brace_depth == fn_brace_depth {
                        // Function ended
                        max_cc = max_cc.max(current_cc);
                        in_function = false;
                    }
                }
                _ => {}
            }
        }

        // Count complexity contributors inside functions
        if in_function {
            if trimmed.starts_with("if ")
                || trimmed.contains(" if ")
                || trimmed.starts_with("else if ")
            {
                current_cc += 1;
            }
            if trimmed.starts_with("match ") || trimmed.contains(" match ") {
                current_cc += 1;
            }
            if trimmed.starts_with("for ") || trimmed.contains(" for ") {
                current_cc += 1;
            }
            if trimmed.starts_with("while ") || trimmed.contains(" while ") {
                current_cc += 1;
            }
            if trimmed.contains("&&") || trimmed.contains("||") {
                current_cc += 1;
            }
        }
    }

    // Capture last function if file ends without closing brace
    if in_function {
        max_cc = max_cc.max(current_cc);
    }

    max_cc
}

/// Collect top files with defect indicators (panics, unwraps, stubs).
/// Returns up to 5 file paths sorted by defect score descending.
fn collect_defect_files(project_path: &Path) -> Vec<String> {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return Vec::new();
    }

    let entries = match walkdir::WalkDir::new(&src_dir)
        .max_depth(5)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut file_defects: Vec<(String, usize)> = entries
        .iter()
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        })
        .filter_map(|entry| {
            let content = std::fs::read_to_string(entry.path()).ok()?;
            let stubs = content.matches("todo!()").count()
                + content.matches("unimplemented!()").count();
            let unwraps = content.matches(".unwrap()").count();
            let total = stubs * 10 + unwraps; // Same weighting as measure_defects
            if total > 0 {
                let rel = entry
                    .path()
                    .strip_prefix(project_path)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                Some((rel, total))
            } else {
                None
            }
        })
        .collect();

    file_defects.sort_by_key(|b| std::cmp::Reverse(b.1));
    file_defects
        .into_iter()
        .take(5)
        .map(|(p, d)| {
            if d >= 10 {
                format!("{} ({} defect pts)", p, d)
            } else {
                format!("{} ({} unwrap)", p, d)
            }
        })
        .collect()
}
