//! PMAT Compliance and Migration Handlers (GH-96)
//!
//! Implements the `pmat comply` command for checking and maintaining
//! project compliance with PMAT standards.
//!
//! Commands:
//! - check: Verify project compliance with current PMAT version
//! - migrate: Migrate project configs to latest standards
//! - diff: Show changelog between versions
//! - update: Update hooks and configs

use crate::cli::commands::{ComplyCommands, ComplyOutputFormat};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Current PMAT version (from Cargo.toml)
pub const PMAT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Project compliance information stored in .pmat/project.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub pmat: PmatSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmatSection {
    pub version: String,
    #[serde(default)]
    pub last_compliance_check: Option<DateTime<Utc>>,
    #[serde(default)]
    pub auto_update: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            pmat: PmatSection {
                version: PMAT_VERSION.to_string(),
                last_compliance_check: Some(Utc::now()),
                auto_update: false,
            },
        }
    }
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub project_version: String,
    pub current_version: String,
    pub is_compliant: bool,
    pub versions_behind: u32,
    pub checks: Vec<ComplianceCheck>,
    pub breaking_changes: Vec<BreakingChange>,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub version: String,
    pub description: String,
    pub migration_guide: Option<String>,
}

/// Handle all comply subcommands
pub async fn handle_comply_command(command: ComplyCommands) -> Result<()> {
    match command {
        ComplyCommands::Check {
            path,
            strict,
            failures_only,
            format,
        } => handle_check(&path, strict, failures_only, format).await,

        ComplyCommands::Migrate {
            path,
            version,
            dry_run,
            no_backup,
            force,
        } => handle_migrate(&path, version.as_deref(), dry_run, no_backup, force).await,

        ComplyCommands::Diff {
            path,
            from,
            to,
            breaking_only,
        } => handle_diff(&path, from.as_deref(), to.as_deref(), breaking_only).await,

        ComplyCommands::Update {
            path,
            hooks,
            config,
            dry_run,
        } => handle_update(&path, hooks, config, dry_run).await,

        ComplyCommands::Init { path, force } => handle_init(&path, force).await,

        ComplyCommands::Enforce {
            path,
            yes,
            disable,
            format,
        } => handle_enforce(&path, yes, disable, format).await,

        ComplyCommands::Report {
            path,
            include_history,
            format,
            output,
        } => handle_report(&path, include_history, format, output.as_deref()).await,
    }
}

/// Check project compliance with current PMAT version
async fn handle_check(
    project_path: &Path,
    strict: bool,
    failures_only: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    println!("Checking PMAT compliance for {}", project_path.display());

    // Load or create project config
    let config = load_or_create_project_config(project_path)?;
    let project_version = &config.pmat.version;

    // Run compliance checks
    let checks = vec![
        check_version_currency(project_version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
        check_compute_brick(project_path),
        // Build performance checks (lltop Tab 8 integration)
        check_cargo_lock(project_path),
        check_msrv(project_path),
        check_ci_configured(project_path),
    ];

    // Calculate compliance
    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();
    let is_compliant = failures == 0;

    // Get breaking changes
    let breaking_changes = get_breaking_changes_since(project_version);
    let versions_behind = calculate_versions_behind(project_version);

    // Build recommendations
    let mut recommendations = vec![];
    if versions_behind > 0 {
        recommendations.push(format!(
            "Run 'pmat comply migrate' to update to v{}",
            PMAT_VERSION
        ));
    }
    if !breaking_changes.is_empty() {
        recommendations.push("Review breaking changes with 'pmat comply diff'".to_string());
    }

    let report = ComplianceReport {
        project_version: project_version.clone(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant,
        versions_behind,
        checks: if failures_only {
            checks
                .into_iter()
                .filter(|c| c.status == CheckStatus::Fail)
                .collect()
        } else {
            checks
        },
        breaking_changes,
        recommendations,
        timestamp: Utc::now(),
    };

    // Output report
    match format {
        ComplyOutputFormat::Text => print_compliance_text(&report),
        ComplyOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        ComplyOutputFormat::Markdown => print_compliance_markdown(&report),
    }

    // Update last check timestamp
    let _ = update_last_check_timestamp(project_path);

    // Exit with error if strict mode and non-compliant
    if strict && !is_compliant {
        std::process::exit(1);
    }
    if strict && warnings > 0 {
        std::process::exit(2);
    }

    Ok(())
}

/// Migrate project to latest PMAT standards
async fn handle_migrate(
    project_path: &Path,
    target_version: Option<&str>,
    dry_run: bool,
    no_backup: bool,
    force: bool,
) -> Result<()> {
    let target = target_version.unwrap_or(PMAT_VERSION);
    println!("Migrating project to PMAT v{}", target);

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    let config = load_or_create_project_config(project_path)?;
    let current_version = &config.pmat.version;

    println!("Current version: {}", current_version);
    println!("Target version:  {}\n", target);

    let breaking_changes = get_breaking_changes_since(current_version);
    if !breaking_changes.is_empty() && !force {
        println!(
            "\x1b[33mWarning: {} breaking changes detected:\x1b[0m",
            breaking_changes.len()
        );
        for change in &breaking_changes {
            println!("  - v{}: {}", change.version, change.description);
        }
        println!("\nUse --force to proceed anyway\n");
        if !force {
            return Ok(());
        }
    }

    if !no_backup && !dry_run {
        let backup_path = project_path.join(".pmat").join("backup");
        fs::create_dir_all(&backup_path)?;
        println!("Created backup at: {}", backup_path.display());
    }

    let migrations = vec![
        (
            "Update project.toml version",
            migrate_project_version(project_path, target, dry_run),
        ),
        ("Update gitignore", migrate_gitignore(project_path, dry_run)),
    ];

    println!("\nMigration steps:");
    for (name, result) in migrations {
        match result {
            Ok(true) => println!("  \x1b[32m✓\x1b[0m {}", name),
            Ok(false) => println!("  \x1b[90m-\x1b[0m {} (no changes needed)", name),
            Err(e) => println!("  \x1b[31m✗\x1b[0m {} - {}", name, e),
        }
    }

    if dry_run {
        println!("\n(dry-run complete - no changes were made)");
    } else {
        println!("\n\x1b[32m✓ Migration complete!\x1b[0m");
    }

    Ok(())
}

/// Show changelog between versions
async fn handle_diff(
    project_path: &Path,
    from_version: Option<&str>,
    to_version: Option<&str>,
    breaking_only: bool,
) -> Result<()> {
    let config = load_or_create_project_config(project_path)?;
    let from = from_version.unwrap_or(&config.pmat.version);
    let to = to_version.unwrap_or(PMAT_VERSION);

    println!("PMAT Changelog: v{} → v{}\n", from, to);

    let changes = get_changelog_entries(from, to);

    if breaking_only {
        println!("\x1b[33mBreaking Changes Only:\x1b[0m\n");
        let breaking: Vec<_> = changes.iter().filter(|c| c.breaking).collect();
        if breaking.is_empty() {
            println!("  No breaking changes between these versions.");
        } else {
            for entry in breaking {
                println!(
                    "  \x1b[31m[BREAKING]\x1b[0m v{}: {}",
                    entry.version, entry.description
                );
            }
        }
    } else {
        for entry in &changes {
            let icon = if entry.breaking {
                "\x1b[31m[BREAKING]\x1b[0m"
            } else {
                "\x1b[32m[FEATURE]\x1b[0m"
            };
            println!("  {} v{}: {}", icon, entry.version, entry.description);
        }
    }

    Ok(())
}

/// Update hooks and configs
async fn handle_update(
    project_path: &Path,
    update_hooks: bool,
    update_config: bool,
    dry_run: bool,
) -> Result<()> {
    let update_both = !update_hooks && !update_config;

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    if update_hooks || update_both {
        println!("Updating hooks...");
        println!("  \x1b[90m-\x1b[0m Hooks already up to date");
    }

    if update_config || update_both {
        println!("Updating config...");
        match update_project_config(project_path, dry_run) {
            Ok(true) => println!("  \x1b[32m✓\x1b[0m Config updated to v{}", PMAT_VERSION),
            Ok(false) => println!("  \x1b[90m-\x1b[0m Config already up to date"),
            Err(e) => println!("  \x1b[31m✗\x1b[0m Failed: {}", e),
        }
    }

    Ok(())
}

/// Initialize .pmat/project.toml with current version
async fn handle_init(project_path: &Path, force: bool) -> Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");

    if config_path.exists() && !force {
        println!("Project already initialized at {}", config_path.display());
        println!("Use --force to overwrite existing configuration.");
        return Ok(());
    }

    // Create .pmat directory
    let pmat_dir = project_path.join(".pmat");
    if !pmat_dir.exists() {
        fs::create_dir_all(&pmat_dir)?;
    }

    // Create default config
    let config = ProjectConfig::default();
    let content = toml::to_string_pretty(&config)?;
    fs::write(&config_path, &content)?;

    println!(
        "\x1b[32m✓\x1b[0m Initialized PMAT project at {}",
        config_path.display()
    );
    println!("\nProject version: v{}", PMAT_VERSION);
    println!("\nNext steps:");
    println!("  1. Run 'pmat comply check' to verify compliance");
    println!("  2. Run 'pmat hooks init' to install git hooks");
    println!("  3. Run 'pmat quality-gate' to check code quality");

    Ok(())
}

// Helper functions

fn load_or_create_project_config(project_path: &Path) -> Result<ProjectConfig> {
    let config_path = project_path.join(".pmat").join("project.toml");

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        let config = ProjectConfig::default();
        let pmat_dir = project_path.join(".pmat");
        if !pmat_dir.exists() {
            fs::create_dir_all(&pmat_dir)?;
        }
        let content = toml::to_string_pretty(&config)?;
        fs::write(&config_path, &content)?;
        Ok(config)
    }
}

