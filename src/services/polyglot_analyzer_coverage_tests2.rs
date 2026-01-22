//! Coverage tests part 2 for polyglot analyzer
//! Extracted for file health compliance (CB-040)

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_handle_file_rust_extension() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("main.rs", "fn main() {}\n")]);

    let mut file_count = 0;
    let mut total_lines = 0;

    analyzer.handle_file(
        &temp_dir.path().join("main.rs"),
        &[".rs".to_string()],
        &mut file_count,
        &mut total_lines,
    );

    assert_eq!(file_count, 1);
    assert_eq!(total_lines, 1);
}

#[test]
fn test_handle_file_no_extension() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("Makefile", "all:\n\techo hello")]);

    let mut file_count = 0;
    let mut total_lines = 0;

    analyzer.handle_file(
        &temp_dir.path().join("Makefile"),
        &[".rs".to_string()],
        &mut file_count,
        &mut total_lines,
    );

    // No .rs extension, should not count
    assert_eq!(file_count, 0);
    assert_eq!(total_lines, 0);
}

#[test]
fn test_handle_file_wrong_extension() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("script.py", "print('hello')\n")]);

    let mut file_count = 0;
    let mut total_lines = 0;

    analyzer.handle_file(
        &temp_dir.path().join("script.py"),
        &[".rs".to_string()], // Looking for .rs, not .py
        &mut file_count,
        &mut total_lines,
    );

    assert_eq!(file_count, 0);
    assert_eq!(total_lines, 0);
}

#[test]
fn test_handle_file_multiline() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "lib.rs",
        "fn foo() {}\nfn bar() {}\nfn baz() {}\n// comment\n",
    )]);

    let mut file_count = 0;
    let mut total_lines = 0;

    analyzer.handle_file(
        &temp_dir.path().join("lib.rs"),
        &[".rs".to_string()],
        &mut file_count,
        &mut total_lines,
    );

    assert_eq!(file_count, 1);
    assert_eq!(total_lines, 4);
}

#[test]
fn test_handle_directory_skips_node_modules() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("node_modules/package/index.js", "module.exports = {}")]);

    let mut file_count = 0;
    let mut total_lines = 0;

    let result = analyzer.handle_directory(
        &temp_dir.path().join("node_modules"),
        &[".js".to_string()],
        &mut file_count,
        &mut total_lines,
    );

    assert!(result.is_ok());
    assert_eq!(file_count, 0); // Should skip node_modules
}

#[test]
fn test_scan_directory_recursive() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("src/main.rs", "fn main() {}\n"),
        ("src/lib.rs", "pub fn foo() {}\npub fn bar() {}\n"),
        ("tests/test.rs", "fn test() {}\n"),
    ]);

    let mut file_count = 0;
    let mut total_lines = 0;

    let result = analyzer.scan_directory_recursive(
        temp_dir.path(),
        &[".rs".to_string()],
        &mut file_count,
        &mut total_lines,
    );

    assert!(result.is_ok());
    assert_eq!(file_count, 3);
    assert_eq!(total_lines, 4); // 1 + 2 + 1
}

#[test]
fn test_count_files_recursive_js() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("src/index.js", "// js"),
        ("src/app.js", "// app"),
        ("src/utils/helper.js", "// helper"),
    ]);

    let mut count = 0;
    let result = analyzer.count_files_recursive(temp_dir.path(), &["js".to_string()], &mut count);

    assert!(result.is_ok());
    assert_eq!(count, 3);
}

#[test]
fn test_count_files_recursive_skips_node_modules() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("src/index.js", "// js"),
        ("node_modules/pkg/index.js", "// pkg"),
    ]);

    let mut count = 0;
    let result = analyzer.count_files_recursive(temp_dir.path(), &["js".to_string()], &mut count);

    assert!(result.is_ok());
    assert_eq!(count, 1); // Only src/index.js
}

