//! Example demonstrating the Unified CLI/MCP/Help Integration (Issue #118)
//!
//! This example shows how pmat provides a single source of truth for all
//! command metadata, enabling:
//! - Dynamic --help generation
//! - MCP tool schema auto-generation
//! - RAG-powered semantic help search
//! - Documentation drift detection
//!
//! Run with: cargo run --example unified_help_demo

use pmat::cli::{
    CommandMetadata, CommandRegistry, HelpGenerator, HelpResponse, McpSchemaGenerator,
    UnifiedHelpService,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PMAT Unified CLI/MCP/Help Integration Demo ===\n");
    println!("This demo shows the single source of truth architecture");
    println!("that prevents documentation drift (Issue #118).\n");

    // Step 1: Create the CommandRegistry - single source of truth
    println!("=== Step 1: CommandRegistry (Single Source of Truth) ===\n");

    let mut registry = CommandRegistry::new(env!("CARGO_PKG_VERSION"));

    // Register analyze command with subcommands
    let complexity_sub = CommandMetadata::builder("complexity")
        .short_description("Analyze cyclomatic and cognitive complexity")
        .long_description(
            "Calculates cyclomatic complexity (CC) and cognitive complexity \
             for all functions in the project. Supports Rust, Python, TypeScript, \
             Go, and 15+ other languages.",
        )
        .argument(pmat::cli::registry::ArgumentMetadata {
            name: "path".to_string(),
            short: Some('p'),
            long: Some("path".to_string()),
            description: "Path to analyze".to_string(),
            required: false,
            default: Some(".".to_string()),
            value_type: pmat::cli::registry::ValueType::Path,
            ..Default::default()
        })
        .argument(pmat::cli::registry::ArgumentMetadata {
            name: "threshold".to_string(),
            short: Some('t'),
            long: Some("threshold".to_string()),
            description: "Complexity threshold for warnings".to_string(),
            required: false,
            default: Some("10".to_string()),
            value_type: pmat::cli::registry::ValueType::Integer,
            ..Default::default()
        })
        .example(pmat::cli::registry::ExampleMetadata {
            description: "Analyze current directory".to_string(),
            command: "pmat analyze complexity".to_string(),
            expected_exit_code: 0,
            output_patterns: vec!["CC=".to_string()],
            requires_project: false,
            project_type: None,
        })
        .example(pmat::cli::registry::ExampleMetadata {
            description: "With custom threshold".to_string(),
            command: "pmat analyze complexity --threshold 15".to_string(),
            expected_exit_code: 0,
            output_patterns: vec![],
            requires_project: false,
            project_type: None,
        })
        .category("analysis")
        .tags(["metrics", "complexity", "quality"])
        .build();

    let satd_sub = CommandMetadata::builder("satd")
        .short_description("Detect self-admitted technical debt")
        .long_description(
            "Finds TODO, FIXME, HACK, and other technical debt markers in code comments. \
             Categorizes by severity and provides actionable insights.",
        )
        .category("analysis")
        .tags(["debt", "quality", "comments"])
        .build();

    let dead_code_sub = CommandMetadata::builder("dead-code")
        .short_description("Find unused code")
        .long_description("Identifies unreachable functions, unused imports, and dead code paths.")
        .category("analysis")
        .tags(["dead-code", "cleanup", "quality"])
        .build();

    registry.register(
        CommandMetadata::builder("analyze")
            .short_description("Run code analysis")
            .long_description("Comprehensive suite of code analysis tools")
            .subcommand(complexity_sub)
            .subcommand(satd_sub)
            .subcommand(dead_code_sub)
            .category("analysis")
            .build(),
    );

    // Register context command
    registry.register(
        CommandMetadata::builder("context")
            .short_description("Generate project context for AI assistants")
            .long_description(
                "Creates a comprehensive markdown document describing your project's \
                 structure, dependencies, and architecture. Perfect for providing context \
                 to Claude, ChatGPT, or other AI assistants.",
            )
            .aliases(["ctx"])
            .category("generation")
            .tags(["ai", "context", "markdown"])
            .build(),
    );

    // Register quality-gate command
    registry.register(
        CommandMetadata::builder("quality-gate")
            .short_description("Run quality checks")
            .long_description("Enforces code quality standards before commits or releases.")
            .aliases(["qg"])
            .category("enforcement")
            .tags(["quality", "ci", "enforcement"])
            .build(),
    );

    println!("Registered {} commands", registry.commands.len());
    println!(
        "Commands: {:?}\n",
        registry.commands.keys().collect::<Vec<_>>()
    );

    // Step 2: Dynamic --help Generation
    println!("=== Step 2: Dynamic --help Generation ===\n");

    let help_gen = HelpGenerator::new(registry.clone());

    println!("--- Command Overview ---");
    println!("{}", help_gen.generate_overview());

    println!("\n--- Specific Command Help ---");
    println!("{}", help_gen.generate("analyze complexity"));

    println!("\n--- Typo Suggestion ---");
    let typo_help = help_gen.generate("analize");
    println!("{}", typo_help);

    // Step 3: MCP Schema Generation
    println!("\n=== Step 3: MCP Schema Auto-Generation ===\n");

    let mcp_gen = McpSchemaGenerator::new(registry.clone());
    let tools = mcp_gen.generate_tools_list();

    println!("Generated {} MCP tool definitions", tools.len());
    if let Some(tool) = tools.iter().find(|t| t.name == "analyze_complexity") {
        println!(
            "\nSample tool schema (analyze_complexity):\n{}",
            serde_json::to_string_pretty(tool)?
        );
    } else {
        println!("\n(Note: No MCP-enabled commands in demo registry)");
    }

    // Step 4: RAG-Powered Semantic Help
    println!("\n=== Step 4: RAG-Powered Semantic Help ===\n");

    let unified_help = UnifiedHelpService::new(registry.clone());

    // Semantic search
    let queries = [
        "how to find complex functions",
        "technical debt",
        "generate context for AI",
    ];

    for query in &queries {
        println!("Query: \"{}\"", query);
        let results = unified_help.search(query, 2);
        if !results.is_empty() {
            for result in results {
                println!(
                    "  -> {} (score: {:.2}): {}",
                    result.command, result.combined_score, result.snippet
                );
            }
        } else {
            println!("  No results found");
        }
        println!();
    }

    // Intelligent lookup
    println!("--- Intelligent Lookup ---");
    let lookup_tests = ["analyze complexity", "ctx", "find bugs"];

    for query in &lookup_tests {
        println!("Lookup: \"{}\"", query);
        match unified_help.lookup(query) {
            HelpResponse::Exact(cmd) => {
                println!("  Exact match: {} - {}", cmd.name, cmd.short_description);
            }
            HelpResponse::DidYouMean {
                suggestion,
                confidence,
            } => {
                println!(
                    "  Did you mean: '{}' (confidence: {:.2})?",
                    suggestion, confidence
                );
            }
            HelpResponse::SearchResults { query: q, results } => {
                println!("  Semantic search for '{}':", q);
                for r in results.iter().take(2) {
                    println!("    - {} (score: {:.2})", r.command, r.combined_score);
                }
            }
        }
        println!();
    }

    // Step 5: Benefits Summary
    println!("=== Benefits of Unified Architecture ===\n");
    println!("1. Single Source of Truth:");
    println!("   - CommandRegistry defines all metadata ONCE");
    println!("   - --help, MCP schemas, docs all generated from same source");
    println!("   - No more drift between CLI and documentation\n");

    println!("2. Error Prevention (Poka-yoke):");
    println!("   - DriftDetector validates docs against actual commands");
    println!("   - Pre-commit hooks catch references to non-existent commands");
    println!("   - Impossible to document commands that don't exist\n");

    println!("3. Enhanced Developer Experience:");
    println!("   - Semantic search finds commands by intent, not just name");
    println!("   - Typo suggestions help users find the right command");
    println!("   - PageRank identifies most important commands\n");

    println!("4. MCP Integration:");
    println!("   - Tool schemas auto-generated from CLI definitions");
    println!("   - Arguments, types, descriptions stay in sync");
    println!("   - AI assistants always have accurate tool information\n");

    println!("=== Demo Complete ===");
    println!("\nFor more information:");
    println!("  - Specification: docs/specifications/unified-cli-mcp-help-integration.md");
    println!("  - GitHub Issue: #118");
    println!("  - Toyota Way principles: Jidoka, Poka-yoke, Genchi Genbutsu");

    Ok(())
}
