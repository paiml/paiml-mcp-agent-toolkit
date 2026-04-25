//! Contract query handlers extracted from command_routing.rs (CB-040 file health).
//!
//! Handles: --contract-gaps, --asset-contracts, pv query delegation.

/// Delegate to `pv query` for cross-project contract search.
/// pv-compatibility spec §2.6: pv query integration.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn handle_pv_query_delegation(
    query: &str,
    limit: usize,
    format: &crate::cli::QueryOutputFormat,
) -> anyhow::Result<()> {
    let format_arg = match format {
        crate::cli::QueryOutputFormat::Json => "json",
        _ => "text",
    };
    let pv_dir = std::fs::canonicalize(".")
        .ok()
        .and_then(|p| p.parent().map(|pp| pp.join("provable-contracts")))
        .filter(|p| p.exists());

    if pv_dir.is_none() {
        eprintln!("error: ../provable-contracts/ directory not found.");
        eprintln!("  pmat query --contracts requires a provable-contracts sibling repo.");
        eprintln!(
            "  Clone it: git clone https://github.com/paiml/provable-contracts ../provable-contracts"
        );
        std::process::exit(1);
    }

    let mut cmd = std::process::Command::new("pv");
    cmd.args([
        "query",
        query,
        "--limit",
        &limit.to_string(),
        "-f",
        format_arg,
    ]);
    cmd.current_dir(pv_dir.as_ref().expect("checked above"));
    let output = cmd
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match output {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(_) => {
            eprintln!("error: `pv` CLI not found. Install with:");
            eprintln!("  cargo install --path ../provable-contracts/crates/provable-contracts-cli");
            std::process::exit(1);
        }
    }
}

/// Show functions without contract bindings, ranked by importance.
/// Uses ContractIndex from .pmat/binding-index.json + function index.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn handle_contract_gaps(
    project_path: &std::path::Path,
    limit: usize,
    format: &crate::cli::QueryOutputFormat,
) -> anyhow::Result<()> {
    use crate::services::contract_index::ContractIndex;

    let idx = ContractIndex::load(project_path);
    let idx = match idx {
        Some(i) => i,
        None => {
            eprintln!("No .pmat/binding-index.json found. Run: pmat comply refresh-bindings");
            std::process::exit(1);
        }
    };

    let src_dir = project_path.join("src");
    let mut all_files: Vec<String> = Vec::new();
    if src_dir.exists() {
        collect_rs_files(&src_dir, project_path, &mut all_files);
    }

    let gaps = idx.find_gaps(&all_files);
    let bound_count = all_files.len() - gaps.len();

    if matches!(format, crate::cli::QueryOutputFormat::Json) {
        let json = serde_json::json!({
            "total_files": all_files.len(),
            "bound_files": bound_count,
            "gap_files": gaps.len(),
            "gaps": gaps.iter().take(limit).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!(
            "Contract gaps: {}/{} source file(s) lack bindings\n",
            gaps.len(),
            all_files.len()
        );
        if idx.total_bindings > 0 {
            let pct = bound_count as f64 / all_files.len().max(1) as f64 * 100.0;
            println!(
                "Coverage: {:.1}% ({} bound, {} total bindings)\n",
                pct, bound_count, idx.total_bindings
            );
        }
        for (i, gap) in gaps.iter().enumerate().take(limit) {
            println!("  {}. {}", i + 1, gap);
        }
        if gaps.len() > limit {
            println!("  ... and {} more", gaps.len() - limit);
        }
    }

    Ok(())
}

/// Show non-code asset contract status via asset_validator service.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn handle_asset_contracts(
    project_path: &std::path::Path,
    format: &crate::cli::QueryOutputFormat,
) -> anyhow::Result<()> {
    use crate::services::asset_validator::{validate_all_assets, AssetStatus};

    let results = validate_all_assets(project_path);

    if matches!(format, crate::cli::QueryOutputFormat::Json) {
        let json: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "asset": r.name,
                    "cb_id": r.asset_type.cb_id(),
                    "status": format!("{:?}", r.status),
                    "message": r.message,
                    "issues": r.issues,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("Asset contract status:\n");
        for r in &results {
            let icon = match r.status {
                AssetStatus::Pass => "✓",
                AssetStatus::Warn => "⚠",
                AssetStatus::Skip => "-",
            };
            println!(
                "  {} {} ({}): {}",
                icon,
                r.name,
                r.asset_type.cb_id(),
                r.message
            );
        }
        let pass = results
            .iter()
            .filter(|r| r.status == AssetStatus::Pass)
            .count();
        let warn = results
            .iter()
            .filter(|r| r.status == AssetStatus::Warn)
            .count();
        let skip = results
            .iter()
            .filter(|r| r.status == AssetStatus::Skip)
            .count();
        println!("\n{} pass, {} warn, {} skip", pass, warn, skip);
    }

    Ok(())
}

/// Recursively collect .rs files relative to project root.
fn collect_rs_files(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
            if name == "target" || name == ".git" {
                continue;
            }
            collect_rs_files(&path, root, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().to_string());
            }
        }
    }
}