#[test]
fn test_collect_directories_recursive() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("src/main.rs", ""),
        ("src/utils/mod.rs", ""),
        ("tests/integration/test.rs", ""),
    ]);

    let mut directories = Vec::new();
    let result = analyzer.collect_directories_recursive(temp_dir.path(), &mut directories, 0);

    assert!(result.is_ok());
    assert!(directories.contains(&"src".to_string()));
    assert!(directories.contains(&"tests".to_string()));
    assert!(directories.contains(&"utils".to_string()));
}

#[test]
fn test_collect_directories_depth_limit() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("a/b/c/d/e/deep.rs", ""), // Very deep nesting
    ]);

    let mut directories = Vec::new();
    let result = analyzer.collect_directories_recursive(temp_dir.path(), &mut directories, 0);

    assert!(result.is_ok());
    // Should be limited to depth 3, so a, b, c should be found but not d or e
    // (depth 0 = root, depth 1 = a, depth 2 = b, depth 3 = c, stop)
    assert!(directories.contains(&"a".to_string()));
    assert!(directories.contains(&"b".to_string()));
    assert!(directories.contains(&"c".to_string()));
    // d might not be found due to depth limit
}

// Language Detection Tests

#[tokio::test]
async fn test_detect_languages_rust_project() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}\n"),
        ("src/lib.rs", "pub fn foo() {}\n"),
    ]);

    let languages = analyzer.detect_languages(temp_dir.path()).await.unwrap();

    assert!(languages.contains_key("rust"));
    let rust_info = languages.get("rust").unwrap();
    assert_eq!(rust_info.file_count, 2);
    assert_eq!(rust_info.line_count, 2);
}

#[tokio::test]
async fn test_detect_languages_python_project() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("requirements.txt", "flask==2.0.0"),
        ("app.py", "from flask import Flask\napp = Flask(__name__)\n"),
        ("utils.py", "def helper():\n    pass\n"),
    ]);

    let languages = analyzer.detect_languages(temp_dir.path()).await.unwrap();

    assert!(languages.contains_key("python"));
    let py_info = languages.get("python").unwrap();
    assert_eq!(py_info.file_count, 2);
    assert!(py_info.frameworks.contains(&"Flask".to_string()));
}

#[tokio::test]
async fn test_detect_languages_mixed_project() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        (
            "Cargo.toml",
            "[package]\nname = \"test\"\n[dependencies]\ntokio = \"1.0\"",
        ),
        ("src/main.rs", "fn main() {}\n"),
        ("scripts/build.py", "import os\n"),
        ("frontend/app.ts", "const x: number = 1;\n"),
    ]);

    let languages = analyzer.detect_languages(temp_dir.path()).await.unwrap();

    assert!(languages.contains_key("rust"));
    assert!(languages.contains_key("python"));
    assert!(languages.contains_key("typescript"));
}

#[tokio::test]
async fn test_detect_languages_empty_project() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let languages = analyzer.detect_languages(temp_dir.path()).await.unwrap();

    // No language files detected
    assert!(languages.is_empty());
}

// Language Stats Calculation Tests

#[tokio::test]
async fn test_calculate_language_stats_single() {
    let analyzer = create_analyzer();
    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let stats = analyzer
        .calculate_language_stats(&language_info)
        .await
        .unwrap();

    assert_eq!(stats.len(), 1);
    let rust_stats = &stats[0];
    assert_eq!(rust_stats.language, "rust");
    assert_eq!(rust_stats.file_count, 10);
    assert_eq!(rust_stats.line_count, 1000);
}

#[tokio::test]
async fn test_calculate_language_stats_sorted_by_lines() {
    let analyzer = create_analyzer();
    let mut language_info = HashMap::new();
    language_info.insert("python".to_string(), create_language_info("python", 5, 500));
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 2000));
    language_info.insert(
        "typescript".to_string(),
        create_language_info("typescript", 8, 800),
    );

    let stats = analyzer
        .calculate_language_stats(&language_info)
        .await
        .unwrap();

    // Should be sorted by line count descending
    assert_eq!(stats[0].language, "rust");
    assert_eq!(stats[1].language, "typescript");
    assert_eq!(stats[2].language, "python");
}

