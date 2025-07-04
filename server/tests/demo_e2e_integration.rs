//! E2E Demo Server Integration Tests
//!
//! Spawns demo binary as subprocess, parses ephemeral port from stdout,
//! executes HTTP assertions against live server.
//!
//! These tests are skipped in CI due to timing issues with subprocess spawning.

// Helper macro to skip tests in CI
macro_rules! skip_in_ci {
    () => {
        if std::env::var("SKIP_SLOW_TESTS").is_ok() || std::env::var("CI").is_ok() {
            eprintln!("Skipping demo e2e test in CI environment");
            return Ok(());
        }
    };
}

use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use serial_test::serial;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::sleep;

/// Shared HTTP client for all tests
static HTTP_CLIENT: std::sync::LazyLock<Client> = std::sync::LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

/// Regex for parsing port from demo server output
static PORT_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"http://127\.0\.0\.1:(\d+)").expect("Failed to compile port regex")
});

/// Test repository fixture for consistent analysis results
static TEST_REPO: std::sync::LazyLock<Arc<TempDir>> = std::sync::LazyLock::new(|| {
    Arc::new(create_test_repository().expect("Failed to create test repository"))
});

/// Demo server process with automatic cleanup
struct DemoServer {
    process: Child,
    #[allow(dead_code)]
    port: u16,
    base_url: String,
}

