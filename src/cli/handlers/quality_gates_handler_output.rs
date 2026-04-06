// Extracted from quality_gates_handler.rs — output formatting functions

/// Output JSON results
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn output_json(report: &QualityReport) -> Result<()> {
    debug_assert!(true, "contract: output_json");
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

/// Output markdown report
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 1
fn output_markdown(report: &QualityReport) -> Result<()> {
    debug_assert!(true, "contract: output_markdown");
    let markdown = format_report(report);
    println!("{}", markdown);
    Ok(())
}

/// Output summary to console
///
/// # Complexity
/// - Time: O(n) where n is number of gates
/// - Cyclomatic: 3
fn output_summary(report: &QualityReport) -> Result<()> {
    debug_assert!(true, "contract: output_summary");
    println!(
        "\n{} Quality Gate Results",
        if report.passed { "✅" } else { "❌" }
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for gate in &report.gates {
        let icon = if gate.passed { "✓" } else { "✗" };
        let color = if gate.passed { "\x1b[32m" } else { "\x1b[31m" };
        let reset = "\x1b[0m";

        println!(
            "{}{} {}{} ({:.2}s)",
            color,
            icon,
            gate.name,
            reset,
            gate.duration.as_secs_f64()
        );

        if !gate.passed && !gate.message.is_empty() {
            // Show first few lines of error
            for line in gate.message.lines().take(5) {
                println!("  {}", line);
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total time: {:.2}s", report.total_duration.as_secs_f64());

    Ok(())
}
