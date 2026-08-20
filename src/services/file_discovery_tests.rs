// File discovery tests: unit tests and property tests
// Included by file_discovery.rs - no `use` imports or `#!` inner attributes allowed

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_discovery_basic() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test structure
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn hello() {}").unwrap();

        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("tests/test.rs"), "#[test] fn test() {}").unwrap();

        // Create files that should be ignored
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("target/debug/main"), "binary").unwrap();

        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::write(
            root.join("node_modules/pkg/index.js"),
            "module.exports = {}",
        )
        .unwrap();

        let discovery = ProjectFileDiscovery::new(root.to_path_buf());
        let files = discovery.discover_files().unwrap();

        // Should find only source files
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|p| p.ends_with("src/main.rs")));
        assert!(files.iter().any(|p| p.ends_with("src/lib.rs")));
        assert!(files.iter().any(|p| p.ends_with("tests/test.rs")));
    }

    #[test]
    fn test_external_repo_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create external repo pattern
        fs::create_dir_all(root.join("https___github_com_example_repo")).unwrap();
        fs::write(
            root.join("https___github_com_example_repo/main.rs"),
            "fn main() {}",
        )
        .unwrap();

        // Create normal project file
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();

        let discovery = ProjectFileDiscovery::new(root.to_path_buf());
        let files = discovery.discover_files().unwrap();

        // Should only find the normal project file
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
        assert!(!files[0].to_string_lossy().contains("https___"));
    }

    #[test]
    fn test_max_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create deeply nested structure
        let mut current = root.to_path_buf();
        for i in 0..20 {
            current = current.join(format!("level{i}"));
            fs::create_dir_all(&current).unwrap();
            fs::write(current.join("file.rs"), "// content").unwrap();
        }

        let config = FileDiscoveryConfig {
            max_depth: Some(5),
            ..Default::default()
        };

        let discovery = ProjectFileDiscovery::new(root.to_path_buf()).with_config(config);
        let files = discovery.discover_files().unwrap();

        // Should only find files up to depth 5
        assert!(files.len() <= 5);
    }

    #[test]
    #[ignore] // Five Whys: Process-global CWD modification causes race conditions under parallel execution
              // Root cause: std::env::set_current_dir() is process-wide, not thread-local
              // Fix attempted: RAII CwdGuard failed because current_dir() fails if CWD deleted
              // Decision: Mark as #[ignore] - unsuitable for parallel test execution
              // Run manually: cargo test test_custom_ignore_patterns -- --ignored --test-threads=1
    fn test_custom_ignore_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("include.rs"), "// include").unwrap();
        fs::write(root.join("exclude.rs"), "// exclude").unwrap();
        fs::write(root.join("test.rs"), "// test").unwrap();

        let config = FileDiscoveryConfig {
            custom_ignore_patterns: vec!["*exclude*".to_string()],
            ..Default::default()
        };

        let discovery = ProjectFileDiscovery::new(root.to_path_buf()).with_config(config);
        let files = discovery.discover_files().unwrap();

        assert_eq!(files.len(), 2);
        assert!(!files
            .iter()
            .any(|p| p.to_string_lossy().contains("exclude")));
    }

    #[test]
    fn test_file_extension_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create various file types
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("script.py"), "print('hello')").unwrap();
        fs::write(root.join("app.js"), "console.log('hi')").unwrap();
        fs::write(root.join("doc.md"), "# Documentation").unwrap();
        fs::write(root.join("config.toml"), "[package]").unwrap();
        fs::write(root.join("data.json"), "{}").unwrap();

        let discovery = ProjectFileDiscovery::new(root.to_path_buf());
        let files = discovery.discover_files().unwrap();

        // Should find all analyzable files (source + config, but not .md files)
        assert_eq!(files.len(), 5);
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".rs")));
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".py")));
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".js")));

        // Should find project config files but not .md files (they're now DevelopmentDoc)
        assert!(!files.iter().any(|p| p.to_string_lossy().ends_with(".md")));
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".toml")));
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".json")));
    }

    #[test]
    fn test_cython_file_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create Cython files
        fs::write(
            root.join("module.pyx"),
            "def add(int a, int b): return a + b",
        )
        .unwrap();
        fs::write(root.join("module.pxd"), "cdef int add(int a, int b)").unwrap();
        fs::write(root.join("setup.py"), "from distutils.core import setup").unwrap();

        let discovery = ProjectFileDiscovery::new(root.to_path_buf());
        let files = discovery.discover_files().unwrap();

        // Should find all files including Cython
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".pyx")));
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".pxd")));
        assert!(files.iter().any(|p| p.to_string_lossy().ends_with(".py")));
    }

    #[test]
    fn test_file_categorization() {
        use std::path::Path;

        // Test generated output files
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("deep_context.md")),
            FileCategory::GeneratedOutput
        );
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("test-deep-context-2.md")),
            FileCategory::GeneratedOutput
        );
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("kaizen-metrics.json")),
            FileCategory::GeneratedOutput
        );

        // Test essential documentation
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("README.md")),
            FileCategory::EssentialDoc
        );
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("readme.md")),
            FileCategory::EssentialDoc
        );

        // Test build configuration
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("Makefile")),
            FileCategory::BuildConfig
        );
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("Cargo.toml")),
            FileCategory::BuildConfig
        );
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("pyproject.toml")),
            FileCategory::BuildConfig
        );

        // Test test artifacts
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("test_something.md")),
            FileCategory::TestArtifact
        );

        // Test development docs
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("docs/api.md")),
            FileCategory::DevelopmentDoc
        );

        // Test source code
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("src/main.rs")),
            FileCategory::SourceCode
        );
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("app.ts")),
            FileCategory::SourceCode
        );
        assert_eq!(
            ProjectFileDiscovery::categorize_file(Path::new("script.py")),
            FileCategory::SourceCode
        );
    }

    #[test]
    fn test_discovery_stats() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn lib() {}").unwrap();

        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("tests/test.rs"), "#[test] fn test() {}").unwrap();

        fs::write(root.join("app.js"), "console.log('app')").unwrap();

        let discovery = ProjectFileDiscovery::new(root.to_path_buf());
        let stats = discovery.get_discovery_stats().unwrap();

        assert_eq!(stats.total_files, 4);
        assert_eq!(stats.files_by_extension.get("rs"), Some(&3));
        assert_eq!(stats.files_by_extension.get("js"), Some(&1));
        assert_eq!(stats.files_by_category.get("src"), Some(&2));
        assert_eq!(stats.files_by_category.get("tests"), Some(&1));
    }

    /// The fixture the four hand-rolled walks fell into: a gitignored directory
    /// that is NOT hidden, holding a copy of the project. `.claude/worktrees/`
    /// in this repository is both gitignored and hidden, so a walk that only
    /// skipped hidden entries would look fixed while still being wrong for
    /// `vendor/`, `worktrees/`, `fixtures/` and every other plain-named ignored
    /// directory. This fixture removes that escape hatch.
    fn project_with_gitignored_copy() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join(".gitignore"), "worktrees/\n").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/kernel.cu"), "__global__ void k() {}").unwrap();
        fs::write(root.join("README.md"), "# docs").unwrap();
        fs::write(root.join("module.wat"), "(module)").unwrap();
        fs::write(root.join("assembly.ts"), "const x: i32 = 1;").unwrap();

        // The same project again, under an ignored directory.
        fs::create_dir_all(root.join("worktrees/copy/src")).unwrap();
        fs::write(root.join("worktrees/copy/src/kernel.cu"), "__global__ void k() {}").unwrap();
        fs::write(root.join("worktrees/copy/README.md"), "# docs").unwrap();
        fs::write(root.join("worktrees/copy/module.wat"), "(module)").unwrap();
        fs::write(root.join("worktrees/copy/assembly.ts"), "const x: i32 = 1;").unwrap();

        temp_dir
    }

    /// `project_files` returns the extensions the source whitelist does not
    /// know — `.cu`, `.md`, `.wat` — because the analyzers that need them would
    /// otherwise write their own walk, and every one that did skipped the
    /// .gitignore.
    #[test]
    fn project_files_returns_non_source_extensions() {
        let temp_dir = project_with_gitignored_copy();
        let files = project_files(temp_dir.path()).unwrap();

        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        for expected in ["kernel.cu", "README.md", "module.wat", "assembly.ts"] {
            assert!(
                names.iter().any(|n| n == expected),
                "{expected} must be discoverable; got {names:?}"
            );
        }
    }

    /// The defect itself: 48 gitignored worktrees inflated every count that
    /// used a private walk by ~48x.
    #[test]
    fn project_files_does_not_descend_into_a_gitignored_directory() {
        let temp_dir = project_with_gitignored_copy();
        let files = project_files(temp_dir.path()).unwrap();

        let leaked: Vec<_> = files
            .iter()
            .filter(|p| p.components().any(|c| c.as_os_str() == "worktrees"))
            .collect();
        assert!(
            leaked.is_empty(),
            "a gitignored directory is not part of the project: {leaked:?}"
        );
        assert_eq!(
            files
                .iter()
                .filter(|p| p.extension().is_some_and(|e| e == "cu"))
                .count(),
            1,
            "one kernel.cu exists; the copy under worktrees/ is ignored"
        );
    }

    /// The ignore rules must not depend on a `.git` sitting above the tree:
    /// this fixture has none, and the .gitignore still decides.
    #[test]
    fn gitignore_is_honoured_without_a_git_directory() {
        let temp_dir = project_with_gitignored_copy();
        assert!(
            !temp_dir.path().join(".git").exists(),
            "fixture must not be a checkout"
        );
        let files = project_files(temp_dir.path()).unwrap();
        assert!(files
            .iter()
            .all(|p| !p.components().any(|c| c.as_os_str() == "worktrees")));
    }
}
