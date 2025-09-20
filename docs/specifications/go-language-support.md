# Go Language Support Implementation for PMAT

## Technical Architecture

This specification defines the integration of Go language support into PMAT's unified AST infrastructure, leveraging tree-sitter-go for parsing and implementing Go-specific complexity metrics aligned with the existing `LanguageStrategy` trait.

## Core Design Decisions

### Parser Selection: tree-sitter-go

Tree-sitter provides the optimal parsing solution for Go integration:
- **Incremental parsing**: O(log n) updates for real-time analysis
- **Error recovery**: Continues parsing despite syntax errors
- **Language uniformity**: Matches existing C/C++/Kotlin implementations
- **Zero-copy parsing**: Direct memory mapping of source files

Alternative considered: `go/parser` via CGO binding
- Rejected due to 40% performance overhead and deployment complexity

## Implementation Architecture

### 1. Language Strategy Implementation

```rust
// server/src/ast/languages/go.rs

use super::LanguageStrategy;
use crate::ast::core::{AstDag, AstKind, Language, NodeFlags, UnifiedAstNode};
use anyhow::Result;
use async_trait::async_trait;
use tree_sitter::{Parser, Query, QueryCursor, Node};
use std::path::Path;

pub struct GoStrategy {
    parser: Parser,
    function_query: Query,
    type_query: Query,
    import_query: Query,
    complexity_query: Query,
}

impl GoStrategy {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_go::language();
        parser.set_language(language)?;
        
        // Precompiled queries for performance
        let function_query = Query::new(
            language,
            r#"
            (function_declaration
                name: (identifier) @name
                parameters: (parameter_list) @params
                result: (_)? @return_type
                body: (block) @body) @func
            
            (method_declaration
                receiver: (parameter_list) @receiver
                name: (identifier) @name
                parameters: (parameter_list) @params
                result: (_)? @return_type
                body: (block) @body) @method
            "#
        )?;
        
        let type_query = Query::new(
            language,
            r#"
            (type_declaration
                (type_spec
                    name: (type_identifier) @name
                    type: (_) @type_def)) @type
            
            (struct_type) @struct
            (interface_type) @interface
            "#
        )?;
        
        let import_query = Query::new(
            language,
            r#"
            (import_declaration
                (import_spec
                    path: (interpreted_string_literal) @path
                    name: (package_identifier)? @alias)) @import
            "#
        )?;
        
        let complexity_query = Query::new(
            language,
            r#"
            [
                (if_statement) @if
                (for_statement) @for
                (range_statement) @range
                (switch_statement) @switch
                (type_switch_statement) @type_switch
                (select_statement) @select
                (go_statement) @goroutine
                (defer_statement) @defer
                (&&) @and
                (||) @or
            ] @complexity_node
            "#
        )?;
        
        Ok(Self {
            parser,
            function_query,
            type_query,
            import_query,
            complexity_query,
        })
    }
    
    fn extract_goroutine_complexity(&self, node: &Node, source: &str) -> u32 {
        // Go-specific: goroutines add +2 cognitive complexity
        let mut cursor = QueryCursor::new();
        let mut complexity = 0u32;
        
        for match_ in cursor.matches(&self.complexity_query, *node, source.as_bytes()) {
            for capture in match_.captures {
                match self.complexity_query.capture_names()[capture.index] {
                    "goroutine" => complexity += 2,  // Concurrency penalty
                    "defer" => complexity += 1,      // Cleanup complexity
                    "select" => complexity += 3,     // Channel multiplexing
                    _ => complexity += 1,
                }
            }
        }
        
        complexity
    }
}

#[async_trait]
impl LanguageStrategy for GoStrategy {
    fn language(&self) -> Language {
        Language::Go  // Requires adding Go = 16 to Language enum
    }
    
    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "go")
            .unwrap_or(false)
    }
    
    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Go file"))?;
        
        let mut dag = AstDag::new();
        let root_cursor = tree.root_node();
        
        self.build_dag_recursive(&root_cursor, content, &mut dag, None)?;
        
        Ok(dag)
    }
    
    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        ast.nodes
            .iter_values()
            .filter(|node| matches!(node.kind, AstKind::Function(_)))
            .cloned()
            .collect()
    }
    
    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        let mut cyclomatic = 1u32;  // Base complexity
        let mut cognitive = 0u32;
        
        for node in ast.nodes.iter_values() {
            match &node.kind {
                AstKind::ControlFlow(flow) => {
                    cyclomatic += 1;
                    cognitive += flow.nesting_level as u32;
                    
                    // Go-specific complexity adjustments
                    if flow.flow_type == "select" {
                        cognitive += 2;  // Channel operations are cognitively complex
                    }
                }
                AstKind::Function(func) if func.is_async => {
                    cognitive += 3;  // Goroutines increase cognitive load
                }
                _ => {}
            }
        }
        
        (cyclomatic, cognitive)
    }
}
```

