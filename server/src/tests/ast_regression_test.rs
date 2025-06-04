//! AST analysis regression test
//!
//! This test ensures that the deep context analysis always includes AST analysis
//! and prevents regressions where AST analysis returns empty results.

use tempfile::TempDir;

#[tokio::test]
async fn test_ast_analysis_not_empty_regression() {
    // Create a temporary directory with sample source files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create a sample Rust file
    let rust_file = temp_path.join("sample.rs");
    std::fs::write(
        &rust_file,
        r#"
pub struct TestStruct {
    pub field: i32,
}

impl TestStruct {
    pub fn new(field: i32) -> Self {
        Self { field }
    }
    
    pub fn get_field(&self) -> i32 {
        self.field
    }
}

pub fn test_function() -> String {
    "hello world".to_string()
}

pub enum TestEnum {
    Variant1,
    Variant2(i32),
}
"#,
    )
    .expect("Failed to write Rust file");

    // Create a sample TypeScript file
    let ts_file = temp_path.join("sample.ts");
    std::fs::write(
        &ts_file,
        r#"
export interface TestInterface {
    field: number;
}

export class TestClass implements TestInterface {
    field: number;
    
    constructor(field: number) {
        this.field = field;
    }
    
    getField(): number {
        return this.field;
    }
}

export function testFunction(): string {
    return "hello world";
}
"#,
    )
    .expect("Failed to write TypeScript file");

    // Create a sample Python file
    let py_file = temp_path.join("sample.py");
    std::fs::write(
        &py_file,
        r#"
class TestClass:
    def __init__(self, field: int):
        self.field = field
    
    def get_field(self) -> int:
        return self.field

def test_function() -> str:
    return "hello world"

def another_function(x: int, y: int) -> int:
    return x + y
"#,
    )
    .expect("Failed to write Python file");

    // Test via the full deep context analysis since analyze_ast_contexts is private
    use crate::services::deep_context::{AnalysisType, DeepContextAnalyzer, DeepContextConfig};

    // Only run AST analysis to focus the test
    let config = DeepContextConfig {
        include_analyses: vec![AnalysisType::Ast],
        ..Default::default()
    };

    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = analyzer
        .analyze_project(&temp_path.to_path_buf())
        .await
        .expect("Deep context analysis should not fail");

    let ast_contexts = &deep_context.analyses.ast_contexts;

    // CRITICAL REGRESSION CHECK: AST analysis must not return empty results
    assert!(
        !ast_contexts.is_empty(),
        "AST analysis returned empty contexts - this is a regression! \
         AST analysis must parse source files and return file contexts."
    );

    // Verify we have contexts for each file type
    let rust_contexts: Vec<_> = ast_contexts
        .iter()
        .filter(|ctx| ctx.base.language == "rust")
        .collect();
    let ts_contexts: Vec<_> = ast_contexts
        .iter()
        .filter(|ctx| ctx.base.language == "typescript")
        .collect();
    let py_contexts: Vec<_> = ast_contexts
        .iter()
        .filter(|ctx| ctx.base.language == "python")
        .collect();

    assert!(!rust_contexts.is_empty(), "Should have Rust file contexts");
    assert!(
        !ts_contexts.is_empty(),
        "Should have TypeScript file contexts"
    );
    assert!(!py_contexts.is_empty(), "Should have Python file contexts");

    // Verify AST items are actually extracted
    let rust_ctx = &rust_contexts[0];
    assert!(
        !rust_ctx.base.items.is_empty(),
        "Rust file context should contain AST items (functions, structs, etc.)"
    );

    // Check that we found the expected items in the Rust file
    let rust_functions: Vec<_> = rust_ctx
        .base
        .items
        .iter()
        .filter(|item| matches!(item, crate::services::context::AstItem::Function { .. }))
        .collect();
    let rust_structs: Vec<_> = rust_ctx
        .base
        .items
        .iter()
        .filter(|item| matches!(item, crate::services::context::AstItem::Struct { .. }))
        .collect();
    let rust_enums: Vec<_> = rust_ctx
        .base
        .items
        .iter()
        .filter(|item| matches!(item, crate::services::context::AstItem::Enum { .. }))
        .collect();

    assert!(
        !rust_functions.is_empty(),
        "Should find functions in Rust file"
    );
    assert!(!rust_structs.is_empty(), "Should find structs in Rust file");
    assert!(!rust_enums.is_empty(), "Should find enums in Rust file");

    println!("✅ AST regression test passed:");
    println!("  - Generated {} file contexts", ast_contexts.len());
    println!("  - Rust contexts: {}", rust_contexts.len());
    println!("  - TypeScript contexts: {}", ts_contexts.len());
    println!("  - Python contexts: {}", py_contexts.len());
    println!("  - Rust AST items: {}", rust_ctx.base.items.len());
}

#[tokio::test]
async fn test_deep_context_includes_ast_analysis() {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    // Create a temporary project with source files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create a sample file that should be analyzed
    let rust_file = temp_path.join("lib.rs");
    std::fs::write(
        &rust_file,
        r#"
pub fn sample_function() -> i32 {
    42
}
"#,
    )
    .expect("Failed to write sample file");

    // Run full deep context analysis with all default analyses enabled
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = analyzer
        .analyze_project(&temp_path.to_path_buf())
        .await
        .expect("Deep context analysis should succeed");

    // CRITICAL: Verify AST analysis is included and not empty
    assert!(
        !deep_context.analyses.ast_contexts.is_empty(),
        "Deep context analysis must include non-empty AST contexts. \
         This is a critical regression - AST analysis is not working!"
    );

    // Verify that when rendered as markdown, it includes AST-related content
    let markdown = crate::services::context::format_deep_context_as_markdown(&deep_context);
    // Check for AST content in a more lenient way since the exact section title may vary
    let has_ast_content = markdown.contains("Enhanced AST Analysis")
        || markdown.contains("AST Analysis")
        || markdown.contains("Language:")
        || markdown.contains("Total Symbols:");

    assert!(
        has_ast_content,
        "Rendered deep context markdown should include AST-related content. \
         Got markdown length: {} chars",
        markdown.len()
    );

    println!(
        "✅ Deep context AST integration test passed - markdown length: {} chars",
        markdown.len()
    );
}
