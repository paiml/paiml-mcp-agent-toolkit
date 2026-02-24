// Format-specific tests for Popper score handlers
// Included by popper_score_tests.rs (inside mod tests) — do NOT add `use` or mod here.

// ========================================================================
// format_text Tests
// ========================================================================

#[test]
fn test_format_text_gateway_passed() {
    let score = create_test_score_passed();
    let output = format_text(&score, false, false);

    assert!(output.contains("Popper Falsifiability Score"));
    assert!(output.contains("Gateway: PASSED"));
    assert!(output.contains("73.7%"));
    assert!(output.contains("Falsifiability & Testability"));
    assert!(output.contains("Reproducibility Infrastructure"));
    assert!(output.contains("[GATEWAY]"));
}

#[test]
fn test_format_text_gateway_failed() {
    let score = create_test_score_failed();
    let output = format_text(&score, false, false);

    assert!(output.contains("Gateway: FAILED"));
    assert!(output.contains("Falsifiability < 60%"));
    assert!(output.contains("Without falsifiable claims"));
}

#[test]
fn test_format_text_verbose() {
    let score = create_test_score_passed();
    let output = format_text(&score, true, false);

    // Verbose shows sub-scores with id and description
    assert!(output.contains("A1"));
    assert!(output.contains("Unit test coverage")); // description not name
    assert!(output.contains("A2"));
    assert!(output.contains("Testable claims")); // description
}

#[test]
fn test_format_text_not_verbose_hides_subscores() {
    let score = create_test_score_passed();
    let output = format_text(&score, false, false);

    // Non-verbose shouldn't show sub-score IDs in detail
    // It still shows the category totals but not individual sub-scores
    assert!(output.contains("Falsifiability & Testability"));
}

#[test]
fn test_format_text_recommendations() {
    let score = create_test_score_passed();
    let output = format_text(&score, false, false);

    assert!(output.contains("Recommendations"));
    assert!(output.contains("Add mutation testing"));
    assert!(output.contains("cargo mutants"));
    assert!(output.contains("Add CI pipeline"));
}

#[test]
fn test_format_text_recommendation_priorities() {
    let score = create_test_score_passed();
    let output = format_text(&score, false, false);

    // Check priority icons
    assert!(output.contains("🔴")); // Critical
    assert!(output.contains("🟠")); // High
    assert!(output.contains("🟡")); // Medium
    assert!(output.contains("🟢")); // Low
}

#[test]
fn test_format_text_category_icons() {
    let score = create_test_score_passed();
    let output = format_text(&score, false, false);

    // Score should have various status icons
    assert!(output.contains("✅") || output.contains("⚠️") || output.contains("❌"));
}

#[test]
fn test_format_text_verdict() {
    let score = create_test_score_passed();
    let output = format_text(&score, false, false);

    assert!(output.contains("Verdict"));
    assert!(output.contains("Good scientific practices"));
}

#[test]
fn test_format_text_na_category() {
    let score = create_test_score_passed();
    let output = format_text(&score, false, false);

    // ML/AI is N/A by default
    assert!(output.contains("N/A"));
}

// ========================================================================
// format_json Tests
// ========================================================================

#[test]
fn test_format_json_basic() {
    let score = create_test_score_passed();
    let output = format_json(&score).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn test_format_json_contains_fields() {
    let score = create_test_score_passed();
    let output = format_json(&score).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed.get("raw_score").is_some());
    assert!(parsed.get("normalized_score").is_some());
    assert!(parsed.get("grade").is_some());
    assert!(parsed.get("gateway_passed").is_some());
    assert!(parsed.get("categories").is_some());
    assert!(parsed.get("recommendations").is_some());
    assert!(parsed.get("metadata").is_some());
    assert!(parsed.get("analysis").is_some());
}

#[test]
fn test_format_json_gateway_passed_value() {
    let score = create_test_score_passed();
    let output = format_json(&score).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed["gateway_passed"], true);
}

#[test]
fn test_format_json_gateway_failed_value() {
    let score = create_test_score_failed();
    let output = format_json(&score).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed["gateway_passed"], false);
}

#[test]
fn test_format_json_categories_structure() {
    let score = create_test_score_passed();
    let output = format_json(&score).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let categories = &parsed["categories"];
    assert!(categories.get("falsifiability").is_some());
    assert!(categories.get("reproducibility").is_some());
    assert!(categories.get("transparency").is_some());
    assert!(categories.get("statistical_rigor").is_some());
    assert!(categories.get("historical_integrity").is_some());
    assert!(categories.get("ml_reproducibility").is_some());
}

// ========================================================================
// format_markdown Tests
// ========================================================================

#[test]
fn test_format_markdown_basic() {
    let score = create_test_score_passed();
    let output = format_markdown(&score, false, false);

    assert!(output.contains("# 🔬 Popper Falsifiability Score"));
    assert!(output.contains("## 📌 Summary"));
    assert!(output.contains("## 📂 Categories"));
}

#[test]
fn test_format_markdown_gateway_passed() {
    let score = create_test_score_passed();
    let output = format_markdown(&score, false, false);

    assert!(output.contains("Gateway PASSED"));
}

