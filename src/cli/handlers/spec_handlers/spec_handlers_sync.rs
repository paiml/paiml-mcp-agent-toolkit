/// Handle spec sync command - bidirectional spec-roadmap linking
pub async fn handle_spec_sync(
    spec_path: &Path,
    roadmap_path: &Path,
    dry_run: bool,
    direction: crate::cli::commands::SpecSyncDirection,
) -> anyhow::Result<()> {
    debug_assert!(spec_path.exists(), "spec_path must exist: {}", spec_path.display());
    debug_assert!(roadmap_path.exists(), "roadmap_path must exist: {}", roadmap_path.display());
    use crate::cli::commands::SpecSyncDirection;
    use crate::services::roadmap_service::RoadmapService;
    use regex::Regex;

    let parser = SpecParser::new();
    let specs = parser.find_specs(spec_path)?;
    let roadmap_service = RoadmapService::new(roadmap_path);

    if !roadmap_service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    let mut roadmap = roadmap_service.load()?;
    let ticket_re = Regex::new(r"(?m)^\*?\*?Ticket\*?\*?:\s*(\S+)")?;
    let mut updates = Vec::new();

    if matches!(direction, SpecSyncDirection::SpecToRoadmap | SpecSyncDirection::Both) {
        sync_spec_to_roadmap(&specs, &ticket_re, &mut roadmap, dry_run, &mut updates)?;
    }

    if matches!(direction, SpecSyncDirection::RoadmapToSpec | SpecSyncDirection::Both) {
        sync_roadmap_to_spec(&ticket_re, &roadmap, &mut updates)?;
    }

    print_sync_results(&updates, dry_run, &roadmap_service, &roadmap, roadmap_path)?;
    Ok(())
}

fn sync_spec_to_roadmap(
    specs: &[std::path::PathBuf],
    ticket_re: &regex::Regex,
    roadmap: &mut crate::models::roadmap::Roadmap,
    dry_run: bool,
    updates: &mut Vec<String>,
) -> anyhow::Result<()> {
    for spec_file in specs {
        let content = std::fs::read_to_string(spec_file)?;
        let ticket_id = match ticket_re.captures(&content).and_then(|c| c.get(1)) {
            Some(m) if !m.as_str().is_empty() => m.as_str().to_string(),
            _ => continue,
        };
        let rel_spec_path = spec_file
            .strip_prefix(std::env::current_dir()?)
            .unwrap_or(spec_file);
        update_roadmap_item(&mut roadmap.roadmap, &ticket_id, rel_spec_path, dry_run, updates);
    }
    Ok(())
}

fn update_roadmap_item(
    items: &mut [crate::models::roadmap::RoadmapItem],
    ticket_id: &str,
    rel_spec_path: &Path,
    dry_run: bool,
    updates: &mut Vec<String>,
) {
    debug_assert!(rel_spec_path.exists(), "rel_spec_path must exist: {}", rel_spec_path.display());
    debug_assert!(!ticket_id.is_empty(), "ticket_id must not be empty");
    for item in items.iter_mut() {
        if !item.id.eq_ignore_ascii_case(ticket_id) {
            continue;
        }
        let new_spec = Some(rel_spec_path.to_path_buf());
        if item.spec != new_spec {
            updates.push(format!("  {} → spec: {}", item.id, rel_spec_path.display()));
            if !dry_run {
                item.spec = new_spec;
            }
        }
        break;
    }
}

fn sync_roadmap_to_spec(
    ticket_re: &regex::Regex,
    roadmap: &crate::models::roadmap::Roadmap,
    updates: &mut Vec<String>,
) -> anyhow::Result<()> {
    for item in &roadmap.roadmap {
        let spec_file = match &item.spec {
            Some(s) => s,
            None => continue,
        };
        let full_path = if spec_file.is_absolute() {
            spec_file.clone()
        } else {
            std::env::current_dir()?.join(spec_file)
        };
        if full_path.exists() {
            let content = std::fs::read_to_string(&full_path)?;
            if !ticket_re.is_match(&content) {
                updates.push(format!(
                    "  {} ← needs Ticket: {} in frontmatter",
                    spec_file.display(),
                    item.id
                ));
            }
        }
    }
    Ok(())
}

fn print_sync_results(
    updates: &[String],
    dry_run: bool,
    roadmap_service: &crate::services::roadmap_service::RoadmapService,
    roadmap: &crate::models::roadmap::Roadmap,
    roadmap_path: &Path,
) -> anyhow::Result<()> {
    debug_assert!(roadmap_path.exists(), "roadmap_path must exist: {}", roadmap_path.display());
    use crate::cli::colors as c;
    if updates.is_empty() {
        println!("{}", c::pass("Specs and roadmap are in sync. No updates needed."));
    } else {
        if dry_run {
            println!("{}", c::dim("Dry run - would make these changes:"));
        } else {
            println!("{}", c::pass("Applied changes:"));
        }
        for update in updates {
            println!("{}", update);
        }
        if !dry_run {
            roadmap_service.save(roadmap)?;
            println!(
                "\n{} {}",
                c::pass("Saved roadmap to"),
                c::path(&roadmap_path.display().to_string())
            );
        }
    }
    Ok(())
}

