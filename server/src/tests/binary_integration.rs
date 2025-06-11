#[cfg(test)]
mod binary_integration_tests {
    use std::process::{Command, Stdio};
    use std::io::Write;
    use tempfile::TempDir;

    fn get_binary_path() -> String {
        // Use the test binary path
        env!("CARGO_BIN_EXE_pmat")
    }

    #[test]
    fn test_binary_help_flag() {
        let output = Command::new(get_binary_path())
            .arg("--help")
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("paiml-mcp-agent-toolkit"));
        assert!(stdout.contains("Commands:"));
    }

    #[test]
    fn test_binary_version_flag() {
        let output = Command::new(get_binary_path())
            .arg("--version")
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("paiml-mcp-agent-toolkit"));
    }

    #[test]
    fn test_binary_invalid_command() {
        let output = Command::new(get_binary_path())
            .arg("invalid-command")
            .output()
            .expect("Failed to execute binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("error") || stderr.contains("unrecognized"));
    }

    #[test]
    fn test_binary_list_templates() {
        let output = Command::new(get_binary_path())
            .args(&["list", "templates"])
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("makefile") || stdout.contains("gitignore"));
    }

    #[test]
    fn test_binary_analyze_complexity() {
        let test_dir = TempDir::new().unwrap();
        let src_dir = test_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        
        std::fs::write(
            src_dir.join("main.rs"),
            r#"
            fn main() {
                println!("Hello, world!");
            }
            "#
        ).unwrap();

        let output = Command::new(get_binary_path())
            .args(&["analyze", "complexity", test_dir.path().to_str().unwrap()])
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
    }

    #[test]
    fn test_binary_mcp_mode() {
        let mut child = Command::new(get_binary_path())
            .env("MCP_VERSION", "1.0")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn binary");

        // Send initialize request
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(br#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"1.0","capabilities":{}},"id":1}"#).unwrap();
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();

        // Give it time to process
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Kill the process
        let _ = child.kill();
        
        // Check that it started in MCP mode
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("jsonrpc") || stdout.contains("result"));
    }

    #[test]
    fn test_binary_environment_variables() {
        let output = Command::new(get_binary_path())
            .env("RUST_LOG", "debug")
            .env("PMAT_THREADS", "2")
            .args(&["list", "templates"])
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
    }

    #[test]
    fn test_binary_json_output() {
        let test_dir = TempDir::new().unwrap();
        std::fs::write(
            test_dir.path().join("test.rs"),
            "fn main() {}"
        ).unwrap();

        let output = Command::new(get_binary_path())
            .args(&[
                "analyze", 
                "complexity", 
                test_dir.path().to_str().unwrap(),
                "--output", 
                "json"
            ])
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should be valid JSON
        assert!(stdout.contains("{") || stdout.contains("["));
    }

    #[test]
    fn test_binary_analyze_dag() {
        let test_dir = TempDir::new().unwrap();
        let src_dir = test_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
            pub mod utils;
            pub fn lib_func() {}
            "#
        ).unwrap();
        
        std::fs::write(
            src_dir.join("utils.rs"),
            r#"
            pub fn util_func() {}
            "#
        ).unwrap();

        let output = Command::new(get_binary_path())
            .args(&["analyze", "dag", test_dir.path().to_str().unwrap()])
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
    }

    #[test]
    fn test_binary_generate_template() {
        let test_dir = TempDir::new().unwrap();
        let output_file = test_dir.path().join(".gitignore");

        let output = Command::new(get_binary_path())
            .args(&[
                "generate",
                "gitignore",
                "rust/cli",
                "-o",
                output_file.to_str().unwrap()
            ])
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
        assert!(output_file.exists());
        
        let content = std::fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("target/") || content.contains("Cargo.lock"));
    }

    #[test]
    fn test_binary_context_command() {
        let test_dir = TempDir::new().unwrap();
        std::fs::write(
            test_dir.path().join("README.md"),
            "# Test Project"
        ).unwrap();

        let output = Command::new(get_binary_path())
            .args(&["context", test_dir.path().to_str().unwrap()])
            .output()
            .expect("Failed to execute binary");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("README.md") || stdout.contains("Project Structure"));
    }

    #[test]
    fn test_binary_diagnose_command() {
        let output = Command::new(get_binary_path())
            .args(&["diagnose", "."])
            .output()
            .expect("Failed to execute binary");

        // Diagnose should always succeed
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Diagnostics") || stdout.contains("System"));
    }

    #[test]
    fn test_binary_error_handling() {
        // Test with non-existent directory
        let output = Command::new(get_binary_path())
            .args(&["analyze", "complexity", "/non/existent/path"])
            .output()
            .expect("Failed to execute binary");

        assert!(!output.status.success());
    }

    #[test]
    fn test_binary_concurrent_execution() {
        use std::thread;
        
        let handles: Vec<_> = (0..3).map(|i| {
            thread::spawn(move || {
                let output = Command::new(get_binary_path())
                    .args(&["list", "templates", "--output", "json"])
                    .output()
                    .expect("Failed to execute binary");
                
                assert!(output.status.success(), "Thread {} failed", i);
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}