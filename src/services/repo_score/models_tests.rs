#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_from_score_a_plus() {
        let scores = vec![95.0, 100.0, 105.0, 110.0];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::APlus);
            assert_eq!(grade.as_str(), "A+");
        }
    }

    #[test]
    fn test_grade_from_score_a() {
        let scores = vec![90.0, 91.5, 93.9, 94.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::A);
            assert_eq!(grade.as_str(), "A");
        }
    }

    #[test]
    fn test_grade_from_score_a_minus() {
        let scores = vec![85.0, 87.0, 89.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::AMinus);
            assert_eq!(grade.as_str(), "A-");
        }
    }

    #[test]
    fn test_grade_from_score_b_plus() {
        let scores = vec![80.0, 82.0, 84.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::BPlus);
        }
    }

    #[test]
    fn test_grade_from_score_b() {
        let scores = vec![70.0, 75.0, 79.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::B);
        }
    }

    #[test]
    fn test_grade_from_score_c() {
        let scores = vec![60.0, 65.0, 69.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::C);
        }
    }

    #[test]
    fn test_grade_from_score_d() {
        let scores = vec![50.0, 55.0, 59.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::D);
        }
    }

    #[test]
    fn test_grade_from_score_f() {
        let scores = vec![0.0, 25.0, 49.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::F);
        }
    }

    #[test]
    fn test_grade_boundary_values() {
        assert_eq!(Grade::from_score(95.0), Grade::APlus);
        assert_eq!(Grade::from_score(94.99), Grade::A);
        assert_eq!(Grade::from_score(90.0), Grade::A);
        assert_eq!(Grade::from_score(89.99), Grade::AMinus);
    }

    #[test]
    fn test_category_scores_total() {
        let scores = CategoryScores {
            documentation: CategoryScore::new(18.0, 20.0, vec![], vec![]),
            precommit_hooks: CategoryScore::new(18.0, 20.0, vec![], vec![]),
            repository_hygiene: CategoryScore::new(8.0, 10.0, vec![], vec![]),
            build_test_automation: CategoryScore::new(22.0, 25.0, vec![], vec![]),
            continuous_integration: CategoryScore::new(18.0, 20.0, vec![], vec![]),
            pmat_compliance: CategoryScore::new(5.0, 5.0, vec![], vec![]),
        };

        // 18 + 18 + 8 + 22 + 18 + 5 = 89
        assert_eq!(scores.total(), 89.0);
    }

    #[test]
    fn test_category_scores_max_total() {
        let scores = CategoryScores {
            documentation: CategoryScore::new(20.0, 20.0, vec![], vec![]),
            precommit_hooks: CategoryScore::new(20.0, 20.0, vec![], vec![]),
            repository_hygiene: CategoryScore::new(10.0, 10.0, vec![], vec![]),
            build_test_automation: CategoryScore::new(25.0, 25.0, vec![], vec![]),
            continuous_integration: CategoryScore::new(20.0, 20.0, vec![], vec![]),
            pmat_compliance: CategoryScore::new(5.0, 5.0, vec![], vec![]),
        };

        assert_eq!(scores.total(), 100.0);
    }

    #[test]
    fn test_bonus_scores_total() {
        let bonus = BonusScores {
            property_tests: BonusItem {
                points: 3.0,
                max_points: 3.0,
                detected: true,
                evidence: vec![],
            },
            fuzzing: BonusItem {
                points: 2.0,
                max_points: 2.0,
                detected: true,
                evidence: vec![],
            },
            mutation_testing: BonusItem {
                points: 2.0,
                max_points: 2.0,
                detected: true,
                evidence: vec![],
            },
            living_docs: BonusItem {
                points: 0.0,
                max_points: 3.0,
                detected: false,
                evidence: vec![],
            },
        };

        assert_eq!(bonus.total(), 7.0);
    }

    #[test]
    fn test_bonus_scores_max_total() {
        let bonus = BonusScores {
            property_tests: BonusItem {
                points: 3.0,
                max_points: 3.0,
                detected: true,
                evidence: vec![],
            },
            fuzzing: BonusItem {
                points: 2.0,
                max_points: 2.0,
                detected: true,
                evidence: vec![],
            },
            mutation_testing: BonusItem {
                points: 2.0,
                max_points: 2.0,
                detected: true,
                evidence: vec![],
            },
            living_docs: BonusItem {
                points: 3.0,
                max_points: 3.0,
                detected: true,
                evidence: vec![],
            },
        };

        assert_eq!(bonus.total(), 10.0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_score_status_pass() {
        let score = CategoryScore::new(18.0, 20.0, vec![], vec![]);
        assert_eq!(score.percentage, 90.0);
        assert_eq!(score.status, ScoreStatus::Pass);
    }

    #[test]
    fn test_score_status_warning() {
        let score = CategoryScore::new(16.0, 20.0, vec![], vec![]);
        assert_eq!(score.percentage, 80.0);
        assert_eq!(score.status, ScoreStatus::Warning);
    }

    #[test]
    fn test_score_status_fail() {
        let score = CategoryScore::new(10.0, 20.0, vec![], vec![]);
        assert_eq!(score.percentage, 50.0);
        assert_eq!(score.status, ScoreStatus::Fail);
    }
}
