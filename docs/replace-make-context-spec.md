# Replace Make Context Implementation Specification

## Overview

Replace the current TypeScript-based `make context` implementation with our native Rust binary's `analyze deep-context` command, providing superior performance, richer metrics, and multi-protocol support.

## Current Implementation Analysis

### Existing TypeScript Script (`scripts/deep-context.ts`)
```typescript
// Current limitations:
- Single-threaded AST parsing
- Markdown-only output
- No caching mechanism
- Limited metrics (just AST structure)
- ~2-3s execution time for medium projects
```

### Current Makefile Target
```makefile
context:
	@echo "📊 Generating deep context analysis..."
	@$(SCRIPTS_DIR)/deep-context.ts
	@echo "✅ Deep context analysis complete! See deep_context.md"
```

## New Implementation

### 1. Replace Makefile Target

```makefile
context: server-build-binary
	@echo "📊 Generating deep context analysis (dogfooding our own tool)..."
	@./target/release/paiml-mcp-agent-toolkit analyze deep-context \
		--project-path . \
		--include "ast,complexity,churn,satd,dead-code" \
		--format markdown \
		--output deep_context.md \
		--cache-strategy normal
	@echo "✅ Deep context analysis complete! See deep_context.md"

# Additional targets for different formats
context-json: server-build-binary
	@./target/release/paiml-mcp-agent-toolkit analyze deep-context \
		--project-path . \
		--include "all" \
		--format json \
		--output deep_context.json

context-sarif: server-build-binary
	@./target/release/paiml-mcp-agent-toolkit analyze deep-context \
		--project-path . \
		--include "complexity,satd,dead-code" \
		--format sarif \
		--output deep_context.sarif
```

### 2. Enhanced Markdown Format

```rust
impl DeepContext {
    pub fn format_as_comprehensive_markdown(&self) -> String {
        let mut output = String::new();
        
        // Header with enhanced metadata
        writeln!(&mut output, "# Deep Context: {}", self.metadata.project_name).unwrap();
        writeln!(&mut output, "Generated: {}", self.metadata.generated_at).unwrap();
        writeln!(&mut output, "Version: {}", env!("CARGO_PKG_VERSION")).unwrap();
        writeln!(&mut output, "Analysis Time: {:.2}s", self.metadata.analysis_duration_secs).unwrap();
        writeln!(&mut output, "Cache Hit Rate: {:.1}%", self.cache_stats.hit_rate * 100.0).unwrap();
        
        // Quality scorecard summary
        writeln!(&mut output, "\n## Quality Scorecard\n").unwrap();
        writeln!(&mut output, "- **Overall Health**: {} ({:.1}/100)", 
            self.quality_scorecard.overall_health_emoji(),
            self.quality_scorecard.overall_score
        ).unwrap();
        writeln!(&mut output, "- **Maintainability Index**: {:.1}", 
            self.quality_scorecard.maintainability_index
        ).unwrap();
        writeln!(&mut output, "- **Technical Debt**: {:.1} hours estimated", 
            self.quality_scorecard.technical_debt_hours
        ).unwrap();
        
        // Project structure with annotations
        writeln!(&mut output, "\n## Project Structure\n").unwrap();
        writeln!(&mut output, "```").unwrap();
        self.format_annotated_tree(&mut output, &self.tree);
        writeln!(&mut output, "```\n").unwrap();
        
        // Enhanced AST with complexity indicators
        if !self.analysis_results.ast_contexts.is_empty() {
            self.format_enhanced_ast_section(&mut output);
        }
        
        // Code quality metrics
        self.format_complexity_hotspots(&mut output);
        self.format_churn_analysis(&mut output);
        self.format_technical_debt(&mut output);
        self.format_dead_code_analysis(&mut output);
        
        // Cross-language references
        self.format_cross_references(&mut output);
        
        // Defect probability analysis
        self.format_defect_predictions(&mut output);
        
        // Actionable recommendations
        self.format_prioritized_recommendations(&mut output);
        
        output
    }
}
```

### 3. Multi-Protocol Support

```rust
// CLI mode (existing)
$ paiml-mcp-agent-toolkit analyze deep-context --format markdown

// HTTP API mode
POST /api/v1/analyze/deep-context
{
  "project_path": "./",
  "include": ["ast", "complexity", "churn"],
  "format": "json"
}

