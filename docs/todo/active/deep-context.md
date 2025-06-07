```markdown
# Deep Context Generation Specification

## Abstract

This specification defines the `analyze deep-context` command for the PAIML MCP Agent Toolkit, which generates comprehensive project analysis by orchestrating existing AST, complexity, churn, and dependency analysis subsystems. The command produces deterministic, cache-optimized output suitable for AI agent consumption and human review.

## Command Syntax

### CLI Interface

```bash
paiml-mcp-agent-toolkit analyze deep-context [OPTIONS]
```

#### Options

```
--project-path <PATH>         Project root directory (default: current directory)
--output-file <PATH>         Output file path (default: ./deep-context)
--format <FORMAT>            Output format: md|json|sarif (default: md)
--include <ANALYSES>         Comma-separated list of analyses to include
                            Options: ast,complexity,churn,dag,all (default: all)
--exclude <ANALYSES>         Comma-separated list of analyses to exclude
--period-days <DAYS>         Period for churn analysis (default: 30)
--dag-type <TYPE>           DAG type: call-graph|import-graph|inheritance|full-dependency
                            (default: call-graph)
--complexity-thresholds      Custom complexity thresholds as JSON
--max-depth <N>              Maximum directory traversal depth (default: 10)
--include-patterns <GLOB>    Include file patterns (can be specified multiple times)
--exclude-patterns <GLOB>    Exclude file patterns (default: standard ignores)
--cache-strategy <STRATEGY>  Cache usage: normal|force-refresh|offline (default: normal)
--parallel <N>               Parallelism level for analysis (default: num_cpus)
--verbose                    Enable verbose logging
```

### MCP Protocol Interface

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_deep_context",
    "arguments": {
      "project_path": "/path/to/project",
      "format": "json",
      "include_analyses": ["ast", "complexity", "churn", "dag"],
      "period_days": 30,
      "dag_type": "call-graph",
      "complexity_thresholds": {
        "max_cyclomatic": 15,
        "max_cognitive": 20
      }
    }
  }
}
```

## Architecture

### Data Flow Pipeline

```rust
pub struct DeepContextPipeline {
    // Phase 1: Project Discovery
    file_scanner: FileScanner,           // Parallel directory traversal
    
    // Phase 2: Cached AST Analysis  
    ast_analyzer: Arc<dyn AstCacheManager>, // Persistent cross-session cache
    
    // Phase 3: Metric Generation (parallel)
    complexity_analyzer: ComplexityAnalyzer,
    churn_analyzer: GitAnalysisService,
    dag_builder: DagBuilder,
    
    // Phase 4: Aggregation
    context_aggregator: ContextAggregator,
    
    // Phase 5: Rendering
    renderer: Box<dyn ContextRenderer>,
}
```

### Execution Phases

1. **Discovery Phase** (< 50ms for 10K files)
    - Parallel directory traversal using `walkdir` with custom ignore rules
    - File type detection via extension mapping
    - Template provenance detection (check for `.paiml-scaffold.json` markers)

2. **AST Extraction Phase** (< 10ms/file cached, < 100ms/file uncached)
    - Leverage existing `SessionCacheManager` with 5-minute TTL
    - Unified AST representation (`UnifiedAstNode`) for cross-language analysis
    - Parallel parsing with work-stealing queue (Tokio bounded channel)

3. **Analysis Phase** (parallel execution)
    - **Complexity**: Zero-overhead via AST visitor pattern
    - **Churn**: Git log parsing with commit-graph optimization
    - **DAG**: Vectorized dependency resolution with SIMD operations
    - **Tree**: Streaming generation with depth-first traversal

4. **Aggregation Phase**
    - Deterministic ordering by file path (lexicographic sort)
    - Cross-reference resolution for unified symbols
    - Quality score calculation (weighted composite metric)

5. **Rendering Phase**
    - Format-specific renderers implementing `ContextRenderer` trait
    - Streaming output for large projects (> 100MB contexts)

## Data Structures

### Core Types

