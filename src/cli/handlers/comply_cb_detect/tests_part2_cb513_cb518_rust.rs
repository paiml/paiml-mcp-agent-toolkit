// ---- CB-513: Silent Error Swallowing ----

#[test]
fn test_cb513_detects_unwrap_or_else_discard() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("config.rs"),
        r#"
fn load_config() -> Config {
let val = std::env::var("KEY").unwrap_or_else(|_| "default".to_string());
Config { val }
}
"#,
    )
    .unwrap();

    let violations = detect_cb513_silent_error_swallowing(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-513");
    assert!(violations[0].description.contains("unwrap_or_else"));
}

#[test]
fn test_cb513_detects_map_err_discard() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("io.rs"),
        r#"
fn read_file() -> Result<String, MyError> {
fs::read_to_string("f.txt").map_err(|_| MyError::IoFailed)
}
"#,
    )
    .unwrap();

    let violations = detect_cb513_silent_error_swallowing(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("map_err"));
}

#[test]
fn test_cb513_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
#[test]
fn test_it() {
    let v = "123".parse::<i32>().unwrap_or_else(|_| 0);
    assert_eq!(v, 123);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb513_silent_error_swallowing(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb513_skips_comments() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
// Note: we could use .unwrap_or_else(|_| default) here
fn foo() -> i32 { 42 }
"#,
    )
    .unwrap();

    let violations = detect_cb513_silent_error_swallowing(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-514: Debug Eprintln Leaks ----

#[test]
fn test_cb514_detects_debug_eprintln() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("parser.rs"),
        "fn parse(input: &str) {\n    eprintln!(\"[DEBUG] parsing: {}\", input);\n}\n",
    )
    .unwrap();

    let violations = detect_cb514_debug_eprintln_leaks(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-514");
}

#[test]
fn test_cb514_detects_trace_eprintln() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("net.rs"),
        "fn connect() {\n    eprintln!(\"[TRACE] connecting to server\");\n}\n",
    )
    .unwrap();

    let violations = detect_cb514_debug_eprintln_leaks(temp.path());
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_cb514_allows_normal_eprintln() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("main.rs"),
        "fn main() {\n    eprintln!(\"Error: file not found\");\n}\n",
    )
    .unwrap();

    let violations = detect_cb514_debug_eprintln_leaks(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb514_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        eprintln!(\"[DEBUG] test output\");\n    }\n}\n",
    )
    .unwrap();

    let violations = detect_cb514_debug_eprintln_leaks(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-515: Catch-All Match Default ----

#[test]
fn test_cb515_detects_concrete_catch_all() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("arch.rs"),
        r#"
fn get_arch(name: &str) -> Architecture {
match name {
    "gpt" => Architecture::Gpt,
    "llama" => Architecture::Llama,
    _ => Architecture::Qwen2,
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb515_catch_all_match_default(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-515");
    assert!(violations[0].description.contains("Architecture::Qwen2"));
}

#[test]
fn test_cb515_allows_error_catch_all() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("parse.rs"),
        r#"
fn parse_mode(s: &str) -> Result<Mode, Error> {
match s {
    "fast" => Ok(Mode::Fast),
    "slow" => Ok(Mode::Slow),
    _ => Err(Error::UnknownMode(s.to_string())),
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb515_catch_all_match_default(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb515_allows_none_catch_all() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lookup.rs"),
        r#"
fn find(key: &str) -> Option<Value> {
match key {
    "a" => Some(Value::A),
    _ => None,
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb515_catch_all_match_default(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb515_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
#[test]
fn test_match() {
    let result = match "x" {
        "a" => 1,
        _ => 99,
    };
    assert_eq!(result, 99);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb515_catch_all_match_default(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-516: Hardcoded Magic Numbers ----

#[test]
fn test_cb516_detects_magic_in_some() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("config.rs"),
        r#"
fn default_config() -> Config {
Config {
    rope_theta: Some(10000.0),
    max_seq_len: Some(2048),
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
    assert!(violations.len() >= 1);
    assert_eq!(violations[0].pattern_id, "CB-516");
    assert!(violations[0].description.contains("10000.0"));
}

#[test]
fn test_cb516_skips_const_declarations() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        "const MAX_RETRY: usize = 10000;\nstatic TIMEOUT: u64 = 30000;\n",
    )
    .unwrap();

    let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb516_skips_common_values() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("buf.rs"),
        r#"
fn create_buffer() -> Buffer {
Buffer { size: Some(1024) }
}
"#,
    )
    .unwrap();

    let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb516_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
#[test]
fn t() {
    let x = Some(99999.0);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-517: Stale Debug Artifacts ----

#[test]
fn test_cb517_detects_atomic_counter() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("metrics.rs"),
        r#"
use std::sync::atomic::AtomicUsize;
static DEBUG_COUNTER: AtomicUsize = AtomicUsize::new(0);
fn process() {
DEBUG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
"#,
    )
    .unwrap();

    let violations = detect_cb517_stale_debug_artifacts(temp.path());
    assert!(violations.len() >= 1);
    assert_eq!(violations[0].pattern_id, "CB-517");
    assert!(violations[0].description.contains("Atomic"));
}

#[test]
fn test_cb517_detects_allow_unused_static() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("debug.rs"),
        "#[allow(unused)]\nstatic TRACE_LOG: bool = false;\n",
    )
    .unwrap();

    let violations = detect_cb517_stale_debug_artifacts(temp.path());
    assert!(violations.len() >= 1);
    assert!(violations[0].description.contains("allow(unused)"));
}

#[test]
fn test_cb517_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
use std::sync::atomic::AtomicUsize;
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
#[test]
fn t() {}
}
"#,
    )
    .unwrap();

    let violations = detect_cb517_stale_debug_artifacts(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-518: Expensive Clone in Loop ----

#[test]
fn test_cb518_detects_excessive_clones_in_loop() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("process.rs"),
        r#"
fn process(items: &[Item], config: &Config) {
for item in items {
    let a = config.name.clone();
    let b = config.path.clone();
    let c = config.data.clone();
    let d = config.extra.clone();
    do_work(item, &a, &b, &c, &d);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb518_expensive_clone_in_loop(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-518");
    assert!(violations[0].description.contains("4 .clone()"));
}

#[test]
fn test_cb518_allows_few_clones() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("small.rs"),
        r#"
fn process(items: &[Item]) {
for item in items {
    let name = item.name.clone();
    process_name(&name);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb518_expensive_clone_in_loop(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb518_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
#[test]
fn t() {
    for i in 0..10 {
        let a = "x".to_string().clone();
        let b = "y".to_string().clone();
        let c = "z".to_string().clone();
        let d = "w".to_string().clone();
    }
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb518_expensive_clone_in_loop(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb518_detects_while_loop() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("iter.rs"),
        r#"
fn drain(queue: &mut Vec<Job>, cfg: &Config) {
while let Some(job) = queue.pop() {
    let a = cfg.name.clone();
    let b = cfg.path.clone();
    let c = cfg.data.clone();
    let d = cfg.meta.clone();
    run(job, a, b, c, d);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb518_expensive_clone_in_loop(temp.path());
    assert_eq!(violations.len(), 1);
}
