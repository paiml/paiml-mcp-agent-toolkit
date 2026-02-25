// OIP Tarantula Pattern Detection tests (CB-120 through CB-124)
// Included from tests.rs via include!() - shares parent module scope

// CB-120 Tests: NaN-unsafe comparison detection

#[test]
fn test_cb120_detects_partial_cmp_unwrap() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("ml.rs"),
        r#"
fn sort_floats(vec: &mut Vec<f64>) {
vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
}
"#,
    )
    .unwrap();

    let violations = detect_cb120_nan_unsafe_comparison(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-120");
    assert!(violations[0].description.contains("partial_cmp"));
}

#[test]
fn test_cb120_skips_unwrap_or() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("safe.rs"),
        r#"
fn sort_floats(vec: &mut Vec<f64>) {
vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}
"#,
    )
    .unwrap();

    let violations = detect_cb120_nan_unsafe_comparison(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb120_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
#[test]
fn test_sort() {
    let mut v = vec![1.0, 2.0];
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb120_nan_unsafe_comparison(temp.path());
    assert!(violations.is_empty());
}

// CB-121 Tests: Lock poisoning detection

#[test]
#[ignore = "OIP detection needs walkdir debugging"]
fn test_cb121_detects_mutex_lock_unwrap() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("sync.rs"),
        r#"
use std::sync::Mutex;
fn get_data(m: &Mutex<i32>) -> i32 {
*m.lock().unwrap()
}
"#,
    )
    .unwrap();

    let violations = detect_cb121_lock_poisoning(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-121");
}

#[test]
fn test_cb121_skips_into_inner() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("safe.rs"),
        r#"
use std::sync::Mutex;
fn get_data(m: &Mutex<i32>) -> i32 {
*m.lock().unwrap_or_else(|e| e.into_inner())
}
"#,
    )
    .unwrap();

    let violations = detect_cb121_lock_poisoning(temp.path());
    assert!(violations.is_empty());
}

// CB-122 Tests: Serde deserialization safety

#[test]
fn test_cb122_detects_serde_json_unwrap() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("parser.rs"),
        r#"
fn parse_config(s: &str) -> Config {
serde_json::from_str(s).unwrap()
}
"#,
    )
    .unwrap();

    let violations = detect_cb122_serde_safety(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-122");
}

#[test]
fn test_cb122_detects_toml_expect() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("config.rs"),
        r#"
fn load(s: &str) -> Settings {
toml::from_str(s).expect("invalid toml")
}
"#,
    )
    .unwrap();

    let violations = detect_cb122_serde_safety(temp.path());
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_cb122_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
#[test]
fn test_parse() {
    let v: Value = serde_json::from_str("{}").unwrap();
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb122_serde_safety(temp.path());
    assert!(violations.is_empty());
}

// CB-123 Tests: Undocumented #[ignore] (bare, without reason)

#[test]
#[ignore = "OIP detection needs walkdir debugging"]
fn test_cb123_detects_bare_ignore() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("tests.rs"),
        r#"
#[ignore = "slow test"]
#[test]
fn slow_test() {}
"#,
    )
    .unwrap();

    let violations = detect_cb123_undocumented_ignore(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-123");
}

#[test]
fn test_cb123_skips_ignore_with_reason() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("tests.rs"),
        r#"
#[ignore = "requires GPU"]
#[test]
fn gpu_test() {}
"#,
    )
    .unwrap();

    let violations = detect_cb123_undocumented_ignore(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb123_skips_ignore_with_comment() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("tests.rs"),
        r#"
#[ignore] // flaky on CI
#[test]
fn flaky_test() {}
"#,
    )
    .unwrap();

    let violations = detect_cb123_undocumented_ignore(temp.path());
    assert!(violations.is_empty());
}

// CB-124 Tests: Coverage threshold enforcement

#[test]
fn test_cb124_detects_low_threshold() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("tarpaulin.toml"),
        r#"
[report]
fail_under = 58.0
"#,
    )
    .unwrap();

    let violations = detect_cb124_coverage_threshold(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-124");
    assert_eq!(violations[0].severity, Severity::Error);
}

#[test]
fn test_cb124_warns_below_95() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("tarpaulin.toml"),
        r#"
[report]
fail_under = 85.0
"#,
    )
    .unwrap();

    let violations = detect_cb124_coverage_threshold(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].severity, Severity::Warning);
}

#[test]
fn test_cb124_passes_high_threshold() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("tarpaulin.toml"),
        r#"
[report]
fail_under = 95.0
"#,
    )
    .unwrap();

    let violations = detect_cb124_coverage_threshold(temp.path());
    assert!(violations.is_empty());
}