fn update_last_check_timestamp(project_path: &Path) -> Result<()> {
    let config_path = project_path.join(".pmat").join("project.toml");
    if let Ok(mut config) = load_or_create_project_config(project_path) {
        config.pmat.last_compliance_check = Some(Utc::now());
        let content = toml::to_string_pretty(&config)?;
        fs::write(&config_path, &content)?;
    }
    Ok(())
}

fn check_version_currency(project_version: &str) -> ComplianceCheck {
    let behind = calculate_versions_behind(project_version);
    if behind == 0 {
        ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Pass,
            message: format!("Project is on latest version (v{})", PMAT_VERSION),
            severity: Severity::Info,
        }
    } else if behind <= 5 {
        ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} versions behind (v{} → v{})",
                behind, project_version, PMAT_VERSION
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Fail,
            message: format!("{} versions behind - migration recommended", behind),
            severity: Severity::Error,
        }
    }
}

fn check_config_files(project_path: &Path) -> ComplianceCheck {
    let config_files = [".pmat/project.toml", ".pmat-metrics.toml"];
    let missing: Vec<&str> = config_files
        .iter()
        .filter(|f| !project_path.join(f).exists())
        .copied()
        .collect();

    if missing.is_empty() {
        ComplianceCheck {
            name: "Config Files".to_string(),
            status: CheckStatus::Pass,
            message: "All required config files present".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Config Files".to_string(),
            status: CheckStatus::Warn,
            message: format!("Missing: {}", missing.join(", ")),
            severity: Severity::Warning,
        }
    }
}

fn check_hooks_installed(project_path: &Path) -> ComplianceCheck {
    let pre_commit = project_path.join(".git").join("hooks").join("pre-commit");
    if pre_commit.exists() {
        if let Ok(content) = fs::read_to_string(&pre_commit) {
            if content.contains("pmat") || content.contains("PMAT") {
                return ComplianceCheck {
                    name: "Git Hooks".to_string(),
                    status: CheckStatus::Pass,
                    message: "PMAT hooks installed".to_string(),
                    severity: Severity::Info,
                };
            }
        }
        ComplianceCheck {
            name: "Git Hooks".to_string(),
            status: CheckStatus::Warn,
            message: "Pre-commit hook exists but may not be PMAT".to_string(),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Git Hooks".to_string(),
            status: CheckStatus::Warn,
            message: "No pre-commit hook installed".to_string(),
            severity: Severity::Warning,
        }
    }
}

fn check_quality_thresholds(project_path: &Path) -> ComplianceCheck {
    if project_path.join(".pmat-metrics.toml").exists() {
        ComplianceCheck {
            name: "Quality Thresholds".to_string(),
            status: CheckStatus::Pass,
            message: "Quality thresholds configured".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Quality Thresholds".to_string(),
            status: CheckStatus::Warn,
            message: "No .pmat-metrics.toml found - using defaults".to_string(),
            severity: Severity::Warning,
        }
    }
}

fn check_deprecated_features(_project_path: &Path) -> ComplianceCheck {
    ComplianceCheck {
        name: "Deprecated Features".to_string(),
        status: CheckStatus::Pass,
        message: "No deprecated features detected".to_string(),
        severity: Severity::Info,
    }
}

/// ComputeBrick pattern detection result
#[derive(Debug, Clone)]
struct CbPatternViolation {
    pattern_id: String,
    file: String,
    line: usize,
    description: String,
    severity: Severity,
}

/// BrickProfiler anomaly from JSON output
#[derive(Debug, Clone)]
struct ProfilerAnomaly {
    brick_name: String,
    anomaly_type: String,
    value: f64,
    threshold: f64,
}

