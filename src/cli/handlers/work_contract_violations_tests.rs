// Tests for work_contract_violations.rs (DBC spec §14.7)

#[test]
fn test_violation_type_display() {
    assert_eq!(ViolationType::CommandFailure.to_string(), "command_failure");
    assert_eq!(ViolationType::TimingAnomaly.to_string(), "timing_anomaly");
    assert_eq!(ViolationType::TrustViolation.to_string(), "trust_violation");
}

#[test]
fn test_violation_tracker_default() {
    let tracker = ViolationTracker::default();
    assert!(tracker.violations.is_empty());
    assert!(tracker.timings.is_empty());
}

#[test]
fn test_record_failure() {
    let mut tracker = ViolationTracker::default();
    tracker.record_failure("TEST-001", "cargo test", 1, "test failed");

    assert_eq!(tracker.violations.len(), 1);
    assert_eq!(tracker.violations[0].violation_type, ViolationType::CommandFailure);
    assert_eq!(tracker.violations[0].exit_code, Some(1));
    assert_eq!(tracker.violations[0].command, "cargo test");
}

#[test]
fn test_record_execution_no_anomaly() {
    let mut tracker = ViolationTracker::default();
    // First execution — no history, can't be anomalous
    let anomalous = tracker.record_execution("TEST-001", "cargo build", 5000);
    assert!(!anomalous);
    assert!(tracker.violations.is_empty());

    // Timings recorded
    assert!(tracker.timings.contains_key("cargo build"));
    assert_eq!(tracker.timings["cargo build"].durations.len(), 1);
}

#[test]
fn test_record_execution_timing_anomaly() {
    let mut tracker = ViolationTracker::default();

    // Build up normal history (10 samples ~1000ms)
    for _ in 0..10 {
        tracker.record_execution("TEST-001", "npm test", 1000);
    }
    assert!(tracker.violations.is_empty());

    // Huge spike should trigger anomaly
    let anomalous = tracker.record_execution("TEST-001", "npm test", 100_000);
    assert!(anomalous);
    assert_eq!(tracker.violations.len(), 1);
    assert_eq!(tracker.violations[0].violation_type, ViolationType::TimingAnomaly);
    assert!(tracker.violations[0].duration_ms.is_some());
}

#[test]
fn test_record_trust_violation() {
    let mut tracker = ViolationTracker::default();
    tracker.record_trust_violation("TEST-001", ".dbc-stack.toml", "manifest hash changed");

    assert_eq!(tracker.violations.len(), 1);
    assert_eq!(tracker.violations[0].violation_type, ViolationType::TrustViolation);
    assert_eq!(tracker.violations[0].command, ".dbc-stack.toml");
}

#[test]
fn test_violation_summary() {
    let mut tracker = ViolationTracker::default();
    tracker.record_failure("TEST-001", "cmd1", 1, "fail");
    tracker.record_failure("TEST-001", "cmd2", 2, "fail");
    tracker.record_trust_violation("TEST-001", "manifest", "changed");

    let summary = tracker.summary(1.0);
    assert_eq!(summary.total_violations, 3);
    assert_eq!(summary.command_failures, 2);
    assert_eq!(summary.timing_anomalies, 0);
    assert_eq!(summary.trust_violations, 1);
    assert!(summary.elevated); // 3 > 1.0 * 2
}

#[test]
fn test_violation_summary_not_elevated() {
    let mut tracker = ViolationTracker::default();
    tracker.record_failure("TEST-001", "cmd1", 1, "fail");

    let summary = tracker.summary(5.0);
    assert_eq!(summary.total_violations, 1);
    assert!(!summary.elevated); // 1 < 5.0 * 2
}

#[test]
fn test_command_timing_new() {
    let timing = CommandTiming::new("cargo test".to_string(), 3000);
    assert_eq!(timing.mean_ms, 3000.0);
    assert_eq!(timing.std_dev_ms, 0.0);
    assert_eq!(timing.durations.len(), 1);
}

