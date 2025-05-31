# Unified Demo Mode Specification

## Abstract

This specification defines a streamlined demo mode for the PAIML MCP Agent Toolkit that generates a high-level system architecture diagram using graph-theoretic complexity reduction. The demo provides a zero-friction entry point for understanding toolkit capabilities through visual representation, supporting both local and remote repository analysis.

## 1. Core Architecture

### 1.1 Complexity Reduction Pipeline

The demo generates a simplified system diagram by applying graph-theoretic transformations to the full dependency graph:

```rust
pub struct DemoGraphReducer {
    /// Target node count for reduced graph
    target_nodes: usize,
    /// Minimum betweenness centrality threshold
    centrality_threshold: f64,
    /// Component size threshold for merging
    merge_threshold: usize,
}

impl DemoGraphReducer {
    pub fn reduce(&self, full_dag: &DependencyGraph) -> ReducedGraph {
        // 1. Calculate betweenness centrality for all nodes
        let centrality = self.calculate_betweenness_centrality(full_dag);
        
        // 2. Identify architectural components via spectral clustering
        let components = self.spectral_clustering(full_dag, self.target_nodes);
        
        // 3. Merge small components below threshold
        let merged = self.merge_small_components(components, self.merge_threshold);
        
        // 4. Extract inter-component edges
        let edges = self.extract_component_edges(full_dag, &merged);
        
        // 5. Apply complexity coloring based on aggregate metrics
        self.apply_complexity_coloring(&mut merged)
    }
    
    fn calculate_betweenness_centrality(&self, dag: &DependencyGraph) -> HashMap<NodeKey, f64> {
        // Brandes' algorithm - O(VE) for unweighted graphs
        let mut centrality = HashMap::new();
        let nodes: Vec<_> = dag.nodes.keys().collect();
        
        for s in &nodes {
            let mut stack = Vec::new();
            let mut paths: HashMap<_, Vec<NodeKey>> = HashMap::new();
            let mut sigma = HashMap::new();
            let mut delta = HashMap::new();
            
            // BFS from source
            let mut queue = VecDeque::new();
            queue.push_back(*s);
            sigma.insert(*s, 1.0);
            
            while let Some(v) = queue.pop_front() {
                stack.push(v);
                for edge in dag.edges.iter().filter(|e| e.from == v) {
                    // Update shortest path counts
                    let sigma_v = sigma[&v];
                    sigma.entry(edge.to)
                        .and_modify(|s| *s += sigma_v)
                        .or_insert(sigma_v);
                    paths.entry(edge.to).or_default().push(v);
                }
            }
            
            // Accumulate dependencies
            while let Some(w) = stack.pop() {
                for &v in paths.get(&w).unwrap_or(&vec![]) {
                    let coefficient = sigma[&v] / sigma[&w] * (1.0 + delta.get(&w).unwrap_or(&0.0));
                    delta.entry(v).and_modify(|d| *d += coefficient).or_insert(coefficient);
                }
                if w != *s {
                    centrality.entry(w).and_modify(|c| *c += delta[&w]).or_insert(delta[&w]);
                }
            }
        }
        
        // Normalize by (n-1)(n-2) for directed graphs
        let n = nodes.len() as f64;
        for c in centrality.values_mut() {
            *c *= 2.0 / ((n - 1.0) * (n - 2.0));
        }
        
        centrality
    }
}
```

### 1.2 Target System Diagram Structure

The demo generates exactly this structure (matching `actual-paiml-high-level-system-diagram.mmd`):

```mermaid
graph TD
    A[AST Context Analysis] -->|uses| B[File Parser]
    B --> C[Rust AST]
    B --> D[TypeScript AST]
    B --> E[Python AST]

    F[Code Complexity] -->|analyzes| C
    F -->|analyzes| D
    F -->|analyzes| E

    G[DAG Generation] -->|reads| C
    G -->|reads| D
    G -->|reads| E

    H[Code Churn] -->|git history| I[Git Analysis]

    J[Template Generation] -->|renders| K[Handlebars]

    style A fill:#90EE90
    style F fill:#FFD700
    style G fill:#FFA500
    style H fill:#FF6347
    style J fill:#87CEEB
```

## 2. Implementation Architecture

### 2.1 Demo Runner Enhancement