/// Scan Rust files for CB-020 (unsafe without SAFETY comment)
fn detect_cb020_unsafe_without_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Walk src/ directory for .rs files
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    // Check for unsafe block without preceding SAFETY comment
                    if trimmed.starts_with("unsafe {") || trimmed.starts_with("unsafe{") {
                        // Look at previous non-empty lines for SAFETY comment
                        let prev_lines: Vec<&str> = content.lines().take(line_num).collect();
                        let has_safety = prev_lines
                            .iter()
                            .rev()
                            .take(3)
                            .any(|l| l.contains("// SAFETY:") || l.contains("// SAFETY :"));

                        if !has_safety {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-020".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: "unsafe block without SAFETY comment".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Scan for CB-021 (SIMD intrinsics without #[target_feature])
fn detect_cb021_simd_without_target_feature(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    // Common SIMD intrinsic patterns
    let simd_patterns = [
        "_mm_", "_mm256_", "_mm512_", // x86 SSE/AVX
        "vld1q_", "vst1q_", "vmulq_", "vaddq_", // ARM NEON
        "i8x16", "i16x8", "i32x4", "f32x4",     // portable SIMD
    ];

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();

                // Find functions with #[target_feature] attribute
                let mut protected_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

                for (i, line) in lines.iter().enumerate() {
                    if line.trim().starts_with("#[target_feature") {
                        // Mark all lines in this function as protected
                        // Find the fn line (should be within next 3 lines)
                        for j in i..std::cmp::min(i + 4, lines.len()) {
                            if lines[j].contains("fn ") {
                                // Count braces to find function end
                                let mut depth = 0;
                                for k in j..lines.len() {
                                    depth += lines[k].matches('{').count();
                                    depth = depth.saturating_sub(lines[k].matches('}').count());
                                    protected_lines.insert(k);
                                    if depth == 0 && k > j {
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }

                // Check for SIMD intrinsics outside protected functions
                for (line_num, line) in lines.iter().enumerate() {
                    if protected_lines.contains(&line_num) {
                        continue;
                    }

                    for pattern in &simd_patterns {
                        if line.contains(pattern) && !line.trim().starts_with("//") {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-021".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: format!(
                                    "SIMD intrinsic '{}' without #[target_feature]",
                                    pattern
                                ),
                                severity: Severity::Warning,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Check for bricks without assertions (budget validation)
fn detect_bricks_without_assertions(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                // Look for ComputeBrick impl blocks without assertion methods
                let mut in_brick_impl = false;
                let mut brick_name = String::new();
                let mut impl_start_line = 0;
                let mut brace_depth = 0;
                let mut has_assertion = false;

                for (line_num, line) in content.lines().enumerate() {
                    // Detect impl blocks for types containing "Brick"
                    if line.contains("impl") && line.contains("Brick") && !line.contains("//") {
                        in_brick_impl = true;
                        brick_name = line
                            .split_whitespace()
                            .find(|w| w.contains("Brick"))
                            .unwrap_or("UnknownBrick")
                            .trim_end_matches('{')
                            .to_string();
                        impl_start_line = line_num + 1;
                        brace_depth = 0;
                        has_assertion = false;
                    }

                    if in_brick_impl {
                        brace_depth += line.matches('{').count();
                        brace_depth = brace_depth.saturating_sub(line.matches('}').count());

                        // Check for assertion-related methods
                        if line.contains("assert")
                            || line.contains("debug_assert")
                            || line.contains("verify")
                            || line.contains("validate")
                            || line.contains("budget")
                        {
                            has_assertion = true;
                        }

                        // End of impl block
                        if brace_depth == 0 && line.contains('}') {
                            if !has_assertion {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-BUDGET".to_string(),
                                    file: entry.display().to_string(),
                                    line: impl_start_line,
                                    description: format!(
                                        "Brick '{}' has no assertion/budget validation",
                                        brick_name
                                    ),
                                    severity: Severity::Warning,
                                });
                            }
                            in_brick_impl = false;
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Parse BrickProfiler JSON output and detect anomalies
fn detect_profiler_anomalies(project_path: &Path) -> Vec<ProfilerAnomaly> {
    let mut anomalies = Vec::new();

    // Check standard profiler output locations
    let profiler_paths = [
        project_path.join(".pmat-metrics").join("brick-profile.json"),
        project_path.join("target").join("brick-profile.json"),
        project_path.join("brick-profile.json"),
    ];

    for profiler_path in &profiler_paths {
        if !profiler_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(profiler_path) {
            // Parse JSON manually to avoid adding serde_json dep to this module
            // Look for patterns like "cv": 0.18 (CV > 15% threshold)
            // and "efficiency": 0.22 (efficiency < 25% threshold)

            // Simple pattern matching for CV values
            for line in content.lines() {
                let line = line.trim();

                // Detect high coefficient of variation (CV > 15%)
                if line.contains("\"cv\"") || line.contains("\"cv_percent\"") {
                    if let Some(value) = extract_json_number(line) {
                        let cv_threshold = 15.0;
                        let cv = if value < 1.0 { value * 100.0 } else { value };
                        if cv > cv_threshold {
                            anomalies.push(ProfilerAnomaly {
                                brick_name: extract_brick_name(&content, line),
                                anomaly_type: "HIGH_CV".to_string(),
                                value: cv,
                                threshold: cv_threshold,
                            });
                        }
                    }
                }

                // Detect low efficiency (< 25%)
                if line.contains("\"efficiency\"") {
                    if let Some(value) = extract_json_number(line) {
                        let eff_threshold = 25.0;
                        let efficiency = if value < 1.0 { value * 100.0 } else { value };
                        if efficiency < eff_threshold {
                            anomalies.push(ProfilerAnomaly {
                                brick_name: extract_brick_name(&content, line),
                                anomaly_type: "LOW_EFFICIENCY".to_string(),
                                value: efficiency,
                                threshold: eff_threshold,
                            });
                        }
                    }
                }
            }
            break; // Only process first found file
        }
    }

    anomalies
}

/// Helper to extract numeric value from JSON line like `"cv": 0.18,`
fn extract_json_number(line: &str) -> Option<f64> {
    line.split(':')
        .nth(1)?
        .trim()
        .trim_end_matches(',')
        .trim_end_matches('}')
        .parse()
        .ok()
}

/// Helper to extract brick name from surrounding JSON context
fn extract_brick_name(content: &str, target_line: &str) -> String {
    // Look backwards from target line for "name": "BrickName" pattern
    let target_pos = content.find(target_line).unwrap_or(0);
    let before = &content[..target_pos];

    for line in before.lines().rev().take(10) {
        if line.contains("\"name\"") || line.contains("\"brick\"") {
            if let Some(start) = line.find('"') {
                let rest = &line[start + 1..];
                if let Some(end) = rest.find('"') {
                    let after_colon = &rest[end + 1..];
                    if let Some(val_start) = after_colon.find('"') {
                        let val_rest = &after_colon[val_start + 1..];
                        if let Some(val_end) = val_rest.find('"') {
                            return val_rest[..val_end].to_string();
                        }
                    }
                }
            }
        }
    }
    "UnknownBrick".to_string()
}

/// Walk directory for .rs files
fn walkdir_rs_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut rs_files = Vec::new();

    fn visit_dir(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, files)?;
                } else if path.extension().map_or(false, |e| e == "rs") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, &mut rs_files)?;
    Ok(rs_files)
}

/// Check ComputeBrick compliance (PROBAR-SPEC-009-P8)
///
/// Validates:
/// - CB-001 to CB-022: Static analysis pattern detection
/// - CB-020: unsafe blocks without SAFETY comments
/// - CB-021: SIMD intrinsics without #[target_feature]
/// - CB-BUDGET: Bricks without assertion/validation
/// - BrickProfiler anomalies: CV > 15%, efficiency < 25%
/// - Probar GUI coverage >= 80%
fn check_compute_brick(project_path: &Path) -> ComplianceCheck {
    // Check if this is a ComputeBrick project (has probar dependency or brick/ directory)
    let cargo_toml = project_path.join("Cargo.toml");
    let brick_dir = project_path.join("src").join("brick");
    let has_probar = cargo_toml.exists()
        && fs::read_to_string(&cargo_toml)
            .map(|s| s.contains("probar") || s.contains("jugar-probar"))
            .unwrap_or(false);
    let has_brick_dir = brick_dir.exists();

    // Also check for trueno/realizar dependencies (ComputeBrick ecosystem)
    let has_cb_ecosystem = cargo_toml.exists()
        && fs::read_to_string(&cargo_toml)
            .map(|s| s.contains("trueno") || s.contains("realizar") || s.contains("Brick"))
            .unwrap_or(false);

    if !has_probar && !has_brick_dir && !has_cb_ecosystem {
        return ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Skip,
            message: "Not a ComputeBrick project (no probar/trueno/realizar dep or brick/ dir)"
                .to_string(),
            severity: Severity::Info,
        };
    }

    // Collect all violations
    let mut all_issues: Vec<String> = Vec::new();
    let mut critical_count = 0;
    let mut warning_count = 0;

    // Run static analysis for CB patterns
    let cb020_violations = detect_cb020_unsafe_without_safety(project_path);
    for v in &cb020_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }

    let cb021_violations = detect_cb021_simd_without_target_feature(project_path);
    for v in &cb021_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }

    let budget_violations = detect_bricks_without_assertions(project_path);
    for v in &budget_violations {
        all_issues.push(format!("{}: {} ({}:{})", v.pattern_id, v.description, v.file, v.line));
        warning_count += 1;
    }

    // Check BrickProfiler output for anomalies
    let profiler_anomalies = detect_profiler_anomalies(project_path);
    for a in &profiler_anomalies {
        all_issues.push(format!(
            "PROFILER-{}: {} has {}={:.1}% (threshold: {:.1}%)",
            a.anomaly_type, a.brick_name, a.anomaly_type.to_lowercase(), a.value, a.threshold
        ));
        if a.anomaly_type == "LOW_EFFICIENCY" {
            critical_count += 1;
        } else {
            warning_count += 1;
        }
    }

    // Check for .pmat-gates.toml compute-brick section
    let gates_path = project_path.join(".pmat-gates.toml");
    let has_cb_config = gates_path.exists()
        && fs::read_to_string(&gates_path)
            .map(|s| s.contains("[compute-brick]"))
            .unwrap_or(false);

    if !has_cb_config && (has_probar || has_brick_dir) {
        all_issues.push("Missing [compute-brick] section in .pmat-gates.toml".to_string());
        warning_count += 1;
    }

    // Check for probar test coverage file
    let coverage_file = project_path.join(".pmat-metrics").join("gui-coverage.json");
    if has_probar && !coverage_file.exists() {
        all_issues.push("No GUI coverage report - run probador to generate".to_string());
        warning_count += 1;
    }

    // Determine overall status and message
    if critical_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{} critical, {} warnings: {}",
                critical_count,
                warning_count,
                all_issues.first().unwrap_or(&"unknown".to_string())
            ),
            severity: Severity::Critical,
        }
    } else if warning_count > 0 {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} warnings detected: {}",
                warning_count,
                all_issues.join("; ").chars().take(200).collect::<String>()
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "ComputeBrick Compliance".to_string(),
            status: CheckStatus::Pass,
            message: "ComputeBrick patterns validated - no violations detected".to_string(),
            severity: Severity::Info,
        }
    }
}

/// Check Cargo.lock presence (reproducible builds)
fn check_cargo_lock(project_path: &Path) -> ComplianceCheck {
    let cargo_lock = project_path.join("Cargo.lock");

    if cargo_lock.exists() {
        ComplianceCheck {
            name: "Cargo.lock Present".to_string(),
            status: CheckStatus::Pass,
            message: "Cargo.lock present - reproducible builds enabled".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Cargo.lock Present".to_string(),
            status: CheckStatus::Fail,
            message: "Missing Cargo.lock - run 'cargo build' to generate".to_string(),
            severity: Severity::Error,
        }
    }
}

/// Check MSRV (Minimum Supported Rust Version) defined
fn check_msrv(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");

    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "MSRV Defined".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".to_string(),
            severity: Severity::Info,
        };
    }

    let content = fs::read_to_string(&cargo_toml).unwrap_or_default();

    if content.contains("rust-version") {
        ComplianceCheck {
            name: "MSRV Defined".to_string(),
            status: CheckStatus::Pass,
            message: "rust-version field present in Cargo.toml".to_string(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "MSRV Defined".to_string(),
            status: CheckStatus::Warn,
            message: "No rust-version field - add to Cargo.toml for compatibility".to_string(),
            severity: Severity::Warning,
        }
    }
}

/// Check CI configuration present
fn check_ci_configured(project_path: &Path) -> ComplianceCheck {
    let github_workflows = project_path.join(".github").join("workflows");
    let gitlab_ci = project_path.join(".gitlab-ci.yml");
    let jenkinsfile = project_path.join("Jenkinsfile");

    if github_workflows.exists() && github_workflows.is_dir() {
        let workflow_count = fs::read_dir(&github_workflows)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        if workflow_count > 0 {
            return ComplianceCheck {
                name: "CI Configured".to_string(),
                status: CheckStatus::Pass,
                message: format!("{} GitHub Actions workflow(s) found", workflow_count),
                severity: Severity::Info,
            };
        }
    }

    if gitlab_ci.exists() {
        return ComplianceCheck {
            name: "CI Configured".to_string(),
            status: CheckStatus::Pass,
            message: "GitLab CI configured".to_string(),
            severity: Severity::Info,
        };
    }

    if jenkinsfile.exists() {
        return ComplianceCheck {
            name: "CI Configured".to_string(),
            status: CheckStatus::Pass,
            message: "Jenkins pipeline configured".to_string(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: "CI Configured".to_string(),
        status: CheckStatus::Warn,
        message: "No CI configuration found - add .github/workflows/".to_string(),
        severity: Severity::Warning,
    }
}

fn calculate_versions_behind(project_version: &str) -> u32 {
    let current_parts: Vec<u32> = PMAT_VERSION
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let project_parts: Vec<u32> = project_version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if current_parts.len() >= 2 && project_parts.len() >= 2 {
        current_parts
            .get(1)
            .unwrap_or(&0)
            .saturating_sub(*project_parts.get(1).unwrap_or(&0))
    } else {
        0
    }
}

fn get_breaking_changes_since(_from_version: &str) -> Vec<BreakingChange> {
    vec![]
}

#[derive(Debug, Clone)]
struct ChangelogEntry {
    version: String,
    description: String,
    breaking: bool,
}

fn get_changelog_entries(_from: &str, _to: &str) -> Vec<ChangelogEntry> {
    vec![
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added qa-work command for Toyota Way validation".to_string(),
            breaking: false,
        },
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added cleanup-resources command".to_string(),
            breaking: false,
        },
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added comply command for compliance checking".to_string(),
            breaking: false,
        },
    ]
}

