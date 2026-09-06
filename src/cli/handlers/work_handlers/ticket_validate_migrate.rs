/// Collect semantic warnings for roadmap items (helper for handle_work_validate)
fn collect_semantic_warnings(roadmap: &crate::models::roadmap::Roadmap) -> Vec<String> {
    let mut warnings = Vec::new();
    for item in &roadmap.roadmap {
        if item.acceptance_criteria.is_empty()
            && !matches!(item.status, ItemStatus::Cancelled)
        {
            warnings.push(format!("⚠️  {} has no acceptance criteria", item.id));
        }
        if item.id.chars().count() > 50 {
            let truncated: String = item.id.chars().take(30).collect();
            warnings.push(format!(
                "⚠️  {} has a long ID ({} chars) - consider using shorter IDs",
                truncated,
                item.id.chars().count()
            ));
        }
    }
    warnings
}

/// Print valid roadmap with semantic validation (helper for handle_work_validate)
fn print_valid_roadmap(
    roadmap: &crate::models::roadmap::Roadmap,
    verbose: bool,
    fix: bool,
) {
    use crate::cli::colors as c;
    println!("{}", c::pass("Syntax valid"));
    println!("   {} {}", c::label("Version:"), roadmap.roadmap_version);
    println!("   {} {}", c::label("Items:"), c::number(&roadmap.roadmap.len().to_string()));
    println!(
        "   {} {}",
        c::label("GitHub:"),
        if roadmap.github_enabled {
            roadmap.github_repo.as_deref().unwrap_or("not configured")
        } else {
            "disabled"
        }
    );
    println!();

    let warnings = collect_semantic_warnings(roadmap);
    if !warnings.is_empty() {
        println!("{}", c::subheader(&format!("Warnings ({}):", warnings.len())));
        for warning in &warnings {
            println!("   {}", warning);
        }
        println!();
    }

    if verbose {
        println!("{}", c::subheader("📋 Items:"));
        for item in &roadmap.roadmap {
            println!("   {} [{:?}] - {}", c::path(&item.id), item.status, item.title);
        }
    }

    if fix && !warnings.is_empty() {
        println!("{}", c::dim("💡 Tip: Use `pmat work migrate` to auto-fix issues"));
    }

    println!("{}", c::pass("Validation passed"));
}

/// Print YAML parse error with context and suggestions (helper for handle_work_validate)
fn print_yaml_error_context(error_msg: &str, content: &str) {
    use crate::cli::colors as c;
    println!("{}", c::fail("Validation failed"));
    println!();
    println!("{} {}", c::label("Error:"), error_msg);
    println!();

    if let Some(line) = extract_line_from_yaml_error(error_msg) {
        let lines: Vec<&str> = content.lines().collect();
        if line > 0 && line <= lines.len() {
            println!("{}", c::subheader(&format!("Context (around line {}):", line)));
            let start = line.saturating_sub(3);
            let end = std::cmp::min(line + 2, lines.len());
            for (i, l) in lines[start..end].iter().enumerate() {
                let line_num = start + i + 1;
                let marker = if line_num == line {
                    format!("{}>>>{}", c::RED, c::RESET)
                } else {
                    "   ".to_string()
                };
                println!("{} {:4}: {}", marker, line_num, l);
            }
            println!();
        }
    }

    println!("{}", c::dim("💡 Common fixes:"));
    println!(
        "   {}",
        c::dim("- `status` is lenient (aliases, any case): planned, inprogress, blocked, review, completed, cancelled")
    );
    println!(
        "   {}",
        c::dim("- `item_type` is exact lowercase: task, epic, bug, feature, enhancement, documentation, refactor")
    );
    println!(
        "   {}",
        c::dim("- `priority` is exact lowercase: low, medium, high, critical")
    );
    println!(
        "   {}",
        c::dim("- Quote strings with special characters: `:`, `<`, `>`")
    );
    println!(
        "   {}",
        c::dim("- Use proper YAML indentation (2 spaces)")
    );
    println!();
    println!(
        "{}",
        c::dim("Run `pmat work list-statuses` for the status vocabulary, or see docs/roadmap-schema.md.")
    );
}

