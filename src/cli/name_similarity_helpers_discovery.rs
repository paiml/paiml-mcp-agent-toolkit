/// Discover and filter source files based on configuration
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn discover_source_files(
    project_path: PathBuf,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<(PathBuf, String)>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut discovery_config = FileDiscoveryConfig::default();

    if let Some(exclude_pattern) = exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
    }

    let discovery = ProjectFileDiscovery::new(project_path).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;

    let mut analyzed_files = Vec::new();
    for file_path in discovered_files {
        if let Some(include_pattern) = include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            analyzed_files.push((file_path, content));
        }
    }

    Ok(analyzed_files)
}

/// Extract all identifiers from analyzed files
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn extract_all_identifiers(
    analyzed_files: &[(PathBuf, String)],
    _scope: &SearchScope,
) -> Vec<NameInfo> {
    debug_assert!(!analyzed_files.is_empty(), "analyzed_files must not be empty");
    let mut all_names = Vec::new();
    for (_file_path, content) in analyzed_files {
        let names = super::analysis_utilities::extract_identifiers(content);
        all_names.extend(names);
    }
    all_names
}
