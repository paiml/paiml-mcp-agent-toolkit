//! PMAT compliance and migration handlers (GH-96)
//!
//! Implements `pmat comply` commands for project compliance checking and migration.

use crate::cli::commands::{ComplyCommands, ComplyOutputFormat};
use crate::models::project_metadata::ProjectMetadata;
use anyhow::{Context, Result};
use std::path::Path;

/// Handle comply command
pub async fn handle_comply_command(command: ComplyCommands) -> Result<()> {
    match command {
        ComplyCommands::Check {
            path,
            strict,
            failures_only,
            format,
        } => handle_comply_check(&path, strict, failures_only, format).await,

        ComplyCommands::Init { path, force } => handle_comply_init(&path, force).await,

        ComplyCommands::Migrate {
            path,
            version,
            dry_run,
            no_backup,
            force,
        } => handle_comply_migrate(&path, version, dry_run, no_backup, force).await,

        ComplyCommands::Diff {
            path,
            from,
            to,
            breaking_only,
        } => handle_comply_diff(&path, from, to, breaking_only).await,

        ComplyCommands::Update {
            path,
            hooks,
            config,
            dry_run,
        } => handle_comply_update(&path, hooks, config, dry_run).await,
    }
}

/// Handle `pmat comply check` - detect version drift and breaking changes
async fn handle_comply_check(
    project_path: &Path,
    strict: bool,
    failures_only: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    // Get current PMAT binary version
    let current_version = env!("CARGO_PKG_VERSION");

    // Check if project metadata exists
    if !ProjectMetadata::exists(project_path) {
        eprintln!("⚠️  No .pmat/project.toml found");
        eprintln!("   Run: pmat comply init");
        eprintln!();
        if strict {
            anyhow::bail!("Project is not tracking PMAT version (strict mode)");
        }
        return Ok(());
    }

    // Load project metadata
    let mut metadata = ProjectMetadata::load(project_path)
        .context("Failed to load project metadata")?;

    // Update last compliance check timestamp
    metadata.update_compliance_check();
    metadata.save(project_path)
        .context("Failed to save updated metadata")?;

    // Compare versions
    let project_version = &metadata.pmat.version;
    let versions_behind = calculate_versions_behind(project_version, current_version)?;

    // Detect breaking changes
    let breaking_changes = detect_breaking_changes(project_version, current_version);
    let unaccepted_breaking_changes: Vec<_> = breaking_changes
        .iter()
        .filter(|bc| !metadata.is_breaking_change_accepted(&bc.version))
        .collect();

    // Determine compliance status
    let is_compliant = versions_behind == 0 && unaccepted_breaking_changes.is_empty();

    // Output results
    match format {
        ComplyOutputFormat::Text => {
            output_compliance_text(
                project_version,
                current_version,
                versions_behind,
                &unaccepted_breaking_changes,
                is_compliant,
                failures_only,
            );
        }
        ComplyOutputFormat::Json => {
            output_compliance_json(
                project_version,
                current_version,
                versions_behind,
                &unaccepted_breaking_changes,
                is_compliant,
            )?;
        }
        ComplyOutputFormat::Markdown => {
            output_compliance_markdown(
                project_version,
                current_version,
                versions_behind,
                &unaccepted_breaking_changes,
                is_compliant,
            );
        }
    }

    // Exit with error if strict and non-compliant
    if strict && !is_compliant {
        anyhow::bail!("Project is not compliant with current PMAT version");
    }

    Ok(())
}

/// Handle `pmat comply init` - create .pmat/project.toml
async fn handle_comply_init(project_path: &Path, force: bool) -> Result<()> {
    let path = ProjectMetadata::get_path(project_path);

    if path.exists() && !force {
        eprintln!("✅ .pmat/project.toml already exists");
        eprintln!("   Use --force to overwrite");
        return Ok(());
    }

    let current_version = env!("CARGO_PKG_VERSION");
    let metadata = ProjectMetadata::new(current_version);

    metadata.save(project_path)
        .context("Failed to save project metadata")?;

    println!("✅ Created {}", path.display());
    println!("   PMAT version: {}", current_version);
    println!();
    println!("Next steps:");
    println!("  • Run: pmat comply check");
    println!("  • Commit: git add .pmat/project.toml");

    Ok(())
}

