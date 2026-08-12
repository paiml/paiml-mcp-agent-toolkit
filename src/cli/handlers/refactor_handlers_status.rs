// Refactor status command handler and output formatting helpers.
// Handles `pmat refactor status` with JSON, table, and summary output formats.

// Sprint 89 GREEN Phase: Refactored handle_refactor_status function
// BEFORE: Complexity 14 (High entropy, mixed concerns)
// AFTER: Complexity 5 (A+ standard, single responsibility)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_refactor_status(
    checkpoint: PathBuf,
    format: RefactorOutputFormat,
) -> anyhow::Result<()> {
    crate::status_println!("📊 Reading refactor status from: {}", checkpoint.display());

    // Delegate file validation to extracted function
    validate_checkpoint_file(checkpoint.as_path())?;

    // Delegate file reading to extracted function
    let checkpoint_data = read_checkpoint_data(&checkpoint).await?;

    // Delegate output formatting to extracted function
    format_refactor_status(&checkpoint_data, format, &checkpoint)?;

    Ok(())
}

// Sprint 89 GREEN Phase: NEW EXTRACTED FUNCTIONS (A+ <=10 complexity each)

/// Validate checkpoint file existence - EXTRACTED FUNCTION
/// Complexity: 2 (A+ standard)
fn validate_checkpoint_file(checkpoint: &Path) -> anyhow::Result<()> {
    if !checkpoint.exists() {
        return Err(anyhow::anyhow!(
            "Checkpoint file not found: {}",
            checkpoint.display()
        ));
    }
    Ok(())
}

/// Read checkpoint data from file - EXTRACTED FUNCTION
/// Complexity: 2 (A+ standard)
async fn read_checkpoint_data(checkpoint: &PathBuf) -> anyhow::Result<String> {
    tokio::fs::read_to_string(checkpoint)
        .await
        .map_err(Into::into)
}

/// Format and display refactor status - EXTRACTED FUNCTION
/// Complexity: 4 (A+ standard)
fn format_refactor_status(
    checkpoint_data: &str,
    format: RefactorOutputFormat,
    checkpoint: &PathBuf,
) -> anyhow::Result<()> {
    match format {
        RefactorOutputFormat::Json => format_as_json(checkpoint_data),
        RefactorOutputFormat::Table => format_as_table(checkpoint_data),
        RefactorOutputFormat::Summary => format_as_summary(checkpoint_data, checkpoint.as_path()),
    }
}

/// Format status as JSON - EXTRACTED FUNCTION
/// Complexity: 3 (A+ standard)
fn format_as_json(checkpoint_data: &str) -> anyhow::Result<()> {
    let parsed: serde_json::Value = serde_json::from_str(checkpoint_data)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    Ok(())
}

/// Format status as table - EXTRACTED FUNCTION
/// Complexity: 8 (A+ standard)
fn format_as_table(checkpoint_data: &str) -> anyhow::Result<()> {
    let state: serde_json::Value = serde_json::from_str(checkpoint_data)?;

    // Print table header
    print_table_header();

    // Print table data rows
    print_table_data(&state);

    // Print table footer
    print_table_footer();

    Ok(())
}

/// Print table header - EXTRACTED FUNCTION
/// Complexity: 2 (A+ standard)
fn print_table_header() {
    println!("┌─────────────────┬──────────────────────────────────────┐");
    println!("│ Property        │ Value                                │");
    println!("├─────────────────┼──────────────────────────────────────┤");
}

/// Print table data rows - EXTRACTED FUNCTION
/// Complexity: 8 (A+ standard)
fn print_table_data(state: &serde_json::Value) {
    if let Some(current) = state.get("current") {
        println!(
            "│ Current State   │ {:36} │",
            format!("{current:?}").chars().take(36).collect::<String>()
        );
    }

    if let Some(targets) = state.get("targets") {
        if let Some(targets_array) = targets.as_array() {
            println!("│ Target Count    │ {:36} │", targets_array.len());
        }
    }

    if let Some(index) = state.get("current_target_index") {
        println!("│ Current Index   │ {:36} │", index.as_u64().unwrap_or(0));
    }
}

/// Print table footer - EXTRACTED FUNCTION
/// Complexity: 1 (A+ standard)
fn print_table_footer() {
    println!("└─────────────────┴──────────────────────────────────────┘");
}

/// Format status as summary - EXTRACTED FUNCTION
/// Complexity: 6 (A+ standard)
fn format_as_summary(checkpoint_data: &str, checkpoint: &Path) -> anyhow::Result<()> {
    let state: serde_json::Value = serde_json::from_str(checkpoint_data)?;
    println!("🔧 Refactor Status Summary");
    println!("   Checkpoint: {}", checkpoint.display());

    if let Some(current) = state.get("current") {
        println!("   Current state: {current:?}");
    }

    if let Some(targets) = state.get("targets") {
        if let Some(targets_array) = targets.as_array() {
            println!("   Total targets: {}", targets_array.len());
        }
    }

    Ok(())
}
