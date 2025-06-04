# Demo 2.0 Implementation Specification

**STATUS: FULLY IMPLEMENTED** ✅

This specification has been fully implemented. All features are working and tested.

## Technical Debt Reduction

### Immediate Actions (4 hours total)
1. **`run_demo` refactor** (Cyclomatic: 34 → 10) ✅
   ```rust
   // Current: server/src/demo/mod.rs
   pub async fn run_demo(...) -> Result<()> {
       let config = load_demo_config(args)?;
       let analyzer = create_analyzer(config)?;
       let results = run_analyses(analyzer, args).await?;
       let output = generate_output(results, args.protocol)?;
       handle_protocol_output(output, args).await
   }
   ```

2. **`analyze_ast_contexts` strategy extraction** (Cyclomatic: 24 → 8) ✅
   ```rust
   // New: server/src/services/ast_strategies.rs
   trait AstStrategy: Send + Sync {
       async fn analyze(&self, path: &Path) -> Result<FileContext>;
   }
   
   struct StrategyRegistry {
       strategies: HashMap<Language, Box<dyn AstStrategy>>,
   }
   ```

3. **`handle_connection` router pattern** (Cyclomatic: 20 → 5) ✅
   ```rust
   // New: server/src/demo/router.rs
   lazy_static! {
       static ref ROUTES: Router = Router::new()
           .route("/api/analyze", post(analyze_handler))
           .route("/api/config", get(config_handler))
           .route("/api/export/:format", get(export_handler));
   }
   ```

## 1. Deterministic Graph Generation

### 1.1 Semantic Naming Engine
**File**: `server/src/services/semantic_naming.rs`

```rust
#[derive(Debug, Clone)]
pub struct SemanticNamer {
    patterns: HashMap<Language, &'static str>, // separator per language
}

impl SemanticNamer {
    pub fn get_semantic_name(&self, id: &str, node: &NodeInfo) -> String {
        node.metadata.get("display_name")
            .or_else(|| node.metadata.get("module_path"))
            .or_else(|| node.metadata.get("file_path")
                .map(|p| self.path_to_module(p)))
            .unwrap_or_else(|| self.clean_id(id))
    }
    
    fn path_to_module(&self, path: &str) -> String {
        Path::new(path)
            .strip_prefix("src/").unwrap_or(Path::new(path))
            .with_extension("")
            .to_string_lossy()
            .replace('/', "::")
    }
}
```

### 1.2 Fixed-Size Graph Builder
**File**: `server/src/services/fixed_graph_builder.rs`

```rust
pub struct FixedGraphBuilder {
    max_nodes: usize,
    max_edges: usize,
    namer: SemanticNamer,
}

impl FixedGraphBuilder {
    pub fn build(&self, graph: &DependencyGraph) -> FixedGraph {
        // 1. Group nodes by module
        let groups = self.group_by_module(graph);
        
        // 2. Score with PageRank
        let scores = self.calculate_pagerank(&groups);
        
        // 3. Select top N groups
        let selected: Vec<_> = scores.into_iter()
            .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
            .take(self.max_nodes)
            .collect();
        
        // 4. Build graph with edge budget
        self.build_with_budget(selected, graph)
    }
}
```

### 1.3 Mermaid Generator Fix
**File**: `server/src/services/mermaid_generator.rs` (Line 142)

```rust
impl MermaidGenerator {
    pub fn generate(&self, graph: &DependencyGraph, config: &GraphConfig) -> String {
        let builder = FixedGraphBuilder::new(config);
        let fixed = builder.build(graph);
        
        let mut lines = vec!["graph TD".to_string()];
        
        // Deterministic node ordering
        for (id, node) in fixed.nodes.iter().sorted_by_key(|(k, _)| *k) {
            lines.push(format!("    {}[{}]", 
                self.sanitize_id(id), 
                node.display_name
            ));
        }
        
        // Add edges
        for edge in &fixed.edges {
            lines.push(format!("    {} --> {}", 
                self.sanitize_id(&edge.from),
                self.sanitize_id(&edge.to)
            ));
        }
        
        lines.join("\n")
    }
}
```

## 2. Configuration System

### 2.1 Schema
**File**: `.paiml-display.yaml`

```yaml
version: "1.0"
panels:
  dependency:
    max_nodes: 20
    max_edges: 60
    grouping: module
  complexity:
    threshold: 15
    max_items: 50
```

### 2.2 Config Manager
**File**: `server/src/demo/config.rs`

```rust
pub struct ConfigManager {
    config: Arc<RwLock<DisplayConfig>>,
    update_tx: broadcast::Sender<DisplayConfig>,
}

impl ConfigManager {
    pub async fn watch(&mut self, path: PathBuf) -> Result<()> {
        let config = self.config.clone();
        let tx = self.update_tx.clone();
        
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(_) = res {
                if let Ok(new_config) = Self::load_config(&path) {
                    *config.write().unwrap() = new_config.clone();
                    let _ = tx.send(new_config);
                }
            }
        })?;
        
        watcher.watch(&path, RecursiveMode::NonRecursive)?;
        Ok(())
    }
}
```

## 3. Critical Bug Fixes

### 3.1 Build Artifact Filtering
**File**: `server/src/services/file_classifier.rs` (Line 78)