```rust
pub struct DeepContext {
    pub metadata: ContextMetadata,
    pub file_tree: FileTree,
    pub analyses: AnalysisResults,
    pub quality_scorecard: QualityScorecard,
    pub template_provenance: Option<TemplateProvenance>,
}

pub struct ContextMetadata {
    pub generated_at: DateTime<Utc>,
    pub tool_version: Version,
    pub project_root: PathBuf,
    pub cache_stats: CacheStats,
    pub analysis_duration: Duration,
}

pub struct AnalysisResults {
    pub ast_contexts: Vec<FileContext>,        // From existing context analyzer
    pub complexity_report: ComplexityReport,   // From complexity analyzer
    pub churn_analysis: CodeChurnAnalysis,     // From git analyzer
    pub dependency_graph: DependencyGraph,     // From DAG builder
    pub cross_language_refs: CrossLangReferenceGraph,
}

pub struct QualityScorecard {
    pub overall_health: f64,              // 0.0-1.0 composite score
    pub complexity_score: f64,            // Inverse of avg complexity
    pub maintainability_index: f64,       // Based on churn vs complexity
    pub modularity_score: f64,            // From DAG clustering coefficient
    pub test_coverage: Option<f64>,       // If coverage data available
    pub technical_debt_hours: f64,        // Estimated from violations
}

pub struct TemplateProvenance {
    pub scaffold_timestamp: DateTime<Utc>,
    pub templates_used: Vec<String>,
    pub parameters: HashMap<String, Value>,
    pub drift_analysis: DriftAnalysis,
}
```

### Enhanced AST Information

Building on the existing `AstItem` enum, the deep context includes:

```rust
pub struct EnhancedAstItem {
    pub base: AstItem,                    // Existing AST item
    pub complexity_metrics: ComplexityMetrics,
    pub churn_metrics: Option<FileChurnMetrics>,
    pub dependencies: Vec<DependencyRef>,
    pub symbol_id: SymbolId,              // For cross-language references
}
```

## Output Formats

### Markdown Format

The markdown output follows the structure demonstrated in the example, with enhancements:

```markdown
# Deep Context: [project_name]

Generated: [ISO-8601 timestamp]
Tool Version: [version]
Cache Hit Rate: [percentage]
Analysis Time: [duration]

## Executive Summary

**Overall Health Score**: [score]/100
- Complexity: [score] (Avg McCabe: [n], Avg Cognitive: [n])
- Maintainability: [score] (Churn/Complexity ratio: [n])
- Modularity: [score] (Clustering coefficient: [n])
- Technical Debt: [hours] hours estimated

## Project Structure

[File tree with template provenance markers]

## Code Analysis

### AST Summary ([n] files analyzed)
[Language-grouped AST summaries with cross-references]

### Complexity Hotspots
[Top 10 complex functions with metrics]

### Churn Hotspots
[Top 10 frequently changed files]

### Dependency Graph
[Mermaid diagram with complexity coloring]

### Cross-Language References
[Rust→TS→Python call chains detected]

## Template Drift Analysis
[If scaffold metadata found, show deviations]
```

### JSON Format

```json
{
  "version": "1.0.0",
  "metadata": { ... },
  "fileTree": { ... },
  "analyses": {
    "ast": [ ... ],
    "complexity": { ... },
    "churn": { ... },
    "dag": { ... }
  },
  "qualityScorecard": { ... },
  "templateProvenance": { ... }
}
```

### SARIF Format

Extends existing SARIF output with deep context annotations:

```json
{
  "version": "2.1.0",
  "runs": [{
    "tool": {
      "driver": {
        "name": "paiml-mcp-agent-toolkit",
        "version": "[version]",
        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
      }
    },
    "results": [ ... ],
    "properties": {
      "deepContext": { ... }
    }
  }]
}
```

## Performance Characteristics

### Cache Optimization

1. **Multi-tier caching strategy**:
    - L1: In-memory LRU (session cache)
    - L2: Persistent file cache (5-minute TTL)
    - L3: Git object cache (commit-keyed)

2. **Cache key design**:
   ```rust
   format!("deep-context:{}:{}:{:x}", 
           project_hash,
           analysis_version,
           config_hash)
   ```