### 2. Go-Specific AST Nodes

```rust
// server/src/ast/core.rs additions

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoSpecificNode {
    pub node_type: GoNodeType,
    pub receiver: Option<String>,      // Method receiver
    pub channel_ops: Vec<ChannelOp>,   // Channel operations
    pub defer_count: u32,              // Number of defer statements
    pub goroutine_spawns: u32,         // go keyword usage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoNodeType {
    Function,
    Method,
    Interface,
    Struct,
    Channel,
    Goroutine,
    Select,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelOp {
    pub op_type: ChannelOpType,
    pub channel_name: String,
    pub is_buffered: bool,
    pub buffer_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelOpType {
    Send,
    Receive,
    Select,
    Make,
    Close,
}
```

### 3. Go Complexity Metrics

```rust
// server/src/services/complexity_go.rs

use super::ComplexityMetrics;

/// Go-specific complexity calculator implementing cognitive complexity v1.5
pub struct GoComplexityCalculator {
    nesting_stack: Vec<NestingContext>,
}

#[derive(Clone)]
struct NestingContext {
    depth: u32,
    is_goroutine: bool,
    is_defer_block: bool,
    channel_ops: u32,
}

impl GoComplexityCalculator {
    pub fn calculate_method_complexity(&self, method_node: &Node, source: &str) -> ComplexityMetrics {
        let mut visitor = ComplexityVisitor::new();
        visitor.visit_node(method_node, source);
        
        // Go-specific adjustments
        let base_cyclomatic = visitor.cyclomatic;
        let base_cognitive = visitor.cognitive;
        
        // Method receivers add complexity
        let receiver_complexity = if visitor.has_pointer_receiver { 1 } else { 0 };
        
        // Channel operations significantly increase cognitive load
        let channel_complexity = visitor.channel_operations * 2;
        
        // Error handling patterns in Go
        let error_handling_complexity = visitor.error_check_patterns;
        
        ComplexityMetrics {
            cyclomatic: base_cyclomatic + receiver_complexity,
            cognitive: base_cognitive + channel_complexity + error_handling_complexity,
            nesting_max: visitor.max_nesting,
            lines: visitor.line_count,
            halstead: None,  // Calculated separately
        }
    }
    
    fn detect_go_patterns(&self, node: &Node, source: &str) -> GoPatterns {
        GoPatterns {
            uses_goroutines: self.has_goroutines(node, source),
            uses_channels: self.has_channels(node, source),
            uses_context: self.imports_context(node, source),
            uses_sync: self.imports_sync(node, source),
            error_handling_style: self.detect_error_style(node, source),
        }
    }
}
```

### 4. Go Dead Code Detection