#[test]
fn test_command_timing_record() {
    let mut timing = CommandTiming::new("cargo test".to_string(), 1000);
    timing.record(2000);
    timing.record(3000);

    assert_eq!(timing.durations.len(), 3);
    assert!((timing.mean_ms - 2000.0).abs() < f64::EPSILON);
    assert!(timing.std_dev_ms > 0.0);
}

#[test]
fn test_command_timing_is_anomalous_insufficient_data() {
    let timing = CommandTiming::new("cmd".to_string(), 1000);
    // Only 1 data point — can't detect anomaly
    assert!(!timing.is_anomalous(999999));
}

#[test]
fn test_command_timing_is_anomalous_with_data() {
    let mut timing = CommandTiming::new("cmd".to_string(), 100);
    for _ in 0..20 {
        timing.record(100);
    }
    // Mean ~100, std_dev ~0, so 100000 is way out
    assert!(timing.is_anomalous(100000));
    // Normal value should not be anomalous
    assert!(!timing.is_anomalous(100));
}

#[test]
fn test_command_timing_caps_at_50() {
    let mut timing = CommandTiming::new("cmd".to_string(), 100);
    for i in 0..100 {
        timing.record(100 + i);
    }
    assert_eq!(timing.durations.len(), 50);
}

#[test]
fn test_trust_chain_entry_new() {
    let entry = TrustChainEntry::new(".dbc-stack.toml", "abc123", "");
    assert_eq!(entry.manifest_path, ".dbc-stack.toml");
    assert_eq!(entry.content_hash, "abc123");
    assert_eq!(entry.prev_hash, "");
    assert!(!entry.chain_hash.is_empty());
}

#[test]
fn test_trust_chain_entry_verify() {
    let entry = TrustChainEntry::new(".dbc-stack.toml", "abc123", "prev456");
    assert!(entry.verify());

    // Tampered entry should fail
    let mut tampered = entry.clone();
    tampered.content_hash = "tampered".to_string();
    assert!(!tampered.verify());
}

#[test]
fn test_trust_chain_linking() {
    let entry1 = TrustChainEntry::new(".dbc-stack.toml", "hash1", "");
    let entry2 = TrustChainEntry::new(".dbc-stack.toml", "hash2", &entry1.chain_hash);

    assert!(entry1.verify());
    assert!(entry2.verify());
    assert_eq!(entry2.prev_hash, entry1.chain_hash);
}

#[test]
fn test_violation_tracker_save_load() {
    let tmp = tempfile::tempdir().unwrap();
    let mut tracker = ViolationTracker::default();
    tracker.record_failure("TEST-001", "cmd", 1, "fail");

    tracker.save(tmp.path(), "TEST-001").unwrap();

    let loaded = ViolationTracker::load(tmp.path(), "TEST-001");
    assert_eq!(loaded.violations.len(), 1);
    assert_eq!(loaded.violations[0].command, "cmd");
}

#[test]
fn test_violation_tracker_load_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let loaded = ViolationTracker::load(tmp.path(), "NONEXISTENT");
    assert!(loaded.violations.is_empty());
}

#[test]
fn test_runtime_violation_serialization() {
    let violation = RuntimeViolation {
        work_item_id: "TEST-001".to_string(),
        timestamp: "12345Z".to_string(),
        command: "cargo test".to_string(),
        violation_type: ViolationType::CommandFailure,
        duration_ms: None,
        exit_code: Some(1),
        message: "test failed".to_string(),
    };
    let json = serde_json::to_string(&violation).unwrap();
    let deserialized: RuntimeViolation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.work_item_id, "TEST-001");
    assert_eq!(deserialized.violation_type, ViolationType::CommandFailure);
}

#[test]
fn test_violation_summary_serialization() {
    let summary = ViolationSummary {
        total_violations: 5,
        command_failures: 3,
        timing_anomalies: 1,
        trust_violations: 1,
        violations_per_session_avg: 2.0,
        elevated: true,
    };
    let json = serde_json::to_string(&summary).unwrap();
    let deserialized: ViolationSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_violations, 5);
    assert!(deserialized.elevated);
}
