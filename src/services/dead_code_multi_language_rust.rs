// =============================================================================
// Rust-specific helpers
// =============================================================================
//
// There was an `analyze_rust_dead_code_with_cargo` here, and a
// `count_rust_functions_and_files` beside it that walked and re-parsed the same
// tree a second time to recover a count the first walk had already produced.
// Neither ran cargo — the name was left over from a plan to — and the strategy
// now does the one walk itself, because the library-root seeding has to happen
// between "definitions found" and "dead functions computed", which is inside
// what those two helpers each did privately.

/// Analyze Rust files
fn analyze_rust_files(
    files: &[std::path::PathBuf],
) -> Result<(Vec<FunctionInfo>, HashSet<String>)> {
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
                            // `pub` is Rust's export keyword. In a LIBRARY
                            // crate a `pub fn` is reachable from outside the
                            // crate, which is precisely why rustc's own
                            // dead-code pass does not flag it — and why this
                            // engine, which has no rustc, must not either.
                            exported: line.trim_start().starts_with("pub "),
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