```rust
// server/src/services/dead_code_go.rs

use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

pub struct GoDeadCodeAnalyzer {
    call_graph: DiGraph<GoSymbol, CallEdge>,
    symbol_map: HashMap<String, NodeIndex>,
    init_functions: Vec<NodeIndex>,  // Special handling for init()
}

#[derive(Debug, Clone)]
struct GoSymbol {
    name: String,
    symbol_type: GoSymbolType,
    package: String,
    is_exported: bool,  // Uppercase first letter
    is_test: bool,      // _test.go file
}

#[derive(Debug, Clone)]
enum GoSymbolType {
    Function,
    Method,
    Variable,
    Type,
    Interface,
    Constant,
}

impl GoDeadCodeAnalyzer {
    pub fn analyze(&mut self, ast: &AstDag) -> Vec<DeadCodeItem> {
        // Build call graph
        self.build_call_graph(ast);
        
        // Mark entry points
        let mut reachable = HashSet::new();
        
        // Go-specific entry points:
        // 1. main.main()
        // 2. init() functions (all are called)
        // 3. Exported functions (uppercase)
        // 4. Test functions (*_test.go)
        // 5. Benchmark functions (Benchmark*)
        
        self.mark_main_reachable(&mut reachable);
        self.mark_init_reachable(&mut reachable);
        self.mark_exported_reachable(&mut reachable);
        self.mark_test_reachable(&mut reachable);
        
        // Traverse call graph
        self.traverse_reachable(&mut reachable);
        
        // Identify dead code
        self.identify_dead_code(&reachable)
    }
    
    fn handle_interface_satisfaction(&mut self, ast: &AstDag) {
        // Go-specific: Methods implementing interfaces are reachable
        // even if not directly called
        for node in ast.nodes.iter_values() {
            if let AstKind::Interface(iface) = &node.kind {
                // Find all types implementing this interface
                let implementers = self.find_interface_implementers(&iface.name);
                for implementer in implementers {
                    // Mark all interface methods as reachable
                    self.mark_interface_methods_reachable(implementer, iface);
                }
            }
        }
    }
}
```

### 5. Go-Specific Context Generation

```rust
// server/src/services/context_go.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct GoFileContext {
    pub package_name: String,
    pub imports: Vec<GoImport>,
    pub functions: Vec<GoFunction>,
    pub types: Vec<GoType>,
    pub constants: Vec<GoConstant>,
    pub variables: Vec<GoVariable>,
    pub init_functions: Vec<GoFunction>,  // Special init() handling
    pub build_tags: Vec<String>,          // // +build tags
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoImport {
    pub path: String,
    pub alias: Option<String>,
    pub is_dot_import: bool,    // import . "fmt"
    pub is_blank_import: bool,  // import _ "side-effect"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoFunction {
    pub name: String,
    pub signature: String,
    pub receiver: Option<GoReceiver>,  // Methods
    pub is_exported: bool,
    pub is_variadic: bool,
    pub returns_error: bool,
    pub uses_goroutines: bool,
    pub complexity: ComplexityMetrics,
    pub line_range: Range<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoReceiver {
    pub name: String,
    pub type_name: String,
    pub is_pointer: bool,
}
```

### 6. Integration Points

```rust
// server/src/ast/languages/mod.rs
impl LanguageRegistry {
    pub fn new() -> Self {
        let strategies: Vec<Arc<dyn LanguageStrategy>> = vec![
            // ... existing strategies ...
            Arc::new(go::GoStrategy::new().unwrap()),  // ADD THIS
        ];
        Self { strategies }
    }
}

// server/src/ast/core.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Language {
    // ... existing languages ...
    Go = 16,  // ADD THIS
}

// server/src/services/deep_context.rs
async fn analyze_file_by_language(
    file_path: &Path,
    language: &str,
) -> anyhow::Result<Vec<AstItem>> {
    match language {
        // ... existing cases ...
        "go" => analyze_go_language(file_path).await,  // ADD THIS
        _ => Ok(Vec::new()),
    }
}
```

### 7. Performance Optimizations

