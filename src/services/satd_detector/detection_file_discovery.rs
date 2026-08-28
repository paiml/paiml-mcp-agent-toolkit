// SATD file discovery: source file finding, filtering, test/vendor detection, and directory traversal.

impl SATDDetector {
    /// Find all source files in a directory, respecting .gitignore.
    /// Uses `git ls-files` for tracked repos, falls back to recursive walk.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn find_source_files(
        &self,
        root: &Path,
    ) -> Result<Vec<PathBuf>, TemplateError> {
        self.find_source_files_counting_tests(root)
            .await
            .map(|(files, _)| files)
    }

    /// As [`Self::find_source_files`], but also reports HOW MANY test files the
    /// filter dropped.
    ///
    /// Discovery drops test files silently, so `SkipCounts::tests` — the bucket
    /// whose whole job is to disclose them, and whose `note()` renders
    /// "N test (use --include-tests)" — could never be anything but 0: by the
    /// time the counting loop in `analyze_directory_with_stats` ran, the test
    /// files were already gone. `analyze satd` on a tree with a `tests/`
    /// directory reported `files_not_read.tests: 0`, which is the same
    /// count-with-no-denominator #1015 set out to remove, one level down.
    pub(crate) async fn find_source_files_counting_tests(
        &self,
        root: &Path,
    ) -> Result<(Vec<PathBuf>, usize), TemplateError> {
        let (files, tests) = self.find_source_files_partitioned(root).await?;
        Ok((files, tests.len()))
    }

    /// The same walk, keeping the test files instead of counting them.
    ///
    /// Issue #1050 P9. `--include-tests` used to switch to a SECOND discovery
    /// implementation (`collect_files_including_tests`), a filesystem walk that
    /// skips any directory named `book`, `dist`, `build`, `node_modules`,
    /// `target`, `__pycache__` or starting with `.` — while this one asks
    /// `git ls-files`, which does not. So a flag that only WIDENS the scan
    /// turned
    ///
    /// ```text
    ///   all 1 source file(s) under /tmp/bookfix were skipped — test, example,
    ///   fuzz, vendored, generated, minified or oversized …
    /// ```
    ///
    /// into
    ///
    /// ```text
    ///   no source files were found under /tmp/bookfix …
    /// ```
    ///
    /// The file exists and is a source file in both runs; the denominator
    /// vanished because the second walk never descended into `book/`. One
    /// discovery for both, and the per-file skip reasons — which already know
    /// about generated output — do the excluding, where they can be counted
    /// and named.
    pub(crate) async fn find_source_files_partitioned(
        &self,
        root: &Path,
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>), TemplateError> {
        // Try git ls-files first to respect .gitignore
        if let Ok(output) = tokio::process::Command::new("git")
            .args(["ls-files", "--cached", "--others", "--exclude-standard"])
            .current_dir(root)
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let (files, tests): (Vec<PathBuf>, Vec<PathBuf>) = stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| root.join(line))
                    .filter(|path| self.is_source_file(path))
                    .partition(|path| !self.is_test_file(path));
                // Unchanged: an empty *analysable* list still falls through to
                // the walk below, which is what makes a non-git checkout work.
                if !files.is_empty() {
                    return Ok((files, tests));
                }
            }
        }
        // Fallback: recursive walk (non-git projects)
        let mut candidates = Vec::new();
        self.collect_files_recursive(root, &mut candidates).await?;
        let (files, tests): (Vec<PathBuf>, Vec<PathBuf>) = candidates
            .into_iter()
            .partition(|path| !self.is_test_file(path));
        Ok((files, tests))
    }

    /// Recursively collect source files
    fn collect_files_recursive<'a>(
        &'a self,
        dir: &'a Path,
        files: &'a mut Vec<PathBuf>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), TemplateError>> + Send + 'a>>
    {
        Box::pin(async move {
            if !dir.is_dir() {
                return Ok(());
            }

            let mut entries = tokio::fs::read_dir(dir).await.map_err(TemplateError::Io)?;

            while let Some(entry) = entries.next_entry().await.map_err(TemplateError::Io)? {
                let path = entry.path();
                self.process_directory_entry(&path, files).await?;
            }

            Ok(())
        })
    }

    async fn process_directory_entry(
        &self,
        path: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        if path.is_dir() {
            self.process_subdirectory(path, files).await
        } else {
            self.process_file(path, files);
            Ok(())
        }
    }

    async fn process_subdirectory(
        &self,
        path: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), TemplateError> {
        if self.should_skip_directory(path) {
            return Ok(());
        }
        self.collect_files_recursive(path, files).await
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.is_excluded_directory_name(name)
        } else {
            false
        }
    }

    fn is_excluded_directory_name(&self, name: &str) -> bool {
        name.starts_with('.') || self.is_common_build_directory(name)
    }

    fn is_common_build_directory(&self, name: &str) -> bool {
        [
            "target",
            "node_modules",
            "dist",
            "build",
            "__pycache__",
            "book",
        ]
        .contains(&name)
    }

    /// Collect every SOURCE file, test files included.
    ///
    /// The test-file drop used to happen here, inside a predicate named
    /// `is_valid_source_file` (`is_source_file() && !is_test_file()`), which
    /// conflated two questions and made the drop uncountable — see
    /// [`Self::find_source_files_counting_tests`], which now partitions instead
    /// so the files it declines to read can be disclosed rather than vanish.
    fn process_file(&self, path: &Path, files: &mut Vec<PathBuf>) {
        if self.is_source_file(path) {
            files.push(path.to_path_buf());
        }
    }

    /// Check if a file is a supported source file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "java"
                    | "cpp"
                    | "c"
                    | "h"
                    | "hpp"
                    | "cs"
                    | "go"
                    | "php"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "scala"
                    | "clj"
                    | "hs"
                    | "ml"
                    | "elm"
            )
        } else {
            false
        }
    }

    /// Check if a file is a test file
    ///
    /// The directory test runs against the project-relative path (#923): it
    /// used to run against the absolute one, so a checkout that merely sat
    /// under a directory named `tests/` — any CI runner or monorepo with such
    /// a segment — classified every file in the project as test code and
    /// reported the whole tree clean.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn is_test_file(&self, path: &Path) -> bool {
        // Check if path contains test directories
        let path_str = source_scope::project_relative_str(path);
        if source_scope::has_dir_component(&path_str, &["tests", "test"]) {
            return true;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Common test file patterns
            file_name.contains("test")
                || file_name.contains("spec")
                || file_name.ends_with("_test.rs")
                || file_name.ends_with("_test.py")
                || file_name.ends_with("_test.js")
                || file_name.ends_with("_test.ts")
                || file_name.ends_with(".test.js")
                || file_name.ends_with(".test.ts")
                || file_name.ends_with(".spec.js")
                || file_name.ends_with(".spec.ts")
        } else {
            false
        }
    }

    /// Check if file is minified or in vendor directory
    /// Check if file should be excluded from SATD analysis
    ///
    /// Every predicate below reads the path RELATIVE TO ITS OWN PROJECT ROOT
    /// (see [`source_scope`]). They used to read the absolute path, so an
    /// ancestor directory named `examples`, `demo`, `fuzz`, `vendor`, `book`
    /// or `target` — none of which the analysed project chose — excluded the
    /// entire tree and reported "0 violations in 0 files" with exit 0 (#923).
    ///
    /// `examples/` and `demo/` are NOT on this list any more (#1035, Cluster 1).
    /// They were, and the audits behind that issue measured what it cost: on
    /// pforge, 25 `.rs` files — 37% of the repository — were invisible to every
    /// SATD run, confirmed the same way on depyler, forjar and pepita. An
    /// example is shipped code: `cargo build --examples` compiles it, `cargo
    /// publish` ships it, and it is the first thing a new user reads. A marker
    /// there is debt like any other, and the exclusion was not disclosed — the
    /// files did not appear as skipped, they simply were not in the answer.
    ///
    /// The filename rule that went with it (`contains("_demo")`) is gone for the
    /// same reason #925 deleted `contains("/build.rs")`: a substring test on a
    /// name excludes production source that merely reads like support code —
    /// here `src/services/repo_score/scorers/demo_scorer_find_demo_files.rs`.
    ///
    /// What stays excluded is code this project cannot fix in place (vendored,
    /// generated, minified), plus its own build script and manifests and its
    /// fuzz harnesses. Every one of those is COUNTED — see
    /// [`FileCensus`](crate::services::satd_detector::FileCensus) — so a
    /// narrowing shows up as a change in a disclosed denominator rather than as
    /// a silent drop in findings.
    ///
    /// A fifth predicate, `is_satd_analysis_tool`, is gone. It excluded any path
    /// CONTAINING `satd_detector`, `satd_property_tests`, `quality_proxy`, or
    /// both `test` and `satd` — the last raw `contains()` over a whole path in
    /// this function, and so #923 rewritten in place. `project_relative_str`
    /// falls back to the ABSOLUTE path when no VCS marker and no manifest is
    /// found above the file, so that predicate read directory names the analysed
    /// project never chose. Two byte-identical manifest-less trees differing
    /// only in the name of the directory holding them:
    ///
    /// ```text
    ///   <tmp>/plainbed      -> Found 1 SATD violations in 1 files (analysed 1 of 1)
    ///   <tmp>/satd-testbed  -> Error: all 1 source file(s) ... were skipped
    /// ```
    ///
    /// and no flag recovered the second. The `test`-and-`satd` conjunct even
    /// outranked `--include-tests`, re-excluding files that flag had explicitly
    /// been asked to include.
    ///
    /// The self-exemption was redundant as well as harmful. This detector is
    /// content-aware twice over — a marker inside a string literal is not a
    /// comment, and an inline `#[cfg(test)]` block is skipped unless
    /// `--include-tests` — so bypassing the filter over pmat's own
    /// `satd_detector/` and `quality_proxy/` trees yields no production
    /// findings at all. It hid nothing in this repository and only ever fired
    /// on other people's code.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn should_exclude_file(&self, file_path: &Path) -> bool {
        let path_str = source_scope::project_relative_str(file_path);

        self.is_build_or_config_file(&path_str)
            || self.is_fuzz_target(&path_str)
            || self.is_generated_or_vendor(&path_str)
    }

    /// The package's own build script and manifest — the ones that sit beside
    /// `src/`, not every file that happens to be called `build.rs`.
    ///
    /// #925's false negative was exactly this: `src/services/context_impl/
    /// build.rs` holds a production `// TODO: Implement call graph edge
    /// extraction in future iteration`, and `contains("/build.rs")` excluded
    /// the whole file — so the one real marker the issue looked for was
    /// invisible while 57 pieces of prose were reported.
    fn is_build_or_config_file(&self, path_str: &str) -> bool {
        matches!(path_str, "/build.rs" | "/Cargo.toml")
            || path_str.ends_with(".gitignore")
            || path_str.ends_with("README.md")
    }

    fn is_fuzz_target(&self, path_str: &str) -> bool {
        source_scope::has_dir_component(path_str, &["fuzz", "fuzz_targets"])
    }

    fn is_generated_or_vendor(&self, path_str: &str) -> bool {
        source_scope::has_dir_component(path_str, &["target", "vendor", "node_modules", "book"])
            || path_str.contains(".generated")
    }

    /// Whether `path` is vendored or minified.
    ///
    /// The rule itself lives in [`source_scope::is_vendored_or_minified`] and
    /// is shared with the Known-Defects walk, which needed exactly this
    /// predicate once it learned to read JavaScript (#926) — this method used
    /// to BE the rule, and four of its eight name patterns
    /// (`ends_with(".min.js")`, `".min.css"`, `".bundle.js"`,
    /// `".production.js"`) were already dead, subsumed by the `contains(".min.")`
    /// / `contains(".bundle.")` / `contains(".production.")` tests above them.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_minified_or_vendor_file(&self, path: &Path) -> bool {
        source_scope::is_vendored_or_minified(path)
    }

    /// Check if file content suggests it's minified (has very long lines)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn is_likely_minified_content(&self, path: &Path) -> bool {
        use tokio::io::{AsyncBufReadExt, BufReader};

        match tokio::fs::File::open(path).await {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines = reader.lines();

                // Check first few lines for length
                for _ in 0..3 {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            if line.len() > 5000 {
                                return true; // Very long line, likely minified
                            }
                        }
                        Ok(None) => break,
                        Err(_) => return false,
                    }
                }
                false
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod analyser_self_exclusion_regression_tests {
    //! `should_exclude_file` used to carry a fifth predicate,
    //! `is_satd_analysis_tool`, that dropped any path CONTAINING
    //! `satd_detector`, `satd_property_tests`, `quality_proxy`, or both `test`
    //! and `satd`. It was the only raw `contains()` over a whole path left in
    //! that function, and it reintroduced #923 exactly: when no VCS marker and
    //! no manifest sits above a file, `project_relative_str` hands back the
    //! ABSOLUTE path, so the verdict became a property of a directory name the
    //! analysed project never chose. A scratch checkout under a directory
    //! called `satd-testbed` lost its whole measurement while the same bytes
    //! under `plainbed` reported a finding, and no flag recovered it.
    use super::*;

    /// One analysable file whose only content is a marker-leading comment. The
    /// marker is inside a Rust string literal here, which this detector does
    /// not read as a comment, so it does not become debt of its own.
    const MARKED: &str = "// TODO: measure this file wherever it sits\n\
                          pub fn f() -> i32 { 1 }\n";

    const MANIFEST: &str =
        "[package]\nname = \"myproject\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

    /// The runtime proof, as a test: four trees holding byte-identical source,
    /// none of them a repository and none of them carrying a manifest, so the
    /// path every predicate reads is the caller's absolute one. Whether the
    /// directory above the source mentions this analyser must not decide
    /// whether the analyser measures anything.
    #[tokio::test]
    async fn a_tree_whose_own_name_mentions_this_analyser_is_still_measured() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let detector = SATDDetector::new();
        let mut verdicts = Vec::new();

        for dir in ["plainbed", "satd-testbed", "quality_proxy", "satd_detector"] {
            let root = tmp.path().join(dir);
            std::fs::create_dir_all(&root).expect("scratch tree");
            let file = root.join("main.rs");
            std::fs::write(&file, MARKED).expect("source file");

            assert!(
                !detector.should_exclude_file(&file),
                "a directory named {dir:?} above the tree excluded its own source"
            );

            let measured = detector.analyze_directory(&root).await;
            assert!(
                measured.is_ok(),
                "{dir}: the tree holds one analysable file, so a measurement \
                 is possible — every candidate was excluded instead: {measured:?}"
            );
            verdicts.push((dir, measured.map_or(0, |found| found.len())));
        }

        assert!(
            verdicts.iter().all(|(_, n)| *n == 1),
            "the enclosing directory's name changed how much debt exists: {verdicts:?}"
        );
    }

    /// The conjunct that defeated the flag. `--include-tests` exists to widen
    /// the scan to test code; `contains("test") && contains("satd")` narrowed
    /// it back again for any test file whose name mentions this analyser, and
    /// this one is project-relative (`/tests/satd_it.rs`), so a manifest at the
    /// root did not save it either.
    #[tokio::test]
    async fn include_tests_reaches_a_test_file_that_names_this_analyser() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().join("myproject");
        std::fs::create_dir_all(root.join("tests")).expect("tests dir");
        std::fs::write(root.join("Cargo.toml"), MANIFEST).expect("manifest");
        let it = root.join("tests/satd_it.rs");
        std::fs::write(&it, MARKED).expect("test file");

        let detector = SATDDetector::new();
        assert!(
            !detector.should_exclude_file(&it),
            "a test file named after this analyser is still test code, and \
             --include-tests is the flag that decides whether it is read"
        );

        let found = detector
            .analyze_directory_with_tests(&root, true)
            .await
            .expect("--include-tests was passed, so the one test file is in scope");
        assert_eq!(
            found.len(),
            1,
            "--include-tests was overruled by the file's name: {found:?}"
        );
    }

    /// Guard rail: dropping the self-exemption did not widen anything else.
    /// The three surviving predicates still read the path RELATIVE to its own
    /// project root, and still exclude what the project cannot fix in place.
    #[test]
    fn the_surviving_exclusions_are_unchanged() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().join("myproject");
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(root.join("Cargo.toml"), MANIFEST).expect("manifest");
        let detector = SATDDetector::new();

        for excluded in ["fuzz/target.rs", "vendor/dep.rs", "node_modules/dep.js"] {
            let file = root.join(excluded);
            std::fs::create_dir_all(file.parent().expect("parent")).expect("dir");
            std::fs::write(&file, MARKED).expect("file");
            assert!(
                detector.should_exclude_file(&file),
                "{excluded} must stay out of scope"
            );
        }

        let production = root.join("src/satd_detector.rs");
        std::fs::write(&production, MARKED).expect("file");
        assert!(
            !detector.should_exclude_file(&production),
            "a module a project chose to call satd_detector.rs is its own \
             production code, and this analyser has no say over it"
        );
    }
}
