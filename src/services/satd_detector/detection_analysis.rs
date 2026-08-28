// SATD project analysis: project scanning, directory analysis, and result aggregation.

impl SATDDetector {
    /// Analyze project for SATD patterns
    /// Toyota Way: Extract Method - reduced complexity from 25-><=8
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_project(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<SATDAnalysisResult, TemplateError> {
        let (files, tests_dropped) = self.discover_files(root, include_tests).await?;
        let mut analysis_stats = ProjectAnalysisStats::new();
        // The denominator: everything the walk found, tests included, before
        // any skip rule ran. Without it the buckets below are counts with
        // nothing to divide them by (#1035).
        analysis_stats.census = FileCensus::over(files.len() + tests_dropped);
        // Discovery, not the loop below, is where test files are dropped, so
        // the count has to be seeded here or the bucket reads 0 forever.
        analysis_stats
            .census
            .record_discovery_dropped_tests(tests_dropped);

        self.process_project_files(&files, include_tests, &mut analysis_stats)
            .await;
        let avg_age_days = self
            .calculate_project_debt_age(&analysis_stats.all_debts, root)
            .await;

        Ok(self.build_analysis_result(analysis_stats, avg_age_days))
    }

    /// Discover the files an analysis will read, and how many test files that
    /// discovery dropped.
    ///
    /// `find_source_files` drops test files during DISCOVERY. So
    /// `--include-tests` — whose only job is to add them — had nothing left to
    /// add, and pointing satd straight at a `tests/` directory reported 0
    /// violations. When tests are wanted the walk below applies the same
    /// directory exclusions and the same source-file test, minus that drop.
    ///
    /// The second element of the pair is the number of test files dropped, and
    /// it exists because a silent drop is unreportable: `SkipCounts::tests` was
    /// structurally pinned at 0 while the walk was quietly declining to read
    /// every test file in the tree.
    async fn discover_files(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<(Vec<std::path::PathBuf>, usize), TemplateError> {
        // Issue #1050 P9. ONE discovery, both ways. The `include_tests` branch
        // used to call `collect_files_including_tests`, a second walk with a
        // different directory policy, so a flag that only widens the scan could
        // make the denominator go to ZERO — `all 1 source file(s) … were
        // skipped` became `no source files were found` for a file that exists
        // and is a source file in both runs. See
        // `find_source_files_partitioned`.
        let (files, tests) = self.find_source_files_partitioned(root).await?;
        if !include_tests {
            return Ok((files, tests.len()));
        }
        // Nothing was declined for being a test: they are all in `files`.
        let mut all = files;
        all.extend(tests);
        Ok((all, 0))
    }

    /// Toyota Way: Extract Method - process all files in project (complexity <=8)
    async fn process_project_files(
        &self,
        files: &[std::path::PathBuf],
        include_tests: bool,
        stats: &mut ProjectAnalysisStats,
    ) {
        for file_path in files {
            if let Some(reason) = self.skip_reason(file_path, include_tests).await {
                stats.census.record_skip(file_path, reason);
                continue;
            }

            // Increment only on a file that was actually read and scanned; see
            // `process_file_for_debts`.
            if let Some(reason) = self
                .process_single_file(file_path, include_tests, stats)
                .await
            {
                stats.census.record_skip(file_path, reason);
                continue;
            }
            stats.total_files_analyzed += 1;
            stats.census.record_analyzed();
        }
    }

    /// Why this file will not be read, or `None` if it will be.
    ///
    /// Returns the REASON rather than a bare bool so the report can disclose
    /// the scope it measured over. Previously this answered only "skip: yes",
    /// the count went nowhere, and a run that declined to read every candidate
    /// was indistinguishable in the output from a run that read them all and
    /// found nothing.
    async fn skip_reason(&self, file_path: &Path, include_tests: bool) -> Option<SkipReason> {
        // Skip test files if not requested
        if !include_tests && self.is_test_file(file_path) {
            return Some(SkipReason::Test);
        }

        // Files whose every line is suppressed by `should_exclude_file` were
        // read, scanned and thrown away one line at a time, so they still
        // counted towards `total_files_analyzed`. A file nothing can be
        // reported from was not analysed; saying otherwise is what makes
        // "excluded everything" look like "measured clean" (#923).
        if self.should_exclude_file(file_path) {
            return Some(SkipReason::OutOfScope);
        }

        // Skip minified/vendor files
        if self.is_minified_or_vendor_file(file_path) {
            return Some(SkipReason::MinifiedOrVendor);
        }

        // Check file size constraints
        if let Some(reason) = self.size_skip_reason(file_path).await {
            return Some(reason);
        }

        None
    }

    /// Why this file is too big to read, or `None`.
    ///
    /// THE size rule, shared by both walks. This one used to drop anything over
    /// `LARGE_FILE_THRESHOLD` (512,000 bytes) while `skip_reason_for_analysis`
    /// had no size rule at all below 1 MB of *minified* content, so
    /// `quality-gate --checks satd` and `analyze satd` read different
    /// populations of the same tree and reported different numbers for it
    /// (#1035). Both now use [`MAX_FILE_BYTES`].
    ///
    /// The drop no longer announces itself only on stderr, either — the caller
    /// records it into [`FileCensus::oversized`], which survives `--format json`
    /// and `--output FILE`, both of which discard stderr. A file skipped for
    /// size used to leave a clean-looking total behind it.
    async fn size_skip_reason(&self, file_path: &Path) -> Option<SkipReason> {
        let metadata = tokio::fs::metadata(file_path).await.ok()?;
        if metadata.len() > MAX_FILE_BYTES {
            return Some(SkipReason::TooLarge {
                bytes: metadata.len(),
            });
        }
        if metadata.len() > 1_000_000 && self.is_likely_minified_content(file_path).await {
            return Some(SkipReason::MinifiedOrVendor);
        }
        None
    }

    /// Read and scan one file, or return WHY it was not scanned.
    ///
    /// The three early returns below used to be bare `return`s and empty
    /// match arms, after the caller had already counted the file as analysed.
    /// They now surface as a `SkipReason` so the caller can count them where a
    /// reader will see them.
    async fn process_single_file(
        &self,
        file_path: &Path,
        include_tests: bool,
        stats: &mut ProjectAnalysisStats,
    ) -> Option<SkipReason> {
        let content = match tokio::fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(_e) => return Some(SkipReason::Unreadable),
        };

        if content.len() as u64 > MAX_FILE_BYTES {
            return Some(SkipReason::TooLarge {
                bytes: content.len() as u64,
            });
        }

        match self.extract_from_content_with_tests(&content, file_path, include_tests) {
            Ok(debts) => {
                if !debts.is_empty() {
                    stats.files_with_debt += 1;
                }
                stats.all_debts.extend(debts);
                None
            }
            // e.g. a line too long for the scanner. The file was opened and
            // not scanned; that is not the same as scanned and clean.
            Err(_e) => Some(SkipReason::Unreadable),
        }
    }

    /// Toyota Way: Extract Method - calculate debt age (complexity <=3)
    async fn calculate_project_debt_age(&self, debts: &[TechnicalDebt], root: &Path) -> f64 {
        if !debts.is_empty() && root.join(".git").exists() {
            self.calculate_average_debt_age(debts, root)
                .await
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Toyota Way: Extract Method - build analysis result (complexity <=5)
    fn build_analysis_result(
        &self,
        stats: ProjectAnalysisStats,
        avg_age_days: f64,
    ) -> SATDAnalysisResult {
        SATDAnalysisResult {
            items: stats.all_debts.clone(),
            summary: SATDSummary {
                total_items: stats.all_debts.len(),
                by_severity: self.group_debts_by_severity(&stats.all_debts),
                by_category: self.group_debts_by_category(&stats.all_debts),
                files_with_satd: stats.files_with_debt,
                avg_age_days,
            },
            total_files_analyzed: stats.total_files_analyzed,
            files_with_debt: stats.files_with_debt,
            census: stats.census.clone(),
            analysis_timestamp: chrono::Utc::now(),
        }
    }

    /// Toyota Way: Extract Method - group debts by severity (complexity <=3)
    fn group_debts_by_severity(
        &self,
        debts: &[TechnicalDebt],
    ) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::with_capacity(3);
        for debt in debts {
            *map.entry(format!("{:?}", debt.severity)).or_insert(0) += 1;
        }
        map
    }

    /// Toyota Way: Extract Method - group debts by category (complexity <=3)
    fn group_debts_by_category(
        &self,
        debts: &[TechnicalDebt],
    ) -> std::collections::HashMap<String, usize> {
        let mut map = std::collections::HashMap::with_capacity(5);
        for debt in debts {
            *map.entry(format!("{:?}", debt.category)).or_insert(0) += 1;
        }
        map
    }

    /// Analyze debt in a directory recursively (excluding test files by default)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_directory(
        &self,
        root: &Path,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        self.analyze_directory_with_tests(root, false).await
    }

    /// Analyze debt in a directory recursively with test file inclusion control
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_directory_with_tests(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        self.analyze_directory_with_stats(root, include_tests)
            .await
            .map(|(debts, _)| debts)
    }

    /// As [`Self::analyze_directory_with_tests`], but also returns WHAT THE WALK
    /// DECLINED TO READ.
    ///
    /// The plain variant returns only the debts, so a caller cannot tell a tree
    /// with no markers from a tree where almost every file was skipped — the two
    /// arrive as the same empty vec. `nothing_measured` below catches the
    /// extreme case where *nothing at all* was read, but the ordinary case, a
    /// run that read some files and silently skipped a hundred others, had no
    /// disclosure at all. `analyze satd` on a project root skips every
    /// `examples/`, `demo/`, fuzz, vendored and generated file and never said so.
    pub async fn analyze_directory_with_stats(
        &self,
        root: &Path,
        include_tests: bool,
    ) -> Result<(Vec<TechnicalDebt>, FileCensus), TemplateError> {
        let mut all_debts = Vec::new();
        let (files, tests_dropped) = self.discover_files(root, include_tests).await?;
        // The denominator, counted before any rule ran. `files` has already had
        // the test files taken out of it, so they are added back here or the
        // population would silently shrink by the size of `tests/`.
        let mut census = FileCensus::over(files.len() + tests_dropped);
        // Discovery already declined these; the loop below never sees them, so
        // this is the only place the count can be recorded.
        census.record_discovery_dropped_tests(tests_dropped);

        for file_path in files {
            if let Some(reason) = self
                .skip_reason_for_analysis(&file_path, include_tests)
                .await
            {
                census.record_skip(&file_path, reason);
                continue;
            }

            // `analyzed` is incremented only once the file has actually been
            // read and scanned. It used to be incremented BEFORE the read, so
            // an unreadable file both inflated the analysed denominator and
            // contributed a guaranteed-empty finding list.
            match self.process_file_for_debts(&file_path, include_tests).await {
                Ok(debts) => {
                    census.record_analyzed();
                    all_debts.extend(debts);
                }
                Err(reason) => census.record_skip(&file_path, reason),
            }
        }

        if census.analyzed == 0 {
            return Err(Self::nothing_measured(
                root,
                census.discovered,
                include_tests,
            ));
        }

        Ok((all_debts, census))
    }

    /// The refusal returned when a walk analysed nothing.
    ///
    /// #923, second half: this entry point returns `Vec<TechnicalDebt>`, and an
    /// empty vec from it is the *only* thing the CLI sees — so "every candidate
    /// was excluded" and "the code is clean" arrived as the same value and were
    /// rendered as the same sentence, `Found 0 SATD violations in 0 files`,
    /// with exit 0. On the real tree `analyze satd -p <repo>/examples` printed
    /// exactly that over 113 `.rs` files holding 10 marker-leading TODO/FIXME
    /// comments. A gate cannot pass on a measurement that was never taken, so
    /// the absence of a measurement is reported as such, the same way a
    /// nonexistent path already is (`ensure_analysis_path_exists`).
    ///
    /// The sentence itself is NOT written here. It used to be — a second copy
    /// of the wording `analyze defects` uses for the identical event, which is
    /// the shape #923 was about in the first place: one rule, two
    /// implementations, free to drift the moment either is edited. Both copies
    /// were byte-identical, so this is a pure substitution.
    ///
    /// Issue #1050 P9, second half. The reason list and the remedy were fixed
    /// strings, so a run that had ALREADY passed `--include-tests` was told to
    /// pass `--include-tests` — advice that cannot work, beside a skip reason
    /// ("test") that the flag in force had already ruled out. Both halves are
    /// now a function of the flag, so the sentence describes the run that
    /// produced it.
    fn nothing_measured(root: &Path, discovered: usize, include_tests: bool) -> TemplateError {
        let (skipped_because, remedy) = if include_tests {
            (
                "fuzz, vendored, generated, minified or oversized",
                "point the analysis at the project root: --include-tests is already in force, \
                 so test code is not what was excluded here.",
            )
        } else {
            (
                "test, fuzz, vendored, generated, minified or oversized",
                "point the analysis at the project root, or pass --include-tests to measure \
                 test code.",
            )
        };
        TemplateError::ValidationError {
            parameter: "path".to_string(),
            reason: crate::services::defect_detector::unmeasured::refusal(
                "SATD",
                root,
                discovered,
                skipped_because,
                remedy,
            ),
        }
    }

    /// Why this file will not be read on the `analyze_directory` path, or `None`.
    ///
    /// The one difference from `skip_reason` used to be size, and it was a real
    /// disagreement rather than a nuance: this path read anything under 10 MB
    /// while that one dropped anything over 512,000 bytes, so the two commands
    /// measured different populations of the same tree and printed different
    /// finding counts for it. Both now call [`Self::size_skip_reason`] — the
    /// unification is UPWARDS, so no file that was read before is skipped now
    /// (#1035).
    ///
    /// `pub(crate)` because the MCP `analyze_satd` tool needs the SAME answer:
    /// it built its own file list and then reported nothing at all about what
    /// it declined to read, so its payload was a count with no denominator.
    /// A second opinion about "why was this file not read" is how these two
    /// surfaces drifted apart in the first place (#997).
    pub(crate) async fn skip_reason_for_analysis(
        &self,
        file_path: &Path,
        include_tests: bool,
    ) -> Option<SkipReason> {
        // Skip test files unless explicitly requested
        if !include_tests && self.is_test_file(file_path) {
            return Some(SkipReason::Test);
        }

        // See `skip_reason`: an excluded file is not an analysed file.
        if self.should_exclude_file(file_path) {
            return Some(SkipReason::OutOfScope);
        }

        // Skip minified/vendor files
        if self.is_minified_or_vendor_file(file_path) {
            return Some(SkipReason::MinifiedOrVendor);
        }

        // Check file size and minification for large files
        self.size_skip_reason(file_path).await
    }

    /// Read one file and scan it, or say why it could not be scanned.
    ///
    /// Returns `Err(SkipReason)` rather than an empty `Vec` for the three
    /// failure paths below. That empty vec is #1035's root cause in its
    /// original form — `Ok(Vec::new())` on a failure to measure — and it sat
    /// one level BELOW the skip predicate, so every disclosure fix above it
    /// counted these files as analysed-and-clean. A file whose bytes were
    /// never decoded cannot support the claim that it contains no debt.
    async fn process_file_for_debts(
        &self,
        file_path: &Path,
        include_tests: bool,
    ) -> Result<Vec<TechnicalDebt>, SkipReason> {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => self.extract_debts_from_content(&content, file_path, include_tests),
            // Not silent any more: an I/O error or non-UTF-8 content is a file
            // this run did not measure, and it is counted as such.
            Err(_e) => Err(SkipReason::Unreadable),
        }
    }

    fn extract_debts_from_content(
        &self,
        content: &str,
        file_path: &Path,
        include_tests: bool,
    ) -> Result<Vec<TechnicalDebt>, SkipReason> {
        // Validate file size before processing. `size_skip_reason` already
        // declined anything this large from its metadata, so this is a second
        // line of defence for callers that arrive with content in hand.
        if content.len() as u64 > MAX_FILE_BYTES {
            return Err(SkipReason::TooLarge {
                bytes: content.len() as u64,
            });
        }

        self.extract_from_content_with_tests(content, file_path, include_tests)
            .map_err(|_| SkipReason::Unreadable)
    }
}