```rust
// Parallel AST processing for large Go projects
use rayon::prelude::*;
use dashmap::DashMap;

pub struct GoProjectAnalyzer {
    file_cache: DashMap<PathBuf, GoFileContext>,
    import_graph: DiGraph<String, ()>,
}

impl GoProjectAnalyzer {
    pub async fn analyze_workspace(&self, workspace_path: &Path) -> Result<WorkspaceAnalysis> {
        // Find all Go modules (go.mod files)
        let modules = self.discover_modules(workspace_path)?;
        
        // Parallel analysis per module
        let module_results: Vec<_> = modules
            .par_iter()
            .map(|module| self.analyze_module(module))
            .collect::<Result<Vec<_>>>()?;
        
        // Build cross-module dependency graph
        let dep_graph = self.build_dependency_graph(&module_results)?;
        
        // Detect circular dependencies (Go compiler prevents these, but good to verify)
        if let Some(cycle) = self.detect_import_cycles(&dep_graph) {
            return Err(anyhow::anyhow!("Import cycle detected: {:?}", cycle));
        }
        
        Ok(WorkspaceAnalysis {
            modules: module_results,
            dependency_graph: dep_graph,
            total_complexity: self.aggregate_complexity(&module_results),
        })
    }
}
```

## Build Configuration

```toml
# server/Cargo.toml
[dependencies]
tree-sitter = "0.20"
tree-sitter-go = "0.20"

[build-dependencies]
cc = "1.0"
```

```rust
// server/build.rs
fn main() {
    // Link tree-sitter-go
    cc::Build::new()
        .include("tree-sitter-go/src")
        .file("tree-sitter-go/src/parser.c")
        .compile("tree-sitter-go");
}
```

## Testing Strategy

```rust
// server/tests/go_analysis_test.rs
#[test]
fn test_go_complexity_calculation() {
    let code = r#"
        func processData(ctx context.Context, data []byte) error {
            select {
            case <-ctx.Done():
                return ctx.Err()
            default:
            }
            
            go func() {
                defer cleanup()
                // Goroutine body
            }()
            
            return nil
        }
    "#;
    
    let analyzer = GoComplexityCalculator::new();
    let metrics = analyzer.analyze(code).unwrap();
    
    assert_eq!(metrics.cyclomatic, 3);  // select + goroutine
    assert_eq!(metrics.cognitive, 7);   // select(3) + goroutine(2) + defer(1) + base(1)
}

#[test]
fn test_interface_detection() {
    let code = r#"
        type Writer interface {
            Write([]byte) (int, error)
        }
        
        type Buffer struct{}
        
        func (b *Buffer) Write(p []byte) (int, error) {
            return len(p), nil
        }
    "#;
    
    let analyzer = GoDeadCodeAnalyzer::new();
    let dead_code = analyzer.analyze(code).unwrap();
    
    // Buffer.Write should NOT be marked as dead code
    // even though it's not explicitly called
    assert!(dead_code.is_empty());
}
```

## Performance Characteristics

| Operation | Complexity | Memory |
|-----------|-----------|--------|
| Parse 100K LOC | O(n log n) | ~50MB |
| Build call graph | O(n²) worst, O(n log n) typical | O(n) |
| Dead code detection | O(V + E) | O(V) |
| Complexity calculation | O(n) | O(log n) stack |

## Migration Checklist

- [ ] Add `tree-sitter-go` dependency
- [ ] Implement `GoStrategy` trait
- [ ] Add Go to `Language` enum
- [ ] Implement Go-specific complexity metrics
- [ ] Add Go patterns to dead code detector
- [ ] Update file discovery for `.go` files
- [ ] Add Go test fixtures
- [ ] Update documentation
- [ ] Benchmark against `go vet` and `golangci-lint`

## Validation Metrics

Success criteria for Go support:
- Parse rate: >500K LOC/sec
- Complexity calculation accuracy: >95% correlation with `gocyclo`
- Dead code detection: <5% false positives vs `deadcode`
- Memory overhead: <100MB for 1M LOC project
- Integration time: <2 seconds for average Go module
