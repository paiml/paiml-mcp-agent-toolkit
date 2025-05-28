#[cfg(test)]
mod tests {
    use crate::installer::INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL;

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

        // Check for proper download URL construction
        assert!(script.contains("DOWNLOAD_URL="));
        assert!(script.contains("github.com"));
        assert!(script.contains("releases/download"));

        // Check for Rust target triple formats
        assert!(script.contains("x86_64-unknown-linux-gnu"));
        assert!(script.contains("x86_64-apple-darwin"));
    }

    #[test]
    fn test_installer_is_posix_compliant() {
        let script = INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL;

        // Should not contain bash-specific features
        assert!(!script.contains("[["));
        assert!(!script.contains("]]"));
        assert!(!script.contains("function "));
        assert!(!script.contains("source "));
        assert!(!script.contains("declare "));

        // Should use POSIX-compliant constructs
        assert!(script.contains("#!/bin/sh"));
        assert!(script.contains("case \"$os\""));
        assert!(script.contains("case \"$arch\""));
    }
}
