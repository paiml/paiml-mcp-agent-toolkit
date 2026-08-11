#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::collections::HashSet;
use std::path::PathBuf;

// ============ Grade Tests ============

#[test]
fn test_grade_from_score() {
    assert_eq!(Grade::from_score(95.0), Grade::APlus);
    assert_eq!(Grade::from_score(90.0), Grade::A);
    assert_eq!(Grade::from_score(85.0), Grade::AMinus);
    assert_eq!(Grade::from_score(80.0), Grade::BPlus);
    assert_eq!(Grade::from_score(75.0), Grade::B);
    assert_eq!(Grade::from_score(70.0), Grade::BMinus);
    assert_eq!(Grade::from_score(65.0), Grade::CPlus);
    assert_eq!(Grade::from_score(60.0), Grade::C);
    assert_eq!(Grade::from_score(55.0), Grade::CMinus);
    assert_eq!(Grade::from_score(50.0), Grade::D);
    assert_eq!(Grade::from_score(45.0), Grade::F);
}

#[test]
fn test_grade_from_score_boundaries() {
    assert_eq!(Grade::from_score(100.0), Grade::APlus);
    assert_eq!(Grade::from_score(94.9), Grade::A);
    assert_eq!(Grade::from_score(89.9), Grade::AMinus);
    assert_eq!(Grade::from_score(49.9), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
    assert_eq!(Grade::from_score(-10.0), Grade::F);
}

#[test]
fn test_grade_display_all() {
    assert_eq!(format!("{}", Grade::APlus), "A+");
    assert_eq!(format!("{}", Grade::A), "A");
    assert_eq!(format!("{}", Grade::AMinus), "A-");
    assert_eq!(format!("{}", Grade::BPlus), "B+");
    assert_eq!(format!("{}", Grade::B), "B");
    assert_eq!(format!("{}", Grade::BMinus), "B-");
    assert_eq!(format!("{}", Grade::CPlus), "C+");
    assert_eq!(format!("{}", Grade::C), "C");
    assert_eq!(format!("{}", Grade::CMinus), "C-");
    assert_eq!(format!("{}", Grade::D), "D");
    assert_eq!(format!("{}", Grade::F), "F");
}

#[test]
fn test_grade_default() {
    let grade = Grade::default();
    assert_eq!(grade, Grade::C);
}

#[test]
fn test_grade_ordering() {
    assert!(Grade::APlus < Grade::A);
    assert!(Grade::A < Grade::AMinus);
    assert!(Grade::AMinus < Grade::BPlus);
    assert!(Grade::D < Grade::F);
}

#[test]
fn test_grade_clone_copy() {
    let g1 = Grade::APlus;
    let g2 = g1;
    let g3 = g1;
    assert_eq!(g1, g2);
    assert_eq!(g1, g3);
}

#[test]
fn test_grade_serialization() {
    let grade = Grade::BPlus;
    let json = serde_json::to_string(&grade).unwrap();
    let deserialized: Grade = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, Grade::BPlus);
}

#[test]
fn test_grade_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Grade::A);
    set.insert(Grade::B);
    assert!(set.contains(&Grade::A));
    assert!(!set.contains(&Grade::F));
}

// ============ TdgScore Tests ============

#[test]
fn test_tdg_score_default() {
    let score = TdgScore::default();
    assert_eq!(score.structural_complexity, 25.0);
    assert_eq!(score.semantic_complexity, 20.0);
    assert_eq!(score.duplication_ratio, 20.0);
    assert_eq!(score.coupling_score, 15.0);
    assert_eq!(score.doc_coverage, 10.0);
    assert_eq!(score.consistency_score, 10.0);
    assert_eq!(score.entropy_score, 0.0);
    assert_eq!(score.total, 100.0);
    assert_eq!(score.grade, Grade::APlus);
    assert_eq!(score.confidence, 1.0);
    assert_eq!(score.language, Language::Unknown);
    assert!(score.file_path.is_none());
    assert!(score.penalties_applied.is_empty());
    assert_eq!(score.critical_defects_count, 0);
    assert!(!score.has_critical_defects);
}

