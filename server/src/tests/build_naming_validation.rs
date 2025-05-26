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

        assert_eq!(
            binaries.len(),
            1,
            "There should be exactly one binary target, found: {:?}",
            binaries
        );

        assert_eq!(
            binaries[0], "paiml-mcp-agent-toolkit",
            "Binary name must be 'paiml-mcp-agent-toolkit'"
        );
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
        let output = Command::new("grep")
            .args([
                "-r",
                "pragmatic-ai-labs/paiml-mcp-agent-toolkit",
                "../.github/workflows/",
                "--include=*.yml",
                "--include=*.yaml",
            ])
            .output()
            .expect("Failed to run grep");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.is_empty(),
            "Found references to incorrect repository URL 'pragmatic-ai-labs/paiml-mcp-agent-toolkit' in workflows:\n{}\nShould be 'paiml/paiml-mcp-agent-toolkit'",
            stdout
        );
    }
}
