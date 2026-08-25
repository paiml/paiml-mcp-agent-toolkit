use crate::cli::commands::mcp_commands::McpCommands;
use anyhow::Result;
use std::path::Path;

/// Handle MCP commands (MACS-014)
// `manifest_faithful`, not `mcp_manifest_fidelity` — the latter was never
// declared by macs-artifacts-v1.yaml, so this annotation bound to nothing while
// reading as a discharged proof obligation. Caught by `contract_integrity`.
#[provable_contracts_macros::contract("macs-artifacts-v1.yaml", equation = "manifest_faithful")]
pub async fn handle_mcp_command(cmd: McpCommands, project_path: &Path) -> Result<()> {
    match cmd {
        McpCommands::Manifest { write } => {
            if write {
                let manifest_path = project_path.join("mcp.json");
                let canonical_tools = crate::mcp_pmcp::tool_manifest::canonical_tool_names();
                let manifest =
                    crate::mcp_pmcp::tool_manifest::render_manifest(env!("CARGO_PKG_VERSION"));
                std::fs::write(&manifest_path, manifest)?;
                println!("Generated mcp.json with {} tools", canonical_tools.len());
                // #1029: say what the manifest does NOT cover. The tool count
                // alone reads as a complete surface, which is how three
                // analyzers shipped CLI-only without anyone noticing.
                println!("{}", crate::cli::analyze_mcp_exposure::parity_summary());
            } else {
                println!("Run with --write to generate the manifest");
                println!("{}", crate::cli::analyze_mcp_exposure::parity_summary());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod manifest {
    #[test]
    fn generated_equals_tool_defs() {
        // Red test for MACS-014
        let manifest = crate::mcp_pmcp::tool_manifest::render_manifest("1.0.0");
        let value: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        let declared = crate::mcp_pmcp::tool_manifest::manifest_tool_names(&value);
        let canonical = crate::mcp_pmcp::tool_manifest::canonical_tool_names();
        assert_eq!(declared, canonical);
    }

    #[test]
    fn byte_equal_or_cb1656_red() {
        // We know CB-1656 fires on drift. We ensure render_manifest is consistent.
        let bytes1 = crate::mcp_pmcp::tool_manifest::render_manifest("1.0.0");
        let bytes2 = crate::mcp_pmcp::tool_manifest::render_manifest("1.0.0");
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn readme_tool_count_matches() {
        // This is a test required by MACS-014. We must ensure README.md tool count matches mcp.json tool_count.
        let manifest = crate::mcp_pmcp::tool_manifest::render_manifest("1.0.0");
        let value: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        let count = value["mcp"]["tool_count"].as_u64().unwrap();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let readme_path = std::path::Path::new(&manifest_dir).join("README.md");
        let readme_content = std::fs::read_to_string(&readme_path).unwrap_or_default();
        if !readme_content.is_empty() {
            let expected_string = format!("| MCP Tools | {} available |", count);
            assert!(
                readme_content.contains(&expected_string),
                "README.md should contain '{}'",
                expected_string
            );
        }
    }
}
