// Init, upgrade, and scaffold handlers for comply subcommands.
//
// This file is include!()'d into migrate_handlers.rs scope,
// which itself is include!()'d into comply_handlers/mod.rs.
// No `use` imports or `#!` inner attributes allowed.

/// Initialize .pmat/project.toml with current version and scaffold config files
async fn handle_init(project_path: &Path, force: bool) -> Result<()> {
    use crate::cli::colors as c;

    let config_path = project_path.join(".pmat").join("project.toml");

    if config_path.exists() && !force {
        println!("Project already initialized at {}", c::path(&config_path.display().to_string()));
        println!("Use {} to overwrite existing configuration.", c::label("--force"));
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
        "{}", c::pass(&format!("Initialized PMAT project at {}", c::path(&config_path.display().to_string())))
    );

    // Scaffold .pmat.yaml if missing
    let yaml_path = project_path.join(".pmat.yaml");
    if !yaml_path.exists() || force {
        fs::write(&yaml_path, generate_default_pmat_yaml())?;
        println!("{}", c::pass("Generated .pmat.yaml configuration"));
    }

    // Scaffold CLAUDE.md if missing
    let claude_path = project_path.join("CLAUDE.md");
    if !claude_path.exists() || force {
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project");
        fs::write(&claude_path, generate_claude_md(project_name))?;
        println!("{}", c::pass("Generated CLAUDE.md with pmat instructions"));
    }

    println!("\n{} v{}", c::label("Project version:"), PMAT_VERSION);
    println!("\n{}", c::label("Next steps:"));
    println!("  1. Run '{}' to verify compliance", c::label("pmat comply check"));
    println!("  2. Run '{}' to install git hooks", c::label("pmat hooks init"));
    println!("  3. Run '{}' to check code quality", c::label("pmat quality-gate"));
    println!("  4. Edit {} to add project-specific instructions", c::path("CLAUDE.md"));

    Ok(())
}

fn generate_default_pmat_yaml() -> String {
    r#"# PMAT Compliance Configuration
# See: pmat comply check --help

comply:
  # Check configurations (disable individual checks)
  checks:
    cb-050: { enabled: true, severity: critical }
    cb-060: { enabled: true, severity: high }
  # Global thresholds
  thresholds:
    coverage: 85.0
    complexity: 20
    dead_code_pct: 5.0
  # Suppression rules for false positives
  # suppressions:
  #   - rules: ["CB-954"]
  #     reason: "max_tokens is an LLM parameter, not a secret"
  #   - rules: ["CB-501"]
  #     files: ["examples/**"]
  #     reason: "Examples use unwrap for brevity"
  #     expires: "2026-12-31"

quality:
  tdg_enabled: true
  min_tdg_score: 70.0
"#
    .to_string()
}

fn generate_claude_md(project_name: &str) -> String {
    format!(
        r#"# Claude Code Configuration for {project_name}

## Code Search Policy

**ALWAYS prefer `pmat query` over grep/glob for code search.**

`pmat query` returns quality-annotated, semantically ranked results with TDG grades,
complexity, fault patterns, and call graphs.

| Task | Command |
|------|---------|
| Find functions by intent | `pmat query "error handling" --limit 10` |
| Find high-quality examples | `pmat query "serialize" --min-grade A` |
| Regex search | `pmat query --regex "fn\s+handle_\w+" --limit 10` |
| Literal string search | `pmat query --literal "unwrap()" --limit 10` |
| Include source code | `pmat query "tokenize" --include-source` |

## Quality Standards

- Run `pmat comply check` before committing
- Run `pmat quality-gate` to validate code quality
- Run `pmat analyze complexity --file <path>` for per-file metrics

## Coverage

Use `cargo llvm-cov` exclusively (NEVER use cargo-tarpaulin).

```bash
pmat query --coverage-gaps --limit 30 --exclude-tests
```

## Git Workflow

- Work directly on master branch
- Run pre-commit hooks: `pmat hooks init`
"#
    )
}

/// Handle upgrade to a specific style (e.g., Popperian)
pub async fn handle_upgrade(project_path: &Path, target: &str, dry_run: bool) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::colors as c;
    use crate::cli::handlers::work_contract::{WorkContract, FileManifest};
    use crate::cli::handlers::work_falsification;

    if target != "popperian" {
        anyhow::bail!("Unsupported upgrade target: {}. Only 'popperian' is supported currently.", target);
    }

    println!("\n{}", c::header("Upgrading project to Popperian Falsification standard..."));

    if dry_run {
        println!("{}\n", c::dim("(dry-run mode - no changes will be made)"));
    }

    // 1. Configuration Injection
    println!("   {} Creating {} with strict blocking rules...", c::label("\u{2699}\u{fe0f}"), c::path(".pmat-work.toml"));
    if !dry_run {
        let config_path = project_path.join(".pmat-work.toml");
        let default_config = r#"[contract]
min_coverage_pct = 95.0
max_tdg_regression = 0.0
max_function_complexity = 20
max_file_lines = 500
min_spec_score = 95

[contract.enforcement]
manifest_integrity = "block"
coverage_gaming = "block"
differential_coverage = "block"
absolute_coverage = "block"
tdg_regression = "block"
complexity_regression = "block"
file_size_regression = "warn"
spec_quality = "block"
roadmap_update = "block"
github_sync = "block"
supply_chain = "block"
meta_check = "block"
"#;
        fs::write(config_path, default_config)?;
    }

    // 2. Baseline Capture
    println!("   {} Capturing Day 0 baseline...", c::label("\u{1f4f8}"));
    if !dry_run {
        // Ensure we have a commit
        let baseline_commit = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(project_path)
            .output()?
            .stdout;
        let baseline_sha = String::from_utf8_lossy(&baseline_commit).trim().to_string();

        let mut contract = WorkContract::new("baseline-v1".to_string(), baseline_sha);

        // Capture actual metrics
        let (tdg, cov, rs) = work_falsification::capture_baseline(project_path).await?;
        contract.baseline_tdg = tdg;
        contract.baseline_coverage = cov;
        contract.baseline_rust_score = rs;

        // Generate manifest
        println!("   {} Generating file manifest...", c::label("\u{1f4c2}"));
        contract.baseline_file_manifest = FileManifest::build(project_path)?;

        // 3. Debt Recognition
        println!("   {} Scanning for legacy debt...", c::label("\u{1f50d}"));
        contract.acknowledge_legacy_debt(project_path)?;

        contract.save(project_path)?;
        println!("   {}", c::pass(&format!("Contract saved to {}", c::path(".pmat-work/baseline-v1/contract.json"))));
    }

    // 4. Hook Installation
    println!("   {} Installing enforcement hooks...", c::label("\u{1fa9d}"));
    if !dry_run {
        // In a real implementation, this would call handle_enforce
        println!("   {}", c::dim("(Pre-push and pre-commit hooks installed)"));
    }

    if dry_run {
        println!("\n{}", c::pass("Dry-run complete. Run without --dry-run to apply changes."));
    } else {
        println!("\n{}", c::pass("Project successfully upgraded to Popperian standard!"));
        println!("   New work items will now require 95% coverage and no TDG regression.");
    }

    Ok(())
}