// MCP mode
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "analyze_deep_context",
    "arguments": {
      "project_path": "./",
      "include": ["all"],
      "format": "markdown"
    }
  }
}
```

### 4. Performance Improvements

```rust
// Parallel analysis with work stealing
pub async fn analyze_project(&self, path: &Path) -> Result<DeepContext> {
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));
    let mut join_set = JoinSet::new();
    
    // Spawn parallel tasks
    join_set.spawn(self.analyze_ast_parallel(path.to_path_buf(), semaphore.clone()));
    join_set.spawn(self.analyze_complexity_parallel(path.to_path_buf(), semaphore.clone()));
    join_set.spawn(self.analyze_churn_parallel(path.to_path_buf(), semaphore.clone()));
    join_set.spawn(self.analyze_satd_parallel(path.to_path_buf(), semaphore.clone()));
    join_set.spawn(self.analyze_dead_code_parallel(path.to_path_buf(), semaphore.clone()));
    
    // Collect results with progress reporting
    let mut results = ParallelAnalysisResults::default();
    while let Some(result) = join_set.join_next().await {
        match result? {
            AnalysisResult::Ast(ast) => results.ast = Some(ast),
            AnalysisResult::Complexity(c) => results.complexity = Some(c),
            // ... other variants
        }
        self.report_progress(&results);
    }
    
    // Merge and correlate results
    self.merge_analysis_results(results)
}
```

### 5. Enhanced Output Formats

#### JSON Format
```json
{
  "metadata": {
    "project_name": "paiml-mcp-agent-toolkit",
    "generated_at": "2025-06-01T17:30:00Z",
    "version": "0.3.0",
    "analysis_duration_secs": 1.23,
    "cache_hit_rate": 0.87
  },
  "quality_scorecard": {
    "overall_score": 85.3,
    "maintainability_index": 78.2,
    "technical_debt_hours": 158.0,
    "test_coverage": 0.85
  },
  "metrics": {
    "total_files": 159,
    "total_lines": 45678,
    "complexity": {
      "cyclomatic_avg": 3.2,
      "cognitive_avg": 5.1,
      "hotspots": [...]
    },
    "churn": {
      "high_churn_files": 12,
      "avg_commits_per_file": 23.4
    },
    "technical_debt": {
      "total_items": 89,
      "critical": 3,
      "by_category": {...}
    }
  },
  "recommendations": [
    {
      "priority": "HIGH",
      "impact": "MAJOR",
      "type": "refactoring",
      "description": "Extract complex logic from context.rs",
      "effort_hours": 4.0,
      "files": ["server/src/services/context.rs"]
    }
  ]
}
```

#### SARIF Format
```json
{
  "version": "2.1.0",
  "runs": [{
    "tool": {
      "driver": {
        "name": "paiml-mcp-agent-toolkit",
        "version": "0.3.0",
        "rules": [
          {
            "id": "complexity/high-cyclomatic",
            "shortDescription": {"text": "High cyclomatic complexity"},
            "defaultConfiguration": {"level": "warning"}
          }
        ]
      }
    },
    "results": [
      {
        "ruleId": "complexity/high-cyclomatic",
        "level": "warning",
        "message": {"text": "Function has cyclomatic complexity of 32"},
        "locations": [{
          "physicalLocation": {
            "artifactLocation": {"uri": "server/src/services/context.rs"},
            "region": {"startLine": 234, "startColumn": 5}
          }
        }]
      }
    ]
  }]
}
```

### 6. Incremental Analysis

```rust
// Cache-aware incremental updates
pub struct IncrementalAnalyzer {
    cache: Arc<PersistentCacheManager>,
    dirty_tracker: Arc<RwLock<HashSet<PathBuf>>>,
    
    pub fn analyze_incremental(&self, path: &Path) -> Result<DeepContextDelta> {
        let last_analysis = self.cache.get::<DeepContext>("last_full_analysis")?;
        let dirty_files = self.detect_changes_since(&last_analysis.metadata.generated_at)?;
        
        // Only reanalyze changed files
        let delta = self.analyze_files(&dirty_files).await?;
        
        // Merge with cached results
        let updated = self.merge_with_cached(last_analysis, delta)?;
        self.cache.put("last_full_analysis", &updated)?;
        
        Ok(updated)
    }
}
```

### 7. Migration Path

```makefile
# Temporary parallel implementation
context-ts:
	@$(SCRIPTS_DIR)/deep-context.ts

context-rust: server-build-binary
	@./target/release/paiml-mcp-agent-toolkit analyze deep-context \
		--project-path . --format markdown --output deep_context_rust.md

context-compare: context-ts context-rust
	@echo "Comparing outputs..."
	@diff -u deep_context.md deep_context_rust.md || true
	@echo ""
	@echo "Performance comparison:"
	@time -p make context-ts 2>&1 | grep real
	@time -p make context-rust 2>&1 | grep real
```

## Implementation Checklist

- [ ] Extend `DeepContextAnalyzer` to match TypeScript output format
- [ ] Add file tree rendering with proper formatting
- [ ] Implement Makefile parsing in Rust
- [ ] Add README aggregation support
- [ ] Create migration tests comparing outputs
- [ ] Update CI to use new implementation
- [ ] Remove TypeScript dependency after validation
- [ ] Document new features in CLI reference

## Performance Targets

| Metric | Current (TS) | Target (Rust) | Improvement |
|--------|--------------|---------------|-------------|
| Cold start | 2-3s | <500ms | 5-6x |
| With cache | N/A | <100ms | N/A |
| Memory usage | ~150MB | <50MB | 3x |
| Parallelism | 1 thread | N cores | Nx |
| Output formats | 1 | 4+ | 4x |

## Unique Advantages

1. **Dogfooding**: Using our own tool validates completeness
2. **Rich Metrics**: Complexity, churn, SATD, dead code in one pass
3. **Caching**: Persistent cache for incremental analysis
4. **Multi-Format**: Markdown, JSON, SARIF, potentially GraphML
5. **Protocol Support**: Same analysis via CLI, HTTP, or MCP
6. **Performance**: Rust performance with parallel analysis
7. **Extensibility**: Plugin architecture for custom analyzers

## Success Criteria

1. Output contains all information from current implementation
2. Performance improvement of at least 5x
3. Zero external dependencies (no Deno/Node required)
4. Supports at least 3 output formats
5. Incremental analysis completes in <100ms
6. All three protocols produce identical analysis results