/// Handle work validate command (Part B: UX Improvements)
///
/// Validates roadmap.yaml syntax and content with actionable error messages.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_validate(path: Option<PathBuf>, verbose: bool, fix: bool) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("{}", c::label(&format!("🔍 Validating roadmap: {}", c::path(&roadmap_path.display().to_string()))));
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    let content = std::fs::read_to_string(&roadmap_path).context("Failed to read roadmap file")?;

    match serde_yaml_ng::from_str::<crate::models::roadmap::Roadmap>(&content) {
        Ok(roadmap) => {
            // A duplicated id survives the strict parse: `roadmap` is a
            // sequence, so two rows sharing an id are two well-formed rows and
            // serde has nothing to complain about. Only the raw text can see
            // the collision, and only the raw text can locate it.
            //
            // PMAT-676: through `check_roadmap_text`, the one validator that
            // `work add` and `work edit` also run before they write, so the
            // three commands cannot disagree about what a valid roadmap is.
            if let Err(invalid) = crate::services::roadmap_text::check_roadmap_text(
                &content,
                &roadmap_path,
            ) {
                return Err(report_duplicate_ids(invalid.duplicates(), &roadmap_path));
            }
            print_valid_roadmap(&roadmap, verbose, fix);
            Ok(())
        }
        Err(e) => {
            print_yaml_error_context(&format!("{}", e), &content);
            print_row_violations(&collect_row_violations(&content));
            Err(locate_parse_error(&e, &roadmap_path))
        }
    }
}

/// Print every duplicated id and return the error `work validate` fails with.
///
/// Both carry the same text: the stdout lines are for a human reading the run,
/// the error is what a caller that only captures stderr (or a `?` chain) gets.
fn report_duplicate_ids(duplicates: &[(String, Vec<usize>)], roadmap_path: &Path) -> anyhow::Error {
    let file = roadmap_path.display().to_string();
    let mut located = Vec::with_capacity(duplicates.len());
    for (id, lines) in duplicates {
        // PMAT-676: the one renderer, so the line a reader sees here and the
        // line `work add`/`work edit` refuse with are the same string.
        let at = crate::services::roadmap_text::located(&file, lines);
        println!("error: duplicate id {id} at {at}");
        located.push(format!("{id} at {at}"));
    }
    anyhow::anyhow!(
        "Roadmap validation failed: {} duplicate id(s): {}",
        duplicates.len(),
        located.join("; ")
    )
}

/// Prefix a YAML parse failure with the position it happened at.
///
/// `serde_yaml_ng` knows the line and column; the previous bail threw them away
/// and returned the bare string "Roadmap validation failed", so the only copy
/// of the position was in the context block printed to stdout.
fn locate_parse_error(error: &serde_yaml_ng::Error, roadmap_path: &Path) -> anyhow::Error {
    let file = roadmap_path.display();
    match error.location() {
        Some(at) => anyhow::anyhow!("{file}:{}:{}: {error}", at.line(), at.column()),
        None => anyhow::anyhow!("{file}: {error}"),
    }
}

/// Every `id` key line of the raw roadmap text, 1-based, in file order.
///
/// PMAT-676: a thin wrapper over `services::roadmap_text::id_lines`, which is
/// now the ONE scanner. It used to live here, beside a second, laxer copy in
/// the `work add` allocator (PMAT-673); the two disagreed about what an id
/// line is, and that disagreement is exactly how `add` came to accept a
/// roadmap this function's caller rejects. Kept as a name at this path because
/// the PMAT-674 tests address it here.
pub(crate) fn collect_id_lines(raw: &str) -> Vec<(usize, String)> {
    crate::services::roadmap_text::id_lines(raw)
}

/// Ids declared on more than one line, ordered by first occurrence, each with
/// every line it was declared on.
///
/// PMAT-676: a thin wrapper over `services::roadmap_text::duplicate_ids`.
pub(crate) fn duplicate_ids(raw: &str) -> Vec<(String, Vec<usize>)> {
    crate::services::roadmap_text::duplicate_ids(raw)
}

/// A schema violation in a single roadmap row, located by index and id.
#[derive(Debug, PartialEq, Eq)]
struct RowViolation {
    index: usize,
    id: Option<String>,
    message: String,
}

/// Validate every roadmap row independently, so one run reports every row that
/// is broken instead of only the first.
///
/// The strict `Roadmap` parse is a single serde pass and therefore stops at the
/// first violation. Issue #628 reports that conforming a ~1300-entry roadmap
/// took three fix-and-rerun cycles, one per violation *class* — each on a
/// different row. Re-deserialising row by row surfaces all of them at once.
///
/// Returns empty when the failure is structural (not a per-row problem), in
/// which case the single strict error is already the most useful output.
fn collect_row_violations(content: &str) -> Vec<RowViolation> {
    let Ok(doc) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(content) else {
        return Vec::new();
    };
    let Some(rows) = doc.get("roadmap").and_then(|r| r.as_sequence()) else {
        return Vec::new();
    };

    rows.iter()
        .enumerate()
        .filter_map(|(index, row)| {
            serde_yaml_ng::from_value::<crate::models::roadmap::RoadmapItem>(row.clone())
                .err()
                .map(|e| RowViolation {
                    index,
                    id: row
                        .get("id")
                        .and_then(serde_yaml_ng::Value::as_str)
                        .map(String::from),
                    message: e.to_string(),
                })
        })
        .collect()
}

