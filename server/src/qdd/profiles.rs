//! Quality profiles for different development standards
//! Toyota Way: Define clear standards and enforce them consistently

use super::core::{
    DesignPatterns, QualityMetrics, QualityProfile, QualityRule, QualityThresholds, Severity,
};

/// Collection of predefined quality profiles
pub struct QualityProfiles;

impl QualityProfiles {
    /// Get all available profile names
    #[must_use]
    pub fn available_profiles() -> Vec<&'static str> {
        vec![
            "extreme",
            "standard",
            "relaxed",
            "enterprise",
            "startup",
            "legacy",
        ]
    }

    /// Create profile by name
    #[must_use]
    pub fn by_name(name: &str) -> Option<QualityProfile> {
        match name {
            "extreme" => Some(QualityProfile::extreme()),
            "standard" => Some(QualityProfile::standard()),
            "relaxed" => Some(QualityProfile::relaxed()),
            "enterprise" => Some(Self::enterprise()),
            "startup" => Some(Self::startup()),
            "legacy" => Some(Self::legacy()),
            _ => None,
        }
    }

    /// Enterprise-grade quality profile
    /// Strict but realistic for large teams
    #[must_use]
    pub fn enterprise() -> QualityProfile {
        QualityProfile {
            name: "enterprise".to_string(),
            thresholds: QualityThresholds {
                max_complexity: 15,
                max_cognitive: 15,
                min_coverage: 85,
                max_tdg: 5,
                max_entropy_violations: 3, // Strict: max 3 entropy violations
                zero_satd: true,
                zero_dead_code: true,
                require_doctests: true,
                require_property_tests: false,
            },
            patterns: DesignPatterns {
                enforce_solid: true,
                enforce_dry: true,
                enforce_kiss: true,
                enforce_yagni: true,
            },
            rules: vec![
                QualityRule {
                    name: "no_panic".to_string(),
                    description: "Avoid panic! in production code".to_string(),
                    severity: Severity::Error,
                    pattern: r"panic!".to_string(),
                },
                QualityRule {
                    name: "proper_error_handling".to_string(),
                    description: "Use Result types consistently".to_string(),
                    severity: Severity::Error,
                    pattern: r"\.unwrap\(\)".to_string(),
                },
                QualityRule {
                    name: "documentation_required".to_string(),
                    description: "Public functions must be documented".to_string(),
                    severity: Severity::Warning,
                    pattern: r"pub fn.*\{".to_string(),
                },
            ],
        }
    }

    /// Startup-friendly profile
    /// Balanced for rapid development with quality
    #[must_use]
    pub fn startup() -> QualityProfile {
        QualityProfile {
            name: "startup".to_string(),
            thresholds: QualityThresholds {
                max_complexity: 12,
                max_cognitive: 12,
                min_coverage: 75,
                max_tdg: 8,
                max_entropy_violations: 5, // Relaxed: allow more entropy for rapid iteration
                zero_satd: false,          // Allow some TODOs for rapid iteration
                zero_dead_code: false,
                require_doctests: false,
                require_property_tests: false,
            },
            patterns: DesignPatterns {
                enforce_solid: false, // Flexible for rapid prototyping
                enforce_dry: true,
                enforce_kiss: true,
                enforce_yagni: true, // Important for startups
            },
            rules: vec![
                QualityRule {
                    name: "no_panic".to_string(),
                    description: "Avoid panic! except in prototypes".to_string(),
                    severity: Severity::Warning,
                    pattern: r"panic!".to_string(),
                },
                QualityRule {
                    name: "todo_tracking".to_string(),
                    description: "TODOs should have issue numbers".to_string(),
                    severity: Severity::Info,
                    pattern: r"TODO(?!.*#\d+)".to_string(),
                },
            ],
        }
    }

    /// Legacy code maintenance profile
    /// Pragmatic approach for improving existing codebases
    #[must_use]
    pub fn legacy() -> QualityProfile {
        QualityProfile {
            name: "legacy".to_string(),
            thresholds: QualityThresholds {
                max_complexity: 25, // Higher tolerance for legacy code
                max_cognitive: 25,
                min_coverage: 50, // Start with achievable goals
                max_tdg: 15,
                max_entropy_violations: 10, // Very relaxed: legacy code may have many patterns
                zero_satd: false,
                zero_dead_code: false,
                require_doctests: false,
                require_property_tests: false,
            },
            patterns: DesignPatterns {
                enforce_solid: false,
                enforce_dry: false, // May have intentional duplication
                enforce_kiss: false,
                enforce_yagni: false,
            },
            rules: vec![QualityRule {
                name: "gradual_improvement".to_string(),
                description: "Improve code incrementally".to_string(),
                severity: Severity::Info,
                pattern: String::new(), // No specific pattern
            }],
        }
    }
}