/// Handle `pmat comply migrate` - migrate project to latest standards
async fn handle_comply_migrate(
    _project_path: &Path,
    _version: Option<String>,
    dry_run: bool,
    _no_backup: bool,
    _force: bool,
) -> Result<()> {
    if dry_run {
        println!("🔍 Migration Plan (Dry Run)");
        println!();
        println!("This is a placeholder - full migration coming in future PR");
        println!();
        println!("Would migrate:");
        println!("  ✓ .pmat-gates.toml (if format changed)");
        println!("  ✓ .pmat/hooks/ (if API changed)");
        println!("  ✓ .pmat/project.toml (version bump)");
    } else {
        eprintln!("⚠️  Full migration not yet implemented");
        eprintln!("   This feature is coming in a future PR");
        eprintln!();
        eprintln!("For now:");
        eprintln!("  1. Update manually following changelog");
        eprintln!("  2. Run: pmat comply check");
    }

    Ok(())
}

/// Handle `pmat comply diff` - show changelog
async fn handle_comply_diff(
    _project_path: &Path,
    _from: Option<String>,
    _to: Option<String>,
    _breaking_only: bool,
) -> Result<()> {
    eprintln!("⚠️  Changelog diff not yet implemented");
    eprintln!("   This feature is coming in a future PR");
    eprintln!();
    eprintln!("For now, see:");
    eprintln!("  • CHANGELOG.md in repo");
    eprintln!("  • GitHub releases: https://github.com/paiml/paiml-mcp-agent-toolkit/releases");

    Ok(())
}

/// Handle `pmat comply update` - update hooks and configs
async fn handle_comply_update(
    _project_path: &Path,
    _hooks: bool,
    _config: bool,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("🔍 Update Plan (Dry Run)");
        println!();
        println!("This is a placeholder - full update coming in future PR");
    } else {
        eprintln!("⚠️  Automatic updates not yet implemented");
        eprintln!("   This feature is coming in a future PR");
    }

    Ok(())
}

// Helper functions

/// Calculate how many versions behind project is
fn calculate_versions_behind(project: &str, current: &str) -> Result<usize> {
    // Parse versions (assuming semver format like "2.205.0")
    let parse_version = |v: &str| -> Result<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid version format: {}", v);
        }
        Ok((
            parts[0].parse()?,
            parts[1].parse()?,
            parts[2].parse()?,
        ))
    };

    let (p_major, p_minor, p_patch) = parse_version(project)?;
    let (c_major, c_minor, c_patch) = parse_version(current)?;

    // Simple diff calculation (MINOR versions behind for weekly releases)
    if c_major != p_major {
        // Major version difference
        Ok((c_major - p_major) as usize * 1000)
    } else if c_minor > p_minor {
        Ok((c_minor - p_minor) as usize)
    } else if c_minor == p_minor && c_patch > p_patch {
        Ok(1) // Count patch as 1 version behind
    } else {
        Ok(0)
    }
}

/// Breaking change record
#[derive(Debug, Clone)]
struct BreakingChange {
    version: String,
    description: String,
}

/// Detect breaking changes between versions
fn detect_breaking_changes(from: &str, to: &str) -> Vec<BreakingChange> {
    // TODO: Load from changelog or database
    // For now, hardcode known breaking changes

    let all_breaking_changes = vec![
        BreakingChange {
            version: "2.180.0".to_string(),
            description: ".pmat-gates.toml format changed (added parallel_tests option)".to_string(),
        },
        BreakingChange {
            version: "2.195.0".to_string(),
            description: "Hook script API updated (new TDG enforcement)".to_string(),
        },
    ];

    // Filter breaking changes between versions
    all_breaking_changes
        .into_iter()
        .filter(|bc| version_is_between(&bc.version, from, to))
        .collect()
}

/// Check if version is between from and to (inclusive)
fn version_is_between(version: &str, from: &str, to: &str) -> bool {
    version > from && version <= to
}

