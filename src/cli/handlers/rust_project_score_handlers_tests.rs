// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::rust_project_score::models::CategoryScore;
    use crate::services::rust_project_score::orchestrator::{ProjectScore, SPEC_VERSION};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn strip_ansi(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").to_string()
    }

    fn create_test_score() -> ProjectScore {
        let mut categories = HashMap::new();
        categories.insert("Rust Tooling".to_string(), CategoryScore::new(20.0, 25.0));
        categories.insert("Code Quality".to_string(), CategoryScore::new(15.0, 26.0));
        categories.insert("Testing".to_string(), CategoryScore::new(18.0, 20.0));

        ProjectScore {
            total_earned: 53.0,
            total_possible: 71.0,
            percentage: 74.6,
            grade: crate::services::rust_project_score::models::Grade::B,
            categories,
            recommendations: vec![
                "Add more tests".to_string(),
                "Improve documentation".to_string(),
            ],
        }
    }

    #[tokio::test]
    async fn test_handler_invalid_path() {
        let result = handle_rust_project_score(
            Path::new("/nonexistent/path"),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
            false, // full mode
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_handler_not_a_directory() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("file.txt");
        std::fs::write(&file_path, "not a directory").unwrap();

        let result = handle_rust_project_score(
            &file_path,
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
            false, // full mode
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[tokio::test]
    async fn test_handler_no_cargo_toml() {
        let temp = TempDir::new().unwrap();

        let result = handle_rust_project_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
            false, // full mode
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cargo.toml"));
    }

    // =========================================================================
    // Format function tests
    // =========================================================================

    #[test]
    fn test_format_text_contains_header() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false, false);

        assert!(output.contains("Rust Project Score"));
        assert!(output.contains(SPEC_VERSION));
    }

    #[test]
    fn test_format_text_contains_summary() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false, false);

        assert!(output.contains("Summary"));
        assert!(output.contains("Score:"));
        assert!(output.contains("Normalized:"));
        assert!(output.contains("Grade:"));
    }

    #[test]
    fn test_format_text_contains_categories() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false, false);

        assert!(output.contains("Categories"));
        assert!(output.contains("Rust Tooling"));
        assert!(output.contains("Code Quality"));
        assert!(output.contains("Testing"));
    }

    #[test]
    fn test_format_text_contains_recommendations() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false, false);

        assert!(output.contains("Recommendations"));
        assert!(output.contains("Add more tests"));
        assert!(output.contains("Improve documentation"));
    }

    #[test]
    fn test_format_text_no_recommendations() {
        let mut score = create_test_score();
        score.recommendations = vec![];
        let output = format_text(&score, &[], false, false);

        // Should not contain Recommendations section when empty
        assert!(!output.contains("Recommendations"));
    }

    #[test]
    fn test_format_text_icons_passing() {
        let mut score = create_test_score();
        score.categories.clear();
        score
            .categories
            .insert("Perfect".to_string(), CategoryScore::new(95.0, 100.0));
        let output = strip_ansi(&format_text(&score, &[], false, false));

        assert!(output.contains("\u{2713}")); // ✓ checkmark
    }

    #[test]
    fn test_format_text_icons_warning() {
        let mut score = create_test_score();
        score.categories.clear();
        score
            .categories
            .insert("Warning".to_string(), CategoryScore::new(75.0, 100.0));
        let output = strip_ansi(&format_text(&score, &[], false, false));

        assert!(output.contains("\u{26A0}")); // ⚠ warning sign
    }

    #[test]
    fn test_format_text_icons_failing() {
        let mut score = create_test_score();
        score.categories.clear();
        score
            .categories
            .insert("Failing".to_string(), CategoryScore::new(50.0, 100.0));
        let output = strip_ansi(&format_text(&score, &[], false, false));

        assert!(output.contains("\u{2717}")); // ✗ ballot x
    }

    #[test]
    fn test_format_json_valid_json() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations, false).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_format_json_contains_fields() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations, false).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["version"].is_string());
        assert!(parsed["total_earned"].is_f64());
        assert!(parsed["total_possible"].is_f64());
        assert!(parsed["percentage"].is_f64());
        assert!(parsed["grade"].is_string());
        assert!(parsed["categories"].is_array());
        assert!(parsed["recommendations"].is_array());
    }

    #[test]
    fn test_format_json_correct_values() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations, false).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_earned"].as_f64().unwrap(), 53.0);
        assert_eq!(parsed["percentage"].as_f64().unwrap(), 74.6);
    }

    /// Issue #240: Verify that `applicable` field is present on every category in JSON output
    #[test]
    fn test_format_json_categories_have_applicable_field() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations, false).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let categories = parsed["categories"].as_array().unwrap();
        assert!(!categories.is_empty(), "categories array should not be empty");

        for cat in categories {
            let name = cat["name"].as_str().unwrap_or("unknown");
            assert!(
                cat.get("applicable").is_some(),
                "category '{}' missing 'applicable' field in JSON output",
                name
            );
            assert!(
                cat["applicable"].is_boolean(),
                "category '{}' 'applicable' field should be boolean",
                name
            );
        }
    }

    /// Issue #240: Verify that non-applicable categories are correctly marked
    #[test]
    fn test_format_json_non_applicable_category() {
        let mut categories = HashMap::new();
        categories.insert("Rust Tooling".to_string(), CategoryScore::new(20.0, 25.0));
        categories.insert(
            "GPU/SIMD".to_string(),
            CategoryScore::not_applicable(10.0),
        );

        let score = ProjectScore {
            total_earned: 20.0,
            total_possible: 35.0,
            percentage: 80.0,
            grade: crate::services::rust_project_score::models::Grade::BPlus,
            categories,
            recommendations: vec![],
        };

        let output = format_json(&score, &[], false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let cats = parsed["categories"].as_array().unwrap();

        let gpu_cat = cats
            .iter()
            .find(|c| c["name"].as_str() == Some("GPU/SIMD"))
            .expect("GPU/SIMD category should be in JSON output");
        assert_eq!(gpu_cat["applicable"].as_bool(), Some(false));

        let tooling_cat = cats
            .iter()
            .find(|c| c["name"].as_str() == Some("Rust Tooling"))
            .expect("Rust Tooling category should be in JSON output");
        assert_eq!(tooling_cat["applicable"].as_bool(), Some(true));
    }

    #[test]
    fn test_format_markdown_contains_header() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false, false);

        assert!(output.contains("# Rust Project Score"));
        assert!(output.contains(SPEC_VERSION));
    }

    #[test]
    fn test_format_markdown_contains_table() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false, false);

        // Should contain markdown table syntax
        assert!(output.contains("| Category | Score | Percentage |"));
        assert!(output.contains("|----------|-------|------------|"));
    }

    #[test]
    fn test_format_markdown_contains_categories() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false, false);

        assert!(output.contains("## Categories"));
        assert!(output.contains("Rust Tooling"));
        assert!(output.contains("Code Quality"));
    }

    #[test]
    fn test_format_markdown_recommendations_as_list() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false, false);

        assert!(output.contains("## Recommendations"));
        assert!(output.contains("- Add more tests"));
        assert!(output.contains("- Improve documentation"));
    }

    #[test]
    fn test_format_yaml_valid_yaml() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_yaml(&score, &recommendations, false).unwrap();

        // Should be valid YAML
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&output).unwrap();
        assert!(parsed.is_mapping());
    }

    #[test]
    fn test_format_yaml_contains_fields() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_yaml(&score, &recommendations, false).unwrap();

        assert!(output.contains("version:"));
        assert!(output.contains("total_earned:"));
        assert!(output.contains("total_possible:"));
        assert!(output.contains("percentage:"));
        assert!(output.contains("grade:"));
        assert!(output.contains("categories:"));
        assert!(output.contains("recommendations:"));
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_format_text_empty_categories() {
        let mut score = create_test_score();
        score.categories.clear();
        let output = format_text(&score, &[], false, false);

        // Should still have Categories section but no entries
        assert!(output.contains("Categories"));
    }

    #[test]
    fn test_format_json_empty_recommendations() {
        let mut score = create_test_score();
        score.recommendations.clear();
        let output = format_json(&score, &[], false).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let recommendations = parsed["recommendations"].as_array().unwrap();
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_format_markdown_empty_recommendations() {
        let mut score = create_test_score();
        score.recommendations.clear();
        let output = format_markdown(&score, &[], false, false);

        // Should not contain recommendations section when empty
        assert!(!output.contains("## 💡 Recommendations"));
    }

    // ── #687: `--format json` must be byte-reproducible ─────────────────────
    //
    // Observed defect: five runs on an UNCHANGED project produced two distinct
    // md5 sums, differing only in
    //   "percentage": 28.001373626373628  vs  28.001373626373624
    // because the categories live in a HashMap whose iteration order is
    // randomised per process. Each iteration below builds a *fresh* HashMap,
    // which gets its own RandomState and therefore its own iteration order —
    // the in-process stand-in for "run the binary again". A single run proves
    // nothing, so these run 10+ times.

    /// The eleven real pmat categories with values whose per-category
    /// percentages are non-terminating in binary, so their sum genuinely
    /// depends on the fold order.
    fn realistic_score() -> ProjectScore {
        use crate::services::rust_project_score::aggregation;

        let raw: [(&str, f64, f64); 11] = [
            ("Build Performance", 4.0, 15.0),
            ("Code Quality", 7.0, 26.0),
            ("Dependency Health", 5.0, 12.0),
            ("Documentation", 11.0, 15.0),
            ("Formal Verification", 1.0, 16.0),
            ("GPU/SIMD Quality", 3.0, 10.0),
            ("Known Defects", 13.0, 20.0),
            ("Performance & Benchmarking", 7.0, 10.0),
            ("Reproducibility", 2.0, 15.0),
            ("Rust Tooling & CI/CD", 91.0, 130.0),
            ("Testing Excellence", 3.0, 20.0),
        ];
        let categories: HashMap<String, CategoryScore> = raw
            .iter()
            .map(|(name, earned, max)| ((*name).to_string(), CategoryScore::new(*earned, *max)))
            .collect();

        let percentage = aggregation::normalized_percentage(&categories);
        ProjectScore {
            total_earned: aggregation::total_earned(&categories),
            total_possible: 289.0,
            percentage,
            grade: crate::services::rust_project_score::models::Grade::from_normalized(percentage),
            categories,
            recommendations: vec!["Add more tests".to_string()],
        }
    }

    #[test]
    fn test_format_json_is_byte_reproducible_across_12_runs() {
        let first = format_json(&realistic_score(), &["Add more tests".to_string()], false).unwrap();
        for i in 1..12 {
            let again = format_json(&realistic_score(), &["Add more tests".to_string()], false).unwrap();
            assert_eq!(
                first, again,
                "#687: --format json differed on run {i}; an unchanged project must \
                 produce byte-identical JSON so it can be a CI baseline"
            );
        }
    }

    #[test]
    fn test_format_yaml_is_byte_reproducible_across_12_runs() {
        let first = format_yaml(&realistic_score(), &[], false).unwrap();
        for i in 1..12 {
            assert_eq!(
                first,
                format_yaml(&realistic_score(), &[], false).unwrap(),
                "#687: --format yaml differed on run {i}"
            );
        }
    }

    #[test]
    fn test_format_json_categories_are_alphabetical() {
        // Random HashMap order must not leak into the emitted array.
        let output = format_json(&realistic_score(), &[], false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let names: Vec<String> = parsed["categories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect();
        let mut expected = names.clone();
        expected.sort();
        assert_eq!(names, expected, "#687: JSON category order must be sorted");
    }

    /// "Two renderers of the same command MUST NOT disagree about the same
    /// number" — json, yaml, text and markdown all read the same totals.
    #[test]
    fn test_renderers_agree_on_totals() {
        let score = realistic_score();
        let json: serde_json::Value =
            serde_json::from_str(&format_json(&score, &[], false).unwrap()).unwrap();
        let yaml: serde_json::Value =
            serde_yaml_ng::from_str(&format_yaml(&score, &[], false).unwrap()).unwrap();

        assert_eq!(json["total_earned"], yaml["total_earned"]);
        assert_eq!(json["total_possible"], yaml["total_possible"]);
        assert_eq!(json["percentage"], yaml["percentage"]);

        // Text and markdown print the same figures at 1 decimal place.
        let earned = json["total_earned"].as_f64().unwrap();
        let possible = json["total_possible"].as_f64().unwrap();
        let text = strip_ansi(&format_text(&score, &[], false, false));
        let markdown = format_markdown(&score, &[], false, false);
        assert!(
            text.contains(&format!("{earned:.1}")),
            "text output must show the same earned total: {text}"
        );
        assert!(
            markdown.contains(&format!("{earned:.1}/{possible:.0}")),
            "markdown output must show the same earned total: {markdown}"
        );
    }

    /// ARITHMETIC SANITY (round-3 sweep): `percentage` does NOT follow from the
    /// `total_earned` / `total_possible` printed beside it — it is the
    /// unweighted MEAN of the per-category percentages. On pmat's own tree json
    /// and yaml emitted `total_earned: 236.9`, `total_possible: 289.0`,
    /// `percentage: 87.22669`, while `236.9 / 289 * 100 = 81.972318`. Every
    /// category was `applicable: true`, so the documented "excludes categories
    /// that do not apply" caveat did not explain the gap. Only the text
    /// renderer admitted what the number was; markdown printed the two adjacent
    /// with no disclaimer and json/yaml carried no label at all.
    ///
    /// The document must now name both quantities and both must be checkable.
    #[test]
    fn every_renderer_says_which_percentage_it_is_showing() {
        let score = realistic_score();
        let json: serde_json::Value =
            serde_json::from_str(&format_json(&score, &[], false).unwrap()).unwrap();

        let earned = json["total_earned"].as_f64().unwrap();
        let possible = json["total_possible"].as_f64().unwrap();
        let percentage = json["percentage"].as_f64().unwrap();
        let points = json["points_percentage"].as_f64().unwrap();

        // The two are genuinely different numbers on this fixture, or the test
        // would pass without proving anything.
        assert!(
            (percentage - points).abs() > 1.0,
            "fixture must exercise the discrepancy: {percentage} vs {points}"
        );

        // `points_percentage` follows from the two totals beside it.
        assert!(
            (points - (earned / possible) * 100.0).abs() < 0.001,
            "points_percentage must equal total_earned/total_possible: \
             {points} vs {earned}/{possible}"
        );
        // And `percentage` is labelled as what it actually is.
        assert_eq!(
            json["percentage_basis"],
            serde_json::json!("mean of applicable category percentages")
        );
        // No percentage above 100.
        assert!((0.0..=100.0).contains(&percentage));
        assert!((0.0..=100.0).contains(&points));

        // yaml carries the same fields.
        let yaml: serde_json::Value =
            serde_yaml_ng::from_str(&format_yaml(&score, &[], false).unwrap()).unwrap();
        assert_eq!(yaml["points_percentage"], json["points_percentage"]);
        assert_eq!(yaml["percentage_basis"], json["percentage_basis"]);

        // markdown no longer prints the two adjacent without saying which is
        // which, and text names the points ratio too.
        let markdown = format_markdown(&score, &[], false, false);
        assert!(
            markdown.contains("mean of category percentages"),
            "markdown must label the percentage: {markdown}"
        );
        assert!(
            markdown.contains("of possible points"),
            "markdown must also give the points ratio: {markdown}"
        );
        let text = strip_ansi(&format_text(&score, &[], false, false));
        assert!(
            text.contains("of possible points"),
            "text must also give the points ratio: {text}"
        );
    }

    // ── #943: `--failures-only` was a provable no-op ────────────────────────
    //
    // Both arms of `if failures_only` in the handler returned the same
    // expression and the flag reached nothing else, so stdout was byte-
    // identical with and without it (md5 a44c0542 both ways on a real project)
    // while four passing categories were still printed with a ✓ and `--help`
    // promised "Show only failures and warnings".

    /// A score with both a passing and a failing category, so the filter has
    /// something to remove and something to keep.
    fn mixed_score() -> ProjectScore {
        use crate::services::rust_project_score::aggregation;

        let mut categories = HashMap::new();
        categories.insert("Documentation".to_string(), CategoryScore::new(15.0, 15.0));
        categories.insert("Known Defects".to_string(), CategoryScore::new(20.0, 20.0));
        categories.insert("Testing Excellence".to_string(), CategoryScore::new(2.0, 20.0));
        categories.insert(
            "GPU/SIMD Quality".to_string(),
            CategoryScore::not_applicable(10.0),
        );

        let percentage = aggregation::normalized_percentage(&categories);
        ProjectScore {
            total_earned: aggregation::applicable_earned(&categories),
            total_possible: aggregation::applicable_possible(&categories),
            percentage,
            grade: crate::services::rust_project_score::models::Grade::from_normalized(percentage),
            categories,
            recommendations: vec!["Add more tests".to_string()],
        }
    }

    #[test]
    fn test_failures_only_hides_passing_categories_in_text() {
        let score = mixed_score();
        let full = strip_ansi(&format_text(&score, &[], false, false));
        let filtered = strip_ansi(&format_text(&score, &[], false, true));

        assert_ne!(
            full, filtered,
            "--failures-only produced byte-identical output"
        );
        assert!(full.contains("Documentation"));
        assert!(
            !filtered.contains("Documentation"),
            "a 100% category must not survive --failures-only: {filtered}"
        );
        assert!(
            !filtered.contains("Known Defects"),
            "a 100% category must not survive --failures-only: {filtered}"
        );
        assert!(
            filtered.contains("Testing Excellence"),
            "the failing category must be kept: {filtered}"
        );
        // N/A is not a pass — hiding it would leave the denominator unexplained.
        assert!(filtered.contains("GPU/SIMD Quality"));
        assert!(
            filtered.contains("hidden by --failures-only"),
            "a filtered list must say what it omitted: {filtered}"
        );
    }

    /// The flag selects what is listed, never what is scored.
    #[test]
    fn test_failures_only_does_not_change_the_score() {
        let score = mixed_score();
        let full: serde_json::Value =
            serde_json::from_str(&format_json(&score, &[], false).unwrap()).unwrap();
        let filtered: serde_json::Value =
            serde_json::from_str(&format_json(&score, &[], true).unwrap()).unwrap();

        assert_eq!(full["total_earned"], filtered["total_earned"]);
        assert_eq!(full["total_possible"], filtered["total_possible"]);
        assert_eq!(full["percentage"], filtered["percentage"]);
        assert_eq!(full["points_percentage"], filtered["points_percentage"]);
        assert_eq!(full["grade"], filtered["grade"]);
    }

    #[test]
    fn test_failures_only_filters_every_renderer() {
        let score = mixed_score();

        let json: serde_json::Value =
            serde_json::from_str(&format_json(&score, &[], true).unwrap()).unwrap();
        let names: Vec<&str> = json["categories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap())
            .collect();
        assert!(!names.contains(&"Documentation"), "json: {names:?}");
        assert!(names.contains(&"Testing Excellence"), "json: {names:?}");
        assert_eq!(json["categories_omitted"].as_u64(), Some(2));
        assert_eq!(json["categories_filtered"].as_bool(), Some(true));

        let yaml: serde_json::Value =
            serde_yaml_ng::from_str(&format_yaml(&score, &[], true).unwrap()).unwrap();
        assert_eq!(yaml["categories"], json["categories"]);

        let markdown = format_markdown(&score, &[], false, true);
        assert!(!markdown.contains("Documentation"), "markdown: {markdown}");
        assert!(markdown.contains("Testing Excellence"), "markdown: {markdown}");
    }

    /// With nothing to hide, the flag must not change the output at all.
    #[test]
    fn test_failures_only_is_a_no_op_when_everything_fails() {
        let mut score = mixed_score();
        score.categories.clear();
        score
            .categories
            .insert("Testing Excellence".to_string(), CategoryScore::new(2.0, 20.0));

        assert_eq!(
            format_text(&score, &[], false, false),
            format_text(&score, &[], false, true)
        );
    }
}