#[test]
fn test_tdg_score_calculate_total() {
    let mut score = TdgScore {
        structural_complexity: 20.0,
        semantic_complexity: 18.0,
        duplication_ratio: 19.0,
        coupling_score: 14.0,
        doc_coverage: 9.0,
        consistency_score: 8.0,
        entropy_score: 12.0, // Will be clamped to 10.0 by calculate_total()
        has_contract_coverage: true, // Enable to test raw grade mapping
        ..TdgScore::default()
    };

    score.calculate_total();

    // After clamping: 20+18+19+14+9+8+10(clamped) = 98.0
    assert_eq!(score.total, 98.0);
    assert_eq!(score.grade, Grade::APlus); // 98.0 >= 95.0 = A+
}

#[test]
fn test_tdg_score_calculate_total_clamping() {
    let mut score = TdgScore {
        structural_complexity: 50.0, // Will be clamped to 25.0
        semantic_complexity: 30.0,   // Will be clamped to 20.0
        duplication_ratio: 40.0,     // Will be clamped to 20.0
        coupling_score: 25.0,        // Will be clamped to 15.0
        doc_coverage: 20.0,          // Will be clamped to 10.0
        consistency_score: 15.0,     // Will be clamped to 10.0
        entropy_score: 20.0,         // Will be clamped to 10.0
        ..TdgScore::default()
    };

    score.calculate_total();

    // All clamped to max: 25+20+20+15+10+10+10 = 110 > 100
    // Normalized: (110/110) * 100 = 100.0
    assert_eq!(score.total, 100.0);
}

#[test]
fn test_tdg_score_calculate_total_zero() {
    let mut score = TdgScore {
        structural_complexity: 0.0,
        semantic_complexity: 0.0,
        duplication_ratio: 0.0,
        coupling_score: 0.0,
        doc_coverage: 0.0,
        consistency_score: 0.0,
        entropy_score: 0.0,
        ..TdgScore::default()
    };

    score.calculate_total();

    assert_eq!(score.total, 0.0);
    assert_eq!(score.grade, Grade::F);
}

#[test]
fn test_tdg_score_critical_defects_are_penalised_not_annihilated() {
    let mut score = TdgScore {
        structural_complexity: 25.0,
        semantic_complexity: 20.0,
        duplication_ratio: 20.0,
        coupling_score: 15.0,
        doc_coverage: 10.0,
        consistency_score: 10.0,
        has_critical_defects: true,
        critical_defects_count: 1,
        ..TdgScore::default()
    };

    score.calculate_total();

    // Was `assert_eq!(score.total, 0.0)` / `Grade::F`. Expressing the auto-fail
    // as an annihilated score made every offending file read EXACTLY 0.0 no
    // matter what else was true of it, so a perfect module with one `.unwrap()`
    // and a one-line disaster were indistinguishable, and fixing nine of ten
    // defects moved the number not at all. Whether a build fails is now
    // `CriticalDefectGate`, which reads `has_critical_defects` directly; this
    // number is free to stay informative.
    assert_eq!(score.grade, Grade::CPlus);
    assert!(
        score.total > 0.0 && score.total < 70.0,
        "one defect must cap the score below B- without erasing it: got {}",
        score.total
    );
    // The finding is still reported, and still un-waived — the gate will fail.
    assert!(score.has_critical_defects);
    assert!(score.critical_defects_suppressed.is_none());
}

/// GH #680, second round. Was `test_tdg_score_contract_coverage_caps_a_to_aminus`
/// and asserted `Grade::AMinus` for a file totalling a perfect 100.0 — it
/// pinned the very defect that made `pmat tdg` print
/// `Overall Score: 100.0/100 (A-)`. Rewritten to assert the corrected
/// contract: a perfect file grades A+, measured or not.
#[test]
fn test_tdg_score_perfect_file_grades_a_plus_without_contract_coverage() {
    let mut score = TdgScore {
        structural_complexity: 25.0,
        semantic_complexity: 20.0,
        duplication_ratio: 20.0,
        coupling_score: 15.0,
        doc_coverage: 10.0,
        consistency_score: 10.0,
        has_contract_coverage: false,
        ..TdgScore::default()
    };

    score.calculate_total();

    assert_eq!(score.total, 100.0);
    assert_eq!(score.grade, Grade::APlus);
}

