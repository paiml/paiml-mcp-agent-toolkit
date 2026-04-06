// RepoScore: NormalizedScore trait impl
impl NormalizedScore for RepoScore {
    fn raw(&self) -> f64 {
        self.total_score
    }

    fn max_raw(&self) -> f64 {
        REPO_SCORE_MAX_POINTS
    }
}

// RepoScore: Display impl
impl fmt::Display for RepoScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Repo Score: {:.1}/100 ({}) [raw: {:.1}/{}]",
            self.normalized(),
            self.grade.as_str(),
            self.total_score,
            REPO_SCORE_MAX_POINTS as u32
        )
    }
}

// Grade: conversion and formatting methods
impl Grade {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 95.0 => Grade::APlus,
            s if s >= 90.0 => Grade::A,
            s if s >= 85.0 => Grade::AMinus,
            s if s >= 80.0 => Grade::BPlus,
            s if s >= 70.0 => Grade::B,
            s if s >= 60.0 => Grade::C,
            s if s >= 50.0 => Grade::D,
            _ => Grade::F,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::APlus => "A+",
            Grade::A => "A",
            Grade::AMinus => "A-",
            Grade::BPlus => "B+",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::F => "F",
        }
    }
}

// CategoryScores: aggregate scoring
impl CategoryScores {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total(&self) -> f64 {
        self.documentation.score
            + self.precommit_hooks.score
            + self.repository_hygiene.score
            + self.build_test_automation.score
            + self.continuous_integration.score
            + self.pmat_compliance.score
    }
}

impl Default for CategoryScores {
    fn default() -> Self {
        Self {
            documentation: CategoryScore::default_with_max(15.0),
            precommit_hooks: CategoryScore::default_with_max(20.0),
            repository_hygiene: CategoryScore::default_with_max(15.0),
            build_test_automation: CategoryScore::default_with_max(25.0),
            continuous_integration: CategoryScore::default_with_max(20.0),
            pmat_compliance: CategoryScore::default_with_max(5.0),
        }
    }
}

// CategoryScore: construction and defaults
impl CategoryScore {
    fn default_with_max(max_score: f64) -> Self {
        Self {
            score: 0.0,
            max_score,
            percentage: 0.0,
            status: ScoreStatus::Fail,
            subcategories: vec![],
            findings: vec![],
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(
        score: f64,
        max_score: f64,
        subcategories: Vec<SubcategoryScore>,
        findings: Vec<Finding>,
    ) -> Self {
        let percentage = if max_score > 0.0 {
            (score / max_score) * 100.0
        } else {
            0.0
        };

        let status = if percentage >= 90.0 {
            ScoreStatus::Pass
        } else if percentage >= 70.0 {
            ScoreStatus::Warning
        } else {
            ScoreStatus::Fail
        };

        Self {
            score,
            max_score,
            percentage,
            status,
            subcategories,
            findings,
        }
    }
}

// BonusScores: aggregate and defaults
impl BonusScores {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total(&self) -> f64 {
        self.property_tests.points
            + self.fuzzing.points
            + self.mutation_testing.points
            + self.living_docs.points
    }
}

impl Default for BonusScores {
    fn default() -> Self {
        Self {
            property_tests: BonusItem {
                points: 0.0,
                max_points: 3.0,
                detected: false,
                evidence: vec![],
            },
            fuzzing: BonusItem {
                points: 0.0,
                max_points: 2.0,
                detected: false,
                evidence: vec![],
            },
            mutation_testing: BonusItem {
                points: 0.0,
                max_points: 2.0,
                detected: false,
                evidence: vec![],
            },
            living_docs: BonusItem {
                points: 0.0,
                max_points: 3.0,
                detected: false,
                evidence: vec![],
            },
        }
    }
}

// Priority: manual PartialOrd/Ord for correct ordering
// Critical > High > Medium > Low
impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_rank = match self {
            Priority::Critical => 4,
            Priority::High => 3,
            Priority::Medium => 2,
            Priority::Low => 1,
        };
        let other_rank = match other {
            Priority::Critical => 4,
            Priority::High => 3,
            Priority::Medium => 2,
            Priority::Low => 1,
        };
        self_rank.cmp(&other_rank)
    }
}

// ScoreMetadata: construction
impl ScoreMetadata {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(repository_path: PathBuf) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            repository_path,
            git_branch: None,
            git_commit: None,
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            spec_version: "1.0.0".to_string(),
            execution_time_ms: 0,
        }
    }
}
