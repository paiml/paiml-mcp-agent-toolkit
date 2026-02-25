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
                stub_count += content.matches("todo!()").count();
                stub_count += content.matches("unimplemented!()").count();
                unwrap_count += content.matches(".unwrap()").count();
            }
        }
    }

    // Stubs are critical (10 points each), unwraps are moderate (1 point each)
    let score = (stub_count as f64 * 10.0) + (unwrap_count as f64 * 0.5);
    score.clamp(0.0, 100.0)
}