#[test]
fn test_tdg_score_contract_coverage_allows_a() {
    // A file scoring A+ with contract coverage should keep A+
    let mut score = TdgScore {
        structural_complexity: 25.0,
        semantic_complexity: 20.0,
        duplication_ratio: 20.0,
        coupling_score: 15.0,
        doc_coverage: 10.0,
        consistency_score: 10.0,
        has_contract_coverage: true,
        ..TdgScore::default()
    };

    score.calculate_total();

    assert!(score.total >= 95.0);
    assert_eq!(score.grade, Grade::APlus);
}

#[test]
fn test_tdg_score_contract_coverage_no_effect_below_aminus() {
    // A file scoring B+ without contract coverage should stay B+
    let mut score = TdgScore {
        structural_complexity: 20.0,
        semantic_complexity: 16.0,
        duplication_ratio: 16.0,
        coupling_score: 12.0,
        doc_coverage: 8.0,
        consistency_score: 8.0,
        has_contract_coverage: false,
        ..TdgScore::default()
    };

    score.calculate_total();

    // Total is 80 → B+, no cap applied
    assert_eq!(score.grade, Grade::BPlus);
}

#[test]
fn test_tdg_score_set_metric() {
    let mut score = TdgScore::default();

    score.set_metric(MetricCategory::StructuralComplexity, 15.0);
    assert_eq!(score.structural_complexity, 15.0);

    score.set_metric(MetricCategory::SemanticComplexity, 12.0);
    assert_eq!(score.semantic_complexity, 12.0);

    score.set_metric(MetricCategory::Duplication, 18.0);
    assert_eq!(score.duplication_ratio, 18.0);

    score.set_metric(MetricCategory::Coupling, 10.0);
    assert_eq!(score.coupling_score, 10.0);

    score.set_metric(MetricCategory::Documentation, 8.0);
    assert_eq!(score.doc_coverage, 8.0);

    score.set_metric(MetricCategory::Consistency, 7.0);
    assert_eq!(score.consistency_score, 7.0);
}

#[test]
fn test_tdg_score_clone() {
    let score = TdgScore {
        structural_complexity: 20.0,
        file_path: Some(PathBuf::from("/test/file.rs")),
        ..TdgScore::default()
    };
    let cloned = score.clone();
    assert_eq!(cloned.structural_complexity, 20.0);
    assert_eq!(cloned.file_path, Some(PathBuf::from("/test/file.rs")));
}

#[test]
fn test_tdg_score_serialization() {
    let score = TdgScore {
        structural_complexity: 20.0,
        total: 85.0,
        grade: Grade::AMinus,
        ..TdgScore::default()
    };
    let json = serde_json::to_string(&score).unwrap();
    let deserialized: TdgScore = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.structural_complexity, 20.0);
    assert_eq!(deserialized.total, 85.0);
}

// ============ MetricCategory Tests ============

#[test]
fn test_metric_category_clone_copy() {
    let cat = MetricCategory::StructuralComplexity;
    let cat2 = cat;
    assert_eq!(cat, cat2);
}

#[test]
fn test_metric_category_serialization() {
    let cat = MetricCategory::Duplication;
    let json = serde_json::to_string(&cat).unwrap();
    let deserialized: MetricCategory = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, MetricCategory::Duplication);
}

#[test]
fn test_metric_category_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(MetricCategory::StructuralComplexity);
    set.insert(MetricCategory::Coupling);
    assert!(set.contains(&MetricCategory::StructuralComplexity));
    assert!(!set.contains(&MetricCategory::Documentation));
}

#[test]
fn test_metric_category_debug() {
    let cat = MetricCategory::SemanticComplexity;
    let debug = format!("{:?}", cat);
    assert!(debug.contains("SemanticComplexity"));
}

// ============ PenaltyAttribution Tests ============

#[test]
fn test_penalty_attribution_creation() {
    let penalty = PenaltyAttribution {
        source_metric: MetricCategory::StructuralComplexity,
        amount: 5.0,
        applied_to: HashSet::from([MetricCategory::StructuralComplexity]),
        issue: "High complexity".to_string(),
    };
    assert_eq!(penalty.amount, 5.0);
    assert!(penalty
        .applied_to
        .contains(&MetricCategory::StructuralComplexity));
}

