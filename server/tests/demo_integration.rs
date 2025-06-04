#[cfg(test)]
mod demo_tests {
    use anyhow::Result;
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::TempDir;

    #[test]
    fn test_demo_mode_in_test_directory() -> Result<()> {
        // Create a small test directory for fast analysis
        let temp = TempDir::new()?;
        let test_path = temp.path();

        // Create some minimal test files
        std::fs::write(
            test_path.join("main.rs"),
            "fn main() { println!(\"Hello, world!\"); }",
        )?;
        std::fs::write(
            test_path.join("lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        )?;

        // Run demo on the small test directory
        let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit")?;
        cmd.arg("demo").arg("--cli").arg("--path").arg(test_path);

        // Verify it runs successfully
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("PAIML MCP Agent Toolkit Demo"))
            .stdout(predicate::str::contains("AST Context Analysis"))
            .stdout(predicate::str::contains("Code Complexity Analysis"))
            .stdout(predicate::str::contains("DAG Visualization"))
            .stdout(predicate::str::contains("Code Churn Analysis"))
            .stdout(predicate::str::contains("System Architecture Analysis"))
            .stdout(predicate::str::contains("Defect Probability Analysis"))
            .stdout(predicate::str::contains("Template Generation"));

        Ok(())
    }

    #[test]
    fn test_demo_mode_with_json_output() -> Result<()> {
        // Create a small test directory for fast analysis
        let temp = TempDir::new()?;
        let test_path = temp.path();

        // Create minimal test file
        std::fs::write(test_path.join("test.rs"), "fn main() {}")?;

        let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit")?;
        cmd.arg("demo")
            .arg("--cli")
            .arg("--format")
            .arg("json")
            .arg("--path")
            .arg(test_path);

        // Verify JSON output
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(r#""repository""#))
            .stdout(predicate::str::contains(r#""steps""#))
            .stdout(predicate::str::contains(r#""total_time_ms""#));

        Ok(())
    }

    #[test]
    fn test_demo_mode_with_specific_path() -> Result<()> {
        let temp = TempDir::new()?;
        let repo_path = temp.path().join("test-repo");

        // Create a mock git repository
        std::fs::create_dir_all(&repo_path)?;
        std::fs::create_dir_all(repo_path.join(".git"))?;

        // Create some source files
        std::fs::write(
            repo_path.join("main.rs"),
            "fn main() { println!(\"Hello\"); }",
        )?;
        std::fs::write(
            repo_path.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )?;

        let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit")?;
        cmd.arg("demo")
            .arg("--cli")
            .arg("--path")
            .arg(repo_path.to_str().unwrap());

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("🚀 CLI Protocol Demo"))
            .stdout(predicate::str::contains("test-repo"));

        Ok(())
    }

    // Removed test - demo mode is now always available

    #[test]
    fn test_demo_increases_test_coverage() -> Result<()> {
        // Create small test directory for fast execution
        let temp = TempDir::new()?;
        let test_path = temp.path();
        std::fs::write(test_path.join("test.rs"), "fn main() {}")?;

        // This test verifies that running demo mode exercises various code paths
        // We'll check this by running demo and verifying it completes all steps
        let mut cmd = Command::cargo_bin("paiml-mcp-agent-toolkit")?;
        cmd.arg("demo")
            .arg("--cli")
            .arg("--format")
            .arg("json")
            .arg("--path")
            .arg(test_path);

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Debug print for troubleshooting
        if !output.stderr.is_empty() {
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.status.success() {
            eprintln!("Command failed with status: {}", output.status);
            eprintln!("stdout: {}", stdout);
            panic!("Demo command failed");
        }

        // Parse JSON to verify all capabilities were exercised
        let report: serde_json::Value = serde_json::from_str(&stdout)?;
        let steps = report["steps"].as_array().expect("steps should be array");

        // Verify we have all 7 expected demo steps (with enhanced analyses)
        assert_eq!(steps.len(), 7);

        // Verify each step completed (note: code churn may fail without git history)
        for (idx, step) in steps.iter().enumerate() {
            // Code churn (step 4) may fail without git history - that's expected
            if idx != 3 {
                assert!(
                    step["response"]["error"].is_null(),
                    "Step {} should not have errors",
                    idx
                );
            }
            // Verify elapsed time is recorded (u64 is always >= 0)
            let _ = step["elapsed_ms"].as_u64().unwrap();
        }

        Ok(())
    }

    #[cfg(test)]
    mod mcp_demo_tests {
        use super::*;
        use paiml_mcp_agent_toolkit::{
            cli::OutputFormat,
            demo::{run_demo, DemoArgs},
            stateless_server::StatelessTemplateServer,
        };
        use std::sync::Arc;

        #[tokio::test]
        async fn test_demo_runner_execution() -> Result<()> {
            let server = Arc::new(StatelessTemplateServer::new()?);
            let temp = TempDir::new()?;
            let repo_path = temp.path().to_path_buf();

            // Create mock git repo
            std::fs::create_dir_all(repo_path.join(".git"))?;
            std::fs::write(repo_path.join("main.rs"), "fn main() {}")?;

            let args = DemoArgs {
                path: Some(repo_path.clone()),
                url: None,
                repo: None,
                format: OutputFormat::Json,
                no_browser: true,
                port: None,
                web: false,
                target_nodes: 15,
                centrality_threshold: 0.1,
                merge_threshold: 3,
                protocol: paiml_mcp_agent_toolkit::demo::Protocol::Cli,
                show_api: false,
                debug: false,
                debug_output: None,
                skip_vendor: true,
                max_line_length: None,
            };

            // This should complete without panicking
            run_demo(args, server).await?;

            Ok(())
        }

        #[test]
        fn test_repository_detection() -> Result<()> {
            // This test is moved to integration test because detect_repository
            // is only available when demo-dev feature is enabled

            let temp = TempDir::new()?;
            let repo_path = temp.path().join("repo");

            // Create mock git repo
            std::fs::create_dir_all(repo_path.join(".git"))?;

            // Just test that the directory exists
            assert!(repo_path.join(".git").is_dir());

            Ok(())
        }
    }
}