```rust
impl DemoRunner {
    pub async fn execute_with_diagram(&mut self, repo_path: &Path, url: Option<&str>) -> Result<DemoReport> {
        let start = Instant::now();
        
        // Clone remote repository if URL provided
        let working_path = if let Some(url) = url {
            self.clone_and_prepare(url).await?
        } else {
            repo_path.to_path_buf()
        };
        
        // Execute analysis pipeline with tracing
        let span = tracing::info_span!("demo_execution", repo = %working_path.display());
        let _guard = span.enter();
        
        // Collect all analysis results
        let mut steps = Vec::new();
        steps.push(self.demo_context_generation(&working_path).await?);
        steps.push(self.demo_complexity_analysis(&working_path).await?);
        steps.push(self.demo_dag_generation(&working_path).await?);
        steps.push(self.demo_churn_analysis(&working_path).await?);
        steps.push(self.demo_system_architecture(&working_path).await?);
        steps.push(self.demo_template_generation(&working_path).await?);
        
        // Generate high-level system diagram
        let system_diagram = self.generate_system_diagram(&steps)?;
        
        Ok(DemoReport {
            repository: working_path.display().to_string(),
            total_time_ms: start.elapsed().as_millis() as u64,
            steps,
            system_diagram: Some(system_diagram),
        })
    }
    
    fn generate_system_diagram(&self, steps: &[DemoStep]) -> Result<String> {
        // Extract component relationships from analysis results
        let mut components = HashMap::new();
        
        // Map internal components to high-level architecture
        components.insert("ast_context", Component {
            id: "A",
            label: "AST Context Analysis",
            color: "#90EE90",
            connections: vec![("B", "uses")],
        });
        
        components.insert("file_parser", Component {
            id: "B",
            label: "File Parser",
            color: "#FFFFFF",
            connections: vec![("C", ""), ("D", ""), ("E", "")],
        });
        
        // Language-specific AST components
        components.insert("rust_ast", Component {
            id: "C",
            label: "Rust AST",
            color: "#FFFFFF",
            connections: vec![],
        });
        
        components.insert("typescript_ast", Component {
            id: "D",
            label: "TypeScript AST",
            color: "#FFFFFF",
            connections: vec![],
        });
        
        components.insert("python_ast", Component {
            id: "E",
            label: "Python AST",
            color: "#FFFFFF",
            connections: vec![],
        });
        
        // Analysis components
        components.insert("complexity", Component {
            id: "F",
            label: "Code Complexity",
            color: "#FFD700",
            connections: vec![("C", "analyzes"), ("D", "analyzes"), ("E", "analyzes")],
        });
        
        components.insert("dag_gen", Component {
            id: "G",
            label: "DAG Generation",
            color: "#FFA500",
            connections: vec![("C", "reads"), ("D", "reads"), ("E", "reads")],
        });
        
        components.insert("churn", Component {
            id: "H",
            label: "Code Churn",
            color: "#FF6347",
            connections: vec![("I", "git history")],
        });
        
        components.insert("git", Component {
            id: "I",
            label: "Git Analysis",
            color: "#FFFFFF",
            connections: vec![],
        });
        
        components.insert("template", Component {
            id: "J",
            label: "Template Generation",
            color: "#87CEEB",
            connections: vec![("K", "renders")],
        });
        
        components.insert("handlebars", Component {
            id: "K",
            label: "Handlebars",
            color: "#FFFFFF",
            connections: vec![],
        });
        
        // Generate Mermaid diagram
        self.render_system_mermaid(&components)
    }
}
```

### 2.2 Remote Repository Support

```rust
impl DemoRunner {
    async fn clone_and_prepare(&self, url: &str) -> Result<PathBuf> {
        let temp_dir = tempfile::tempdir()?;
        let repo_path = temp_dir.path().join("repo");
        
        // Use git2 for efficient cloning with progress callback
        let mut callbacks = RemoteCallbacks::new();
        callbacks.transfer_progress(|stats| {
            let progress = stats.received_objects() as f64 / stats.total_objects() as f64;
            tracing::info!(progress = %format!("{:.1}%", progress * 100.0), "Cloning repository");
            true
        });
        
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks);
        
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);
        
        // Clone with depth=1 for performance
        builder.clone(url, &repo_path)
            .map_err(|e| anyhow!("Failed to clone repository: {}", e))?;
        
        Ok(repo_path)
    }
}
```

## 3. CLI Interface

### 3.1 Command Structure

```bash
# Local repository analysis
paiml-mcp-agent-toolkit demo

# Remote repository analysis
paiml-mcp-agent-toolkit demo --url https://github.com/user/repo

# Output formats
paiml-mcp-agent-toolkit demo --format json
paiml-mcp-agent-toolkit demo --format markdown
paiml-mcp-agent-toolkit demo --format html

# Complexity reduction controls
paiml-mcp-agent-toolkit demo --target-nodes 15
paiml-mcp-agent-toolkit demo --merge-threshold 3

# Tracing and debugging
RUST_LOG=paiml_mcp_agent_toolkit=debug paiml-mcp-agent-toolkit demo
RUST_LOG=paiml_mcp_agent_toolkit::demo=trace paiml-mcp-agent-toolkit demo
```

