/// Integration test for MCP discovery fixes
/// 
/// Tests the implementation from docs/todo/mcp-discovery-fixes.md
/// Validates >90% discovery success rate and <10ms initialization

#[cfg(test)]
mod discovery_integration_tests {
    use crate::mcp_pmcp::DiscoveryService;
    use std::time::Instant;

    // Test queries based on real Claude Code usage patterns
    const TEST_QUERIES: &[(&str, &str)] = &[
        // Exact matches
        ("analyze_complexity", "analyze_complexity"),
        ("quality_gate", "quality_gate"),
        ("scaffold_project", "scaffold_project"),
        
        // Alias matches
        ("complexity", "analyze_complexity"),
        ("debt", "analyze_satd"),
        ("technical debt", "analyze_satd"),
        ("dependencies", "analyze_dag"),
        ("dependency graph", "analyze_dag"),
        ("context", "generate_context"),
        ("quality check", "quality_gate"),
        ("refactor", "refactor.start"),
        ("git", "git_operation"),
        
        // Fuzzy matches
        ("complxity", "analyze_complexity"),
        ("complx", "analyze_complexity"),
        ("refactr", "refactor.start"),
        ("qualit", "quality_gate"),
        ("scafold", "scaffold_project"),
        
        // Natural language queries
        ("analyze code complexity", "analyze_complexity"),
        ("find technical debt", "analyze_satd"),
        ("show dependencies", "analyze_dag"),
        ("create project", "scaffold_project"),
        ("check code quality", "quality_gate"),
        ("start refactoring", "refactor.start"),
    ];

    #[test]
    fn test_discovery_success_rate() {
        let service = DiscoveryService::new();
        let mut successful_queries = 0;
        let mut failed_queries = Vec::new();

        for (query, expected_tool) in TEST_QUERIES {
            match service.resolve_tool(query) {
                Some(resolved_tool) => {
                    if resolved_tool == *expected_tool {
                        successful_queries += 1;
                    } else {
                        failed_queries.push(format!("Query '{}' resolved to '{}', expected '{}'", query, resolved_tool, expected_tool));
                    }
                }
                None => {
                    failed_queries.push(format!("Query '{}' failed to resolve (expected '{}')", query, expected_tool));
                }
            }
        }

        let success_rate = successful_queries as f64 / TEST_QUERIES.len() as f64;

        // Print failed queries for debugging
        if !failed_queries.is_empty() {
            println!("Failed queries:");
            for failure in &failed_queries {
                println!("  {}", failure);
            }
        }

        println!("Discovery success rate: {:.1}% ({}/{})", 
                success_rate * 100.0, successful_queries, TEST_QUERIES.len());

        // Verify >90% success rate as specified
        assert!(success_rate >= 0.90, 
               "Discovery success rate {:.1}% below target 90%", success_rate * 100.0);
    }

    #[test]
    fn test_initialization_performance() {
        // Test cold initialization
        let start = Instant::now();
        let _service = DiscoveryService::new();
        let init_time = start.elapsed();

        println!("Cold initialization time: {}ms", init_time.as_millis());

        // Should be <10ms as per specification
        assert!(init_time.as_millis() < 10, 
               "Initialization took {}ms, target <10ms", init_time.as_millis());

        // Test hot initialization (should be even faster)
        let start = Instant::now();
        let _service2 = DiscoveryService::new();
        let hot_init_time = start.elapsed();

        println!("Hot initialization time: {}ms", hot_init_time.as_millis());
        assert!(hot_init_time <= init_time, "Hot initialization should be <= cold initialization");
    }

    #[test]
    fn test_query_resolution_performance() {
        let service = DiscoveryService::new();
        
        let test_queries = vec![
            "analyze_complexity",
            "complexity", 
            "complxity",
            "debt",
            "quality",
            "refactor",
            "dependencies",
            "context",
        ];

        // Warm up
        for query in &test_queries {
            let _ = service.resolve_tool(query);
        }

        // Measure performance
        let start = Instant::now();
        for query in &test_queries {
            let _ = service.resolve_tool(query);
        }
        let total_time = start.elapsed();
        let avg_time = total_time / test_queries.len() as u32;

        println!("Average query resolution time: {}μs", avg_time.as_micros());

        // Should be <5ms as per specification
        assert!(avg_time.as_millis() < 5, 
               "Average query resolution {}ms, target <5ms", avg_time.as_millis());
    }

    #[test]
    fn test_comprehensive_tool_coverage() {
        let service = DiscoveryService::new();
        let tools = service.list_tools();

        // Verify all expected tools are present
        let expected_tools = vec![
            "analyze_complexity",
            "analyze_satd", 
            "analyze_dead_code",
            "analyze_dag",
            "analyze_deep_context",
            "analyze_big_o",
            "refactor.start",
            "refactor.nextIteration",
            "refactor.getState",
            "refactor.stop",
            "quality_gate",
            "quality_proxy",
            "git_operation",
            "generate_context",
            "scaffold_project",
        ];

        for expected_tool in &expected_tools {
            assert!(tools.iter().any(|t| t.name == *expected_tool),
                   "Missing expected tool: {}", expected_tool);
        }

        println!("Total tools available: {}", tools.len());
        assert_eq!(tools.len(), expected_tools.len(), 
                  "Tool count mismatch. Expected {}, got {}", expected_tools.len(), tools.len());
    }

    #[test]
    fn test_disambiguation() {
        let service = DiscoveryService::new();
        
        // Test category priority
        let candidates = vec!["analyze_complexity", "scaffold_project"];
        let result = service.disambiguate(candidates, None);
        assert_eq!(result, "scaffold_project", "Should prefer Generate category over Analyze");

        // Test file extension context
        use crate::mcp_pmcp::Context;
        let context = Context {
            file_extension: Some("rs".to_string()),
            current_directory: None,
            recent_tools: vec![],
        };
        
        let candidates = vec!["analyze_dag", "analyze_complexity"];
        let result = service.disambiguate(candidates, Some(&context));
        assert_eq!(result, "analyze_complexity", "Should prefer complexity analysis for Rust files");
    }

    #[test]
    fn test_memory_usage() {
        // This is a basic smoke test for memory usage
        let service = DiscoveryService::new();
        let tools = service.list_tools();
        
        // Should have reasonable tool metadata
        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool name should not be empty");
            assert!(!tool.description.is_empty(), "Tool description should not be empty");
            assert!(!tool.keywords.is_empty(), "Tool should have keywords");
        }
        
        // Test that we can create multiple instances without issues
        let _services: Vec<_> = (0..10).map(|_| DiscoveryService::new()).collect();
    }

    #[test]
    fn test_edge_cases() {
        let service = DiscoveryService::new();

        // Empty query
        assert_eq!(service.resolve_tool(""), None);

        // Very short query
        assert_eq!(service.resolve_tool("a"), None);

        // Very long query
        let long_query = "a".repeat(1000);
        let _result = service.resolve_tool(&long_query); // Should not crash

        // Special characters
        assert_eq!(service.resolve_tool("@#$%"), None);

        // Mixed case
        assert_eq!(service.resolve_tool("COMPLEXITY"), Some("analyze_complexity"));
        assert_eq!(service.resolve_tool("Quality_Gate"), Some("quality_gate"));
    }
}