#[derive(Debug)]
struct DriftInfo {
    path: std::path::PathBuf,
    title: String,
    has_ticket: bool,
    ticket_id: Option<String>,
    linked_in_roadmap: bool,
}

/// Handle spec drift command - find specs without roadmap links
pub async fn handle_spec_drift(
    spec_path: &Path,
    roadmap_path: &Path,
    format: SpecOutputFormat,
) -> anyhow::Result<()> {
    debug_assert!(spec_path.exists(), "spec_path must exist: {}", spec_path.display());
    debug_assert!(roadmap_path.exists(), "roadmap_path must exist: {}", roadmap_path.display());
    use crate::services::roadmap_service::RoadmapService;
    use regex::Regex;
    use std::collections::HashSet;

    let parser = SpecParser::new();
    let specs = parser.find_specs(spec_path)?;
    let roadmap_service = RoadmapService::new(roadmap_path);

    let mut linked_specs: HashSet<std::path::PathBuf> = HashSet::new();
    if roadmap_service.exists() {
        let roadmap = roadmap_service.load()?;
        for item in &roadmap.roadmap {
            if let Some(ref spec) = item.spec {
                linked_specs.insert(spec.clone());
            }
        }
    }

    let ticket_re = Regex::new(r"(?m)^\*?\*?Ticket\*?\*?:\s*(\S+)")?;
    let orphans = collect_drift_orphans(&specs, &parser, &ticket_re, &linked_specs)?;
    print_drift_report(&orphans, format)?;
    Ok(())
}

fn collect_drift_orphans(
    specs: &[std::path::PathBuf],
    parser: &SpecParser,
    ticket_re: &regex::Regex,
    linked_specs: &std::collections::HashSet<std::path::PathBuf>,
) -> anyhow::Result<Vec<DriftInfo>> {
    let mut orphans = Vec::new();
    for spec_file in specs {
        let rel_path = spec_file
            .strip_prefix(std::env::current_dir()?)
            .unwrap_or(spec_file)
            .to_path_buf();
        let content = std::fs::read_to_string(spec_file)?;
        let ticket_match = ticket_re.captures(&content);
        let has_ticket = ticket_match.is_some();
        let ticket_id = ticket_match.and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
        let linked = linked_specs.contains(&rel_path);
        let title = parser
            .parse_file(spec_file)
            .map(|s| s.title)
            .unwrap_or_else(|_| {
                spec_file.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string()
            });
        if !has_ticket || !linked {
            orphans.push(DriftInfo { path: rel_path, title, has_ticket, ticket_id, linked_in_roadmap: linked });
        }
    }
    Ok(orphans)
}

fn print_drift_report(orphans: &[DriftInfo], format: SpecOutputFormat) -> anyhow::Result<()> {
    match format {
        SpecOutputFormat::Text => print_drift_text(orphans),
        SpecOutputFormat::Json => print_drift_json(orphans)?,
        SpecOutputFormat::Markdown => print_drift_markdown(orphans),
    }
    Ok(())
}

fn print_drift_json(orphans: &[DriftInfo]) -> anyhow::Result<()> {
    let json: Vec<_> = orphans
        .iter()
        .map(|o| {
            serde_json::json!({
                "path": o.path.display().to_string(),
                "title": o.title,
                "has_ticket": o.has_ticket,
                "ticket_id": o.ticket_id,
                "linked_in_roadmap": o.linked_in_roadmap,
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn print_drift_markdown(orphans: &[DriftInfo]) {
    println!("# Spec Drift Report\n");
    if orphans.is_empty() {
        println!("✅ No drift detected.");
    } else {
        println!("| Spec | Has Ticket | In Roadmap |");
        println!("|------|------------|------------|");
        for o in orphans {
            let ticket = if o.has_ticket { "✅" } else { "❌" };
            let roadmap = if o.linked_in_roadmap { "✅" } else { "❌" };
            println!("| {} | {} | {} |", o.path.display(), ticket, roadmap);
        }
    }
}

fn print_drift_text(orphans: &[DriftInfo]) {
    use crate::cli::colors as c;
    if orphans.is_empty() {
        println!("{}", c::pass("No drift detected. All specs are properly linked."));
    } else {
        println!(
            "{}",
            c::warn(&format!("Found {} specs with drift:", orphans.len()))
        );
        println!();
        println!(
            "{}{:<45}{} {:>10} {:>12}",
            c::BOLD, "SPEC", c::RESET, "HAS_TICKET", "IN_ROADMAP"
        );
        println!("{}", c::separator());
        for o in orphans {
            let ticket_status = if o.has_ticket {
                o.ticket_id.as_deref().unwrap_or("yes")
            } else {
                "missing"
            };
            let ticket_display = if o.has_ticket {
                c::pass(ticket_status)
            } else {
                c::fail(ticket_status)
            };
            let roadmap_display = if o.linked_in_roadmap {
                c::pass("yes")
            } else {
                c::fail("no")
            };
            println!(
                "{:<45} {:>18} {:>20}",
                c::path(&o.path.display().to_string()),
                ticket_display,
                roadmap_display
            );
        }
        println!(
            "\n{} Fix with: {}",
            c::dim("Tip:"),
            c::label("pmat spec sync --dry-run")
        );
    }
}