#[test]
fn test_penalty_attribution_clone() {
    let penalty = PenaltyAttribution {
        source_metric: MetricCategory::Duplication,
        amount: 3.0,
        applied_to: HashSet::from([MetricCategory::Duplication, MetricCategory::Consistency]),
        issue: "Code duplication detected".to_string(),
    };
    let cloned = penalty.clone();
    assert_eq!(cloned.amount, 3.0);
    assert_eq!(cloned.applied_to.len(), 2);
}

#[test]
fn test_penalty_attribution_serialization() {
    let penalty = PenaltyAttribution {
        source_metric: MetricCategory::Documentation,
        amount: 2.0,
        applied_to: HashSet::from([MetricCategory::Documentation]),
        issue: "Missing docs".to_string(),
    };
    let json = serde_json::to_string(&penalty).unwrap();
    let deserialized: PenaltyAttribution = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.amount, 2.0);
}

// ============ ProjectScore Tests ============

#[test]
fn test_project_score_default() {
    let score = ProjectScore::default();
    assert!(score.files.is_empty());
    // GH #704: `Default` cannot claim a score either — it analysed nothing.
    assert_eq!(score.average_score, None);
    assert_eq!(score.total_files, 0);
    assert!(score.language_distribution.is_empty());
}

#[test]
fn test_project_score_aggregate_empty() {
    // GH #704: this test used to assert the DEFECT — `average_score == 0.0`
    // and `average_grade == Grade::F` for zero analysed files, which is what
    // `analyze tdg` on an empty directory printed as a measurement. Rewritten
    // to assert the corrected contract: nothing analysed, nothing claimed.
    let score = ProjectScore::aggregate(vec![]);
    assert_eq!(score.total_files, 0);
    assert_eq!(score.average_score, None);
    assert_eq!(score.average_grade, None);
    assert_eq!(
        score.not_measured,
        vec!["average_score".to_string(), "average_grade".to_string()]
    );
}

#[test]
fn test_project_score_aggregate_single() {
    let tdg_score = TdgScore {
        total: 85.0,
        language: Language::Rust,
        ..TdgScore::default()
    };
    let project = ProjectScore::aggregate(vec![tdg_score]);
    assert_eq!(project.total_files, 1);
    assert_eq!(project.average_score, Some(85.0));
    assert_eq!(project.average_grade, Some(Grade::AMinus));
    assert_eq!(
        *project.language_distribution.get(&Language::Rust).unwrap(),
        1
    );
}

