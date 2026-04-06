// Scoring, verification runners, and recommendation methods for FormalVerificationScorer
// Included by formal_verification_scorer.rs — shares parent module scope

impl FormalVerificationScorer {
    /// Run Miri tests and return pass/fail status
    fn run_miri_tests(&self, project_path: &Path) -> ScorerResult<MiriResult> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let output = Command::new("cargo")
            .args(["miri", "test", "--", "--test-threads=1"])
            .current_dir(project_path)
            .output()
            .map_err(|e| ScorerError::CommandError(e.to_string()))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for Miri errors
        let has_ub_errors = stderr.contains("Undefined Behavior")
            || stderr.contains("error: Miri evaluation error");

        // Parse test results
        let passed_tests = parse_test_count(&stdout, "passed");
        let failed_tests = parse_test_count(&stdout, "failed");

        Ok(MiriResult {
            passed: output.status.success() && !has_ub_errors,
            _passed_tests: passed_tests,
            _failed_tests: failed_tests,
            has_ub_errors,
        })
    }

    /// Run Kani verification and return results
    fn run_kani_verification(&self, project_path: &Path) -> ScorerResult<KaniResult> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let output = Command::new("cargo")
            .args(["kani", "--only-codegen"])
            .current_dir(project_path)
            .output()
            .map_err(|e| ScorerError::CommandError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse Kani results
        let verified = stdout.contains("VERIFICATION:- SUCCESSFUL")
            || stdout.contains("Verification succeeded");
        let has_failures =
            stdout.contains("VERIFICATION:- FAILED") || stderr.contains("VERIFICATION FAILED");

        Ok(KaniResult {
            all_verified: verified && !has_failures,
            _has_proofs: true,
        })
    }

    /// Score Miri compliance (3 points)
    fn score_miri(&self, project_path: &Path, mode: ScoringMode, cache: Option<&FileCache>) -> f64 {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let unsafe_count = self.count_unsafe_blocks(project_path, cache);
        if unsafe_count == 0 {
            return MIRI_POINTS; // No unsafe = full credit
        }
        if mode == ScoringMode::Quick || mode == ScoringMode::Fast {
            return MIRI_POINTS * 0.3;
        }
        if !self.is_miri_available() {
            return MIRI_POINTS * 0.5;
        }
        match self.run_miri_tests(project_path) {
            Ok(result) if result.passed => MIRI_POINTS,
            Ok(result) if result.has_ub_errors => 0.0,
            Ok(_) => MIRI_POINTS * 0.5,
            Err(_) => MIRI_POINTS * 0.3,
        }
    }

    /// Score Kani proofs (5 points)
    fn score_kani(&self, project_path: &Path, mode: ScoringMode, cache: Option<&FileCache>) -> f64 {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let kani_proofs = self.count_kani_proofs(project_path, cache);
        if kani_proofs == 0 {
            return 0.0;
        }
        if mode == ScoringMode::Quick || mode == ScoringMode::Fast {
            return KANI_POINTS * 0.4;
        }
        if !self.is_kani_available() {
            return KANI_POINTS * 0.3;
        }
        match self.run_kani_verification(project_path) {
            Ok(result) if result.all_verified => KANI_POINTS,
            Ok(_) => KANI_POINTS * 0.5,
            Err(_) => KANI_POINTS * 0.2,
        }
    }

    /// Score contract-based verification (5 points).
    ///
    /// Scores `#[contract("yaml", equation = "eq")]` annotations —
    /// provable-contracts macros that inject YAML-driven debug_assert
    /// backed by Lean proofs and Kani harnesses.
    ///
    /// Also includes Verus specs if present (decreases, recommends, proof fn).
    fn score_verus(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> f64 {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let contract_macros = self.count_contract_macros(project_path, cache);
        let verus_specs = self.count_verus_specs(project_path, cache);
        let total = contract_macros + verus_specs;

        if total == 0 {
            // Check for contracts/ directory with YAML — partial credit
            let has_contracts = project_path.join("contracts").exists();
            return if has_contracts { VERUS_POINTS * 0.2 } else { 0.0 };
        }

        // Score based on annotation density
        let score = match total {
            1..=3 => 0.4,
            4..=10 => 0.6,
            11..=25 => 0.8,
            _ => 1.0,
        };
        VERUS_POINTS * score
    }

    /// Internal scoring logic with cache support
    fn score_internal(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        let score = self.score_miri(project_path, mode, cache)
            + self.score_kani(project_path, mode, cache)
            + self.score_verus(project_path, mode, cache)
            + self.score_lean(project_path);

        Ok(CategoryScore::new(score.min(MAX_POINTS), self.max_points))
    }

    fn recommend_miri(&self, project_path: &Path, recs: &mut Vec<String>) {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let unsafe_count = self.count_unsafe_blocks(project_path, None);
        if unsafe_count > 0 {
            if !self.is_miri_available() {
                recs.push("Install Miri: rustup +nightly component add miri".into());
            } else {
                recs.push(format!(
                    "Run Miri on {} unsafe blocks: cargo +nightly miri test",
                    unsafe_count
                ));
            }
        }
    }

    fn recommend_kani(&self, project_path: &Path, recs: &mut Vec<String>) {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let unsafe_count = self.count_unsafe_blocks(project_path, None);
        let kani_proofs = self.count_kani_proofs(project_path, None);
        if kani_proofs == 0 && unsafe_count > 0 {
            recs.push(
                "Consider adding Kani proofs for unsafe code: https://model-checking.github.io/kani/"
                    .into(),
            );
        } else if kani_proofs > 0 && !self.is_kani_available() {
            recs.push("Install Kani: cargo install --locked kani-verifier".into());
        }
    }

    fn recommend_verus(&self, project_path: &Path, recs: &mut Vec<String>) {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let contract_macros = self.count_contract_macros(project_path, None);
        let has_contracts_dir = project_path.join("contracts").exists();

        if contract_macros == 0 && has_contracts_dir {
            recs.push(
                "Add #[contract(\"yaml-name\", equation = \"eq\")] to production functions \
                 to enable YAML-driven assertion injection"
                    .into(),
            );
        } else if contract_macros == 0 && !has_contracts_dir {
            let verus_specs = self.count_verus_specs(project_path, None);
            if verus_specs == 0 {
                recs.push(
                    "Consider provable-contracts for formal verification: \
                     add contracts/ YAML and #[contract] macros"
                        .into(),
                );
            }
        }
    }

    fn recommend_lean(&self, project_path: &Path, recs: &mut Vec<String>) {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        if !self.is_lean_project(project_path) {
            return;
        }
        let theorems = self.count_lean_theorems(project_path);
        let sorrys = self.count_lean_sorrys(project_path);
        if sorrys > 0 {
            recs.push(format!(
                "Lean 4 project has {} sorry markers — complete proofs to improve score",
                sorrys
            ));
        }
        if theorems == 0 {
            recs.push("Lean 4 project has no theorems/lemmas — add proven propositions".into());
        }
    }
}
