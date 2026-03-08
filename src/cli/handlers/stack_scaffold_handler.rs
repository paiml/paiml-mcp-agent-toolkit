#![cfg_attr(coverage_nightly, coverage(off))]
//! Stack scaffold handler: one-shot SQI-compliant boilerplate generation
//!
//! Generates standard project files (.clippy.toml, deny.toml, CONTRIBUTING.md, etc.)
//! for sovereign AI stack repos to ensure consistent repo hygiene.

use anyhow::{Context, Result};
use std::path::Path;

use crate::cli::colors as c;

// ── Data Types ────────────────────────────────────────────────────────────

pub struct ScaffoldFile {
    pub path: &'static str,
    pub content: String,
    pub description: &'static str,
}

pub struct ScaffoldResult {
    pub created: usize,
    pub skipped: usize,
    pub overwritten: usize,
}

// ── Template Definitions ──────────────────────────────────────────────────

pub fn get_scaffold_files(template: &str) -> Vec<ScaffoldFile> {
    match template {
        "sqi-a-minus" | _ => get_sqi_a_minus_files(),
    }
}

fn get_sqi_a_minus_files() -> Vec<ScaffoldFile> {
    vec![
        ScaffoldFile {
            path: ".clippy.toml",
            content: r#"# PAIML Standard Clippy Configuration
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 8
type-complexity-threshold = 350
doc-valid-idents = ["TDG", "PMAT", "SIMD", "AVX", "GPU", "CUDA", "WASM", "NEON", "PageRank", "CSV", "JSON", "YAML", "SARIF", "LDA", "BM25", "FTS5", "SQLite", "MinHash", "LSH"]
"#
            .to_string(),
            description: "Standard PAIML clippy config",
        },
        ScaffoldFile {
            path: "deny.toml",
            content: r#"[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0", "Zlib", "BSL-1.0", "Unicode-DFS-2016", "OpenSSL"]

[bans]
multiple-versions = "warn"
wildcards = "allow"
"#
            .to_string(),
            description: "cargo-deny config",
        },
        ScaffoldFile {
            path: "CONTRIBUTING.md",
            content: r#"# Contributing

## Development Setup

```bash
git clone <repo>
cargo build
cargo test
```

## Quality Gates

All PRs must pass:
- `cargo fmt --all -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo deny check`

## Commit Messages

Use conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
"#
            .to_string(),
            description: "Standard contribution guide",
        },
        ScaffoldFile {
            path: "SAFETY.md",
            content: r#"# Safety Documentation

## Unsafe Code Policy

This crate minimizes unsafe code usage. Any `unsafe` blocks must:
1. Have a `// SAFETY:` comment explaining the invariant
2. Be reviewed by at least one maintainer
3. Have corresponding test coverage

## Current Unsafe Usage

Run `cargo geiger` to audit unsafe code in this crate and its dependencies.
"#
            .to_string(),
            description: "Safety documentation",
        },
        ScaffoldFile {
            path: ".github/dependabot.yml",
            content: r#"version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 3
"#
            .to_string(),
            description: "Dependabot config",
        },
    ]
}

// ── Scaffold Execution ────────────────────────────────────────────────────