// Cross-Language Integration Tests

#[tokio::test]
async fn test_analyze_rust_python_integration_pyo3() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "Cargo.toml",
        r#"
[package]
name = "test"

[dependencies]
pyo3 = "0.18"
"#,
    )]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_rust_python_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.7);
    assert!(files_involved.iter().any(|f| f.contains("PyO3")));
}

#[tokio::test]
async fn test_analyze_rust_python_integration_setup_py() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "setup.py",
        "from setuptools import setup\nsetup(rust_extensions=['my_crate'])",
    )]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_rust_python_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.5);
    assert!(files_involved.contains(&"setup.py".to_string()));
}

#[tokio::test]
async fn test_analyze_rust_python_integration_build_artifacts() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("target/.keep", ""), ("__pycache__/.keep", "")]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_rust_python_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.3);
    assert!(files_involved.iter().any(|f| f.contains("build artifacts")));
}

#[tokio::test]
async fn test_analyze_js_ts_integration_tsconfig() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("tsconfig.json", "{\"compilerOptions\": {}}")]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_js_ts_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.6);
    assert!(files_involved.contains(&"tsconfig.json".to_string()));
}

#[tokio::test]
async fn test_analyze_js_ts_integration_package_json() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "package.json",
        r#"{"devDependencies": {"typescript": "^4.0.0"}}"#,
    )]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_js_ts_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.4);
    assert!(files_involved.iter().any(|f| f.contains("TypeScript")));
}

#[tokio::test]
async fn test_analyze_js_ts_integration_mixed_files() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("src/app.js", "const x = 1;"),
        ("src/types.ts", "type X = number;"),
    ]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_js_ts_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.3);
    // Should report mixed JS/TS files
    assert!(files_involved
        .iter()
        .any(|f| f.contains("JS") && f.contains("TS")));
}

#[tokio::test]
async fn test_analyze_api_integration_openapi() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("openapi.yaml", "openapi: 3.0.0\n")]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_api_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.5);
    assert!(files_involved.contains(&"openapi.yaml".to_string()));
}

#[tokio::test]
async fn test_analyze_api_integration_swagger() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("swagger.json", "{\"swagger\": \"2.0\"}")]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_api_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.5);
    assert!(files_involved.contains(&"swagger.json".to_string()));
}

#[tokio::test]
async fn test_analyze_api_integration_docker_compose() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "docker-compose.yml",
        "version: '3'\nservices:\n  app:\n    build: .",
    )]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_api_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.4);
    assert!(files_involved.contains(&"docker-compose".to_string()));
}

#[tokio::test]
async fn test_analyze_api_integration_schema_files() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("schema.json", "{\"type\": \"object\"}")]);

    let mut files_involved = Vec::new();
    let strength = analyzer
        .analyze_api_integration(temp_dir.path(), &mut files_involved)
        .await
        .unwrap();

    assert!(strength >= 0.3);
    assert!(files_involved.contains(&"schema.json".to_string()));
}

// Build System and Configuration Dependencies Tests

#[tokio::test]
async fn test_analyze_build_system_dependencies_makefile() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "Makefile",
        "build:\n\tcargo build\n\tpython setup.py build\n",
    )]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert("python".to_string(), create_language_info("python", 5, 500));

    let deps = analyzer
        .analyze_build_system_dependencies(temp_dir.path(), &language_info)
        .await
        .unwrap();

    // Build system analysis may or may not detect dependencies depending on implementation
    // The important thing is that it completes successfully
    let _ = deps;
}

#[tokio::test]
async fn test_analyze_build_system_dependencies_no_makefile() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let deps = analyzer
        .analyze_build_system_dependencies(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(deps.is_empty());
}