/// Print every row-level violation, grouped so repeated classes are obvious.
fn print_row_violations(violations: &[RowViolation]) {
    use crate::cli::colors as c;

    if violations.is_empty() {
        return;
    }

    println!();
    println!(
        "{} {} row(s) with schema violations:",
        c::subheader("Found"),
        c::number(&violations.len().to_string())
    );
    for v in violations {
        let id = v.id.as_deref().unwrap_or("<no id>");
        println!("   roadmap[{}] {}: {}", v.index, c::path(id), v.message);
    }
    println!();
    println!(
        "{}",
        c::dim("All rows were checked, so this list is complete for row-level errors.")
    );
}

/// Handle work migrate command (Part B: UX Improvements)
///
/// Auto-fixes common roadmap.yaml issues.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_work_migrate(
    path: Option<PathBuf>,
    dry_run: bool,
    backup: bool,
    levels: bool,
) -> Result<()> {
    use crate::cli::colors as c;
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    // MACS-004: --levels migrates .pmat-work contract verification levels
    // and is independent of the roadmap.yaml migration below.
    if levels {
        return migrate_verification_levels(&project_path, dry_run);
    }
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    println!("{}", c::label(&format!("🔄 Migrating roadmap: {}", c::path(&roadmap_path.display().to_string()))));
    println!();

    if !roadmap_path.exists() {
        anyhow::bail!(
            "Roadmap not found: {}\n\nRun `pmat work init` to create one.",
            roadmap_path.display()
        );
    }

    let content = std::fs::read_to_string(&roadmap_path)?;
    let (new_content, changes) = normalize_status_values(&content);
    let suggestions = collect_quoting_suggestions(&content);

    // Advisory suggestions are reported but MUST NOT gate the write. They were
    // previously concatenated into `changes`, so a roadmap needing no migration
    // at all was still rewritten (and backed up) and reported as "Updated",
    // purely because a title tripped the advisory heuristic.
    print_migration_list("change(s) to apply:", &changes);
    print_migration_list(
        "suggestion(s) to review by hand (not applied automatically):",
        &suggestions,
    );

    if changes.is_empty() {
        println!(
            "{}",
            c::pass("No automatic migrations needed - roadmap is already up to date")
        );
        return Ok(());
    }

    if dry_run {
        println!("{}", c::dim("(Dry run - no changes made)"));
        return Ok(());
    }

    write_migration(&roadmap_path, &content, &new_content, backup)
}

/// Print a titled bullet list, or nothing at all when the list is empty.
fn print_migration_list(header: &str, items: &[String]) {
    use crate::cli::colors as c;

    if items.is_empty() {
        return;
    }
    println!(
        "{} {} {}",
        c::subheader("Found"),
        c::number(&items.len().to_string()),
        header
    );
    for item in items {
        println!("   • {}", item);
    }
    println!();
}

/// Status spellings rewritten to their canonical form by `pmat work migrate`.
const STATUS_MIGRATIONS: [(&str, &str); 13] = [
    ("status: done", "status: completed"),
    ("status: Done", "status: completed"),
    ("status: DONE", "status: completed"),
    ("status: finished", "status: completed"),
    ("status: in progress", "status: inprogress"),
    ("status: In Progress", "status: inprogress"),
    ("status: WIP", "status: inprogress"),
    ("status: wip", "status: inprogress"),
    ("status: stuck", "status: blocked"),
    ("status: on-hold", "status: blocked"),
    ("status: todo", "status: planned"),
    ("status: TODO", "status: planned"),
    ("status: open", "status: planned"),
];

/// Indicator characters that are only significant at the *start* of a YAML
/// plain scalar. `-`, `?` and `:` are handled separately because they are only
/// indicators when followed by a space.
const LEADING_INDICATORS: [char; 13] = [
    ',', '[', ']', '{', '}', '#', '&', '*', '!', '\'', '"', '%', '@',
];

/// Rewrite non-canonical status spellings, returning the new content and a
/// human-readable description of each substitution performed.
fn normalize_status_values(content: &str) -> (String, Vec<String>) {
    let mut new_content = content.to_string();
    let mut changes = Vec::new();

    for (old, new) in STATUS_MIGRATIONS {
        if new_content.contains(old) {
            changes.push(format!("Normalize status: {} → {}", old, new));
            new_content = new_content.replace(old, new);
        }
    }

    (new_content, changes)
}

