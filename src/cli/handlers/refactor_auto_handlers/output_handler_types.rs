// Type definitions for output_handler module.
// Included via include!() — shares parent module scope.

/// Result of a refactoring iteration
#[derive(Debug)]
 // Used in refactoring workflow
struct IterationResult {
    iteration_number: u32,
    successful_requests: Vec<RefactoringSuccess>,
    failed_requests: Vec<RefactoringFailure>,
    iteration_duration: std::time::Duration,
    quality_improvement: QualityImprovement,
}

/// Successful refactoring application
#[derive(Debug, Clone)]
 // Used in refactoring workflow
struct RefactoringSuccess {
    request: RefactoringRequest,
    changes_made: Vec<String>,
    application_duration: std::time::Duration,
    verification_status: VerificationStatus,
}

/// Failed refactoring application
#[derive(Debug)]
 // Used in refactoring workflow
struct RefactoringFailure {
    request: RefactoringRequest,
    error_message: String,
    retry_suggested: bool,
}

/// Verification status for refactoring
#[derive(Debug, Clone)]
 // Used by RefactoringSuccess
enum VerificationStatus {
    Pending,
    Verified,
    Failed(String),
}

/// Result of validation checks
#[derive(Debug)]
 // Used in refactoring workflow
struct ValidationResult {
    overall_success: bool,
    compilation_passed: bool,
    tests_passed: bool,
    quality_improved: bool,
    issues_found: Vec<String>,
}

/// Quality improvement metrics
#[derive(Debug)]
 // Used in refactoring workflow
struct QualityImprovement {
    complexity_reduced: u32,
    violations_fixed: u32,
    satd_resolved: u32,
    coverage_increased: f64,
    overall_score: f64,
}

/// Compilation validation result
#[derive(Debug)]
 // Used in refactoring workflow
struct CompilationResult {
    success: bool,
    error_message: String,
    warnings_count: u32,
}

/// Test execution result
#[derive(Debug)]
 // Used in refactoring workflow
struct TestResult {
    success: bool,
    passed_count: u32,
    failed_count: u32,
    output: String,
}

/// Comprehensive refactoring session summary
#[derive(Debug, serde::Serialize)]
struct RefactoringSummary {
    total_successful_requests: usize,
    total_failed_requests: usize,
    total_quality_score: f64,
    total_complexity_reduced: u32,
    total_violations_fixed: u32,
    total_satd_resolved: u32,
    total_coverage_increased: f64,
}

/// Helper struct for iteration continuation
struct IterationContinuation {
    should_continue: bool,
    remaining_requests: Vec<RefactoringRequest>,
}
