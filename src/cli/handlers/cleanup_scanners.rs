// Cleanup resource scanner functions
// Included by cleanup_resources_handler.rs

/// Scan for Rust target directories
fn scan_rust_targets(
    project_dir: &Path,
    exclude: &[String],
    _min_age_days: u32,
    result: &mut CleanupResult,
) -> Result<()> {
    println!("🦀 Scanning Rust target directories...");

    for entry in WalkDir::new(project_dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.path()) && !is_excluded(e.path(), exclude))
        .flatten()
    {
        let path = entry.path();

        // Look for target directories with Cargo.toml sibling
        if path.is_dir() && path.file_name().is_some_and(|n| n == "target") {
            let parent = path.parent();
            if parent.is_some_and(|p| p.join("Cargo.toml").exists()) {
                let size = calculate_dir_size(path);
                result.candidates.push(CleanupCandidate {
                    path: path.to_path_buf(),
                    size_bytes: size,
                    category: "rust".to_string(),
                    description: "Rust build artifacts".to_string(),
                    age_days: 0,
                });
                result.total_size_bytes += size;
                result.items_found += 1;
            }
        }
    }

    println!(
        "   Found {} Rust target directories ({} MB)",
        result
            .candidates
            .iter()
            .filter(|c| c.category == "rust")
            .count(),
        result
            .candidates
            .iter()
            .filter(|c| c.category == "rust")
            .map(|c| c.size_bytes)
            .sum::<u64>()
            / (1024 * 1024)
    );

    Ok(())
}

/// Scan for Node.js node_modules directories
fn scan_node_targets(
    project_dir: &Path,
    exclude: &[String],
    _min_age_days: u32,
    result: &mut CleanupResult,
) -> Result<()> {
    println!("📦 Scanning Node.js node_modules...");

    let mut node_count = 0;
    let mut node_size: u64 = 0;

    for entry in WalkDir::new(project_dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            !is_hidden(path)
                && !is_excluded(path, exclude)
                && path
                    .file_name()
                    .map(|n| n != "node_modules")
                    .unwrap_or(true)
        })
        .flatten()
    {
        let path = entry.path();

        // Look for node_modules directories with package.json sibling
        if path.is_dir() && path.file_name().is_some_and(|n| n == "node_modules") {
            let parent = path.parent();
            if parent.is_some_and(|p| p.join("package.json").exists()) {
                let size = calculate_dir_size(path);
                result.candidates.push(CleanupCandidate {
                    path: path.to_path_buf(),
                    size_bytes: size,
                    category: "node".to_string(),
                    description: "Node.js dependencies".to_string(),
                    age_days: 0,
                });
                result.total_size_bytes += size;
                result.items_found += 1;
                node_count += 1;
                node_size += size;
            }
        }
    }

    println!(
        "   Found {} node_modules directories ({} MB)",
        node_count,
        node_size / (1024 * 1024)
    );

    Ok(())
}

/// Scan for Git garbage collection opportunities
fn scan_git_targets(project_dir: &Path, result: &mut CleanupResult) -> Result<()> {
    println!("📚 Scanning Git repositories...");

    let git_dir = project_dir.join(".git");
    if git_dir.exists() {
        // Check loose objects
        let objects_dir = git_dir.join("objects");
        if objects_dir.exists() {
            let loose_count = count_loose_objects(&objects_dir);
            if loose_count > 100 {
                result.candidates.push(CleanupCandidate {
                    path: objects_dir.clone(),
                    size_bytes: 0, // Git gc will compact, not delete
                    category: "git".to_string(),
                    description: format!("{} loose objects (run git gc)", loose_count),
                    age_days: 0,
                });
                result.items_found += 1;
            }
        }
    }

    println!(
        "   Found {} Git optimization opportunities",
        result
            .candidates
            .iter()
            .filter(|c| c.category == "git")
            .count()
    );

    Ok(())
}

/// Scan for log files
fn scan_log_targets(
    project_dir: &Path,
    exclude: &[String],
    min_age_days: u32,
    result: &mut CleanupResult,
) -> Result<()> {
    println!("📝 Scanning log files...");

    let mut log_count = 0;
    let mut log_size: u64 = 0;

    let log_files = WalkDir::new(project_dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.path()) && !is_excluded(e.path(), exclude))
        .flatten()
        .filter(|e| {
            e.path().is_file() && e.path().extension().and_then(|e| e.to_str()) == Some("log")
        })
        .filter(|e| is_old_enough(e.path(), min_age_days));

    for entry in log_files {
        let path = entry.path();
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        result.candidates.push(CleanupCandidate {
            path: path.to_path_buf(),
            size_bytes: size,
            category: "logs".to_string(),
            description: "Log file".to_string(),
            age_days: 0,
        });
        result.total_size_bytes += size;
        result.items_found += 1;
        log_count += 1;
        log_size += size;
    }

    println!(
        "   Found {} log files ({} MB)",
        log_count,
        log_size / (1024 * 1024)
    );
    Ok(())
}

fn is_old_enough(path: &Path, min_age_days: u32) -> bool {
    if min_age_days == 0 {
        return true;
    }
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|modified| {
            modified.elapsed().unwrap_or_default().as_secs() / 86400 >= min_age_days as u64
        })
        .unwrap_or(true)
}

/// Print cleanup results
fn print_results(result: &CleanupResult, format: OutputFormat) -> Result<()> {
    println!();
    println!("📊 Cleanup Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "items_found": result.items_found,
                "total_size_mb": result.total_size_bytes / (1024 * 1024),
                "candidates": result.candidates.iter().map(|c| {
                    serde_json::json!({
                        "path": c.path.display().to_string(),
                        "size_mb": c.size_bytes / (1024 * 1024),
                        "category": c.category,
                        "description": c.description
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!("   Items found:  {}", result.items_found);
            println!(
                "   Total size:   {} MB",
                result.total_size_bytes / (1024 * 1024)
            );
            println!();

            if !result.candidates.is_empty() {
                println!("📁 Candidates:");
                for candidate in result.candidates.iter().take(20) {
                    println!(
                        "   [{:6}] {:>8} MB  {}",
                        candidate.category,
                        candidate.size_bytes / (1024 * 1024),
                        candidate.path.display()
                    );
                }
                if result.candidates.len() > 20 {
                    println!("   ... and {} more", result.candidates.len() - 20);
                }
            }
        }
    }

    Ok(())
}
