use anyhow::Result;
use ignore::{DirEntry, WalkBuilder, WalkState};
use lazy_static::lazy_static;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::{debug, trace};

use crate::services::file_classifier::FileClassifier;

/// File categorization for deep context analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileCategory {
    SourceCode,      // .rs, .ts, .py - full AST analysis
    EssentialDoc,    // README.md - compress and include
    BuildConfig,     // Makefile, Cargo.toml - compress and include
    GeneratedOutput, // *deep_context*.md - exclude
    DevelopmentDoc,  // docs/*.md - exclude from defect analysis
    TestArtifact,    // test_*.md - exclude
}

lazy_static! {
    /// Patterns for detecting external repository clones
    static ref EXTERNAL_REPO_PATTERNS: RegexSet = RegexSet::new([
        r"https?___",                    // Cloned external repos
        r".*___github_com_.*",           // GitHub clones
        r".*___gitlab_com_.*",           // GitLab clones
        r".*___bitbucket_org_.*",        // Bitbucket clones
        r".*\$\$external\$\$.*",         // Other external markers
        r".*/external_deps/.*",          // External dependencies directory
        r".*/third_party_repos/.*",      // Third party repos
    ]).expect("Invalid regex patterns");

    /// Additional ignore patterns beyond .gitignore
    static ref ADDITIONAL_IGNORE_PATTERNS: Vec<&'static str> = vec![
        "/.cargo/registry/",
        "/.cargo/git/",
        "/.rustup/",
        "/site-packages/",
        "/.venv/",
        "/venv/",
        "/.tox/",
        "/__pycache__/",
        "/.mypy_cache/",
        "/.pytest_cache/",
        "/.gradle/",
        "/gradle/",
        "/.m2/",
        "/.ivy2/",
        "/.sbt/",
        "/.coursier/",
        "/bazel-*/",
        "/.ccache/",
        "/.cache/",
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiscoveryConfig {
    /// Maximum depth to traverse
    pub max_depth: Option<usize>,
    /// Whether to follow symlinks
    pub follow_links: bool,
    /// Whether to respect .gitignore files
    pub respect_gitignore: bool,
    /// Whether to filter external repositories
    pub filter_external_repos: bool,
    /// Additional ignore patterns
    pub custom_ignore_patterns: Vec<String>,
    /// Maximum number of files to discover
    pub max_files: Option<usize>,
}

impl Default for FileDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_depth: Some(15),
            follow_links: false,
            respect_gitignore: true,
            filter_external_repos: true,
            custom_ignore_patterns: vec![],
            max_files: Some(50_000), // Safety limit
        }
    }
}

pub struct ProjectFileDiscovery {
    root: PathBuf,
    config: FileDiscoveryConfig,
    classifier: Arc<FileClassifier>,
}

