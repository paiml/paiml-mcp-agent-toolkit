/// Handle work add command (CRUD: Create)
///
/// Creates a new work ticket in roadmap.yaml with optional GitHub issue creation.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_add(
    title: String,
    description: Option<String>,
    priority: crate::cli::commands::WorkPriority,
    tags: Option<String>,
    path: Option<PathBuf>,
    create_github: bool,
) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    // Validate roadmap exists
    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Load existing roadmap to find next available ID
    let roadmap = service.load()?;
    let next_id = generate_next_id(&roadmap);

    // Create new item
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let item = crate::models::roadmap::RoadmapItem {
        id: next_id.clone(),
        github_issue: None,
        item_type: crate::models::roadmap::ItemType::Task,
        title: title.clone(),
        status: crate::models::roadmap::ItemStatus::Planned,
        priority: priority.to_roadmap_priority(),
        assigned_to: None,
        created: now.clone(),
        updated: now,
        spec: None,
        acceptance_criteria: description
            .as_ref()
            .map(|d| vec![d.clone()])
            .unwrap_or_default(),
        phases: vec![],
        subtasks: vec![],
        estimated_effort: None,
        labels: tags
            .clone()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        notes: None,
    };

    // Save to roadmap
    service.upsert_item(item)?;

    println!("{}", c::pass(&format!("Created ticket: {}", c::path(&next_id))));
    println!("   {} {}", c::label("Title:"), title);
    println!("   {} {:?}", c::label("Priority:"), priority);
    if let Some(desc) = description {
        println!("   {} {}", c::label("Description:"), desc);
    }
    if let Some(t) = tags {
        println!("   {} {}", c::label("Tags:"), t);
    }

    // Create GitHub issue if requested
    if create_github {
        println!();
        println!("{}", c::warn("GitHub issue creation not yet implemented. Use 'pmat work sync' after creating the ticket."));
    }

    Ok(())
}

/// Resolve `--status` into the one status it names.
///
/// The whole alias vocabulary lives in `ItemStatus::from_string`, which is also
/// what `work list-statuses` prints; going through it is what keeps the filter
/// and the advertised aliases from drifting apart. An unparseable value is an
/// error — returning "no items" for a status nobody can spell is the failure
/// mode this replaced.
fn parse_status_filter(status: Option<&str>) -> Result<Option<crate::models::roadmap::ItemStatus>> {
    match status {
        Some(s) => crate::models::roadmap::ItemStatus::from_string(s)
            .map(Some)
            .map_err(|e| anyhow::anyhow!("{e}")),
        None => Ok(None),
    }
}

/// Handle work list command (CRUD: Read - simple list)
///
/// Lists all work tickets with optional filtering.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_list(
    status: Option<String>,
    priority: Option<crate::cli::commands::WorkPriority>,
    count_only: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    let roadmap = service.load()?;

    // `--status` used to be matched with `format!("{:?}", item.status)
    // .to_lowercase().contains(&s.to_lowercase())` — a Debug-string substring
    // test. Every alias `work list-statuses` advertises (done/finished/closed,
    // in-progress/wip, todo/open/pending) therefore matched nothing and
    // returned an empty list, indistinguishable from "no work in that state".
    // `ItemStatus::from_string` already implements the whole advertised alias
    // vocabulary; parse once, compare exactly, and reject an unknown value
    // instead of silently returning zero rows.
    let status_filter = parse_status_filter(status.as_deref())?;

    // Filter items
    let items: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| {
            // Filter by status if specified
            if let Some(ref wanted) = status_filter {
                if item.status != *wanted {
                    return false;
                }
            }
            // Filter by priority if specified
            if let Some(ref p) = priority {
                let roadmap_priority = p.to_roadmap_priority();
                if item.priority != roadmap_priority {
                    return false;
                }
            }
            true
        })
        .collect();

    if count_only {
        println!("{}", items.len());
        return Ok(());
    }

    if items.is_empty() {
        println!("{}", c::dim("No tickets found matching criteria."));
        return Ok(());
    }

    // Print header
    println!(
        "{}{:<12} {:<12} {:<10} TITLE{}",
        c::BOLD, "ID", "STATUS", "PRIORITY", c::RESET
    );
    println!("{}", c::separator());

    // Print items
    for item in items {
        let status_str = format!("{:?}", item.status).to_lowercase();
        let priority_str = format!("{:?}", item.priority).to_lowercase();
        let title_truncated = if item.title.len() > 40 {
            format!("{}...", item.title.get(..37).unwrap_or(&item.title))
        } else {
            item.title.clone()
        };
        println!(
            "{}{:<12}{} {:<12} {:<10} {}",
            c::CYAN, item.id, c::RESET, status_str, priority_str, title_truncated
        );
    }

    Ok(())
}

