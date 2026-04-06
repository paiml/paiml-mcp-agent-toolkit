// =============================================================================
// Rust-specific helpers (existing cargo-based analysis)
// =============================================================================

fn analyze_rust_dead_code_with_cargo(path: &Path) -> Result<Vec<DeadFunction>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Simple regex-based analyzer (similar to C/Python)
    // In future, could integrate cargo check warnings

    let rust_files = find_files_by_extension(path, &["rs"]);
    let (defined_functions, called_functions) = analyze_rust_files(&rust_files)?;
    let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

    Ok(dead_functions)
}

fn count_rust_functions(path: &Path) -> Result<usize> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let rust_files = find_files_by_extension(path, &["rs"]);
    let (defined_functions, _) = analyze_rust_files(&rust_files)?;
    Ok(defined_functions.len())
}

/// Analyze Rust files
fn analyze_rust_files(
    files: &[std::path::PathBuf],
) -> Result<(Vec<FunctionInfo>, HashSet<String>)> {
    debug_assert!(!files.is_empty(), "files must not be empty");
    let mut defined_functions = Vec::new();
    let mut called_functions = HashSet::new();

    for file in files {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {:?}", file))?;

        // Find function definitions: fn function_name(
        for (line_idx, line) in content.lines().enumerate() {
            if let Some(cap) = RUST_DEF_REGEX.captures(line) {
                if let Some(func_name_match) = cap.get(1) {
                    let func_name = func_name_match.as_str().to_string();

                    // Skip main and test functions
                    if func_name != "main" && !func_name.starts_with("test_") {
                        defined_functions.push(FunctionInfo {
                            name: func_name,
                            file: file.display().to_string(),
                            line: line_idx + 1,
                        });
                    }
                }
            }
        }

        // Find function calls
        for line in content.lines() {
            // Skip function definitions
            if RUST_FN_DEF_REGEX.is_match(line) {
                continue;
            }

            for cap in RUST_CALL_REGEX.captures_iter(line) {
                if let Some(func_name_match) = cap.get(1) {
                    let func_name = func_name_match.as_str().to_string();
                    // Filter out Rust keywords
                    if ![
                        "if", "while", "for", "match", "return", "let", "mut", "use", "mod", "fn",
                        "println", "vec", "Some", "None", "Ok", "Err",
                    ]
                    .contains(&func_name.as_str())
                    {
                        called_functions.insert(func_name);
                    }
                }
            }
        }
    }

    debug!(
        "Found {} defined Rust functions, {} unique calls",
        defined_functions.len(),
        called_functions.len()
    );

    Ok((defined_functions, called_functions))
}
