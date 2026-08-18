// Default value functions for serde deserialization

fn default_max_score() -> f64 {
    100.0
}
fn default_weight() -> f64 {
    1.0
}
fn default_true() -> bool {
    true
}
fn default_coverage() -> f64 {
    95.0
}
fn default_per_file_coverage() -> f64 {
    95.0
}
fn default_complexity() -> u32 {
    20
}
fn default_dead_code() -> f64 {
    1.0
}
fn default_file_size() -> u32 {
    500
}
fn default_function_size() -> u32 {
    50
}
fn default_slow_test() -> f64 {
    5.0
}
fn default_slow_coverage() -> f64 {
    10.0
}
fn default_min_tdg_grade() -> String {
    "A".to_string()
}
fn default_tdg_score() -> f64 {
    70.0
}
fn default_min_body_lines() -> usize {
    3
}
fn default_min_tokens() -> usize {
    15
}
fn default_cc003_similarity() -> f64 {
    0.5
}
fn default_cache_warn_hours() -> i64 {
    1
}
fn default_cache_block_hours() -> i64 {
    24
}

/// Create default check configurations for all CB checks
fn default_checks() -> HashMap<String, CheckConfig> {
    let mut checks = HashMap::new();

    // CB-050: Stub detection (Critical - runtime panics)
    checks.insert(
        "cb-050".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Critical,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-060: GPU kernel quality (High - hardware crashes)
    checks.insert(
        "cb-060".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Error,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-070: Critical unwrap detection
    checks.insert(
        "cb-070".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-120: NaN-unsafe comparisons
    checks.insert(
        "cb-120".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-121: Lock poisoning vulnerabilities
    checks.insert(
        "cb-121".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-122: Serde deserialization panics
    checks.insert(
        "cb-122".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-123: Undocumented ignored tests
    checks.insert(
        "cb-123".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Info,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-124: Coverage threshold
    checks.insert(
        "cb-124".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Error,
            threshold: Some(80.0),
            options: HashMap::new(),
        },
    );

    // CB-125: Coverage exclusion gaming
    checks.insert(
        "cb-125".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: Some(20.0), // Max exclusion percentage
            options: HashMap::new(),
        },
    );

    // CB-126: High-latency tests
    checks.insert(
        "cb-126".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: Some(5.0), // seconds
            options: HashMap::new(),
        },
    );

    // CB-127: Expensive coverage
    checks.insert(
        "cb-127".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: Some(10.0), // minutes
            options: HashMap::new(),
        },
    );

    // CB-128: Dead code detection
    checks.insert(
        "cb-128".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: Some(1.0), // Max dead code percentage
            options: HashMap::new(),
        },
    );

    // CB-140: Mono-spec structure enforcement
    checks.insert(
        "cb-140".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: Some(500.0), // Max lines per spec file
            options: HashMap::new(),
        },
    );

    // CB-141: Memory profiling infrastructure
    checks.insert(
        "cb-141".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-142: SWE-CI EvoScore
    checks.insert(
        "cb-142".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Info,
            threshold: Some(0.5), // Minimum EvoScore for pass
            options: HashMap::new(),
        },
    );

    // CB-200: TDG Grade Gate (#214) — "A" or Fail
    checks.insert(
        "cb-200".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Error,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-300: Muda Waste Score (COMPLY-040)
    checks.insert(
        "cb-300".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: Some(60.0), // Max acceptable waste score
            options: HashMap::new(),
        },
    );

    // CB-301: Reproducibility Level (COMPLY-041)
    checks.insert(
        "cb-301".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: None, // Level-based (None/Bronze/Silver/Gold)
            options: HashMap::new(),
        },
    );

    // CB-302: Golden Trace Drift (COMPLY-042)
    checks.insert(
        "cb-302".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Error,
            threshold: None, // Pass/fail based on renacer trace validation
            options: HashMap::new(),
        },
    );

    // CB-303: EDD Compliance (COMPLY-043)
    checks.insert(
        "cb-303".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Warning,
            threshold: Some(80.0), // Minimum EDD compliance percentage
            options: HashMap::new(),
        },
    );

    // CB-1100: Custom Project Scores
    checks.insert(
        "cb-1100".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Error,
            threshold: None,
            options: HashMap::new(),
        },
    );

    // CB-1411: Comply Gate Effect (EV-1) — every severity=error rule must be
    // reachable from a required status check context, through an invocation
    // whose failure can still fail the job. Declared Error, and so a member of
    // the roster it verifies: a gate that does not gate itself is the exact
    // defect this rule exists to find.
    checks.insert(
        "cb-1411".to_string(),
        CheckConfig {
            enabled: true,
            severity: CheckSeverity::Error,
            threshold: None,
            options: HashMap::new(),
        },
    );

    checks
}
