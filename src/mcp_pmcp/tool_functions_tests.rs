// Tests for MCP tool functions
// Extracted for file health compliance (CB-040)

use super::*;

mod coverage_tests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    // ==================== HELPER FUNCTION TESTS ====================

    #[test]
    fn test_select_quality_profile_extreme() {
        let profile = select_quality_profile(Some("extreme"));
        assert_eq!(profile.name, "extreme");
        assert_eq!(profile.thresholds.max_complexity, 5);
        assert!(profile.thresholds.zero_satd);
    }

    #[test]
    fn test_select_quality_profile_standard() {
        let profile = select_quality_profile(Some("standard"));
        assert_eq!(profile.name, "standard");
        assert_eq!(profile.thresholds.max_complexity, 10);
    }

    #[test]
    fn test_select_quality_profile_relaxed() {
        let profile = select_quality_profile(Some("relaxed"));
        assert_eq!(profile.name, "relaxed");
        assert_eq!(profile.thresholds.max_complexity, 20);
        assert!(!profile.thresholds.zero_satd);
    }

    #[test]
    fn test_select_quality_profile_default() {
        let profile = select_quality_profile(None);
        assert_eq!(profile.name, "standard");
    }

    #[test]
    fn test_select_quality_profile_unknown() {
        let profile = select_quality_profile(Some("unknown_profile"));
        assert_eq!(profile.name, "standard"); // Falls back to standard
    }

    #[test]
    fn test_parse_code_type_function() {
        let code_type = parse_code_type(Some("function"));
        assert!(matches!(code_type, CodeType::Function));
    }

    #[test]
    fn test_parse_code_type_module() {
        let code_type = parse_code_type(Some("module"));
        assert!(matches!(code_type, CodeType::Module));
    }

    #[test]
    fn test_parse_code_type_service() {
        let code_type = parse_code_type(Some("service"));
        assert!(matches!(code_type, CodeType::Service));
    }

    #[test]
    fn test_parse_code_type_test() {
        let code_type = parse_code_type(Some("test"));
        assert!(matches!(code_type, CodeType::Test));
    }

    #[test]
    fn test_parse_code_type_default() {
        let code_type = parse_code_type(None);
        assert!(matches!(code_type, CodeType::Function));
    }

    #[test]
    fn test_parse_code_type_unknown() {
        let code_type = parse_code_type(Some("unknown_type"));
        assert!(matches!(code_type, CodeType::Function)); // Falls back to Function
    }

    // ==================== INPUT VALIDATION TESTS ====================

    #[tokio::test]
    async fn test_analyze_complexity_empty_paths() {
        let result = analyze_complexity(&[], None, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_analyze_satd_empty_paths() {
        let result = analyze_satd(&[], false, false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_analyze_dead_code_empty_paths() {
        let result = analyze_dead_code(&[], false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspots_empty_paths() {
        let result = analyze_lint_hotspots(&[], None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_analyze_churn_empty_paths() {
        let result = analyze_churn(&[], None, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_analyze_coupling_empty_paths() {
        let result = analyze_coupling(&[], None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_check_quality_gates_empty_paths() {
        let result = check_quality_gates(&[], false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_quality_gate_summary_empty_paths() {
        let result = quality_gate_summary(&[]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_quality_gate_baseline_empty_paths() {
        let result = quality_gate_baseline(&[], None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_quality_gate_compare_empty_paths() {
        let temp_file = TempDir::new().unwrap();
        let baseline_path = temp_file.path().join("baseline.json");
        fs::write(&baseline_path, "{}").unwrap();

        let result = quality_gate_compare(&baseline_path, &[]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_check_quality_gate_file_nonexistent() {
        let path = Path::new("/tmp/nonexistent_file_12345.rs");
        let result = check_quality_gate_file(path, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    // ============ QUALITY GATE VERDICT DIRECTION TESTS (v3.18.2) ============
    // Regression tests for the inverted `passed` verdict: Grade's derived Ord
    // makes BETTER grades compare as SMALLER (APlus < ... < F), so the old
    // `grade >= threshold_grade` was true only for grades AT OR WORSE than
    // the threshold. Reproduced as: paths=[src/utils] -> passed:false despite
    // score 88.99 / grade AMinus and zero violations.

    #[test]
    fn test_quality_gate_grade_threshold_direction() {
        use crate::tdg::Grade;
        // An A- grade must pass a B+ threshold (better than required)
        assert!(Grade::AMinus.meets_threshold(Grade::BPlus));
        // A C grade must fail a B+ threshold (worse than required)
        assert!(!Grade::C.meets_threshold(Grade::BPlus));
        // Equal grade meets the threshold
        assert!(Grade::BPlus.meets_threshold(Grade::BPlus));
        // The thresholds actually used by the quality gate tools:
        // strict = B, standard = D
        assert!(Grade::AMinus.meets_threshold(Grade::B));
        assert!(!Grade::C.meets_threshold(Grade::B));
        assert!(Grade::C.meets_threshold(Grade::D));
        assert!(!Grade::F.meets_threshold(Grade::D));
    }

    /// Helper: a clean, documented Rust source that grades well.
    fn write_clean_rust_file(dir: &Path) -> PathBuf {
        let file_path = dir.join("clean.rs");
        fs::write(
            &file_path,
            "/// Adds two numbers.\n\
             pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n\
             /// Subtracts two numbers.\n\
             pub fn sub(a: i32, b: i32) -> i32 {\n    a - b\n}\n",
        )
        .unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_check_quality_gate_file_verdict_not_inverted() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = write_clean_rust_file(temp_dir.path());

        // Standard mode: threshold is score >= 50 and grade at least D.
        let result = check_quality_gate_file(&file_path, false).await.unwrap();
        let passed = result["passed"].as_bool().unwrap();
        let score = result["score"].as_f64().unwrap();
        let grade = result["grade"].as_str().unwrap();

        // Only an F grade fails the grade leg in standard mode, so the
        // verdict must satisfy this invariant regardless of exact score.
        assert_eq!(
            passed,
            score >= 50.0 && grade != "F",
            "inverted verdict: score={score} grade={grade} passed={passed}"
        );
        // A clean documented file must pass the standard gate outright.
        assert!(
            passed,
            "clean file should pass standard quality gate (score={score}, grade={grade})"
        );
    }

    #[tokio::test]
    async fn test_check_quality_gate_file_strict_verdict_not_inverted() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = write_clean_rust_file(temp_dir.path());

        // Strict mode: threshold is score >= 70 and grade at least B.
        let result = check_quality_gate_file(&file_path, true).await.unwrap();
        let passed = result["passed"].as_bool().unwrap();
        let score = result["score"].as_f64().unwrap();
        let grade = result["grade"].as_str().unwrap();

        // GH #703: this list used to be spelled in Rust variant names, which is
        // what pinned the `format!("{:?}", ..)` rendering in place. Grades go on
        // the wire in the one symbolic form `Display`/`Serialize` both emit.
        let grade_meets_b = matches!(grade, "A+" | "A" | "A-" | "B+" | "B");
        assert_eq!(
            passed,
            score >= 70.0 && grade_meets_b,
            "inverted strict verdict: score={score} grade={grade} passed={passed}"
        );
    }

    /// GH #703: MCP rendered grades with `format!("{:?}", ..)`, so the shipped
    /// stdio server answered `"grade":"AMinus"` for a score that
    /// `pmat tdg --format json` reported as `"grade":"A-"` — one binary, two
    /// spellings, and no machine consumer able to match on either. The grade a
    /// tool returns must be exactly what `Grade` serialises to.
    #[tokio::test]
    async fn test_mcp_grade_is_the_one_wire_spelling_not_a_variant_name() {
        use crate::tdg::Grade;

        // A single documented function grades A+ — one of the six grades whose
        // variant name ("APlus") and wire spelling ("A+") DIFFER, without which
        // this test would pass against the defect it exists to catch.
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("graded.rs");
        fs::write(&file_path, "/// Doc.\npub fn a() -> i32 { 1 }\n").unwrap();

        let file_result = check_quality_gate_file(&file_path, false).await.unwrap();
        let paths = vec![temp_dir.path().to_path_buf()];
        let project_result = check_quality_gates(&paths, false).await.unwrap();
        let summary = quality_gate_summary(&paths).await.unwrap();

        let observed = [
            file_result["grade"].as_str().unwrap().to_string(),
            project_result["grade"].as_str().unwrap().to_string(),
            summary["summary"]["average_grade"]
                .as_str()
                .unwrap()
                .to_string(),
        ];

        // "A", "B", "C", "D" and "F" spell the same both ways, so a fixture that
        // only ever hits those proves nothing. Fail loudly rather than pass
        // vacuously if the scoring ever moves off the +/- bands.
        let mut discriminating = 0;
        for grade_str in &observed {
            let parsed: Grade = serde_json::from_str(&format!("\"{grade_str}\""))
                .unwrap_or_else(|e| panic!("MCP grade {grade_str:?} is not a Grade: {e}"));
            if format!("{parsed:?}") != parsed.to_string() {
                discriminating += 1;
            }
            assert_eq!(
                *grade_str,
                parsed.to_string(),
                "MCP must emit the Display/Serialize spelling, not the Rust variant name"
            );
            assert_eq!(
                serde_json::to_string(&parsed).unwrap(),
                format!("\"{grade_str}\""),
                "MCP grade must be byte-identical to what serde writes for the same grade"
            );
        }
        assert!(
            discriminating > 0,
            "fixture graded {observed:?} — no +/- grade, so this test could not \
             tell the two spellings apart"
        );
    }

    #[tokio::test]
    async fn test_check_quality_gates_project_verdict_not_inverted() {
        let temp_dir = TempDir::new().unwrap();
        write_clean_rust_file(temp_dir.path());
        // "Clean" has to mean clean for every check this tool runs. It runs the
        // whole `--checks all` suite, and a project with no coverage report is
        // reported by the coverage check as unmeasured — a blocking finding, by
        // the rule that a check which did not run has not passed. The fixture
        // gets the two artifacts the gate reads but does not produce, so this
        // test still measures the verdict inversion it was written for.
        crate::cli::analysis_utilities::write_gate_artifacts(temp_dir.path(), 95.0);

        let paths = vec![temp_dir.path().to_path_buf()];
        let result = check_quality_gates(&paths, false).await.unwrap();
        let passed = result["passed"].as_bool().unwrap();
        let score = result["score"].as_f64().unwrap();
        let grade = result["grade"].as_str().unwrap();

        assert_eq!(
            passed,
            score >= 50.0 && grade != "F",
            "inverted project verdict: score={score} grade={grade} passed={passed}"
        );
        assert!(
            passed,
            "clean project should pass standard quality gate (score={score}, grade={grade})"
        );
        // The original repro: passing grade reported with zero violations
        // yet passed:false. Pin that violations stay consistent too: a passing
        // project carries no blocking (error) finding. Advisory `scope` rows —
        // AD-05's churn disclosure when there is no git history — are allowed;
        // they are disclosure, not breach.
        assert!(
            result["violations"]
                .as_array()
                .unwrap()
                .iter()
                .all(
                    |v| v["severity"].as_str().map(str::to_ascii_lowercase) != Some("error".into())
                ),
            "a passing project must carry no blocking finding: {}",
            result["violations"]
        );
    }

    #[tokio::test]
    async fn test_quality_gate_summary_counts_passing_files() {
        let temp_dir = TempDir::new().unwrap();
        write_clean_rust_file(temp_dir.path());

        let paths = vec![temp_dir.path().to_path_buf()];
        let result = quality_gate_summary(&paths).await.unwrap();
        let summary = &result["summary"];

        assert_eq!(summary["total_files"].as_u64().unwrap(), 1);
        // With the inverted comparison a passing file was counted as failed.
        assert_eq!(
            summary["passed_files"].as_u64().unwrap(),
            1,
            "clean file must count as passed: {summary}"
        );
        assert_eq!(summary["failed_files"].as_u64().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_generate_context_empty_paths() {
        let result = generate_context(&[], None, false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_generate_deep_context_empty_paths() {
        let result = generate_deep_context(&[], None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_analyze_context_empty_paths() {
        let result = analyze_context(&[], &[]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_context_summary_empty_paths() {
        let result = context_summary(&[], None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    #[tokio::test]
    async fn test_analyze_tdg_empty_paths() {
        let result = analyze_tdg(&[], None, None, None, None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one path"));
    }

    // ==================== GIT CLONE TESTS ====================

    #[tokio::test]
    async fn test_git_clone_extracts_repo_name() {
        let result = git_clone("https://github.com/user/test-repo.git", None, None, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("test-repo"));
    }

    #[tokio::test]
    async fn test_git_clone_with_target_dir() {
        let target = Path::new("/tmp/my-target");
        let result = git_clone("https://github.com/user/repo.git", Some(target), None, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/my-target"));
    }

    #[tokio::test]
    async fn test_git_clone_url_without_git_suffix() {
        let result = git_clone("https://github.com/user/some-repo", None, None, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("some-repo"));
    }

    // ==================== STORAGE BACKEND TESTS ====================

    #[test]
    fn test_create_storage_backend_inmemory() {
        let result = create_storage_backend(Some("inmemory"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_storage_backend_libsql() {
        let result = create_storage_backend(Some("libsql"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_storage_backend_default() {
        let result = create_storage_backend(None);
        assert!(result.is_ok()); // Defaults to libsql
    }

    #[test]
    fn test_create_storage_backend_unsupported() {
        let result = create_storage_backend(Some("nonexistent_backend"));
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("Unsupported storage backend"));
    }

    // ==================== RECORD CREATION TESTS ====================

    #[test]
    fn test_create_file_identity() {
        let path = Path::new("/tmp/test.rs");
        let content = b"fn main() {}";
        let hash = blake3::hash(content);

        let identity = create_file_identity(path, &hash, content);

        assert_eq!(identity.path, PathBuf::from("/tmp/test.rs"));
        assert_eq!(identity.content_hash, hash);
        assert_eq!(identity.size_bytes, 12);
    }

    #[test]
    fn test_create_component_scores() {
        let scores = create_component_scores();

        assert!(scores.complexity_breakdown.is_empty());
        assert!(scores.duplication_sources.is_empty());
        assert!(scores.coupling_dependencies.is_empty());
        assert!(scores.doc_missing_items.is_empty());
        assert!(scores.consistency_violations.is_empty());
    }

    #[test]
    fn test_create_semantic_signature() {
        let content = b"fn main() {}";
        let hash = blake3::hash(content);

        let signature = create_semantic_signature(&hash);

        assert_eq!(signature.identifier_pattern, "mcp_analysis");
        assert_eq!(signature.control_flow_pattern, "function_call");
        assert!(signature.import_dependencies.is_empty());
    }

    #[test]
    fn test_create_analysis_metadata() {
        let score = crate::tdg::TdgScore::default();

        let metadata = create_analysis_metadata(&score);

        assert_eq!(metadata.analyzer_version, "2.38.0-mcp");
        assert_eq!(metadata.analysis_duration_ms, 10);
        assert!(!metadata.cache_hit);
    }

    #[test]
    fn test_create_success_result() {
        let path = Path::new("/tmp/test");
        let project_score = crate::tdg::ProjectScore::default();

        let result = create_success_result(path, &project_score);

        assert!(result.get("path").is_some());
        assert!(result.get("total_files").is_some());
        assert!(result.get("average_score").is_some());
        assert!(result.get("average_grade").is_some());
    }

    #[test]
    fn test_create_error_result() {
        let path = Path::new("/tmp/test");
        let error = anyhow::anyhow!("Test error message");

        let result = create_error_result(path, &error);

        assert_eq!(result.get("status").unwrap(), "failed");
        assert!(result
            .get("error")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("Test error message"));
    }

    // ==================== QDD OPERATION TESTS ====================

    #[tokio::test]
    async fn test_quality_driven_development_unknown_operation() {
        let result =
            quality_driven_development("unknown_op", None, None, None, None, None, None, None)
                .await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "failed");
        assert!(json
            .get("message")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("Unknown QDD operation"));
    }

    #[tokio::test]
    async fn test_quality_driven_development_refactor_missing_path() {
        let result = quality_driven_development(
            "refactor",
            Some("standard"),
            None,
            Some("test_func"),
            None,
            None, // Missing file_path
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "failed");
        assert!(json
            .get("message")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("file_path"));
    }

    // ==================== DEFECT AWARE PROMPT TESTS ====================

    #[tokio::test]
    async fn test_generate_defect_aware_prompt_missing_file() {
        let result = generate_defect_aware_prompt(
            "test task".to_string(),
            "test context".to_string(),
            PathBuf::from("/tmp/nonexistent_summary_12345.yaml"),
        )
        .await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "failed");
        assert!(json
            .get("error")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("FILE_NOT_FOUND"));
    }

    // ==================== TDG STORAGE MANAGEMENT TESTS ====================

    #[tokio::test]
    async fn test_tdg_storage_management_unknown_action() {
        let result = tdg_storage_management("unknown_action".to_string(), json!({})).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "error");
        assert!(json
            .get("message")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("Unknown storage action"));
    }

    // ==================== TDG CONFIGURE STORAGE TESTS ====================

    #[tokio::test]
    async fn test_tdg_configure_storage_unsupported_backend() {
        let result =
            tdg_configure_storage("unsupported_backend".to_string(), None, None, None).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "error");
    }

    #[tokio::test]
    async fn test_tdg_configure_storage_inmemory() {
        let result =
            tdg_configure_storage("inmemory".to_string(), None, Some(64), Some(true)).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "completed");
    }

    // ==================== TDG PERFORMANCE METRICS TESTS ====================

    #[tokio::test]
    async fn test_tdg_performance_metrics() {
        let result = tdg_performance_metrics().await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "completed");
        assert!(json.get("adaptive_thresholds").is_some());
        assert!(json.get("performance_stats").is_some());
        assert!(json.get("scheduler_stats").is_some());
    }

    // ==================== TDG HEALTH CHECK TESTS ====================

    #[tokio::test]
    async fn test_tdg_health_check() {
        let result = tdg_health_check().await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json.get("status").unwrap(), "completed");
        assert!(json.get("overall_status").is_some());
        assert!(json.get("health_score").is_some());
        assert!(json.get("components").is_some());
    }

    // ==================== FILE-BASED TESTS (REQUIRE TEMP FILES) ====================

    /// Issue #1090. This test used to read
    ///
    /// ```text
    ///     // Should succeed but with 0 files analyzed
    ///     assert!(result.is_ok());
    ///     assert_eq!(json["results"]["total_files"], 0);
    /// ```
    ///
    /// which pinned the defect instead of the behaviour: `pmat analyze
    /// complexity` refuses a population of nothing with exit 5, and MCP
    /// answered `"status": "completed"` with an empty violation list for the
    /// same input. A measurement that was never taken must not arrive as a
    /// clean one.
    #[tokio::test]
    async fn test_analyze_complexity_with_nonexistent_paths() {
        let paths = vec![PathBuf::from("/tmp/nonexistent_12345.rs")];
        let error = analyze_complexity(&paths, None, None)
            .await
            .expect_err("an empty population must be refused, not reported as completed");

        let message = format!("{error:#}");
        assert!(
            message.contains("no source files were found"),
            "the refusal must be the CLI's own sentence: {message}"
        );
        assert!(
            message.contains("This is not a clean result"),
            "the refusal must say the run is not clean: {message}"
        );
        assert!(
            message.contains("/tmp/nonexistent_12345.rs"),
            "the refusal must name the path it walked: {message}"
        );
    }

    #[tokio::test]
    async fn test_analyze_satd_with_nonexistent_paths() {
        let paths = vec![PathBuf::from("/tmp/nonexistent_12345.rs")];
        let result = analyze_satd(&paths, false, false).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["results"]["total_satd"], 0);
    }

    #[tokio::test]
    async fn test_analyze_dead_code_with_nonexistent_paths() {
        let paths = vec![PathBuf::from("/tmp/nonexistent_12345.rs")];
        let result = analyze_dead_code(&paths, false).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["results"]["total_dead_code"], 0);
        // …and the zero is not a silent one. A path that could not be analysed
        // used to be `continue`d over, which made "no dead code" and "never
        // looked" the same payload.
        let not_analyzed = json["results"]["paths_not_analyzed"]
            .as_array()
            .expect("paths_not_analyzed");
        assert_eq!(not_analyzed.len(), 1, "{json}");
        assert_eq!(
            not_analyzed[0]["path"].as_str(),
            Some("/tmp/nonexistent_12345.rs")
        );
    }

    /// Issue #1090, the same defect one tool over. This used to assert
    /// `result.is_ok()` and `json["context"]["total_files"] == 0` — a walk that
    /// found nothing, reported as a completed context generation.
    #[tokio::test]
    async fn test_generate_context_with_nonexistent_paths() {
        let paths = vec![PathBuf::from("/tmp/nonexistent_12345.rs")];
        let error = generate_context(&paths, None, false)
            .await
            .expect_err("an empty population must be refused, not reported as completed");

        let message = format!("{error:#}");
        assert!(
            message.contains("no source files were found")
                && message.contains("no context measurement was taken")
                && message.contains("This is not a clean result"),
            "the refusal must be the CLI's own sentence: {message}"
        );
        assert!(
            message.contains("/tmp/nonexistent_12345.rs"),
            "the refusal must name the path it walked: {message}"
        );
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_real_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() { println!(\"Hello\"); }")?;

        let paths = vec![file_path];
        let result = analyze_complexity(&paths, None, Some(5)).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["status"], "completed");

        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_satd_with_real_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() {\n    // TODO: fix this\n    println!(\"Hello\");\n}",
        )?;

        let paths = vec![file_path];
        let result = analyze_satd(&paths, false, false).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["status"], "completed");

        Ok(())
    }

    #[tokio::test]
    async fn test_context_summary_with_real_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}")?;

        let paths = vec![temp_dir.path().to_path_buf()];
        let result = context_summary(&paths, None).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["status"], "completed");
        assert!(json["summary"]["total_files"].as_u64().unwrap() >= 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_quality_gate_compare_missing_baseline() {
        let paths = vec![PathBuf::from("/tmp")];
        let result =
            quality_gate_compare(Path::new("/tmp/nonexistent_baseline_12345.json"), &paths).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Baseline file not found"));
    }

    // ==================== PROPERTY-BASED TESTS ====================

    proptest! {
        #[test]
        fn prop_select_quality_profile_never_panics(profile_name in ".*") {
            let _ = select_quality_profile(Some(&profile_name));
        }

        #[test]
        fn prop_parse_code_type_never_panics(code_type in ".*") {
            let _ = parse_code_type(Some(&code_type));
        }

        #[test]
        fn prop_create_file_identity_with_any_content(content in proptest::collection::vec(any::<u8>(), 0..1000)) {
            let path = Path::new("/tmp/test.rs");
            let hash = blake3::hash(&content);
            let identity = create_file_identity(path, &hash, &content);

            prop_assert_eq!(identity.size_bytes, content.len() as u64);
            prop_assert_eq!(identity.content_hash, hash);
        }

        #[test]
        fn prop_create_semantic_signature_deterministic(content in proptest::collection::vec(any::<u8>(), 0..100)) {
            let hash = blake3::hash(&content);
            let sig1 = create_semantic_signature(&hash);
            let sig2 = create_semantic_signature(&hash);

            prop_assert_eq!(sig1.ast_structure_hash, sig2.ast_structure_hash);
            prop_assert_eq!(sig1.identifier_pattern, sig2.identifier_pattern);
        }

        #[test]
        fn prop_git_clone_url_parsing(repo_name in "[a-zA-Z0-9-]+") {
            let url = format!("https://github.com/user/{}.git", repo_name);
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(git_clone(&url, None, None, None));

            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), PathBuf::from(&repo_name));
        }
    }

    // ==================== TDG SYSTEM DIAGNOSTICS TESTS ====================

    #[tokio::test]
    async fn test_tdg_system_diagnostics_all_components() {
        let result = tdg_system_diagnostics(true, vec!["all".to_string()]).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        // Result status depends on system state
        assert!(json.get("status").is_some());
    }

    #[tokio::test]
    async fn test_tdg_system_diagnostics_specific_components() {
        let result = tdg_system_diagnostics(false, vec!["storage".to_string()]).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("status").is_some());
    }

    #[tokio::test]
    async fn test_tdg_system_diagnostics_empty_components() {
        let result = tdg_system_diagnostics(false, vec![]).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("status").is_some());
    }

    // ==================== EDGE CASE TESTS ====================

    #[test]
    fn test_create_file_identity_empty_content() {
        let path = Path::new("/tmp/empty.rs");
        let content = b"";
        let hash = blake3::hash(content);

        let identity = create_file_identity(path, &hash, content);

        assert_eq!(identity.size_bytes, 0);
    }

    #[test]
    fn test_create_semantic_signature_hash_bytes() {
        // Test that the AST structure hash is computed from first 8 bytes
        let content = b"deterministic content";
        let hash = blake3::hash(content);
        let signature = create_semantic_signature(&hash);

        // Verify the hash is computed (non-zero for non-trivial content)
        // The actual value depends on blake3 hash of the content
        assert!(signature.ast_structure_hash > 0 || content.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspots_file_path_error() {
        // Pass a file path instead of directory
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let paths = vec![file_path];
        let result = analyze_lint_hotspots(&paths, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("directory"));
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_threshold() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn simple() {}")?;

        let paths = vec![file_path];
        // High threshold - no violations expected
        let result = analyze_complexity(&paths, Some(5), Some(100)).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["results"]["violations"].as_array().unwrap().len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_top_files_limit() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create multiple files
        for i in 0..5 {
            let file_path = temp_dir.path().join(format!("test{}.rs", i));
            fs::write(&file_path, "fn main() {}")?;
        }

        let paths: Vec<PathBuf> = (0..5)
            .map(|i| temp_dir.path().join(format!("test{}.rs", i)))
            .collect();

        let result = analyze_complexity(&paths, Some(2), None).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        // top_files limits the output
        assert!(json["results"]["top_files"].as_array().unwrap().len() <= 2);

        Ok(())
    }

    // ==================== R17-2 D82 REGRESSION TESTS ====================
    // Directory inputs must produce non-zero counts. Previously handlers
    // required path.is_file() and returned authoritative zeros for dirs.

    #[tokio::test]
    async fn test_analyze_complexity_walks_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("sample.rs"),
            "fn a() { if true { 1 } else { 2 }; } fn b() {}",
        )?;
        fs::write(
            temp_dir.path().join("nested.rs"),
            "fn c() { for i in 0..10 { let _ = i; } }",
        )?;

        let paths = vec![temp_dir.path().to_path_buf()];
        let json = analyze_complexity(&paths, None, None).await.unwrap();

        let total_files = json["results"]["total_files"].as_u64().unwrap_or(0);
        assert!(
            total_files >= 2,
            "expected >=2 files analyzed, got {total_files}; json={json}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_satd_walks_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("debt.rs"),
            "// TODO: refactor this later\nfn x() {}\n// FIXME: edge case\n",
        )?;

        let paths = vec![temp_dir.path().to_path_buf()];
        let json = analyze_satd(&paths, false, false).await.unwrap();

        let total = json["results"]["total_satd"].as_u64().unwrap_or(0);
        assert!(
            total >= 1,
            "expected >=1 SATD item found, got {total}; json={json}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_dead_code_walks_directory() -> Result<()> {
        // Create a minimal Rust project so language detection finds "rust"
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname=\"t\"\nversion=\"0.1.0\"\n",
        )?;
        let src = temp_dir.path().join("src");
        fs::create_dir(&src)?;
        fs::write(
            src.join("lib.rs"),
            "pub fn used() {}\nfn unused_helper() { let _=1; }\n",
        )?;
        crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(temp_dir.path());

        let paths = vec![temp_dir.path().to_path_buf()];
        let json = analyze_dead_code(&paths, false).await.unwrap();

        assert_eq!(json["status"], "completed");
        assert_eq!(
            json["results"]["paths_not_analyzed"]
                .as_array()
                .expect("paths_not_analyzed")
                .len(),
            0,
            "the fixture was not analysed: {json}"
        );
        // NAMED, not counted. This used to assert `total_functions >= 1` — a
        // denominator that says nothing about whether the analysis found the
        // right thing, and one the shared analyzer does not compute. What the
        // tool has to get right here is which of the two functions is dead:
        // `unused_helper` is, and `used` is this LIBRARY's public API, which the
        // reachability analyzer used to call dead.
        let named: Vec<String> = json["results"]["files"]
            .as_array()
            .expect("files array")
            .iter()
            .flat_map(|f| {
                f["dead_functions"]
                    .as_array()
                    .expect("dead_functions")
                    .iter()
            })
            .map(|i| i["name"].as_str().expect("name").to_string())
            .collect();
        assert_eq!(named, vec!["unused_helper".to_string()], "json={json}");
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_context_walks_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("a.rs"), "pub fn alpha() {}\n")?;
        fs::write(temp_dir.path().join("b.rs"), "pub struct Beta;\n")?;

        let paths = vec![temp_dir.path().to_path_buf()];
        let json = generate_context(&paths, None, false).await.unwrap();

        let total = json["context"]["total_files"].as_u64().unwrap_or(0);
        assert!(
            total >= 2,
            "expected >=2 files in context, got {total}; json={json}"
        );
        Ok(())
    }

    // ==================== R21-4 D98 GLOB EXPANSION TESTS ====================
    // MCP analyze_* handlers receive PathBufs built from raw client strings.
    // When the client passes `**/*.rs`, the PathBuf is a literal non-existent
    // path and `path.exists()` returns false. `resolve_paths_with_globs` must
    // expand such glob patterns BEFORE the existence check so downstream
    // file-count metrics are non-zero.

    /// Helper: build a small on-disk Rust project with root + nested files.
    fn make_rust_project(temp: &TempDir) -> Result<(PathBuf, PathBuf, PathBuf)> {
        let root = temp.path().join("root.rs");
        fs::write(&root, "fn root_fn() { let _ = 1 + 1; }")?;

        let src = temp.path().join("src");
        fs::create_dir_all(&src)?;
        let lib = src.join("lib.rs");
        fs::write(&lib, "pub fn lib_fn() { if true { 1 } else { 2 }; }")?;

        let nested = src.join("nested");
        fs::create_dir_all(&nested)?;
        let deep = nested.join("deep.rs");
        fs::write(&deep, "pub fn deep_fn() { for i in 0..3 { let _ = i; } }")?;

        Ok((root, lib, deep))
    }

    #[test]
    fn test_resolve_paths_with_globs_recursive_star_star() -> Result<()> {
        let temp = TempDir::new()?;
        make_rust_project(&temp)?;

        let pattern = temp.path().join("**/*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);

        // Must find all 3 .rs files (root + nested + deep).
        let rs_count = resolved
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .count();
        assert_eq!(
            rs_count, 3,
            "expected 3 .rs files from **/*.rs, got {rs_count}: {resolved:?}"
        );
        Ok(())
    }

    #[test]
    fn test_resolve_paths_with_globs_shallow_star() -> Result<()> {
        let temp = TempDir::new()?;
        make_rust_project(&temp)?;

        let pattern = temp.path().join("*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);

        let rs_files: Vec<_> = resolved
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert!(
            !rs_files.is_empty(),
            "expected at least 1 .rs file from *.rs, got 0: {resolved:?}"
        );
        Ok(())
    }

    #[test]
    fn test_resolve_paths_with_globs_dir_star_star() -> Result<()> {
        let temp = TempDir::new()?;
        make_rust_project(&temp)?;

        // `src/**` is the "everything under src" pattern. The glob crate
        // returns whatever entries match the literal glob; when composed with
        // R17-2's `expand_paths_to_source_files` those dirs are walked.
        // Standalone we assert the glob produced at least one entry — the
        // pre-fix behavior returned zero because the literal pattern path did
        // not exist on disk.
        let pattern = temp.path().join("src").join("**");
        let resolved = resolve_paths_with_globs(&[pattern]);

        assert!(
            !resolved.is_empty(),
            "expected src/** to resolve to at least one entry, got {resolved:?}"
        );
        Ok(())
    }

    #[test]
    fn test_resolve_paths_with_globs_dir_star_star_ext() -> Result<()> {
        let temp = TempDir::new()?;
        make_rust_project(&temp)?;

        let pattern = temp.path().join("src").join("**").join("*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);

        let rs_files: Vec<_> = resolved
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert_eq!(
            rs_files.len(),
            2,
            "expected 2 .rs files from src/**/*.rs (lib + deep), got {}: {resolved:?}",
            rs_files.len()
        );
        Ok(())
    }

    #[test]
    fn test_resolve_paths_with_globs_plain_path_passthrough() -> Result<()> {
        let temp = TempDir::new()?;
        let (root, _, _) = make_rust_project(&temp)?;

        // A plain path (no glob metacharacters) must pass through unchanged,
        // preserving composability with R17-2 expand_paths_to_source_files.
        let resolved = resolve_paths_with_globs(std::slice::from_ref(&root));
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0], root);
        Ok(())
    }

    #[test]
    fn test_resolve_paths_with_globs_no_match_drops_pattern() {
        // A glob with no matches yields an empty Vec, not the literal pattern.
        // Callers downstream will simply see 0 files, matching pre-fix UX.
        let pattern = PathBuf::from("/tmp/pmat-r21-4-definitely-does-not-exist-*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);
        assert!(resolved.is_empty(), "expected empty, got {resolved:?}");
    }

    /// Integration: MCP `analyze_complexity` with `**/*.rs` must return
    /// non-zero file_count across a nested Rust tree. Mirrors the R17-2 D82
    /// directory-walking pattern.
    #[tokio::test]
    async fn test_analyze_complexity_with_double_star_glob() -> Result<()> {
        let temp = TempDir::new()?;
        make_rust_project(&temp)?;

        let pattern = temp.path().join("**/*.rs");
        let json = analyze_complexity(&[pattern], None, None).await?;

        let total_files = json["results"]["total_files"].as_u64().unwrap_or(0);
        assert!(
            total_files >= 3,
            "expected >=3 files from **/*.rs, got {total_files}; json={json}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_satd_with_double_star_glob() -> Result<()> {
        let temp = TempDir::new()?;
        fs::write(
            temp.path().join("root.rs"),
            "// TODO: top-level debt\nfn root_fn() {}",
        )?;
        let src = temp.path().join("src");
        fs::create_dir_all(&src)?;
        fs::write(
            src.join("nested.rs"),
            "// FIXME: nested debt\nfn nested_fn() {}",
        )?;

        let pattern = temp.path().join("**/*.rs");
        let json = analyze_satd(&[pattern], false, false).await?;

        let total = json["results"]["total_satd"].as_u64().unwrap_or(0);
        assert!(
            total >= 2,
            "expected >=2 SATD entries from **/*.rs walk, got {total}; json={json}"
        );
        Ok(())
    }

    /// When the input path is `src/**/*.rs` the composed pipeline —
    /// `resolve_paths_with_globs` → `expand_paths_to_source_files` — must
    /// find every `.rs` file below `src/`. This is a D98 acceptance case.
    #[tokio::test]
    async fn test_analyze_complexity_with_dir_star_star_glob() -> Result<()> {
        let temp = TempDir::new()?;
        make_rust_project(&temp)?;

        let pattern = temp.path().join("src").join("**").join("*.rs");
        let json = analyze_complexity(&[pattern], None, None).await?;

        let total_files = json["results"]["total_files"].as_u64().unwrap_or(0);
        assert!(
            total_files >= 2,
            "expected >=2 files from src/**/*.rs, got {total_files}; json={json}"
        );
        Ok(())
    }

    /// GH #667: `analyze_deep_context` returned maintainability_index 70.0,
    /// modularity_score 85.0 and technical_debt_hours 40.0 for six wildly
    /// different code bases — a 5-file toy fixture and pmat's own 3891-file
    /// tree included. Anything pmat cannot measure must be `null` and named in
    /// `not_measured`, never a plausible number.
    #[test]
    fn mcp_scorecard_marks_unmeasured_fields_instead_of_inventing_them() {
        use crate::services::deep_context::QualityScorecard;

        let nothing_measured = QualityScorecard::default();
        let json = quality_scorecard_json(&nothing_measured);

        for field in [
            "overall_health",
            "complexity_score",
            "maintainability_index",
            "modularity_score",
            "test_coverage",
            "technical_debt_hours",
        ] {
            assert!(
                json[field].is_null(),
                "{field} must be null when unmeasured, got {}",
                json[field]
            );
            assert!(
                json["not_measured"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|n| n == field),
                "{field} must be listed in not_measured: {json}"
            );
        }

        // None of the constants that used to be reported may appear.
        let text = json.to_string();
        for fabricated in ["70.0", "85.0", "40.0", "65.0"] {
            assert!(
                !text.contains(fabricated),
                "unmeasured scorecard still emits {fabricated}: {text}"
            );
        }
    }

    /// Measured fields pass through, and are not listed as unmeasured.
    #[test]
    fn mcp_scorecard_passes_measured_values_through() {
        use crate::services::deep_context::QualityScorecard;

        let scorecard = QualityScorecard {
            overall_health: Some(62.5),
            complexity_score: Some(62.5),
            maintainability_index: None,
            modularity_score: None,
            test_coverage: Some(41.0),
            technical_debt_hours: Some(12.25),
        };
        let json = quality_scorecard_json(&scorecard);

        assert_eq!(json["complexity_score"], 62.5);
        assert_eq!(json["test_coverage"], 41.0);
        assert_eq!(json["technical_debt_hours"], 12.25);

        let not_measured: Vec<_> = json["not_measured"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            not_measured,
            vec!["maintainability_index", "modularity_score"]
        );
    }
}

mod dag_type_validation_tests {
    //! `analyze_dag` used to end its `dag_type` match in
    //! `_ => DagType::FullDependency`, so `"BOGUS"`, `""` and `"12345"` all came
    //! back `status: "completed"` with `results.dag_type: "FullDependency"` and
    //! no warning — a client that typo'd the mode received a successful-looking
    //! result for a DIFFERENT analysis than the one it asked for. The sibling
    //! enums (`generate_context`'s `format`, `scaffold_project`'s `level`) reject
    //! their unknown values; a schema-declared `enum` must not be coerced.
    use super::*;
    use crate::services::deep_context::DagType;

    #[test]
    fn every_advertised_dag_type_parses() {
        for name in DAG_TYPES {
            assert!(
                parse_dag_type(Some(name)).is_ok(),
                "{name} is in the tool's inputSchema enum and must parse"
            );
        }
        // Underscore spellings were accepted before and stay accepted.
        assert!(matches!(
            parse_dag_type(Some("call_graph")).unwrap(),
            DagType::CallGraph
        ));
    }

    #[test]
    fn omitting_dag_type_defaults_to_full_dependency() {
        assert!(matches!(
            parse_dag_type(None).unwrap(),
            DagType::FullDependency
        ));
    }

    #[test]
    fn an_unknown_dag_type_is_rejected_not_coerced() {
        for bad in ["BOGUS", "", "12345", "callgraph"] {
            let err = parse_dag_type(Some(bad))
                .expect_err("an unknown dag_type must be an error, not a silent FullDependency");
            let message = err.to_string();
            assert!(
                message.contains("Unsupported dag_type"),
                "the error must name the offending argument: {message}"
            );
            assert!(
                message.contains("full-dependency"),
                "the error must list the accepted values: {message}"
            );
        }
    }
}
