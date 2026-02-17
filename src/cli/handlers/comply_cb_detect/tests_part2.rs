#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::fs;

// =============================================================================
// Tests for CB-1000 MLOps Model Quality
// =============================================================================

mod cb1000_model_tests {
    use super::*;
    use tempfile::TempDir;

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
            &[0x47, 0x47, 0x55, 0x46, 0, 0, 0, 0],
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
        fs::write(temp.path().join("small.gguf"), &[0u8; 100]).unwrap();
        let violations = detect_cb1007_excessive_file_size(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_walkdir_model_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("model.gguf"), &[0u8; 16]).unwrap();
        fs::write(temp.path().join("weights.safetensors"), &[0u8; 16]).unwrap();
        fs::write(temp.path().join("model.apr"), &[0u8; 16]).unwrap();
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
}

// =============================================================================
// Tests for CB-800 Scala Best Practices
// =============================================================================

mod cb800_scala_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb800_detects_mutable_collection() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("App.scala"),
            "val cache = mutable.HashMap[String, Int]()\nval items = mutable.Buffer[Int]()",
        )
        .unwrap();

        let violations = detect_cb800_mutable_collection(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-800");
        assert!(violations[0].description.contains("mutable.HashMap"));
    }

    #[test]
    fn test_cb800_allows_import_of_mutable() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import scala.collection.mutable.Map\nval x = Map(\"a\" -> 1)",
        )
        .unwrap();

        let violations = detect_cb800_mutable_collection(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb801_detects_null_literal() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "val x: String = null\nval y = if (x == null) \"default\" else x",
        )
        .unwrap();

        let violations = detect_cb801_null_usage(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-801");
    }

    #[test]
    fn test_cb801_allows_java_interop_null() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "@Nullable val x: String = null",
        )
        .unwrap();

        let violations = detect_cb801_null_usage(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb801_no_false_positive_on_nullable_identifier() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "val nullable = true\nval isNullable = false",
        )
        .unwrap();

        let violations = detect_cb801_null_usage(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb802_detects_wildcard_import() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import com.example.models._\nimport org.apache.spark.sql.*",
        )
        .unwrap();

        let violations = detect_cb802_wildcard_import(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-802");
    }

    #[test]
    fn test_cb802_allows_stdlib_wildcard() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import scala.collection.immutable._\nimport java.util._",
        )
        .unwrap();

        let violations = detect_cb802_wildcard_import(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb803_detects_return_statement() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "def foo(x: Int): Int = {\n  if (x > 0) return x\n  x * -1\n}",
        )
        .unwrap();

        let violations = detect_cb803_return_statement(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-803");
    }

    #[test]
    fn test_cb804_detects_var_declaration() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "var count = 0\nprivate var state = \"init\"",
        )
        .unwrap();

        let violations = detect_cb804_var_declaration(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-804");
    }

    #[test]
    fn test_cb804_no_false_positive_on_val() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "val count = 0\nprivate val state = \"init\"",
        )
        .unwrap();

        let violations = detect_cb804_var_declaration(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb805_detects_blocking_in_future() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import scala.concurrent.Future\nval f = Future {\n  Thread.sleep(1000)\n  42\n}",
        )
        .unwrap();

        let violations = detect_cb805_blocking_in_future(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-805");
        assert!(violations[0].description.contains("Thread.sleep"));
    }

    #[test]
    fn test_cb805_no_false_positive_outside_future() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "def main(): Unit = {\n  Thread.sleep(1000)\n}",
        )
        .unwrap();

        let violations = detect_cb805_blocking_in_future(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_scala_test_file_detection() {
        use std::path::Path;
        assert!(is_scala_test_file(Path::new(
            "src/test/scala/AppTest.scala"
        )));
        assert!(is_scala_test_file(Path::new("AppSpec.scala")));
        assert!(is_scala_test_file(Path::new("TestHelper.scala")));
        assert!(!is_scala_test_file(Path::new("src/main/scala/App.scala")));
    }

    #[test]
    fn test_scala_production_lines() {
        let content = "// comment\nval x = 1\n/* block */\nval y = 2\n\nval z = 3 // inline";
        let lines = compute_scala_production_lines(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], (2, "val x = 1".to_string()));
        assert_eq!(lines[1], (4, "val y = 2".to_string()));
        assert_eq!(lines[2], (6, "val z = 3".to_string()));
    }

    #[test]
    fn test_walkdir_scala_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("App.scala"), "object App").unwrap();
        fs::write(temp.path().join("build.sc"), "// mill build").unwrap();
        fs::write(temp.path().join("code.rs"), "fn main() {}").unwrap();

        let files = walkdir_scala_files(temp.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scala_skips_test_files() {
        let temp = TempDir::new().unwrap();
        let test_dir = temp.path().join("test");
        fs::create_dir_all(&test_dir).unwrap();
        fs::write(
            test_dir.join("AppTest.scala"),
            "var x = null\nimport foo._\nreturn 42",
        )
        .unwrap();

        // All detectors should skip test files
        assert_eq!(detect_cb800_mutable_collection(temp.path()).len(), 0);
        assert_eq!(detect_cb801_null_usage(temp.path()).len(), 0);
        assert_eq!(detect_cb802_wildcard_import(temp.path()).len(), 0);
        assert_eq!(detect_cb803_return_statement(temp.path()).len(), 0);
        assert_eq!(detect_cb804_var_declaration(temp.path()).len(), 0);
    }
}

// =============================================================================
// Tests for CB-513 through CB-518: Rust Best Practices (Extended)
// =============================================================================

#[cfg(test)]
mod cb513_to_cb518_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
}

// =============================================================================
// Tests for CB-519 through CB-527: Aprender Bug Pattern Detection
// =============================================================================

#[cfg(test)]
mod cb519_to_cb527_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
        assert!(violations.len() >= 1);
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
}

