// Tests for work_contract_scoring.rs (DBC spec §13.4-13.6)

#[test]
fn test_score_grade_from_score() {
    assert_eq!(ScoreGrade::from_score(1.0), ScoreGrade::A);
    assert_eq!(ScoreGrade::from_score(0.90), ScoreGrade::A);
    assert_eq!(ScoreGrade::from_score(0.89), ScoreGrade::B);
    assert_eq!(ScoreGrade::from_score(0.75), ScoreGrade::B);
    assert_eq!(ScoreGrade::from_score(0.74), ScoreGrade::C);
    assert_eq!(ScoreGrade::from_score(0.60), ScoreGrade::C);
    assert_eq!(ScoreGrade::from_score(0.59), ScoreGrade::D);
    assert_eq!(ScoreGrade::from_score(0.40), ScoreGrade::D);
    assert_eq!(ScoreGrade::from_score(0.39), ScoreGrade::F);
    assert_eq!(ScoreGrade::from_score(0.0), ScoreGrade::F);
}

#[test]
fn test_score_grade_display() {
    assert_eq!(ScoreGrade::A.to_string(), "A");
    assert_eq!(ScoreGrade::B.to_string(), "B");
    assert_eq!(ScoreGrade::C.to_string(), "C");
    assert_eq!(ScoreGrade::D.to_string(), "D");
    assert_eq!(ScoreGrade::F.to_string(), "F");
}

#[test]
fn test_scoring_weights_sum_to_one() {
    let w = ScoringWeights::default();
    let sum = w.spec_depth + w.falsification + w.invariant_health + w.subcontracting + w.traceability;
    assert!((sum - 1.0).abs() < f64::EPSILON, "Weights must sum to 1.0, got {}", sum);
}

#[test]
fn test_score_contract_v4_default() {
    let contract = WorkContract::new("test".to_string(), "abc123".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let score = score_contract(&contract, tmp.path());

    // v4.0 contract with 22 claims, none verified
    assert_eq!(score.spec_depth, 1.0); // 22/22
    assert_eq!(score.falsification_coverage, 0.0); // none verified
    assert_eq!(score.subcontracting, 1.0); // first iteration
    assert_eq!(score.traceability, 0.8); // v4.0 fallback
    assert!(score.total > 0.0);
    assert!(score.total <= 1.0);
}

#[test]
fn test_score_contract_all_verified() {
    let mut contract = WorkContract::new("test".to_string(), "abc123".to_string());
    for claim in &mut contract.claims {
        claim.result = Some(FalsificationResult::passed("ok"));
    }
    let tmp = tempfile::tempdir().unwrap();
    let score = score_contract(&contract, tmp.path());

    assert_eq!(score.falsification_coverage, 1.0);
    assert!(score.total > 0.5);
}

#[test]
fn test_compute_spec_depth_v4() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let depth = compute_spec_depth(&contract);
    assert_eq!(depth, 1.0); // 22/22
}

#[test]
fn test_compute_spec_depth_empty_claims() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    contract.claims.clear();
    let depth = compute_spec_depth(&contract);
    assert_eq!(depth, 0.0);
}

#[test]
fn test_compute_falsification_coverage_empty() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    contract.claims.clear();
    assert_eq!(compute_falsification_coverage(&contract), 0.0);
}

#[test]
fn test_compute_falsification_coverage_partial() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    let total = contract.claims.len();
    // Verify half
    for claim in contract.claims.iter_mut().take(total / 2) {
        claim.result = Some(FalsificationResult::passed("ok"));
    }
    let cov = compute_falsification_coverage(&contract);
    let expected = (total / 2) as f64 / total as f64;
    assert!((cov - expected).abs() < 0.01);
}

#[test]
fn test_compute_subcontracting_first_iteration() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    assert_eq!(compute_subcontracting_score(&contract), 1.0);
}

#[test]
fn test_compute_traceability_v4() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    assert_eq!(compute_traceability(&contract), 0.8);
}

#[test]
fn test_compute_traceability_empty_v4() {
    let mut contract = WorkContract::new("test".to_string(), "abc".to_string());
    contract.claims.clear();
    assert_eq!(compute_traceability(&contract), 0.0);
}

#[test]
fn test_drift_metrics_new_contract() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let drift = compute_drift_metrics(&contract, tmp.path());

    assert!(drift.hours_since_creation >= 0.0);
    assert!(drift.hours_since_checkpoint >= 0.0);
    assert!(drift.drift_rate >= 0.0);
    assert!(drift.recovery_rate > 0.0);
    assert!(drift.bounded_drift >= 0.0);
    // Just created, so not stale (unless test takes >24h!)
    assert!(!drift.is_stale);
}