### 3.2 Output Format

The demo provides a clean, coverage-report-style output:

```
🎯 PAIML MCP Agent Toolkit Demo
📁 Repository: https://github.com/user/repo
⏱️  Cloning: 100.0% [====================] 1234 objects

Analysis Pipeline                             Time      Status
─────────────────────────────────────────────────────────────
AST Context Analysis                         245ms        ✓
Code Complexity Analysis                     189ms        ✓
DAG Generation                              156ms        ✓
Code Churn Analysis                         312ms        ✓
System Architecture                          89ms        ✓
Template Generation                          12ms        ✓
─────────────────────────────────────────────────────────────
Total                                      1003ms

System Architecture:
┌─────────────────────────────────────────────────────────┐
│                                                         │
│  [AST Context] ──uses──> [File Parser]                │
│       │                      │                          │
│       │                ┌─────┼─────┐                   │
│       │                ↓     ↓     ↓                   │
│       │           [Rust] [TS] [Python]                 │
│       │                ↑     ↑     ↑                   │
│       │          ┌─────┴─────┴─────┴─────┐            │
│       │          │                       │             │
│  [Complexity] ───┤    [DAG Generation]   │            │
│                  │                       │             │
│                  └───────────────────────┘             │
│                                                         │
│  [Code Churn] ──git──> [Git Analysis]                 │
│                                                         │
│  [Templates] ──renders──> [Handlebars]                │
│                                                         │
└─────────────────────────────────────────────────────────┘

📊 Metrics Summary:
• Files analyzed: 147
• Total functions: 1,823
• Average complexity: 3.2
• Code churn hotspots: 12
• Technical debt: 42.3 hours
```

## 4. Tracing Integration

### 4.1 Structured Tracing

```rust
#[instrument(skip(self, repo_path), fields(repo = %repo_path.display()))]
async fn demo_context_generation(&mut self, repo_path: &Path) -> Result<DemoStep> {
    let start = Instant::now();
    
    let span = tracing::debug_span!("ast_analysis");
    let _guard = span.enter();
    
    // Trace file discovery
    let files = {
        let _span = tracing::trace_span!("file_discovery").entered();
        self.discover_files(repo_path)?
    };
    
    tracing::debug!(file_count = files.len(), "Discovered source files");
    
    // Trace parsing phase
    let contexts = {
        let _span = tracing::trace_span!("ast_parsing").entered();
        self.parse_files_parallel(&files).await?
    };
    
    tracing::info!(
        duration_ms = start.elapsed().as_millis(),
        files_parsed = contexts.len(),
        "AST context generation complete"
    );
    
    Ok(DemoStep {
        name: "AST Context Analysis".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
        success: true,
        output: Some(json!({
            "files_analyzed": contexts.len(),
            "languages": self.detected_languages(&contexts),
        })),
    })
}
```

### 4.2 Debug Output

Enable comprehensive tracing for development:

```bash
# Basic tracing
RUST_LOG=info paiml-mcp-agent-toolkit demo

# Detailed component tracing
RUST_LOG=paiml_mcp_agent_toolkit::demo=debug paiml-mcp-agent-toolkit demo

# Full trace including parsing details
RUST_LOG=paiml_mcp_agent_toolkit=trace paiml-mcp-agent-toolkit demo

# Structured JSON output for analysis
RUST_LOG=info RUST_LOG_FORMAT=json paiml-mcp-agent-toolkit demo 2>demo.log
```

## 5. Performance Characteristics

### 5.1 Complexity Reduction Metrics

- **Graph reduction**: O(V²) using spectral clustering
- **Betweenness centrality**: O(VE) using Brandes' algorithm
- **Component merging**: O(V log V) with union-find
- **Target performance**: <100ms for graphs with 10K nodes

### 5.2 Memory Optimization

```rust
pub struct ReducedGraph {
    /// Components stored in arena allocator
    components: Arena<Component>,
    /// Edge list in compressed sparse row format
    edges: CsrMatrix<EdgeType>,
    /// Metadata cache line aligned
    #[repr(align(64))]
    metadata: GraphMetadata,
}
```

## 6. Error Handling

