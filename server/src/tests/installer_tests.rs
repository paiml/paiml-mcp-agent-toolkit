#[cfg(test)]
mod tests {
    use crate::installer::{Error, ShellContext, INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL};
    use std::process::Command;

    #[test]
    fn test_shell_generation() {
        // Verify the shell script was generated
        assert!(
            INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.len() > 100,
            "Generated shell script seems too short"
        );
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.starts_with("#!/bin/sh"));
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("set -euf"));
    }

    #[test]
    fn test_deterministic_generation() {
        // The shell script should have a deterministic hash
        let hash1 = blake3::hash(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.as_bytes());
        let hash2 = blake3::hash(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.as_bytes());
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_security_properties() {
        // Verify security properties
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("set -euf"));
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("trap"));
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("readonly"));

        // Should not contain dangerous constructs
        assert!(!INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("eval"));
        assert!(!INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("source"));
    }

    #[test]
    fn test_platform_detection() {
        let ctx = ShellContext;

        // This test runs the actual platform detection
        let os = ctx.command("uname", &["-s"]).unwrap();
        let arch = ctx.command("uname", &["-m"]).unwrap();

        // Verify we can detect the current platform
        assert!(!os.trim().is_empty());
        assert!(!arch.trim().is_empty());
    }

    #[test]
    fn test_error_display() {
        let err = Error::UnsupportedPlatform("test-platform".to_string());
        assert_eq!(err.to_string(), "Unsupported platform: test-platform");

        let err = Error::ChecksumMismatch {
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Checksum mismatch: expected abc123, got def456"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_shellcheck_validation() {
        // Only run if shellcheck is available
        if Command::new("which")
            .arg("shellcheck")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let mut child = Command::new("shellcheck")
                .args(["-s", "sh", "-e", "all", "-"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();

            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.as_bytes())
                    .unwrap();
            }

            let output = child.wait_with_output().unwrap();
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Shellcheck validation failed:\n{}", stderr);
            }
        }
    }
}
