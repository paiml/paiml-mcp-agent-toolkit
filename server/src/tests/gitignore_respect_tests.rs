//! Integration tests for .gitignore respect in file discovery

use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_discovery_respects_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create source files
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("lib.rs"), "pub fn lib() {}").unwrap();

    // Create build artifacts
    fs::create_dir(root.join("target")).unwrap();
    fs::create_dir(root.join("target/debug")).unwrap();
    fs::write(root.join("target/debug/main"), "binary").unwrap();

    // Create node_modules
    fs::create_dir(root.join("node_modules")).unwrap();
    fs::write(root.join("node_modules/package.json"), "{}").unwrap();

    // Create .gitignore
    fs::write(root.join(".gitignore"), "target/\nnode_modules/\n*.log\n").unwrap();

    // Create a log file that should be ignored
    fs::write(root.join("debug.log"), "log data").unwrap();

    // Configure discovery with gitignore respect
    let config = FileDiscoveryConfig {
        respect_gitignore: true,
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(root.to_path_buf()).with_config(config);
    let files = discovery.discover_files().unwrap();

    // Should only find source files
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.ends_with("main.rs")));
    assert!(files.iter().any(|f| f.ends_with("lib.rs")));

    // Should not find ignored files
    assert!(!files.iter().any(|f| f.to_string_lossy().contains("target")));
    assert!(!files
        .iter()
        .any(|f| f.to_string_lossy().contains("node_modules")));
    assert!(!files.iter().any(|f| f.ends_with("debug.log")));
}

#[test]
fn test_file_discovery_without_gitignore_respect() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create source files
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();

    // Create build artifacts
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("target/main"), "binary").unwrap();

    // Create .gitignore
    fs::write(root.join(".gitignore"), "target/\n").unwrap();

    // Configure discovery WITHOUT gitignore respect
    let config = FileDiscoveryConfig {
        respect_gitignore: false,
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(root.to_path_buf()).with_config(config);
    let files = discovery.discover_files().unwrap();

    // Should find all files when gitignore is not respected
    // But still should filter build artifacts through is_build_artifact
    assert!(files.iter().any(|f| f.ends_with("main.rs")));
    // Build artifacts are still filtered by is_build_artifact function
    assert!(!files.iter().any(|f| f.to_string_lossy().contains("target")));
}

#[test]
fn test_c_file_discovery_respects_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create C source files
    fs::write(root.join("main.c"), "int main() { return 0; }").unwrap();
    fs::write(root.join("utils.c"), "void util() {}").unwrap();
    fs::write(root.join("utils.h"), "#ifndef UTILS_H").unwrap();

    // Create build directory with artifacts
    fs::create_dir(root.join("build")).unwrap();
    fs::write(root.join("build/main.o"), "object file").unwrap();
    fs::write(root.join("build/utils.o"), "object file").unwrap();
    fs::write(root.join("build/a.out"), "executable").unwrap();

    // Create CMake artifacts
    fs::create_dir(root.join("CMakeFiles")).unwrap();
    fs::write(root.join("CMakeCache.txt"), "cmake cache").unwrap();

    // Create .gitignore
    fs::write(
        root.join(".gitignore"),
        "build/\n*.o\nCMakeFiles/\nCMakeCache.txt\n",
    )
    .unwrap();

    let config = FileDiscoveryConfig {
        respect_gitignore: true,
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(root.to_path_buf()).with_config(config);
    let files = discovery.discover_files().unwrap();

    // Should find C source files
    assert!(files.iter().any(|f| f.ends_with("main.c")));
    assert!(files.iter().any(|f| f.ends_with("utils.c")));
    assert!(files.iter().any(|f| f.ends_with("utils.h")));

    // Should not find build artifacts
    assert!(!files.iter().any(|f| f.ends_with(".o")));
    assert!(!files.iter().any(|f| f.ends_with("a.out")));
    assert!(!files
        .iter()
        .any(|f| f.to_string_lossy().contains("CMakeFiles")));
    assert!(!files.iter().any(|f| f.ends_with("CMakeCache.txt")));
}

#[test]
fn test_custom_paimlignore_file() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create source files
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("test.rs"), "fn test() {}").unwrap();

    // Create generated files
    fs::write(root.join("generated.rs"), "// Generated file").unwrap();

    // Create .paimlignore (custom ignore file)
    fs::write(root.join(".paimlignore"), "generated.rs\ntest.rs\n").unwrap();

    let config = FileDiscoveryConfig {
        respect_gitignore: true,
        ..Default::default()
    };

    let discovery = ProjectFileDiscovery::new(root.to_path_buf()).with_config(config);
    let files = discovery.discover_files().unwrap();

    // Should only find main.rs
    assert_eq!(files.len(), 1);
    assert!(files.iter().any(|f| f.ends_with("main.rs")));
    assert!(!files.iter().any(|f| f.ends_with("generated.rs")));
    assert!(!files.iter().any(|f| f.ends_with("test.rs")));
}
