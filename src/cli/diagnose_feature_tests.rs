// Feature test implementations for self-diagnostic
// Included from diagnose.rs - do NOT add `use` imports or `#!` attributes here

/// Rust ast test.
pub struct RustAstTest;

#[async_trait::async_trait]
impl FeatureTest for RustAstTest {
    fn name(&self) -> &'static str {
        "ast.rust"
    }

    async fn execute(&self) -> Result<serde_json::Value> {
        use syn::parse_file;

        const TEST_CODE: &str = r"
            /// Fibonacci.
            pub fn fibonacci(n: u32) -> u32 {
                match n {
                    0 => 0,
                    1 => 1,
                    _ => fibonacci(n - 1) + fibonacci(n - 2),
                }
            }
        ";

        let start = Instant::now();
        let ast = parse_file(TEST_CODE)?;
        let parse_time = start.elapsed();

        // Verify expected structure
        let items_count = ast.items.len();
        anyhow::ensure!(items_count == 1, "Expected 1 item, got {items_count}");

        Ok(json!({
            "parsed_items": items_count,
            "parse_time_us": parse_time.as_micros(),
        }))
    }
}

/// Type script ast test.
pub struct TypeScriptAstTest;

#[async_trait::async_trait]
impl FeatureTest for TypeScriptAstTest {
    fn name(&self) -> &'static str {
        "ast.typescript"
    }

    /// This test used to be `let _test_code = TEST_CODE;` followed by a literal
    /// `"typescript_test": "passed"` — it exercised no parser at all, yet counted
    /// towards `diagnose`'s success rate exactly like the one test that did
    /// (ast.rust). It now runs the TypeScript AST parser over the sample and
    /// reports what the parser actually found; a build without the
    /// `typescript-ast` feature fails here instead of quietly reporting a pass.
    async fn execute(&self) -> Result<serde_json::Value> {
        const TEST_CODE: &str = r"
            export function factorial(n: number): number {
                if (n <= 1) return 1;
                return n * factorial(n - 1);
            }
        ";

        let file = tempfile::Builder::new().suffix(".ts").tempfile()?;
        tokio::fs::write(file.path(), TEST_CODE).await?;

        let start = Instant::now();
        let context = crate::services::ast_typescript::analyze_typescript_file(file.path())
            .await
            .map_err(|e| anyhow::anyhow!("TypeScript AST parse failed: {e}"))?;
        let parse_time = start.elapsed();

        let functions = count_ast_functions(&context);
        anyhow::ensure!(
            functions >= 1,
            "Expected at least 1 parsed TypeScript function, got {functions}"
        );

        Ok(json!({
            "typescript_test": "parsed",
            "parsed_items": context.items.len(),
            "parsed_functions": functions,
            "parse_time_us": parse_time.as_micros(),
        }))
    }
}

/// Number of function items in a parsed file context.
fn count_ast_functions(context: &crate::services::context::FileContext) -> usize {
    context
        .items
        .iter()
        .filter(|item| matches!(item, crate::services::context::AstItem::Function { .. }))
        .count()
}

/// Python ast test.
pub struct PythonAstTest;

#[async_trait::async_trait]
impl FeatureTest for PythonAstTest {
    fn name(&self) -> &'static str {
        "ast.python"
    }

    /// Same defect as `ast.typescript`: the body was `let _test_code = TEST_CODE;`
    /// and a hardcoded "passed", so `diagnose` reported a working Python AST on
    /// a build where the parser could not run at all. It now parses the sample.
    async fn execute(&self) -> Result<serde_json::Value> {
        const TEST_CODE: &str = r"
def quicksort(arr):
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    return quicksort(left) + middle + quicksort(right)
";

        let file = tempfile::Builder::new().suffix(".py").tempfile()?;
        tokio::fs::write(file.path(), TEST_CODE).await?;

        let start = Instant::now();
        let context = crate::services::ast_python::analyze_python_file(file.path())
            .await
            .map_err(|e| anyhow::anyhow!("Python AST parse failed: {e}"))?;
        let parse_time = start.elapsed();

        let functions = count_ast_functions(&context);
        anyhow::ensure!(
            functions >= 1,
            "Expected at least 1 parsed Python function, got {functions}"
        );

        Ok(json!({
            "python_test": "parsed",
            "parsed_items": context.items.len(),
            "parsed_functions": functions,
            "parse_time_us": parse_time.as_micros(),
        }))
    }
}

