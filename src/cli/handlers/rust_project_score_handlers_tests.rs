// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::rust_project_score::models::CategoryScore;
    use crate::services::rust_project_score::orchestrator::{ProjectScore, SPEC_VERSION};
    use std::collections::HashMap;
    use tempfile::TempDir;

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
        let output = format_text(&score, &[], false);

        assert!(output.contains("✅"));
    }

    #[test]
    fn test_format_text_icons_warning() {
        let mut score = create_test_score();
        score.categories.clear();
        score
            .categories
            .insert("Warning".to_string(), CategoryScore::new(75.0, 100.0));
        let output = format_text(&score, &[], false);

        assert!(output.contains("⚠️"));
    }

    #[test]
    fn test_format_text_icons_failing() {
        let mut score = create_test_score();
        score.categories.clear();
        score
            .categories
            .insert("Failing".to_string(), CategoryScore::new(50.0, 100.0));
        let output = format_text(&score, &[], false);

        assert!(output.contains("❌"));
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

    #[test]
    fn test_format_markdown_contains_header() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("# 🦀 Rust Project Score"));
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

        assert!(output.contains("## 📂 Categories"));
        assert!(output.contains("Rust Tooling"));
        assert!(output.contains("Code Quality"));
    }

    #[test]
    fn test_format_markdown_recommendations_as_list() {
        let score = create_test_score();
        let recommendations = score.recommendations.clone();
        let output = format_markdown(&score, &recommendations, false);

        assert!(output.contains("## 💡 Recommendations"));
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
}