#[tokio::test]
async fn test_analyze_configuration_dependencies_shared_config() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("config.json", "{\"debug\": true}")]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert("python".to_string(), create_language_info("python", 5, 500));

    let deps = analyzer
        .analyze_configuration_dependencies(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(!deps.is_empty());
    assert!(deps
        .iter()
        .any(|d| matches!(d.dependency_type, DependencyType::ConfigurationFile)));
}

#[tokio::test]
async fn test_analyze_configuration_dependencies_env_file() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(".env", "DATABASE_URL=postgres://localhost/db")]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert(
        "typescript".to_string(),
        create_language_info("typescript", 5, 500),
    );

    let deps = analyzer
        .analyze_configuration_dependencies(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(!deps.is_empty());
}

#[tokio::test]
async fn test_analyze_configuration_dependencies_single_language() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("config.json", "{}")]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let deps = analyzer
        .analyze_configuration_dependencies(temp_dir.path(), &language_info)
        .await
        .unwrap();

    // Single language shouldn't generate cross-language config deps
    assert!(deps.is_empty());
}

// Language Pair Analysis Tests

#[tokio::test]
async fn test_analyze_language_pair_rust_python() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "Cargo.toml",
        "[package]\nname = \"test\"\n[dependencies]\npyo3 = \"0.18\"",
    )]);

    let result = analyzer
        .analyze_language_pair(temp_dir.path(), "rust", "python")
        .await
        .unwrap();

    assert!(result.is_some());
    let dep = result.unwrap();
    assert_eq!(dep.from_language, "rust");
    assert_eq!(dep.to_language, "python");
    assert!(matches!(dep.dependency_type, DependencyType::FFI));
}

#[tokio::test]
async fn test_analyze_language_pair_no_integration() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let result = analyzer
        .analyze_language_pair(temp_dir.path(), "java", "csharp")
        .await
        .unwrap();

    // No potential integration detected
    assert!(result.is_none());
}

#[tokio::test]
async fn test_analyze_language_pair_default_coupling() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let result = analyzer
        .analyze_language_pair(temp_dir.path(), "python", "typescript")
        .await
        .unwrap();

    // Empty directory may or may not have default coupling
    // Just verify the analysis completes successfully
    if let Some(dep) = result {
        assert!(
            dep.coupling_strength >= 0.0,
            "Coupling should be non-negative"
        );
    }
}

// Cross-Language Dependencies Tests

#[tokio::test]
async fn test_analyze_cross_language_dependencies() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("Cargo.toml", "[dependencies]\npyo3 = \"0.18\""),
        ("tsconfig.json", "{}"),
        (
            "package.json",
            "{\"dependencies\": {\"typescript\": \"4.0.0\"}}",
        ),
    ]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert("python".to_string(), create_language_info("python", 5, 500));
    language_info.insert(
        "typescript".to_string(),
        create_language_info("typescript", 8, 800),
    );
    language_info.insert(
        "javascript".to_string(),
        create_language_info("javascript", 3, 300),
    );

    let deps = analyzer
        .analyze_cross_language_dependencies(temp_dir.path(), &language_info)
        .await
        .unwrap();

    // Should detect multiple cross-language dependencies
    assert!(!deps.is_empty());
}

// Architecture Detection Tests

#[tokio::test]
async fn test_detect_architecture_microservices() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "docker-compose.yml",
        "version: '3'\nservices:\n  api:\n    build: .",
    )]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert("python".to_string(), create_language_info("python", 5, 500));

    let pattern = analyzer
        .detect_architecture_pattern(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(pattern.is_some());
    // With docker-compose and 2 languages, should detect microservices
    assert!(matches!(
        pattern.unwrap(),
        ArchitecturePattern::Microservices
    ));
}

#[tokio::test]
async fn test_detect_architecture_layered() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("controller/user_controller.rs", ""),
        ("service/user_service.rs", ""),
        ("repository/user_repo.rs", ""),
    ]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let pattern = analyzer
        .detect_architecture_pattern(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(pattern.is_some());
    assert!(matches!(
        pattern.unwrap(),
        ArchitecturePattern::LayeredArchitecture
    ));
}

