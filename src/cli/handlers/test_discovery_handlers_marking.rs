/// Test file edit for marking
#[derive(Debug, Clone)]
struct TestEdit {
    /// File path
    file: PathBuf,
    /// Test function name
    test_name: String,
    /// Line number (if known)
    #[allow(dead_code)]
    line: Option<u32>,
    /// Ignore reason
    reason: String,
}

/// Phase 3: Mark - Add #[ignore] attributes
async fn handle_mark(input: &Path, apply: bool) -> Result<()> {
    use crate::cli::colors as c;
    println!("{}", c::label("Marking tests as #[ignore]"));
    if !apply {
        println!("   {}", c::dim("(DRY RUN - use --apply to make changes)"));
    }
    println!();

    // Read categorization report
    let content = std::fs::read_to_string(input).context("Failed to read categorization report")?;
    let report: CategorizationReport =
        serde_json::from_str(&content).context("Failed to parse categorization report")?;

    // Collect all edits needed
    let mut edits: Vec<TestEdit> = Vec::new();
    for group in &report.groups {
        for test in &group.tests {
            edits.push(TestEdit {
                file: test.file.clone(),
                test_name: test.name.clone(),
                line: test.line,
                reason: group.ignore_reason.clone(),
            });
        }
    }

    println!(
        "   Found {} tests to mark across files",
        c::number(&edits.len().to_string())
    );

    // Group edits by file
    let mut by_file: std::collections::HashMap<PathBuf, Vec<TestEdit>> =
        std::collections::HashMap::new();
    for edit in edits {
        by_file.entry(edit.file.clone()).or_default().push(edit);
    }

    println!(
        "   {}: {}",
        c::dim("Files to modify"),
        c::number(&by_file.len().to_string())
    );
    println!();

    let mut modified_count = 0;
    let mut error_count = 0;

    for (file, file_edits) in &by_file {
        if !file.exists() || file.to_string_lossy() == "unknown" {
            // Try to resolve file from test name
            println!(
                "   {} Skipping {} (file not found)",
                c::warn(""),
                c::path(&file.display().to_string())
            );
            continue;
        }

        match mark_tests_in_file(file, file_edits, apply) {
            Ok(count) => {
                modified_count += count;
                if apply {
                    println!(
                        "   {} Modified {} tests in {}",
                        c::pass(""),
                        c::number(&count.to_string()),
                        c::path(&file.display().to_string())
                    );
                } else {
                    println!(
                        "   {} Would modify {} tests in {}",
                        c::dim(""),
                        c::number(&count.to_string()),
                        c::path(&file.display().to_string())
                    );
                }
            }
            Err(e) => {
                error_count += 1;
                println!(
                    "   {} Error in {}: {}",
                    c::fail(""),
                    c::path(&file.display().to_string()),
                    e
                );
            }
        }
    }

    println!();
    if apply {
        println!(
            "{} Marking complete: {} tests modified, {} errors",
            c::pass(""),
            c::number(&modified_count.to_string()),
            error_count
        );
    } else {
        println!(
            "{} Dry run complete: {} tests would be modified, {} errors",
            c::pass(""),
            c::number(&modified_count.to_string()),
            error_count
        );
        println!("   {}", c::dim("Run with --apply to make changes"));
    }

    Ok(())
}

/// Mark tests in a single file
fn mark_tests_in_file(file: &Path, edits: &[TestEdit], apply: bool) -> Result<usize> {
    let content = std::fs::read_to_string(file)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut modified = 0;

    for edit in edits {
        // Find the test function
        let test_fn_name = edit.test_name.split("::").last().unwrap_or(&edit.test_name);

        for (i, line) in lines.iter().enumerate() {
            // Look for fn test_name( or async fn test_name(
            let pattern = format!("fn {}(", test_fn_name);
            if line.contains(&pattern) {
                // Check if previous line already has #[ignore]
                if i > 0 && lines[i - 1].contains("#[ignore") {
                    continue; // Already marked
                }

                // Check if previous line has #[test]
                let insert_at = if i > 0 && lines[i - 1].contains("#[test") {
                    i // Insert between #[test] and fn
                } else {
                    i // Insert before fn
                };

                // Calculate indentation
                let indent = line.len() - line.trim_start().len();
                let indent_str = " ".repeat(indent);

                // Create ignore attribute
                let ignore_attr = format!("{}#[ignore = \"{}\"]", indent_str, edit.reason);

                if apply {
                    lines.insert(insert_at, ignore_attr);
                }
                modified += 1;
                break;
            }
        }
    }

    if apply && modified > 0 {
        std::fs::write(file, lines.join("\n"))?;
    }

    Ok(modified)
}

/// Phase 4: Verify - Ensure all tests pass
async fn handle_verify(path: &Path) -> Result<()> {
    use crate::cli::colors as c;
    println!(
        "{} Verifying tests pass in {}",
        c::pass(""),
        c::path(&path.display().to_string())
    );
    println!();

    // Run cargo test
    let mut cmd = Command::new("cargo");
    cmd.arg("test")
        .arg("--workspace")
        .arg("--")
        .arg("--include-ignored")
        .current_dir(path);

    println!(
        "{} Running: {}",
        c::label(""),
        c::dim("cargo test --workspace -- --include-ignored")
    );
    println!(
        "   {}",
        c::dim("(This includes ignored tests to verify they're marked correctly)")
    );
    println!();

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run cargo test")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse results
    let (passed, failed, ignored) = parse_test_summary(&stdout, &stderr);

    println!("{}", c::subheader("Test Results:"));
    println!(
        "   {} Passed: {}",
        c::pass(""),
        c::number(&passed.to_string())
    );
    println!(
        "   {} Failed: {}",
        c::fail(""),
        if failed > 0 {
            format!("{}{}{}", c::BOLD_RED, failed, c::RESET)
        } else {
            failed.to_string()
        }
    );
    println!(
        "   {} Ignored: {}",
        c::skip(""),
        c::number(&ignored.to_string())
    );
    println!();

    if failed > 0 {
        println!(
            "{} {} tests still failing!",
            c::warn(""),
            failed
        );
        println!(
            "   {}",
            c::dim("Run 'pmat test-discovery run' to discover remaining failures")
        );
        anyhow::bail!("{} tests still failing", failed);
    } else {
        println!("{}", c::pass("All tests passing or properly ignored!"));
    }

    Ok(())
}
