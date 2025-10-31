//! BUG-011: Language Detection Tests (RED Phase)
//!
//! These tests define the expected behavior for multi-language detection
//! with confidence scoring, timeout handling, and manual overrides.
//!
//! Current Status: 🔴 RED - These tests will FAIL until implementation complete
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write failing tests that define expected behavior
//! 2. GREEN: Implement minimum code to make tests pass
//! 3. REFACTOR: Clean up implementation
//! 4. COMMIT: Single atomic commit with fix

use anyhow::Result;
use tempfile::TempDir;

// Import the actual implementation
use pmat::services::enhanced_language_detection::{
    detect_all_languages, detect_project_language_enhanced,
    detect_project_language_with_timeout, override_language_detection,
    override_multiple_languages, LanguageDetection, LanguageInfo, MultiLanguageDetection,
};

// =============================================================================
// RED TEST 1: C++ Project Detection
// =============================================================================

#[test]
#[ignore = "BUG-011: RED test - will fail until language detection fixed"]
fn test_cpp_project_detected_correctly() {
    // Arrange: Create mock C++ project similar to Ceph
    let project = create_mock_cpp_project();

    // Act: Detect language
    let detection = detect_project_language_enhanced(&project.path());

    // Assert: Should detect C++ as primary language
    assert_eq!(detection.language, "cpp", "Should detect C++ as primary language");
    assert!(
        detection.confidence > 70.0,
        "Should have high confidence (>70%) for C++ detection, got {}",
        detection.confidence
    );
}

#[test]
#[ignore = "BUG-011: RED test - will fail until confidence calculation fixed"]
fn test_confidence_calculation_cpp_vs_python() {
    // Arrange: C++ project with some Python scripts
    let project = create_mock_cpp_with_python_scripts();

    // Act: Detect language
    let detection = detect_project_language_enhanced(&project.path());

    // Assert: C++ should have higher confidence than Python
    assert_eq!(
        detection.language, "cpp",
        "C++ should be primary language (70% of files)"
    );

    // The confidence should be based on:
    // 1. File count percentage (70% C++ files)
    // 2. Primary indicators (CMakeLists.txt present)
    // Expected: 70 (file %) + 15 (CMakeLists.txt bonus) = 85% confidence
    assert!(
        detection.confidence >= 80.0,
        "C++ confidence should be >= 80% with CMakeLists.txt, got {}",
        detection.confidence
    );
}

// =============================================================================
// RED TEST 2: Multi-Language Detection
// =============================================================================

#[test]
#[ignore = "BUG-011: RED test - multi-language detection not implemented"]
fn test_detect_all_languages_in_polyglot_project() {
    // Arrange: Polyglot project with Rust (45%), Python (30%), TypeScript (25%)
    let project = create_polyglot_project();

    // Act: Detect all languages
    let detection = detect_all_languages(&project.path());

    // Assert: Should detect all three languages
    assert_eq!(detection.languages.len(), 3, "Should detect 3 languages");

    let rust_lang = detection.languages.iter().find(|l| l.language == "rust");
    let python_lang = detection.languages.iter().find(|l| l.language == "python");
    let ts_lang = detection.languages.iter().find(|l| l.language == "typescript");

    assert!(rust_lang.is_some(), "Should detect Rust");
    assert!(python_lang.is_some(), "Should detect Python");
    assert!(ts_lang.is_some(), "Should detect TypeScript");

    // Primary language should be Rust (highest percentage)
    assert_eq!(detection.primary, "rust", "Rust should be primary language");
}

#[test]
#[ignore = "BUG-011: RED test - percentage threshold not implemented"]
fn test_ignore_languages_below_5_percent() {
    // Arrange: Project with Rust (90%), Python (8%), Shell (2%)
    let project = create_project_with_minor_languages();

    // Act: Detect all languages
    let detection = detect_all_languages(&project.path());

    // Assert: Should only include languages >5%
    assert_eq!(detection.languages.len(), 2, "Should only detect Rust and Python (>5%)");

    let has_shell = detection.languages.iter().any(|l| l.language == "bash");
    assert!(!has_shell, "Shell scripts (<5%) should not be included");
}

// =============================================================================
// RED TEST 3: Primary Indicators (Build Files)
// =============================================================================

#[test]
#[ignore = "BUG-011: RED test - primary indicator weighting not implemented"]
fn test_primary_indicators_boost_confidence() {
    // Arrange: Project with equal Rust/Python files but Cargo.toml present
    let project = create_mixed_project_with_cargo_toml();

    // Act: Detect language
    let detection = detect_project_language_enhanced(&project.path());

    // Assert: Rust should win due to Cargo.toml presence
    assert_eq!(
        detection.language, "rust",
        "Rust should be detected due to Cargo.toml"
    );

    // Confidence should be boosted by Cargo.toml presence
    assert!(
        detection.confidence >= 90.0,
        "Cargo.toml should boost confidence to >=90%, got {}",
        detection.confidence
    );
}

#[test]
#[ignore = "BUG-011: RED test - CMakeLists.txt indicator not implemented"]
fn test_cmake_indicates_cpp_project() {
    // Arrange: Project with CMakeLists.txt in root
    let project = create_cpp_project_with_cmake();

    // Act: Detect language
    let detection = detect_project_language_enhanced(&project.path());

    // Assert: Should strongly indicate C++ project
    assert_eq!(detection.language, "cpp");
    assert!(detection.confidence >= 85.0, "CMakeLists.txt should indicate C++ with >=85% confidence");
}

// =============================================================================
// RED TEST 4: Timeout Handling
// =============================================================================