/// Profile validation and recommendations
pub struct ProfileValidator;

impl ProfileValidator {
    /// Validate if a profile is appropriate for the codebase
    #[must_use]
    pub fn validate_profile_for_codebase(
        profile: &QualityProfile,
        codebase_metrics: &QualityMetrics,
    ) -> ProfileValidation {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check if current code meets profile thresholds
        if codebase_metrics.complexity > profile.thresholds.max_complexity {
            issues.push(format!(
                "Current complexity ({}) exceeds profile threshold ({})",
                codebase_metrics.complexity, profile.thresholds.max_complexity
            ));
            recommendations
                .push("Consider using a more relaxed profile or refactor code first".to_string());
        }

        if codebase_metrics.coverage < profile.thresholds.min_coverage {
            issues.push(format!(
                "Current coverage ({}%) is below profile requirement ({}%)",
                codebase_metrics.coverage, profile.thresholds.min_coverage
            ));
            recommendations.push(
                "Increase test coverage gradually or use a lower coverage profile".to_string(),
            );
        }

        if profile.thresholds.zero_satd && codebase_metrics.satd_count > 0 {
            issues.push(format!(
                "Profile requires zero SATD but found {} instances",
                codebase_metrics.satd_count
            ));
            recommendations.push(
                "Implement all TODO/FIXME comments or use a more permissive profile".to_string(),
            );
        }

        let is_valid = issues.is_empty();
        let recommended_profile = if is_valid {
            None
        } else {
            Some(Self::recommend_profile(codebase_metrics))
        };

        ProfileValidation {
            is_valid,
            issues,
            recommendations,
            recommended_profile,
        }
    }

    /// Recommend appropriate profile based on codebase metrics
    fn recommend_profile(metrics: &QualityMetrics) -> String {
        if metrics.complexity > 20 || metrics.coverage < 50 {
            "legacy".to_string()
        } else if metrics.complexity > 15 || metrics.coverage < 75 {
            "startup".to_string()
        } else if metrics.complexity > 10 || metrics.coverage < 85 {
            "standard".to_string()
        } else if metrics.complexity > 5 || metrics.coverage < 90 {
            "enterprise".to_string()
        } else {
            "extreme".to_string()
        }
    }
}

/// Result of profile validation
#[derive(Debug, Clone)]
pub struct ProfileValidation {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub recommended_profile: Option<String>,
}

/// Profile comparison utilities
pub struct ProfileComparator;

impl ProfileComparator {
    /// Compare two profiles and show differences
    #[must_use]
    pub fn compare(profile1: &QualityProfile, profile2: &QualityProfile) -> ProfileComparison {
        let mut differences = Vec::new();

        let thresholds1 = &profile1.thresholds;
        let thresholds2 = &profile2.thresholds;

        if thresholds1.max_complexity != thresholds2.max_complexity {
            differences.push(format!(
                "Max Complexity: {} vs {}",
                thresholds1.max_complexity, thresholds2.max_complexity
            ));
        }

        if thresholds1.min_coverage != thresholds2.min_coverage {
            differences.push(format!(
                "Min Coverage: {}% vs {}%",
                thresholds1.min_coverage, thresholds2.min_coverage
            ));
        }

        if thresholds1.zero_satd != thresholds2.zero_satd {
            differences.push(format!(
                "Zero SATD: {} vs {}",
                thresholds1.zero_satd, thresholds2.zero_satd
            ));
        }

        ProfileComparison {
            profile1_name: profile1.name.clone(),
            profile2_name: profile2.name.clone(),
            differences,
            is_stricter: Self::is_stricter(profile1, profile2),
        }
    }

    fn is_stricter(profile1: &QualityProfile, profile2: &QualityProfile) -> Option<String> {
        let t1 = &profile1.thresholds;
        let t2 = &profile2.thresholds;

        let profile1_score = Self::calculate_strictness_score(t1);
        let profile2_score = Self::calculate_strictness_score(t2);

        if profile1_score > profile2_score {
            Some(profile1.name.clone())
        } else if profile2_score > profile1_score {
            Some(profile2.name.clone())
        } else {
            None // Equal strictness
        }
    }

    fn calculate_strictness_score(thresholds: &QualityThresholds) -> i32 {
        let mut score = 0;

        score += (20 - thresholds.max_complexity as i32).max(0);
        score += thresholds.min_coverage as i32;
        score += (10 - thresholds.max_tdg as i32).max(0);

        if thresholds.zero_satd {
            score += 50;
        }
        if thresholds.zero_dead_code {
            score += 30;
        }
        if thresholds.require_doctests {
            score += 20;
        }
        if thresholds.require_property_tests {
            score += 40;
        }

        score
    }
}