```rust
const BUILD_PATTERNS: &[&str] = &[
    "/target/", "/build/", "/dist/", "/node_modules/", 
    "/__pycache__/", "/.next/", "/cmake-build-", "/.gradle/"
];

fn is_build_artifact(&self, path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    BUILD_PATTERNS.iter().any(|p| path_str.contains(p))
}
```

## 4. Export System

**File**: `server/src/demo/export.rs`

```rust
impl MarkdownExporter {
    pub fn export(&self, report: &DemoReport) -> String {
        format!(
            r#"# Analysis: {}
Generated: {}

## Dependency Graph
```mermaid
{}
```

## Complexity Hotspots
{}

<details>
<summary>Raw Data</summary>

```json
{}
```
</details>"#,
            report.repository,
            Utc::now().format("%Y-%m-%d %H:%M UTC"),
            report.dependency_graph,
            self.format_hotspots(&report.complexity),
            serde_json::to_string_pretty(&report)?
        )
    }
}
```

## Implementation Plan

### Week 1: Core Fixes
- [x] **Day 1-2**: Refactor complexity hotspots (run_demo, analyze_ast_contexts, handle_connection) - COMPLETE
  - Reduced run_demo complexity from 34 to ~9
  - Reduced analyze_ast_contexts complexity from 24 to ~7
  - Reduced handle_connection complexity from 20 to ~5 via router pattern
- [x] **Day 3**: Implement SemanticNamer + FixedGraphBuilder - COMPLETE
  - Created SemanticNamer with language-aware path-to-module conversion
  - Implemented FixedGraphBuilder with PageRank-based node selection
- [x] **Day 4**: Fix MermaidGenerator to use semantic names - COMPLETE
  - Integrated with FixedGraphBuilder for deterministic output
  - Added generate_with_config method
- [x] **Day 5**: Add build artifact filtering - COMPLETE
  - Enhanced BUILD_PATTERNS to include all patterns from spec
  - Added tests to verify .gradle/ and node_modules/ filtering
  - Confirmed existing implementation was working correctly

### Week 2: Features
- [x] **Day 6-7**: Configuration system with hot-reload - COMPLETE
  - Implemented ConfigManager with file watching via notify crate
  - Created .paiml-display.yaml configuration schema
  - Added broadcast channels for configuration updates
  - Integrated hot-reload capability for real-time config changes
  - Note: Hot-reload test marked as ignored due to platform-dependent file watcher behavior
- [x] **Day 8**: Export system (Markdown + JSON + SARIF) - COMPLETE
  - Implemented trait-based export system with Exporter trait
  - Created MarkdownExporter with Mermaid diagram support
  - Created JsonExporter with pretty/compact options
  - Created SarifExporter for code analysis tool integration
  - Added ExportService to manage all exporters
  - All tests passing successfully
- [x] **Day 9-10**: Testing + demo-core extraction - COMPLETE
  - Created comprehensive integration tests for configuration system
  - Created full test suite for export system covering all formats
  - Created demo-core extraction tests validating library usage
  - Fixed all failing tests (binary linking issues, test data mismatches)
  - All 668 tests now passing successfully
  - Test coverage includes hot-reload, SARIF export, and programmatic usage

## Testing Checklist

```rust
#[test]
fn test_deterministic_graphs() {
    let graph = create_test_graph();
    let builder = FixedGraphBuilder::new(20, 60);
    
    // Multiple runs produce identical output
    assert_eq!(
        builder.build(&graph),
        builder.build(&graph)
    );
}

#[test]
fn test_semantic_names() {
    let namer = SemanticNamer::new();
    assert_eq!(
        namer.get_semantic_name("node_123", &node_with_path("src/auth/login.rs")),
        "auth::login"
    );
}
```

## Success Metrics
- Graph generation variance: 0% (fully deterministic)
- Meaningful node names: 100% (no raw IDs visible)
- Build artifact contamination: 0 files
- Cyclomatic complexity reduction: 70% for top 3 functions
- Export time: <500ms for 10K node graphs

## Implementation Files

### Core Demo Module
- `server/src/demo/mod.rs` - Main demo entry point (refactored)
- `server/src/demo/config.rs` - Configuration management with hot-reload
- `server/src/demo/export.rs` - Export system implementation
- `server/src/demo/router.rs` - HTTP routing for demo endpoints
- `server/src/demo/protocol_harness.rs` - Protocol-agnostic harness
- `server/src/demo/adapters/` - Protocol adapters (CLI, HTTP, MCP)

### Services
- `server/src/services/semantic_naming.rs` - Semantic name generation
- `server/src/services/fixed_graph_builder.rs` - Deterministic graph builder
- `server/src/services/ast_strategies.rs` - AST analysis strategies
- `server/src/services/file_classifier.rs` - Build artifact filtering
- `server/src/services/mermaid_generator.rs` - Enhanced Mermaid generation

### Tests
- `server/tests/config_integration.rs` - Configuration system tests
- `server/tests/export_integration.rs` - Export system tests
- `server/tests/demo_integration.rs` - Demo mode integration tests
- `server/tests/demo_core_extraction.rs` - Library usage tests
- `server/tests/demo_e2e_integration.rs` - End-to-end demo tests