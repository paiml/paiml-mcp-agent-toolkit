#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcase_gallery_initialization() {
        let gallery = ShowcaseGallery::new();

        assert!(!gallery.repositories.is_empty());
        assert!(!gallery.categories.is_empty());

        // Should have repositories in different categories
        assert!(gallery.categories.len() >= 4);
    }

    #[test]
    fn test_repository_filtering() {
        let gallery = ShowcaseGallery::new();

        // Test filtering by language
        let rust_repos = gallery.get_repositories_by_language("rust");
        assert!(!rust_repos.is_empty());

        // Test filtering by complexity
        let beginner_repos = gallery.get_repositories_by_complexity(&ComplexityTier::Beginner);
        assert!(!beginner_repos.is_empty());

        // Test filtering by category
        let web_repos = gallery.get_repositories_by_category(&RepositoryCategory::WebFramework);
        assert!(!web_repos.is_empty());
    }

    #[test]
    fn test_showcase_recommendations() {
        let gallery = ShowcaseGallery::new();

        let quick_start = gallery.get_quick_start_recommendations();
        assert!(!quick_start.is_empty());
        assert!(quick_start.len() <= 4);

        let featured = gallery.get_featured_repositories();
        assert!(!featured.is_empty());
        assert_eq!(featured.len(), 4);
    }

    #[test]
    fn test_showcase_summary() {
        let gallery = ShowcaseGallery::new();
        let summary = gallery.generate_showcase_summary();

        assert!(summary.total_repositories > 0);
        assert!(summary.total_languages > 0);
        assert!(summary.total_categories > 0);
        assert!(!summary.complexity_distribution.is_empty());
    }

    #[test]
    fn test_repository_structure() {
        let gallery = ShowcaseGallery::new();

        if let Some(repo) = gallery.get_repository_by_name("Tokio") {
            assert_eq!(repo.name, "Tokio");
            assert_eq!(repo.primary_language, "rust");
            assert!(matches!(repo.complexity_tier, ComplexityTier::Advanced));
            assert!(repo.analysis_preview.is_some());
        } else {
            panic!("Tokio repository should be present in showcase");
        }
    }
}

