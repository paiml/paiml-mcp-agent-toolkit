impl ShowcaseGallery {
    #[must_use]
    pub fn new() -> Self {
        let mut gallery = Self {
            repositories: HashMap::new(),
            categories: HashMap::new(),
        };
        gallery.initialize_showcase_repositories();
        gallery
    }

    #[must_use]
    pub fn get_all_repositories(&self) -> Vec<&ShowcaseRepository> {
        self.repositories.values().collect()
    }

    #[must_use]
    pub fn get_repositories_by_category(
        &self,
        category: &RepositoryCategory,
    ) -> Vec<&ShowcaseRepository> {
        if let Some(repo_names) = self.categories.get(category) {
            repo_names
                .iter()
                .filter_map(|name| self.repositories.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    #[must_use]
    pub fn get_repositories_by_complexity(
        &self,
        tier: &ComplexityTier,
    ) -> Vec<&ShowcaseRepository> {
        self.repositories
            .values()
            .filter(|repo| repo.complexity_tier == *tier)
            .collect()
    }

    #[must_use]
    pub fn get_repositories_by_language(&self, language: &str) -> Vec<&ShowcaseRepository> {
        debug_assert!(!language.is_empty(), "language must not be empty");
        let lang_lower = language.to_lowercase();
        self.repositories
            .values()
            .filter(|repo| {
                repo.primary_language.to_lowercase() == lang_lower
                    || repo
                        .languages
                        .iter()
                        .any(|l| l.to_lowercase() == lang_lower)
            })
            .collect()
    }

    #[must_use]
    pub fn get_repository_by_name(&self, name: &str) -> Option<&ShowcaseRepository> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.repositories.get(name)
    }

    #[must_use]
    pub fn get_categories(&self) -> Vec<&RepositoryCategory> {
        self.categories.keys().collect()
    }

    #[must_use]
    pub fn get_quick_start_recommendations(&self) -> Vec<&ShowcaseRepository> {
        // Return beginner and intermediate repositories for quick starts
        self.repositories
            .values()
            .filter(|repo| {
                matches!(
                    repo.complexity_tier,
                    ComplexityTier::Beginner | ComplexityTier::Intermediate
                )
            })
            .take(4)
            .collect()
    }

    #[must_use]
    pub fn get_featured_repositories(&self) -> Vec<&ShowcaseRepository> {
        // Return a curated selection of featured repositories
        let featured_names = vec!["Tokio", "Django", "React", "VS Code"];
        featured_names
            .into_iter()
            .filter_map(|name| self.repositories.get(name))
            .collect()
    }

    #[must_use]
    pub fn generate_showcase_summary(&self) -> ShowcaseSummary {
        let total_repositories = self.repositories.len();
        let languages: std::collections::HashSet<String> = self
            .repositories
            .values()
            .flat_map(|repo| repo.languages.iter().cloned())
            .collect();

        let categories_count = self.categories.len();

        let complexity_distribution = {
            let mut distribution = HashMap::new();
            for repo in self.repositories.values() {
                *distribution.entry(repo.complexity_tier).or_insert(0) += 1;
            }
            distribution
        };

        ShowcaseSummary {
            total_repositories,
            total_languages: languages.len(),
            total_categories: categories_count,
            complexity_distribution,
            featured_count: 4,
            quick_start_count: self.get_quick_start_recommendations().len(),
        }
    }
}
