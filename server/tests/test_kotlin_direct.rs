#[cfg(feature = "kotlin-ast")]
#[tokio::test]
async fn test_kotlin_parser_directly() {
    use paiml_mcp_agent_toolkit::services::ast_strategies::{AstStrategy, KotlinAstStrategy};
    use paiml_mcp_agent_toolkit::services::file_classifier::FileClassifier;
    use std::path::Path;

    let path = Path::new("../test_kotlin_project/main.kt");
    let classifier = FileClassifier::new();
    let strategy = KotlinAstStrategy;

    match strategy.analyze(path, &classifier).await {
        Ok(context) => {
            println!(
                "Success! Language: {}, Items: {}",
                context.language,
                context.items.len()
            );
            assert_eq!(context.language, "kotlin");
            assert!(
                !context.items.is_empty(),
                "Should have found at least one function"
            );
        }
        Err(e) => {
            panic!("Kotlin parsing failed: {}", e);
        }
    }
}