#[test]
fn test_project_score_aggregate_multiple() {
    let scores = vec![
        TdgScore {
            total: 90.0,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 80.0,
            language: Language::Python,
            ..TdgScore::default()
        },
        TdgScore {
            total: 70.0,
            language: Language::Rust,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);
    assert_eq!(project.total_files, 3);
    assert_eq!(project.average_score, Some(80.0));
    assert_eq!(project.average_grade, Some(Grade::BPlus));
    assert_eq!(
        *project.language_distribution.get(&Language::Rust).unwrap(),
        2
    );
    assert_eq!(
        *project
            .language_distribution
            .get(&Language::Python)
            .unwrap(),
        1
    );
}

#[test]
fn test_project_score_average_empty() {
    // This used to assert `structural_complexity == 25.0` with the comment
    // "Default values" — 25.0 is the STRUCT DEFAULT, i.e. full marks for that
    // category, awarded to a project in which no file was analysed at all.
    // That is what made `pmat tdg <dir> --include-components` print a
    // 25/20/20/15/10/10 breakdown summing to 100 next to a total of 0.0, with
    // an empty directory producing the byte-identical breakdown. Nothing was
    // measured, so every component must claim zero.
    let project = ProjectScore::default();
    let avg = project.average();
    assert_eq!(avg.total, 0.0);
    assert_eq!(avg.confidence, 0.0);
    assert_eq!(avg.structural_complexity, 0.0);
    assert_eq!(avg.semantic_complexity, 0.0);
    assert_eq!(avg.duplication_ratio, 0.0);
    assert_eq!(avg.coupling_score, 0.0);
    assert_eq!(avg.doc_coverage, 0.0);
    assert_eq!(avg.consistency_score, 0.0);
}

#[test]
fn test_project_score_average_single() {
    let tdg_score = TdgScore {
        structural_complexity: 20.0,
        semantic_complexity: 15.0,
        language: Language::TypeScript,
        ..TdgScore::default()
    };
    let project = ProjectScore {
        files: vec![tdg_score],
        language_distribution: std::collections::BTreeMap::from([(Language::TypeScript, 1)]),
        ..ProjectScore::default()
    };
    let avg = project.average();
    assert_eq!(avg.structural_complexity, 20.0);
    assert_eq!(avg.semantic_complexity, 15.0);
    assert_eq!(avg.language, Language::TypeScript);
}

#[test]
fn test_project_score_average_multiple() {
    let scores = vec![
        TdgScore {
            structural_complexity: 20.0,
            semantic_complexity: 10.0,
            ..TdgScore::default()
        },
        TdgScore {
            structural_complexity: 10.0,
            semantic_complexity: 20.0,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);
    let avg = project.average();
    assert_eq!(avg.structural_complexity, 15.0);
    assert_eq!(avg.semantic_complexity, 15.0);
}

// ============ F-Grade Capping Tests ============

#[test]
fn test_project_score_f_grade_capping() {
    // Test that F-grade files cap project grade at B
    // Many A+ files would hide the F-grade in the average without capping
    let scores = vec![
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 40.0, // F-grade - hidden in average!
            grade: Grade::F,
            language: Language::Rust,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);

    // Without capping: (9x95 + 40)/10 = 895/10 = 89.5 -> A- (F hidden!)
    // With F-grade capping: Grade is capped to B
    assert_eq!(project.f_grade_count, 1);
    assert!(project.grade_capped);
    assert_eq!(project.average_grade, Some(Grade::B));
}

#[test]
fn test_project_score_no_f_grade_capping() {
    // Test that projects without F-grades are not capped.
    //
    // `has_contract_coverage: true` is now load-bearing: since GH #680 the
    // project grade goes through the same mapping as the file grades, which
    // caps the A-tier at A- when contract coverage is missing. Leaving it at
    // the `TdgScore::default()` of `false` would test that cap, not F-capping.
    let scores = vec![
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            has_contract_coverage: true,
            ..TdgScore::default()
        },
        TdgScore {
            total: 90.0,
            grade: Grade::A,
            language: Language::Rust,
            has_contract_coverage: true,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);

    assert_eq!(project.f_grade_count, 0);
    assert!(!project.grade_capped);
    // Average: (95+90)/2 = 92.5 -> A (90-94 range)
    assert_eq!(project.average_grade, Some(Grade::A));
}

/// GH #680, second round. This test used to assert the two files WITHOUT
/// contract coverage both graded `AMinus` and dragged the project to `AMinus`
/// — it pinned the cap that made `APlus`/`A` unreachable at any score.
/// Rewritten to assert the corrected contract: the grade is a function of the
/// score alone, so contract coverage changes nothing.
#[test]
fn test_project_grade_ignores_unmeasured_contract_coverage() {
    let scores = vec![
        TdgScore {
            total: 95.0,
            grade: Grade::from_score(95.0),
            language: Language::Rust,
            has_contract_coverage: false,
            ..TdgScore::default()
        },
        TdgScore {
            total: 90.0,
            grade: Grade::from_score(90.0),
            language: Language::Rust,
            has_contract_coverage: false,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);

    // Same two files as test_project_score_no_f_grades_no_cap, which sets
    // has_contract_coverage: true — the answer must be identical.
    assert_eq!(project.average_grade, Some(Grade::A));
    assert_eq!(project.files[0].grade, Grade::APlus);
    assert_eq!(project.files[1].grade, Grade::A);
}

#[test]
fn test_project_score_grade_distribution() {
    let scores = vec![
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 85.0,
            grade: Grade::AMinus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 75.0,
            grade: Grade::B,
            language: Language::Rust,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);

    assert_eq!(*project.grade_distribution.get(&Grade::APlus).unwrap(), 1);
    assert_eq!(*project.grade_distribution.get(&Grade::AMinus).unwrap(), 1);
    assert_eq!(*project.grade_distribution.get(&Grade::B).unwrap(), 1);
}

#[test]
fn test_project_score_multiple_f_grades() {
    // Multiple F-grades that would still average above B+ without capping
    let scores = vec![
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 30.0, // F-grade
            grade: Grade::F,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 40.0, // F-grade
            grade: Grade::F,
            language: Language::Rust,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);

    // Without capping: (4x95 + 30 + 40)/6 = 450/6 = 75.0 -> B (borderline)
    // With F-grades: even at B average, grade_capped should not be set
    // because we only cap grades BETTER than B
    assert_eq!(project.f_grade_count, 2);
    assert_eq!(*project.grade_distribution.get(&Grade::F).unwrap_or(&0), 2);
}

#[test]
fn test_project_score_f_grade_capping_from_a_plus() {
    // Many A+ files with one F grade - demonstrates hiding problem
    let scores = vec![
        TdgScore {
            total: 97.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 97.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 97.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 97.0,
            grade: Grade::APlus,
            language: Language::Rust,
            ..TdgScore::default()
        },
        TdgScore {
            total: 45.0, // F-grade hidden by A+ files
            grade: Grade::F,
            language: Language::Rust,
            ..TdgScore::default()
        },
    ];
    let project = ProjectScore::aggregate(scores);

    // Without capping: (4x97 + 45)/5 = 433/5 = 86.6 -> A- (F hidden!)
    // With capping: Grade should be B
    assert_eq!(project.f_grade_count, 1);
    assert!(project.grade_capped);
    assert_eq!(project.average_grade, Some(Grade::B));
}

// ============ Comparison Tests ============

#[test]
fn test_comparison_new_improvement() {
    let source1 = TdgScore {
        total: 70.0,
        structural_complexity: 15.0,
        semantic_complexity: 10.0,
        file_path: Some(PathBuf::from("source1.rs")),
        ..TdgScore::default()
    };
    let source2 = TdgScore {
        total: 85.0,
        structural_complexity: 20.0,
        semantic_complexity: 15.0,
        file_path: Some(PathBuf::from("source2.rs")),
        ..TdgScore::default()
    };
    let comparison = Comparison::new(source1, source2);
    assert_eq!(comparison.delta, 15.0);
    assert!(comparison.improvement_percentage > 0.0);
    assert_eq!(comparison.winner, "source2.rs");
    assert!(!comparison.improvements.is_empty());
}

#[test]
fn test_comparison_new_regression() {
    let source1 = TdgScore {
        total: 85.0,
        structural_complexity: 20.0,
        doc_coverage: 10.0,
        file_path: Some(PathBuf::from("before.rs")),
        ..TdgScore::default()
    };
    let source2 = TdgScore {
        total: 70.0,
        structural_complexity: 15.0,
        doc_coverage: 5.0,
        file_path: Some(PathBuf::from("after.rs")),
        ..TdgScore::default()
    };
    let comparison = Comparison::new(source1, source2);
    assert_eq!(comparison.delta, -15.0);
    assert!(comparison.improvement_percentage < 0.0);
    assert_eq!(comparison.winner, "before.rs");
    assert!(!comparison.regressions.is_empty());
}

#[test]
fn test_comparison_new_no_path() {
    let source1 = TdgScore {
        total: 70.0,
        ..TdgScore::default()
    };
    let source2 = TdgScore {
        total: 80.0,
        ..TdgScore::default()
    };
    let comparison = Comparison::new(source1, source2);
    assert_eq!(comparison.winner, "source2");
}

#[test]
fn test_comparison_zero_source() {
    let source1 = TdgScore {
        total: 0.0,
        ..TdgScore::default()
    };
    let source2 = TdgScore {
        total: 50.0,
        ..TdgScore::default()
    };
    let comparison = Comparison::new(source1, source2);
    assert_eq!(comparison.improvement_percentage, 0.0); // Div by zero protection
}

#[test]
fn test_comparison_duplication_improvement() {
    let source1 = TdgScore {
        duplication_ratio: 10.0,
        ..TdgScore::default()
    };
    let source2 = TdgScore {
        duplication_ratio: 15.0,
        ..TdgScore::default()
    };
    let comparison = Comparison::new(source1, source2);
    assert!(comparison
        .improvements
        .iter()
        .any(|s| s.contains("duplication")));
}

#[test]
fn test_comparison_serialization() {
    let comparison = Comparison::new(TdgScore::default(), TdgScore::default());
    let json = serde_json::to_string(&comparison).unwrap();
    let deserialized: Comparison = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.delta, 0.0);
}

#[test]
fn test_comparison_missing_branches() {
    // Covers: semantic regression, duplication regression, doc improvement
    let source1 = TdgScore {
        semantic_complexity: 20.0,
        duplication_ratio: 15.0,
        doc_coverage: 5.0,
        ..TdgScore::default()
    };
    let source2 = TdgScore {
        semantic_complexity: 10.0,
        duplication_ratio: 8.0,
        doc_coverage: 12.0,
        ..TdgScore::default()
    };
    let comparison = Comparison::new(source1, source2);
    assert!(comparison
        .regressions
        .iter()
        .any(|s| s.contains("Semantic")));
    assert!(comparison
        .regressions
        .iter()
        .any(|s| s.contains("duplication")));
    assert!(comparison
        .improvements
        .iter()
        .any(|s| s.contains("Documentation")));
}

// ============ PenaltyTracker Tests ============

#[test]
fn test_penalty_tracker() {
    let mut tracker = PenaltyTracker::new();

    let penalty1 = tracker.apply(
        "issue1".to_string(),
        MetricCategory::StructuralComplexity,
        3.5,
        "High cyclomatic complexity".to_string(),
    );
    assert_eq!(penalty1, Some(3.5));

    let penalty2 = tracker.apply(
        "issue1".to_string(),
        MetricCategory::StructuralComplexity,
        3.5,
        "High cyclomatic complexity".to_string(),
    );
    assert_eq!(penalty2, None);

    let attributions = tracker.get_attributions();
    assert_eq!(attributions.len(), 1);
    assert_eq!(attributions[0].amount, 3.5);
}

#[test]
fn test_penalty_tracker_default() {
    let tracker = PenaltyTracker::default();
    assert!(tracker.get_attributions().is_empty());
}

/// Penalties land in serialized TDG scores and baselines; their order must
/// not depend on insertion order or hash seeds (regression: two duplication
/// penalties swapped positions between identical runs, breaking the
/// byte-identical-baseline guarantee).
#[test]
fn test_penalty_tracker_attribution_order_deterministic() {
    let issues = [
        ("dup_ratio_0.11", "Code duplication: 10.6%", 2.11),
        ("dup_lines_16", "Found 16 duplicate code patterns", 5.0),
        ("complexity_a", "High cyclomatic complexity", 3.0),
    ];

    let mut forward = PenaltyTracker::new();
    for (id, issue, amt) in &issues {
        forward.apply(
            (*id).to_string(),
            MetricCategory::Duplication,
            *amt,
            (*issue).to_string(),
        );
    }
    let mut reverse = PenaltyTracker::new();
    for (id, issue, amt) in issues.iter().rev() {
        reverse.apply(
            (*id).to_string(),
            MetricCategory::Duplication,
            *amt,
            (*issue).to_string(),
        );
    }

    let f: Vec<_> = forward
        .get_attributions()
        .into_iter()
        .map(|a| a.issue)
        .collect();
    let r: Vec<_> = reverse
        .get_attributions()
        .into_iter()
        .map(|a| a.issue)
        .collect();
    assert_eq!(f, r, "attribution order must not depend on insertion order");
}

#[test]
fn test_penalty_tracker_multiple_issues() {
    let mut tracker = PenaltyTracker::new();

    tracker.apply(
        "issue1".to_string(),
        MetricCategory::StructuralComplexity,
        3.0,
        "High complexity".to_string(),
    );
    tracker.apply(
        "issue2".to_string(),
        MetricCategory::Duplication,
        2.0,
        "Code duplication".to_string(),
    );
    tracker.apply(
        "issue3".to_string(),
        MetricCategory::Documentation,
        1.5,
        "Missing docs".to_string(),
    );

    let attributions = tracker.get_attributions();
    assert_eq!(attributions.len(), 3);
}

#[test]
fn test_penalty_tracker_same_category_different_ids() {
    let mut tracker = PenaltyTracker::new();

    let p1 = tracker.apply(
        "complexity-func1".to_string(),
        MetricCategory::StructuralComplexity,
        2.0,
        "func1 too complex".to_string(),
    );
    let p2 = tracker.apply(
        "complexity-func2".to_string(),
        MetricCategory::StructuralComplexity,
        3.0,
        "func2 too complex".to_string(),
    );

    assert_eq!(p1, Some(2.0));
    assert_eq!(p2, Some(3.0));
    assert_eq!(tracker.get_attributions().len(), 2);
}
