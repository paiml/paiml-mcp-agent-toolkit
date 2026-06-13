// File discovery filters: entry filtering, external repo detection, build artifact detection,
// and analyzable file classification
// Included by file_discovery.rs - no `use` imports or `#!` inner attributes allowed

impl ProjectFileDiscovery {
    /// Check if an entry should be included in the results
    fn should_include_entry(
        entry: &DirEntry,
        filter_external: bool,
        _classifier: &FileClassifier,
    ) -> bool {
        // Skip directories
        if entry.file_type().is_none_or(|ft| !ft.is_file()) {
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
                | "node_modules" | ".cargo" | ".rustup" | "book" => return true,
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
                    | "lua"
                    | "lean"
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
}
