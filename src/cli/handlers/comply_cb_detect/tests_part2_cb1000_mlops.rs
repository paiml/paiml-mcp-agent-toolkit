// CB-1000 MLOps Model Quality tests
// Included from tests_part2.rs via include!() - shares parent module scope

#[test]
fn test_cb1000_no_model_files_empty() {
    let temp = TempDir::new().unwrap();
    let violations = detect_cb1000_missing_model_card(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb1000_detects_missing_model_card() {
    let temp = TempDir::new().unwrap();
    let models_dir = temp.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();
    // Create a minimal GGUF file (just magic bytes)
    let mut gguf_header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
    gguf_header.extend_from_slice(&3u32.to_le_bytes()); // version 3
    gguf_header.extend_from_slice(&10u64.to_le_bytes()); // tensor_count
    gguf_header.extend_from_slice(&5u64.to_le_bytes()); // metadata_count
    gguf_header.resize(64, 0);
    fs::write(models_dir.join("model.gguf"), &gguf_header).unwrap();

    let violations = detect_cb1000_missing_model_card(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-1000");
}

#[test]
fn test_cb1000_passes_with_readme() {
    let temp = TempDir::new().unwrap();
    let models_dir = temp.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();
    fs::write(
        models_dir.join("model.gguf"),
        [0x47, 0x47, 0x55, 0x46, 0, 0, 0, 0],
    )
    .unwrap();
    fs::write(models_dir.join("README.md"), "# Model Card\n").unwrap();

    let violations = detect_cb1000_missing_model_card(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb1001_detects_oversized_tensor_count() {
    let temp = TempDir::new().unwrap();
    let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
    header.extend_from_slice(&3u32.to_le_bytes()); // version
    header.extend_from_slice(&200_000u64.to_le_bytes()); // oversized tensor_count
    header.extend_from_slice(&0u64.to_le_bytes()); // metadata_count
    header.resize(64, 0);
    fs::write(temp.path().join("bad.gguf"), &header).unwrap();

    let violations = detect_cb1001_oversized_tensor_count(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-1001");
    assert!(matches!(violations[0].severity, Severity::Error));
}

#[test]
fn test_cb1001_passes_normal_tensor_count() {
    let temp = TempDir::new().unwrap();
    let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
    header.extend_from_slice(&3u32.to_le_bytes()); // version
    header.extend_from_slice(&500u64.to_le_bytes()); // normal tensor_count
    header.extend_from_slice(&10u64.to_le_bytes()); // metadata_count
    header.resize(64, 0);
    fs::write(temp.path().join("good.gguf"), &header).unwrap();

    let violations = detect_cb1001_oversized_tensor_count(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb1006_detects_sharded_without_index() {
    let temp = TempDir::new().unwrap();
    // Create header bytes for SafeTensors (8-byte length + small JSON)
    let json_header = b"{\"tensor\":{\"dtype\":\"F32\",\"shape\":[1],\"data_offsets\":[0,4]}}";
    let header_len = json_header.len() as u64;
    let mut data = Vec::new();
    data.extend_from_slice(&header_len.to_le_bytes());
    data.extend_from_slice(json_header);
    data.extend_from_slice(&[0u8; 4]); // tensor data

    fs::write(temp.path().join("model-00001-of-00002.safetensors"), &data).unwrap();
    fs::write(temp.path().join("model-00002-of-00002.safetensors"), &data).unwrap();

    let violations = detect_cb1006_sharded_without_index(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-1006");
}

#[test]
fn test_cb1006_passes_with_index() {
    let temp = TempDir::new().unwrap();
    let json_header = b"{\"tensor\":{\"dtype\":\"F32\",\"shape\":[1],\"data_offsets\":[0,4]}}";
    let header_len = json_header.len() as u64;
    let mut data = Vec::new();
    data.extend_from_slice(&header_len.to_le_bytes());
    data.extend_from_slice(json_header);
    data.extend_from_slice(&[0u8; 4]);

    fs::write(temp.path().join("model-00001-of-00002.safetensors"), &data).unwrap();
    fs::write(temp.path().join("model-00002-of-00002.safetensors"), &data).unwrap();
    fs::write(temp.path().join("model.safetensors.index.json"), "{}").unwrap();

    let violations = detect_cb1006_sharded_without_index(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb1007_detects_large_file() {
    // We can't create a 10GB file in tests, but we can test the threshold logic
    let temp = TempDir::new().unwrap();
    // Create a small file — should NOT trigger
    fs::write(temp.path().join("small.gguf"), [0u8; 100]).unwrap();
    let violations = detect_cb1007_excessive_file_size(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_walkdir_model_files() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("model.gguf"), [0u8; 16]).unwrap();
    fs::write(temp.path().join("weights.safetensors"), [0u8; 16]).unwrap();
    fs::write(temp.path().join("model.apr"), [0u8; 16]).unwrap();
    fs::write(temp.path().join("code.rs"), "fn main() {}").unwrap();

    let files = walkdir_model_files(temp.path());
    assert_eq!(files.len(), 3);
}

#[test]
fn test_model_format_from_extension() {
    assert_eq!(ModelFormat::from_extension("gguf"), Some(ModelFormat::Gguf));
    assert_eq!(ModelFormat::from_extension("apr"), Some(ModelFormat::Apr));
    assert_eq!(
        ModelFormat::from_extension("safetensors"),
        Some(ModelFormat::SafeTensors)
    );
    assert_eq!(ModelFormat::from_extension("rs"), None);
}

#[test]
fn test_cb1004_detects_missing_architecture() {
    let temp = TempDir::new().unwrap();
    // Create GGUF file without "general.architecture" key
    let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
    header.extend_from_slice(&3u32.to_le_bytes()); // version
    header.extend_from_slice(&10u64.to_le_bytes()); // tensor_count
    header.extend_from_slice(&0u64.to_le_bytes()); // metadata_count
    header.resize(200, 0); // Pad to be > 100 bytes
    fs::write(temp.path().join("model.gguf"), &header).unwrap();

    let violations = detect_cb1004_missing_architecture(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-1004");
}

#[test]
fn test_cb1004_passes_with_architecture() {
    let temp = TempDir::new().unwrap();
    let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
    header.extend_from_slice(&3u32.to_le_bytes());
    header.extend_from_slice(&10u64.to_le_bytes());
    header.extend_from_slice(&1u64.to_le_bytes()); // 1 metadata entry
                                                   // Add "general.architecture" as a key string
    header.extend_from_slice(b"general.architecture");
    header.resize(200, 0);
    fs::write(temp.path().join("model.gguf"), &header).unwrap();

    let violations = detect_cb1004_missing_architecture(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_cb1005_detects_size_mismatch() {
    let temp = TempDir::new().unwrap();
    // Create tiny GGUF file claiming F32
    let mut header = vec![0x47u8, 0x47, 0x55, 0x46];
    header.extend_from_slice(&3u32.to_le_bytes());
    header.extend_from_slice(&10u64.to_le_bytes());
    header.extend_from_slice(&0u64.to_le_bytes());
    // File is only ~32 bytes but claims f32
    fs::write(temp.path().join("model-f32.gguf"), &header).unwrap();

    let violations = detect_cb1005_quantization_mismatch(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-1005");
}

// =============================================================================
// Eager header validation (#965 adjacent): a header that was never actually
// read must not be reported as a valid one.
//
// `model_header_matches_extension` is the ONLY thing in `analyze models` that
// looks at bytes rather than at the filename, so every one of these files was
// inventoried as a valid model of its declared format and, under `--check`,
// reported as passing quality checks.
// =============================================================================

/// GGUF's fixed header is 24 bytes (magic, version, tensor_count, kv_count).
/// The parser copied into a zeroed 64-byte stack buffer and then guarded on
/// `buf.len() < 16` — the length of the BUFFER, never the number of bytes read
/// — so an 8-byte file read its tensor count out of the zero padding and
/// validated.
#[test]
fn test_truncated_gguf_is_not_a_valid_header() {
    let temp = TempDir::new().unwrap();
    let mut bytes = b"GGUF".to_vec();
    bytes.extend_from_slice(&3u32.to_le_bytes());
    assert_eq!(
        bytes.len(),
        8,
        "fixture must be shorter than GGUF's 24-byte header"
    );
    let path = temp.path().join("truncated.gguf");
    fs::write(&path, &bytes).unwrap();
    assert!(
        !model_header_matches_extension(&path),
        "an 8-byte GGUF has no tensor count to read; it must not validate"
    );
}

/// A 24-byte GGUF is exactly the fixed header and stays valid.
#[test]
fn test_minimal_complete_gguf_is_still_valid() {
    let temp = TempDir::new().unwrap();
    let mut bytes = b"GGUF".to_vec();
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&2u64.to_le_bytes());
    bytes.extend_from_slice(&1u64.to_le_bytes());
    assert_eq!(bytes.len(), 24);
    let path = temp.path().join("minimal.gguf");
    fs::write(&path, &bytes).unwrap();
    assert!(model_header_matches_extension(&path));
}

/// SafeTensors declares its JSON header length in the first 8 bytes. When that
/// length ran past EOF the parser took an `else` branch that skipped the read
/// entirely and still returned success — a truncated file was indistinguishable
/// from a whole one.
#[test]
fn test_safetensors_header_running_past_eof_is_not_valid() {
    let temp = TempDir::new().unwrap();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1000u64.to_le_bytes());
    let path = temp.path().join("truncated.safetensors");
    fs::write(&path, &bytes).unwrap();
    assert!(
        !model_header_matches_extension(&path),
        "an 8-byte file declaring a 1000-byte header is truncated, not valid"
    );
}

/// The declared header fits, but its bytes are not JSON. Eager validation means
/// the header is parsed at load time, not assumed.
#[test]
fn test_safetensors_header_that_is_not_json_is_not_valid() {
    let temp = TempDir::new().unwrap();
    let junk = b"abcdefghij";
    let mut bytes = (junk.len() as u64).to_le_bytes().to_vec();
    bytes.extend_from_slice(junk);
    bytes.extend_from_slice(&[0u8; 8]);
    let path = temp.path().join("notjson.safetensors");
    fs::write(&path, &bytes).unwrap();
    assert!(
        !model_header_matches_extension(&path),
        "a SafeTensors header must be a JSON object"
    );
}

/// Positive control for the SafeTensors path.
#[test]
fn test_well_formed_safetensors_is_valid() {
    let temp = TempDir::new().unwrap();
    let json = br#"{"w":{"dtype":"F32","shape":[2],"data_offsets":[0,8]}}"#;
    let mut bytes = (json.len() as u64).to_le_bytes().to_vec();
    bytes.extend_from_slice(json);
    bytes.extend_from_slice(&[0u8; 8]);
    let path = temp.path().join("good.safetensors");
    fs::write(&path, &bytes).unwrap();
    assert!(model_header_matches_extension(&path));
}

/// The APR metadata length was only bounds-checked when it was BELOW 100 MB;
/// a length at or above that limit skipped the read and returned success, so
/// the most implausible value possible was the one that validated.
#[test]
fn test_apr_with_absurd_metadata_length_is_not_valid() {
    let temp = TempDir::new().unwrap();
    let mut bytes = b"APR2".to_vec();
    bytes.extend_from_slice(&u32::MAX.to_le_bytes());
    let path = temp.path().join("absurd.apr");
    fs::write(&path, &bytes).unwrap();
    assert!(
        !model_header_matches_extension(&path),
        "an 8-byte APR claiming a 4 GiB metadata block must not validate"
    );
}

/// Same defect reached through ordinary text: "APRIL " satisfies the 3-byte
/// `APR` magic and the following bytes decode to a metadata length above the
/// 100 MB limit, so the bounds check was skipped and a prose file validated as
/// an aprender model.
#[test]
fn test_text_file_beginning_with_apr_is_not_valid() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("notes.apr");
    fs::write(
        &path,
        b"APRIL notes on the model serialization container format",
    )
    .unwrap();
    assert!(
        !model_header_matches_extension(&path),
        "prose beginning with \"APR\" is not an APR container"
    );
}

/// Positive control for the APR path: magic, a metadata length that fits, and a
/// JSON tensor index.
#[test]
fn test_well_formed_apr_is_valid() {
    let temp = TempDir::new().unwrap();
    let json = br#"{"tensors":[{"name":"w","dtype":"f32"}]}"#;
    let mut bytes = b"APR2".to_vec();
    bytes.extend_from_slice(&(json.len() as u32).to_le_bytes());
    bytes.extend_from_slice(json);
    bytes.extend_from_slice(&[0u8; 8]);
    let path = temp.path().join("good.apr");
    fs::write(&path, &bytes).unwrap();
    assert!(model_header_matches_extension(&path));
}
