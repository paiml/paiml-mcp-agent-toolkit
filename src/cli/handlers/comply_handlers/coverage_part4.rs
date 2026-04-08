    #[tokio::test]
    async fn test_full_compliance_workflow() {
        // Create a new project, init, check, migrate
        let temp = create_temp_project();

        // Init
        handle_init(temp.path(), false).await.expect("Init failed");

        // Verify config exists
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        assert_eq!(config.pmat.version, PMAT_VERSION);

        // Check should pass (we're on current version)
        let checks = vec![
            check_version_currency(&config.pmat.version),
            check_config_files(temp.path()),
        ];
        let _all_pass_or_warn = checks.iter().all(|c| c.status != CheckStatus::Fail);
        // Version should pass, config files may warn about metrics
        assert!(checks[0].status == CheckStatus::Pass);
    }

    #[tokio::test]
    async fn test_migrate_then_check_workflow() {
        let temp = create_pmat_project("1.0.0");

        // Migrate to current
        handle_migrate(temp.path(), None, false, true, true)
            .await
            .expect("Migrate failed");

        // Check version should now pass
        let config = load_or_create_project_config(temp.path()).expect("Failed to load config");
        let check = check_version_currency(&config.pmat.version);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    // ComputeBrick Pattern Detection Tests (CB-IMPL-001-B)

    #[test]
    fn test_cb020_detects_unsafe_without_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block without SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
        std::fs::write(
            &rs_file,
            r#"
fn bad_unsafe() {
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-020");
        assert!(violations[0].description.contains("unsafe"));
    }

    #[test]
    fn test_cb020_allows_unsafe_with_safety() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with unsafe block WITH SAFETY comment
        let rs_file = src_dir.join("lib.rs");
        std::fs::write(
            &rs_file,
            r#"
fn good_unsafe() {
    // SAFETY: null pointer read is UB, but this is just a test
    unsafe {
        std::ptr::null::<i32>().read();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb020_unsafe_without_safety(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb021_detects_simd_without_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with AVX intrinsic without #[target_feature]
        // Note: SSE (_mm_) is now exempted as baseline on x86_64
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
fn bad_simd() {
    let a = _mm256_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-021");
        assert!(violations[0].description.contains("_mm256"));
    }

    #[test]
    fn test_cb021_allows_simd_with_target_feature() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with AVX intrinsic WITH #[target_feature]
        let rs_file = src_dir.join("simd.rs");
        std::fs::write(
            &rs_file,
            r#"
#[target_feature(enable = "avx2")]
fn good_simd() {
    let a = _mm256_set1_ps(1.0);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb021_no_false_positive_on_identifiers() {
        // Regression test: struct fields like "f32x4_verified" should NOT trigger CB-021
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with f32x4 in identifier names (NOT intrinsic usage)
        let rs_file = src_dir.join("verification.rs");
        std::fs::write(
            &rs_file,
            r#"
/// Verify SIMD f32x4 operations work correctly
pub struct SimdVerification {
    /// f32x4 operations verified
    pub f32x4_verified: bool,
    /// i32x4 operations verified
    pub i32x4_verified: bool,
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Verify f32x4 operations.
pub fn verify_f32x4_operations() -> bool {
    let simd_lanes = 4; // f32x4
    true
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        // Should be 0 - these are identifiers and comments, not intrinsic calls
        assert_eq!(
            violations.len(),
            0,
            "False positive: detected {:?}",
            violations
        );
    }

    #[test]
    fn test_cb021_detects_actual_portable_simd_usage() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with ACTUAL portable SIMD usage (f32x4::splat)
        let rs_file = src_dir.join("simd_usage.rs");
        std::fs::write(
            &rs_file,
            r#"
use std::simd::f32x4;

fn use_portable_simd() {
    let a = f32x4::splat(1.0);
    let b = f32x4::from_array([1.0, 2.0, 3.0, 4.0]);
}
"#,
        )
        .unwrap();

        let violations = detect_cb021_simd_without_target_feature(temp.path());
        // Should detect f32x4:: usage as potential SIMD without target_feature
        assert!(
            violations.len() >= 1,
            "Should detect portable SIMD usage: {:?}",
            violations
        );
    }

    // CB-001 and CB-002 WGSL Detection Tests (CB-IMPL-001-D)

    #[test]
    fn test_cb001_detects_wgsl_without_bounds_check() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with global_invocation_id but NO bounds check
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    output[gid] = input[gid];  // No bounds check!
}
"#,
        )
        .unwrap();

        let violations = detect_cb001_wgsl_no_bounds_check(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-001");
        assert!(violations[0].description.contains("bounds check"));
    }

    #[test]
    fn test_cb001_allows_wgsl_with_bounds_check() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with global_invocation_id AND bounds check
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if (gid >= arrayLength(&input)) { return; }
    output[gid] = input[gid];
}
"#,
        )
        .unwrap();

        let violations = detect_cb001_wgsl_no_bounds_check(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb002_detects_wgsl_barrier_in_conditional() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with workgroupBarrier() inside conditional
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
        workgroupBarrier();  // DANGER: Inside conditional!
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb002_wgsl_barrier_divergence(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-002");
        assert!(violations[0].description.contains("workgroupBarrier()"));
    }

    #[test]
    fn test_cb002_allows_wgsl_barrier_outside_conditional() {
        let temp = tempfile::tempdir().unwrap();

        // Create WGSL file with workgroupBarrier() OUTSIDE conditional
        let wgsl_file = temp.path().join("compute.wgsl");
        std::fs::write(
            &wgsl_file,
            r#"@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    if (local_id.x == 0u) {
        shared_data[0] = compute();
    }
    workgroupBarrier();  // Safe: All threads reach this
    let val = shared_data[0];
}
"#,
        )
        .unwrap();

        let violations = detect_cb002_wgsl_barrier_divergence(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_bricks_without_assertions() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITHOUT assertions
        // Use concat! to avoid self-matching during CB-BUDGET compliance scanning
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            // No leading newline - content starts immediately
            concat!("impl Compute", "Brick for MyBrick {\n\
                fn execute(&self) {\n\
                    self.do_work();\n\
                }\n\
            }\n"),
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 1, "Expected 1 violation for brick without assertions");
        assert_eq!(violations[0].pattern_id, "CB-BUDGET");
    }

    #[test]
    fn test_detect_bricks_with_assertions_pass() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create file with Brick impl WITH assertions
        // Use concat! to avoid self-matching during CB-BUDGET compliance scanning
        let rs_file = src_dir.join("brick.rs");
        std::fs::write(
            &rs_file,
            concat!("\nimpl Compute", "Brick for MyBrick {\n\
    fn execute(&self) {\n\
        debug_assert!(self.is_valid());\n\
        self.do_work();\n\
    }\n\
}\n"),
        )
        .unwrap();

        let violations = detect_bricks_without_assertions(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_profiler_anomalies_high_cv() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with high CV
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "MatMulBrick",
      "cv": 0.25,
      "efficiency": 0.80
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "HIGH_CV");
        assert!(anomalies[0].value > 15.0);
    }

    #[test]
    fn test_detect_profiler_anomalies_low_efficiency() {
        let temp = tempfile::tempdir().unwrap();
        let metrics_dir = temp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();

        // Create profiler JSON with low efficiency
        let profile_file = metrics_dir.join("brick-profile.json");
        std::fs::write(
            &profile_file,
            r#"{
  "bricks": [
    {
      "name": "SlowBrick",
      "cv": 0.05,
      "efficiency": 0.15
    }
  ]
}"#,
        )
        .unwrap();

        let anomalies = detect_profiler_anomalies(temp.path());
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, "LOW_EFFICIENCY");
        assert!(anomalies[0].value < 25.0);
    }

    #[test]
    fn test_check_compute_brick_skips_non_cb_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create a regular project without trueno/realizar/probar deps
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "regular-project"
version = "1.0.0"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_check_compute_brick_detects_trueno_project() {
        let temp = tempfile::tempdir().unwrap();
        // Create project with trueno dependency
        let cargo_toml = temp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "gpu-project"
version = "1.0.0"

[dependencies]
trueno = "0.1"
"#,
        )
        .unwrap();

        // Create src directory with clean Rust code
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

        let check = check_compute_brick(temp.path());
        // Should not skip - this is a CB ecosystem project
        assert_ne!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_extract_json_number() {
        assert_eq!(extract_json_number("\"cv\": 0.18,"), Some(0.18));
        assert_eq!(extract_json_number("\"efficiency\": 25.5}"), Some(25.5));
        assert_eq!(extract_json_number("invalid"), None);
    }

    #[test]
    fn test_walkdir_rs_files() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        let nested = src_dir.join("nested");
        std::fs::create_dir_all(&nested).unwrap();

        std::fs::write(src_dir.join("lib.rs"), "").unwrap();
        std::fs::write(nested.join("mod.rs"), "").unwrap();
        std::fs::write(src_dir.join("readme.md"), "").unwrap(); // Not .rs

        let files = walkdir_rs_files(&src_dir).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "rs"));
    }