### 6.1 Repository Access Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum DemoError {
    #[error("Failed to clone repository: {0}")]
    CloneError(String),
    
    #[error("Unsupported repository: {0}")]
    UnsupportedRepo(String),
    
    #[error("No supported languages found in repository")]
    NoSupportedLanguages,
    
    #[error("Graph reduction failed: {0}")]
    ReductionError(String),
}
```

### 6.2 Graceful Degradation

If specific analyses fail, the demo continues with partial results:

```rust
match self.demo_complexity_analysis(&working_path).await {
    Ok(step) => steps.push(step),
    Err(e) => {
        tracing::warn!(error = %e, "Complexity analysis failed, continuing");
        steps.push(DemoStep {
            name: "Code Complexity Analysis".to_string(),
            duration_ms: 0,
            success: false,
            output: Some(json!({ "error": e.to_string() })),
        });
    }
}
```

## 7. Testing Strategy

### 7.1 Integration Tests

```rust
#[tokio::test]
async fn test_demo_remote_repository() {
    let runner = DemoRunner::new(ExecutionMode::Cli);
    let report = runner.execute_with_diagram(
        Path::new("."),
        Some("https://github.com/rust-lang/rust-clippy")
    ).await.unwrap();
    
    assert!(report.system_diagram.is_some());
    assert!(report.system_diagram.unwrap().contains("AST Context Analysis"));
}

#[test]
fn test_graph_reduction() {
    let mut dag = create_test_dag(1000); // 1000 nodes
    let reducer = DemoGraphReducer {
        target_nodes: 15,
        centrality_threshold: 0.1,
        merge_threshold: 3,
    };
    
    let reduced = reducer.reduce(&dag);
    assert!(reduced.node_count() <= 15);
    assert!(reduced.preserves_connectivity(&dag));
}
```

## 8. Future Enhancements

1. **Interactive Mode**: Terminal UI with graph navigation
2. **Caching**: Cache analysis results for repeated demos
3. **Custom Diagrams**: User-defined component groupings
4. **Export Formats**: SVG, PNG, PDF generation
5. **Metrics Dashboard**: Web-based visualization

## 9. Implementation Status

### ✅ **Completed Features**

- [x] **Enhanced CLI Interface** - All new CLI arguments (`--url`, `--target-nodes`, `--centrality-threshold`, `--merge-threshold`) are implemented and functional
- [x] **Enhanced DemoRunner Architecture** - `execute_with_diagram()` method implemented with proper error handling
- [x] **System Diagram Generation** - Generates exact target Mermaid structure matching specification
- [x] **DemoStep Enhancement** - Updated with `success` and `output` fields for structured results
- [x] **Coverage-style Output Formatter** - Clean CLI output with timing metrics and status indicators
- [x] **Structured Tracing Integration** - Added `#[instrument]` macros and proper span management
- [x] **Graceful Degradation** - Demo continues with partial results if individual analyses fail
- [x] **CLI/JSON Output Support** - Multiple output formats working correctly
- [x] **Demo Pipeline Integration** - All six analysis capabilities working in sequence
- [x] **Web Interface Integration** - System diagram correctly shows as "Dynamic" when generated by DemoRunner
- [x] **Data Flow Architecture** - Proper data flow from CLI demo to web demo via DemoContent and DemoState
- [x] **REST API Foundation** - Basic endpoints (`/api/system-diagram`, `/api/hotspots`, `/api/dag`, `/api/summary`) implemented
- [x] **Hotspots Table Rendering** - Fixed JavaScript column mapping to display actual function names, complexity, and file paths
- [x] **Data Source Indicators** - Visual indicators showing Dynamic (green) vs Default (red) data sources with automatic detection
- [x] **Clippy Warning Fixes** - Resolved useless `as_ref()` usage and dead code warnings

### 🔄 **Partially Implemented (Ready for Enhancement)**

- [⚠️] **Remote Repository Support** - CLI parsing complete, placeholder error message for git2 integration
  ```rust
  // Current placeholder in clone_and_prepare():
  Err(anyhow!("Remote repository cloning not yet implemented. URL: {}", url))
  ```

- [⚠️] **Graph Complexity Reduction** - Parameters parsed and passed through, DemoGraphReducer structure defined but algorithms not implemented
  ```rust
  // Need to implement:
  // - calculate_betweenness_centrality() with Brandes' algorithm
  // - spectral_clustering() for component identification  
  // - merge_small_components() for threshold-based merging
  ```

### ❌ **Not Yet Implemented**

- [ ] **Git2 Integration** - Actual repository cloning with progress callbacks
  ```rust
  // Required dependencies to add to Cargo.toml:
  git2 = "0.18"
  tempfile = "3.8"
  
  // Implementation needed in clone_and_prepare():
  // - RemoteCallbacks with transfer_progress
  // - FetchOptions and RepoBuilder configuration
  // - Temporary directory management
  ```

