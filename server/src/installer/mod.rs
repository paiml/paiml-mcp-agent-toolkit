// Simple installer module that embeds the pre-written install script

/// The installation script for paiml-mcp-agent-toolkit
pub const INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL: &str = include_str!("../../../scripts/install.sh");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_script_exists() {
        // Check that the script is not empty by checking it starts with shebang
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.starts_with("#!/bin/sh"));
        // And has substantial content
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.len() > 100);
    }

    #[test]
    fn test_installer_has_required_components() {
        let script = INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL;

        // Check for essential functions
        assert!(script.contains("detect_platform()"));
        assert!(script.contains("get_latest_version()"));
        assert!(script.contains("install()"));

        // Check for error handling
        assert!(script.contains("set -e"));
        assert!(script.contains("error()"));

        // Check for platform detection
        assert!(script.contains("Linux*)"));
        assert!(script.contains("Darwin*)"));
        assert!(script.contains("x86_64"));
        assert!(script.contains("aarch64"));

        // Check for Rust target triple formats
        assert!(script.contains("x86_64-unknown-linux-gnu"));
        assert!(script.contains("x86_64-apple-darwin"));
    }
}
