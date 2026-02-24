// Refactoring request generation for refactor-auto
//
// Contains: generate_refactoring_requests, create_complexity_reduction_request,
// create_lint_fix_requests, create_satd_cleanup_requests,
// create_coverage_improvement_requests.
// Split from setup_analysis.rs for file health compliance (CB-040).

/// Generate comprehensive refactoring requests (Phase 3: Extract Core Logic)
///
/// Creates detailed, actionable refactoring requests based on quality analysis.
/// This function has complexity <5 and follows Toyota Way principles.
async fn generate_refactoring_requests(
    quality_analysis: &ProjectQualityAnalysis,
    context: &RefactorContext,
) -> Result<Vec<RefactoringRequest>> {
    eprintln!("🎯 Generating targeted refactoring requests...");

    let mut requests = Vec::new();

    // Generate requests for high-complexity functions
    for violation in &quality_analysis
        .complexity_analysis
        .high_complexity_violations
    {
        let request = create_complexity_reduction_request(violation, context).await?;
        requests.push(request);
    }

    // Generate requests for lint violations
    let lint_requests =
        create_lint_fix_requests(&quality_analysis.lint_violations, context).await?;
    requests.extend(lint_requests);

    // Generate requests for SATD cleanup
    let satd_requests =
        create_satd_cleanup_requests(&quality_analysis.satd_analysis, context).await?;
    requests.extend(satd_requests);

    // Generate requests for coverage improvements
    if quality_analysis.coverage_analysis.overall_coverage_percent
        < context.config.quality_profile.coverage_min
    {
        let coverage_requests =
            create_coverage_improvement_requests(&quality_analysis.coverage_analysis, context)
                .await?;
        requests.extend(coverage_requests);
    }

    eprintln!("📋 Generated {} refactoring requests", requests.len());
    Ok(requests)
}

/// Create complexity reduction request for a high-complexity function
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn create_complexity_reduction_request(
    violation: &ComplexityViolation,
    _context: &RefactorContext,
) -> Result<RefactoringRequest> {
    Ok(RefactoringRequest {
        request_type: RefactoringType::ComplexityReduction,
        target_file: violation.file.clone(),
        priority: if violation.complexity > 20 {
            RefactoringPriority::Critical
        } else {
            RefactoringPriority::High
        },
        description: format!(
            "Reduce complexity of function '{}' from {} to ≤10",
            violation.function_name, violation.complexity
        ),
        ai_instructions: format!(
            "Extract smaller functions, simplify conditional logic, and improve readability. \
             Current complexity: {}. Target: ≤10. Location: {}:{}",
            violation.complexity,
            violation.file.display(),
            violation.line_number
        ),
        estimated_effort: if violation.complexity > 50 {
            RefactoringEffort::Major
        } else if violation.complexity > 20 {
            RefactoringEffort::Moderate
        } else {
            RefactoringEffort::Minor
        },
    })
}

/// Create lint fix requests for violations
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn create_lint_fix_requests(
    violations: &[ViolationDetailJson],
    _context: &RefactorContext,
) -> Result<Vec<RefactoringRequest>> {
    let mut requests = Vec::new();

    for violation in violations {
        let request = RefactoringRequest {
            request_type: RefactoringType::LintFix,
            target_file: violation.file.clone(),
            priority: match violation.severity.as_str() {
                "error" => RefactoringPriority::High,
                "warning" => RefactoringPriority::Medium,
                _ => RefactoringPriority::Low,
            },
            description: format!("Fix lint violation: {}", violation.message),
            ai_instructions: format!(
                "Fix the lint violation '{}' at line {}. Suggestion: {}",
                violation.message,
                violation.line,
                violation
                    .suggestion
                    .as_deref()
                    .unwrap_or("Apply automatic fix")
            ),
            estimated_effort: RefactoringEffort::Trivial,
        };
        requests.push(request);
    }

    Ok(requests)
}

/// Create SATD cleanup requests
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn create_satd_cleanup_requests(
    satd_analysis: &SatdAnalysis,
    _context: &RefactorContext,
) -> Result<Vec<RefactoringRequest>> {
    let mut requests = Vec::new();

    for satd_comment in &satd_analysis.satd_comments {
        let request = RefactoringRequest {
            request_type: RefactoringType::SatdCleanup,
            target_file: satd_comment.file.clone(),
            priority: match satd_comment.satd_type.as_str() {
                "FIXME" | "BUG" => RefactoringPriority::High,
                "TODO" => RefactoringPriority::Medium,
                _ => RefactoringPriority::Low,
            },
            description: format!("Resolve technical debt: {}", satd_comment.comment_text),
            ai_instructions: format!(
                "Remove or implement the technical debt comment '{}' at line {}. \
                 Either implement the suggested improvement or remove if no longer relevant.",
                satd_comment.comment_text, satd_comment.line_number
            ),
            estimated_effort: RefactoringEffort::Minor,
        };
        requests.push(request);
    }

    Ok(requests)
}

/// Create coverage improvement requests
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn create_coverage_improvement_requests(
    coverage_analysis: &CoverageAnalysis,
    _context: &RefactorContext,
) -> Result<Vec<RefactoringRequest>> {
    let mut requests = Vec::new();

    for uncovered_file in &coverage_analysis.files_with_low_coverage {
        let request = RefactoringRequest {
            request_type: RefactoringType::CoverageImprovement,
            target_file: uncovered_file.clone(),
            priority: RefactoringPriority::Medium,
            description: format!("Improve test coverage for {}", uncovered_file.display()),
            ai_instructions: format!(
                "Add comprehensive tests for {}. Focus on edge cases, error conditions, \
                 and critical business logic. Target: ≥80% coverage.",
                uncovered_file.display()
            ),
            estimated_effort: RefactoringEffort::Moderate,
        };
        requests.push(request);
    }

    Ok(requests)
}