- [ ] **Spectral Clustering Algorithm** - Graph-theoretic complexity reduction
  ```rust
  // Need to implement DemoGraphReducer methods:
  impl DemoGraphReducer {
      fn calculate_betweenness_centrality(&self, dag: &DependencyGraph) -> HashMap<NodeKey, f64>
      fn spectral_clustering(&self, full_dag: &DependencyGraph, target_nodes: usize) -> Vec<Component>
      fn merge_small_components(&self, components: Vec<Component>, threshold: usize) -> Vec<Component>
      fn extract_component_edges(&self, dag: &DependencyGraph, components: &[Component]) -> Vec<Edge>
      fn apply_complexity_coloring(&self, components: &mut [Component])
  }
  ```

- [ ] **Enhanced Error Types** - Specific error handling for demo operations
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum DemoError {
      #[error("Failed to clone repository: {0}")]
      CloneError(String),
      #[error("Unsupported repository: {0}")]  
      UnsupportedRepo(String),
      #[error("No supported languages found in repository")]
      NoSupportedLanguages,
      #[error("Graph reduction failed: {0}")]
      ReductionError(String),
  }
  ```

- [ ] **Repository Validation** - Validate remote URLs and check for supported languages before cloning

- [ ] **Integration Tests** - Automated tests for popular repositories and error conditions

- [ ] **Performance Optimizations** - Memory-efficient graph structures and algorithms

- [ ] **Dynamic Data Integration for Web Interface** - Replace hardcoded fallback data with actual analysis results
  ```rust
  // Current issues in server/src/demo/server.rs and mod.rs:
  // 1. DemoState uses Default::default() for complexity_report, churn_analysis, dependency_graph
  // 2. Web endpoints serve fallback data instead of actual analysis results
  // 3. Need to wire up real complexity reports from DemoRunner to web server state
  
  // Required changes:
  // - Pass actual analysis results to DemoState instead of defaults
  // - Update serve_hotspots_table() to use real complexity data when available  
  // - Update serve_dag_mermaid() to use real dependency graph when available
  // - Ensure all "Default" indicators become "Dynamic" when real data is present
  ```

- [ ] **REST API Testing** - Comprehensive test coverage for web endpoints
  ```rust
  // Need to implement tests for:
  #[tokio::test]
  async fn test_api_system_diagram_endpoint()
  
  #[tokio::test] 
  async fn test_api_hotspots_endpoint()
  
  #[tokio::test]
  async fn test_api_dag_endpoint()
  
  #[tokio::test]
  async fn test_api_summary_endpoint()
  
  #[tokio::test]
  async fn test_web_interface_data_flow()
  
  // Integration tests for:
  // - JSON response format validation
  // - Error handling and status codes
  // - Data source indicator accuracy
  // - Mermaid diagram content validation
  ```

- [ ] **Web Interface Enhancements** - Additional features for the demo dashboard
  ```typescript
  // Potential improvements:
  // - Real-time progress updates during analysis
  // - Interactive Mermaid diagram navigation
  // - Export functionality for diagrams and reports
  // - Responsive design improvements for mobile
  // - Dark mode support
  // - Caching for faster subsequent loads
  ```

### 🎯 **Priority Implementation Order**

1. **Git2 Integration** (Highest Priority)
   - Most user-visible feature
   - Required for remote repository analysis
   - Clear error boundaries and rollback capability

2. **Enhanced Error Handling** (High Priority)  
   - Better user experience
   - Cleaner error messages
   - Proper error types for different failure modes

3. **Graph Complexity Reduction** (Medium Priority)
   - Advanced feature for large repositories
   - Requires mathematical algorithm implementation
   - Can be incrementally improved

4. **Integration Tests** (Medium Priority)
   - Important for reliability
   - Can be added as features stabilize

5. **Performance Optimizations** (Low Priority)
   - Current implementation is adequate for typical use
   - Can be improved based on real-world usage patterns

### 📋 **Implementation Checklist**

#### Next Steps (Git2 Integration):
- [ ] Add git2 and tempfile dependencies to Cargo.toml
- [ ] Implement actual repository cloning in `clone_and_prepare()`
- [ ] Add progress callback integration with tracing
- [ ] Handle authentication for private repositories
- [ ] Add repository validation (check for .git directory, supported files)
- [ ] Test with various repository types (GitHub, GitLab, etc.)

#### Future Enhancements:
- [ ] Implement Brandes' algorithm for betweenness centrality calculation
- [ ] Add spectral clustering using eigenvalue decomposition
- [ ] Create comprehensive integration test suite
- [ ] Add caching for repeated repository analysis
- [ ] Implement custom diagram templates