/// Output compliance report in text format
fn output_compliance_text(
    project_version: &str,
    current_version: &str,
    versions_behind: usize,
    breaking_changes: &[&BreakingChange],
    is_compliant: bool,
    failures_only: bool,
) {
    if !failures_only {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔍 PMAT Compliance Check");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("Project PMAT: v{}", project_version);
        println!("Current PMAT: v{}", current_version);
    }

    if versions_behind > 0 {
        println!();
        println!("⚠️  {} version(s) behind", versions_behind);
    }

    if !breaking_changes.is_empty() {
        println!();
        println!("❌ {} breaking change(s) detected:", breaking_changes.len());
        for bc in breaking_changes {
            println!("   • v{}: {}", bc.version, bc.description);
        }
        println!();
        println!("💡 Run: pmat comply migrate");
    } else if versions_behind > 0 {
        println!();
        println!("✅ No breaking changes (safe to update)");
        println!("💡 Run: cargo install --locked pmat");
    }

    if is_compliant && !failures_only {
        println!();
        println!("✅ Project is compliant with current PMAT version");
    }

    if !failures_only {
        println!();
    }
}

/// Output compliance report in JSON format
fn output_compliance_json(
    project_version: &str,
    current_version: &str,
    versions_behind: usize,
    breaking_changes: &[&BreakingChange],
    is_compliant: bool,
) -> Result<()> {
    let json = serde_json::json!({
        "project_version": project_version,
        "current_version": current_version,
        "versions_behind": versions_behind,
        "breaking_changes": breaking_changes.iter().map(|bc| serde_json::json!({
            "version": bc.version,
            "description": bc.description,
        })).collect::<Vec<_>>(),
        "is_compliant": is_compliant,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// Output compliance report in Markdown format
fn output_compliance_markdown(
    project_version: &str,
    current_version: &str,
    versions_behind: usize,
    breaking_changes: &[&BreakingChange],
    is_compliant: bool,
) {
    println!("# PMAT Compliance Report");
    println!();
    println!("**Project PMAT**: v{}", project_version);
    println!("**Current PMAT**: v{}", current_version);
    println!("**Status**: {}", if is_compliant { "✅ Compliant" } else { "❌ Non-Compliant" });
    println!();

    if versions_behind > 0 {
        println!("## ⚠️  Version Drift");
        println!();
        println!("Project is **{} version(s) behind** current PMAT.", versions_behind);
        println!();
    }

    if !breaking_changes.is_empty() {
        println!("## ❌ Breaking Changes");
        println!();
        for bc in breaking_changes {
            println!("- **v{}**: {}", bc.version, bc.description);
        }
        println!();
        println!("**Action Required**: Run `pmat comply migrate` to update.");
        println!();
    }

    if is_compliant {
        println!("## ✅ Compliance Status");
        println!();
        println!("Project is fully compliant with current PMAT version.");
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_versions_behind() {
        // Same version
        assert_eq!(calculate_versions_behind("2.205.0", "2.205.0").unwrap(), 0);

        // Minor versions behind
        assert_eq!(calculate_versions_behind("2.200.0", "2.205.0").unwrap(), 5);
        assert_eq!(calculate_versions_behind("2.150.0", "2.205.0").unwrap(), 55);

        // Patch version behind
        assert_eq!(calculate_versions_behind("2.205.0", "2.205.1").unwrap(), 1);

        // Ahead (should be 0 or handled gracefully)
        assert_eq!(calculate_versions_behind("2.206.0", "2.205.0").unwrap(), 0);
    }

    #[test]
    fn test_version_is_between() {
        assert!(version_is_between("2.180.0", "2.150.0", "2.205.0"));
        assert!(version_is_between("2.195.0", "2.150.0", "2.205.0"));
        assert!(!version_is_between("2.100.0", "2.150.0", "2.205.0"));
        assert!(!version_is_between("2.210.0", "2.150.0", "2.205.0"));
    }

    #[test]
    fn test_detect_breaking_changes() {
        let changes = detect_breaking_changes("2.150.0", "2.205.0");
        assert_eq!(changes.len(), 2);
        assert!(changes.iter().any(|bc| bc.version == "2.180.0"));
        assert!(changes.iter().any(|bc| bc.version == "2.195.0"));

        let no_changes = detect_breaking_changes("2.200.0", "2.205.0");
        assert_eq!(no_changes.len(), 0);
    }
}