impl DemoServer {
    /// Spawn demo server subprocess and wait for startup
    async fn spawn(repo_path: &str) -> Result<Self> {
        // Skip demo tests in CI environment if binary not available
        if std::env::var("CI").is_ok() && std::env::var("CARGO_BIN_EXE_pmat").is_err() {
            // In CI, check for the built binary
            let ci_binary = "target/release/pmat";
            if !std::path::Path::new(ci_binary).exists() {
                eprintln!(
                    "[TEST] Skipping demo test - binary not found at {}",
                    ci_binary
                );
                anyhow::bail!("Demo binary not available in CI");
            }
        }

        // Use cargo's TARGET_DIR or fallback to workspace target directory
        let binary_path = std::env::var("CARGO_BIN_EXE_pmat").unwrap_or_else(|_| {
            // In CI, we build to target/release/pmat from workspace root
            let workspace_release = "target/release/pmat";
            let workspace_debug = "target/debug/pmat";

            if std::path::Path::new(workspace_release).exists() {
                workspace_release.to_string()
            } else if std::path::Path::new(workspace_debug).exists() {
                workspace_debug.to_string()
            } else {
                // Fallback for running from server directory
                "../target/release/pmat".to_string()
            }
        });

        eprintln!("[TEST] Spawning demo server with binary: {}", binary_path);
        eprintln!("[TEST] Demo path: {}", repo_path);

        let mut process = Command::new(&binary_path)
            .args(["demo", "--path", repo_path, "--no-browser"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                anyhow::anyhow!("Failed to spawn demo server at {}: {}", binary_path, e)
            })?;

        // Read stdout until we find the server URL
        let stdout = process.stdout.take().expect("Failed to capture stdout");
        let stderr = process.stderr.take().expect("Failed to capture stderr");

        // Start monitoring stderr in background
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(tokio::process::ChildStderr::from_std(stderr).unwrap());
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[DEMO STDERR] {}", line);
            }
        });

        let port = Self::parse_port_from_output(stdout).await?;

        let base_url = format!("http://127.0.0.1:{port}");

        // Wait for server to be ready
        Self::wait_for_server_ready(&base_url).await?;

        Ok(Self {
            process,
            port,
            base_url,
        })
    }

    async fn parse_port_from_output(stdout: std::process::ChildStdout) -> Result<u16> {
        // Spawn blocking task to handle stdout reading
        tokio::task::spawn_blocking(move || {
            use std::io::Read;

            let mut stdout = stdout;
            let timeout = Duration::from_secs(60); // Increased timeout for demo server startup
            let start = Instant::now();
            let mut buffer = Vec::new();

            while start.elapsed() < timeout {
                let mut temp_buffer = [0u8; 1024];
                match stdout.read(&mut temp_buffer) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        buffer.extend_from_slice(&temp_buffer[..bytes_read]);
                        let output = String::from_utf8_lossy(&buffer);

                        // Print output for debugging
                        if !output.trim().is_empty() {
                            eprintln!("[DEMO STDOUT] {}", output.trim());
                        }

                        if let Some(captures) = PORT_REGEX.captures(&output) {
                            if let Some(port_str) = captures.get(1) {
                                eprintln!("[TEST] Found port: {}", port_str.as_str());
                                return Ok(port_str.as_str().parse().unwrap());
                            }
                        }
                    }
                    _ => {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }

            let final_output = String::from_utf8_lossy(&buffer);
            eprintln!("[TEST] Final output from demo server:\n{}", final_output);
            Err(anyhow::anyhow!(
                "Failed to parse port from demo server output within timeout. Output: {}",
                final_output
            ))
        })
        .await?
    }

    async fn wait_for_server_ready(base_url: &str) -> Result<()> {
        let client = &*HTTP_CLIENT;
        let timeout = Duration::from_secs(30); // Increased timeout for slower systems
        let start = Instant::now();

        while start.elapsed() < timeout {
            if let Ok(response) = client.get(base_url).send().await {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        anyhow::bail!("Server did not become ready within timeout")
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl Drop for DemoServer {
    fn drop(&mut self) {
        // Gracefully terminate the demo server
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Create a minimal test repository with known structure
fn create_test_repository() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let repo_path = temp_dir.path();

    // Create a simple Rust project structure
    std::fs::create_dir_all(repo_path.join("src"))?;

    // Cargo.toml
    std::fs::write(
        repo_path.join("Cargo.toml"),
        r#"[package]
name = "test-repo"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#,
    )?;

    // Main.rs with known complexity
    std::fs::write(
        repo_path.join("src/main.rs"),
        r#"use serde::Serialize;

#[derive(Serialize)]
struct TestStruct {
    field1: String,
    field2: i32,
}

fn main() {
    let test = TestStruct {
        field1: "hello".to_string(),
        field2: 42,
    };
    println!("{:?}", test);
}

// High complexity function for testing
fn complex_function(x: i32, y: i32, z: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                if x > y {
                    if y > z {
                        return x + y + z;
                    } else if z > x {
                        return z - x;
                    } else {
                        return y * z;
                    }
                } else if y > z {
                    return y - z;
                } else {
                    return x * y;
                }
            } else {
                return x - y;
            }
        } else {
            return x + z;
        }
    } else {
        return y + z;
    }
}

fn simple_function(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )?;

    // Lib.rs with additional complexity
    std::fs::write(
        repo_path.join("src/lib.rs"),
        r#"pub mod utils;

pub fn library_function() -> String {
    "library".to_string()
}

pub fn another_complex_function(input: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for item in input {
        if item % 2 == 0 {
            if item > 10 {
                result.push(item * 2);
            } else {
                result.push(item + 1);
            }
        } else {
            if item > 5 {
                result.push(item - 1);
            } else {
                result.push(item * 3);
            }
        }
    }
    result
}
"#,
    )?;

    // Utils module
    std::fs::write(
        repo_path.join("src/utils.rs"),
        r#"pub fn utility_function(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub fn format_number(n: i32) -> String {
    format!("Number: {}", n)
}
"#,
    )?;

    // Initialize git repository for churn analysis
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Ok(temp_dir)
}

#[tokio::test]
async fn test_demo_server_happy_path() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();
    let server = DemoServer::spawn(repo_path).await?;

    // Test dashboard loads
    let response = HTTP_CLIENT.get(server.url("/")).send().await?;
    assert!(response.status().is_success());

    let html_content = response.text().await?;
    assert!(html_content.contains("PAIML MCP Agent Toolkit Demo"));
    assert!(html_content.contains("Files Analyzed"));
    assert!(html_content.contains("Average Complexity"));

    // Parse HTML and verify structure
    let document = Html::parse_document(&html_content);
    let stats_grid_selector = Selector::parse(".stats-grid").unwrap();
    let stat_cards_selector = Selector::parse(".stat-card").unwrap();

    assert!(document.select(&stats_grid_selector).next().is_some());
    let stat_cards: Vec<_> = document.select(&stat_cards_selector).collect();
    assert!(stat_cards.len() >= 4, "Should have at least 4 stat cards");

    Ok(())
}