impl ProjectFileDiscovery {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            config: FileDiscoveryConfig::default(),
            classifier: Arc::new(FileClassifier::default()),
        }
    }

    pub fn with_config(mut self, config: FileDiscoveryConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_classifier(mut self, classifier: Arc<FileClassifier>) -> Self {
        self.classifier = classifier;
        self
    }

    /// Discover all analyzable files in the project
    pub fn discover_files(&self) -> Result<Vec<PathBuf>> {
        let start = std::time::Instant::now();
        debug!("Starting file discovery at: {}", self.root.display());

        let mut builder = self.create_walk_builder()?;

        // Add custom ignore patterns by creating a temporary ignore file
        if !self.config.custom_ignore_patterns.is_empty() {
            let temp_ignore_file =
                std::env::temp_dir().join(format!("paiml_ignore_{}", std::process::id()));
            if let Ok(mut file) = std::fs::File::create(&temp_ignore_file) {
                use std::io::Write;
                for pattern in &self.config.custom_ignore_patterns {
                    let _ = writeln!(file, "{pattern}");
                }
                let _ = file.flush();
                builder.add_ignore(&temp_ignore_file);
                // Note: temporary file will be cleaned up when process exits
            }
        }

        // Add additional ignore patterns by creating a temporary ignore file
        if !ADDITIONAL_IGNORE_PATTERNS.is_empty() {
            let temp_ignore_file2 = std::env::temp_dir()
                .join(format!("paiml_additional_ignore_{}", std::process::id()));
            if let Ok(mut file) = std::fs::File::create(&temp_ignore_file2) {
                use std::io::Write;
                for pattern in ADDITIONAL_IGNORE_PATTERNS.iter() {
                    let _ = writeln!(file, "{pattern}");
                }
                let _ = file.flush();
                builder.add_ignore(&temp_ignore_file2);
                // Note: temporary file will be cleaned up when process exits
            }
        }

        let walker = builder.build_parallel();
        let mut files = Vec::new();
        let max_files = self.config.max_files.unwrap_or(usize::MAX);

        // Use parallel walker for performance
        let (tx, rx) = crossbeam_channel::unbounded();
        let filter_external = self.config.filter_external_repos;

        walker.run(|| {
            let tx = tx.clone();
            let classifier = self.classifier.clone();

            Box::new(move |result| {
                if let Ok(entry) = result {
                    if Self::should_include_entry(&entry, filter_external, &classifier) {
                        let _ = tx.send(entry.into_path());
                    }
                }

                // Stop if we've found enough files
                if tx.len() >= max_files {
                    return WalkState::Quit;
                }

                WalkState::Continue
            })
        });

        drop(tx); // Close sender

        // Collect results
        while let Ok(path) = rx.recv() {
            files.push(path);
            if files.len() >= max_files {
                debug!("Reached maximum file limit: {}", max_files);
                break;
            }
        }

        let elapsed = start.elapsed();
        debug!(
            "File discovery completed in {:?}. Found {} files",
            elapsed,
            files.len()
        );

        // Sort for deterministic output
        files.sort();

        Ok(files)
    }

    /// Create the WalkBuilder with appropriate configuration
    fn create_walk_builder(&self) -> Result<WalkBuilder> {
        let mut builder = WalkBuilder::new(&self.root);

        // Configure ripgrep-style filtering
        builder
            .standard_filters(true) // Enables .gitignore, .ignore, etc.
            .hidden(!self.config.follow_links) // Skip hidden files unless following links
            .parents(true) // Check parent directories for ignore files
            .ignore(self.config.respect_gitignore)
            .git_ignore(self.config.respect_gitignore)
            .git_global(self.config.respect_gitignore)
            .git_exclude(self.config.respect_gitignore)
            .follow_links(self.config.follow_links)
            .max_depth(self.config.max_depth)
            .add_custom_ignore_filename(".paimlignore");

        // Add build artifact filters
        builder.filter_entry(|entry| !Self::is_build_artifact(entry.path()));

        Ok(builder)
    }

    /// Check if an entry should be included in the results
    fn should_include_entry(
        entry: &DirEntry,
        filter_external: bool,
        _classifier: &FileClassifier,
    ) -> bool {
        // Skip directories
        if entry.file_type().map_or(true, |ft| !ft.is_file()) {
            return false;
        }

        let path = entry.path();

        // Skip external repositories if configured
        if filter_external && Self::is_external_repository(path) {
            trace!("Skipping external repository: {}", path.display());
            return false;
        }

        // Check if it's a source file we can analyze
        if !Self::is_analyzable_file(path) {
            return false;
        }

        // Additional classification based on content (if needed)
        // Note: We don't read file content here for performance
        // The actual parsing stage will handle content-based filtering

        true
    }

    /// Check if a path is part of an external repository
    fn is_external_repository(path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        EXTERNAL_REPO_PATTERNS.is_match(&path_str)
    }

    /// Check if a path is a build artifact
    fn is_build_artifact(path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check common build directories
        for component in path.components() {
            let comp_str = component.as_os_str().to_string_lossy();
            match comp_str.as_ref() {
                "target" | "build" | "dist" | "out" | ".next" | "__pycache__" | ".gradle"
                | "node_modules" | ".cargo" | ".rustup" => return true,
                _ => {}
            }
        }

        // Check path patterns
        if path_str.contains("/target/debug/")
            || path_str.contains("/target/release/")
            || path_str.contains("/build/")
            || path_str.contains("/dist/")
            || path_str.contains("/.gradle/")
            || path_str.contains("/bazel-")
        {
            return true;
        }

        false
    }

    /// Check if a file is analyzable based on extension or special name
    /// Apply Kaizen - Include important project files for complete analysis
    fn is_analyzable_file(path: &Path) -> bool {
        // Check for special project files without extensions (Jidoka - build quality in)
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if matches!(
                filename.to_lowercase().as_str(),
                "makefile" | "dockerfile" | "justfile" | "rakefile" | "gemfile" | "podfile"
            ) {
                return true;
            }
        }

        // Check for files with extensions
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_str.as_str(),
                // Programming languages
                "rs" | "js"
                    | "jsx"
                    | "ts"
                    | "tsx"
                    | "py"
                    | "pyi"
                    | "pyx"      // Cython source files
                    | "pxd"      // Cython declaration files
                    | "go"
                    | "java"
                    | "kt"
                    | "scala"
                    | "cpp"
                    | "cc"
                    | "cxx"
                    | "c"
                    | "h"
                    | "hpp"
                    | "cs"
                    | "rb"
                    | "php"
                    | "swift"
                    | "m"
                    | "mm"
                    | "dart"
                    | "vue"
                    | "svelte"
                    // Kaizen improvement - Add important project configuration files  
                    // Note: .md files handled separately in categorize_file
                    | "toml"      // Cargo.toml, pyproject.toml, etc.
                    | "yaml" | "yml"  // GitHub Actions, docker-compose, etc.
                    | "json"      // package.json, tsconfig.json, etc.
                    | "xml"       // pom.xml, build.xml, etc.
                    | "gradle"    // build.gradle
                    | "mk"        // include.mk, common.mk
                    | "cmake"     // CMakeLists.txt equivalent
                    | "sh" | "bash" | "zsh" | "fish"  // Shell scripts
                    | "bat" | "cmd" | "ps1" // Windows scripts
            )
        } else {
            false
        }
    }

    /// Get statistics about discovered files
    pub fn get_discovery_stats(&self) -> Result<DiscoveryStats> {
        let files = self.discover_files()?;
        let mut stats = DiscoveryStats::default();

        for file in &files {
            stats.total_files += 1;

            if let Some(ext) = file.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                *stats
                    .files_by_extension
                    .entry(ext_str.to_string())
                    .or_insert(0) += 1;
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
    pub fn categorize_file(path: &Path) -> FileCategory {
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DiscoveryStats {
    pub total_files: usize,
    pub files_by_extension: std::collections::HashMap<String, usize>,
    pub files_by_category: std::collections::HashMap<String, usize>,
    pub discovered_paths: Vec<PathBuf>,
}

/// External repository filter for precise detection
pub struct ExternalRepoFilter {
    patterns: RegexSet,
}

impl ExternalRepoFilter {
    pub fn new() -> Self {
        Self {
            patterns: EXTERNAL_REPO_PATTERNS.clone(),
        }
    }

    pub fn is_external_dependency(&self, entry: &DirEntry) -> bool {
        let path_str = entry.path().to_string_lossy();
        self.patterns.is_match(&path_str)
    }
}

impl Default for ExternalRepoFilter {
    fn default() -> Self {
        Self::new()
    }
}

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
}