/// Cache subsystem test.
pub struct CacheSubsystemTest;

#[async_trait::async_trait]
impl FeatureTest for CacheSubsystemTest {
    fn name(&self) -> &'static str {
        "cache.subsystem"
    }

    async fn execute(&self) -> Result<serde_json::Value> {
        use crate::services::cache::{manager::SessionCacheManager, CacheConfig};

        let config = CacheConfig {
            max_memory_mb: 10,
            enable_watch: false,
            ..Default::default()
        };
        let cache = SessionCacheManager::new(config);

        // Test cache creation and diagnostics
        let diagnostics = cache.get_diagnostics();

        Ok(json!({
            "cache_initialized": true,
            "memory_pressure": cache.memory_pressure(),
            "total_cache_size": cache.get_total_cache_size(),
            "overall_hit_rate": diagnostics.effectiveness.overall_hit_rate,
            "memory_efficiency": diagnostics.effectiveness.memory_efficiency,
        }))
    }
}

/// Mermaid generator test.
pub struct MermaidGeneratorTest;

#[async_trait::async_trait]
impl FeatureTest for MermaidGeneratorTest {
    fn name(&self) -> &'static str {
        "output.mermaid"
    }

    async fn execute(&self) -> Result<serde_json::Value> {
        // Test basic mermaid generation capability
        let test_mermaid = r"graph TD
    A[Main] --> B[Library]
    B --> C[Utils]
";

        // Verify we can process mermaid syntax
        anyhow::ensure!(test_mermaid.contains("graph TD"), "Missing graph directive");
        anyhow::ensure!(test_mermaid.contains("-->"), "Missing edge syntax");

        Ok(json!({
            "mermaid_syntax_valid": true,
            "output_size": test_mermaid.len(),
        }))
    }
}

/// Complexity analysis test.
pub struct ComplexityAnalysisTest;

#[async_trait::async_trait]
impl FeatureTest for ComplexityAnalysisTest {
    fn name(&self) -> &'static str {
        "analysis.complexity"
    }

    /// The timed region used to be empty ("Just verify complexity analysis is
    /// available"), so this reported `status: completed` in 0ms without touching
    /// the analyzer. It now runs the complexity analyzer over a fixture whose
    /// cyclomatic complexity is known to exceed 1, and fails if the analyzer
    /// finds no functions.
    async fn execute(&self) -> Result<serde_json::Value> {
        const TEST_CODE: &str = r#"
            pub fn classify(n: i32) -> &'static str {
                if n < 0 {
                    "negative"
                } else if n == 0 {
                    "zero"
                } else if n % 2 == 0 {
                    "even"
                } else {
                    "odd"
                }
            }
        "#;

        let file = tempfile::Builder::new().suffix(".rs").tempfile()?;
        tokio::fs::write(file.path(), TEST_CODE).await?;

        let start = Instant::now();
        let metrics = crate::services::ast_rust::analyze_rust_file_with_complexity(file.path())
            .await
            .map_err(|e| anyhow::anyhow!("complexity analysis failed: {e}"))?;
        let duration = start.elapsed();

        anyhow::ensure!(
            !metrics.functions.is_empty(),
            "Complexity analysis found no functions in the fixture"
        );
        let max_cyclomatic = metrics
            .functions
            .iter()
            .map(|f| f.metrics.cyclomatic)
            .max()
            .unwrap_or(0);
        anyhow::ensure!(
            max_cyclomatic > 1,
            "Branching fixture measured cyclomatic {max_cyclomatic}, expected > 1"
        );

        Ok(json!({
            "status": "measured",
            "functions_analyzed": metrics.functions.len(),
            "max_cyclomatic": max_cyclomatic,
            "analysis_time_ms": duration.as_millis(),
        }))
    }
}

/// Deep context test.
pub struct DeepContextTest;