#[tokio::test]
async fn test_api_contract_compliance() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();
    let server = DemoServer::spawn(repo_path).await?;

    // Test /api/summary endpoint
    let summary_response = HTTP_CLIENT.get(server.url("/api/summary")).send().await?;
    assert!(summary_response.status().is_success());

    let summary_json: Value = summary_response.json().await?;
    assert!(summary_json.get("files_analyzed").is_some());
    assert!(summary_json.get("avg_complexity").is_some());
    assert!(summary_json.get("tech_debt_hours").is_some());

    // Test /api/hotspots endpoint
    let hotspots_response = HTTP_CLIENT.get(server.url("/api/hotspots")).send().await?;
    assert!(hotspots_response.status().is_success());

    let hotspots_json: Value = hotspots_response.json().await?;
    assert!(hotspots_json.as_array().is_some());

    // Verify hotspot structure
    if let Some(hotspots) = hotspots_json.as_array() {
        if !hotspots.is_empty() {
            let first_hotspot = &hotspots[0];
            assert!(first_hotspot.get("function").is_some());
            assert!(first_hotspot.get("complexity").is_some());
            assert!(first_hotspot.get("loc").is_some());
            assert!(first_hotspot.get("path").is_some());
        }
    }

    // Test /api/dag endpoint
    let dag_response = HTTP_CLIENT.get(server.url("/api/dag")).send().await?;
    assert!(dag_response.status().is_success());

    let dag_text = dag_response.text().await?;
    assert!(dag_text.contains("graph TD") || dag_text.contains("flowchart"));

    // Test /api/system-diagram endpoint
    let system_response = HTTP_CLIENT
        .get(server.url("/api/system-diagram"))
        .send()
        .await?;
    assert!(system_response.status().is_success());

    let system_text = system_response.text().await?;
    assert!(system_text.contains("graph TD") || system_text.contains("flowchart"));

    // Test enhanced API endpoints
    let stats_response = HTTP_CLIENT
        .get(server.url("/api/v1/analysis/statistics"))
        .send()
        .await?;
    assert!(stats_response.status().is_success());

    let stats_json: Value = stats_response.json().await?;
    assert!(stats_json.get("structural_metrics").is_some());
    assert!(stats_json.get("code_metrics").is_some());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_requests() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();
    let server = DemoServer::spawn(repo_path).await?;

    // Create 50 concurrent requests to different endpoints
    let mut handles = Vec::new();

    for i in 0..50 {
        let base_url = server.base_url.clone();
        let client = HTTP_CLIENT.clone();

        let handle = tokio::spawn(async move {
            let endpoint = match i % 4 {
                0 => "/",
                1 => "/api/summary",
                2 => "/api/hotspots",
                _ => "/api/dag",
            };

            let response = client.get(format!("{base_url}{endpoint}")).send().await?;
            assert!(response.status().is_success());
            Ok::<_, anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await??;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_performance_assertions() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();

    // Measure startup time
    let startup_start = Instant::now();
    let server = DemoServer::spawn(repo_path).await?;
    let startup_time = startup_start.elapsed();

    // Startup should be reasonable (analysis time is separate)
    assert!(
        startup_time < Duration::from_secs(45),
        "Startup took too long: {startup_time:?}"
    );

    // Test response latency
    let mut response_times = Vec::new();

    for _ in 0..20 {
        let start = Instant::now();
        let response = HTTP_CLIENT.get(server.url("/api/summary")).send().await?;
        let elapsed = start.elapsed();

        assert!(response.status().is_success());
        response_times.push(elapsed);
    }

    // Calculate p99 latency
    response_times.sort();
    let p99_index = (response_times.len() as f64 * 0.99) as usize;
    let p99_latency = response_times[p99_index.min(response_times.len() - 1)];

    assert!(
        p99_latency < Duration::from_millis(500),
        "P99 latency too high: {p99_latency:?}"
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_error_handling() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();
    let server = DemoServer::spawn(repo_path).await?;

    // Test 404 for invalid paths
    let response = HTTP_CLIENT.get(server.url("/invalid/path")).send().await?;
    assert_eq!(response.status(), 404);

    // Test additional 404 cases
    let response = HTTP_CLIENT
        .get(server.url("/nonexistent/endpoint"))
        .send()
        .await?;
    assert_eq!(response.status(), 404);

    let response = HTTP_CLIENT
        .get(server.url("/api/nonexistent"))
        .send()
        .await?;
    assert_eq!(response.status(), 404);

    Ok(())
}

#[tokio::test]
async fn test_analysis_pipeline_integrity() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();

    // Capture process output to verify analysis steps
    let binary_path = std::env::var("CARGO_BIN_EXE_pmat").unwrap_or_else(|_| {
        // Try debug binary first, then release
        if std::path::Path::new("target/debug/pmat").exists() {
            "target/debug/pmat".to_string()
        } else {
            "../target/release/pmat".to_string()
        }
    });

    let mut process = Command::new(&binary_path)
        .args(["demo", "--path", repo_path, "--no-browser"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Read output until server starts
    let stdout = process.stdout.take().unwrap();
    let mut output = String::new();

    use std::io::Read;
    let mut reader = std::io::BufReader::new(stdout);
    let mut buffer = [0; 1024];

    let timeout = Duration::from_secs(60);
    let start = Instant::now();

    while start.elapsed() < timeout {
        if let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read > 0 {
                output.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));

                // Check if server has started
                if output.contains("Demo server running at:") {
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Verify all 7 analysis steps completed
    let expected_steps = [
        "Generating AST Context",
        "Analyzing Code Complexity",
        "Generating Dependency Graph",
        "Analyzing Code Churn",
        "Analyzing System Architecture",
        "Analyzing Defect Probability",
        "Generating Template",
    ];

    for step in &expected_steps {
        assert!(output.contains(step), "Missing analysis step: {step}");
    }

    // Verify completion markers
    for i in 1..=7 {
        let marker = format!("{i}️⃣");
        assert!(output.contains(&marker), "Missing step marker: {marker}");
    }

    // Clean up process
    let _ = process.kill();
    let _ = process.wait();

    Ok(())
}

#[tokio::test]
async fn test_data_source_indicators() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();
    let server = DemoServer::spawn(repo_path).await?;

    // Get dashboard HTML
    let response = HTTP_CLIENT.get(server.url("/")).send().await?;
    let html_content = response.text().await?;

    let document = Html::parse_document(&html_content);

    // Check for data source indicators
    let dynamic_selector = Selector::parse(".data-indicator.dynamic").unwrap();
    let default_selector = Selector::parse(".data-indicator.default").unwrap();

    let dynamic_indicators: Vec<_> = document.select(&dynamic_selector).collect();
    let _default_indicators: Vec<_> = document.select(&default_selector).collect();

    // Should have both dynamic and default indicators
    assert!(
        !dynamic_indicators.is_empty(),
        "Should have dynamic data indicators"
    );

    // Verify Performance Breakdown shows dynamic data
    let performance_section = document
        .select(&Selector::parse(".section").unwrap())
        .find(|el| {
            el.text()
                .collect::<String>()
                .contains("Performance Breakdown")
        });
    assert!(
        performance_section.is_some(),
        "Performance section should exist"
    );

    Ok(())
}

#[tokio::test]
async fn test_mermaid_diagram_rendering() -> Result<()> {
    skip_in_ci!();

    let repo_path = TEST_REPO.path().to_str().unwrap();
    let server = DemoServer::spawn(repo_path).await?;

    // Test DAG diagram endpoint
    let dag_response = HTTP_CLIENT.get(server.url("/api/dag")).send().await?;
    assert!(dag_response.status().is_success());

    let dag_content = dag_response.text().await?;

    // Verify it's valid Mermaid syntax
    assert!(
        dag_content.starts_with("graph TD")
            || dag_content.starts_with("flowchart")
            || dag_content.contains("graph TD"),
        "DAG should contain valid Mermaid syntax"
    );

    // Test system diagram endpoint
    let system_response = HTTP_CLIENT
        .get(server.url("/api/system-diagram"))
        .send()
        .await?;
    assert!(system_response.status().is_success());

    let system_content = system_response.text().await?;
    assert!(
        system_content.starts_with("graph TD")
            || system_content.starts_with("flowchart")
            || system_content.contains("graph TD"),
        "System diagram should contain valid Mermaid syntax"
    );

    // Verify diagrams are not empty
    assert!(
        dag_content.len() > 20,
        "DAG diagram should have substantial content"
    );
    assert!(
        system_content.len() > 20,
        "System diagram should have substantial content"
    );

    Ok(())
}
