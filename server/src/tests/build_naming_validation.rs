#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_cargo_build_has_single_correct_binary() {
        // Run cargo metadata to get project info
        let output = Command::new("cargo")
            .args(["metadata", "--format-version", "1", "--no-deps"])
            .output()
            .expect("Failed to run cargo metadata");

        let metadata: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("Failed to parse cargo metadata");

        // Check package name
        let package_name = metadata["packages"][0]["name"].as_str().unwrap();
        assert_eq!(
            package_name, "paiml-mcp-agent-toolkit",
            "Package name must be 'paiml-mcp-agent-toolkit'"
        );

        // Check binary targets
        let targets = metadata["packages"][0]["targets"].as_array().unwrap();
        let binaries: Vec<&str> = targets
            .iter()
            .filter(|t| {
                t["kind"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::Value::String("bin".to_string()))
            })
            .map(|t| t["name"].as_str().unwrap())
            .collect();

        // Filter out feature-gated binaries
        let main_binaries: Vec<_> = binaries
            .iter()
            .filter(|&&name| name == "pmat")
            .copied()
            .collect();

        assert_eq!(
            main_binaries.len(),
            1,
            "There should be exactly one main binary target"
        );

        assert_eq!(main_binaries[0], "pmat", "Binary name must be 'pmat'");

        // Check that generate-installer is feature-gated
        if binaries.contains(&"generate-installer") {
            // This is OK as it's behind a feature flag
            assert!(binaries.len() <= 2, "Too many binaries: {:?}", binaries);
        }
    }

    #[test]
    fn test_no_old_package_references() {
        // Check that we don't have any old references to the old package name
        let output = Command::new("grep")
            .args([
                "-r",
                "mcp_template_server",
                "src/",
                "--include=*.rs",
                "--exclude=build_naming_validation.rs",
            ])
            .output()
            .expect("Failed to run grep");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.is_empty(),
            "Found references to old package name in source files:\n{}",
            stdout
        );
    }

    #[test]
    fn test_no_old_binary_references_in_workflows() {
        // Check GitHub Actions workflows for old binary names
        let old_binary_names = vec![
            "mcp_server_stateless",
            "mcp-template-server",
            "mcp_template_server",
        ];

        for old_name in &old_binary_names {
            let output = Command::new("grep")
                .args([
                    "-r",
                    old_name,
                    "../.github/workflows/",
                    "--include=*.yml",
                    "--include=*.yaml",
                ])
                .output()
                .expect("Failed to run grep");

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.is_empty(),
                "Found references to old binary name '{}' in GitHub Actions workflows:\n{}",
                old_name,
                stdout
            );
        }
    }

    #[test]
    fn test_correct_binary_name_in_workflows() {
        // Ensure workflows use the correct binary name
        let output = Command::new("grep")
            .args([
                "-r",
                "paiml-mcp-agent-toolkit",
                "../.github/workflows/",
                "--include=*.yml",
                "--include=*.yaml",
            ])
            .output()
            .expect("Failed to run grep");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "No references to 'paiml-mcp-agent-toolkit' found in GitHub Actions workflows. Workflows should use the correct binary name."
        );
    }

    #[test]
    fn test_no_wrong_repo_urls_in_workflows() {
        // Check for incorrect repository URLs in workflows
        let wrong_urls = vec![
            "pragmatic-ai-labs/paiml-mcp-agent-toolkit",
            "paiml/mcp-template-server",
            "pragmatic-ai-labs/mcp-template-server",
        ];

        for wrong_url in &wrong_urls {
            let output = Command::new("grep")
                .args([
                    "-r",
                    wrong_url,
                    "../.github/workflows/",
                    "--include=*.yml",
                    "--include=*.yaml",
                ])
                .output()
                .expect("Failed to run grep");

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.is_empty(),
                "Found references to incorrect repository URL '{}' in workflows:\n{}\nShould be 'paiml/paiml-mcp-agent-toolkit'",
                wrong_url,
                stdout
            );
        }
    }

    #[test]
    fn test_workspace_aware_cargo_commands_in_makefile() {
        // Read the server/Makefile
        let makefile_content =
            std::fs::read_to_string("Makefile").expect("Failed to read server/Makefile");

        // Define cargo commands that need workspace awareness
        let workspace_sensitive_commands = vec![
            "cargo audit",
            "cargo outdated",
            "cargo update",
            "cargo upgrade",
        ];

        // Check each line for problematic cargo commands
        for (line_num, line) in makefile_content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // Check for workspace-sensitive cargo commands
            for cmd in &workspace_sensitive_commands {
                if trimmed.contains(cmd)
                    && !trimmed.contains("cd ..")
                    && !trimmed.contains("--manifest-path")
                {
                    // Check if this is part of a shell command that changes directory
                    let is_workspace_aware = line.contains("cd ..")
                        || line.contains("$(PWD)/..")
                        || line.contains("--manifest-path");

                    assert!(
                        is_workspace_aware,
                        "Line {} in server/Makefile contains '{}' without workspace context.\n\
                         This command needs to run from workspace root.\n\
                         Found: {}\n\
                         Fix: Prepend with 'cd .. &&' or use '--manifest-path'",
                        line_num + 1,
                        cmd,
                        line.trim()
                    );
                }
            }
        }
    }

    #[test]
    fn test_cargo_lock_only_in_root() {
        use std::env;

        // Get the current directory to handle different test contexts
        let current_dir = env::current_dir().expect("Failed to get current directory");

        // Check if we're in the server directory or root
        if current_dir.ends_with("server") {
            // Running from server directory
            assert!(
                !std::path::Path::new("Cargo.lock").exists(),
                "Cargo.lock found in server/ directory. It should only exist in the workspace root!"
            );
            assert!(
                std::path::Path::new("../Cargo.lock").exists(),
                "Cargo.lock not found in workspace root directory!"
            );
        } else {
            // Running from root directory
            assert!(
                !std::path::Path::new("server/Cargo.lock").exists(),
                "Cargo.lock found in server/ directory. It should only exist in the workspace root!"
            );
            assert!(
                std::path::Path::new("Cargo.lock").exists(),
                "Cargo.lock not found in workspace root directory!"
            );
        }
    }

    #[test]
    fn test_build_script_workspace_aware() {
        // Read the build.rs file
        let build_script = std::fs::read_to_string("build.rs").expect("Failed to read build.rs");

        // Ensure build script doesn't hardcode Cargo.lock path
        if build_script.contains("\"Cargo.lock\"") {
            // Make sure it checks for workspace structure
            assert!(
                build_script.contains("../Cargo.lock")
                    || build_script.contains("Path::new(\"../Cargo.lock\")"),
                "build.rs references Cargo.lock but doesn't handle workspace structure.\n\
                 The build script should check if ../Cargo.lock exists for workspace builds."
            );
        }
    }
}