#[cfg(test)]
mod contract_query_handlers_tests {
    //! Covers collect_rs_files in contract_query_handlers.rs
    //! (33 uncov on broad, 0% cov). Skips the pv-spawning handlers.
    use super::*;

    #[test]
    fn test_collect_rs_files_empty_dir_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let mut out = Vec::new();
        collect_rs_files(tmp.path(), tmp.path(), &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn test_collect_rs_files_finds_top_level_rs_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.rs"), "").unwrap();
        let mut out = Vec::new();
        collect_rs_files(tmp.path(), tmp.path(), &mut out);
        assert_eq!(out, vec!["a.rs".to_string()]);
    }

    #[test]
    fn test_collect_rs_files_recurses_into_subdirs() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("nested").join("deeper");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(tmp.path().join("a.rs"), "").unwrap();
        std::fs::write(nested.join("b.rs"), "").unwrap();
        let mut out = Vec::new();
        collect_rs_files(tmp.path(), tmp.path(), &mut out);
        out.sort();
        assert_eq!(
            out,
            vec!["a.rs".to_string(), "nested/deeper/b.rs".to_string()]
        );
    }

    #[test]
    fn test_collect_rs_files_skips_target_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("target");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("ignored.rs"), "").unwrap();
        std::fs::write(tmp.path().join("kept.rs"), "").unwrap();
        let mut out = Vec::new();
        collect_rs_files(tmp.path(), tmp.path(), &mut out);
        assert_eq!(out, vec!["kept.rs".to_string()]);
    }

    #[test]
    fn test_collect_rs_files_skips_dot_git_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let git = tmp.path().join(".git");
        std::fs::create_dir(&git).unwrap();
        std::fs::write(git.join("hook.rs"), "").unwrap();
        std::fs::write(tmp.path().join("real.rs"), "").unwrap();
        let mut out = Vec::new();
        collect_rs_files(tmp.path(), tmp.path(), &mut out);
        assert_eq!(out, vec!["real.rs".to_string()]);
    }

    #[test]
    fn test_collect_rs_files_ignores_non_rs_extensions() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.rs"), "").unwrap();
        std::fs::write(tmp.path().join("b.md"), "").unwrap();
        std::fs::write(tmp.path().join("c.txt"), "").unwrap();
        let mut out = Vec::new();
        collect_rs_files(tmp.path(), tmp.path(), &mut out);
        assert_eq!(out, vec!["a.rs".to_string()]);
    }

    #[test]
    fn test_collect_rs_files_unreadable_dir_returns_empty() {
        let missing = std::path::Path::new("/tmp/pmat_missing_xyz_0xC0FFEE/nope");
        let mut out = Vec::new();
        collect_rs_files(missing, missing, &mut out);
        assert!(out.is_empty());
    }
}