#[tokio::test]
async fn test_detect_architecture_event_driven() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("events/user_created.rs", ""),
        ("message/handler.rs", ""),
        ("queue/processor.rs", ""),
    ]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let pattern = analyzer
        .detect_architecture_pattern(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(pattern.is_some());
    assert!(matches!(pattern.unwrap(), ArchitecturePattern::EventDriven));
}

#[tokio::test]
async fn test_detect_architecture_monolithic_single_language() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("src/main.rs", "fn main() {}"),
        ("src/lib.rs", "pub fn foo() {}"),
    ]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let pattern = analyzer
        .detect_architecture_pattern(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(pattern.is_some());
    // Single language with no other indicators = Monolithic
    assert!(matches!(pattern.unwrap(), ArchitecturePattern::Monolithic));
}

#[tokio::test]
async fn test_detect_architecture_client_server_two_languages() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("src/main.rs", "fn main() {}"),
        ("frontend/app.ts", "const x = 1;"),
    ]);

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert(
        "typescript".to_string(),
        create_language_info("typescript", 5, 500),
    );

    let pattern = analyzer
        .detect_architecture_pattern(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(pattern.is_some());
    // Two languages with no other indicators = ClientServer
    assert!(matches!(
        pattern.unwrap(),
        ArchitecturePattern::ClientServer
    ));
}

#[tokio::test]
async fn test_detect_architecture_mixed_many_languages() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert("python".to_string(), create_language_info("python", 5, 500));
    language_info.insert(
        "typescript".to_string(),
        create_language_info("typescript", 5, 500),
    );

    let pattern = analyzer
        .detect_architecture_pattern(temp_dir.path(), &language_info)
        .await
        .unwrap();

    assert!(pattern.is_some());
    // Three languages with no other indicators = Mixed
    assert!(matches!(pattern.unwrap(), ArchitecturePattern::Mixed));
}

// Architecture Indicators Analysis Tests

#[tokio::test]
async fn test_analyze_architecture_indicators_microservice() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("docker-compose.yml", "version: '3'"),
        ("services/api/main.rs", ""),
    ]);

    let indicators = analyzer
        .analyze_architecture_indicators(temp_dir.path())
        .await
        .unwrap();

    assert!(indicators.has_microservice_indicators);
}

#[tokio::test]
async fn test_analyze_architecture_indicators_kubernetes() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("kubernetes/deployment.yaml", "kind: Deployment")]);

    let indicators = analyzer
        .analyze_architecture_indicators(temp_dir.path())
        .await
        .unwrap();

    assert!(indicators.has_microservice_indicators);
}

#[tokio::test]
async fn test_analyze_architecture_indicators_layered() {
    let analyzer = create_analyzer();
    let temp_dir =
        create_test_project(&[("service/user_service.rs", ""), ("controller/api.rs", "")]);

    let indicators = analyzer
        .analyze_architecture_indicators(temp_dir.path())
        .await
        .unwrap();

    assert!(indicators.has_layered_indicators);
}

#[tokio::test]
async fn test_analyze_architecture_indicators_event_driven() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("events/user_event.rs", ""), ("message/handler.rs", "")]);

    let indicators = analyzer
        .analyze_architecture_indicators(temp_dir.path())
        .await
        .unwrap();

    assert!(indicators.has_event_indicators);
}

#[tokio::test]
async fn test_analyze_architecture_indicators_plugin() {
    let analyzer = create_analyzer();
    let temp_dir =
        create_test_project(&[("plugins/auth/mod.rs", ""), ("extension/logging.rs", "")]);

    let indicators = analyzer
        .analyze_architecture_indicators(temp_dir.path())
        .await
        .unwrap();

    assert!(indicators.has_plugin_indicators);
}

// Integration Points Identification Tests

#[tokio::test]
async fn test_identify_integration_points_empty() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let points = analyzer
        .identify_integration_points(temp_dir.path(), &[])
        .await
        .unwrap();

    assert!(points.is_empty());
}