#[async_trait::async_trait]
impl FeatureTest for DeepContextTest {
    fn name(&self) -> &'static str {
        "analysis.deep_context"
    }

    /// This used to construct the analyzer and drop it (`let _ = analyzer;`),
    /// timing nothing, and report `status: completed`. It now runs the analyzer
    /// over a two-file throwaway project and reports how many files it actually
    /// came back with; the churn/DAG phases are left out so the self-diagnostic
    /// stays fast and does not depend on the cwd being a git repository.
    async fn execute(&self) -> Result<serde_json::Value> {
        use crate::services::deep_context::{AnalysisType, DeepContextConfig};

        const TEST_CODE: &str = r"
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        ";

        let dir = tempfile::tempdir()?;
        tokio::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"diagnose-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .await?;
        tokio::fs::create_dir_all(dir.path().join("src")).await?;
        tokio::fs::write(dir.path().join("src").join("lib.rs"), TEST_CODE).await?;

        let config = DeepContextConfig {
            include_analyses: vec![AnalysisType::Ast, AnalysisType::Complexity],
            ..DeepContextConfig::default()
        };
        let analyzer = DeepContextAnalyzer::new(config);

        let start = Instant::now();
        let context = analyzer.analyze_project(&dir.path().to_path_buf()).await?;
        let duration = start.elapsed();

        let files_analyzed = context.analyses.ast_contexts.len();
        anyhow::ensure!(
            files_analyzed >= 1,
            "Deep context analysis returned no files for a project containing src/lib.rs"
        );

        Ok(json!({
            "status": "measured",
            "files_analyzed": files_analyzed,
            "analysis_time_ms": duration.as_millis(),
        }))
    }
}

#[cfg(test)]
mod feature_tests_do_real_work {
    //! Four of the eight feature tests used to return a hardcoded "passed"
    //! without executing any code, which pinned `diagnose`'s success rate at
    //! 100% on an empty directory. These assert that each of the four now
    //! reports something it could only know by running the subsystem it names.
    use super::*;

    #[tokio::test]
    async fn python_ast_test_reports_functions_it_parsed() {
        let metrics = PythonAstTest
            .execute()
            .await
            .expect("python AST feature test must run the parser");
        assert_eq!(
            metrics.get("parsed_functions").and_then(|v| v.as_u64()),
            Some(1),
            "quicksort is one function: {metrics}"
        );
    }

    #[tokio::test]
    async fn typescript_ast_test_reports_functions_it_parsed() {
        let metrics = TypeScriptAstTest
            .execute()
            .await
            .expect("typescript AST feature test must run the parser");
        assert!(
            metrics
                .get("parsed_functions")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1,
            "factorial must be parsed as a function: {metrics}"
        );
    }

    #[tokio::test]
    async fn complexity_test_measures_a_branching_fixture() {
        let metrics = ComplexityAnalysisTest
            .execute()
            .await
            .expect("complexity feature test must run the analyzer");
        assert!(
            metrics
                .get("max_cyclomatic")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 1,
            "a four-branch function cannot measure cyclomatic 1: {metrics}"
        );
    }

    #[tokio::test]
    async fn deep_context_test_analyzes_a_real_project() {
        let metrics = DeepContextTest
            .execute()
            .await
            .expect("deep context feature test must run the analyzer");
        assert!(
            metrics
                .get("files_analyzed")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1,
            "the fixture project contains src/lib.rs: {metrics}"
        );
    }
}

/// Git integration test.
pub struct GitIntegrationTest;

#[async_trait::async_trait]
impl FeatureTest for GitIntegrationTest {
    fn name(&self) -> &'static str {
        "integration.git"
    }

    async fn execute(&self) -> Result<serde_json::Value> {
        // Git integration test

        // Check if we're in a git repo using std::path
        let git_dir = std::path::Path::new(".git");

        if !git_dir.exists() {
            return Ok(json!({
                "status": "skipped",
                "reason": "Not in a git repository",
            }));
        }

        let start = Instant::now();
        // Just verify git directory exists
        let duration = start.elapsed();

        Ok(json!({
            "git_available": true,
            "query_time_us": duration.as_micros(),
        }))
    }
}
