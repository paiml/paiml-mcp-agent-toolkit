/// Extracts patterns from AST
pub struct PatternExtractor {
    config: EntropyConfig,
}

impl PatternExtractor {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Extract patterns from every source file under `project_path`.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn extract_patterns(&self, project_path: &Path) -> Result<PatternCollection> {
        if !project_path.exists() {
            anyhow::bail!(
                "entropy analysis path does not exist: {}",
                project_path.display()
            );
        }

        // Files arrive in path order (ProjectContext is a BTreeMap), so the walk
        // and everything derived from it is reproducible.
        let context = self.get_project_context(project_path).await?;

        let mut collection = PatternCollection::new();

        for (file_path, ast_data) in &context.files {
            if self.should_process_file(file_path) {
                self.extract_file_patterns(file_path, ast_data, &mut collection)?;
                collection.total_files += 1;
                // Measured, not estimated: the number of source lines we actually
                // read. Reporting 0 here for a populated crate (defect #650) made
                // the metrics block indistinguishable from an empty directory.
                collection.total_loc += Self::count_source_lines(ast_data);
            }
        }

        // Post-process to calculate variations
        self.calculate_pattern_variations(&mut collection);

        Ok(collection)
    }

    /// Count non-blank source lines in a file's contents.
    fn count_source_lines(content: &str) -> usize {
        content.lines().filter(|l| !l.trim().is_empty()).count()
    }

    /// Read the source files to analyze.
    ///
    /// DETERMINISM: this used to shell out to `pmat context <path> --format json`
    /// and only scan the directory when that call failed. That made the analyzed
    /// file set depend on whichever `pmat` happened to be on `$PATH` (a stale or
    /// differently-configured binary yields a different file set, and a context
    /// payload without `files[].content` silently yielded zero files). The
    /// directory scan is now the only path: same directory in, same file set out.
    async fn get_project_context(&self, project_path: &Path) -> Result<ProjectContext> {
        self.scan_source_files(project_path).await
    }

    /// Walk `project_path` and read every source file we know how to analyze.
    ///
    /// DISCOVERY POLICY: the walk used to be a raw `walkdir::WalkDir` filtered
    /// only by this analyzer's glob exclude list, so it descended into hidden
    /// and gitignored trees. On this repo that meant `.claude/worktrees`:
    /// entropy reported "Files Analyzed: 128656" where `analyze complexity`
    /// reported 4260, with 6639 of 6778 referenced paths living under
    /// `/.claude/worktrees/`. The walk now applies the same policy as the shared
    /// `ProjectFileDiscovery` — .gitignore/.ignore/.pmatignore honoured, hidden
    /// directories skipped, build artifacts skipped. It is spelled out here
    /// rather than delegated because `ProjectFileDiscovery`'s extension list has
    /// no `.ruchy`/`.rh`, which entropy analyzes.
    async fn scan_source_files(&self, project_path: &Path) -> Result<ProjectContext> {
        use std::fs;

        // BTreeMap: the walker yields entries in readdir order, which is not
        // stable across machines or runs; the map re-imposes path order.
        let mut files = BTreeMap::new();

        let walker = ignore::WalkBuilder::new(project_path)
            .standard_filters(true)
            .hidden(true)
            .parents(true)
            .follow_links(false)
            .add_custom_ignore_filename(".pmatignore")
            .add_custom_ignore_filename(".paimlignore")
            .build();

        // Source files in a language this analyzer has no pattern set for. They
        // were dropped in silence: `analyze entropy` on a TypeScript/Python tree
        // printed "Files Analyzed: 0" — byte-identical to the output for an
        // EMPTY directory — so "we cannot analyze this language" and "there is
        // nothing here" were indistinguishable. Counted so the caller can say
        // which one happened.
        let mut skipped_unsupported = 0usize;

        for entry in walker.filter_map(std::result::Result::ok) {
            let path = entry.path();

            // Process Rust and Ruchy files
            if let Some(extension) = path.extension() {
                if (extension == "rs" || extension == "ruchy" || extension == "rh")
                    && self.should_process_file(path)
                {
                    match fs::read_to_string(path) {
                        Ok(content) => {
                            files.insert(path.to_path_buf(), content);
                        }
                        Err(_) => continue, // Skip files we can't read
                    }
                } else if Self::is_unsupported_source_extension(extension)
                    && self.should_process_file(path)
                {
                    skipped_unsupported += 1;
                }
            }
        }

        // Nothing analyzable, but there IS source here. Reporting a block of
        // zeros would be a clean bill of health for code that was never read —
        // and `quality-gate --checks entropy` would pass on it.
        if files.is_empty() && skipped_unsupported > 0 {
            anyhow::bail!(
                "entropy analysis found no analyzable source under {}: it reads Rust and Ruchy \
                 (.rs/.ruchy/.rh), and the {skipped_unsupported} source file(s) there are in \
                 other languages. No entropy was measured.",
                project_path.display()
            );
        }

        Ok(ProjectContext { files })
    }