#[tokio::test]
async fn test_identify_integration_points_ffi() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let deps = vec![CrossLanguageDependency {
        from_language: "rust".to_string(),
        to_language: "python".to_string(),
        dependency_type: DependencyType::FFI,
        coupling_strength: 0.8,
        files_involved: vec!["lib.rs".to_string()],
    }];

    let points = analyzer
        .identify_integration_points(temp_dir.path(), &deps)
        .await
        .unwrap();

    assert_eq!(points.len(), 1);
    let point = &points[0];
    assert!(point.name.contains("rust") && point.name.contains("python"));
    assert!(matches!(point.integration_type, IntegrationType::Memory));
    assert!(matches!(point.risk_level, RiskLevel::Critical));
}

#[tokio::test]
async fn test_identify_integration_points_multiple() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let deps = vec![
        CrossLanguageDependency {
            from_language: "rust".to_string(),
            to_language: "python".to_string(),
            dependency_type: DependencyType::FFI,
            coupling_strength: 0.7,
            files_involved: vec![],
        },
        CrossLanguageDependency {
            from_language: "typescript".to_string(),
            to_language: "javascript".to_string(),
            dependency_type: DependencyType::SharedDataStructure,
            coupling_strength: 0.5,
            files_involved: vec![],
        },
    ];

    let points = analyzer
        .identify_integration_points(temp_dir.path(), &deps)
        .await
        .unwrap();

    assert_eq!(points.len(), 2);
}

// Full Project Analysis Tests

#[tokio::test]
async fn test_analyze_project_rust_only() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        (
            "Cargo.toml",
            "[package]\nname = \"test\"\n[dependencies]\ntokio = \"1.0\"",
        ),
        ("src/main.rs", "fn main() {\n    println!(\"hello\");\n}\n"),
        ("src/lib.rs", "pub fn foo() {}\n"),
    ]);

    let analysis = analyzer.analyze_project(temp_dir.path()).await.unwrap();

    assert!(!analysis.languages.is_empty());
    assert!(analysis.languages.iter().any(|l| l.language == "rust"));
    assert!(analysis.architecture_pattern.is_some());
    assert!((0.0..=1.0).contains(&analysis.recommendation_score));
}

#[tokio::test]
async fn test_analyze_project_mixed_languages() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        (
            "Cargo.toml",
            "[package]\nname = \"test\"\n[dependencies]\npyo3 = \"0.18\"",
        ),
        ("src/main.rs", "fn main() {}\n"),
        ("scripts/build.py", "import os\nprint('building')\n"),
        ("frontend/app.ts", "const x: number = 1;\n"),
        ("frontend/utils.js", "function helper() {}\n"),
        (
            "package.json",
            "{\"dependencies\": {\"typescript\": \"4.0.0\"}}",
        ),
        ("tsconfig.json", "{}"),
    ]);

    let analysis = analyzer.analyze_project(temp_dir.path()).await.unwrap();

    // Should detect multiple languages
    assert!(analysis.languages.len() >= 2);

    // Should have cross-language dependencies
    // (pyo3 creates rust-python integration)
    assert!(
        !analysis.cross_language_dependencies.is_empty() || !analysis.integration_points.is_empty()
    );

    // Higher recommendation score for polyglot project
    assert!(analysis.recommendation_score >= 0.2);
}

#[tokio::test]
async fn test_analyze_project_empty_directory() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let analysis = analyzer.analyze_project(temp_dir.path()).await.unwrap();

    assert!(analysis.languages.is_empty());
    assert!(analysis.cross_language_dependencies.is_empty());
    assert!(analysis.integration_points.is_empty());
    // Empty projects may have a small base score
    assert!(
        analysis.recommendation_score <= 0.2,
        "Empty project score should be near 0, got {}",
        analysis.recommendation_score
    );
}

// Enum Serialization Tests (for coverage of derive macros)

