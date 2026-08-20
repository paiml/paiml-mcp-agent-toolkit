// Tests for agent_context_tools
// Split from agent_context_tools.rs for maintainability

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_code_schema() {
        let schema = QueryCodeTool::schema();
        assert_eq!(schema["name"], "pmat_query_code");
        assert!(schema["parameters"]["properties"]["query"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["query"]));
    }

    #[test]
    fn test_get_function_schema() {
        let schema = GetFunctionTool::schema();
        assert_eq!(schema["name"], "pmat_get_function");
        assert!(schema["parameters"]["properties"]["function_id"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["function_id"]));
    }

    #[test]
    fn test_find_similar_schema() {
        let schema = FindSimilarTool::schema();
        assert_eq!(schema["name"], "pmat_find_similar");
        assert!(schema["parameters"]["properties"]["function_id"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["function_id"]));
    }

    /// REGRESSION: `min_grade` was validated by nobody.
    ///
    /// The schema declared `enum: [A, B, C, D, F]` and the filter underneath
    /// simply failed to match anything it did not understand, so
    /// `min_grade: "Z"` and `min_grade: ""` both came back `total: 0` — the
    /// same answer as a query with genuinely no matches. A typo was
    /// indistinguishable from an empty result set.
    #[test]
    fn test_min_grade_out_of_enum_is_rejected_not_silently_empty() {
        for bad in ["Z", "", "  ", "A++", "E", "grade A", "1"] {
            let err = validate_min_grade(bad)
                .expect_err("an out-of-enum min_grade must be rejected, not silently return zero");
            assert!(
                err.is_invalid_params(),
                "{bad:?} is the caller's mistake: {err:?}"
            );
            assert!(
                err.message().contains("Invalid min_grade"),
                "the message must name the offending argument: {err}"
            );
        }
    }

    /// The other half of the same contract: everything the schema advertises
    /// must be accepted. A validator that rejected `A-` would trade a
    /// silent-zero for a false rejection.
    #[test]
    fn test_every_advertised_min_grade_is_accepted() {
        let advertised = min_grade_enum();
        assert_eq!(
            advertised.len(),
            11,
            "all eleven grades, not the five-letter table: {advertised:?}"
        );
        for grade in &advertised {
            assert!(
                validate_min_grade(grade).is_ok(),
                "the schema advertises {grade:?} but the validator rejects it"
            );
        }
    }

    /// `min_grade: "a"` and `min_grade: "A+"` already worked before any
    /// validation existed, and the CLI's `--min-grade` accepts both, so the
    /// deliberate decision is to keep them — being stricter than the CLI would
    /// swap a silent-zero for a surface-to-surface contradiction. The schema
    /// says "case-insensitive" and lists the modifier grades so this is
    /// documented rather than accidental.
    #[test]
    fn test_case_insensitivity_and_modifier_grades_are_supported_deliberately() {
        for accepted in ["a", "f", "a+", "A+", "b-", "C-"] {
            assert!(
                validate_min_grade(accepted).is_ok(),
                "{accepted:?} worked before validation and must keep working"
            );
        }
        let schema = QueryCodeTool::schema();
        let advertised = &schema["parameters"]["properties"]["min_grade"];
        assert_eq!(
            advertised["enum"],
            json!(min_grade_enum()),
            "the schema must list exactly what the validator accepts"
        );
        assert!(
            advertised["description"]
                .as_str()
                .unwrap_or_default()
                .contains("Case-insensitive"),
            "case-insensitivity is a supported behaviour, so it must be documented: {advertised}"
        );
    }

    /// The advertised schema is served from `mcp_tool_schemas/*.json` (build.rs
    /// codegen), NOT from `QueryCodeTool::schema()`. Two hand-written copies of
    /// one enum is how the five-letter table survived; assert they agree.
    #[test]
    fn test_generated_manifest_min_grade_enum_matches_the_validator() {
        let info = crate::mcp_pmcp::tool_schemas_generated::tool_info_for("pmat_query_code");
        let advertised = &info.input_schema["properties"]["min_grade"]["enum"];
        assert_eq!(
            *advertised,
            json!(min_grade_enum()),
            "tools/list advertises a different min_grade enum than the validator enforces"
        );
    }

    #[test]
    fn test_index_stats_schema() {
        let schema = IndexStatsTool::schema();
        assert_eq!(schema["name"], "pmat_index_stats");
        assert!(schema["parameters"]["properties"]["rebuild"].is_object());
    }

    #[test]
    fn test_all_tool_names() {
        assert_eq!(QueryCodeTool::schema()["name"], "pmat_query_code");
        assert_eq!(GetFunctionTool::schema()["name"], "pmat_get_function");
        assert_eq!(FindSimilarTool::schema()["name"], "pmat_find_similar");
        assert_eq!(IndexStatsTool::schema()["name"], "pmat_index_stats");
    }

    #[test]
    fn test_parse_function_id_valid() {
        let (file, func) = parse_function_id("src/handlers/auth.rs::handle_login").unwrap();
        assert_eq!(file, "src/handlers/auth.rs");
        assert_eq!(func, "handle_login");
    }

    #[test]
    fn test_parse_function_id_nested() {
        let (file, func) = parse_function_id("src/foo/bar.rs::baz::qux").unwrap();
        assert_eq!(file, "src/foo/bar.rs::baz");
        assert_eq!(func, "qux");
    }

    #[test]
    fn test_parse_function_id_invalid_no_separator() {
        let result = parse_function_id("no_separator");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("Invalid function_id format"));
        // A malformed ID is the caller's, so it must carry the variant that
        // becomes -32602 rather than -32603.
        assert!(err.is_invalid_params(), "got: {err:?}");
    }

    #[test]
    fn test_parse_function_id_invalid_empty_parts() {
        let result = parse_function_id("::function_only");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_index_manager_new() {
        let manager = IndexManager::new(PathBuf::from("/tmp/test"));
        let guard = manager.index.read().await;
        assert!(guard.is_none());
    }

    /// A `QueryResult` carrying a source body, built through serde so the test
    /// does not have to name every field of the struct.
    fn query_result_with_source() -> crate::services::agent_context::QueryResult {
        serde_json::from_value(json!({
            "file_path": "src/workflow/recovery.rs",
            "function_name": "handle_error",
            "signature": "fn handle_error()",
            "doc_comment": null,
            "start_line": 1,
            "end_line": 3,
            "language": "rust",
            "tdg_score": 1.0,
            "tdg_grade": "A",
            "complexity": 1,
            "big_o": "O(1)",
            "satd_count": 0,
            "loc": 3,
            "relevance_score": 1.0,
            "source": "fn handle_error() {\n    todo!()\n}"
        }))
        .expect("QueryResult fixture must deserialize")
    }

    /// `include_source` was parsed into a discarded `_include_source`, so
    /// `pmat_get_function` returned the whole body even when the caller asked
    /// it not to — the flag was documentation for behaviour the tool lacked.
    #[test]
    fn test_get_function_response_omits_source_when_not_requested() {
        let result = query_result_with_source();

        let with = super::build_get_function_response(
            "src/workflow/recovery.rs::handle_error",
            &result,
            true,
        );
        assert!(
            with["source"].is_string(),
            "include_source: true must return the body"
        );

        let without = super::build_get_function_response(
            "src/workflow/recovery.rs::handle_error",
            &result,
            false,
        );
        assert!(
            without.get("source").is_none(),
            "include_source: false must not return the body, got: {without}"
        );
        // Everything else is unaffected by the flag.
        assert_eq!(without["name"], "handle_error");
        assert_eq!(without["quality"]["grade"], "A");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod argument_validation_tests {
    //! REGRESSION: the bounds were enforced at ONE end.
    //!
    //! `{"limit": 9999}` was refused with "Limit exceeds maximum of 100", while
    //!
    //! ```text
    //! {"query":"x","limit":-1}    -> 10 results
    //! {"query":"x","limit":2.5}   -> 10 results
    //! {"query":"x","limit":"10"}  -> 10 results
    //! ```
    //!
    //! all came back as a perfectly ordinary default page, because
    //! `params["limit"].as_u64().unwrap_or(10)` cannot tell "the caller said
    //! nothing" from "the caller said something a `u64` cannot hold". A typo was
    //! again indistinguishable from an intended value — the exact thing the
    //! upper bound was added to prevent — and the same `unwrap_or(default)`
    //! shape sat on every other optional argument of all four tools.
    use super::*;

    /// A tiny indexable tree, so the ACCEPT half of each test exercises the
    /// whole `execute` path rather than stopping at validation.
    fn indexable_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("lib.rs"),
            "/// Handles an error.\npub fn handle_error(x: i32) -> i32 {\n    if x > 0 { 1 } else { 0 }\n}\n",
        )
        .expect("write fixture");
        dir
    }

    fn query_tool(dir: &tempfile::TempDir) -> QueryCodeTool {
        QueryCodeTool::new(Arc::new(IndexManager::new(dir.path().to_path_buf())))
    }

    /// Values no schema in this file admits: below the minimum, fractional, the
    /// right number wearing quotes, a bool, a container.
    fn bad_integers() -> Vec<Value> {
        vec![
            json!(-1),
            json!(0),
            json!(2.5),
            json!("10"),
            json!(true),
            json!([10]),
            json!({"n": 10}),
        ]
    }

    fn assert_rejected(err: &ToolError, key: &str) {
        assert!(
            err.is_invalid_params(),
            "a bad {key} is the CALLER's mistake (-32602), not ours: {err:?}"
        );
        assert!(
            err.message().contains(key),
            "the message must name the offending argument {key}: {err}"
        );
    }

    #[tokio::test]
    async fn out_of_range_and_non_numeric_limit_are_rejected_not_defaulted() {
        let dir = indexable_dir();
        let tool = query_tool(&dir);

        for bad in bad_integers().into_iter().chain([json!(101), json!(9999)]) {
            let err = tool
                .execute(json!({"query": "error", "limit": bad}))
                .await
                .expect_err(&format!(
                    "limit {bad} must be rejected, not defaulted to 10"
                ));
            assert_rejected(&err, "limit");
        }
    }

    /// A silently dropped `max_complexity` is worse than a silently defaulted
    /// `limit`: the caller asked for a FILTER and got the unfiltered set back,
    /// with nothing in the payload to say the filter never ran.
    #[tokio::test]
    async fn out_of_range_and_non_numeric_max_complexity_are_rejected_not_dropped() {
        let dir = indexable_dir();
        let tool = query_tool(&dir);

        for bad in bad_integers() {
            let err = tool
                .execute(json!({"query": "error", "max_complexity": bad}))
                .await
                .expect_err(&format!(
                    "max_complexity {bad} must be rejected, not silently dropped"
                ));
            assert_rejected(&err, "max_complexity");
        }
    }

    /// `include_source: "false"` read as the default — for `pmat_get_function`
    /// that default is `true`, so a JSON-typing slip returned the whole body a
    /// caller had explicitly tried to keep out of their context window.
    #[tokio::test]
    async fn non_boolean_flags_are_rejected_not_defaulted() {
        let dir = indexable_dir();
        let manager = Arc::new(IndexManager::new(dir.path().to_path_buf()));

        for bad in [json!("false"), json!("true"), json!(0), json!(1), json!([])] {
            let err = QueryCodeTool::new(manager.clone())
                .execute(json!({"query": "error", "include_source": bad}))
                .await
                .expect_err(&format!("include_source {bad} must be rejected"));
            assert_rejected(&err, "include_source");

            let err = GetFunctionTool::new(manager.clone())
                .execute(json!({"function_id": "lib.rs::handle_error", "include_source": bad}))
                .await
                .expect_err(&format!("include_source {bad} must be rejected"));
            assert_rejected(&err, "include_source");

            let err = IndexStatsTool::new(manager.clone())
                .execute(json!({"rebuild": bad}))
                .await
                .expect_err(&format!("rebuild {bad} must be rejected"));
            assert_rejected(&err, "rebuild");
        }
    }

    /// A non-string `min_grade` skipped `validate_min_grade` entirely, so the
    /// validation added for `"Z"` never saw `123` at all.
    #[tokio::test]
    async fn non_string_filters_are_rejected_not_ignored() {
        let dir = indexable_dir();
        let tool = query_tool(&dir);

        for key in ["min_grade", "language", "path_pattern"] {
            for bad in [json!(123), json!(true), json!(["A"])] {
                let err = tool
                    .execute(json!({"query": "error", key: bad}))
                    .await
                    .expect_err(&format!("{key} {bad} must be rejected, not ignored"));
                assert_rejected(&err, key);
            }
        }
    }

    /// `pmat_find_similar` carried the same pair of half-bounds.
    #[tokio::test]
    async fn find_similar_bounds_are_enforced_at_both_ends() {
        let dir = indexable_dir();
        let tool = FindSimilarTool::new(Arc::new(IndexManager::new(dir.path().to_path_buf())));
        let id = json!("lib.rs::handle_error");

        for bad in bad_integers().into_iter().chain([json!(21)]) {
            let err = tool
                .execute(json!({"function_id": id, "limit": bad}))
                .await
                .expect_err(&format!("limit {bad} must be rejected"));
            assert_rejected(&err, "limit");
        }

        for bad in [json!(-0.1), json!(1.1), json!("high"), json!(true)] {
            let err = tool
                .execute(json!({"function_id": id, "min_similarity": bad}))
                .await
                .expect_err(&format!("min_similarity {bad} must be rejected"));
            assert_rejected(&err, "min_similarity");
        }
    }

    /// The other half of the contract. Without this, "reject everything" would
    /// pass every test above: an ABSENT key, and an explicit `null` (how every
    /// other pmat tool's `#[serde(default)] Option<T>` spells "unset"), still
    /// mean "use the default", and every in-range value is still accepted.
    #[tokio::test]
    async fn absent_null_and_in_range_arguments_are_still_accepted() {
        let dir = indexable_dir();
        let tool = query_tool(&dir);

        for good in [
            json!({"query": "error"}),
            json!({"query": "error", "limit": null, "max_complexity": null}),
            json!({"query": "error", "limit": 1}),
            json!({"query": "error", "limit": 100, "max_complexity": 100}),
            json!({"query": "error", "include_source": true, "min_grade": "A-"}),
            json!({"query": "error", "language": "rust", "path_pattern": "lib"}),
        ] {
            tool.execute(good.clone())
                .await
                .unwrap_or_else(|e| panic!("{good} is a legal call and must succeed: {e}"));
        }
    }
}