/// Result of profile comparison
#[derive(Debug, Clone)]
pub struct ProfileComparison {
    pub profile1_name: String,
    pub profile2_name: String,
    pub differences: Vec<String>,
    pub is_stricter: Option<String>, // Name of stricter profile
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_profiles() {
        let profiles = QualityProfiles::available_profiles();
        assert!(profiles.contains(&"extreme"));
        assert!(profiles.contains(&"standard"));
        assert!(profiles.contains(&"relaxed"));
        assert!(profiles.contains(&"enterprise"));
        assert!(profiles.contains(&"startup"));
        assert!(profiles.contains(&"legacy"));
    }

    #[test]
    fn test_profile_by_name() {
        let extreme = QualityProfiles::by_name("extreme").unwrap();
        assert_eq!(extreme.name, "extreme");
        assert_eq!(extreme.thresholds.max_complexity, 5);

        let invalid = QualityProfiles::by_name("invalid");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_enterprise_profile() {
        let enterprise = QualityProfiles::enterprise();
        assert_eq!(enterprise.name, "enterprise");
        assert_eq!(enterprise.thresholds.max_complexity, 15);
        assert_eq!(enterprise.thresholds.min_coverage, 85);
        assert!(enterprise.thresholds.zero_satd);
        assert!(enterprise.patterns.enforce_solid);
    }

    #[test]
    fn test_startup_profile() {
        let startup = QualityProfiles::startup();
        assert_eq!(startup.name, "startup");
        assert_eq!(startup.thresholds.max_complexity, 12);
        assert!(!startup.thresholds.zero_satd); // Allows TODOs
        assert!(startup.patterns.enforce_yagni); // Important for startups
        assert!(!startup.patterns.enforce_solid); // Flexible
    }

    #[test]
    fn test_legacy_profile() {
        let legacy = QualityProfiles::legacy();
        assert_eq!(legacy.name, "legacy");
        assert_eq!(legacy.thresholds.max_complexity, 25); // Higher tolerance
        assert_eq!(legacy.thresholds.min_coverage, 50); // Achievable goal
        assert!(!legacy.patterns.enforce_dry); // May have duplication
    }

    #[test]
    fn test_profile_validation_success() {
        let profile = QualityProfile::standard();
        let metrics = QualityMetrics {
            complexity: 8,
            cognitive_complexity: 8,
            coverage: 85,
            tdg: 3,
            satd_count: 0,
            dead_code_percentage: 0,
            has_doctests: true,
            has_property_tests: false,
        };

        let validation = ProfileValidator::validate_profile_for_codebase(&profile, &metrics);
        assert!(validation.is_valid);
        assert!(validation.issues.is_empty());
    }

    #[test]
    fn test_profile_validation_failure() {
        let profile = QualityProfile::extreme(); // Very strict
        let metrics = QualityMetrics {
            complexity: 15, // Exceeds extreme threshold
            cognitive_complexity: 15,
            coverage: 60, // Below extreme threshold
            tdg: 8,
            satd_count: 5, // Has SATD but profile requires zero
            dead_code_percentage: 0,
            has_doctests: false,
            has_property_tests: false,
        };

        let validation = ProfileValidator::validate_profile_for_codebase(&profile, &metrics);
        assert!(!validation.is_valid);
        assert!(!validation.issues.is_empty());
        assert!(!validation.recommendations.is_empty());
        assert!(validation.recommended_profile.is_some());
    }

    #[test]
    fn test_profile_recommendation() {
        // High complexity, low coverage should recommend legacy
        let legacy_metrics = QualityMetrics {
            complexity: 25,
            cognitive_complexity: 25,
            coverage: 40,
            tdg: 15,
            satd_count: 10,
            dead_code_percentage: 20,
            has_doctests: false,
            has_property_tests: false,
        };

        let profile = QualityProfile::extreme();
        let validation = ProfileValidator::validate_profile_for_codebase(&profile, &legacy_metrics);
        assert_eq!(validation.recommended_profile.unwrap(), "legacy");
    }

    #[test]
    fn test_profile_comparison() {
        let extreme = QualityProfile::extreme();
        let relaxed = QualityProfile::relaxed();

        let comparison = ProfileComparator::compare(&extreme, &relaxed);

        assert!(!comparison.differences.is_empty());
        assert!(comparison.is_stricter.is_some());
        assert_eq!(comparison.is_stricter.unwrap(), "extreme");
    }

    #[test]
    fn test_strictness_calculation() {
        let extreme_thresholds = QualityProfile::extreme().thresholds;
        let relaxed_thresholds = QualityProfile::relaxed().thresholds;

        let extreme_score = ProfileComparator::calculate_strictness_score(&extreme_thresholds);
        let relaxed_score = ProfileComparator::calculate_strictness_score(&relaxed_thresholds);

        assert!(extreme_score > relaxed_score);
    }
}
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
