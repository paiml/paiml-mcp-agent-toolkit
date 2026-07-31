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
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Rust Project Score"));
        assert!(output.contains(SPEC_VERSION));
    }

    #[test]
    fn test_format_text_contains_summary() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Summary"));
        assert!(output.contains("Score:"));
        assert!(output.contains("Normalized:"));
        assert!(output.contains("Grade:"));
    }

    #[test]
    fn test_format_text_contains_categories() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Categories"));
        assert!(output.contains("Rust Tooling"));
        assert!(output.contains("Code Quality"));
        assert!(output.contains("Testing"));
    }

    #[test]
    fn test_format_text_contains_recommendations() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_text(&score, &recommendations, false);

        assert!(output.contains("Recommendations"));
        assert!(output.contains("Add more tests"));
        assert!(output.contains("Improve documentation"));
    }

    #[test]
    fn test_format_text_no_recommendations() {
        let mut score = create_test_score();
        score.recommendations = vec![];
        let output = format_text(&score, &[], false);

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
        let output = strip_ansi(&format_text(&score, &[], false));

        assert!(output.contains("\u{2713}")); // ✓ checkmark
    }

    #[test]
    fn test_format_text_icons_warning() {
        let mut score = create_test_score();
        score.categories.clear();
        score
            .categories
            .insert("Warning".to_string(), CategoryScore::new(75.0, 100.0));
        let output = strip_ansi(&format_text(&score, &[], false));

        assert!(output.contains("\u{26A0}")); // ⚠ warning sign
    }

    #[test]
    fn test_format_text_icons_failing() {
        let mut score = create_test_score();
        score.categories.clear();
        score
            .categories
            .insert("Failing".to_string(), CategoryScore::new(50.0, 100.0));
        let output = strip_ansi(&format_text(&score, &[], false));

        assert!(output.contains("\u{2717}")); // ✗ ballot x
    }

    #[test]
    fn test_format_json_valid_json() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_format_json_contains_fields() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations).unwrap();

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
        let output = format_json(&score, &recommendations).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_earned"].as_f64().unwrap(), 53.0);
        assert_eq!(parsed["percentage"].as_f64().unwrap(), 74.6);
    }

    /// Issue #240: Verify that `applicable` field is present on every category in JSON output
    #[test]
    fn test_format_json_categories_have_applicable_field() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_json(&score, &recommendations).unwrap();

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

        let output = format_json(&score, &[]).unwrap();
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
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("# Rust Project Score"));
        assert!(output.contains(SPEC_VERSION));
    }

    #[test]
    fn test_format_markdown_contains_table() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        // Should contain markdown table syntax
        assert!(output.contains("| Category | Score | Percentage |"));
        assert!(output.contains("|----------|-------|------------|"));
    }

    #[test]
    fn test_format_markdown_contains_categories() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("## Categories"));
        assert!(output.contains("Rust Tooling"));
        assert!(output.contains("Code Quality"));
    }

    #[test]
    fn test_format_markdown_recommendations_as_list() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("## Recommendations"));
        assert!(output.contains("- Add more tests"));
        assert!(output.contains("- Improve documentation"));
    }

    #[test]
    fn test_format_yaml_valid_yaml() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_yaml(&score, &recommendations).unwrap();

        // Should be valid YAML
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&output).unwrap();
        assert!(parsed.is_mapping());
    }

    #[test]
    fn test_format_yaml_contains_fields() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_yaml(&score, &recommendations).unwrap();

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
        let output = format_text(&score, &[], false);

        // Should still have Categories section but no entries
        assert!(output.contains("Categories"));
    }

    #[test]
    fn test_format_json_empty_recommendations() {
        let mut score = create_test_score();
        score.recommendations.clear();
        let output = format_json(&score, &[]).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let recommendations = parsed["recommendations"].as_array().unwrap();
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_format_markdown_empty_recommendations() {
        let mut score = create_test_score();
        score.recommendations.clear();
        let output = format_markdown(&score, &[], false);

        // Should not contain recommendations section when empty
        assert!(!output.contains("## 💡 Recommendations"));
    }

    /// GH #687: three runs over an unchanged project produced three different
    /// `categories` array orders because `ProjectScore::categories` is a
    /// HashMap. Rebuild the map repeatedly and require one order.
    #[test]
    fn json_categories_are_ordered_stably() {
        let names = || {
            let output = format_json(&create_test_score(), &[]).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
            parsed["categories"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["name"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };

        let expected = vec![
            "Code Quality".to_string(),
            "Rust Tooling".to_string(),
            "Testing".to_string(),
        ];
        for _ in 0..64 {
            assert_eq!(names(), expected, "category order must not vary between runs");
        }
    }

    /// The YAML renderer emitted "yet another order" per the issue; it must
    /// agree with JSON.
    #[test]
    fn yaml_categories_match_json_order() {
        let score = create_test_score();
        let json: serde_json::Value = serde_json::from_str(&format_json(&score, &[]).unwrap())
            .expect("json renderer output");
        let yaml: serde_json::Value = serde_yaml_ng::from_str(&format_yaml(&score, &[]).unwrap())
            .expect("yaml renderer output");
        assert_eq!(json["categories"], yaml["categories"]);
    }

    /// GH #685: `--help` advertised "0-106 scale" while the command emitted
    /// 279 and CLAUDE.md claimed 289. The help must state the scale the
    /// orchestrator actually has, and this test fails if either drifts.
    #[test]
    fn rust_project_score_scale_matches_help() {
        use crate::services::rust_project_score::orchestrator::RustProjectScoreOrchestrator;
        let max = RustProjectScoreOrchestrator::new().max_points();
        assert!(
            (max - 289.0).abs() < f64::EPSILON,
            "scorer maxima now sum to {max}; update the `rust-project-score` help text \
             in src/cli/commands/commands_enum/definition.rs to match"
        );
    }
}