/// Extract the inline value of a `title:` line, if this is one.
///
/// Returns `None` for non-title lines and for titles whose value lives on
/// following lines (block scalars and empty values).
fn title_value(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("- title:")
        .or_else(|| trimmed.strip_prefix("title:"))?;
    let value = rest.trim();
    if value.is_empty() || is_block_scalar_header(value) {
        return None;
    }
    Some(value)
}

/// `|`, `>` and their variants (`|-`, `>+`, `|2`) introduce a block scalar, so
/// the value is on subsequent lines and quoting does not apply.
fn is_block_scalar_header(value: &str) -> bool {
    let mut chars = value.chars();
    if !matches!(chars.next(), Some('|') | Some('>')) {
        return false;
    }
    chars.all(|c| matches!(c, '-' | '+') || c.is_ascii_digit())
}

/// Whether an unquoted YAML plain scalar actually needs quoting.
///
/// This checks the *value*, not the whole line. The previous implementation
/// tested the entire line against a list that included `:` — and every
/// `title:` line contains one by construction, so it advised quoting every
/// unquoted title (202 of 202 on this repo's own roadmap). It also flagged
/// characters such as `≤` and `→` that are perfectly legal in a plain scalar.
fn needs_quoting(value: &str) -> bool {
    if is_quoted(value) {
        return false;
    }

    // ": " ends a plain scalar; a trailing ':' makes the line read as a key.
    if value.contains(": ") || value.ends_with(':') {
        return true;
    }
    // " #" begins a trailing comment.
    if value.contains(" #") {
        return true;
    }

    let Some(first) = value.chars().next() else {
        return false;
    };
    if LEADING_INDICATORS.contains(&first) {
        return true;
    }
    // `-`, `?` and `:` are indicators only when followed by a space.
    matches!(first, '-' | '?' | ':') && value[first.len_utf8()..].starts_with(' ')
}

/// A value already wrapped in matching quotes needs no advice.
fn is_quoted(value: &str) -> bool {
    let mut chars = value.chars();
    let (Some(first), Some(last)) = (chars.next(), value.chars().next_back()) else {
        return false;
    };
    value.chars().count() >= 2 && (first == '"' || first == '\'') && first == last
}

/// Flag `title:` values that would not survive as unquoted YAML plain scalars.
///
/// This is a line-level heuristic, not a YAML parse, so it only ever advises —
/// `handle_work_migrate` never rewrites titles and, since the advisory was
/// decoupled from `changes`, never writes the file on account of one.
fn collect_quoting_suggestions(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| title_value(line).map(|value| (line, value)))
        .filter(|(_, value)| needs_quoting(value))
        .map(|(line, _)| format!("Consider quoting: {}", line.trim()))
        .collect()
}

/// Back up (optionally), write the migrated roadmap, and report whether the
/// result still parses.
fn write_migration(
    roadmap_path: &std::path::Path,
    original: &str,
    new_content: &str,
    backup: bool,
) -> Result<()> {
    use crate::cli::colors as c;

    if backup {
        let backup_path = roadmap_path.with_extension("yaml.bak");
        std::fs::write(&backup_path, original)?;
        println!(
            "{}",
            c::pass(&format!(
                "Created backup: {}",
                c::path(&backup_path.display().to_string())
            ))
        );
    }

    std::fs::write(roadmap_path, new_content)?;
    println!(
        "{}",
        c::pass(&format!(
            "Updated roadmap: {}",
            c::path(&roadmap_path.display().to_string())
        ))
    );

    if serde_yaml_ng::from_str::<crate::models::roadmap::Roadmap>(new_content).is_ok() {
        println!("{}", c::pass("Verified: updated roadmap is valid"));
    } else {
        println!(
            "{}",
            c::warn("Warning: updated roadmap may have issues - check manually")
        );
    }

    Ok(())
}

/// Handle work list-statuses command (Part B: UX Improvements)
///
/// Lists all valid status values with descriptions and aliases.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_work_list_statuses() -> Result<()> {
    use crate::cli::colors as c;
    println!("{}\n", c::subheader("📋 Valid Status Values"));
    println!("{}{:<15} {:<25} DESCRIPTION{}", c::BOLD, "STATUS", "ALIASES", c::RESET);
    println!("{}", c::separator());

    // GH #628: this table used to be a hand-maintained copy and had drifted --
    // it omitted `working`, which `from_string` accepts and the schema doc
    // documents. It now renders the one vocabulary the parser uses.
    for &(status, aliases, description) in crate::models::roadmap::ItemStatus::STATUS_TABLE {
        println!("{}{:<15}{} {:<25} {}", c::CYAN, status, c::RESET, aliases, description);
    }

    println!();
    println!("{}", c::dim("💡 All status values are case-insensitive and support hyphens/underscores."));
    println!("   {}", c::dim("Example: 'In-Progress', 'in_progress', 'InProgress', 'WIP' all work."));

    Ok(())
}

