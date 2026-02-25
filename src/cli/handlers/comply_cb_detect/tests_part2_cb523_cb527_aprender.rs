// ---- CB-523: External Config Over Embedded Metadata ----

#[test]
fn test_cb523_detects_sibling_config_discovery() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("loader.rs"),
        r#"
fn load_model(path: &Path) -> Model {
let config_path = path.with_file_name("config.json");
let config = fs::read_to_string(config_path).unwrap();
parse_model(config)
}
"#,
    )
    .unwrap();

    let violations = detect_cb523_external_config_over_embedded(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-523");
}

#[test]
fn test_cb523_allows_non_config_file_ops() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("util.rs"),
        r#"
fn get_log_path(path: &Path) -> PathBuf {
path.with_file_name("output.log")
}
"#,
    )
    .unwrap();

    let violations = detect_cb523_external_config_over_embedded(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-524: Incomplete Enum Match Coverage ----

#[test]
fn test_cb524_detects_multiple_wildcard_matches() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("dispatch.rs"),
        r#"
fn get_name(arch: Architecture) -> &'static str {
match arch {
    Architecture::Gpt => "gpt",
    Architecture::Llama => "llama",
    _ => "unknown",
}
}
fn get_layers(arch: Architecture) -> usize {
match arch {
    Architecture::Gpt => 12,
    _ => 32,
}
}
fn get_hidden(arch: Architecture) -> usize {
match arch {
    Architecture::Llama => 4096,
    _ => 768,
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb524_incomplete_enum_match(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-524");
    assert!(violations[0].description.contains("3"));
}

#[test]
fn test_cb524_allows_few_wildcard_matches() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("simple.rs"),
        r#"
fn name(x: Kind) -> &'static str {
match x {
    Kind::A => "a",
    _ => "other",
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb524_incomplete_enum_match(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb524_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
fn a() -> i32 { match x { X::A => 1, _ => 2 } }
fn b() -> i32 { match x { X::B => 3, _ => 4 } }
fn c() -> i32 { match x { X::C => 5, _ => 6 } }
}
"#,
    )
    .unwrap();

    let violations = detect_cb524_incomplete_enum_match(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-525: Hardcoded Field Names Without Aliases ----

#[test]
fn test_cb525_detects_many_get_without_fallback() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("config.rs"),
        r#"
fn parse_config(json: &serde_json::Value) -> Config {
let hidden = json.get("hidden_size").unwrap();
let layers = json.get("num_hidden_layers").unwrap();
let heads = json.get("num_attention_heads").unwrap();
let vocab = json.get("vocab_size").unwrap();
let intermediate = json.get("intermediate_size").unwrap();
Config { hidden, layers, heads, vocab, intermediate }
}
"#,
    )
    .unwrap();

    let violations = detect_cb525_hardcoded_field_names(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-525");
}

#[test]
fn test_cb525_allows_with_or_fallback() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("config.rs"),
        r#"
fn parse_config(json: &serde_json::Value) -> Config {
let hidden = json.get("hidden_size").or_else(|| json.get("n_embd")).unwrap();
let layers = json.get("num_hidden_layers").or_else(|| json.get("n_layer")).unwrap();
let heads = json.get("num_attention_heads").or_else(|| json.get("n_head")).unwrap();
let vocab = json.get("vocab_size").unwrap();
let intermediate = json.get("intermediate_size").or(json.get("n_inner")).unwrap();
Config { hidden, layers, heads, vocab, intermediate }
}
"#,
    )
    .unwrap();

    let violations = detect_cb525_hardcoded_field_names(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb525_allows_few_gets() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("small.rs"),
        r#"
fn get_name(json: &serde_json::Value) -> String {
json.get("name").unwrap().as_str().unwrap().to_string()
}
"#,
    )
    .unwrap();

    let violations = detect_cb525_hardcoded_field_names(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-526: Single-Path File Resolution ----

#[test]
fn test_cb526_detects_single_path_exists() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("discovery.rs"),
        r#"
fn find_tokenizer(model_path: &Path) -> Option<PathBuf> {
if model_path.join("tokenizer.json").exists() {
    Some(model_path.join("tokenizer.json"))
} else {
    None
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb526_single_path_resolution(temp.path());
    assert!(violations.len() >= 1);
    assert_eq!(violations[0].pattern_id, "CB-526");
}

#[test]
fn test_cb526_allows_with_fallback() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("discovery.rs"),
        r#"
fn find_tokenizer(model_path: &Path) -> Option<PathBuf> {
let tok_path = model_path.join("tokenizer.json");
if tok_path.exists() || model_path.parent().map(|p| p.join("tokenizer.json").exists()).unwrap_or(false) {
    Some(tok_path)
} else {
    None
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb526_single_path_resolution(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-527: Incomplete Pattern List ----

#[test]
fn test_cb527_detects_classification_chain() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("classify.rs"),
        r#"
fn is_embedding(name: &str) -> bool {
name.contains("embed") || name.contains("wte") || name.contains("wpe") || name.contains("position")
}
"#,
    )
    .unwrap();

    let violations = detect_cb527_incomplete_pattern_list(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-527");
}

#[test]
fn test_cb527_allows_short_chains() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("simple.rs"),
        r#"
fn is_special(name: &str) -> bool {
name.contains("test") || name.contains("bench")
}
"#,
    )
    .unwrap();

    let violations = detect_cb527_incomplete_pattern_list(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb527_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
fn check(s: &str) -> bool {
    s.contains("a") || s.contains("b") || s.contains("c") || s.contains("d")
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb527_incomplete_pattern_list(temp.path());
    assert!(violations.is_empty());
}
