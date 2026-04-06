// File scanning: scan_rust_files_only, scan_and_analyze_files,
// and scan_and_analyze_files_persistent.

/// Optimized file scanner for dead code - only scans Rust source files
async fn scan_rust_files_only(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    const MAX_DEPTH: usize = 5; // Shallower search for performance
    const MAX_FILES: usize = 100; // Even lower limit for fast dead code analysis
    const BATCH_SIZE: usize = 20; // Smaller batches for responsiveness

    // Only scan Rust source files, skip tests and examples
    let paths: Vec<_> = WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let path = entry.path();
            if path.is_dir() || gitignore.matched(path, false).is_ignore() {
                return false;
            }
            // Only Rust files
            if !path.extension().is_some_and(|ext| ext == "rs") {
                return false;
            }
            // Skip test and example files for dead code analysis
            let path_str = path.to_string_lossy();
            !path_str.contains("/tests/")
                && !path_str.contains("/test/")
                && !path_str.contains("/examples/")
                && !path_str.contains("/benches/")
                && !path_str.contains("_test.rs")
                && !path_str.ends_with("/build.rs")
        })
        .take(MAX_FILES)
        .map(|entry| entry.path().to_path_buf())
        .collect();

    eprintln!(
        "🎯 Dead code analysis: scanning {} Rust source files (max {})",
        paths.len(),
        MAX_FILES
    );

    let mut all_results = Vec::new();
    for chunk in paths.chunks(BATCH_SIZE) {
        let batch_tasks: Vec<_> = chunk
            .iter()
            .map(|path| {
                let path = path.clone();
                let toolchain = toolchain.to_string();
                let cache_manager = cache_manager.clone();
                tokio::spawn(async move {
                    let timeout_duration = tokio::time::Duration::from_secs(2);
                    tokio::time::timeout(timeout_duration, async move {
                        analyze_file_by_toolchain(&path, &toolchain, cache_manager).await
                    })
                    .await
                    .ok()
                    .flatten()
                })
            })
            .collect();

        let batch_results = join_all(batch_tasks).await;
        all_results.extend(
            batch_results
                .into_iter()
                .filter_map(std::result::Result::ok)
                .flatten(),
        );
    }

    all_results
}

async fn scan_and_analyze_files(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<SessionCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    // FIXED: Add depth limit and file count limit to prevent hanging
    const MAX_DEPTH: usize = 10; // Prevent infinite recursion
    const MAX_FILES: usize = 10000; // Prevent resource exhaustion
    const BATCH_SIZE: usize = 100; // Process files in batches to avoid overwhelming the system

    // First, collect all file paths to analyze with limits
    let mut file_count = 0;
    let paths: Vec<_> = WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH) // TDD Fix: Limit directory traversal depth
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let path = entry.path();
            !path.is_dir() && !gitignore.matched(path, false).is_ignore()
        })
        .take(MAX_FILES) // TDD Fix: Limit total files analyzed
        .map(|entry| {
            file_count += 1;
            if file_count % 1000 == 0 {
                eprintln!("📁 Scanning files... ({file_count} so far)");
            }
            entry.path().to_path_buf()
        })
        .collect();

    if file_count > MAX_FILES / 2 {
        eprintln!(
            "⚠️ Large project detected: {file_count} files. Limited to {MAX_FILES} for performance."
        );
    }

    // Process files in controlled batches instead of all at once
    let mut all_results = Vec::new();

    for chunk in paths.chunks(BATCH_SIZE) {
        let batch_tasks: Vec<_> = chunk
            .iter()
            .map(|path| {
                let path = path.clone();
                let toolchain = toolchain.to_string();
                let cache_manager = cache_manager.clone();
                tokio::spawn(async move {
                    // Add timeout for individual file analysis
                    let timeout_duration = tokio::time::Duration::from_secs(5);
                    tokio::time::timeout(timeout_duration, async move {
                        analyze_file_by_toolchain(&path, &toolchain, cache_manager).await
                    })
                    .await
                    .ok()
                    .flatten()
                })
            })
            .collect();

        // Wait for this batch to complete before starting the next
        let batch_results = join_all(batch_tasks).await;
        all_results.extend(
            batch_results
                .into_iter()
                .filter_map(std::result::Result::ok)
                .flatten(),
        );
    }

    all_results
}

async fn scan_and_analyze_files_persistent(
    root_path: &Path,
    toolchain: &str,
    cache_manager: Option<Arc<PersistentCacheManager>>,
    gitignore: &ignore::gitignore::Gitignore,
) -> Vec<FileContext> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    // FIXED: Add same depth and file limits as non-persistent version
    const MAX_DEPTH: usize = 10; // Prevent infinite recursion
    const MAX_FILES: usize = 10000; // Prevent resource exhaustion

    let mut files = Vec::new();
    let mut file_count = 0;

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .max_depth(MAX_DEPTH) // TDD Fix: Limit directory traversal depth
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();

        // Skip if gitignored
        if gitignore.matched(path, path.is_dir()).is_ignore() {
            continue;
        }

        // TDD Fix: Limit total files analyzed
        file_count += 1;
        if file_count > MAX_FILES {
            eprintln!("⚠️ Reached file limit of {MAX_FILES}. Stopping analysis.");
            break;
        }

        if file_count % 1000 == 0 {
            eprintln!("📁 Scanning files... ({file_count} so far)");
        }

        // Add timeout for individual file analysis
        let timeout_duration = tokio::time::Duration::from_secs(5);
        let result = tokio::time::timeout(timeout_duration, async {
            analyze_file_by_toolchain_persistent(path, toolchain, cache_manager.clone()).await
        })
        .await;

        if let Ok(Some(file_context)) = result {
            files.push(file_context);
        }
    }

    files
}