/// MACS-004: rewrite legacy `verification_level` strings in every
/// `.pmat-work/<TICKET>/contract.json`. Lenient-parseable values ("l4",
/// "L3 ", "L4 (kani_proof)") are canonicalized to recover intent; values
/// outside the ladder become "L0" plus an audit note in
/// `references.spec_sections` (no silent rewrite).
fn migrate_verification_levels(project_path: &std::path::Path, dry_run: bool) -> Result<()> {
    use crate::cli::colors as c;
    use crate::cli::handlers::work_verification_level::VerificationLevel;

    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        println!("{}", c::warn("No .pmat-work directory — nothing to migrate"));
        return Ok(());
    }

    let mut migrated = 0usize;
    let mut invalid = 0usize;
    let mut scanned = 0usize;

    for entry in std::fs::read_dir(&work_dir)
        .context("read .pmat-work")?
        .flatten()
    {
        let contract_path = entry.path().join("contract.json");
        let Some((mut value, raw)) = load_contract_level(&contract_path) else {
            continue;
        };
        scanned += 1;
        if VerificationLevel::parse_strict(&raw).is_some() {
            continue; // already canonical
        }

        let (new_level, note) = resolve_level(&raw);
        if note.is_downgrade() {
            invalid += 1;
        }

        println!(
            "  {} {}: '{}' -> {}",
            if dry_run {
                c::dim("[dry-run]")
            } else {
                c::pass("")
            },
            entry.file_name().to_string_lossy(),
            raw,
            new_level
        );
        migrated += 1;
        if dry_run {
            continue;
        }

        let audit = format!(
            "MIGRATION(MACS-004): verification_level '{}' -> {} ({})",
            raw,
            new_level,
            note.as_str()
        );
        apply_level_migration(&mut value, new_level, audit);
        let pretty = serde_json::to_string_pretty(&value).context("serialize contract")?;
        std::fs::write(&contract_path, pretty).context("write contract")?;
    }

    println!();
    println!(
        "{}",
        c::pass(&format!(
            "Level migration: {scanned} contract(s) scanned, {migrated} rewritten, {invalid} invalid -> L0"
        ))
    );
    Ok(())
}

/// Why a contract's verification level was rewritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelMigrationNote {
    /// A recognisable spelling was mapped onto its canonical form.
    Canonicalized,
    /// Nothing recognisable was found, so the level was reset to L0.
    Downgraded,
}

impl LevelMigrationNote {
    fn as_str(self) -> &'static str {
        match self {
            Self::Canonicalized => "canonicalized",
            Self::Downgraded => "invalid; downgraded",
        }
    }

    fn is_downgrade(self) -> bool {
        matches!(self, Self::Downgraded)
    }
}

