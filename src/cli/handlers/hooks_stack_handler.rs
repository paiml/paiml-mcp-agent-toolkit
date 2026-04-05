#![cfg_attr(coverage_nightly, coverage(off))]
//! Stack-wide hook management for sovereign AI repos.
//!
//! Implements `pmat hooks install --stack` and `pmat hooks status --stack`
//! to manage pre-commit hooks across all batuta stack repositories.

use crate::cli::colors as c;
use crate::cli::OutputFormat;
use anyhow::Result;
use std::path::PathBuf;

/// PMAT marker comment embedded in generated hooks for identification.
const PMAT_STACK_MARKER: &str = "# PMAT Standardized Pre-Commit Hook (auto-installed)";

/// Default sovereign AI stack repository names.
const DEFAULT_STACK_REPOS: &[&str] = &[
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

/// Information about a discovered stack repository.
#[derive(Debug, Clone)]
struct StackRepo {
    name: String,
    path: PathBuf,
    exists: bool,
}

/// Hook status for a single repository.
#[derive(Debug, Clone, PartialEq, Eq)]
enum HookState {
    /// PMAT hook is installed and current
    Installed,
    /// No pre-commit hook present
    Missing,
    /// Hook exists but is not PMAT-managed
    NonPmat,
    /// PMAT hook exists but content differs from current template
    Outdated,
}

impl std::fmt::Display for HookState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookState::Installed => write!(f, "installed"),
            HookState::Missing => write!(f, "missing"),
            HookState::NonPmat => write!(f, "non-pmat"),
            HookState::Outdated => write!(f, "outdated"),
        }
    }
}

/// Discover stack repos from `~/src/<name>`.
///
/// Falls back to default list; each entry includes whether the repo
/// directory (with a `.git` subdirectory) actually exists on disk.
fn discover_stack_repos() -> Vec<StackRepo> {
    let home = std::env::var("HOME").unwrap_or_default();
    let src_dir = PathBuf::from(&home).join("src");

    DEFAULT_STACK_REPOS
        .iter()
        .map(|name| {
            let path = src_dir.join(name);
            let exists = path.join(".git").exists();
            StackRepo {
                name: name.to_string(),
                path,
                exists,
            }
        })
        .collect()
}

/// Generate the standardized pre-commit hook shell script.
fn generate_hook_content() -> String {
    format!(
        r#"#!/bin/sh
{PMAT_STACK_MARKER}
set -e

# 1. Cargo fmt check (Rust projects only)
if [ -f Cargo.toml ]; then
    cargo fmt --all -- --check 2>/dev/null || {{
        echo "cargo fmt failed. Run: cargo fmt --all"
        exit 1
    }}
fi

# 2. Orphan test detection (check for #[test] in files not in module tree)
if command -v pmat >/dev/null 2>&1; then
    pmat comply check --check orphan-tests --quiet 2>/dev/null || true
fi

# 3. TDG baseline update (fast, O(1) check)
if command -v pmat >/dev/null 2>&1 && [ -f .pmat/baseline.json ]; then
    pmat baseline update --quiet 2>/dev/null || true
fi
"#
    )
}

/// Detect the state of a pre-commit hook in a given repository.
fn detect_hook_state(repo_path: &PathBuf) -> HookState {
    let hook_path = repo_path.join(".git/hooks/pre-commit");
    if !hook_path.exists() {
        return HookState::Missing;
    }

    match std::fs::read_to_string(&hook_path) {
        Ok(content) => {
            if !content.contains(PMAT_STACK_MARKER) {
                HookState::NonPmat
            } else if content.trim() == generate_hook_content().trim() {
                HookState::Installed
            } else {
                HookState::Outdated
            }
        }
        Err(_) => HookState::NonPmat,
    }
}