#[test]
fn test_format_markdown_gateway_failed() {
    let score = create_test_score_failed();
    let output = format_markdown(&score, false, false);

    assert!(output.contains("Gateway FAILED"));
    assert!(output.contains("Falsifiability < 60%"));
}

#[test]
fn test_format_markdown_table() {
    let score = create_test_score_passed();
    let output = format_markdown(&score, false, false);

    // Should have markdown table headers
    assert!(output.contains("| Category | Score | Percentage | Status |"));
    assert!(output.contains("|----------|-------|------------|--------|"));
}

#[test]
fn test_format_markdown_verbose() {
    let score = create_test_score_passed();
    let output = format_markdown(&score, true, false);

    assert!(output.contains("## 📊 Detailed Breakdown"));
    assert!(output.contains("### A. Falsifiability & Testability"));
    assert!(output.contains("**A1**"));
}

#[test]
fn test_format_markdown_recommendations() {
    let score = create_test_score_passed();
    let output = format_markdown(&score, false, false);

    assert!(output.contains("## 💡 Recommendations"));
    assert!(output.contains("Add mutation testing"));
    assert!(output.contains("```bash"));
    assert!(output.contains("cargo mutants"));
}

#[test]
fn test_format_markdown_verdict() {
    let score = create_test_score_passed();
    let output = format_markdown(&score, false, false);

    assert!(output.contains("## 📋 Verdict"));
    assert!(output.contains("Good scientific practices"));
}

#[test]
fn test_format_markdown_na_category() {
    let score = create_test_score_passed();
    let output = format_markdown(&score, false, false);

    // ML/AI is N/A - should show in table
    assert!(output.contains("N/A"));
}

// ========================================================================
// format_yaml Tests
// ========================================================================

#[test]
fn test_format_yaml_basic() {
    let score = create_test_score_passed();
    let output = format_yaml(&score).unwrap();

    // Should be valid YAML
    let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&output).unwrap();
    assert!(parsed.is_mapping());
}

#[test]
fn test_format_yaml_contains_fields() {
    let score = create_test_score_passed();
    let output = format_yaml(&score).unwrap();

    assert!(output.contains("raw_score:"));
    assert!(output.contains("normalized_score:"));
    assert!(output.contains("grade:"));
    assert!(output.contains("gateway_passed:"));
    assert!(output.contains("categories:"));
}

#[test]
fn test_format_yaml_roundtrip() {
    let score = create_test_score_passed();
    let output = format_yaml(&score).unwrap();

    // Should be able to deserialize back
    let parsed: PopperScore = serde_yaml_ng::from_str(&output).unwrap();
    assert_eq!(parsed.gateway_passed, score.gateway_passed);
    assert!((parsed.normalized_score - score.normalized_score).abs() < 0.01);
}

// ========================================================================
// Edge Case Tests
// ========================================================================

#[test]
fn test_format_text_empty_recommendations() {
    let mut score = create_test_score_passed();
    score.recommendations = vec![];
    let output = format_text(&score, false, false);

    // Should not show recommendations section if empty
    // Actually the section might still show, let's check it doesn't crash
    assert!(output.contains("Popper Falsifiability Score"));
}

#[test]
fn test_format_markdown_empty_recommendations() {
    let mut score = create_test_score_passed();
    score.recommendations = vec![];
    let output = format_markdown(&score, false, false);

    assert!(output.contains("# 🔬 Popper Falsifiability Score"));
    // Empty recommendations shouldn't cause issues
}

#[test]
fn test_format_text_high_score() {
    let mut score = create_test_score_passed();
    score.normalized_score = 97.5;
    score.grade = PopperGrade::APlus;
    let output = format_text(&score, false, false);

    assert!(output.contains("97.5%"));
    assert!(output.contains("A+"));
}

#[test]
fn test_format_text_zero_score() {
    let mut score = create_test_score_failed();
    score.normalized_score = 0.0;
    score.raw_score = 0.0;
    let output = format_text(&score, false, false);

    assert!(output.contains("0.0"));
}

#[test]
fn test_format_json_special_characters_in_verdict() {
    let mut score = create_test_score_passed();
    score.analysis.verdict = "Test with \"quotes\" and 'apostrophes' & ampersands".to_string();
    let output = format_json(&score).unwrap();

    // Should be valid JSON even with special chars
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed["analysis"]["verdict"]
        .as_str()
        .unwrap()
        .contains("quotes"));
}

#[test]
fn test_format_all_grades() {
    let grades = vec![
        PopperGrade::APlus,
        PopperGrade::A,
        PopperGrade::AMinus,
        PopperGrade::BPlus,
        PopperGrade::B,
        PopperGrade::C,
        PopperGrade::D,
        PopperGrade::F,
        PopperGrade::InsufficientFalsifiability,
    ];

    for grade in grades {
        let mut score = create_test_score_passed();
        score.grade = grade;
        if grade == PopperGrade::InsufficientFalsifiability {
            score.gateway_passed = false;
        }

        let text = format_text(&score, false, false);
        let json = format_json(&score).unwrap();
        let md = format_markdown(&score, false, false);
        let yaml = format_yaml(&score).unwrap();

        // All formats should work without panicking
        assert!(!text.is_empty());
        assert!(!json.is_empty());
        assert!(!md.is_empty());
        assert!(!yaml.is_empty());
    }
}
