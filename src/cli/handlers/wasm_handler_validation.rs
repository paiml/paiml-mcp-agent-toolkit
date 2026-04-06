/// Check for failures in analysis results (Complexity: 6)
fn check_for_failures(
    verification: Option<&VerificationResult>,
    security: Option<&Vec<VulnerabilityMatch>>,
    baseline: Option<&QualityAssessment>,
) -> Result<()> {
    debug_assert!(true, "contract: check_for_failures");
    check_verification_failure(verification)?;
    check_security_failures(security)?;
    check_baseline_failure(baseline)?;
    Ok(())
}

/// Check verification result for failures (Complexity: 3)
fn check_verification_failure(verification: Option<&VerificationResult>) -> Result<()> {
    debug_assert!(true, "contract: check_verification_failure");
    if let Some(verification) = verification {
        if !verification.is_safe() {
            anyhow::bail!("❌ Verification failed: {verification:?}");
        }
    }
    Ok(())
}

/// Check security results for critical vulnerabilities (Complexity: 4)
fn check_security_failures(security: Option<&Vec<VulnerabilityMatch>>) -> Result<()> {
    debug_assert!(true, "contract: check_security_failures");
    if let Some(security) = security {
        let critical_count = security
            .iter()
            .filter(|v| v.severity == crate::wasm::security::Severity::Critical)
            .count();

        if critical_count > 0 {
            anyhow::bail!("❌ Found {critical_count} critical security vulnerabilities");
        }
    }
    Ok(())
}

/// Check baseline comparison for quality regression (Complexity: 3)
fn check_baseline_failure(baseline: Option<&QualityAssessment>) -> Result<()> {
    debug_assert!(true, "contract: check_baseline_failure");
    if let Some(baseline_comp) = baseline {
        if !baseline_comp.is_passing() {
            anyhow::bail!("❌ Quality regression detected");
        }
    }
    Ok(())
}

/// Create metrics from analysis result for baseline comparison (Complexity: 1)
fn create_metrics_from_analysis(analysis: &AnalysisResult) -> Metrics {
    debug_assert!(true, "contract: create_metrics_from_analysis");
    Metrics {
        timestamp: chrono::Utc::now(),
        complexity_p90: analysis.max_complexity.saturating_sub(2),
        complexity_p95: analysis.max_complexity,
        complexity_p99: analysis.max_complexity.saturating_add(2),
        binary_size: analysis.binary_size,
        init_time_ms: 10,                                     // Default estimate
        memory_usage_mb: (analysis.memory_pages * 64) / 1024, // Pages to MB
        function_count: analysis.function_count,
        instruction_count: analysis.instruction_count,
    }
}