3. **Incremental updates**:
    - Track file modification times
    - Reuse unchanged AST analyses
    - Differential git log processing

### Parallelization Strategy

```rust
pub async fn analyze_parallel(files: Vec<PathBuf>) -> Result<Vec<FileContext>> {
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));
    let (tx, rx) = mpsc::channel(1000);  // Bounded for backpressure
    
    // Producer tasks
    let producers = files.into_iter().map(|file| {
        let sem = semaphore.clone();
        let tx = tx.clone();
        
        tokio::spawn(async move {
            let _permit = sem.acquire().await?;
            let context = analyze_file_with_cache(file).await?;
            tx.send(context).await?;
            Ok::<_, Error>(())
        })
    });
    
    // Streaming aggregation
    let aggregator = tokio::spawn(async move {
        let mut contexts = Vec::new();
        while let Some(ctx) = rx.recv().await {
            contexts.push(ctx);
        }
        contexts.sort_by(|a, b| a.path.cmp(&b.path));  // Deterministic
        contexts
    });
    
    futures::future::try_join_all(producers).await?;
    drop(tx);  // Signal completion
    
    aggregator.await?
}
```

### Memory Management

- **Streaming file processing**: Never load entire file tree into memory
- **Bounded channels**: Prevent memory exhaustion on large repos
- **Zero-copy AST visitor**: Process without cloning nodes
- **Mmap for large files**: Memory-mapped I/O for files > 10MB

## Integration Points

### With Existing Commands

1. **Reuses `context` command's AST infrastructure**
2. **Invokes `analyze complexity` internally**
3. **Calls `analyze churn` with configured period**
4. **Executes `analyze dag` for dependency graphs**

### With Template System

When analyzing scaffolded projects:

1. **Detect `.paiml-scaffold.json`** in project root
2. **Compare current files with original templates**
3. **Calculate drift metrics**:
    - Added files not in scaffold
    - Modified template files (diff size)
    - Deleted template files
    - Parameter evolution

### With MCP Protocol

New tool registration:

```rust
Tool {
    name: "analyze_deep_context",
    description: "Generate comprehensive project analysis combining AST, \
                 complexity, churn, and dependency insights",
    input_schema: json!({
        "type": "object",
        "properties": {
            "project_path": { "type": "string" },
            "format": { 
                "type": "string", 
                "enum": ["md", "json", "sarif"] 
            },
            // ... other properties
        }
    })
}
```

## Quality Metrics Calculation

### Overall Health Score (0-100)

```rust
pub fn calculate_health_score(ctx: &DeepContext) -> f64 {
    let weights = HealthWeights {
        complexity: 0.3,
        maintainability: 0.3,
        modularity: 0.2,
        coverage: 0.2,
    };
    
    let complexity_score = 100.0 * (1.0 / (1.0 + avg_complexity));
    let maintainability_score = 100.0 * (1.0 / (1.0 + churn_complexity_ratio));
    let modularity_score = 100.0 * clustering_coefficient;
    let coverage_score = coverage.unwrap_or(50.0);  // Assume 50% if unknown
    
    weights.complexity * complexity_score +
    weights.maintainability * maintainability_score +
    weights.modularity * modularity_score +
    weights.coverage * coverage_score
}
```

## Error Handling

1. **Graceful degradation**: Missing analyses don't fail entire context
2. **Partial results**: Return what succeeded with error annotations
3. **Cache corruption recovery**: Auto-invalidate corrupted cache entries
4. **Git-less projects**: Skip churn analysis with warning

## Security Considerations

1. **Path traversal prevention**: Canonicalize all paths
2. **Resource limits**: Max file size (100MB), max files (100K)
3. **Timeout protection**: 5-minute hard limit per analysis
4. **Memory limits**: OOM killer integration via cgroups

## Future Extensions

1. **Incremental deep context**: Stream updates via WebSocket
2. **Distributed analysis**: Shard across multiple machines
3. **AI-optimized format**: Token-efficient encoding for LLMs
4. **IDE integration**: Live deep context in VS Code sidebar
```