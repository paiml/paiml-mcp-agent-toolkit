// File collection methods for DependencyGraphBuilder
// Extracted from builder.rs

impl DependencyGraphBuilder {
    /// Collect all source files from workspace
    /// Complexity: 6 (directory traversal)
    fn collect_source_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        debug_assert!(root.exists(), "root must exist: {}", root.display());
        let mut files = Vec::new();
        self.collect_files_recursive(root, &mut files)?;
        Ok(files)
    }

    /// Recursively collect source files
    /// Complexity: 8 (recursive with early exit)
    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip common non-source directories
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !dir_name.starts_with('.') && dir_name != "target" && dir_name != "node_modules"
                {
                    self.collect_files_recursive(&path, files)?;
                }
            } else if Self::is_source_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Check if file is a source file we can analyze
    /// Complexity: 3
    fn is_source_file(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("rs")
                | Some("py")
                | Some("js")
                | Some("jsx")
                | Some("ts")
                | Some("tsx")
                | Some("go")
                | Some("java")
                | Some("c")
                | Some("cpp")
                | Some("cc")
                | Some("h")
                | Some("hpp")
        )
    }
}
