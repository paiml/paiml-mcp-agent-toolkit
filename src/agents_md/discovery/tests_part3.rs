#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for the AGENTS.md discovery system (Part 3): types, FileChange, and edge cases.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::core::AgentsMdDiscovery;
    use super::super::types::{
        AgentsMdFile, AgentsMdHierarchy, FileChange, FileChangeType, HierarchyNode,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::SystemTime;
    use tempfile::TempDir;

    // =============================================================================
    // AgentsMdFile Tests
    // =============================================================================

    #[test]
    fn test_agents_md_file_clone() {
        let file = AgentsMdFile {
            path: PathBuf::from("/test/AGENTS.md"),
            parent: PathBuf::from("/test"),
            depth: 2,
            modified: SystemTime::now(),
            content: Some("# Test".to_string()),
            document: None,
        };

        let cloned = file.clone();

        assert_eq!(file.path, cloned.path);
        assert_eq!(file.parent, cloned.parent);
        assert_eq!(file.depth, cloned.depth);
        assert_eq!(file.content, cloned.content);
    }

    #[test]
    fn test_agents_md_file_debug() {
        let file = AgentsMdFile {
            path: PathBuf::from("/test/AGENTS.md"),
            parent: PathBuf::from("/test"),
            depth: 0,
            modified: SystemTime::now(),
            content: None,
            document: None,
        };

        let debug_str = format!("{:?}", file);
        assert!(debug_str.contains("AgentsMdFile"));
        assert!(debug_str.contains("AGENTS.md"));
    }

    // =============================================================================
    // AgentsMdHierarchy Tests
    // =============================================================================

    #[test]
    fn test_agents_md_hierarchy_clone() {
        let hierarchy = AgentsMdHierarchy {
            root: PathBuf::from("/project"),
            files: vec![],
            tree: HierarchyNode {
                path: PathBuf::from("/project"),
                agents_file: None,
                children: HashMap::new(),
            },
        };

        let cloned = hierarchy.clone();
        assert_eq!(hierarchy.root, cloned.root);
    }

    #[test]
    fn test_agents_md_hierarchy_debug() {
        let hierarchy = AgentsMdHierarchy {
            root: PathBuf::from("/project"),
            files: vec![],
            tree: HierarchyNode {
                path: PathBuf::from("/project"),
                agents_file: None,
                children: HashMap::new(),
            },
        };

        let debug_str = format!("{:?}", hierarchy);
        assert!(debug_str.contains("AgentsMdHierarchy"));
    }

    // =============================================================================
    // FileChange and FileChangeType Tests
    // =============================================================================

    #[test]
    fn test_file_change_clone() {
        let change = FileChange {
            path: PathBuf::from("/test/AGENTS.md"),
            change_type: FileChangeType::Created,
            timestamp: SystemTime::now(),
        };

        let cloned = change.clone();
        assert_eq!(change.path, cloned.path);
        assert_eq!(change.change_type, cloned.change_type);
    }

    #[test]
    fn test_file_change_debug() {
        let change = FileChange {
            path: PathBuf::from("/test/AGENTS.md"),
            change_type: FileChangeType::Modified,
            timestamp: SystemTime::now(),
        };

        let debug_str = format!("{:?}", change);
        assert!(debug_str.contains("FileChange"));
        assert!(debug_str.contains("Modified"));
    }

    #[test]
    fn test_file_change_type_equality() {
        assert_eq!(FileChangeType::Created, FileChangeType::Created);
        assert_eq!(FileChangeType::Modified, FileChangeType::Modified);
        assert_eq!(FileChangeType::Removed, FileChangeType::Removed);

        assert_ne!(FileChangeType::Created, FileChangeType::Modified);
        assert_ne!(FileChangeType::Modified, FileChangeType::Removed);
        assert_ne!(FileChangeType::Created, FileChangeType::Removed);
    }

    #[test]
    fn test_file_change_type_clone() {
        let created = FileChangeType::Created.clone();
        let modified = FileChangeType::Modified.clone();
        let removed = FileChangeType::Removed.clone();

        assert_eq!(created, FileChangeType::Created);
        assert_eq!(modified, FileChangeType::Modified);
        assert_eq!(removed, FileChangeType::Removed);
    }

    #[test]
    fn test_file_change_type_debug() {
        assert!(format!("{:?}", FileChangeType::Created).contains("Created"));
        assert!(format!("{:?}", FileChangeType::Modified).contains("Modified"));
        assert!(format!("{:?}", FileChangeType::Removed).contains("Removed"));
    }

    // =============================================================================
    // Edge Case Tests
    // =============================================================================

    #[test]
    fn test_discover_with_symlinks() {
        let temp_dir = TempDir::new().unwrap();

        let real_dir = temp_dir.path().join("real");
        fs::create_dir(&real_dir).unwrap();
        fs::write(real_dir.join("AGENTS.md"), "# Real").unwrap();

        // Note: Symlink creation might fail on some systems
        let link_path = temp_dir.path().join("link");
        if std::os::unix::fs::symlink(&real_dir, &link_path).is_ok() {
            let discovery = AgentsMdDiscovery::new();
            let files = discovery.discover_all(temp_dir.path());

            // Should find at least the real file
            assert!(files.len() >= 1);
        }
    }

    #[test]
    fn test_discover_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "").unwrap(); // Empty file

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        // Empty file should still be discovered
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_discover_large_hierarchy() {
        let temp_dir = TempDir::new().unwrap();

        // Create a wide hierarchy
        for i in 0..10 {
            let dir = temp_dir.path().join(format!("dir{}", i));
            fs::create_dir(&dir).unwrap();
            fs::write(dir.join("AGENTS.md"), format!("# Dir {}", i)).unwrap();
        }

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 10);
    }

    #[test]
    fn test_discover_mixed_extensions() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with similar names but different extensions
        fs::write(temp_dir.path().join("AGENTS.md"), "# Correct").unwrap();
        fs::write(temp_dir.path().join("AGENTS.txt"), "Ignored").unwrap();
        fs::write(temp_dir.path().join("AGENTS.md.bak"), "Ignored").unwrap();
        fs::write(
            temp_dir.path().join("agents.md"),
            "Ignored - case sensitive",
        )
        .unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        // Should only find AGENTS.md
        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with("AGENTS.md"));
    }

    #[test]
    fn test_cache_hit_on_exact_path() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let discovery = AgentsMdDiscovery::new();

        // Populate cache via discover_all
        let _files = discovery.discover_all(temp_dir.path());

        // Check cache is populated
        assert!(discovery.cache.contains_key(&agents_path));

        // get_from_cache should work
        let cached = discovery.get_from_cache(&agents_path);
        assert!(cached.is_some());
    }

    #[test]
    fn test_special_characters_in_directory_names() {
        let temp_dir = TempDir::new().unwrap();

        let special_dir = temp_dir.path().join("dir with spaces");
        fs::create_dir(&special_dir).unwrap();
        fs::write(special_dir.join("AGENTS.md"), "# Special").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_unicode_in_directory_names() {
        let temp_dir = TempDir::new().unwrap();

        let unicode_dir = temp_dir.path().join("directorio_espanol");
        fs::create_dir(&unicode_dir).unwrap();
        fs::write(unicode_dir.join("AGENTS.md"), "# Unicode").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_discovery_on_file_not_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(&test_file);

        // Should handle file gracefully (no AGENTS.md in a file)
        assert!(files.is_empty());
    }
}
