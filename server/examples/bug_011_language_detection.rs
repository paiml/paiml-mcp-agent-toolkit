//! BUG-011: Language Detection Hang Example
//!
//! This example reproduces BUG-011 where:
//! 1. A C++ project (like Ceph) is incorrectly detected as "python-uv"
//! 2. The discovery phase hangs indefinitely
//!
//! Expected behavior:
//! - Should detect C++ as primary language
//! - Should complete within reasonable timeout (30s)
//! - Should support --language override flag
//!
//! Run with: `cargo run --example bug_011_language_detection`

use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🐛 BUG-011: Language Detection Hang Reproduction\n");

    // Example 1: Reproduce the bug - should detect C++ but detects python-uv
    println!("Example 1: Simulating Ceph-like C++ project detection");
    println!("{}", "=".repeat(60));

    // Create a mock C++ project structure
    let test_dir = create_mock_cpp_project().await?;
    println!("Created mock C++ project at: {:?}", test_dir);

    // Attempt to detect language (THIS IS WHERE THE BUG OCCURS)
    println!("\n🔍 Detecting project language...");

    // This should timeout because the current implementation hangs
    let detection_result = timeout(
        Duration::from_secs(5),
        detect_project_language(&test_dir),
    )
    .await;

    match detection_result {
        Ok(Ok(detection)) => {
            println!("✅ Detected: {} (confidence: {:.1}%)", detection.language, detection.confidence);

            // BUG: This shows wrong language
            if detection.language == "python-uv" {
                println!("❌ BUG REPRODUCED: Detected python-uv instead of C++!");
            }
        }
        Ok(Err(e)) => {
            println!("❌ Detection failed: {}", e);
        }
        Err(_) => {
            println!("❌ BUG REPRODUCED: Detection timed out after 5 seconds!");
            println!("   (Current implementation hangs on discovery phase)");
        }
    }

    // Example 2: What the fix should enable - manual override
    println!("\n\nExample 2: Language override (not yet implemented)");
    println!("{}", "=".repeat(60));
    println!("After fix, you should be able to run:");
    println!("  pmat context --language cpp");
    println!("  pmat context --languages rust,python,typescript");

    // Example 3: Multi-language detection (not yet implemented)
    println!("\n\nExample 3: Multi-language detection (not yet implemented)");
    println!("{}", "=".repeat(60));
    println!("After fix, should detect all languages with >5% file count:");
    println!("  C++: 70% (primary)");
    println!("  Python: 20%");
    println!("  Shell: 10%");

    // Cleanup
    cleanup_mock_project(&test_dir).await?;

    println!("\n🎯 To fix this bug:");
    println!("  1. Implement multi-language detection based on file extensions");
    println!("  2. Weight primary indicators (CMakeLists.txt, Cargo.toml) higher");
    println!("  3. Add timeout to discovery phase");
    println!("  4. Add --language and --languages CLI flags");

    Ok(())
}

/// Detect language using the fixed implementation
async fn detect_project_language(path: &PathBuf) -> Result<LanguageDetection> {
    use pmat::services::enhanced_language_detection::detect_project_language_enhanced;

    // Use the fixed detection function
    Ok(detect_project_language_enhanced(path))
}

// Re-export from the module
use pmat::services::enhanced_language_detection::LanguageDetection;

/// Create a mock C++ project structure similar to Ceph
async fn create_mock_cpp_project() -> Result<PathBuf> {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Create C++ files (70% of files)
    fs::create_dir_all(base_path.join("src"))?;
    for i in 0..70 {
        fs::write(
            base_path.join(format!("src/module_{}.cc", i)),
            format!("// C++ file {}\nint main() {{ return 0; }}", i),
        )?;
    }

    // Create headers
    for i in 0..70 {
        fs::write(
            base_path.join(format!("src/module_{}.h", i)),
            format!("// Header file {}\n#pragma once", i),
        )?;
    }

    // Create CMakeLists.txt (primary indicator for C++)
    fs::write(
        base_path.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(TestProject)\n",
    )?;

    // Create some Python files (20% of files)
    fs::create_dir_all(base_path.join("scripts"))?;
    for i in 0..20 {
        fs::write(
            base_path.join(format!("scripts/helper_{}.py", i)),
            format!("# Python helper {}\nprint('hello')", i),
        )?;
    }

    // Create a pyproject.toml (which might confuse the detector)
    fs::write(
        base_path.join("scripts/pyproject.toml"),
        "[project]\nname = \"helpers\"\n",
    )?;

    // Keep the temp dir alive by leaking it
    // (In real code, we'd use a better approach)
    let leaked_path = base_path.to_path_buf();
    std::mem::forget(temp_dir);

    Ok(leaked_path)
}

async fn cleanup_mock_project(path: &PathBuf) -> Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