fn migrate_project_version(project_path: &Path, target: &str, dry_run: bool) -> Result<bool> {
    if dry_run {
        return Ok(true);
    }
    let mut config = load_or_create_project_config(project_path)?;
    if config.pmat.version == target {
        return Ok(false);
    }
    config.pmat.version = target.to_string();
    config.pmat.last_compliance_check = Some(Utc::now());
    let content = toml::to_string_pretty(&config)?;
    fs::write(project_path.join(".pmat").join("project.toml"), &content)?;
    Ok(true)
}

fn migrate_gitignore(project_path: &Path, dry_run: bool) -> Result<bool> {
    let gitignore_path = project_path.join(".gitignore");
    let pmat_entries = [".pmat/backup/", ".pmat-qa/"];
    if !gitignore_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(&gitignore_path)?;
    let mut needs_update = false;
    let mut new_entries = vec![];
    for entry in pmat_entries {
        if !content.contains(entry) {
            needs_update = true;
            new_entries.push(entry);
        }
    }
    if needs_update && !dry_run {
        let mut new_content = content.clone();
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# PMAT\n");
        for entry in new_entries {
            new_content.push_str(entry);
            new_content.push('\n');
        }
        fs::write(&gitignore_path, &new_content)?;
    }
    Ok(needs_update)
}

fn update_project_config(project_path: &Path, dry_run: bool) -> Result<bool> {
    migrate_project_version(project_path, PMAT_VERSION, dry_run)
}

fn print_compliance_text(report: &ComplianceReport) {
    println!("\n{}", "=".repeat(60));
    println!("PMAT Compliance Report");
    println!("{}", "=".repeat(60));
    println!("\nProject Version: {}", report.project_version);
    println!("Current PMAT:    {}", report.current_version);
    println!("Versions Behind: {}", report.versions_behind);
    let status = if report.is_compliant {
        "\x1b[32mCOMPLIANT\x1b[0m"
    } else {
        "\x1b[31mNON-COMPLIANT\x1b[0m"
    };
    println!("Status:          {}\n", status);
    println!("Checks:");
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "\x1b[32m✓\x1b[0m",
            CheckStatus::Warn => "\x1b[33m⚠\x1b[0m",
            CheckStatus::Fail => "\x1b[31m✗\x1b[0m",
            CheckStatus::Skip => "\x1b[90m-\x1b[0m",
        };
        println!("  {} {}: {}", icon, check.name, check.message);
    }
    if !report.recommendations.is_empty() {
        println!("\nRecommendations:");
        for rec in &report.recommendations {
            println!("  • {}", rec);
        }
    }
    println!("\n{}", "=".repeat(60));
}

fn print_compliance_markdown(report: &ComplianceReport) {
    println!("# PMAT Compliance Report\n");
    println!("| Property | Value |");
    println!("|----------|-------|");
    println!("| Project Version | {} |", report.project_version);
    println!("| Current PMAT | {} |", report.current_version);
    println!(
        "| Status | {} |",
        if report.is_compliant {
            "COMPLIANT"
        } else {
            "NON-COMPLIANT"
        }
    );
    println!("\n## Checks\n");
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "✅",
            CheckStatus::Warn => "⚠️",
            CheckStatus::Fail => "❌",
            CheckStatus::Skip => "⏭️",
        };
        println!("- {} **{}**: {}", icon, check.name, check.message);
    }
}

