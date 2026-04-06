// File discovery categorization: get_discovery_stats() and categorize_file() methods
// Included by file_discovery.rs - no `use` imports or `#!` inner attributes allowed

impl ProjectFileDiscovery {
    /// Get statistics about discovered files
    pub fn get_discovery_stats(&self) -> Result<DiscoveryStats> {
        let files = self.discover_files()?;
        let mut stats = DiscoveryStats::default();

        for file in &files {
            stats.total_files += 1;

            if let Some(ext) = file.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                *stats.files_by_extension.entry(ext_str.clone()).or_insert(0) += 1;
            }

            // Categorize by directory
            if let Some(parent) = file.parent() {
                for component in parent.components() {
                    let comp_str = component.as_os_str().to_string_lossy();
                    if matches!(
                        comp_str.as_ref(),
                        "src" | "lib" | "test" | "tests" | "spec" | "specs"
                    ) {
                        *stats
                            .files_by_category
                            .entry(comp_str.to_string())
                            .or_insert(0) += 1;
                        break;
                    }
                }
            }
        }

        stats.discovered_paths = files;
        Ok(stats)
    }

    /// Categorize a file for deep context analysis
    #[must_use]
    pub fn categorize_file(path: &Path) -> FileCategory {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Generated deep context reports - MUST EXCLUDE
        if (file_name.contains("deep_context") || file_name.contains("deep-context"))
            && path.extension() == Some(std::ffi::OsStr::new("md"))
        {
            return FileCategory::GeneratedOutput;
        }

        // Kaizen metrics files - also exclude
        if file_name.contains("kaizen") && path.extension() == Some(std::ffi::OsStr::new("json")) {
            return FileCategory::GeneratedOutput;
        }

        // Test artifacts
        if file_name.starts_with("test_") && path.extension() == Some(std::ffi::OsStr::new("md")) {
            return FileCategory::TestArtifact;
        }

        // Essential documentation
        if file_name.eq_ignore_ascii_case("readme.md") {
            return FileCategory::EssentialDoc;
        }

        // Build configuration
        match file_name.to_lowercase().as_str() {
            "makefile" | "gnumakefile" | "bsdmakefile" => return FileCategory::BuildConfig,
            _ => {}
        }

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if ext_str == "toml" && (file_name == "Cargo.toml" || file_name == "pyproject.toml") {
                return FileCategory::BuildConfig;
            }
        }

        // Development docs in docs/ directory
        if let Some(path_str) = path.to_str() {
            if (path_str.contains("/docs/") || path_str.starts_with("docs/"))
                && path.extension() == Some(std::ffi::OsStr::new("md"))
            {
                return FileCategory::DevelopmentDoc;
            }
        }

        // All other markdown files - should not be analyzed as source code
        if path.extension() == Some(std::ffi::OsStr::new("md")) {
            return FileCategory::DevelopmentDoc;
        }

        // Check if it's source code
        if Self::is_analyzable_file(path) {
            return FileCategory::SourceCode;
        }

        // Default to development doc for other files
        FileCategory::DevelopmentDoc
    }
}
