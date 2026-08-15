#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for the AGENTS.md discovery system (part 1: config, creation, find_nearest, discover_all, ignore patterns, build hierarchy).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::types::{AgentsMdDiscovery, DiscoveryConfig};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    // =============================================================================
    // DiscoveryConfig Tests
    // =============================================================================

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();

        assert_eq!(config.file_name, "AGENTS.md");
        assert_eq!(config.max_depth, 10);
        assert!(!config.watch_enabled);
        assert_eq!(config.cache_ttl, 300);
        assert_eq!(config.ignore_patterns.len(), 4);
        assert!(config.ignore_patterns.contains(&".git".to_string()));
        assert!(config.ignore_patterns.contains(&"node_modules".to_string()));
        assert!(config.ignore_patterns.contains(&"target".to_string()));
        assert!(config.ignore_patterns.contains(&".venv".to_string()));
    }

    #[test]
    fn test_discovery_config_custom() {
        let config = DiscoveryConfig {
            file_name: "CUSTOM.md".to_string(),
            max_depth: 5,
            watch_enabled: true,
            cache_ttl: 600,
            ignore_patterns: vec!["build".to_string()],
        };

        assert_eq!(config.file_name, "CUSTOM.md");
        assert_eq!(config.max_depth, 5);
        assert!(config.watch_enabled);
        assert_eq!(config.cache_ttl, 600);
        assert_eq!(config.ignore_patterns, vec!["build".to_string()]);
    }

    #[test]
    fn test_discovery_config_clone() {
        let config = DiscoveryConfig::default();
        let cloned = config.clone();

        assert_eq!(config.file_name, cloned.file_name);
        assert_eq!(config.max_depth, cloned.max_depth);
        assert_eq!(config.watch_enabled, cloned.watch_enabled);
        assert_eq!(config.cache_ttl, cloned.cache_ttl);
        assert_eq!(config.ignore_patterns, cloned.ignore_patterns);
    }

    #[test]
    fn test_discovery_config_debug() {
        let config = DiscoveryConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DiscoveryConfig"));
        assert!(debug_str.contains("AGENTS.md"));
    }

    // =============================================================================
    // AgentsMdDiscovery Creation Tests
    // =============================================================================

    #[test]
    fn test_discovery_new() {
        let discovery = AgentsMdDiscovery::new();
        // Verify it uses default config
        assert!(discovery.watcher.is_none());
    }

    #[test]
    fn test_discovery_default() {
        let discovery = AgentsMdDiscovery::default();
        // Should behave same as new()
        assert!(discovery.watcher.is_none());
    }

    #[test]
    fn test_discovery_with_config() {
        let config = DiscoveryConfig {
            file_name: "CUSTOM.md".to_string(),
            ..Default::default()
        };
        let discovery = AgentsMdDiscovery::with_config(config);
        assert!(discovery.watcher.is_none());
    }

    // =============================================================================
    // find_nearest Tests
    // =============================================================================

    #[test]
    fn test_find_nearest_in_same_directory() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let found = discovery.find_nearest(temp_dir.path());

        assert_eq!(found, Some(agents_path));
    }

    #[test]
    fn test_find_nearest_in_parent() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let discovery = AgentsMdDiscovery::new();
        let found = discovery.find_nearest(&subdir);

        assert_eq!(found, Some(agents_path));
    }

    #[test]
    fn test_find_nearest_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let discovery = AgentsMdDiscovery::with_config(DiscoveryConfig {
            max_depth: 1,
            ..Default::default()
        });

        let found = discovery.find_nearest(temp_dir.path());
        assert_eq!(found, None);
    }

    #[test]
    fn test_find_nearest_from_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let found = discovery.find_nearest(&test_file);

        assert_eq!(found, Some(agents_path));
    }

    #[test]
    fn test_find_nearest_multiple_levels_up() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        // Create nested subdirectories
        let level1 = temp_dir.path().join("level1");
        let level2 = level1.join("level2");
        let level3 = level2.join("level3");
        fs::create_dir_all(&level3).unwrap();

        let discovery = AgentsMdDiscovery::new();
        let found = discovery.find_nearest(&level3);

        assert_eq!(found, Some(agents_path));
    }

    #[test]
    fn test_find_nearest_uses_cache() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let discovery = AgentsMdDiscovery::new();

        // First call discovers and caches
        let found1 = discovery.find_nearest(&agents_path);
        assert!(found1.is_some());

        // Cache should now have the entry
        assert!(!discovery.cache.is_empty());

        // Second call should use cache
        let found2 = discovery.find_nearest(&agents_path);
        assert_eq!(found1, found2);
    }

    #[test]
    fn test_find_nearest_custom_file_name() {
        let temp_dir = TempDir::new().unwrap();
        let custom_path = temp_dir.path().join("CUSTOM.md");
        fs::write(&custom_path, "# Custom").unwrap();

        let config = DiscoveryConfig {
            file_name: "CUSTOM.md".to_string(),
            ..Default::default()
        };
        let discovery = AgentsMdDiscovery::with_config(config);
        let found = discovery.find_nearest(temp_dir.path());

        assert_eq!(found, Some(custom_path));
    }

    #[test]
    fn test_find_nearest_nonexistent_path() {
        let discovery = AgentsMdDiscovery::new();
        let found = discovery.find_nearest(Path::new("/nonexistent/path/to/nowhere"));
        assert_eq!(found, None);
    }

    // =============================================================================
    // discover_all Tests
    // =============================================================================

    #[test]
    fn test_discover_all() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple AGENTS.md files
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let sub1 = temp_dir.path().join("sub1");
        fs::create_dir(&sub1).unwrap();
        fs::write(sub1.join("AGENTS.md"), "# Sub1").unwrap();

        let sub2 = temp_dir.path().join("sub2");
        fs::create_dir(&sub2).unwrap();
        fs::write(sub2.join("AGENTS.md"), "# Sub2").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].depth, 0); // Root file should be first
    }

    #[test]
    fn test_discover_all_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert!(files.is_empty());
    }

    #[test]
    fn test_discover_all_sorted_by_depth() {
        let temp_dir = TempDir::new().unwrap();

        // Create files at different depths
        let deep = temp_dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("AGENTS.md"), "# Deep").unwrap();

        let shallow = temp_dir.path().join("shallow");
        fs::create_dir(&shallow).unwrap();
        fs::write(shallow.join("AGENTS.md"), "# Shallow").unwrap();

        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 3);
        // Should be sorted by depth (ascending)
        assert!(files[0].depth <= files[1].depth);
        assert!(files[1].depth <= files[2].depth);
    }

    #[test]
    fn test_discover_all_caches_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let _files = discovery.discover_all(temp_dir.path());

        // Cache should be populated
        assert!(!discovery.cache.is_empty());
    }

    #[test]
    fn test_discover_all_respects_max_depth() {
        let temp_dir = TempDir::new().unwrap();

        // Create files at various depths
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let level1 = temp_dir.path().join("level1");
        fs::create_dir(&level1).unwrap();
        fs::write(level1.join("AGENTS.md"), "# Level 1").unwrap();

        let level2 = level1.join("level2");
        fs::create_dir(&level2).unwrap();
        fs::write(level2.join("AGENTS.md"), "# Level 2").unwrap();

        let config = DiscoveryConfig {
            max_depth: 1,
            ..Default::default()
        };
        let discovery = AgentsMdDiscovery::with_config(config);
        let files = discovery.discover_all(temp_dir.path());

        // Should only find files at depth 0 and 1
        assert!(files.len() <= 2);
        for file in &files {
            assert!(file.depth <= 1);
        }
    }

    // =============================================================================
    // Ignore Patterns Tests
    // =============================================================================

    #[test]
    fn test_ignore_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create AGENTS.md in ignored directory
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("AGENTS.md"), "# Ignored").unwrap();

        // Create AGENTS.md in allowed directory
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
        assert!(!files[0].path.to_string_lossy().contains("node_modules"));
    }

    #[test]
    fn test_ignore_git_directory() {
        let temp_dir = TempDir::new().unwrap();

        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("AGENTS.md"), "# Ignored").unwrap();

        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
        assert!(!files[0].path.to_string_lossy().contains(".git"));
    }

    #[test]
    fn test_ignore_target_directory() {
        let temp_dir = TempDir::new().unwrap();

        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();
        fs::write(target_dir.join("AGENTS.md"), "# Ignored").unwrap();

        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_ignore_venv_directory() {
        let temp_dir = TempDir::new().unwrap();

        let venv_dir = temp_dir.path().join(".venv");
        fs::create_dir(&venv_dir).unwrap();
        fs::write(venv_dir.join("AGENTS.md"), "# Ignored").unwrap();

        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_custom_ignore_patterns() {
        let temp_dir = TempDir::new().unwrap();

        let build_dir = temp_dir.path().join("build");
        fs::create_dir(&build_dir).unwrap();
        fs::write(build_dir.join("AGENTS.md"), "# Should be ignored").unwrap();

        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("AGENTS.md"), "# Should be found").unwrap();

        let config = DiscoveryConfig {
            ignore_patterns: vec!["build".to_string()],
            ..Default::default()
        };
        let discovery = AgentsMdDiscovery::with_config(config);
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
        assert!(files[0].path.to_string_lossy().contains("src"));
    }

    #[test]
    fn test_no_ignore_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create files in directories that would normally be ignored
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("AGENTS.md"), "# NodeModules").unwrap();

        let config = DiscoveryConfig {
            ignore_patterns: vec![],
            ..Default::default()
        };
        let discovery = AgentsMdDiscovery::with_config(config);
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
        assert!(files[0].path.to_string_lossy().contains("node_modules"));
    }

    // =============================================================================
    // Build Hierarchy Tests
    // =============================================================================

    #[test]
    fn test_build_hierarchy() {
        let temp_dir = TempDir::new().unwrap();

        // Create hierarchy
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let sub = temp_dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("AGENTS.md"), "# Sub").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());
        let hierarchy = discovery.build_hierarchy(files);

        assert_eq!(hierarchy.root, temp_dir.path());
        assert_eq!(hierarchy.files.len(), 2);
        assert!(hierarchy.tree.agents_file.is_some());
        assert_eq!(hierarchy.tree.children.len(), 1);
    }

    #[test]
    fn test_build_hierarchy_empty() {
        let discovery = AgentsMdDiscovery::new();
        let hierarchy = discovery.build_hierarchy(vec![]);

        assert_eq!(hierarchy.root, PathBuf::new());
        assert!(hierarchy.files.is_empty());
        assert!(hierarchy.tree.agents_file.is_none());
        assert!(hierarchy.tree.children.is_empty());
    }

    #[test]
    fn test_build_hierarchy_single_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());
        let hierarchy = discovery.build_hierarchy(files);

        assert_eq!(hierarchy.files.len(), 1);
        assert!(hierarchy.tree.agents_file.is_some());
        assert!(hierarchy.tree.children.is_empty());
    }

    #[test]
    fn test_build_hierarchy_deep_nesting() {
        let temp_dir = TempDir::new().unwrap();

        // Create deeply nested structure
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let level1 = temp_dir.path().join("level1");
        fs::create_dir(&level1).unwrap();
        fs::write(level1.join("AGENTS.md"), "# Level1").unwrap();

        let level2 = level1.join("level2");
        fs::create_dir(&level2).unwrap();
        fs::write(level2.join("AGENTS.md"), "# Level2").unwrap();

        let level3 = level2.join("level3");
        fs::create_dir(&level3).unwrap();
        fs::write(level3.join("AGENTS.md"), "# Level3").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());
        let hierarchy = discovery.build_hierarchy(files);

        assert_eq!(hierarchy.files.len(), 4);
        assert!(hierarchy.tree.agents_file.is_some());
        assert_eq!(hierarchy.tree.children.len(), 1);

        // Verify nested structure
        let level1_node = hierarchy.tree.children.get("level1");
        assert!(level1_node.is_some());
        let level1_node = level1_node.unwrap();
        assert!(level1_node.agents_file.is_some());
    }

    #[test]
    fn test_build_hierarchy_sibling_directories() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        // Create sibling directories with AGENTS.md
        for name in ["frontend", "backend", "shared"] {
            let dir = temp_dir.path().join(name);
            fs::create_dir(&dir).unwrap();
            fs::write(dir.join("AGENTS.md"), format!("# {}", name)).unwrap();
        }

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());
        let hierarchy = discovery.build_hierarchy(files);

        assert_eq!(hierarchy.files.len(), 4);
        assert_eq!(hierarchy.tree.children.len(), 3);
    }
}