/// Install pre-commit hooks across all sovereign AI stack repos.
///
/// When `update` is true, existing PMAT hooks are overwritten with the
/// latest template.  Non-PMAT hooks are never overwritten.
pub async fn handle_hooks_install_stack(update: bool) -> Result<()> {
    let repos = discover_stack_repos();
    let hook_content = generate_hook_content();

    println!("{}", c::header("Installing PMAT hooks across stack repos"));
    println!();

    let mut installed = 0u32;
    let mut skipped = 0u32;
    let mut missing_repos = 0u32;
    let mut errors = 0u32;

    for repo in &repos {
        if !repo.exists {
            missing_repos += 1;
            println!(
                "  {} {} {}",
                c::dim(&repo.name),
                c::dim("-"),
                c::dim("repo not found")
            );
            continue;
        }

        let hook_path = repo.path.join(".git/hooks/pre-commit");
        let state = detect_hook_state(&repo.path);

        match state {
            HookState::Missing => match write_hook(&hook_path, &hook_content) {
                Ok(()) => {
                    installed += 1;
                    println!(
                        "  {} {} {}",
                        c::pass("installed"),
                        c::label(&repo.name),
                        c::path(hook_path.to_string_lossy().as_ref())
                    );
                }
                Err(e) => {
                    errors += 1;
                    println!("  {} {} - {}", c::fail("error"), c::label(&repo.name), e);
                }
            },
            HookState::NonPmat => {
                skipped += 1;
                println!(
                    "  {} {} - non-PMAT hook exists, skipping",
                    c::warn("skipped"),
                    c::label(&repo.name),
                );
            }
            HookState::Installed => {
                skipped += 1;
                println!(
                    "  {} {} - already up to date",
                    c::skip("skipped"),
                    c::label(&repo.name),
                );
            }
            HookState::Outdated => {
                if update {
                    match write_hook(&hook_path, &hook_content) {
                        Ok(()) => {
                            installed += 1;
                            println!(
                                "  {} {} {}",
                                c::pass("updated"),
                                c::label(&repo.name),
                                c::path(hook_path.to_string_lossy().as_ref())
                            );
                        }
                        Err(e) => {
                            errors += 1;
                            println!("  {} {} - {}", c::fail("error"), c::label(&repo.name), e);
                        }
                    }
                } else {
                    skipped += 1;
                    println!(
                        "  {} {} - outdated, use --update to overwrite",
                        c::warn("skipped"),
                        c::label(&repo.name),
                    );
                }
            }
        }
    }

    println!();
    println!("{}", c::separator());
    println!(
        "  {}: {}  {}: {}  {}: {}  {}: {}",
        c::dim("Installed"),
        c::number(&installed.to_string()),
        c::dim("Skipped"),
        c::number(&skipped.to_string()),
        c::dim("Missing repos"),
        c::number(&missing_repos.to_string()),
        c::dim("Errors"),
        if errors > 0 {
            format!("{}{}{}", c::RED, errors, c::RESET)
        } else {
            c::number(&errors.to_string())
        },
    );

    if errors > 0 {
        Err(anyhow::anyhow!("{} hook(s) failed to install", errors))
    } else {
        Ok(())
    }
}

/// Show hook installation status across all sovereign AI stack repos.
pub async fn handle_hooks_status_stack(format: &OutputFormat) -> Result<()> {
    let repos = discover_stack_repos();

    match format {
        OutputFormat::Json => print_status_json(&repos),
        _ => print_status_table(&repos),
    }

    Ok(())
}

/// Print stack hook status as a table.
fn print_status_table(repos: &[StackRepo]) {
    println!("{}", c::header("Stack Hook Status"));
    println!();
    println!(
        "  {:<28} {:<12} {}",
        c::subheader("Repository"),
        c::subheader("Status"),
        c::subheader("Path"),
    );
    println!("  {}", c::separator());

    let mut installed_count = 0u32;
    let mut total_present = 0u32;

    for repo in repos {
        if !repo.exists {
            println!(
                "  {:<28} {:<12} {}",
                c::dim(&repo.name),
                c::dim("not found"),
                c::dim("-"),
            );
            continue;
        }

        total_present += 1;
        let state = detect_hook_state(&repo.path);
        let status_str = match &state {
            HookState::Installed => {
                installed_count += 1;
                c::pass("installed")
            }
            HookState::Missing => c::fail("missing"),
            HookState::NonPmat => format!("{}{}non-pmat{}", c::YELLOW, "", c::RESET),
            HookState::Outdated => c::warn("outdated"),
        };

        let hook_path = repo.path.join(".git/hooks/pre-commit");
        let path_str = if hook_path.exists() {
            c::path(hook_path.to_string_lossy().as_ref())
        } else {
            c::dim("-")
        };

        // Use raw name (no ANSI) for fixed-width alignment, then print colored
        println!("  {:<28} {} {}", &repo.name, status_str, path_str,);
    }

    println!();
    println!(
        "  {}/{} stack repos have PMAT hooks installed",
        c::number(&installed_count.to_string()),
        c::number(&total_present.to_string()),
    );
}