    /// Extensions this analyzer recognises as source but has no pattern set for.
    ///
    /// Deliberately a fixed list rather than "anything that is not .rs": a
    /// README or a lockfile is not source, and must not turn a Rust-only project
    /// into an error.
    fn is_unsupported_source_extension(extension: &std::ffi::OsStr) -> bool {
        let Some(ext) = extension.to_str() else {
            return false;
        };
        matches!(
            ext,
            "ts" | "tsx"
                | "js"
                | "jsx"
                | "mjs"
                | "cjs"
                | "py"
                | "pyi"
                | "go"
                | "java"
                | "kt"
                | "kts"
                | "scala"
                | "c"
                | "h"
                | "cc"
                | "cpp"
                | "cxx"
                | "hpp"
                | "cs"
                | "rb"
                | "php"
                | "swift"
                | "dart"
                | "lua"
                | "ex"
                | "exs"
                | "zig"
        )
    }

    /// Check if file should be processed
    fn should_process_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        !self.config.exclude_paths.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
        })
    }

    /// Extract patterns from a single file's AST
    fn extract_file_patterns(
        &self,
        file_path: &Path,
        ast_data: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        // Extract patterns using regex-based AST pattern matching
        // Language-specific extraction based on file extension

        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            match extension {
                "ruchy" | "rh" => {
                    // Ruchy-specific pattern extraction
                    self.extract_ruchy_actor_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_pipeline_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_message_passing_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_error_handling_patterns(file_path, ast_data, collection)?;
                    self.extract_ruchy_pattern_matching_patterns(file_path, ast_data, collection)?;
                }
                "rs" => {
                    // Standard Rust pattern extraction
                    self.extract_error_handling_patterns(file_path, ast_data, collection)?;
                    self.extract_data_validation_patterns(file_path, ast_data, collection)?;
                    self.extract_resource_management_patterns(file_path, ast_data, collection)?;
                    self.extract_control_flow_patterns(file_path, ast_data, collection)?;
                    self.extract_data_transformation_patterns(file_path, ast_data, collection)?;
                    self.extract_api_call_patterns(file_path, ast_data, collection)?;
                }
                _ => {
                    // Generic pattern extraction for other languages
                    self.extract_control_flow_patterns(file_path, ast_data, collection)?;
                    self.extract_data_transformation_patterns(file_path, ast_data, collection)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod discovery_scope_tests {
    //! The entropy walk must not descend into hidden tool caches or gitignored
    //! trees: with a raw WalkDir it reported 128,656 files analyzed on a repo
    //! where `analyze complexity` reported 4,260, almost all of the extra paths
    //! coming from `.claude/worktrees`.
    use super::*;

    #[tokio::test]
    async fn scan_skips_hidden_tool_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join(".claude/worktrees/wt/src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
        std::fs::write(
            root.join(".claude/worktrees/wt/src/lib.rs"),
            "pub fn b() {}\n",
        )
        .unwrap();

        let extractor = PatternExtractor::new(EntropyConfig::default());
        let context = extractor.scan_source_files(root).await.expect("scan");

        assert_eq!(
            context.files.len(),
            1,
            "only the project's own source counts: {:?}",
            context.files.keys().collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn scan_skips_gitignored_trees() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("generated")).unwrap();
        // `.git` must exist for gitignore rules to apply, exactly as in a checkout.
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(root.join(".gitignore"), "generated/\n").unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
        std::fs::write(root.join("generated/g.rs"), "pub fn g() {}\n").unwrap();

        let extractor = PatternExtractor::new(EntropyConfig::default());
        let context = extractor.scan_source_files(root).await.expect("scan");

        assert_eq!(
            context.files.len(),
            1,
            "gitignored output is not source: {:?}",
            context.files.keys().collect::<Vec<_>>()
        );
    }

    /// Round-5 dogfood: `analyze entropy -p <ts+py dir>` printed
    /// "Files Analyzed: 0 / Source Lines Analyzed: 0 / Pattern Diversity: not
    /// measured" and exited 0 — byte-for-byte what an EMPTY directory produces.
    /// The walk only reads .rs/.ruchy/.rh; every other source file was dropped
    /// without a word, so a clean report was indistinguishable from "we never
    /// looked at your code".
    #[tokio::test]
    async fn a_tree_of_unanalyzable_source_is_not_reported_as_a_clean_zero() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(root.join("app.ts"), "export function a() { return 1; }\n").unwrap();
        std::fs::write(root.join("main.py"), "def a():\n    return 1\n").unwrap();

        let extractor = PatternExtractor::new(EntropyConfig::default());
        let err = extractor
            .extract_patterns(root)
            .await
            .expect_err("a directory of unreadable-to-us source is not a measured zero");

        let msg = err.to_string();
        assert!(
            msg.contains("no analyzable source"),
            "the report must say why nothing was analyzed: {msg}"
        );
        assert!(msg.contains('2'), "and how much was skipped: {msg}");
    }

    /// An actually empty directory still reports zero, quietly: there is
    /// nothing to explain.
    #[tokio::test]
    async fn an_empty_directory_still_analyzes_to_zero_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let extractor = PatternExtractor::new(EntropyConfig::default());
        let collection = extractor
            .extract_patterns(dir.path())
            .await
            .expect("an empty directory is not an error");
        assert_eq!(collection.total_files, 0);
    }

    /// A Rust project with docs and configs alongside it is untouched.
    #[tokio::test]
    async fn non_source_companions_do_not_turn_a_rust_project_into_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::write(root.join("lib.rs"), "pub fn a() {}\n").unwrap();
        std::fs::write(root.join("README.md"), "# hi\n").unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();

        let extractor = PatternExtractor::new(EntropyConfig::default());
        let collection = extractor.extract_patterns(root).await.expect("scan");
        assert_eq!(collection.total_files, 1);
    }
}