#[test]
fn test_drift_metrics_recovery_rate_floor() {
    let contract = WorkContract::new("test".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();
    let drift = compute_drift_metrics(&contract, tmp.path());

    // Recovery rate should never be zero (floored at 0.01)
    assert!(drift.recovery_rate >= 0.01);
}

#[test]
fn test_trend_direction_display() {
    assert_eq!(TrendDirection::Improving.to_string(), "improving");
    assert_eq!(TrendDirection::Stable.to_string(), "stable");
    assert_eq!(TrendDirection::Declining.to_string(), "declining");
}

#[test]
fn test_analyze_trend_empty() {
    let trend = analyze_trend(vec![]);
    assert_eq!(trend.snapshots.len(), 0);
    assert_eq!(trend.rolling_average, 0.0);
    assert!(!trend.drift_detected);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn test_analyze_trend_single_snapshot() {
    let snapshot = QualityTrendSnapshot {
        timestamp: chrono::Utc::now(),
        score: 0.75,
        grade: ScoreGrade::B,
        active_claims: 22,
        verified_claims: 10,
        bounded_drift: 0.5,
        git_sha: "abc123".to_string(),
    };
    let trend = analyze_trend(vec![snapshot]);
    assert_eq!(trend.rolling_average, 0.75);
    assert_eq!(trend.delta_from_average, 0.0);
    assert!(!trend.drift_detected);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn test_analyze_trend_improving() {
    let now = chrono::Utc::now();
    let snapshots = vec![
        QualityTrendSnapshot {
            timestamp: now - chrono::Duration::hours(2),
            score: 0.50,
            grade: ScoreGrade::D,
            active_claims: 22,
            verified_claims: 5,
            bounded_drift: 1.0,
            git_sha: "aaa".to_string(),
        },
        QualityTrendSnapshot {
            timestamp: now,
            score: 0.80,
            grade: ScoreGrade::B,
            active_claims: 22,
            verified_claims: 15,
            bounded_drift: 0.3,
            git_sha: "bbb".to_string(),
        },
    ];
    let trend = analyze_trend(snapshots);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(!trend.drift_detected);
}

#[test]
fn test_analyze_trend_drift_detected() {
    let now = chrono::Utc::now();
    let mut snapshots = Vec::new();
    // 7 snapshots at 0.80
    for i in 0..7 {
        snapshots.push(QualityTrendSnapshot {
            timestamp: now - chrono::Duration::hours(10 - i),
            score: 0.80,
            grade: ScoreGrade::B,
            active_claims: 22,
            verified_claims: 15,
            bounded_drift: 0.3,
            git_sha: format!("sha{}", i),
        });
    }
    // 8th snapshot drops to 0.60 (>5% drop from 0.80 rolling avg)
    snapshots.push(QualityTrendSnapshot {
        timestamp: now,
        score: 0.60,
        grade: ScoreGrade::C,
        active_claims: 22,
        verified_claims: 10,
        bounded_drift: 0.8,
        git_sha: "sha_drop".to_string(),
    });
    let trend = analyze_trend(snapshots);
    assert!(trend.drift_detected);
    assert_eq!(trend.direction, TrendDirection::Declining);
}

#[test]
fn test_record_and_load_trend() {
    let contract = WorkContract::new("test-trend".to_string(), "abc".to_string());
    let tmp = tempfile::tempdir().unwrap();

    let score = ContractScore {
        spec_depth: 1.0,
        falsification_coverage: 0.5,
        invariant_health: 1.0,
        subcontracting: 1.0,
        traceability: 0.8,
        total: 0.85,
        grade: ScoreGrade::B,
    };
    let drift = DriftMetrics {
        hours_since_checkpoint: 1.0,
        hours_since_creation: 2.0,
        drift_rate: 0.02,
        recovery_rate: 0.5,
        bounded_drift: 0.04,
        is_stale: false,
    };

    let path = record_trend_snapshot(&contract, &score, &drift, "abc123", tmp.path()).unwrap();
    assert!(path.exists());

    let trend = load_quality_trend(tmp.path(), "test-trend");
    assert_eq!(trend.snapshots.len(), 1);
    assert!((trend.snapshots[0].score - 0.85).abs() < f64::EPSILON);
}

#[test]
fn test_load_quality_trend_no_data() {
    let tmp = tempfile::tempdir().unwrap();
    let trend = load_quality_trend(tmp.path(), "nonexistent");
    assert_eq!(trend.snapshots.len(), 0);
    assert!(!trend.drift_detected);
}