/// Print stack hook status as JSON.
fn print_status_json(repos: &[StackRepo]) {
    let mut entries = Vec::new();
    for repo in repos {
        let (status, path_str) = if !repo.exists {
            ("not_found".to_string(), String::new())
        } else {
            let state = detect_hook_state(&repo.path);
            let hook_path = repo.path.join(".git/hooks/pre-commit");
            (
                state.to_string(),
                if hook_path.exists() {
                    hook_path.to_string_lossy().into_owned()
                } else {
                    String::new()
                },
            )
        };
        entries.push(format!(
            r#"    {{"name": "{}", "status": "{}", "path": "{}"}}"#,
            repo.name, status, path_str
        ));
    }
    println!("[\n{}\n]", entries.join(",\n"));
}

/// Write hook content to disk atomically (temp + rename, CB-1334).
fn write_hook(hook_path: &PathBuf, content: &str) -> Result<()> {
    // Ensure the hooks directory exists
    if let Some(parent) = hook_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Atomic write: temp file + rename
    let tmp_path = hook_path.with_extension("tmp");
    std::fs::write(&tmp_path, content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&tmp_path, perms)?;
    }

    std::fs::rename(&tmp_path, hook_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_stack_repos() {
        let repos = discover_stack_repos();
        assert!(!repos.is_empty());
        // Every entry should have a non-empty name
        for repo in &repos {
            assert!(!repo.name.is_empty());
            assert!(!repo.path.as_os_str().is_empty());
        }
    }

    #[test]
    fn test_default_repos_count() {
        assert_eq!(DEFAULT_STACK_REPOS.len(), 14);
        assert!(DEFAULT_STACK_REPOS.contains(&"trueno"));
        assert!(DEFAULT_STACK_REPOS.contains(&"paiml-mcp-agent-toolkit"));
        assert!(DEFAULT_STACK_REPOS.contains(&"aprender"));
    }

    #[test]
    fn test_generate_hook_content() {
        let content = generate_hook_content();
        // Must start with shebang
        assert!(content.starts_with("#!/bin/sh"));
        // Must contain the PMAT marker
        assert!(content.contains(PMAT_STACK_MARKER));
        // Must contain cargo fmt check
        assert!(content.contains("cargo fmt --all -- --check"));
        // Must contain orphan test detection
        assert!(content.contains("orphan-tests"));
        // Must contain TDG baseline update
        assert!(content.contains("pmat baseline update"));
    }

    #[test]
    fn test_hook_marker_detection() {
        // A hook containing the marker should be detected
        let content = generate_hook_content();
        assert!(content.contains(PMAT_STACK_MARKER));

        // Some random script should not have the marker
        let other = "#!/bin/sh\necho hello\n";
        assert!(!other.contains(PMAT_STACK_MARKER));
    }

    #[test]
    fn test_detect_hook_state_missing() {
        // A path that doesn't exist should yield Missing
        let fake = PathBuf::from("/tmp/pmat-test-nonexistent-repo-xyz");
        assert_eq!(detect_hook_state(&fake), HookState::Missing);
    }

    #[test]
    fn test_hook_state_display() {
        assert_eq!(HookState::Installed.to_string(), "installed");
        assert_eq!(HookState::Missing.to_string(), "missing");
        assert_eq!(HookState::NonPmat.to_string(), "non-pmat");
        assert_eq!(HookState::Outdated.to_string(), "outdated");
    }

    #[test]
    fn test_write_and_detect_hook() {
        let tmp = std::env::temp_dir().join("pmat-hook-test");
        let git_hooks = tmp.join(".git/hooks");
        let _ = std::fs::create_dir_all(&git_hooks);
        let hook_path = git_hooks.join("pre-commit");

        let content = generate_hook_content();

        // Write the hook
        write_hook(&hook_path, &content).unwrap();

        // Detect state - should be Installed
        assert_eq!(detect_hook_state(&tmp), HookState::Installed);

        // Modify the hook slightly to make it outdated
        std::fs::write(&hook_path, format!("{}\n# extra", content)).unwrap();
        assert_eq!(detect_hook_state(&tmp), HookState::Outdated);

        // Write a non-PMAT hook
        std::fs::write(&hook_path, "#!/bin/sh\necho hello\n").unwrap();
        assert_eq!(detect_hook_state(&tmp), HookState::NonPmat);

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