pub fn scaffold_repo(
    repo_path: &Path,
    files: &[ScaffoldFile],
    force: bool,
    diff: bool,
) -> Result<ScaffoldResult> {
    let mut created = 0;
    let mut skipped = 0;
    let mut overwritten = 0;

    for file in files {
        let target = repo_path.join(file.path);
        let exists = target.exists();

        if diff {
            // Diff mode: just show what would happen, no writes
            if exists && !force {
                println!(
                    "  [{}] {} {}",
                    c::dim("SKIP"),
                    c::dim(file.path),
                    c::dim("(exists)")
                );
                skipped += 1;
            } else if exists && force {
                println!("  [{}] {}", c::warn("OVERWRITE"), c::path(file.path));
                overwritten += 1;
            } else {
                println!(
                    "  [{}] {} - {}",
                    c::pass("CREATE"),
                    c::path(file.path),
                    c::dim(file.description)
                );
                created += 1;
            }
            continue;
        }

        // Write mode
        if exists && !force {
            println!(
                "  [{}] {} {}",
                c::dim("SKIP"),
                c::dim(file.path),
                c::dim("(exists)")
            );
            skipped += 1;
            continue;
        }

        // Create parent directories if needed
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory {}", parent.display()))?;
            }
        }

        if exists {
            println!("  [{}] {}", c::warn("OVERWRITE"), c::path(file.path));
            overwritten += 1;
        } else {
            println!(
                "  [{}] {} - {}",
                c::pass("CREATE"),
                c::path(file.path),
                c::dim(file.description)
            );
            created += 1;
        }

        std::fs::write(&target, &file.content)
            .with_context(|| format!("Failed to write {}", target.display()))?;
    }

    Ok(ScaffoldResult {
        created,
        skipped,
        overwritten,
    })
}

// ── Handler ───────────────────────────────────────────────────────────────

pub async fn handle_stack_scaffold(
    all: bool,
    template: &str,
    diff: bool,
    force: bool,
) -> Result<()> {
    let files = get_scaffold_files(template);

    println!();
    println!("{}", c::header("Stack Scaffold"));
    println!("{}", c::rule());
    println!("  Template: {}", c::path(template));
    println!("  Files:    {}", c::number(&files.len().to_string()));
    if diff {
        println!("  Mode:     {}", c::dim("diff (dry run)"));
    } else if force {
        println!("  Mode:     {}", c::warn("force overwrite"));
    }
    println!("{}", c::separator());

    let target_paths = discover_targets(all)?;

    let mut total_created = 0;
    let mut total_skipped = 0;
    let mut total_overwritten = 0;

    for target in &target_paths {
        let name = target
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| target.display().to_string());
        println!("  {}", c::subheader(&name));

        let result = scaffold_repo(target, &files, force, diff)?;
        total_created += result.created;
        total_skipped += result.skipped;
        total_overwritten += result.overwritten;
    }

    // Summary
    println!("{}", c::separator());
    let mut summary_parts = Vec::new();
    if total_created > 0 {
        summary_parts.push(format!("{} created", c::pass(&total_created.to_string())));
    }
    if total_skipped > 0 {
        summary_parts.push(format!("{} skipped", c::dim(&total_skipped.to_string())));
    }
    if total_overwritten > 0 {
        summary_parts.push(format!(
            "{} overwritten",
            c::warn(&total_overwritten.to_string())
        ));
    }
    if summary_parts.is_empty() {
        println!("  {}", c::dim("No changes"));
    } else {
        println!("  {}", summary_parts.join("  "));
    }
    println!("{}", c::rule());
    println!();

    Ok(())
}

// ── Target Discovery ──────────────────────────────────────────────────────

fn discover_targets(all: bool) -> Result<Vec<std::path::PathBuf>> {
    if !all {
        // Current repo only
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        return Ok(vec![cwd]);
    }

    // Use the same stack repo discovery logic as stack_sync_handler
    let base_path = if let Ok(cwd) = std::env::current_dir() {
        if let Some(parent) = cwd.parent() {
            parent.to_path_buf()
        } else {
            std::env::var_os("HOME")
                .map(|h| std::path::PathBuf::from(h).join("src"))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        }
    } else {
        std::env::var_os("HOME")
            .map(|h| std::path::PathBuf::from(h).join("src"))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    };

    let stack_repos = [
        "trueno",
        "trueno-graph",
        "trueno-db",
        "trueno-rag",
        "trueno-viz",
        "trueno-zram-core",
        "aprender",
        "paiml-mcp-agent-toolkit",
        "certeza",
        "bashrs",
        "probar",
        "renacer",
        "presentar",
        "pmcp",
    ];

    let mut targets = Vec::new();
    for repo in &stack_repos {
        let path = base_path.join(repo);
        if path.exists() {
            targets.push(path);
        }
    }

    if targets.is_empty() {
        anyhow::bail!(
            "No stack repos found in {}. Run from within a stack repo or check your directory layout.",
            base_path.display()
        );
    }

    Ok(targets)
}

