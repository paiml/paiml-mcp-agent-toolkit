// Tests for Design by Contract types - Part 1: Threshold Comparison
// Spec: docs/specifications/dbc.md

// === compare_thresholds tests ===

#[test]
fn test_compare_thresholds_none_none() {
    assert_eq!(compare_thresholds(&None, &None), ThresholdComparison::Equal);
}

#[test]
fn test_compare_thresholds_none_to_some_is_strengthened() {
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&None, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_thresholds_some_to_none_is_weakened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &None),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_gte_higher_is_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 96.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_gte_lower_is_weakened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 90.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_gte_equal() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "coverage".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Equal
    );
}

#[test]
fn test_compare_lte_lower_is_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 20.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 15.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_lte_higher_is_weakened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 20.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "complexity".to_string(),
        op: ThresholdOp::Lte,
        value: 25.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_eq_same_value() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 42.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 42.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Equal
    );
}

#[test]
fn test_compare_eq_different_value_is_incompatible() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 42.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Eq,
        value: 43.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Incompatible
    );
}

#[test]
fn test_compare_different_ops_incompatible() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Lte,
        value: 95.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Incompatible
    );
}

#[test]
fn test_compare_different_types_incompatible() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gte,
        value: 95.0,
    });
    let child = Some(ClauseThreshold::Boolean { expected: true });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Incompatible
    );
}

#[test]
fn test_compare_boolean_true_true_equal() {
    let parent = Some(ClauseThreshold::Boolean { expected: true });
    let child = Some(ClauseThreshold::Boolean { expected: true });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Equal
    );
}

#[test]
fn test_compare_boolean_false_to_true_strengthened() {
    let parent = Some(ClauseThreshold::Boolean { expected: false });
    let child = Some(ClauseThreshold::Boolean { expected: true });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_boolean_true_to_false_weakened() {
    let parent = Some(ClauseThreshold::Boolean { expected: true });
    let child = Some(ClauseThreshold::Boolean { expected: false });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Weakened
    );
}

#[test]
fn test_compare_delta_gte_higher_strengthened() {
    let parent = Some(ClauseThreshold::Delta {
        metric: "tdg".to_string(),
        op: ThresholdOp::Gte,
        value: 0.5,
    });
    let child = Some(ClauseThreshold::Delta {
        metric: "tdg".to_string(),
        op: ThresholdOp::Gte,
        value: 1.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_gt_higher_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gt,
        value: 10.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Gt,
        value: 15.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

#[test]
fn test_compare_lt_lower_strengthened() {
    let parent = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Lt,
        value: 20.0,
    });
    let child = Some(ClauseThreshold::Numeric {
        metric: "x".to_string(),
        op: ThresholdOp::Lt,
        value: 15.0,
    });
    assert_eq!(
        compare_thresholds(&parent, &child),
        ThresholdComparison::Strengthened
    );
}

// === ContractQuality tests ===

#[test]
fn test_contract_quality_full() {
    let q = ContractQuality::calculate(14, 14);
    assert_eq!(q.score, 1.0);
    assert_eq!(q.rating, "Full");
}

#[test]
fn test_contract_quality_strong() {
    let q = ContractQuality::calculate(12, 14);
    assert!(q.score > 0.8);
    assert_eq!(q.rating, "Strong");
}

#[test]
fn test_contract_quality_partial() {
    let q = ContractQuality::calculate(8, 14);
    assert!(q.score >= 0.5);
    assert_eq!(q.rating, "Partial");
}

#[test]
fn test_contract_quality_weak() {
    let q = ContractQuality::calculate(3, 14);
    assert!(q.score < 0.5);
    assert_eq!(q.rating, "Weak");
}

#[test]
fn test_contract_quality_zero_applicable() {
    let q = ContractQuality::calculate(0, 0);
    assert_eq!(q.score, 0.0);
    assert_eq!(q.rating, "Weak");
}

// === ClauseKind display ===

#[test]
fn test_clause_kind_display() {
    assert_eq!(format!("{}", ClauseKind::Require), "require");
    assert_eq!(format!("{}", ClauseKind::Ensure), "ensure");
    assert_eq!(format!("{}", ClauseKind::Invariant), "invariant");
}

// === ThresholdOp display ===

#[test]
fn test_threshold_op_display() {
    assert_eq!(format!("{}", ThresholdOp::Gte), ">=");
    assert_eq!(format!("{}", ThresholdOp::Lte), "<=");
    assert_eq!(format!("{}", ThresholdOp::Eq), "==");
    assert_eq!(format!("{}", ThresholdOp::Gt), ">");
    assert_eq!(format!("{}", ThresholdOp::Lt), "<");
}
