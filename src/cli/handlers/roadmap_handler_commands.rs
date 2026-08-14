// Command handler functions for roadmap maintenance
// Included by roadmap_handler.rs - shares parent module scope

/// Handle roadmap maintenance command (TICKET-PMAT-6012)
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 7
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_maintain_roadmap(
    roadmap_path: PathBuf,
    tickets_dir: PathBuf,
    config: RoadmapMaintenanceConfig,
    format: OutputFormat,
) -> Result<()> {
    if config.validate {
        validate_roadmap(&roadmap_path, &tickets_dir).await?;
    }

    if config.health {
        show_health_report(&roadmap_path, &tickets_dir, &format).await?;
    }

    if config.fix {
        fix_roadmap_status(&roadmap_path, &tickets_dir, config.dry_run).await?;
    }

    if config.generate_tickets {
        generate_missing_ticket_files(&roadmap_path, &tickets_dir, config.dry_run).await?;
    }

    if !config.has_actions() {
        // Default: show health
        show_health_report(&roadmap_path, &tickets_dir, &format).await?;
    }

    Ok(())
}

/// Validate roadmap structure and ticket consistency (internal, reusable)
/// (TICKET-PMAT-6019)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn validate_roadmap_internal(
    roadmap_path: &Path,
    tickets_dir: &Path,
) -> Result<RoadmapValidation> {
    let roadmap_content = fs::read_to_string(roadmap_path).map_err(|_| {
        let error = crate::cli::error_context::roadmap_not_found(roadmap_path);
        anyhow::anyhow!(error.format_detailed())
    })?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Parse roadmap tickets
    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    // Check each ticket file
    for (ticket_id, checkbox_status) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if !ticket_path.exists() {
            errors.push(format!("Missing ticket file: {ticket_id}.md"));
            continue;
        }

        let ticket_status = get_ticket_status(&ticket_path)?;

        // Check consistency
        if *checkbox_status && ticket_status != TicketStatus::Green {
            warnings.push(format!(
                "{ticket_id}: Checked in roadmap but status is {:?}",
                ticket_status
            ));
        }

        if !checkbox_status && ticket_status == TicketStatus::Green {
            warnings.push(format!(
                "{ticket_id}: Unchecked in roadmap but status is GREEN"
            ));
        }
    }

    Ok(RoadmapValidation {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

/// Validate roadmap structure and ticket consistency (CLI wrapper)
async fn validate_roadmap(roadmap_path: &Path, tickets_dir: &Path) -> Result<()> {
    let validation = validate_roadmap_internal(roadmap_path, tickets_dir).await?;

    // Report results
    if validation.valid && validation.warnings.is_empty() {
        crate::status_eprintln!("✅ Roadmap validation passed!");
    } else {
        if !validation.errors.is_empty() {
            eprintln!("❌ Validation errors:");
            for error in &validation.errors {
                eprintln!("  - {error}");
            }
        }
        if !validation.warnings.is_empty() {
            eprintln!("⚠️  Warnings:");
            for warning in &validation.warnings {
                eprintln!("  - {warning}");
            }
        }
        if !validation.valid {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Fix roadmap checkbox status based on ticket files
async fn fix_roadmap_status(roadmap_path: &Path, tickets_dir: &Path, dry_run: bool) -> Result<()> {
    let roadmap_content = fs::read_to_string(roadmap_path)?;
    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    let mut updated_content = roadmap_content;
    let mut changes = Vec::new();

    for (ticket_id, checkbox_status) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if !ticket_path.exists() {
            continue;
        }

        let ticket_status = get_ticket_status(&ticket_path)?;
        let should_be_checked = ticket_status == TicketStatus::Green;

        if *checkbox_status != should_be_checked {
            changes.push((ticket_id.clone(), should_be_checked));

            let old_pattern = build_checkbox_pattern(ticket_id, *checkbox_status);
            let new_pattern = build_checkbox_pattern(ticket_id, should_be_checked);

            updated_content = updated_content.replace(&old_pattern, &new_pattern);
        }
    }

    if changes.is_empty() {
        crate::status_eprintln!("✅ Roadmap is already up-to-date!");
        return Ok(());
    }

    apply_roadmap_changes(roadmap_path, updated_content, &changes, dry_run)
}

/// Generate missing ticket files from roadmap (internal, reusable)
/// (TICKET-PMAT-6021)
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 6
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn generate_tickets_internal(
    roadmap_path: &Path,
    tickets_dir: &Path,
    dry_run: bool,
) -> Result<TicketGenerationResult> {
    let roadmap_content = fs::read_to_string(roadmap_path)?;
    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    let mut generated = Vec::new();
    let mut skipped = Vec::new();

    for (ticket_id, checked) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if ticket_path.exists() {
            skipped.push(ticket_id.clone());
            continue;
        }

        generated.push(ticket_id.clone());

        // Extract sprint context from roadmap
        let sprint = extract_sprint_for_ticket(&roadmap_content, ticket_id);
        let status = if *checked {
            "GREEN ✅"
        } else {
            "PLANNED 📋"
        };

        let template = generate_ticket_template(ticket_id, &sprint, status);

        if !dry_run {
            fs::create_dir_all(tickets_dir)?;
            fs::write(&ticket_path, template)?;
        }
    }

    Ok(TicketGenerationResult { generated, skipped })
}

/// Generate missing ticket files from roadmap (CLI wrapper)
/// (TICKET-PMAT-6012)
async fn generate_missing_ticket_files(
    roadmap_path: &Path,
    tickets_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    crate::status_eprintln!("📝 Checking for missing ticket files...\n");

    let result = generate_tickets_internal(roadmap_path, tickets_dir, dry_run).await?;

    // Print each generated ticket
    for ticket_id in &result.generated {
        if dry_run {
            let roadmap_content = fs::read_to_string(roadmap_path)?;
            let sprint = extract_sprint_for_ticket(&roadmap_content, ticket_id);
            eprintln!("Would create: {ticket_id}.md (Sprint: {sprint})");
        } else {
            eprintln!("Created: {ticket_id}.md");
        }
    }

    crate::status_eprintln!();
    if result.generated.is_empty() {
        crate::status_eprintln!("✅ No missing ticket files");
    } else {
        crate::status_eprintln!("✅ Generated {} ticket file(s)", result.generated.len());
        if dry_run {
            crate::status_eprintln!("🔍 Dry-run mode - no files created");
        }
    }

    if !result.skipped.is_empty() {
        crate::status_eprintln!("⏭️  Skipped {} existing ticket(s)", result.skipped.len());
    }

    Ok(())
}
