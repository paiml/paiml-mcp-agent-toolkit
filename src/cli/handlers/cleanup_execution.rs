// Cleanup execution and helper functions
// Included by cleanup_resources_handler.rs

/// Execute cleanup operations
fn execute_cleanup(result: &mut CleanupResult) -> Result<()> {
    debug_assert!(true, "contract: execute_cleanup");
    let candidates: Vec<_> = result
        .candidates
        .iter()
        .map(|c| (c.path.clone(), c.size_bytes, c.category.clone()))
        .collect();
    for (path, size_bytes, category) in &candidates {
        match category.as_str() {
            "rust" | "node" => cleanup_directory(path, *size_bytes, result),
            "git" => cleanup_git(path, result),
            "logs" => cleanup_file(path, *size_bytes, result),
            _ => {}
        }
    }
    if !result.errors.is_empty() {
        println!();
        println!("⚠️  Errors:");
        for error in &result.errors {
            println!("   {}", error);
        }
    }
    Ok(())
}

fn cleanup_directory(path: &Path, size_bytes: u64, result: &mut CleanupResult) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if !path.is_dir() {
        return;
    }
    match std::fs::remove_dir_all(path) {
        Ok(_) => {
            result.items_cleaned += 1;
            result.space_freed_bytes += size_bytes;
            println!("   ✓ Removed: {}", path.display());
        }
        Err(e) => result
            .errors
            .push(format!("Failed to remove {}: {}", path.display(), e)),
    }
}

fn cleanup_git(path: &Path, result: &mut CleanupResult) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let Some(repo_path) = path.parent().and_then(|p| p.parent()) else {
        return;
    };
    match std::process::Command::new("git")
        .args(["gc", "--aggressive"])
        .current_dir(repo_path)
        .output()
    {
        Ok(o) if o.status.success() => {
            result.items_cleaned += 1;
            println!("   ✓ Git gc: {}", repo_path.display());
        }
        Ok(o) => result.errors.push(format!(
            "Git gc failed: {}",
            String::from_utf8_lossy(&o.stderr)
        )),
        Err(e) => result.errors.push(format!("Git gc error: {}", e)),
    }
}

fn cleanup_file(path: &Path, size_bytes: u64, result: &mut CleanupResult) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if !path.is_file() {
        return;
    }
    match std::fs::remove_file(path) {
        Ok(_) => {
            result.items_cleaned += 1;
            result.space_freed_bytes += size_bytes;
            println!("   ✓ Removed: {}", path.display());
        }
        Err(e) => result
            .errors
            .push(format!("Failed to remove {}: {}", path.display(), e)),
    }
}

// Helper functions

fn is_hidden(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.') && n != ".git")
        .unwrap_or(false)
}

fn is_excluded(path: &Path, exclude: &[String]) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let path_str = path.to_string_lossy();
    exclude.iter().any(|pattern| {
        if pattern.contains('*') {
            // Simple glob matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                path_str.starts_with(parts[0]) && path_str.ends_with(parts[1])
            } else {
                path_str.contains(pattern.trim_matches('*'))
            }
        } else {
            path_str.contains(pattern)
        }
    })
}

fn calculate_dir_size(path: &Path) -> u64 {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn count_loose_objects(objects_dir: &Path) -> usize {
    debug_assert!(objects_dir.exists(), "objects_dir must exist: {}", objects_dir.display());
    WalkDir::new(objects_dir)
        .max_depth(2)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|n| n.len() == 2 && n.chars().all(|c| c.is_ascii_hexdigit()))
                .unwrap_or(false)
        })
        .count()
}
