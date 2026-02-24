// Scoring, verification runners, and recommendation methods for FormalVerificationScorer
// Included by formal_verification_scorer.rs — shares parent module scope

impl FormalVerificationScorer {
    /// Run Miri tests and return pass/fail status
    fn run_miri_tests(&self, project_path: &Path) -> ScorerResult<MiriResult> {
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

    /// Score Verus specifications (5 points)
    fn score_verus(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> f64 {
        let verus_specs = self.count_verus_specs(project_path, cache);
        let has_vstd = self.has_vstd_dependency(project_path);
        if verus_specs == 0 && !has_vstd {
            return 0.0;
        }
        let use_partial =
            mode == ScoringMode::Quick || mode == ScoringMode::Fast || !self.is_verus_available();
        let spec_score = if use_partial {
            Self::verus_partial_score(verus_specs)
        } else {
            Self::verus_full_score(verus_specs)
        };
        VERUS_POINTS * spec_score
    }

    /// Verus scoring when tool is unavailable or in quick/fast mode
    fn verus_partial_score(verus_specs: usize) -> f64 {
        match verus_specs {
            0 => 0.2,
            1..=5 => 0.4,
            6..=20 => 0.6,
            _ => 0.8,
        }
    }

    /// Verus scoring when tool is available and in full mode
    fn verus_full_score(verus_specs: usize) -> f64 {
        match verus_specs {
            0 => 0.3,
            1..=5 => 0.6,
            6..=20 => 0.8,
            _ => 1.0,
        }
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
        let unsafe_count = self.count_unsafe_blocks(project_path, None);
        let verus_specs = self.count_verus_specs(project_path, None);
        let has_vstd = self.has_vstd_dependency(project_path);
        if verus_specs == 0 && !has_vstd && unsafe_count > 0 {
            recs.push(
                "Consider Verus for formal verification of unsafe code: https://verus-lang.github.io/verus/guide/"
                    .into(),
            );
        } else if (verus_specs > 0 || has_vstd) && !self.is_verus_available() {
            recs.push(
                "Install Verus to verify specs: https://github.com/verus-lang/verus#building"
                    .into(),
            );
        } else if verus_specs < 5 && has_vstd {
            recs.push(
                "Add more #[requires], #[ensures] specs to increase verification coverage".into(),
            );
        }
    }

    fn recommend_lean(&self, project_path: &Path, recs: &mut Vec<String>) {
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