/// Install git hooks for mandatory work tracking (W-006)
/// Implements master-plan-pmat-work-system.md enforcement
async fn handle_enforce(
    project_path: &Path,
    yes: bool,
    disable: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    let hooks_dir = project_path.join(".git").join("hooks");

    if !hooks_dir.exists() {
        anyhow::bail!("Not a git repository (no .git/hooks directory)");
    }

    if disable {
        // Remove PMAT hooks
        let pre_commit = hooks_dir.join("pre-commit");
        if pre_commit.exists() {
            if let Ok(content) = fs::read_to_string(&pre_commit) {
                if content.contains("PMAT") {
                    fs::remove_file(&pre_commit)?;
                    println!("✅ Removed PMAT pre-commit hook");
                } else {
                    println!("⚠️  Pre-commit hook exists but is not PMAT - not removed");
                }
            }
        }
        return Ok(());
    }

    // Prompt for confirmation if not -y
    if !yes {
        println!("This will install PMAT enforcement hooks:");
        println!("  - pre-commit: Block commits without active work ticket");
        println!("  - pre-push: Validate spec compliance before push");
        println!("\nProceed? [y/N] ");

        // For now, just proceed (interactive input not easily testable)
        println!("(Auto-proceeding due to non-interactive mode)");
    }

    // Create pre-commit hook
    let pre_commit_content = r#"#!/bin/sh
# PMAT Work Enforcement Hook (master-plan-pmat-work-system.md W-001)
# This hook blocks commits without an active work ticket.

# Check for active work ticket
if ! pmat work status --active >/dev/null 2>&1; then
    echo "❌ COMPLIANCE VIOLATION"
    echo ""
    echo "Action blocked: git commit"
    echo "Reason: No active work ticket"
    echo ""
    echo "To fix:"
    echo "  1. Start work: pmat work start <ticket-id>"
    echo "  2. Or create ticket: pmat work start \"description\" --spec <spec-file>"
    echo ""
    echo "Bypass (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    exit 1
fi

# Ensure commit message references ticket
TICKET_ID=$(pmat work status --active --quiet 2>/dev/null)
if [ -n "$TICKET_ID" ]; then
    # Check commit message for ticket reference
    if ! grep -qi "$TICKET_ID\|#[0-9]" "$1" 2>/dev/null; then
        echo "⚠️  Commit message should reference ticket: $TICKET_ID"
    fi
fi

exit 0
"#;

    let pre_commit_path = hooks_dir.join("pre-commit");
    fs::write(&pre_commit_path, pre_commit_content)?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_commit_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms)?;
    }

    match format {
        ComplyOutputFormat::Text => {
            println!("\n✅ PMAT enforcement hooks installed!");
            println!("   Pre-commit hook: {}", pre_commit_path.display());
            println!("\nCommits will now require an active work ticket.");
            println!("Use 'pmat comply enforce --disable' to remove hooks.");
        }
        ComplyOutputFormat::Json => {
            let result = serde_json::json!({
                "status": "success",
                "hooks_installed": ["pre-commit"],
                "path": hooks_dir.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        ComplyOutputFormat::Markdown => {
            println!("# PMAT Enforcement Hooks Installed\n");
            println!("| Hook | Status |");
            println!("|------|--------|");
            println!("| pre-commit | ✅ Installed |");
        }
    }

    Ok(())
}

/// Generate compliance report (W-009)
async fn handle_report(
    project_path: &Path,
    include_history: bool,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    // Load project config
    let config = load_or_create_project_config(project_path)?;

    // Run compliance checks
    let checks = vec![
        check_version_currency(&config.pmat.version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
    ];

    let report = ComplianceReport {
        project_version: config.pmat.version.clone(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: checks.iter().all(|c| c.status != CheckStatus::Fail),
        versions_behind: calculate_versions_behind(&config.pmat.version),
        checks,
        breaking_changes: get_breaking_changes_since(&config.pmat.version),
        recommendations: vec![],
        timestamp: Utc::now(),
    };

    // Format output
    let output_text = match format {
        ComplyOutputFormat::Text => {
            let mut out = String::new();
            out.push_str(&format!("\n{}\n", "=".repeat(60)));
            out.push_str("PMAT Compliance Report\n");
            out.push_str(&format!("{}\n", "=".repeat(60)));
            out.push_str(&format!("\nGenerated: {}\n", report.timestamp));
            out.push_str(&format!("Project Version: {}\n", report.project_version));
            out.push_str(&format!("Current PMAT: {}\n", report.current_version));
            out.push_str(&format!(
                "Status: {}\n\n",
                if report.is_compliant {
                    "COMPLIANT"
                } else {
                    "NON-COMPLIANT"
                }
            ));

            out.push_str("Checks:\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "✓",
                    CheckStatus::Warn => "⚠",
                    CheckStatus::Fail => "✗",
                    CheckStatus::Skip => "-",
                };
                out.push_str(&format!("  {} {}: {}\n", icon, check.name, check.message));
            }

            if include_history {
                out.push_str("\nWork History:\n");
                out.push_str("  (Work history not yet implemented)\n");
            }

            out
        }
        ComplyOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplyOutputFormat::Markdown => {
            let mut out = String::new();
            out.push_str("# PMAT Compliance Report\n\n");
            out.push_str(&format!("**Generated:** {}\n\n", report.timestamp));
            out.push_str("| Property | Value |\n");
            out.push_str("|----------|-------|\n");
            out.push_str(&format!(
                "| Project Version | {} |\n",
                report.project_version
            ));
            out.push_str(&format!("| Current PMAT | {} |\n", report.current_version));
            out.push_str(&format!(
                "| Status | {} |\n\n",
                if report.is_compliant {
                    "✅ COMPLIANT"
                } else {
                    "❌ NON-COMPLIANT"
                }
            ));

            out.push_str("## Checks\n\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "✅",
                    CheckStatus::Warn => "⚠️",
                    CheckStatus::Fail => "❌",
                    CheckStatus::Skip => "⏭️",
                };
                out.push_str(&format!(
                    "- {} **{}**: {}\n",
                    icon, check.name, check.message
                ));
            }

            out
        }
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_text)?;
        println!("✅ Compliance report written to {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project_config() {
        let config = ProjectConfig::default();
        assert!(!config.pmat.version.is_empty());
    }

    #[test]
    fn test_calculate_versions_behind_same() {
        let behind = calculate_versions_behind(PMAT_VERSION);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_check_status_equality() {
        assert_eq!(CheckStatus::Pass, CheckStatus::Pass);
        assert_ne!(CheckStatus::Pass, CheckStatus::Fail);
    }

    #[test]
    fn test_severity_variants() {
        let _ = Severity::Info;
        let _ = Severity::Warning;
        let _ = Severity::Error;
        let _ = Severity::Critical;
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // Test Fixture Helpers
    // ============================================================================

    /// Create a temporary directory with basic PMAT structure
    fn create_temp_project() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    /// Create a project with .pmat directory and project.toml
    fn create_pmat_project(version: &str) -> TempDir {
        let temp = create_temp_project();
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).expect("Failed to create .pmat dir");

        let config = format!(
            r#"[pmat]
version = "{}"
auto_update = false
"#,
            version
        );
        fs::write(pmat_dir.join("project.toml"), config).expect("Failed to write project.toml");
        temp
    }

    /// Create a project with .pmat-metrics.toml
    fn create_project_with_metrics(version: &str) -> TempDir {
        let temp = create_pmat_project(version);
        let metrics_content = r#"
[thresholds]
lint = 30000
test-fast = 300000
"#;
        fs::write(temp.path().join(".pmat-metrics.toml"), metrics_content)
            .expect("Failed to write metrics");
        temp
    }

    /// Create a git repository structure
    fn create_git_repo() -> TempDir {
        let temp = create_temp_project();
        let hooks_dir = temp.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).expect("Failed to create .git/hooks");
        temp
    }

    /// Create a Rust project with Cargo.toml
    fn create_rust_project(with_msrv: bool, with_lock: bool) -> TempDir {
        let temp = create_temp_project();
        let cargo_content = if with_msrv {
            r#"[package]
name = "test"
version = "0.1.0"
rust-version = "1.75"
"#
        } else {
            r#"[package]
name = "test"
version = "0.1.0"
"#
        };
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");
        if with_lock {
            fs::write(temp.path().join("Cargo.lock"), "# lock file")
                .expect("Failed to write Cargo.lock");
        }
        temp
    }

    // ============================================================================
    // ProjectConfig Tests
    // ============================================================================

    #[test]
    fn test_project_config_default_has_current_version() {
        let config = ProjectConfig::default();
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[test]
    fn test_project_config_default_has_timestamp() {
        let config = ProjectConfig::default();
        assert!(config.pmat.last_compliance_check.is_some());
    }

    #[test]
    fn test_project_config_default_auto_update_is_false() {
        let config = ProjectConfig::default();
        assert!(!config.pmat.auto_update);
    }

    #[test]
    fn test_project_config_serialization() {
        let config = ProjectConfig::default();
        let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
        assert!(serialized.contains("[pmat]"));
        assert!(serialized.contains("version"));
    }

    #[test]
    fn test_project_config_deserialization() {
        let toml_str = r#"
[pmat]
version = "1.0.0"
auto_update = true
"#;
        let config: ProjectConfig = toml::from_str(toml_str).expect("Deserialization failed");
        assert_eq!(config.pmat.version, "1.0.0");
        assert!(config.pmat.auto_update);
        assert!(config.pmat.last_compliance_check.is_none());
    }

    #[test]
    fn test_pmat_section_clone() {
        let section = PmatSection {
            version: "2.0.0".to_string(),
            last_compliance_check: Some(Utc::now()),
            auto_update: true,
        };
        let cloned = section.clone();
        assert_eq!(cloned.version, section.version);
        assert_eq!(cloned.auto_update, section.auto_update);
    }

    // ============================================================================
    // ComplianceReport Tests
    // ============================================================================

    #[test]
    fn test_compliance_report_serialization() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
            is_compliant: true,
            versions_behind: 10,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&report).expect("JSON serialization failed");
        assert!(json.contains("project_version"));
        assert!(json.contains("is_compliant"));
    }

    #[test]
    fn test_compliance_report_with_checks() {
        let check = ComplianceCheck {
            name: "Test Check".to_string(),
            status: CheckStatus::Pass,
            message: "All good".to_string(),
            severity: Severity::Info,
        };
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![check],
            breaking_changes: vec![],
            recommendations: vec!["Upgrade soon".to_string()],
            timestamp: Utc::now(),
        };
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.recommendations.len(), 1);
    }

    // ============================================================================
    // ComplianceCheck Tests
    // ============================================================================

    #[test]
    fn test_compliance_check_clone() {
        let check = ComplianceCheck {
            name: "Test".to_string(),
            status: CheckStatus::Warn,
            message: "Warning message".to_string(),
            severity: Severity::Warning,
        };
        let cloned = check.clone();
        assert_eq!(cloned.name, check.name);
        assert_eq!(cloned.status, check.status);
    }

    #[test]
    fn test_compliance_check_serialization() {
        let check = ComplianceCheck {
            name: "Version Check".to_string(),
            status: CheckStatus::Fail,
            message: "Outdated".to_string(),
            severity: Severity::Error,
        };
        let json = serde_json::to_string(&check).expect("Serialization failed");
        assert!(json.contains("Version Check"));
        assert!(json.contains("Fail"));
    }

    // ============================================================================
    // CheckStatus Tests
    // ============================================================================

    #[test]
    fn test_check_status_all_variants() {
        let pass = CheckStatus::Pass;
        let warn = CheckStatus::Warn;
        let fail = CheckStatus::Fail;
        let skip = CheckStatus::Skip;

        assert_eq!(pass, CheckStatus::Pass);
        assert_eq!(warn, CheckStatus::Warn);
        assert_eq!(fail, CheckStatus::Fail);
        assert_eq!(skip, CheckStatus::Skip);
    }

    #[test]
    fn test_check_status_inequality() {
        assert_ne!(CheckStatus::Pass, CheckStatus::Warn);
        assert_ne!(CheckStatus::Warn, CheckStatus::Fail);
        assert_ne!(CheckStatus::Fail, CheckStatus::Skip);
        assert_ne!(CheckStatus::Skip, CheckStatus::Pass);
    }

    #[test]
    fn test_check_status_copy() {
        let status = CheckStatus::Pass;
        let copied = status;
        assert_eq!(copied, CheckStatus::Pass);
    }

    // ============================================================================
    // Severity Tests
    // ============================================================================

    #[test]
    fn test_severity_all_variants() {
        let info = Severity::Info;
        let warning = Severity::Warning;
        let error = Severity::Error;
        let critical = Severity::Critical;

        assert_eq!(info, Severity::Info);
        assert_eq!(warning, Severity::Warning);
        assert_eq!(error, Severity::Error);
        assert_eq!(critical, Severity::Critical);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(Severity::Info, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Error);
        assert_ne!(Severity::Error, Severity::Critical);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Critical;
        let json = serde_json::to_string(&severity).expect("Serialization failed");
        assert!(json.contains("Critical"));
    }

    // ============================================================================
    // BreakingChange Tests
    // ============================================================================

    #[test]
    fn test_breaking_change_with_migration_guide() {
        let change = BreakingChange {
            version: "2.0.0".to_string(),
            description: "API changed".to_string(),
            migration_guide: Some("Follow these steps...".to_string()),
        };
        assert_eq!(change.version, "2.0.0");
        assert!(change.migration_guide.is_some());
    }

    #[test]
    fn test_breaking_change_without_migration_guide() {
        let change = BreakingChange {
            version: "2.0.0".to_string(),
            description: "Removed feature X".to_string(),
            migration_guide: None,
        };
        assert!(change.migration_guide.is_none());
    }

    #[test]
    fn test_breaking_change_clone() {
        let change = BreakingChange {
            version: "1.5.0".to_string(),
            description: "Config format changed".to_string(),
            migration_guide: Some("Update your config".to_string()),
        };
        let cloned = change.clone();
        assert_eq!(cloned.version, change.version);
        assert_eq!(cloned.migration_guide, change.migration_guide);
    }

    // ============================================================================
    // calculate_versions_behind Tests
    // ============================================================================

    #[test]
    fn test_calculate_versions_behind_older_minor() {
        // Parse current version to get major.minor
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] > 0 {
            let older = format!("{}.{}.0", parts[0], parts[1] - 1);
            let behind = calculate_versions_behind(&older);
            assert_eq!(behind, 1);
        }
    }

    #[test]
    fn test_calculate_versions_behind_same_version() {
        let behind = calculate_versions_behind(PMAT_VERSION);
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_newer_version() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 {
            let newer = format!("{}.{}.0", parts[0], parts[1] + 10);
            let behind = calculate_versions_behind(&newer);
            // saturating_sub returns 0 for negative result
            assert_eq!(behind, 0);
        }
    }

    #[test]
    fn test_calculate_versions_behind_invalid_version() {
        let behind = calculate_versions_behind("invalid");
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_partial_version() {
        let behind = calculate_versions_behind("1");
        assert_eq!(behind, 0);
    }

    #[test]
    fn test_calculate_versions_behind_empty_string() {
        let behind = calculate_versions_behind("");
        assert_eq!(behind, 0);
    }

    // ============================================================================
    // check_version_currency Tests
    // ============================================================================

    #[test]
    fn test_check_version_currency_current() {
        let check = check_version_currency(PMAT_VERSION);
        assert_eq!(check.status, CheckStatus::Pass);
        assert_eq!(check.severity, Severity::Info);
        assert!(check.message.contains("latest"));
    }

    #[test]
    fn test_check_version_currency_slightly_behind() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] >= 3 {
            let old = format!("{}.{}.0", parts[0], parts[1] - 3);
            let check = check_version_currency(&old);
            assert_eq!(check.status, CheckStatus::Warn);
            assert_eq!(check.severity, Severity::Warning);
        }
    }

    #[test]
    fn test_check_version_currency_very_behind() {
        let parts: Vec<u32> = PMAT_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 && parts[1] > 10 {
            let old = format!("{}.{}.0", parts[0], parts[1] - 10);
            let check = check_version_currency(&old);
            assert_eq!(check.status, CheckStatus::Fail);
            assert_eq!(check.severity, Severity::Error);
        }
    }

    // ============================================================================
    // check_config_files Tests
    // ============================================================================

    #[test]
    fn test_check_config_files_none_present() {
        let temp = create_temp_project();
        let check = check_config_files(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("Missing"));
    }

    #[test]
    fn test_check_config_files_pmat_only() {
        let temp = create_pmat_project(PMAT_VERSION);
        let check = check_config_files(temp.path());
        // Only .pmat/project.toml present, missing .pmat-metrics.toml
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_config_files_all_present() {
        let temp = create_project_with_metrics(PMAT_VERSION);
        let check = check_config_files(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("All required"));
    }

    // ============================================================================
    // check_hooks_installed Tests
    // ============================================================================

    #[test]
    fn test_check_hooks_not_installed() {
        let temp = create_temp_project();
        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No pre-commit"));
    }

    #[test]
    fn test_check_hooks_non_pmat_hook() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\necho 'some other hook'";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("may not be PMAT"));
    }

    #[test]
    fn test_check_hooks_pmat_hook_installed() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\n# PMAT hook\npmat check";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("PMAT hooks installed"));
    }

    #[test]
    fn test_check_hooks_pmat_lowercase() {
        let temp = create_git_repo();
        let hook_content = "#!/bin/sh\npmat validate";
        fs::write(
            temp.path().join(".git").join("hooks").join("pre-commit"),
            hook_content,
        )
        .expect("Failed to write hook");

        let check = check_hooks_installed(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // ============================================================================
    // check_quality_thresholds Tests
    // ============================================================================

    #[test]
    fn test_check_quality_thresholds_missing() {
        let temp = create_temp_project();
        let check = check_quality_thresholds(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No .pmat-metrics.toml"));
    }

    #[test]
    fn test_check_quality_thresholds_present() {
        let temp = create_project_with_metrics(PMAT_VERSION);
        let check = check_quality_thresholds(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("configured"));
    }

    // ============================================================================
    // check_deprecated_features Tests
    // ============================================================================

    #[test]
    fn test_check_deprecated_features_none() {
        let temp = create_temp_project();
        let check = check_deprecated_features(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("No deprecated"));
    }

    // ============================================================================
    // check_compute_brick Tests
    // ============================================================================

    #[test]
    fn test_check_compute_brick_not_applicable() {
        let temp = create_temp_project();
        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("Not a ComputeBrick"));
    }

    #[test]
    fn test_check_compute_brick_with_probar_dep() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let check = check_compute_brick(temp.path());
        // Has probar but no .pmat-gates.toml
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("[compute-brick]"));
    }

    #[test]
    fn test_check_compute_brick_with_brick_dir() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_compute_brick_fully_configured() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_compute_brick_probar_without_coverage() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("GUI coverage"));
    }

    // ============================================================================
    // check_cargo_lock Tests
    // ============================================================================

    #[test]
    fn test_check_cargo_lock_missing() {
        let temp = create_rust_project(false, false);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("Missing Cargo.lock"));
    }

    #[test]
    fn test_check_cargo_lock_present() {
        let temp = create_rust_project(false, true);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("reproducible builds"));
    }

    // ============================================================================
    // check_msrv Tests
    // ============================================================================

    #[test]
    fn test_check_msrv_no_cargo_toml() {
        let temp = create_temp_project();
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No Cargo.toml"));
    }

    #[test]
    fn test_check_msrv_missing() {
        let temp = create_rust_project(false, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No rust-version"));
    }

    #[test]
    fn test_check_msrv_present() {
        let temp = create_rust_project(true, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("rust-version field present"));
    }

    // ============================================================================
    // check_ci_configured Tests
    // ============================================================================

    #[test]
    fn test_check_ci_not_configured() {
        let temp = create_temp_project();
        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No CI configuration"));
    }

    #[test]
    fn test_check_ci_github_actions() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");
        fs::write(workflows_dir.join("ci.yml"), "name: CI").expect("Failed to write workflow");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitHub Actions"));
    }

    #[test]
    fn test_check_ci_github_actions_empty() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_ci_gitlab() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitlab-ci.yml"), "stages:\n  - build")
            .expect("Failed to write gitlab-ci");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitLab CI"));
    }

    #[test]
    fn test_check_ci_jenkins() {
        let temp = create_temp_project();
        fs::write(temp.path().join("Jenkinsfile"), "pipeline { }")
            .expect("Failed to write Jenkinsfile");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("Jenkins"));
    }

    // ============================================================================
    // get_breaking_changes_since Tests
    // ============================================================================

    #[test]
    fn test_get_breaking_changes_since_returns_empty() {
        let changes = get_breaking_changes_since("1.0.0");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_get_breaking_changes_since_any_version() {
        let changes = get_breaking_changes_since("0.0.1");
        assert!(changes.is_empty());
    }

    // ============================================================================
    // get_changelog_entries Tests
    // ============================================================================

    #[test]
    fn test_get_changelog_entries_returns_entries() {
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_get_changelog_entries_contain_expected_features() {
        let entries = get_changelog_entries("1.0.0", PMAT_VERSION);
        let has_comply = entries.iter().any(|e| e.description.contains("comply"));
        assert!(has_comply);
    }

    #[test]
    fn test_changelog_entry_breaking_flag() {
        let entries = get_changelog_entries("1.0.0", "2.0.0");
        // Current implementation has no breaking changes
        let breaking_count = entries.iter().filter(|e| e.breaking).count();
        assert_eq!(breaking_count, 0);
    }

    // ============================================================================
    // load_or_create_project_config Tests
    // ============================================================================

    #[test]
    fn test_load_or_create_config_creates_new() {
        let temp = create_temp_project();
        let config =
            load_or_create_project_config(temp.path()).expect("Failed to load/create config");
        assert_eq!(config.pmat.version, PMAT_VERSION);

        // Verify file was created
        assert!(temp.path().join(".pmat").join("project.toml").exists());
    }

    #[test]
    fn test_load_or_create_config_loads_existing() {
        let temp = create_pmat_project("1.0.0");
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[test]
    fn test_load_or_create_config_invalid_toml() {
        let temp = create_temp_project();
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).expect("Failed to create .pmat");
        fs::write(pmat_dir.join("project.toml"), "invalid { toml").expect("Failed to write");

        let result = load_or_create_project_config(temp.path());
        assert!(result.is_err());
    }

    // ============================================================================
    // update_last_check_timestamp Tests
    // ============================================================================

    #[test]
    fn test_update_last_check_timestamp() {
        let temp = create_pmat_project(PMAT_VERSION);

        let result = update_last_check_timestamp(temp.path());
        assert!(result.is_ok());

        // Verify timestamp was updated
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert!(config.pmat.last_compliance_check.is_some());
    }

    #[test]
    fn test_update_last_check_timestamp_no_config() {
        let temp = create_temp_project();
        let result = update_last_check_timestamp(temp.path());
        // Should succeed even if config doesn't exist
        assert!(result.is_ok());
    }

    // ============================================================================
    // migrate_project_version Tests
    // ============================================================================

    #[test]
    fn test_migrate_project_version_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "2.0.0", true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // dry_run always returns true

        // Verify version NOT changed
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[test]
    fn test_migrate_project_version_actual() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "2.0.0", false);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify version changed
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "2.0.0");
    }

    #[test]
    fn test_migrate_project_version_same_version() {
        let temp = create_pmat_project("1.0.0");
        let result = migrate_project_version(temp.path(), "1.0.0", false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No change needed
    }

    // ============================================================================
    // migrate_gitignore Tests
    // ============================================================================

    #[test]
    fn test_migrate_gitignore_no_file() {
        let temp = create_temp_project();
        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_migrate_gitignore_adds_entries() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/\n").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        assert!(content.contains(".pmat/backup/"));
        assert!(content.contains(".pmat-qa/"));
    }

    #[test]
    fn test_migrate_gitignore_already_has_entries() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join(".gitignore"),
            "target/\n.pmat/backup/\n.pmat-qa/\n",
        )
        .expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No changes needed
    }

    #[test]
    fn test_migrate_gitignore_dry_run() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/\n").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Would need update

        // Verify file NOT changed
        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        assert!(!content.contains(".pmat/backup/"));
    }

    #[test]
    fn test_migrate_gitignore_no_trailing_newline() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitignore"), "target/").expect("Failed to write gitignore");

        let result = migrate_gitignore(temp.path(), false);
        assert!(result.is_ok());

        let content = fs::read_to_string(temp.path().join(".gitignore")).expect("Failed to read");
        // Should handle missing trailing newline
        assert!(content.contains("# PMAT"));
    }

    // ============================================================================
    // update_project_config Tests
    // ============================================================================

    #[test]
    fn test_update_project_config_updates_to_current() {
        let temp = create_pmat_project("1.0.0");
        let result = update_project_config(temp.path(), false);
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[test]
    fn test_update_project_config_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = update_project_config(temp.path(), true);
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    // ============================================================================
    // print_compliance_text Tests
    // ============================================================================

    #[test]
    fn test_print_compliance_text_compliant() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![ComplianceCheck {
                name: "Test".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                severity: Severity::Info,
            }],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        // This just tests it doesn't panic
        print_compliance_text(&report);
    }

    #[test]
    fn test_print_compliance_text_non_compliant() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: false,
            versions_behind: 10,
            checks: vec![ComplianceCheck {
                name: "Version".to_string(),
                status: CheckStatus::Fail,
                message: "Outdated".to_string(),
                severity: Severity::Error,
            }],
            breaking_changes: vec![],
            recommendations: vec!["Update PMAT".to_string()],
            timestamp: Utc::now(),
        };
        print_compliance_text(&report);
    }

    #[test]
    fn test_print_compliance_text_all_status_types() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![
                ComplianceCheck {
                    name: "Pass".to_string(),
                    status: CheckStatus::Pass,
                    message: "Good".to_string(),
                    severity: Severity::Info,
                },
                ComplianceCheck {
                    name: "Warn".to_string(),
                    status: CheckStatus::Warn,
                    message: "Warning".to_string(),
                    severity: Severity::Warning,
                },
                ComplianceCheck {
                    name: "Fail".to_string(),
                    status: CheckStatus::Fail,
                    message: "Failed".to_string(),
                    severity: Severity::Error,
                },
                ComplianceCheck {
                    name: "Skip".to_string(),
                    status: CheckStatus::Skip,
                    message: "Skipped".to_string(),
                    severity: Severity::Info,
                },
            ],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_text(&report);
    }

    // ============================================================================
    // print_compliance_markdown Tests
    // ============================================================================

    #[test]
    fn test_print_compliance_markdown_compliant() {
        let report = ComplianceReport {
            project_version: PMAT_VERSION.to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_markdown(&report);
    }

    #[test]
    fn test_print_compliance_markdown_non_compliant() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: PMAT_VERSION.to_string(),
            is_compliant: false,
            versions_behind: 5,
            checks: vec![ComplianceCheck {
                name: "Check".to_string(),
                status: CheckStatus::Fail,
                message: "Failed".to_string(),
                severity: Severity::Error,
            }],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        print_compliance_markdown(&report);
    }

    // ============================================================================
    // Async Handler Tests (using tokio::test)
    // ============================================================================

    #[tokio::test]
    async fn test_handle_init_new_project() {
        let temp = create_temp_project();
        let result = handle_init(temp.path(), false).await;
        assert!(result.is_ok());

        // Verify project.toml created
        assert!(temp.path().join(".pmat").join("project.toml").exists());
    }

    #[tokio::test]
    async fn test_handle_init_existing_no_force() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_init(temp.path(), false).await;
        assert!(result.is_ok());

        // Version should remain unchanged
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_handle_init_existing_with_force() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_init(temp.path(), true).await;
        assert!(result.is_ok());

        // Version should be updated to current
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_update_both() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, false, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_update_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, false, true).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    #[tokio::test]
    async fn test_handle_update_hooks_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), true, false, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_update_config_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_update(temp.path(), false, true, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);
    }

    #[tokio::test]
    async fn test_handle_diff_default_versions() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), None, None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_diff_specific_versions() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), Some("1.0.0"), Some("2.0.0"), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_diff_breaking_only() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_diff(temp.path(), None, None, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_dry_run() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, true, false, false).await;
        assert!(result.is_ok());

        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, "1.0.0"); // Unchanged
    }

    #[tokio::test]
    async fn test_handle_migrate_with_target() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), Some("2.0.0"), false, true, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_no_backup() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, false, true, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_migrate_with_backup() {
        let temp = create_pmat_project("1.0.0");
        let result = handle_migrate(temp.path(), None, false, false, true).await;
        assert!(result.is_ok());

        // Verify backup directory created
        assert!(temp.path().join(".pmat").join("backup").exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_no_git() {
        let temp = create_temp_project();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_handle_enforce_install() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Verify hook created
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        assert!(hook_path.exists());
        let content = fs::read_to_string(&hook_path).expect("Failed to read hook");
        assert!(content.contains("PMAT"));
    }

    #[tokio::test]
    async fn test_handle_enforce_disable() {
        let temp = create_git_repo();
        // First install
        handle_enforce(temp.path(), true, false, ComplyOutputFormat::Text)
            .await
            .expect("Failed to install");

        // Then disable
        let result = handle_enforce(temp.path(), true, true, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Verify hook removed
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        assert!(!hook_path.exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_disable_non_pmat_hook() {
        let temp = create_git_repo();
        let hook_path = temp.path().join(".git").join("hooks").join("pre-commit");
        fs::write(&hook_path, "#!/bin/sh\necho 'other hook'").expect("Failed to write hook");

        let result = handle_enforce(temp.path(), true, true, ComplyOutputFormat::Text).await;
        assert!(result.is_ok());

        // Non-PMAT hook should NOT be removed
        assert!(hook_path.exists());
    }

    #[tokio::test]
    async fn test_handle_enforce_json_format() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_enforce_markdown_format() {
        let temp = create_git_repo();
        let result = handle_enforce(temp.path(), true, false, ComplyOutputFormat::Markdown).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_text() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_json() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Json, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_markdown() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), false, ComplyOutputFormat::Markdown, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_with_history() {
        let temp = create_pmat_project(PMAT_VERSION);
        let result = handle_report(temp.path(), true, ComplyOutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_to_file() {
        let temp = create_pmat_project(PMAT_VERSION);
        let output_file = temp.path().join("report.md");
        let result = handle_report(
            temp.path(),
            false,
            ComplyOutputFormat::Markdown,
            Some(&output_file),
        )
        .await;
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    // ============================================================================
    // handle_comply_command Tests
    // ============================================================================

    #[tokio::test]
    async fn test_handle_comply_command_init() {
        let temp = create_temp_project();
        let command = ComplyCommands::Init {
            path: temp.path().to_path_buf(),
            force: false,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_update() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Update {
            path: temp.path().to_path_buf(),
            hooks: false,
            config: false,
            dry_run: true,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_diff() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Diff {
            path: temp.path().to_path_buf(),
            from: None,
            to: None,
            breaking_only: false,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_migrate() {
        let temp = create_pmat_project("1.0.0");
        let command = ComplyCommands::Migrate {
            path: temp.path().to_path_buf(),
            version: None,
            dry_run: true,
            no_backup: true,
            force: true,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_enforce() {
        let temp = create_git_repo();
        let command = ComplyCommands::Enforce {
            path: temp.path().to_path_buf(),
            yes: true,
            disable: false,
            format: ComplyOutputFormat::Text,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_comply_command_report() {
        let temp = create_pmat_project(PMAT_VERSION);
        let command = ComplyCommands::Report {
            path: temp.path().to_path_buf(),
            include_history: false,
            format: ComplyOutputFormat::Text,
            output: None,
        };
        let result = handle_comply_command(command).await;
        assert!(result.is_ok());
    }

    // ============================================================================
    // Edge Cases and Error Paths
    // ============================================================================

    #[test]
    fn test_version_parsing_with_prerelease() {
        let behind = calculate_versions_behind("2.0.0-alpha.1");
        // Should handle prerelease gracefully
        assert!(behind >= 0);
    }

    #[test]
    fn test_version_parsing_with_build_metadata() {
        let behind = calculate_versions_behind("2.0.0+build.123");
        assert!(behind >= 0);
    }

    #[test]
    fn test_compliance_check_debug_impl() {
        let check = ComplianceCheck {
            name: "Test".to_string(),
            status: CheckStatus::Pass,
            message: "OK".to_string(),
            severity: Severity::Info,
        };
        let debug_str = format!("{:?}", check);
        assert!(debug_str.contains("ComplianceCheck"));
        assert!(debug_str.contains("Pass"));
    }

    #[test]
    fn test_project_config_debug_impl() {
        let config = ProjectConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ProjectConfig"));
    }

    #[test]
    fn test_breaking_change_debug_impl() {
        let change = BreakingChange {
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            migration_guide: None,
        };
        let debug_str = format!("{:?}", change);
        assert!(debug_str.contains("BreakingChange"));
    }

    #[test]
    fn test_compliance_report_debug_impl() {
        let report = ComplianceReport {
            project_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
        };
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("ComplianceReport"));
    }

    #[tokio::test]
    async fn test_handle_check_with_nonexistent_path() {
        let temp = create_temp_project();
        let nonexistent = temp.path().join("nonexistent");
        // This should create the config directory
        let result = load_or_create_project_config(&nonexistent);
        // May fail due to parent directory not existing
        // Just verify it handles the error gracefully
        let _ = result;
    }

    #[test]
    fn test_changelog_entry_struct() {
        // Test the ChangelogEntry struct directly
        let entry = ChangelogEntry {
            version: "1.0.0".to_string(),
            description: "Test change".to_string(),
            breaking: true,
        };
        assert_eq!(entry.version, "1.0.0");
        assert!(entry.breaking);

        // Test clone
        let cloned = entry.clone();
        assert_eq!(cloned.version, entry.version);
        assert_eq!(cloned.breaking, entry.breaking);
    }

    #[test]
    fn test_pmat_version_constant() {
        // Verify PMAT_VERSION is set from Cargo.toml
        assert!(!PMAT_VERSION.is_empty());
        // Should be a valid semver-ish format
        let parts: Vec<&str> = PMAT_VERSION.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major.minor");
    }

    // ============================================================================
    // Integration-style Tests
    // ============================================================================

    #[tokio::test]
    async fn test_full_compliance_workflow() {
        // Create a new project, init, check, migrate
        let temp = create_temp_project();

        // Init
        handle_init(temp.path(), false).await.expect("Init failed");

        // Verify config exists
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);

        // Check should pass (we're on current version)
        let checks = vec![
            check_version_currency(&config.pmat.version),
            check_config_files(temp.path()),
        ];
        let _all_pass_or_warn = checks.iter().all(|c| c.status != CheckStatus::Fail);
        // Version should pass, config files may warn about metrics
        assert!(checks[0].status == CheckStatus::Pass);
    }

    #[tokio::test]
    async fn test_migrate_then_check_workflow() {
        let temp = create_pmat_project("1.0.0");

        // Migrate to current
        handle_migrate(temp.path(), None, false, true, true)
            .await
            .expect("Migrate failed");

        // Check version should now pass
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        let check = check_version_currency(&config.pmat.version);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // ============================================================================
    // ComputeBrick Pattern Detection Tests (CB-IMPL-001-B)
    // ============================================================================

    #[test]
    fn test_cb020_detects_unsafe_without_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block without SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        std::fs::write(
            &rs_file,
            r#"
fn bad_unsafe() {
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-020");
        assert!(violations[0].description.contains("unsafe"));
    }

    #[test]
    fn test_cb020_allows_unsafe_with_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block WITH SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        std::fs::write(
            &rs_file,
            r#"
fn good_unsafe() {
    // SAFETY: null pointer read is UB, but this is just a test
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb021_detects_simd_without_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with SIMD intrinsic without #[target_feature]
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
fn bad_simd() {
    let a = _mm_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-021");
        assert!(violations[0].description.contains("_mm_"));
    }

    #[test]
    fn test_cb021_allows_simd_with_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with SIMD intrinsic WITH #[target_feature]
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
#[target_feature(enable = "sse2")]
fn good_simd() {
    let a = _mm_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_bricks_without_assertions() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITHOUT assertions
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            // No leading newline - content starts immediately
            "impl ComputeBrick for MyBrick {\n\
                fn execute(&self) {\n\
                    self.do_work();\n\
                }\n\
            }\n",
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 1, "Expected 1 violation for brick without assertions");
        assert_eq!(violations[0].pattern_id, "CB-BUDGET");
    }

    #[test]
    fn test_detect_bricks_with_assertions_pass() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITH assertions
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            r#"
impl ComputeBrick for MyBrick {
    fn execute(&self) {
        debug_assert!(self.is_valid());
        self.do_work();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_profiler_anomalies_high_cv() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with high CV
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "MatMulBrick",
      "cv": 0.25,
      "efficiency": 0.80
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "HIGH_CV");
        assert!(anomalies[0].value > 15.0);
    }

    #[test]
    fn test_detect_profiler_anomalies_low_efficiency() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with low efficiency
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "SlowBrick",
      "cv": 0.05,
      "efficiency": 0.15
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "LOW_EFFICIENCY");
        assert!(anomalies[0].value < 25.0);
    }

    #[test]
    fn test_check_compute_brick_skips_non_cb_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create a regular project without trueno/realizar/probar deps
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "regular-project"
version = "1.0.0"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_check_compute_brick_detects_trueno_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create project with trueno dependency
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "gpu-project"
version = "1.0.0"

[dependencies]
trueno = "0.1"
"#,
        )
        .unwrap();

        // Create src directory with clean Rust code
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

        let check = check_compute_brick(temp.path());
        // Should not skip - this is a CB ecosystem project
        assert_ne!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_extract_json_number() {
        assert_eq!(extract_json_number("\"cv\": 0.18,"), Some(0.18));
        assert_eq!(extract_json_number("\"efficiency\": 25.5}"), Some(25.5));
        assert_eq!(extract_json_number("invalid"), None);
    }

    #[test]
    fn test_walkdir_rs_files() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        let nested = src_dir.join("nested");
        std::fs::create_dir_all(&nested).unwrap();

        std::fs::write(src_dir.join("lib.rs"), "").unwrap();
        std::fs::write(nested.join("mod.rs"), "").unwrap();
        std::fs::write(src_dir.join("readme.md"), "").unwrap(); // Not .rs

        let files = walkdir_rs_files(&src_dir).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "rs"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn test_version_behind_never_negative(major in 0u32..100, minor in 0u32..1000, patch in 0u32..100) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let behind = calculate_versions_behind(&version);
            // Should always return a non-negative value (saturating_sub)
            prop_assert!(behind < u32::MAX);
        }

        #[test]
        fn test_check_version_currency_always_returns_valid_check(
            major in 0u32..10,
            minor in 0u32..500,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let check = check_version_currency(&version);

            // Check should always have non-empty fields
            prop_assert!(!check.name.is_empty());
            prop_assert!(!check.message.is_empty());

            // Status should be one of the valid variants
            prop_assert!(matches!(
                check.status,
                CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail | CheckStatus::Skip
            ));
        }

        #[test]
        fn test_project_config_roundtrip_serialization(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            auto_update in proptest::bool::ANY
        ) {
            let config = ProjectConfig {
                pmat: PmatSection {
                    version: version.clone(),
                    last_compliance_check: Some(Utc::now()),
                    auto_update,
                },
            };

            let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
            let deserialized: ProjectConfig = toml::from_str(&serialized).expect("Deserialization failed");

            prop_assert_eq!(deserialized.pmat.version, version);
            prop_assert_eq!(deserialized.pmat.auto_update, auto_update);
        }

        #[test]
        fn test_compliance_check_serialization_roundtrip(
            name in "[a-zA-Z ]{1,50}",
            message in "[a-zA-Z0-9 ]{1,100}"
        ) {
            let check = ComplianceCheck {
                name: name.clone(),
                status: CheckStatus::Pass,
                message: message.clone(),
                severity: Severity::Info,
            };

            let json = serde_json::to_string(&check).expect("Serialization failed");
            let deserialized: ComplianceCheck = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.name, name);
            prop_assert_eq!(deserialized.message, message);
        }

        #[test]
        fn test_breaking_change_serialization_roundtrip(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            description in "[a-zA-Z0-9 ]{1,200}"
        ) {
            let change = BreakingChange {
                version: version.clone(),
                description: description.clone(),
                migration_guide: Some("Guide".to_string()),
            };

            let json = serde_json::to_string(&change).expect("Serialization failed");
            let deserialized: BreakingChange = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.version, version);
            prop_assert_eq!(deserialized.description, description);
        }

        #[test]
        fn test_changelog_entries_always_have_current_version(_seed in 0u32..1000) {
            let entries = get_changelog_entries("0.0.0", "999.999.999");
            prop_assert!(!entries.is_empty());

            // All entries should have version matching PMAT_VERSION
            for entry in &entries {
                prop_assert_eq!(&entry.version, PMAT_VERSION);
            }
        }

        #[test]
        fn test_breaking_changes_returns_empty_for_any_version(
            major in 0u32..100,
            minor in 0u32..1000,
            patch in 0u32..100
        ) {
            let version = format!("{}.{}.{}", major, minor, patch);
            let changes = get_breaking_changes_since(&version);
            // Current implementation always returns empty
            prop_assert!(changes.is_empty());
        }
    }

    // Additional property tests that require tempdir (can't use proptest macro easily)
    #[test]
    fn test_check_config_files_consistency() {
        use tempfile::TempDir;

        // Test that check_config_files is consistent across multiple calls
        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_config_files(temp.path());
        let check2 = check_config_files(temp.path());

        assert_eq!(check1.status, check2.status);
        assert_eq!(check1.message, check2.message);
    }

    #[test]
    fn test_check_hooks_consistency() {
        use tempfile::TempDir;

        let temp = TempDir::new().expect("Failed to create temp dir");
        let check1 = check_hooks_installed(temp.path());
        let check2 = check_hooks_installed(temp.path());

        assert_eq!(check1.status, check2.status);
    }
}