/// Handle work edit command (CRUD: Update)
///
/// Edits an existing work ticket.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_edit(
    id: String,
    title: Option<String>,
    description: Option<String>,
    priority: Option<crate::cli::commands::WorkPriority>,
    status: Option<String>,
    tags: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the item (with fuzzy matching)
    let item = find_item_fuzzy(&service, &id)?;
    let mut updated_item = item.clone();
    let mut changes = vec![];

    // Apply changes
    if let Some(new_title) = title {
        updated_item.title = new_title.clone();
        changes.push(format!("title: {}", new_title));
    }

    if let Some(desc) = description {
        updated_item.acceptance_criteria = vec![desc.clone()];
        changes.push(format!("description: {}", desc));
    }

    if let Some(p) = priority {
        updated_item.priority = p.to_roadmap_priority();
        changes.push(format!("priority: {:?}", p));
    }

    if let Some(s) = status {
        let new_status = crate::models::roadmap::ItemStatus::from_string(&s)
            .map_err(|e| anyhow::anyhow!("Invalid status '{}': {}", s, e))?;
        // DBC §work_lifecycle: Validate state transition via adjacency matrix
        if !updated_item.status.can_transition_to(new_status) {
            anyhow::bail!(
                "Invalid transition: {} → {}. See work-dbc-v1.yaml §work_lifecycle for valid transitions.",
                updated_item.status.display_name(),
                new_status.display_name(),
            );
        }
        updated_item.status = new_status;
        changes.push(format!("status: {}", s));
    }

    if let Some(t) = tags {
        updated_item.labels = t.split(',').map(|s| s.trim().to_string()).collect();
        changes.push(format!("labels: {}", t));
    }

    if changes.is_empty() {
        println!("{}", c::warn("No changes specified. Use --title, --description, --priority, --status, or --tags."));
        return Ok(());
    }

    // Update timestamp
    updated_item.updated = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Save
    service.upsert_item(updated_item)?;

    println!("{}", c::pass(&format!("Updated ticket: {}", c::path(&item.id))));
    for change in changes {
        println!("   {}", change);
    }

    Ok(())
}

/// Handle work delete command (CRUD: Delete)
///
/// Deletes a work ticket from roadmap.yaml.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_delete(id: String, force: bool, path: Option<PathBuf>) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);

    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    // Find the item (with fuzzy matching)
    let item = find_item_fuzzy(&service, &id)?;

    // Confirm deletion unless --force
    if !force {
        println!("{}", c::subheader("About to delete ticket:"));
        println!("  {} {}", c::label("ID:"), c::path(&item.id));
        println!("  {} {}", c::label("Title:"), item.title);
        println!("  {} {:?}", c::label("Status:"), item.status);
        println!();
        println!("{}", c::warn("Use --force to skip this confirmation."));
        return Ok(());
    }

    // Delete
    service.remove_item(&item.id)?;
    println!("🗑️  Deleted ticket: {} - {}", c::path(&item.id), item.title);

    Ok(())
}

#[cfg(test)]
mod status_filter_tests {
    use super::parse_status_filter;
    use crate::models::roadmap::ItemStatus;

    /// `work list --status done` returned 0 items while `--status completed`
    /// returned 169: the filter was a Debug-string substring test, so none of
    /// the aliases `work list-statuses` documents ever matched.
    #[test]
    fn every_documented_alias_resolves_to_its_canonical_status() {
        let cases = [
            ("completed", ItemStatus::Completed),
            ("done", ItemStatus::Completed),
            ("finished", ItemStatus::Completed),
            ("closed", ItemStatus::Completed),
            ("inprogress", ItemStatus::InProgress),
            ("In-Progress", ItemStatus::InProgress),
            ("in_progress", ItemStatus::InProgress),
            ("InProgress", ItemStatus::InProgress),
            ("wip", ItemStatus::InProgress),
            ("planned", ItemStatus::Planned),
            ("todo", ItemStatus::Planned),
            ("open", ItemStatus::Planned),
            ("pending", ItemStatus::Planned),
        ];

        for (input, expected) in cases {
            assert_eq!(
                parse_status_filter(Some(input)).expect("documented alias must parse"),
                Some(expected),
                "--status {input} must select {expected:?}"
            );
        }
    }

    /// The alias table itself is the contract; walk it rather than a copy.
    #[test]
    fn the_whole_status_table_is_filterable() {
        for &(canonical, aliases, _) in ItemStatus::STATUS_TABLE {
            let expected = ItemStatus::from_string(canonical).expect("canonical parses");
            for alias in aliases.split(',').map(str::trim).filter(|a| !a.is_empty()) {
                assert_eq!(
                    parse_status_filter(Some(alias)).expect("table alias must parse"),
                    Some(expected),
                    "--status {alias} must select {canonical}"
                );
            }
        }
    }

    /// A status nobody can spell must say so, not quietly return zero rows.
    #[test]
    fn an_unknown_status_is_an_error() {
        let err = parse_status_filter(Some("definitely-not-a-status"))
            .expect_err("an unknown --status must be rejected");
        assert!(err.to_string().contains("unknown status"), "{err}");
    }

    #[test]
    fn no_status_means_no_filter() {
        assert_eq!(parse_status_filter(None).expect("no filter"), None);
    }
}
