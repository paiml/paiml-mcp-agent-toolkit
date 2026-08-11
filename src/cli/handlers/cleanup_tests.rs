// Tests for cleanup_resources_handler (part 1: unit tests)
// Included by cleanup_resources_handler.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ============================================================================
    // CleanupTarget::parse tests
    // ============================================================================

    #[test]
    fn test_cleanup_target_parse_rust() {
        assert_eq!(CleanupTarget::parse("rust"), Some(CleanupTarget::Rust));
        assert_eq!(CleanupTarget::parse("RUST"), Some(CleanupTarget::Rust));
        assert_eq!(CleanupTarget::parse("Rust"), Some(CleanupTarget::Rust));
    }

    /// `docker` parsed as a valid target but `scan_targets` has no arm for it,
    /// so `--targets docker` reported "Items found: 0" and exited 0 without
    /// looking at anything. This test used to pin that: it asserted the name
    /// parsed and said nothing about a scan ever running.
    #[test]
    fn test_cleanup_target_parse_docker_is_rejected_until_a_scanner_exists() {
        assert_eq!(CleanupTarget::parse("docker"), None);
        assert_eq!(CleanupTarget::parse("DOCKER"), None);
        assert!(!SCANNABLE_TARGETS.contains(&"docker"));
    }

    #[test]
    fn test_cleanup_target_parse_node() {
        assert_eq!(CleanupTarget::parse("node"), Some(CleanupTarget::Node));
        assert_eq!(CleanupTarget::parse("NODE"), Some(CleanupTarget::Node));
    }

    #[test]
    fn test_cleanup_target_parse_git() {
        assert_eq!(CleanupTarget::parse("git"), Some(CleanupTarget::Git));
        assert_eq!(CleanupTarget::parse("GIT"), Some(CleanupTarget::Git));
    }

    #[test]
    fn test_cleanup_target_parse_logs() {
        assert_eq!(CleanupTarget::parse("logs"), Some(CleanupTarget::Logs));
        assert_eq!(CleanupTarget::parse("LOGS"), Some(CleanupTarget::Logs));
    }

    /// Same as `docker`: accepted by `parse`, dispatched by nothing.
    #[test]
    fn test_cleanup_target_parse_caches_is_rejected_until_a_scanner_exists() {
        assert_eq!(CleanupTarget::parse("caches"), None);
        assert_eq!(CleanupTarget::parse("CACHES"), None);
        assert!(!SCANNABLE_TARGETS.contains(&"caches"));
    }

    /// Every advertised target must have a scanner behind it: `scan_targets`
    /// dispatches exactly Rust/Node/Git/Logs, and `all` means those four.
    #[test]
    fn test_every_parseable_target_is_scannable() {
        for name in SCANNABLE_TARGETS {
            assert!(
                CleanupTarget::parse(name).is_some(),
                "{name} is advertised as valid but does not parse"
            );
        }
        for name in ["docker", "caches", "bogus", ""] {
            assert_eq!(
                CleanupTarget::parse(name).is_some(),
                SCANNABLE_TARGETS.contains(&name),
                "{name}: parse and the advertised list disagree"
            );
        }
    }

    #[test]
    fn test_cleanup_target_parse_all() {
        assert_eq!(CleanupTarget::parse("all"), Some(CleanupTarget::All));
        assert_eq!(CleanupTarget::parse("ALL"), Some(CleanupTarget::All));
    }

    #[test]
    fn test_cleanup_target_parse_invalid() {
        assert_eq!(CleanupTarget::parse("invalid"), None);
        assert_eq!(CleanupTarget::parse(""), None);
        assert_eq!(CleanupTarget::parse("foo"), None);
    }

    // ============================================================================
    // is_hidden tests
    // ============================================================================

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new("/foo/.hidden")));
        assert!(!is_hidden(Path::new("/foo/.git"))); // .git is not hidden
        assert!(!is_hidden(Path::new("/foo/bar")));
    }

    #[test]
    fn test_is_hidden_dotfiles() {
        assert!(is_hidden(Path::new(".bashrc")));
        assert!(is_hidden(Path::new("/home/.profile")));
        assert!(is_hidden(Path::new(".env")));
    }

    #[test]
    fn test_is_hidden_normal_files() {
        assert!(!is_hidden(Path::new("foo.txt")));
        assert!(!is_hidden(Path::new("/path/to/file.rs")));
    }

    #[test]
    fn test_is_hidden_empty_path() {
        assert!(!is_hidden(Path::new("")));
    }

    // ============================================================================
    // is_excluded tests
    // ============================================================================

    #[test]
    fn test_is_excluded() {
        let exclude = vec!["node_modules".to_string(), "*.log".to_string()];
        assert!(is_excluded(Path::new("/foo/node_modules"), &exclude));
        assert!(is_excluded(Path::new("/foo/bar.log"), &exclude));
        assert!(!is_excluded(Path::new("/foo/bar"), &exclude));
    }

    #[test]
    fn test_is_excluded_empty_patterns() {
        let exclude: Vec<String> = vec![];
        assert!(!is_excluded(Path::new("/foo/bar"), &exclude));
    }

    #[test]
    fn test_is_excluded_glob_patterns() {
        let exclude = vec!["*.tmp".to_string()];
        assert!(is_excluded(Path::new("/foo/test.tmp"), &exclude));
        // The is_excluded function does simple matching, not full glob
        let exclude2 = vec!["build".to_string()];
        assert!(is_excluded(Path::new("/foo/build/output"), &exclude2));
    }

    #[test]
    fn test_is_excluded_partial_match() {
        let exclude = vec!["target".to_string()];
        assert!(is_excluded(Path::new("/project/target/debug"), &exclude));
        assert!(is_excluded(Path::new("/target/release"), &exclude));
    }

    // ============================================================================
    // calculate_dir_size tests
    // ============================================================================

    #[test]
    fn test_calculate_dir_size_empty() {
        let temp_dir = TempDir::new().unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "hello world").unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert!(size > 0);
        assert_eq!(size, 11); // "hello world" is 11 bytes
    }

    #[test]
    fn test_calculate_dir_size_nested() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("sub");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("file.txt"), "content").unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert_eq!(size, 7); // "content" is 7 bytes
    }

    #[test]
    fn test_calculate_dir_size_nonexistent() {
        let size = calculate_dir_size(Path::new("/nonexistent/path"));
        assert_eq!(size, 0);
    }

    // ============================================================================
    // count_loose_objects tests
    // ============================================================================

    #[test]
    fn test_count_loose_objects_empty() {
        let temp_dir = TempDir::new().unwrap();
        let count = count_loose_objects(temp_dir.path());
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_loose_objects_nonexistent() {
        let count = count_loose_objects(Path::new("/nonexistent/path"));
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_loose_objects_with_hex_dirs() {
        let temp_dir = TempDir::new().unwrap();
        // Create directories that look like git object dirs (2-char hex)
        let hex_dir = temp_dir.path().join("ab");
        std::fs::create_dir(&hex_dir).unwrap();
        std::fs::write(hex_dir.join("cdef1234"), "object content").unwrap();
        let count = count_loose_objects(temp_dir.path());
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_loose_objects_non_hex_dirs() {
        let temp_dir = TempDir::new().unwrap();
        // Create directories that don't look like git object dirs
        let non_hex_dir = temp_dir.path().join("info");
        std::fs::create_dir(&non_hex_dir).unwrap();
        std::fs::write(non_hex_dir.join("packs"), "pack info").unwrap();
        let count = count_loose_objects(temp_dir.path());
        assert_eq!(count, 0); // "info" is not 2-char hex
    }

    // ============================================================================
    // CleanupCandidate tests
    // ============================================================================

    #[test]
    fn test_cleanup_candidate_clone() {
        let candidate = CleanupCandidate {
            path: PathBuf::from("/test/path"),
            size_bytes: 1024,
            category: "rust".to_string(),
            description: "Test candidate".to_string(),
            age_days: 5,
        };
        let cloned = candidate.clone();
        assert_eq!(cloned.path, candidate.path);
        assert_eq!(cloned.size_bytes, candidate.size_bytes);
        assert_eq!(cloned.category, candidate.category);
    }

    #[test]
    fn test_cleanup_candidate_debug() {
        let candidate = CleanupCandidate {
            path: PathBuf::from("/test/path"),
            size_bytes: 1024,
            category: "rust".to_string(),
            description: "Test candidate".to_string(),
            age_days: 5,
        };
        let debug = format!("{:?}", candidate);
        assert!(debug.contains("CleanupCandidate"));
        assert!(debug.contains("/test/path"));
    }

    // ============================================================================
    // CleanupResult tests
    // ============================================================================

    #[test]
    fn test_cleanup_result_default() {
        let result = CleanupResult::default();
        assert!(result.candidates.is_empty());
        assert_eq!(result.total_size_bytes, 0);
        assert_eq!(result.items_found, 0);
        assert_eq!(result.items_cleaned, 0);
        assert_eq!(result.space_freed_bytes, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_cleanup_result_debug() {
        let result = CleanupResult::default();
        let debug = format!("{:?}", result);
        assert!(debug.contains("CleanupResult"));
    }

    // ============================================================================
    // CleanupTarget equality tests
    // ============================================================================

    #[test]
    fn test_cleanup_target_equality() {
        assert_eq!(CleanupTarget::Rust, CleanupTarget::Rust);
        assert_ne!(CleanupTarget::Rust, CleanupTarget::Docker);
    }

    #[test]
    fn test_cleanup_target_clone() {
        let target = CleanupTarget::Node;
        let cloned = target.clone();
        assert_eq!(target, cloned);
    }

    #[test]
    fn test_cleanup_target_debug() {
        let target = CleanupTarget::All;
        let debug = format!("{:?}", target);
        assert!(debug.contains("All"));
    }

    // ============================================================================
    // --project-dir must exist (a typo must not read as "nothing to clean")
    // ============================================================================

    /// A nonexistent --project-dir used to print a full 0-item report and exit
    /// 0, so a typo looked exactly like a clean project — with the same flag
    /// driving the destructive --execute mode.
    #[tokio::test]
    async fn test_nonexistent_project_dir_is_an_error() {
        let missing = std::path::Path::new("/does/not/exist/pmat-cleanup-probe");
        let err = handle_cleanup_resources(
            missing,
            &["rust".to_string()],
            false,
            &[],
            0,
            OutputFormat::Table,
        )
        .await
        .expect_err("a nonexistent --project-dir must not report a clean scan");
        assert!(
            err.to_string().contains("does not exist"),
            "unexpected error: {err}"
        );
    }

    /// A regular file is not a project directory either.
    #[tokio::test]
    async fn test_file_as_project_dir_is_an_error() {
        let dir = TempDir::new().expect("tempdir");
        let file = dir.path().join("not-a-dir.txt");
        std::fs::write(&file, "x").expect("write");
        let err = handle_cleanup_resources(
            &file,
            &["rust".to_string()],
            false,
            &[],
            0,
            OutputFormat::Table,
        )
        .await
        .expect_err("a file is not a directory to clean");
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    /// An existing directory still scans and still succeeds.
    #[tokio::test]
    async fn test_existing_project_dir_still_scans() {
        let dir = TempDir::new().expect("tempdir");
        handle_cleanup_resources(
            dir.path(),
            &["rust".to_string()],
            false,
            &[],
            0,
            OutputFormat::Table,
        )
        .await
        .expect("an existing directory must still be scannable");
    }

    include!("cleanup_tests_part2.rs");
}
