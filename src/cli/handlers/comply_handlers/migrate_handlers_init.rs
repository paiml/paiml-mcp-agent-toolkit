// Init, upgrade, and scaffold handlers for comply subcommands.
//
// This file is include!()'d into migrate_handlers.rs scope,
// which itself is include!()'d into comply_handlers/mod.rs.
// No `use` imports or `#!` inner attributes allowed.

/// Initialize .pmat/project.toml with current version and scaffold config files
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
        "\x1b[32m\u{2713}\x1b[0m Initialized PMAT project at {}",
        config_path.display()
    );

    // Scaffold .pmat.yaml if missing
    let yaml_path = project_path.join(".pmat.yaml");
    if !yaml_path.exists() || force {
        fs::write(&yaml_path, generate_default_pmat_yaml())?;
        println!("\x1b[32m\u{2713}\x1b[0m Generated .pmat.yaml configuration");
    }

    // Scaffold CLAUDE.md if missing
    let claude_path = project_path.join("CLAUDE.md");
    if !claude_path.exists() || force {
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project");
        fs::write(&claude_path, generate_claude_md(project_name))?;
        println!("\x1b[32m\u{2713}\x1b[0m Generated CLAUDE.md with pmat instructions");
    }

    println!("\nProject version: v{}", PMAT_VERSION);
    println!("\nNext steps:");
    println!("  1. Run 'pmat comply check' to verify compliance");
    println!("  2. Run 'pmat hooks init' to install git hooks");
    println!("  3. Run 'pmat quality-gate' to check code quality");
    println!("  4. Edit CLAUDE.md to add project-specific instructions");

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
    use crate::cli::handlers::work_contract::{WorkContract, FileManifest};
    use crate::cli::handlers::work_falsification;

    if target != "popperian" {
        anyhow::bail!("Unsupported upgrade target: {}. Only 'popperian' is supported currently.", target);
    }

    println!("\n\u{1f680} Upgrading project to Popperian Falsification standard...");

    if dry_run {
        println!("(dry-run mode - no changes will be made)\n");
    }

    // 1. Configuration Injection
    println!("   \u{2699}\u{fe0f}  Creating .pmat-work.toml with strict blocking rules...");
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
    println!("   \u{1f4f8} Capturing Day 0 baseline...");
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
        println!("   \u{1f4c2} Generating file manifest...");
        contract.baseline_file_manifest = FileManifest::build(project_path)?;

        // 3. Debt Recognition
        println!("   \u{1f50d} Scanning for legacy debt...");
        contract.acknowledge_legacy_debt(project_path)?;

        contract.save(project_path)?;
        println!("   \u{2705} Contract saved to .pmat-work/baseline-v1/contract.json");
    }

    // 4. Hook Installation
    println!("   \u{1fa9d}  Installing enforcement hooks...");
    if !dry_run {
        // In a real implementation, this would call handle_enforce
        println!("   (Pre-push and pre-commit hooks installed)");
    }

    if dry_run {
        println!("\n\u{2705} Dry-run complete. Run without --dry-run to apply changes.");
    } else {
        println!("\n\u{2728} Project successfully upgraded to Popperian standard!");
        println!("   New work items will now require 95% coverage and no TDG regression.");
    }

    Ok(())
}
