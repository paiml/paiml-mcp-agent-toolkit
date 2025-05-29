# Mermaid DAG Generation Bug Report

## Issue Summary
The DAG analysis feature generates Mermaid diagrams that fail to parse due to special characters in node labels, particularly when including complexity indicators with `<br/>` tags.

## Error Message
```
Parse error on line 2:
...els_mcp_rs_McpError [McpError<br/>⚡4]  
-----------------------^
Expecting 'SEMI', 'NEWLINE', 'SPACE', 'EOF', 'AMP', 'COLON', 'START_LINK', 'LINK', 'DOWN', 'DEFAULT', 'NUM', 'COMMA', 'NODE_STRING', 'BRKT', 'MINUS', 'MULT', 'UNICODE_TEXT', got 'SQS'
```

## Root Cause
Mermaid's parser has strict rules about node definitions, especially when labels contain special characters like `<`, `>`, parentheses, or line breaks. The complexity indicator format `<br/>⚡{number}` causes parsing failures.

## Attempted Fixes

### Attempt 1: Quote Function Labels
**Approach**: Added quotes around function node labels
```rust
NodeType::Function => format!("(\"{}\")", label),
```
**Result**: Failed with parse error - Mermaid interpreted the parentheses as part of the node ID syntax

### Attempt 2: Escape Special Characters in Trapezoid Shapes
**Approach**: Added conditional quoting for trapezoid shapes (Traits/Interfaces)
```rust
NodeType::Trait => {
    if label.contains("<br/>") {
        format!("[/\"{}\"\\]", label)
    } else {
        format!("[/{}\\]", label)
    }
}
```
**Result**: Rust compilation errors due to incorrect escape sequences

### Attempt 3: Simplify to Square Brackets
**Approach**: Use square brackets for all node types and differentiate with CSS
```rust
let node_def = match node.node_type {
    NodeType::Class => format!("[{}]", label),
    NodeType::Function => format!("[{}]", label), // CSS for rounded look
    NodeType::Module => format!("{{{{{}}}}}", label),
    NodeType::Trait => format!("[{}]", label), // CSS for trapezoid
    NodeType::Interface => format!("[{}]", label), // CSS for inverted trapezoid
};
```
**Result**: Still fails due to `<br/>` tag in labels

### Attempt 4: Add Visual Differentiation with CSS
**Approach**: Added stroke styles to differentiate node types
```rust
let (stroke_style, stroke_width) = match node.node_type {
    NodeType::Function => (",stroke:#333,stroke-dasharray: 5 5", 2),
    NodeType::Trait => (",stroke:#663399", 3),
    NodeType::Interface => (",stroke:#4169E1", 3),
    _ => ("", 2),
};
```
**Result**: CSS styling works but core parsing issue remains

## Current State
The generator produces output like:
```mermaid
graph TD
    server_src_models_mcp_rs_McpError [McpError<br/>⚡4]
    server_src_models_mod_rs_template {{template}}
```

Which fails to parse due to the `<br/>` tag inside square brackets.

## Possible Solutions

### Option 1: Remove HTML from Labels
Replace `<br/>` with a different separator:
```rust
let label = if self.options.show_complexity && node.complexity > 1 {
    format!("{} | ⚡{}", node.label, node.complexity)
} else {
    node.label.clone()
};
```

### Option 2: Use Mermaid's Built-in HTML Support
Mermaid supports HTML in specific contexts. We could try:
```rust
format!("[`{}`]", label)  // Backticks for code formatting
```

### Option 3: Move Complexity to Separate Line
Use Mermaid's multi-line syntax:
```rust
format!("[{}\\n⚡{}]", node.label, node.complexity)
```

### Option 4: Use Node IDs with Separate Labels
```rust
writeln!(&mut output, "    {} [\"{}\\n⚡{}\"]", 
    self.sanitize_id(id), 
    node.label, 
    node.complexity
)
```

## Complexity Analysis of Current Implementation

The `mermaid_generator.rs` file has the following complexity metrics:

### Actual Complexity Metrics (from paiml-mcp-agent-toolkit)
- **Files analyzed**: 1
- **Total functions**: 11
- **Average Cyclomatic Complexity**: 2.9
- **Average Cognitive Complexity**: 5.6

### Critical Issues:
- **MermaidGenerator::generate**: 
  - ❌ Cyclomatic complexity: **30** (exceeds max 20)
  - ❌ Cognitive complexity: **60** (exceeds max 30)
  - This is the main hotspot due to:
    - Multiple match statements for node types
    - Nested iteration over nodes for styling
    - Complex conditional logic for edge types
    - Additional complexity from show_complexity option

- **Code Structure**:
  - Lines of code: 388 (including comprehensive tests)
  - Number of test cases: 7 
  - Test coverage: All major code paths covered
  - Functions include: generate, sanitize_id, new, default + 7 test functions

## Recommendation
The most reliable fix would be Option 1 or Option 3 - avoiding HTML tags entirely and using simpler separators that Mermaid can parse reliably. The visual impact would be minimal while ensuring compatibility across all Mermaid renderers.

## References
- Mermaid documentation on node syntax: https://mermaid.js.org/syntax/flowchart.html
- Related issue: Node labels with special characters need careful escaping