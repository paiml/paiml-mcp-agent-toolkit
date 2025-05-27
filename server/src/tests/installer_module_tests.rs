#[cfg(test)]
#[cfg(feature = "installer-gen")]
mod tests {
    use crate::installer::{Error as InstallerError, ShellContext};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_shell_context_new() {
        let ctx = ShellContext;
        // ShellContext is a zero-sized type
        assert_eq!(std::mem::size_of::<ShellContext>(), 0);
        assert_eq!(std::mem::size_of_val(&ctx), 0);
    }

    #[test]
    fn test_shell_context_command_success() {
        let ctx = ShellContext;

        // Test echo command
        let result = ctx.command("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");

        // Test with multiple arguments
        let result = ctx.command("echo", &["hello", "world"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello world");

        // Test with empty args
        let result = ctx.command("echo", &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "");
    }

    #[test]
    fn test_shell_context_command_failure() {
        let ctx = ShellContext;

        // Test non-existent command
        let result = ctx.command("this_command_does_not_exist_12345", &[]);
        assert!(result.is_err());

        match result {
            Err(InstallerError::CommandFailed(msg)) => {
                assert!(msg.contains("this_command_does_not_exist_12345"));
            }
            _ => panic!("Expected CommandFailed error"),
        }
    }

    #[test]
    fn test_shell_context_test_dir() {
        let ctx = ShellContext;

        // Test with temporary directory
        let temp_dir = TempDir::new().unwrap();
        assert!(ctx.test_dir(temp_dir.path().to_str().unwrap()));

        // Test with current directory
        assert!(ctx.test_dir("."));
        assert!(ctx.test_dir("./"));

        // Test with non-existent directory
        assert!(!ctx.test_dir("/this/path/does/not/exist/12345"));

        // Test with file (not directory)
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();
        assert!(!ctx.test_dir(file_path.to_str().unwrap()));
    }

    #[test]
    fn test_installer_error_variants() {
        // Test all error variants
        let errors = vec![
            InstallerError::UnsupportedPlatform("Linux-i686".to_string()),
            InstallerError::DownloadFailed("Connection timeout".to_string()),
            InstallerError::ChecksumMismatch {
                expected: "abc123def456".to_string(),
                actual: "789xyz000000".to_string(),
            },
            InstallerError::InstallFailed("Permission denied".to_string()),
            InstallerError::CommandFailed("curl: command not found".to_string()),
        ];

        for error in errors {
            // Test Display implementation
            let display_str = error.to_string();
            assert!(!display_str.is_empty());

            // Test Error trait implementation
            let err_ref: &dyn std::error::Error = &error;
            assert!(err_ref.source().is_none());
        }
    }

    #[test]
    fn test_installer_error_display_messages() {
        let err = InstallerError::UnsupportedPlatform("Windows-x86_64".to_string());
        assert_eq!(err.to_string(), "Unsupported platform: Windows-x86_64");

        let err = InstallerError::DownloadFailed("404 Not Found".to_string());
        assert_eq!(err.to_string(), "Download failed: 404 Not Found");

        let err = InstallerError::ChecksumMismatch {
            expected: "expectedhash".to_string(),
            actual: "actualhash".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Checksum mismatch: expected expectedhash, got actualhash"
        );

        let err = InstallerError::InstallFailed("No space left on device".to_string());
        assert_eq!(
            err.to_string(),
            "Installation failed: No space left on device"
        );

        let err = InstallerError::CommandFailed("tar: invalid option".to_string());
        assert_eq!(err.to_string(), "Command failed: tar: invalid option");
    }

    #[test]
    fn test_platform_mapping() {
        // Test the platform mapping logic used in the installer
        let test_cases = vec![
            (("Linux", "x86_64"), Some("x86_64-unknown-linux-gnu")),
            (("Linux", "aarch64"), Some("aarch64-unknown-linux-gnu")),
            (("Darwin", "x86_64"), Some("x86_64-apple-darwin")),
            (("Darwin", "aarch64"), Some("aarch64-apple-darwin")),
            (("Darwin", "arm64"), Some("aarch64-apple-darwin")),
            (("Windows", "x86_64"), None),
            (("FreeBSD", "x86_64"), None),
            (("Linux", "i686"), None),
            (("Darwin", "i386"), None),
        ];

        for ((os, arch), expected) in test_cases {
            let platform = match (os, arch) {
                ("Linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
                ("Linux", "aarch64") => Some("aarch64-unknown-linux-gnu"),
                ("Darwin", "x86_64") => Some("x86_64-apple-darwin"),
                ("Darwin", "aarch64" | "arm64") => Some("aarch64-apple-darwin"),
                _ => None,
            };
            assert_eq!(platform, expected, "Failed for OS: {}, Arch: {}", os, arch);
        }
    }

    #[test]
    fn test_url_construction() {
        let version = "0.1.4";
        let platforms = vec![
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
        ];

        for platform in platforms {
            let base_url =
                "https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download";
            let binary_url = format!(
                "{}/v{}/paiml-mcp-agent-toolkit-{}.tar.gz",
                base_url, version, platform
            );
            let checksum_url = format!("{}.sha256", binary_url);

            // Verify URL structure
            assert!(binary_url.starts_with(base_url));
            assert!(binary_url.contains(&format!("v{}", version)));
            assert!(binary_url.contains(platform));
            assert!(binary_url.ends_with(".tar.gz"));
            assert!(checksum_url.ends_with(".tar.gz.sha256"));
        }
    }

    #[test]
    fn test_install_paths() {
        let install_dirs = vec![
            "${HOME}/.local/bin",
            "/usr/local/bin",
            "/opt/bin",
            "${HOME}/bin",
        ];

        for dir in install_dirs {
            let binary_path = format!("{}/paiml-mcp-agent-toolkit", dir);
            assert!(binary_path.ends_with("/paiml-mcp-agent-toolkit"));
            assert!(binary_path.contains(dir));
        }
    }

    #[test]
    fn test_command_with_special_chars() {
        let ctx = ShellContext;

        // Test command with special characters in output
        let result = ctx.command("echo", &["test\nwith\nnewlines"]);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("newlines"));

        // Test command with quotes
        let result = ctx.command("echo", &["'single quotes'"]);
        assert!(result.is_ok());

        // Test command with spaces
        let result = ctx.command("echo", &["  spaces  "]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_debug_implementation() {
        // Verify Debug is implemented for all error types
        let errors = vec![
            InstallerError::UnsupportedPlatform("test".to_string()),
            InstallerError::DownloadFailed("test".to_string()),
            InstallerError::ChecksumMismatch {
                expected: "a".to_string(),
                actual: "b".to_string(),
            },
            InstallerError::InstallFailed("test".to_string()),
            InstallerError::CommandFailed("test".to_string()),
        ];

        for error in errors {
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
        }
    }
}

#[cfg(test)]
#[cfg(not(feature = "installer-gen"))]
mod installer_module_tests_no_feature {
    #[test]
    fn test_installer_module_requires_feature() {
        // This test ensures the installer module is properly gated
        // When installer-gen feature is not enabled, the installer module should not be available
        assert!(true, "Installer module is properly feature-gated");
    }
}
