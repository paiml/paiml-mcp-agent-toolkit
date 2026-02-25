// Tests for repo-score handler
// Included from repo_score_handlers.rs — no `use` imports or inner attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::repo_score::{CategoryScore, CategoryScores, ScoreMetadata, ScoreStatus};
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create test RepoScore
    fn create_test_score(total_score: f64, grade: Grade) -> RepoScore {
        RepoScore {
            total_score,
            grade,
            categories: CategoryScores {
                documentation: CategoryScore {
                    score: 15.0,
                    max_score: 15.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                precommit_hooks: CategoryScore {
                    score: 20.0,
                    max_score: 20.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                repository_hygiene: CategoryScore {
                    score: 15.0,
                    max_score: 15.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                build_test_automation: CategoryScore {
                    score: 25.0,
                    max_score: 25.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                continuous_integration: CategoryScore {
                    score: 20.0,
                    max_score: 20.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
                pmat_compliance: CategoryScore {
                    score: 5.0,
                    max_score: 5.0,
                    percentage: 100.0,
                    status: ScoreStatus::Pass,
                    subcategories: vec![],
                    findings: vec![],
                },
            },
            recommendations: vec![],
            metadata: ScoreMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                repository_path: PathBuf::from("/tmp/test"),
                git_branch: Some("master".to_string()),
                git_commit: Some("test123".to_string()),
                pmat_version: env!("CARGO_PKG_VERSION").to_string(),
                spec_version: "1.0.0".to_string(),
                execution_time_ms: 0,
            },
        }
    }

    // ========================================================================
    // RED TEST 1: Badge Insertion in New README
    // ========================================================================

    #[test]
    fn test_badge_insertion_in_new_readme() {
        let temp_dir = TempDir::new().unwrap();
        let readme = temp_dir.path().join("README.md");
        fs::write(&readme, "# My Project\n\nDescription here.").unwrap();

        let score = create_test_score(104.0, Grade::APlus);
        update_readme_badge(temp_dir.path(), &score).unwrap();

        let content = fs::read_to_string(&readme).unwrap();

        // RED TEST ASSERTIONS (will FAIL until implementation correct)
        assert!(
            content.contains("<!-- PMAT-REPO-SCORE:START -->"),
            "Badge start marker not found"
        );
        assert!(
            content.contains("<!-- PMAT-REPO-SCORE:END -->"),
            "Badge end marker not found"
        );
        assert!(
            content.contains("repo%20health-104"),
            "Badge score not found in URL"
        );
        assert!(
            content.contains("brightgreen"),
            "Badge color not correct for A+ grade"
        );
        assert!(
            content.contains("A%2B"),
            "Badge grade not encoded properly (A+ → A%2B)"
        );
    }

    // ========================================================================
    // RED TEST 2: Badge Replacement in Existing README
    // ========================================================================

    #[test]
    fn test_badge_replacement_in_existing_readme() {
        let temp_dir = TempDir::new().unwrap();
        let readme = temp_dir.path().join("README.md");
        let initial = "# My Project\n\n<!-- PMAT-REPO-SCORE:START -->\n![Old Badge](https://img.shields.io/badge/repo%20health-50%2F125%20(F)-red)\n<!-- PMAT-REPO-SCORE:END -->\n\nDescription text.";
        fs::write(&readme, initial).unwrap();

        let score = create_test_score(99.0, Grade::A);
        update_readme_badge(temp_dir.path(), &score).unwrap();

        let content = fs::read_to_string(&readme).unwrap();

        // RED TEST ASSERTIONS
        assert!(
            content.contains("repo%20health-99"),
            "Badge not updated with new score"
        );
        assert!(
            content.contains("brightgreen"),
            "Badge color not updated for A grade"
        );
        assert!(
            !content.contains("repo%20health-50"),
            "Old badge score still present (replacement failed)"
        );
        assert!(
            content.matches("<!-- PMAT-REPO-SCORE:START -->").count() == 1,
            "Badge markers duplicated (should only appear once)"
        );
        assert!(
            content.contains("Description text."),
            "Badge update removed other content"
        );
    }

    // ========================================================================
    // RED TEST 3: Badge URL Generation with Different Grades
    // ========================================================================

    #[test]
    fn test_badge_url_generation_with_grades() {
        // Test A+ grade
        let score_a_plus = create_test_score(110.0, Grade::APlus);
        let url_a_plus = generate_badge_url(&score_a_plus);
        assert!(
            url_a_plus.contains("brightgreen"),
            "A+ should be brightgreen"
        );
        assert!(url_a_plus.contains("A%2B"), "A+ should be URL-encoded");

        // Test B grade
        let score_b = create_test_score(85.0, Grade::B);
        let url_b = generate_badge_url(&score_b);
        assert!(url_b.contains("yellow"), "B should be yellow");
        assert!(url_b.contains("repo%20health-85"), "Score should be 85");

        // Test C grade
        let score_c = create_test_score(75.0, Grade::C);
        let url_c = generate_badge_url(&score_c);
        assert!(url_c.contains("orange"), "C should be orange");

        // Test F grade
        let score_f = create_test_score(50.0, Grade::F);
        let url_f = generate_badge_url(&score_f);
        assert!(url_f.contains("red"), "F should be red");

        // Test URL format
        assert!(url_a_plus.starts_with("https://img.shields.io/badge/"));
        assert!(url_a_plus.contains("style=flat-square"));
    }

    // ========================================================================
    // RED TEST 4: Badge Insertion After Heading
    // ========================================================================

    #[test]
    fn test_badge_insertion_after_heading_skips_blank_lines() {
        let temp_dir = TempDir::new().unwrap();
        let readme = temp_dir.path().join("README.md");
        let initial = "# My Project\n\n\n\nFirst paragraph here.";
        fs::write(&readme, initial).unwrap();

        let score = create_test_score(95.0, Grade::A);
        update_readme_badge(temp_dir.path(), &score).unwrap();

        let content = fs::read_to_string(&readme).unwrap();

        // Badge should be inserted after blank lines following heading
        assert!(content.contains("# My Project"), "Heading should remain");
        assert!(
            content.contains("<!-- PMAT-REPO-SCORE:START -->"),
            "Badge should be inserted"
        );
        assert!(
            content.contains("First paragraph here."),
            "Original content should remain"
        );

        // Check ordering: heading → badge → content
        let heading_pos = content.find("# My Project").unwrap();
        let badge_pos = content.find("<!-- PMAT-REPO-SCORE:START -->").unwrap();
        let content_pos = content.find("First paragraph here.").unwrap();
        assert!(heading_pos < badge_pos, "Badge should come after heading");
        assert!(badge_pos < content_pos, "Badge should come before content");
    }

    // ========================================================================
    // RED TEST 5: Graceful Handling of Missing README
    // ========================================================================

    #[test]
    fn test_missing_readme_handled_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        // No README.md created

        let score = create_test_score(100.0, Grade::APlus);
        let result = update_readme_badge(temp_dir.path(), &score);

        // Should not error, just skip
        assert!(result.is_ok(), "Missing README should not cause error");
    }
}