// ── Unit Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_scaffold_files_default_template() {
        let files = get_scaffold_files("sqi-a-minus");
        assert!(files.len() >= 5);
        assert!(files.iter().any(|f| f.path == ".clippy.toml"));
        assert!(files.iter().any(|f| f.path == "deny.toml"));
        assert!(files.iter().any(|f| f.path == "CONTRIBUTING.md"));
    }

    #[test]
    fn test_scaffold_file_content_not_empty() {
        let files = get_scaffold_files("sqi-a-minus");
        for f in &files {
            assert!(!f.content.is_empty(), "File {} has empty content", f.path);
        }
    }

    #[test]
    fn test_scaffold_creates_files_in_temp_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let files = get_scaffold_files("sqi-a-minus");
        let result = scaffold_repo(temp_dir.path(), &files, false, false);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(temp_dir.path().join(".clippy.toml").exists());
        assert!(temp_dir.path().join("deny.toml").exists());
        assert!(temp_dir.path().join("CONTRIBUTING.md").exists());
        assert!(temp_dir.path().join("SAFETY.md").exists());
        assert!(temp_dir.path().join(".github/dependabot.yml").exists());
        assert_eq!(r.created, 5);
        assert_eq!(r.skipped, 0);
    }

    #[test]
    fn test_scaffold_skips_existing() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::write(temp_dir.path().join(".clippy.toml"), "existing").unwrap();
        let files = get_scaffold_files("sqi-a-minus");
        let result = scaffold_repo(temp_dir.path(), &files, false, false);
        assert!(result.is_ok());
        // Existing file should NOT be overwritten
        let content = std::fs::read_to_string(temp_dir.path().join(".clippy.toml")).unwrap();
        assert_eq!(content, "existing");
        let r = result.unwrap();
        assert_eq!(r.skipped, 1);
        assert_eq!(r.created, 4);
    }

    #[test]
    fn test_scaffold_force_overwrites() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::write(temp_dir.path().join(".clippy.toml"), "old").unwrap();
        let files = get_scaffold_files("sqi-a-minus");
        let result = scaffold_repo(temp_dir.path(), &files, true, false);
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp_dir.path().join(".clippy.toml")).unwrap();
        assert_ne!(content, "old");
        let r = result.unwrap();
        assert_eq!(r.overwritten, 1);
        assert_eq!(r.created, 4);
    }

    #[test]
    fn test_scaffold_diff_mode_no_writes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let files = get_scaffold_files("sqi-a-minus");
        let result = scaffold_repo(temp_dir.path(), &files, false, true);
        assert!(result.is_ok());
        // diff mode should NOT create files
        assert!(!temp_dir.path().join("deny.toml").exists());
        assert!(!temp_dir.path().join(".clippy.toml").exists());
        let r = result.unwrap();
        assert_eq!(r.created, 5);
        assert_eq!(r.skipped, 0);
    }

    #[test]
    fn test_scaffold_diff_mode_shows_existing_as_skip() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::write(temp_dir.path().join("deny.toml"), "existing").unwrap();
        let files = get_scaffold_files("sqi-a-minus");
        let result = scaffold_repo(temp_dir.path(), &files, false, true);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.skipped, 1);
        assert_eq!(r.created, 4);
    }

    #[test]
    fn test_scaffold_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let files = get_scaffold_files("sqi-a-minus");
        let result = scaffold_repo(temp_dir.path(), &files, false, false);
        assert!(result.is_ok());
        // .github/ directory should have been created
        assert!(temp_dir.path().join(".github").is_dir());
        assert!(temp_dir.path().join(".github/dependabot.yml").exists());
    }

    #[test]
    fn test_unknown_template_falls_back() {
        let files = get_scaffold_files("unknown-template");
        // Falls back to sqi-a-minus
        assert!(files.len() >= 5);
    }
}
