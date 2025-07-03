# MCP Methods and Tools Documentation

This document describes the MCP (Model Context Protocol) methods and tools available in PMAT.

### Available MCP Methods

The following standard MCP methods are supported:

- `initialize` - Initialize the MCP connection
- `tools/list` - List available tools
- `tools/call` - Call a specific tool with parameters
- `resources/list` - List available resources
- `resources/read` - Read a specific resource
- `prompts/list` - List available prompts

### Available Tools

The following tools are available through the MCP protocol:

### get_server_info
Get information about the server and its capabilities.

### analyze_complexity
Analyze code complexity metrics including cyclomatic and cognitive complexity.

### analyze_churn
Analyze code change patterns over time.

### analyze_code_churn
Analyze code change patterns over time (alias for analyze_churn).

### generate_context
Generate comprehensive project context for AI assistants.

### analyze_dag
Generate and analyze dependency graphs.

### analyze_dead_code
Detect unused code in the project.

### analyze_deep_context
Perform deep contextual analysis of the codebase.

### analyze_satd
Analyze self-admitted technical debt in code comments.

### analyze_tdg
Calculate technical debt gradient based on complexity and churn.

### analyze_duplicates_vectorized
Analyze code duplicates using SIMD optimizations.

### analyze_graph_metrics_vectorized
Analyze graph metrics using vectorization.

### analyze_name_similarity_vectorized
Analyze name similarity using SIMD.

### analyze_symbol_table_vectorized
Analyze symbol tables with vectorization.

### analyze_incremental_coverage_vectorized
Analyze incremental coverage with SIMD.

### analyze_big_o_vectorized
Analyze Big O complexity using vectorization.

### generate_enhanced_report
Generate enhanced analysis report.

### generate_template
Generate templates with parameter substitution.

### list_templates
List available templates.

### scaffold_project
Scaffold a complete project.

### search_templates
Search for templates.

### validate_template
Validate template parameters.

### Error Codes

The following JSON-RPC error codes are used:

| Code | Description |
|------|-------------|
| -32700 | Parse error: Invalid JSON |
| -32600 | Invalid request: Missing required fields |
| -32601 | Method not found: Unknown method |
| -32602 | Invalid params: Invalid method parameters |