#[test]
#[ignore = "BUG-011: RED test - timeout not implemented"]
fn test_discovery_completes_within_timeout() {
    use std::time::{Duration, Instant};

    // Arrange: Large project structure
    let project = create_large_project();

    // Act: Detect with timeout
    let start = Instant::now();
    let result = detect_project_language_with_timeout(&project.path(), Duration::from_secs(5));
    let elapsed = start.elapsed();

    // Assert: Should complete within 5 seconds
    assert!(elapsed < Duration::from_secs(5), "Detection should complete within timeout");
    assert!(result.is_ok(), "Detection should succeed within timeout");
}

// =============================================================================
// RED TEST 5: Manual Override (CLI Flags)
// =============================================================================

#[test]
#[ignore = "BUG-011: RED test - manual override not implemented"]
fn test_language_override_flag() {
    // Arrange: Python project but user specifies --language cpp
    let project = create_python_project();

    // Act: Override language detection
    let detection = override_language_detection(&project.path(), "cpp");

    // Assert: Should use overridden language
    assert_eq!(
        detection.language, "cpp",
        "Should use manually overridden language"
    );
    assert_eq!(
        detection.confidence, 100.0,
        "Manual override should have 100% confidence"
    );
}

#[test]
#[ignore = "BUG-011: RED test - multi-language override not implemented"]
fn test_languages_override_flag() {
    // Arrange: Complex project
    let project = create_polyglot_project();

    // Act: Override with specific languages
    let languages = vec!["rust".to_string(), "python".to_string()];
    let detection = override_multiple_languages(&project.path(), languages);

    // Assert: Should analyze only specified languages
    assert_eq!(detection.languages.len(), 2);
    assert!(detection.languages.iter().any(|l| l.language == "rust"));
    assert!(detection.languages.iter().any(|l| l.language == "python"));

    // Should NOT include TypeScript even if present
    assert!(!detection.languages.iter().any(|l| l.language == "typescript"));
}

// =============================================================================
// Mock Project Creators
// =============================================================================

fn create_mock_cpp_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create C++ files (70%)
    std::fs::create_dir_all(base.join("src")).unwrap();
    for i in 0..70 {
        std::fs::write(base.join(format!("src/module_{}.cc", i)), "int main() { return 0; }").unwrap();
        std::fs::write(base.join(format!("src/module_{}.h", i)), "#pragma once").unwrap();
    }

    // Create CMakeLists.txt (primary indicator)
    std::fs::write(
        base.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(TestProject)\n",
    ).unwrap();

    // Some Python scripts (20%)
    std::fs::create_dir_all(base.join("scripts")).unwrap();
    for i in 0..20 {
        std::fs::write(base.join(format!("scripts/helper_{}.py", i)), "print('hello')").unwrap();
    }

    temp
}

fn create_mock_cpp_with_python_scripts() -> TempDir {
    create_mock_cpp_project() // Same as above for this test
}

fn create_polyglot_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Rust (45%)
    std::fs::create_dir_all(base.join("src")).unwrap();
    for i in 0..45 {
        std::fs::write(base.join(format!("src/module_{}.rs", i)), "fn main() {}").unwrap();
    }
    std::fs::write(base.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    // Python (30%)
    std::fs::create_dir_all(base.join("scripts")).unwrap();
    for i in 0..30 {
        std::fs::write(base.join(format!("scripts/tool_{}.py", i)), "print('hello')").unwrap();
    }

    // TypeScript (25%)
    std::fs::create_dir_all(base.join("frontend")).unwrap();
    for i in 0..25 {
        std::fs::write(base.join(format!("frontend/component_{}.ts", i)), "export {}").unwrap();
    }
    std::fs::write(base.join("package.json"), "{}").unwrap();

    temp
}

fn create_project_with_minor_languages() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Rust (90%)
    std::fs::create_dir_all(base.join("src")).unwrap();
    for i in 0..90 {
        std::fs::write(base.join(format!("src/module_{}.rs", i)), "fn main() {}").unwrap();
    }

    // Python (8%)
    std::fs::create_dir_all(base.join("scripts")).unwrap();
    for i in 0..8 {
        std::fs::write(base.join(format!("scripts/tool_{}.py", i)), "print('hello')").unwrap();
    }

    // Shell (2%) - should be ignored
    for i in 0..2 {
        std::fs::write(base.join(format!("scripts/build_{}.sh", i)), "#!/bin/bash").unwrap();
    }

    temp
}

fn create_mixed_project_with_cargo_toml() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Equal Rust and Python files (50/50)
    std::fs::create_dir_all(base.join("src")).unwrap();
    std::fs::create_dir_all(base.join("scripts")).unwrap();
    for i in 0..50 {
        std::fs::write(base.join(format!("src/module_{}.rs", i)), "fn main() {}").unwrap();
        std::fs::write(base.join(format!("scripts/tool_{}.py", i)), "print('hello')").unwrap();
    }

    // But Cargo.toml indicates Rust project
    std::fs::write(base.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    temp
}

fn create_cpp_project_with_cmake() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // C++ files
    std::fs::create_dir_all(base.join("src")).unwrap();
    for i in 0..50 {
        std::fs::write(base.join(format!("src/module_{}.cpp", i)), "int main() { return 0; }").unwrap();
    }

    // CMakeLists.txt in root
    std::fs::write(
        base.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(TestProject)\n",
    ).unwrap();

    temp
}

fn create_large_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create many files to test timeout
    std::fs::create_dir_all(base.join("src")).unwrap();
    for i in 0..1000 {
        std::fs::write(base.join(format!("src/file_{}.rs", i)), "fn main() {}").unwrap();
    }

    temp
}

fn create_python_project() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    std::fs::create_dir_all(base.join("src")).unwrap();
    for i in 0..50 {
        std::fs::write(base.join(format!("src/module_{}.py", i)), "print('hello')").unwrap();
    }

    std::fs::write(base.join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();

    temp
}
