// ---- CB-519: Lossy Data Pipeline ----

#[test]
fn test_cb519_detects_quantize_dequantize_roundtrip() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("export.rs"),
        r#"
fn convert_tensor(data: &[f32]) -> Vec<u8> {
let quantized = quantize_q4(data);
let dequantized = dequantize_q4(&quantized);
pack_bytes(&dequantized)
}
"#,
    )
    .unwrap();

    let violations = detect_cb519_lossy_data_pipeline(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-519");
    assert!(violations[0].description.contains("quantize"));
}

#[test]
fn test_cb519_detects_encode_decode_roundtrip() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("pipeline.rs"),
        r#"
fn process(data: &[u8]) -> Vec<u8> {
let encoded = encode_base64(data);
let decoded = decode_base64(&encoded);
decoded
}
"#,
    )
    .unwrap();

    let violations = detect_cb519_lossy_data_pipeline(temp.path());
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_cb519_allows_single_direction() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("quant.rs"),
        r#"
fn compress(data: &[f32]) -> Vec<u8> {
quantize_q4(data)
}
"#,
    )
    .unwrap();

    let violations = detect_cb519_lossy_data_pipeline(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb519_skips_test_code() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"
#[cfg(test)]
mod tests {
#[test]
fn test_roundtrip() {
    let q = quantize(data);
    let d = dequantize(&q);
    assert_eq!(data, d);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb519_lossy_data_pipeline(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-520: Expensive Init in Hot Path ----

#[test]
fn test_cb520_detects_new_in_loop() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("handler.rs"),
        r#"
fn process(items: &[Item]) {
for item in items {
    let client = HttpClient::new(config);
    let conn = Database::connect("url");
    client.send(item);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb520_expensive_init_in_loop(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-520");
}

#[test]
fn test_cb520_allows_single_init() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("simple.rs"),
        r#"
fn process(items: &[Item]) {
for item in items {
    let result = String::new();
    process_item(item, &result);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb520_expensive_init_in_loop(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb520_skips_test_code() {
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
        let c = Client::new();
        let d = Database::connect("url");
        let f = File::open("test");
    }
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb520_expensive_init_in_loop(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-521: Format Detection Without Magic Bytes ----

#[test]
fn test_cb521_detects_binary_read_without_magic() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("parser.rs"),
        r#"
fn parse_file(reader: &mut impl Read) -> Result<Header, Error> {
let mut buf = [0u8; 8];
reader.read_exact(&mut buf)?;
let size = u64::from_le_bytes(buf);
let mut data = vec![0u8; size as usize];
reader.read_exact(&mut data)?;
Ok(Header { data })
}
"#,
    )
    .unwrap();

    let violations = detect_cb521_format_without_magic_bytes(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-521");
}

#[test]
fn test_cb521_allows_with_magic_check() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("safe_parser.rs"),
        r#"
fn parse_file(reader: &mut impl Read) -> Result<Header, Error> {
let mut magic = [0u8; 4];
reader.read_exact(&mut magic)?;
if &magic != FILE_MAGIC {
    return Err(Error::InvalidFormat);
}
let mut buf = [0u8; 8];
reader.read_exact(&mut buf)?;
Ok(Header { size: u64::from_le_bytes(buf) })
}
"#,
    )
    .unwrap();

    let violations = detect_cb521_format_without_magic_bytes(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb521_skips_test_code() {
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
    let mut buf = [0u8; 8];
    cursor.read_exact(&mut buf).unwrap();
    let val = u64::from_le_bytes(buf);
}
}
"#,
    )
    .unwrap();

    let violations = detect_cb521_format_without_magic_bytes(temp.path());
    assert!(violations.is_empty());
}

// ---- CB-522: Untested Path Normalization ----

#[test]
fn test_cb522_detects_path_manipulation_chains() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("uri.rs"),
        r#"
fn normalize_uri(uri: &str) -> String {
let without_scheme = uri.strip_prefix("http://").unwrap_or(uri);
let cleaned = without_scheme.replace("//", "/");
let no_resolve = cleaned.replace("resolve/", "");
let trimmed = no_resolve.trim_start_matches("http://");
trimmed.to_string()
}
"#,
    )
    .unwrap();

    let violations = detect_cb522_untested_path_normalization(temp.path());
    assert!(!violations.is_empty());
    assert_eq!(violations[0].pattern_id, "CB-522");
}

#[test]
fn test_cb522_allows_simple_path_ops() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("simple.rs"),
        r#"
fn get_name(path: &Path) -> &str {
path.file_name().unwrap().to_str().unwrap()
}
"#,
    )
    .unwrap();

    let violations = detect_cb522_untested_path_normalization(temp.path());
    assert!(violations.is_empty());
}

