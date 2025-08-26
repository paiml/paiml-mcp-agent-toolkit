use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRecommendation {
    pub repository: String,
    pub reason: String,
    pub confidence: f64,
    pub framework_detected: Option<String>,
    pub complexity_tier: ComplexityTier,
    pub learning_focus: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityTier {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkSignature {
    pub name: String,
    pub files: Vec<String>,
    pub imports: Vec<String>,
    pub dependencies: Vec<String>,
    pub confidence_threshold: f64,
}

pub struct RecommendationEngine {
    framework_signatures: HashMap<String, Vec<FrameworkSignature>>,
    curated_repositories: HashMap<String, Vec<RepositoryRecommendation>>,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            framework_signatures: HashMap::new(),
            curated_repositories: HashMap::new(),
        };
        engine.initialize_framework_signatures();
        engine.initialize_curated_repositories();
        engine
    }

    fn initialize_framework_signatures(&mut self) {
        let rust_frameworks = vec![
            FrameworkSignature {
                name: "Actix Web".to_string(),
                files: vec!["Cargo.toml".to_string()],
                imports: vec!["actix_web".to_string(), "actix".to_string()],
                dependencies: vec!["actix-web".to_string()],
                confidence_threshold: 0.8,
            },
            FrameworkSignature {
                name: "Tokio".to_string(),
                files: vec!["Cargo.toml".to_string()],
                imports: vec!["tokio".to_string()],
                dependencies: vec!["tokio".to_string()],
                confidence_threshold: 0.7,
            },
            FrameworkSignature {
                name: "Serde".to_string(),
                files: vec!["Cargo.toml".to_string()],
                imports: vec!["serde".to_string()],
                dependencies: vec!["serde".to_string()],
                confidence_threshold: 0.6,
            },
        ];

        let python_frameworks = vec![
            FrameworkSignature {
                name: "Django".to_string(),
                files: vec!["manage.py".to_string(), "settings.py".to_string()],
                imports: vec!["django".to_string()],
                dependencies: vec!["Django".to_string()],
                confidence_threshold: 0.9,
            },
            FrameworkSignature {
                name: "FastAPI".to_string(),
                files: vec!["requirements.txt".to_string(), "pyproject.toml".to_string()],
                imports: vec!["fastapi".to_string()],
                dependencies: vec!["fastapi".to_string()],
                confidence_threshold: 0.8,
            },
            FrameworkSignature {
                name: "Flask".to_string(),
                files: vec!["app.py".to_string(), "requirements.txt".to_string()],
                imports: vec!["flask".to_string()],
                dependencies: vec!["Flask".to_string()],
                confidence_threshold: 0.7,
            },
        ];

        let typescript_frameworks = vec![
            FrameworkSignature {
                name: "React".to_string(),
                files: vec!["package.json".to_string()],
                imports: vec!["react".to_string()],
                dependencies: vec!["react".to_string()],
                confidence_threshold: 0.8,
            },
            FrameworkSignature {
                name: "Next.js".to_string(),
                files: vec!["next.config.js".to_string(), "package.json".to_string()],
                imports: vec!["next".to_string()],
                dependencies: vec!["next".to_string()],
                confidence_threshold: 0.9,
            },
            FrameworkSignature {
                name: "Express".to_string(),
                files: vec!["package.json".to_string()],
                imports: vec!["express".to_string()],
                dependencies: vec!["express".to_string()],
                confidence_threshold: 0.7,
            },
        ];

        self.framework_signatures
            .insert("rust".to_string(), rust_frameworks);
        self.framework_signatures
            .insert("python".to_string(), python_frameworks);
        self.framework_signatures
            .insert("typescript".to_string(), typescript_frameworks.clone());
        self.framework_signatures
            .insert("javascript".to_string(), typescript_frameworks);
    }

    fn initialize_curated_repositories(&mut self) {
        let rust_repos = vec![
            RepositoryRecommendation {
                repository: "tokio-rs/tokio".to_string(),
                reason: "Excellent async runtime showcasing advanced Rust patterns".to_string(),
                confidence: 0.95,
                framework_detected: Some("Tokio".to_string()),
                complexity_tier: ComplexityTier::Advanced,
                learning_focus: vec![
                    "async programming".to_string(),
                    "runtime design".to_string(),
                ],
            },
            RepositoryRecommendation {
                repository: "actix/actix-web".to_string(),
                reason: "High-performance web framework with clean architecture".to_string(),
                confidence: 0.90,
                framework_detected: Some("Actix Web".to_string()),
                complexity_tier: ComplexityTier::Intermediate,
                learning_focus: vec!["web development".to_string(), "performance".to_string()],
            },
            RepositoryRecommendation {
                repository: "clap-rs/clap".to_string(),
                reason: "Well-documented CLI library with excellent API design".to_string(),
                confidence: 0.85,
                framework_detected: None,
                complexity_tier: ComplexityTier::Beginner,
                learning_focus: vec!["CLI design".to_string(), "API ergonomics".to_string()],
            },
        ];

        let python_repos = vec![
            RepositoryRecommendation {
                repository: "tiangolo/fastapi".to_string(),
                reason: "Modern, fast web framework with excellent type hints".to_string(),
                confidence: 0.95,
                framework_detected: Some("FastAPI".to_string()),
                complexity_tier: ComplexityTier::Intermediate,
                learning_focus: vec!["API design".to_string(), "type hints".to_string()],
            },
            RepositoryRecommendation {
                repository: "pallets/flask".to_string(),
                reason: "Lightweight web framework perfect for learning".to_string(),
                confidence: 0.85,
                framework_detected: Some("Flask".to_string()),
                complexity_tier: ComplexityTier::Beginner,
                learning_focus: vec!["web basics".to_string(), "microframework".to_string()],
            },
            RepositoryRecommendation {
                repository: "encode/starlette".to_string(),
                reason: "ASGI framework demonstrating async patterns".to_string(),
                confidence: 0.80,
                framework_detected: None,
                complexity_tier: ComplexityTier::Advanced,
                learning_focus: vec!["ASGI".to_string(), "async patterns".to_string()],
            },
        ];

        let typescript_repos = vec![
            RepositoryRecommendation {
                repository: "facebook/react".to_string(),
                reason: "Industry-standard UI library with excellent patterns".to_string(),
                confidence: 0.95,
                framework_detected: Some("React".to_string()),
                complexity_tier: ComplexityTier::Intermediate,
                learning_focus: vec!["UI patterns".to_string(), "component design".to_string()],
            },
            RepositoryRecommendation {
                repository: "vercel/next.js".to_string(),
                reason: "Full-stack React framework with modern features".to_string(),
                confidence: 0.90,
                framework_detected: Some("Next.js".to_string()),
                complexity_tier: ComplexityTier::Advanced,
                learning_focus: vec!["full-stack".to_string(), "SSR".to_string()],
            },
            RepositoryRecommendation {
                repository: "expressjs/express".to_string(),
                reason: "Minimal web framework for Node.js".to_string(),
                confidence: 0.85,
                framework_detected: Some("Express".to_string()),
                complexity_tier: ComplexityTier::Beginner,
                learning_focus: vec!["Node.js".to_string(), "web server".to_string()],
            },
        ];

        self.curated_repositories
            .insert("rust".to_string(), rust_repos);
        self.curated_repositories
            .insert("python".to_string(), python_repos);
        self.curated_repositories
            .insert("typescript".to_string(), typescript_repos.clone());
        self.curated_repositories
            .insert("javascript".to_string(), typescript_repos);
    }

    pub fn analyze_repository(
        &self,
        path: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut detected_frameworks = Vec::new();

        for (language, signatures) in &self.framework_signatures {
            for signature in signatures {
                let confidence = self.calculate_framework_confidence(path, signature)?;
                if confidence >= signature.confidence_threshold {
                    detected_frameworks.push(format!(
                        "{}: {} (confidence: {:.2})",
                        language, signature.name, confidence
                    ));
                }
            }
        }

        Ok(detected_frameworks)
    }

    fn calculate_framework_confidence(
        &self,
        _path: &str,
        signature: &FrameworkSignature,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let confidence = 0.0;
        let mut total_checks = 0.0;

        total_checks += signature.files.len() as f64;
        total_checks += signature.imports.len() as f64;
        total_checks += signature.dependencies.len() as f64;

        if total_checks == 0.0 {
            return Ok(0.0);
        }

        Ok(confidence / total_checks)
    }

    pub fn get_recommendations(
        &self,
        detected_language: &str,
        complexity_preference: Option<ComplexityTier>,
    ) -> Vec<RepositoryRecommendation> {
        let language_key = detected_language.to_lowercase();

        if let Some(repos) = self.curated_repositories.get(&language_key) {
            let mut recommendations = repos.clone();

            if let Some(pref) = complexity_preference {
                recommendations.retain(|repo| {
                    matches!(
                        (&repo.complexity_tier, &pref),
                        (ComplexityTier::Beginner, ComplexityTier::Beginner)
                            | (ComplexityTier::Intermediate, ComplexityTier::Intermediate)
                            | (ComplexityTier::Advanced, ComplexityTier::Advanced)
                            | (ComplexityTier::Expert, ComplexityTier::Expert)
                    )
                });
            }

            recommendations.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            recommendations
        } else {
            Vec::new()
        }
    }

    pub fn get_framework_specific_recommendations(
        &self,
        framework: &str,
    ) -> Vec<RepositoryRecommendation> {
        let mut recommendations = Vec::new();

        for repos in self.curated_repositories.values() {
            for repo in repos {
                if let Some(detected_framework) = &repo.framework_detected {
                    if detected_framework
                        .to_lowercase()
                        .contains(&framework.to_lowercase())
                    {
                        recommendations.push(repo.clone());
                    }
                }
            }
        }

        recommendations.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        recommendations
    }

    pub fn generate_learning_path(
        &self,
        language: &str,
        detected_frameworks: &[String],
    ) -> Vec<String> {
        let mut learning_path = Vec::new();

        learning_path.push(format!("Start with basic {} syntax and concepts", language));

        for framework_info in detected_frameworks {
            if let Some(colon_pos) = framework_info.find(':') {
                let framework = framework_info[colon_pos + 1..]
                    .split(" (")
                    .next()
                    .unwrap_or("")
                    .trim();
                learning_path.push(format!("Study {} framework patterns", framework));
            }
        }

        learning_path.push("Explore advanced patterns and best practices".to_string());
        learning_path.push("Contribute to open source projects".to_string());

        learning_path
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_signature_creation() {
        let signature = FrameworkSignature {
            name: "Test Framework".to_string(),
            files: vec!["test.config".to_string()],
            imports: vec!["test_lib".to_string()],
            dependencies: vec!["test-dep".to_string()],
            confidence_threshold: 0.8,
        };

        assert_eq!(signature.name, "Test Framework");
        assert_eq!(signature.confidence_threshold, 0.8);
    }

    #[test]
    fn test_recommendation_engine_initialization() {
        let engine = RecommendationEngine::new();
        assert!(!engine.framework_signatures.is_empty());
        assert!(!engine.curated_repositories.is_empty());
    }

    #[test]
    fn test_get_rust_recommendations() {
        let engine = RecommendationEngine::new();
        let recommendations = engine.get_recommendations("rust", None);
        assert!(!recommendations.is_empty());

        let has_tokio = recommendations
            .iter()
            .any(|r| r.repository.contains("tokio"));
        assert!(has_tokio);
    }

    #[test]
    fn test_complexity_tier_filtering() {
        let engine = RecommendationEngine::new();
        let beginner_recs = engine.get_recommendations("rust", Some(ComplexityTier::Beginner));

        for rec in beginner_recs {
            assert!(matches!(rec.complexity_tier, ComplexityTier::Beginner));
        }
    }

    #[test]
    fn test_framework_specific_recommendations() {
        let engine = RecommendationEngine::new();
        let react_recs = engine.get_framework_specific_recommendations("React");

        assert!(!react_recs.is_empty());
        for rec in react_recs {
            if let Some(framework) = &rec.framework_detected {
                assert!(framework.to_lowercase().contains("react"));
            }
        }
    }

    #[test]
    fn test_learning_path_generation() {
        let engine = RecommendationEngine::new();
        let detected_frameworks = vec!["rust: Tokio (confidence: 0.85)".to_string()];
        let learning_path = engine.generate_learning_path("rust", &detected_frameworks);

        assert!(!learning_path.is_empty());
        assert!(learning_path[0].contains("basic rust"));
        assert!(learning_path.iter().any(|step| step.contains("Tokio")));
    }
}
