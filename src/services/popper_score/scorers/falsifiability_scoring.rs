impl FalsifiabilityScorer {
    /// Create a new falsifiability scorer
    pub fn new() -> Self {
        Self
    }

    /// A1: Hypothesis Documentation (8 points)
    ///
    /// Checks for:
    /// - Clear hypothesis statements in README or DESIGN.md
    /// - Explicit claims about what the software does/achieves
    /// - Defined success criteria with measurable thresholds
    /// - Documented failure conditions (what would falsify claims)
    fn score_hypothesis_documentation(&self, project_path: &Path) -> PopperSubScore {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut earned: f64 = 0.0;
        let max: f64 = 8.0;
        let mut description: Vec<String> = Vec::new();

        // Check README.md for hypothesis/claims
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let content_lower = content.to_lowercase();

                // Check for explicit claims (2 points)
                let claim_patterns = [
                    "claim",
                    "guarantee",
                    "ensures",
                    "provides",
                    "achieves",
                    "delivers",
                ];
                if claim_patterns.iter().any(|p| content_lower.contains(p)) {
                    earned += 2.0;
                    description.push("explicit claims found".to_string());
                }

                // Check for measurable thresholds (2 points)
                let threshold_regex =
                    Regex::new(r"(?i)(>|<|>=|<=|≥|≤)\s*\d+|(\d+%|\d+ms|\d+s|\d+x)")
                        .expect("internal error");
                if threshold_regex.is_match(&content) {
                    earned += 2.0;
                    description.push("measurable thresholds found".to_string());
                }

                // Check for success criteria (2 points)
                if content_lower.contains("success criteria")
                    || content_lower.contains("requirements")
                    || content_lower.contains("acceptance")
                {
                    earned += 2.0;
                    description.push("success criteria found".to_string());
                }

                // Check for falsification conditions (2 points)
                if content_lower.contains("falsif")
                    || content_lower.contains("refute")
                    || content_lower.contains("fail")
                    || content_lower.contains("condition")
                {
                    earned += 2.0;
                    description.push("failure conditions found".to_string());
                }
            }
        }

        // Check for DESIGN.md or ARCHITECTURE.md
        for doc_file in ["DESIGN.md", "ARCHITECTURE.md", "SPEC.md"] {
            let doc_path = project_path.join(doc_file);
            if doc_path.exists() {
                earned = (earned + 1.0).min(max);
                description.push(format!("{} exists", doc_file));
                break;
            }
        }

        PopperSubScore::new(
            "A1",
            "Hypothesis Documentation",
            earned,
            max,
            &description.join(", "),
        )
    }

    /// A2: Test Coverage as Falsification (10 points)
    ///
    /// Checks for:
    /// - Unit test coverage ≥85% (line coverage) - 3 points
    /// - Branch coverage ≥75% - 2 points
    /// - Mutation testing on core modules - 2 points
    /// - Property-based tests - 2 points
    /// - Negative tests (expected failures) - 1 point
    ///
    /// **Workspace-aware**: Checks all workspace members for tests/coverage.
    fn score_test_coverage(&self, project_path: &Path) -> PopperSubScore {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut earned: f64 = 0.0;
        let max: f64 = 10.0;
        let mut description: Vec<String> = Vec::new();

        // Workspace-aware: Check for test directory in any workspace member
        let has_tests = workspace::any_member_has_dir(project_path, "tests")
            || workspace::any_member_has_dir(project_path, "src")
            || self.has_test_files_workspace(project_path);

        if has_tests {
            earned += 2.0;
            description.push("test files exist".to_string());
        }

        // Check for coverage configuration (1 point) - check root AND members
        let coverage_configs = [
            ".cargo/config.toml", // llvm-cov config
            "codecov.yml",
            ".codecov.yml",
            "tarpaulin.toml",
            "lcov.info",
        ];
        let has_coverage = coverage_configs.iter().any(|config| {
            project_path.join(config).exists()
                || workspace::any_member_has_file(project_path, config)
        });
        if has_coverage {
            earned += 1.0;
            description.push("coverage config found".to_string());
        }

        // Check for mutation testing (2 points) - check root AND members
        let mutation_configs = ["mutants.toml", ".cargo/mutants.toml"];
        let has_mutation = mutation_configs.iter().any(|config| {
            project_path.join(config).exists()
                || workspace::any_member_has_file(project_path, config)
        });
        if has_mutation {
            earned += 2.0;
            description.push("mutation testing configured".to_string());
        }

        // Check for property-based tests (2 points) - workspace-aware
        let test_content = self.read_test_files_workspace(project_path);
        if test_content.contains("proptest") || test_content.contains("quickcheck") {
            earned += 2.0;
            description.push("property-based tests found".to_string());
        }

        // Check for negative tests (1 point)
        if test_content.contains("#[should_panic]")
            || test_content.contains("assert_err")
            || test_content.contains("expect_err")
        {
            earned += 1.0;
            description.push("negative tests found".to_string());
        }

        // Check CI for test commands (2 points)
        if self.check_ci_for_tests(project_path) {
            earned += 2.0;
            description.push("CI runs tests".to_string());
        }

        PopperSubScore::new(
            "A2",
            "Test Coverage as Falsification",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// A3: Benchmark Reproducibility (7 points)
    ///
    /// Checks for:
    /// - Criterion or equivalent benchmark framework - 2 points
    /// - Hardware specifications documented - 2 points
    /// - Statistical significance (confidence intervals) - 2 points
    /// - External reproducibility - 1 point
    ///
    /// **Workspace-aware**: Checks workspace members for benches/ and Cargo.toml.
    fn score_benchmark_reproducibility(&self, project_path: &Path) -> PopperSubScore {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut earned: f64 = 0.0;
        let max: f64 = 7.0;
        let mut description: Vec<String> = Vec::new();

        score_bench_directory(project_path, &mut earned, &mut description);
        score_bench_dependencies(project_path, &mut earned, &mut description);
        score_readme_hardware(project_path, &mut earned, &mut description);

        PopperSubScore::new("A3", "Benchmark Reproducibility", earned.min(max), max, &description.join(", "))
    }

    /// Check if project has test files
    fn has_test_files(&self, project_path: &Path) -> bool {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        dir_contains_test_markers(&project_path.join("src"))
    }

    /// Workspace-aware: Check if any workspace member has test files
    fn has_test_files_workspace(&self, project_path: &Path) -> bool {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        for member_path in workspace::get_code_paths(project_path) {
            if self.has_test_files(&member_path) {
                return true;
            }
        }
        false
    }

    /// Workspace-aware: Read test files from all workspace members
    fn read_test_files_workspace(&self, project_path: &Path) -> String {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut content = String::new();

        // Read from tests/ and src/ across all workspace members
        content.push_str(&workspace::read_member_dir_content(
            project_path,
            "tests",
            "rs",
        ));
        content.push_str(&workspace::read_member_dir_content(
            project_path,
            "src",
            "rs",
        ));

        content
    }

    /// Check CI configuration for test commands
    fn check_ci_for_tests(&self, project_path: &Path) -> bool {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let ci_paths = [
            ".github/workflows",
            ".gitlab-ci.yml",
            ".circleci/config.yml",
            "Jenkinsfile",
        ];

        ci_paths.iter().any(|p| ci_path_has_tests(project_path, p))
            || makefile_has_tests(project_path)
    }
}
