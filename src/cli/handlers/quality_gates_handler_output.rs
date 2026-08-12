// Extracted from quality_gates_handler.rs — output formatting functions

/// Output JSON results
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn output_json(report: &QualityReport) -> Result<()> {
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
    println!(
        "\n{} Quality Gate Results",
        if report.passed { "✅" } else { "❌" }
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for gate in &report.gates {
        // These were raw `"\x1b[32m"` literals, so `quality-gates --color never`
        // wrote the same 8 escape sequences as `--color always` — the flag was
        // inert on this printer. `colors::colored` consults the one enablement
        // rule (`--color`, `NO_COLOR`, TTY) that the rest of the CLI is on.
        use crate::cli::colors as c;
        let icon = if gate.passed { "✓" } else { "✗" };
        let color = if gate.passed { c::GREEN } else { c::RED };

        println!(
            "{} ({:.2}s)",
            c::colored(color, &format!("{icon} {}", gate.name)),
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