/// Read a contract and pull out its raw `verification_level` string.
///
/// Returns `None` for anything unreadable, unparseable, or lacking the field —
/// migration skips those rather than failing the whole run.
fn load_contract_level(contract_path: &std::path::Path) -> Option<(serde_json::Value, String)> {
    let text = std::fs::read_to_string(contract_path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let raw = value.get("verification_level")?.as_str()?.to_string();
    Some((value, raw))
}

/// Map a non-canonical level spelling onto a canonical level, falling back to
/// the first whitespace-delimited token before giving up and downgrading to L0.
fn resolve_level(
    raw: &str,
) -> (
    crate::cli::handlers::work_verification_level::VerificationLevel,
    LevelMigrationNote,
) {
    use crate::cli::handlers::work_verification_level::VerificationLevel;

    let first_token = raw.split_whitespace().next().unwrap_or("");
    match VerificationLevel::parse_lenient(raw)
        .or_else(|| VerificationLevel::parse_lenient(first_token))
    {
        Some(level) => (level, LevelMigrationNote::Canonicalized),
        None => (VerificationLevel::L0, LevelMigrationNote::Downgraded),
    }
}

/// Write the new level into the contract and append an audit breadcrumb to
/// `references.spec_sections`, creating that block if it is absent.
fn apply_level_migration(
    value: &mut serde_json::Value,
    new_level: crate::cli::handlers::work_verification_level::VerificationLevel,
    audit: String,
) {
    value["verification_level"] = serde_json::Value::String(new_level.to_string());

    if let Some(sections) = value
        .get_mut("references")
        .and_then(|r| r.get_mut("spec_sections"))
        .and_then(|s| s.as_array_mut())
    {
        sections.push(serde_json::Value::String(audit));
    } else {
        value["references"] = serde_json::json!({
            "arxiv": [], "spec_sections": [audit],
            "five_whys_id": null, "oracle_context": null
        });
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod level_migration_tests {
    use super::*;

    fn write_contract_with_level(project: &std::path::Path, ticket: &str, level: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "work_item_id": ticket,
                "verification_level": level,
                "references": {"arxiv": [], "spec_sections": []}
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn read_level(project: &std::path::Path, ticket: &str) -> (String, Vec<String>) {
        let text = std::fs::read_to_string(
            project.join(".pmat-work").join(ticket).join("contract.json"),
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let level = v["verification_level"].as_str().unwrap().to_string();
        let notes = v["references"]["spec_sections"]
            .as_array()
            .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();
        (level, notes)
    }

    #[test]
    fn legacy_strings_interactive_to_typed() {
        let project = tempfile::tempdir().unwrap();
        write_contract_with_level(project.path(), "T-CASE", "l4");
        write_contract_with_level(project.path(), "T-ANNOT", "L4 (kani_proof)");
        write_contract_with_level(project.path(), "T-BAD", "strong");
        write_contract_with_level(project.path(), "T-OK", "L3");

        migrate_verification_levels(project.path(), false).unwrap();

        let (level, notes) = read_level(project.path(), "T-CASE");
        assert_eq!(level, "L4", "case corruption recovers intent");
        assert!(notes.iter().any(|n| n.contains("MIGRATION(MACS-004)")));

        let (level, _) = read_level(project.path(), "T-ANNOT");
        assert_eq!(level, "L4", "annotated variant recovers first token");

        let (level, notes) = read_level(project.path(), "T-BAD");
        assert_eq!(level, "L0", "invalid value downgrades to L0");
        assert!(
            notes.iter().any(|n| n.contains("'strong'") && n.contains("invalid")),
            "audit note records the original value: {notes:?}"
        );

        let (level, notes) = read_level(project.path(), "T-OK");
        assert_eq!(level, "L3", "canonical values untouched");
        assert!(notes.is_empty(), "no audit note for untouched contracts");
    }

    #[test]
    fn dry_run_leaves_files_untouched() {
        let project = tempfile::tempdir().unwrap();
        write_contract_with_level(project.path(), "T-DRY", "l5");
        migrate_verification_levels(project.path(), true).unwrap();
        let (level, notes) = read_level(project.path(), "T-DRY");
        assert_eq!(level, "l5");
        assert!(notes.is_empty());
    }

    #[test]
    fn contract_deserializes_typed_with_migration() {
        use crate::cli::handlers::work_contract::WorkContract;
        use crate::cli::handlers::work_verification_level::VerificationLevel;
        let parse = |raw: &str| -> VerificationLevel {
            let base = WorkContract::new("T".to_string(), "abc".to_string());
            let mut value = serde_json::to_value(&base).expect("to_value");
            value["verification_level"] = serde_json::Value::String(raw.to_string());
            let contract: WorkContract =
                serde_json::from_value(value).expect("contract parses");
            contract.verification_level
        };
        assert_eq!(parse("L4"), VerificationLevel::L4);
        assert_eq!(parse("l4"), VerificationLevel::L4, "lenient read migration");
        assert_eq!(parse("L4 (kani_proof)"), VerificationLevel::L4, "annotated");
        assert_eq!(parse("strong"), VerificationLevel::L0, "invalid -> L0");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod migrate_helper_tests {
    use super::*;

    #[test]
    fn normalize_rewrites_each_legacy_spelling() {
        let (out, changes) = normalize_status_values(
            "status: done\nstatus: WIP\nstatus: TODO\nstatus: on-hold\n",
        );
        assert!(out.contains("status: completed"));
        assert!(out.contains("status: inprogress"));
        assert!(out.contains("status: planned"));
        assert!(out.contains("status: blocked"));
        assert_eq!(changes.len(), 4, "one change entry per applied pattern");
    }

    #[test]
    fn normalize_is_idempotent_and_silent_on_canonical_input() {
        let canonical = "status: completed\nstatus: inprogress\nstatus: planned\n";
        let (out, changes) = normalize_status_values(canonical);
        assert_eq!(out, canonical);
        assert!(
            changes.is_empty(),
            "canonical input must report no migrations"
        );

        // Re-running over its own output must be a no-op.
        let (again, changes2) = normalize_status_values(&out);
        assert_eq!(again, out);
        assert!(changes2.is_empty());
    }

    /// Table of YAML plain-scalar hazards. The predicate examines the *value*
    /// after `title:`, not the whole line, so the `title:` prefix no longer
    /// supplies a `:` that flags every title in the file.
    #[test]
    fn needs_quoting_matches_yaml_plain_scalar_rules() {
        // Genuinely unsafe unquoted.
        for bad in [
            "Fix: the parser",   // ": " terminates the scalar
            "Refactor:",         // trailing ':' reads as a key
            "Cleanup # 3",       // " #" starts a comment
            "- leading dash",    // indicator + space
            "? leading question",
            ": leading colon",
            "[bracketed]",
            "{braced}",
            "&anchor",
            "*alias",
            "!tag",
            "#comment",
            "%directive",
            "@reserved",
            ",leading comma",
        ] {
            assert!(needs_quoting(bad), "expected {bad:?} to need quoting");
        }

        // Safe unquoted -- these were false positives before.
        for ok in [
            "Perfectly fine title",
            "Latency \u{2264} 5ms",       // non-ASCII is legal in a plain scalar
            "a \u{2192} b",
            "ratio 3:1",                  // ':' without a following space is fine
            "C++ and C#",                 // '#' not preceded by a space
            "issue-123",
            "50% faster",                 // '%' only matters at the start
            "user@example",
        ] {
            assert!(!needs_quoting(ok), "expected {ok:?} to be left alone");
        }
    }

    #[test]
    fn quoting_suggestions_only_flag_real_hazards() {
        let suggestions = collect_quoting_suggestions(
            "  title: Fix: the parser\n  \
             title: \"Already: quoted\"\n  \
             - title: Latency \u{2264} 5ms\n  \
             title: Plain title\n  \
             notes: Ignored: not a title\n",
        );

        assert_eq!(suggestions.len(), 1, "got {suggestions:?}");
        assert!(suggestions[0].contains("Fix: the parser"));
        assert!(!suggestions.iter().any(|s| s.contains("Already")));
        assert!(!suggestions.iter().any(|s| s.contains("Plain title")));
        assert!(!suggestions.iter().any(|s| s.contains("notes:")));
    }

    /// Block scalars put the value on following lines, so the header itself is
    /// never a quoting candidate. A naive "first char is an indicator" check
    /// flags `|` and `>`; this one must not.
    #[test]
    fn quoting_suggestions_skip_block_scalar_headers() {
        for header in ["  title: |\n", "  title: >\n", "  title: |-\n", "  title: >+\n", "  title: |2\n"] {
            assert!(
                collect_quoting_suggestions(header).is_empty(),
                "block scalar header {header:?} must not be flagged"
            );
        }
        // An empty inline value is likewise not a quoting problem.
        assert!(collect_quoting_suggestions("  title:\n").is_empty());
    }

    #[test]
    fn quoting_suggestions_treat_unterminated_quotes_as_hazards() {
        // Only a *matched* pair counts as quoted; a stray leading quote is
        // itself a YAML indicator and must still be reported.
        assert_eq!(collect_quoting_suggestions("  title: 'tis a test\n").len(), 1);
        assert_eq!(collect_quoting_suggestions("  title: \"unterminated\n").len(), 1);
        assert!(collect_quoting_suggestions("  title: 'closed'\n").is_empty());
        assert!(collect_quoting_suggestions("  title: \"closed\"\n").is_empty());
    }

    #[test]
    fn quoting_suggestions_skip_non_title_keys() {
        assert!(collect_quoting_suggestions("  notes: nothing: here\n").is_empty());
        assert!(collect_quoting_suggestions("  description: a: b\n").is_empty());
    }

    #[test]
    fn resolve_level_canonicalizes_annotates_and_downgrades() {
        use crate::cli::handlers::work_verification_level::VerificationLevel;

        let (level, note) = resolve_level("l4");
        assert_eq!(level, VerificationLevel::L4);
        assert_eq!(note, LevelMigrationNote::Canonicalized);
        assert!(!note.is_downgrade());

        // Falls back to the first token for annotated values.
        let (level, note) = resolve_level("L4 (kani_proof)");
        assert_eq!(level, VerificationLevel::L4);
        assert_eq!(note, LevelMigrationNote::Canonicalized);

        let (level, note) = resolve_level("strong");
        assert_eq!(level, VerificationLevel::L0);
        assert!(note.is_downgrade());
        assert_eq!(note.as_str(), "invalid; downgraded");
    }

    #[test]
    fn load_contract_level_skips_unusable_files() {
        let dir = tempfile::tempdir().unwrap();

        let missing = dir.path().join("nope.json");
        assert!(load_contract_level(&missing).is_none(), "missing file");

        let bad_json = dir.path().join("bad.json");
        std::fs::write(&bad_json, "{not json").unwrap();
        assert!(load_contract_level(&bad_json).is_none(), "unparseable");

        let no_field = dir.path().join("nofield.json");
        std::fs::write(&no_field, r#"{"work_item_id":"T"}"#).unwrap();
        assert!(load_contract_level(&no_field).is_none(), "field absent");

        let ok = dir.path().join("ok.json");
        std::fs::write(&ok, r#"{"verification_level":"L2"}"#).unwrap();
        let (_, raw) = load_contract_level(&ok).expect("readable contract");
        assert_eq!(raw, "L2");
    }

    #[test]
    fn apply_level_migration_creates_references_block_when_absent() {
        use crate::cli::handlers::work_verification_level::VerificationLevel;

        let mut value = serde_json::json!({"verification_level": "l1"});
        apply_level_migration(&mut value, VerificationLevel::L1, "audit-note".to_string());

        assert_eq!(value["verification_level"], "L1");
        assert_eq!(value["references"]["spec_sections"][0], "audit-note");
    }

    #[test]
    fn apply_level_migration_appends_to_existing_sections() {
        use crate::cli::handlers::work_verification_level::VerificationLevel;

        let mut value = serde_json::json!({
            "verification_level": "l1",
            "references": {"arxiv": [], "spec_sections": ["pre-existing"]}
        });
        apply_level_migration(&mut value, VerificationLevel::L1, "audit-note".to_string());

        let sections = value["references"]["spec_sections"].as_array().unwrap();
        assert_eq!(sections.len(), 2, "must append, not replace");
        assert_eq!(sections[0], "pre-existing");
        assert_eq!(sections[1], "audit-note");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod row_violation_tests {
    use super::*;

    /// The scenario from issue #628: three violation classes on three different
    /// rows, which the strict parse reports one run at a time.
    #[test]
    fn collects_every_broken_row_in_one_pass() {
        let yaml = "roadmap_version: \"1.0\"\nroadmap:\n  \
                    - id: OK-1\n    title: fine\n    status: planned\n  \
                    - id: BAD-TYPE\n    title: t\n    item_type: verification\n    status: planned\n  \
                    - id: BAD-STATUS\n    title: t\n    status: obsolete\n  \
                    - id: NO-STATUS\n    title: t\n";

        let violations = collect_row_violations(yaml);

        assert_eq!(violations.len(), 3, "got {violations:?}");
        assert_eq!(violations[0].index, 1);
        assert_eq!(violations[0].id.as_deref(), Some("BAD-TYPE"));
        // Was serde's stock "unknown variant"; `item_type` now reports like
        // `status` does, which is the last of #628's three asks.
        assert!(
            violations[0].message.contains("unknown item_type"),
            "got: {}",
            violations[0].message
        );
        assert!(
            violations[0].message.contains("task, epic, bug"),
            "the error must still enumerate the vocabulary: {}",
            violations[0].message
        );
        assert_eq!(violations[1].id.as_deref(), Some("BAD-STATUS"));
        assert!(violations[1].message.contains("unknown status"));
        assert_eq!(violations[2].id.as_deref(), Some("NO-STATUS"));
        assert!(violations[2].message.contains("missing field"));
    }

    #[test]
    fn reports_nothing_for_a_valid_roadmap() {
        let yaml = "roadmap_version: \"1.0\"\nroadmap:\n  - id: A\n    title: t\n    status: done\n";
        assert!(collect_row_violations(yaml).is_empty());
    }

    /// A structural failure is not a row problem; the single strict error is
    /// already the best output, so the collector must stay quiet.
    #[test]
    fn stays_quiet_on_structural_failures() {
        assert!(collect_row_violations("roadmap: not-a-sequence\n").is_empty());
        assert!(collect_row_violations("{{{ not yaml at all\n").is_empty());
        assert!(collect_row_violations("github_enabled: false\n").is_empty());
    }

    #[test]
    fn tolerates_rows_without_an_id() {
        let yaml = "roadmap_version: \"1.0\"\nroadmap:\n  - title: t\n    status: nonsense\n";
        let violations = collect_row_violations(yaml);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].id, None);
    }
}