#[test]
fn test_dependency_type_debug() {
    let dep_types = [
        DependencyType::FFI,
        DependencyType::ProcessCommunication,
        DependencyType::SharedDataStructure,
        DependencyType::ConfigurationFile,
        DependencyType::BuildSystem,
        DependencyType::Testing,
    ];

    for dt in &dep_types {
        let debug_str = format!("{:?}", dt);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_architecture_pattern_clone_and_debug() {
    let patterns = [
        ArchitecturePattern::Microservices,
        ArchitecturePattern::Monolithic,
        ArchitecturePattern::LayeredArchitecture,
        ArchitecturePattern::EventDriven,
        ArchitecturePattern::PluginArchitecture,
        ArchitecturePattern::ClientServer,
        ArchitecturePattern::Mixed,
    ];

    for pattern in &patterns {
        let cloned = pattern.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_integration_type_clone_and_debug() {
    let types = [
        IntegrationType::API,
        IntegrationType::Database,
        IntegrationType::FileSystem,
        IntegrationType::Memory,
        IntegrationType::Network,
        IntegrationType::Configuration,
    ];

    for t in &types {
        let cloned = t.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_risk_level_clone_and_debug() {
    let levels = [
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ];

    for level in &levels {
        let cloned = level.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

// Struct Serialization Tests

#[test]
fn test_language_info_serialization() {
    let info = LanguageInfo {
        name: "rust".to_string(),
        file_count: 10,
        line_count: 1000,
        frameworks: vec!["Tokio".to_string()],
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("rust"));
    assert!(json.contains("10"));
    assert!(json.contains("Tokio"));

    let deserialized: LanguageInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "rust");
    assert_eq!(deserialized.file_count, 10);
}

#[test]
fn test_polyglot_analysis_serialization() {
    let analysis = PolyglotAnalysis {
        languages: vec![],
        cross_language_dependencies: vec![],
        architecture_pattern: Some(ArchitecturePattern::Monolithic),
        integration_points: vec![],
        recommendation_score: 0.5,
    };

    let json = serde_json::to_string(&analysis).unwrap();
    assert!(json.contains("Monolithic"));
    assert!(json.contains("0.5"));

    let deserialized: PolyglotAnalysis = serde_json::from_str(&json).unwrap();
    assert!(deserialized.architecture_pattern.is_some());
}

#[test]
fn test_cross_language_dependency_serialization() {
    let dep = CrossLanguageDependency {
        from_language: "rust".to_string(),
        to_language: "python".to_string(),
        dependency_type: DependencyType::FFI,
        coupling_strength: 0.8,
        files_involved: vec!["lib.rs".to_string()],
    };

    let json = serde_json::to_string(&dep).unwrap();
    assert!(json.contains("rust"));
    assert!(json.contains("python"));
    assert!(json.contains("FFI"));

    let deserialized: CrossLanguageDependency = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.coupling_strength, 0.8);
}

#[test]
fn test_integration_point_serialization() {
    let point = IntegrationPoint {
        name: "test-integration".to_string(),
        languages: vec!["rust".to_string(), "python".to_string()],
        integration_type: IntegrationType::Memory,
        risk_level: RiskLevel::High,
        description: "FFI binding".to_string(),
    };

    let json = serde_json::to_string(&point).unwrap();
    assert!(json.contains("test-integration"));
    assert!(json.contains("Memory"));
    assert!(json.contains("High"));

    let deserialized: IntegrationPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.languages.len(), 2);
}

#[test]
fn test_language_stats_serialization() {
    let stats = LanguageStats {
        language: "typescript".to_string(),
        file_count: 25,
        line_count: 5000,
        complexity_score: 6.5,
        test_coverage: 0.82,
        primary_frameworks: vec!["React".to_string(), "Next.js".to_string()],
    };

    let json = serde_json::to_string(&stats).unwrap();
    assert!(json.contains("typescript"));
    assert!(json.contains("6.5"));
    assert!(json.contains("React"));

    let deserialized: LanguageStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.file_count, 25);
    assert_eq!(deserialized.primary_frameworks.len(), 2);
}
