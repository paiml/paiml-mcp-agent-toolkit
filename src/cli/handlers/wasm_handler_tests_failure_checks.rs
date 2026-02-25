// ==================== check_for_failures Tests ====================

#[test]
fn test_check_for_failures_all_none() {
    let result = check_for_failures(None, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_check_for_failures_safe_verification() {
    let verification = mock_verification_safe();
    let result = check_for_failures(Some(&verification), None, None);
    assert!(result.is_ok());
}

#[test]
fn test_check_for_failures_unsafe_verification() {
    let verification = mock_verification_unsafe();
    let result = check_for_failures(Some(&verification), None, None);
    assert!(result.is_err());
}

#[test]
fn test_check_for_failures_no_critical_security() {
    let security = mock_security_results_no_critical();
    let result = check_for_failures(None, Some(&security), None);
    assert!(result.is_ok());
}

#[test]
fn test_check_for_failures_critical_security() {
    let security = mock_security_results_with_critical();
    let result = check_for_failures(None, Some(&security), None);
    assert!(result.is_err());
}

#[test]
fn test_check_for_failures_passing_baseline() {
    let baseline = mock_quality_assessment_passing();
    let result = check_for_failures(None, None, Some(&baseline));
    assert!(result.is_ok());
}

#[test]
fn test_check_for_failures_failing_baseline() {
    let baseline = mock_quality_assessment_failing();
    let result = check_for_failures(None, None, Some(&baseline));
    assert!(result.is_err());
}

// ==================== check_verification_failure Tests ====================

#[test]
fn test_check_verification_failure_none() {
    let result = check_verification_failure(None);
    assert!(result.is_ok());
}

#[test]
fn test_check_verification_failure_safe() {
    let verification = mock_verification_safe();
    let result = check_verification_failure(Some(&verification));
    assert!(result.is_ok());
}

#[test]
fn test_check_verification_failure_unsafe() {
    let verification = mock_verification_unsafe();
    let result = check_verification_failure(Some(&verification));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Verification failed"));
}

// ==================== check_security_failures Tests ====================

#[test]
fn test_check_security_failures_none() {
    let result = check_security_failures(None);
    assert!(result.is_ok());
}

#[test]
fn test_check_security_failures_empty() {
    let security: Vec<VulnerabilityMatch> = vec![];
    let result = check_security_failures(Some(&security));
    assert!(result.is_ok());
}

#[test]
fn test_check_security_failures_non_critical() {
    let security = mock_security_results_no_critical();
    let result = check_security_failures(Some(&security));
    assert!(result.is_ok());
}

#[test]
fn test_check_security_failures_with_critical() {
    let security = mock_security_results_with_critical();
    let result = check_security_failures(Some(&security));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("critical security vulnerabilities"));
}

// ==================== check_baseline_failure Tests ====================

#[test]
fn test_check_baseline_failure_none() {
    let result = check_baseline_failure(None);
    assert!(result.is_ok());
}

#[test]
fn test_check_baseline_failure_passing() {
    let baseline = mock_quality_assessment_passing();
    let result = check_baseline_failure(Some(&baseline));
    assert!(result.is_ok());
}

#[test]
fn test_check_baseline_failure_failing() {
    let baseline = mock_quality_assessment_failing();
    let result = check_baseline_failure(Some(&baseline));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Quality regression"));
}
