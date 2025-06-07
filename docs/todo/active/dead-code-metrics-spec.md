# Dead Code Metrics Specification

## Overview

This specification defines the implementation of dead code analysis metrics for the PAIML MCP Agent Toolkit, including a `--top-files` ranking system consistent with existing complexity analysis features.

## Goals

1. Expose the existing `DeadCodeAnalyzer` through the CLI interface
2. Provide actionable insights with file-level ranking
3. Support multiple output formats (summary, JSON, SARIF)
4. Integrate with dogfooding metrics in README
5. Add MCP tool support for dead code analysis

## CLI Interface

### Command Structure

```bash
paiml-mcp-agent-toolkit analyze dead-code [OPTIONS]
```

### Options

```rust
#[derive(Args, Debug)]
pub struct AnalyzeDeadCodeArgs {
    /// Path to analyze (defaults to current directory)
    #[arg(long, short = 'p')]
    path: Option<PathBuf>,
    
    /// Output format
    #[arg(long, short = 'f', value_enum, default_value = "summary")]
    format: DeadCodeOutputFormat,
    
    /// Show top N files with most dead code
    #[arg(long, short = 't')]
    top_files: Option<usize>,
    
    /// Include unreachable code blocks in analysis
    #[arg(long, short = 'u')]
    include_unreachable: bool,
    
    /// Minimum dead lines to report a file (default: 10)
    #[arg(long, default_value = "10")]
    min_dead_lines: usize,
    
    /// Include test files in analysis
    #[arg(long)]
    include_tests: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DeadCodeOutputFormat {
    Summary,
    Json,
    Sarif,
    Markdown,
}
```

## Data Models

### File-Level Metrics

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeadCodeMetrics {
    pub path: String,
    pub dead_lines: usize,
    pub total_lines: usize,
    pub dead_percentage: f32,
    pub dead_functions: usize,
    pub dead_classes: usize,
    pub dead_modules: usize,
    pub unreachable_blocks: usize,
    pub dead_score: f32,
    pub confidence: ConfidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,      // Definitely dead (no references)
    Medium,    // Possibly dead (only internal references)
    Low,       // Might be used (dynamic calls possible)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeRankingResult {
    pub summary: DeadCodeSummary,
    pub ranked_files: Vec<FileDeadCodeMetrics>,
    pub analysis_timestamp: DateTime<Utc>,
    pub config: DeadCodeAnalysisConfig,
}
```

### Scoring Algorithm

```rust
impl FileDeadCodeMetrics {
    pub fn calculate_score(&mut self) {
        // Weighted scoring similar to complexity ranker
        let percentage_weight = 0.35;
        let absolute_weight = 0.30;
        let function_weight = 0.20;
        let confidence_weight = 0.15;
        
        let confidence_multiplier = match self.confidence {
            ConfidenceLevel::High => 1.0,
            ConfidenceLevel::Medium => 0.7,
            ConfidenceLevel::Low => 0.4,
        };
        
        self.dead_score = 
            (self.dead_percentage * percentage_weight) +
            (self.dead_lines.min(1000) as f32 / 10.0 * absolute_weight) +
            (self.dead_functions.min(50) as f32 * 2.0 * function_weight) +
            (100.0 * confidence_multiplier * confidence_weight);
    }
}
```

## Implementation Details

### Integration with Existing DeadCodeAnalyzer

```rust
impl DeadCodeAnalyzer {
    pub async fn analyze_with_ranking(
        &self,
        project_path: &Path,
        config: DeadCodeAnalysisConfig,
    ) -> Result<DeadCodeRankingResult> {
        // 1. Build AST DAG for project
        let context = analyze_project(project_path).await?;
        let dag_builder = DagBuilder::new();
        let dag = dag_builder.build_from_project(&context);
        
        // 2. Convert to unified AST format
        let ast_dag = self.build_ast_dag(&context)?;
        
        // 3. Run dead code analysis
        let report = self.analyze(&ast_dag)?;
        
        // 4. Aggregate by file
        let mut file_metrics = self.aggregate_by_file(&report, &context)?;
        
        // 5. Calculate scores and sort
        for metrics in &mut file_metrics {
            metrics.calculate_score();
        }
        file_metrics.sort_by(|a, b| b.dead_score.partial_cmp(&a.dead_score).unwrap());
        
        // 6. Apply filters
        if !config.include_tests {
            file_metrics.retain(|f| !f.path.contains("test"));
        }
        file_metrics.retain(|f| f.dead_lines >= config.min_dead_lines);
        
        Ok(DeadCodeRankingResult {
            summary: report.summary,
            ranked_files: file_metrics,
            analysis_timestamp: Utc::now(),
            config,
        })
    }
}
```

### Handler Implementation

```rust
pub async fn handle_analyze_dead_code(
    args: &AnalyzeDeadCodeArgs,
    cache_manager: Arc<SessionCacheManager>,
) -> Result<()> {
    let project_path = args.path.as_deref().unwrap_or(Path::new("."));
    
    // Check cache
    let cache_key = format!("dead_code:{}:{}", project_path.display(), args.include_unreachable);
    if let Some(cached) = cache_manager.get_dead_code(&cache_key).await {
        return format_dead_code_output(cached, args);
    }
    
    // Run analysis
    let analyzer = DeadCodeAnalyzer::new()
        .with_coverage(None);  // TODO: Support coverage data
        
    let config = DeadCodeAnalysisConfig {
        include_unreachable: args.include_unreachable,
        include_tests: args.include_tests,
        min_dead_lines: args.min_dead_lines,
    };
    
    let mut result = analyzer.analyze_with_ranking(project_path, config).await?;
    
    // Apply top_files limit
    if let Some(limit) = args.top_files {
        result.ranked_files.truncate(limit);
    }
    
    // Cache result
    cache_manager.put_dead_code(cache_key, result.clone()).await;
    
    // Format output
    format_dead_code_output(result, args)
}
```

## Output Formats

### Summary Format

```
Dead Code Analysis Summary:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Total files analyzed: 146
  Files with dead code: 23 (15.8%)
  
  Total dead lines: 1,847 (3.2% of codebase)
  Dead functions: 142
  Dead classes: 18
  Dead modules: 3
  Unreachable blocks: 47

🏆 Top 5 Files with Most Dead Code:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. ./server/src/services/old_cache.rs (Score: 98.5) [HIGH confidence]
   └─ 285 dead lines (100.0% of file)
   └─ 12 functions, 2 classes
   └─ Recommendation: Safe to delete entire file

2. ./server/src/services/legacy_renderer.rs (Score: 72.3) [MEDIUM confidence]
   └─ 198 dead lines (45.2% of file)
   └─ 8 functions, 1 class
   └─ Recommendation: Review and remove unused sections

3. ./server/src/tests/deprecated_tests.rs (Score: 65.1) [HIGH confidence]
   └─ 156 dead lines (78.0% of file)
   └─ 15 functions
   └─ Recommendation: Archive or remove old tests
```

### JSON Format

```json
{
  "summary": {
    "total_files_analyzed": 146,
    "files_with_dead_code": 23,
    "total_dead_lines": 1847,
    "dead_percentage": 3.2,
    "dead_functions": 142,
    "dead_classes": 18,
    "dead_modules": 3,
    "unreachable_blocks": 47
  },
  "ranked_files": [
    {
      "path": "./server/src/services/old_cache.rs",
      "dead_lines": 285,
      "total_lines": 285,
      "dead_percentage": 100.0,
      "dead_functions": 12,
      "dead_classes": 2,
      "dead_modules": 0,
      "unreachable_blocks": 0,
      "dead_score": 98.5,
      "confidence": "High",
      "items": [
        {
          "type": "function",
          "name": "warm_cache",
          "line": 45,
          "reason": "No references found"
        }
      ]
    }
  ],
  "metadata": {
    "analysis_timestamp": "2025-05-31T14:23:45Z",
    "analyzer_version": "0.2.0",
    "config": {
      "include_unreachable": false,
      "include_tests": false,
      "min_dead_lines": 10
    }
  }
}
```

### SARIF Format

For IDE integration, supporting the SARIF 2.1.0 format with dead code as "code smell" issues.

## MCP Tool Integration

Add new MCP tool: `analyze_dead_code`

```rust
ToolInfo {
    name: "analyze_dead_code",
    description: "Analyze codebase for dead and unreachable code",
    input_schema: json!({
        "type": "object",
        "properties": {
            "project_path": {
                "type": "string",
                "description": "Path to analyze"
            },
            "top_files": {
                "type": "integer",
                "description": "Number of top files to return"
            },
            "include_unreachable": {
                "type": "boolean",
                "description": "Include unreachable code blocks"
            },
            "format": {
                "type": "string",
                "enum": ["summary", "json", "sarif"],
                "default": "summary"
            }
        }
    }),
}
```

## Dogfooding Integration

Update the `make dogfood` target to include dead code metrics:

```rust
// In dogfood command handler
let dead_code_result = analyze_dead_code(&AnalyzeDeadCodeArgs {
    path: Some(project_path),
    format: DeadCodeOutputFormat::Json,
    top_files: Some(3),
    include_unreachable: false,
    min_dead_lines: 20,
    include_tests: false,
}).await?;

// Generate markdown section
let dead_code_section = format!(r#"
**☠️ Dead Code Analysis (Top 3 Files):**
- **Total dead code**: {} lines ({:.1}% of codebase)
- **Dead functions**: {}
- **Files with dead code**: {} of {} analyzed
- **Hotspots**:
{}
"#, 
    dead_code_result.summary.total_dead_lines,
    dead_code_result.summary.dead_percentage,
    dead_code_result.summary.dead_functions,
    dead_code_result.summary.files_with_dead_code,
    dead_code_result.summary.total_files_analyzed,
    format_dead_code_hotspots(&dead_code_result.ranked_files)
);
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_dead_code_scoring() {
        // Test scoring algorithm with known inputs
    }
    
    #[test]
    fn test_confidence_levels() {
        // Test confidence detection for different scenarios
    }
    
    #[test]
    fn test_file_filtering() {
        // Test include_tests and min_dead_lines filters
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_dead_code_analysis_e2e() {
    // Create test project with known dead code
    let temp_dir = create_test_project_with_dead_code();
    
    // Run analysis
    let args = AnalyzeDeadCodeArgs {
        path: Some(temp_dir.path().to_path_buf()),
        format: DeadCodeOutputFormat::Json,
        top_files: Some(5),
        ..Default::default()
    };
    
    let result = handle_analyze_dead_code(&args, cache).await.unwrap();
    
    // Verify expected dead code is found
    assert_eq!(result.summary.dead_functions, 3);
    assert!(result.ranked_files[0].path.contains("unused.rs"));
}
```

Good catch! The specification currently covers **CLI** and **MCP** but is missing **HTTP/REST API** exposure. Let me add that section to make it complete:

## HTTP/REST API Interface

### Endpoint Definition

```rust
// In server/src/unified_protocol/service.rs - add to router
.route("/api/analyze/dead-code", post(analyze_dead_code_handler))
.route("/api/analyze/dead-code/:project_id", get(get_cached_dead_code))
```

### Request/Response Models

```rust
#[derive(Debug, Deserialize)]
pub struct DeadCodeAnalysisRequest {
    pub project_path: Option<String>,
    pub top_files: Option<usize>,
    pub include_unreachable: bool,
    pub include_tests: bool,
    pub min_dead_lines: usize,
    pub format: Option<String>,  // "summary" | "json" | "sarif" | "markdown"
}

#[derive(Debug, Serialize)]
pub struct DeadCodeAnalysisResponse {
    pub summary: DeadCodeSummary,
    pub ranked_files: Vec<FileDeadCodeMetrics>,
    pub metadata: AnalysisMetadata,
}
```

### Handler Implementation

```rust
pub async fn analyze_dead_code_handler(
    Extension(state): Extension<AppState>,
    Json(params): Json<DeadCodeAnalysisRequest>,
) -> Result<impl IntoResponse, AppError> {
    let args = AnalyzeDeadCodeArgs {
        path: params.project_path.map(PathBuf::from),
        format: params.format
            .and_then(|f| DeadCodeOutputFormat::from_str(&f).ok())
            .unwrap_or(DeadCodeOutputFormat::Json),
        top_files: params.top_files,
        include_unreachable: params.include_unreachable,
        include_tests: params.include_tests,
        min_dead_lines: params.min_dead_lines,
    };
    
    let result = handle_analyze_dead_code(&args, state.cache_manager).await
        .map_err(|e| AppError::AnalysisError(e.to_string()))?;
    
    Ok(Json(result))
}
```

### REST API Examples

```bash
# Basic dead code analysis
curl -X POST http://localhost:3000/api/analyze/dead-code \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "./server",
    "top_files": 5,
    "include_unreachable": false
  }'

# Get cached results
curl http://localhost:3000/api/analyze/dead-code/project123

# With all options
curl -X POST http://localhost:3000/api/analyze/dead-code \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "/path/to/project",
    "top_files": 10,
    "include_unreachable": true,
    "include_tests": false,
    "min_dead_lines": 20,
    "format": "sarif"
  }'
```

### Unified Protocol Integration

```rust
// In unified_protocol/adapters/http.rs
impl UnifiedService {
    async fn handle_dead_code_analysis(
        &self,
        request: UnifiedRequest,
    ) -> Result<UnifiedResponse, AppError> {
        // Extract parameters from unified request
        let params: DeadCodeAnalysisRequest = request.get_json()?;
        
        // Call shared analysis logic
        let result = self.analysis_service
            .analyze_dead_code(params)
            .await?;
        
        // Return unified response
        Ok(UnifiedResponse::ok().with_json(result)?)
    }
}
```

## Complete Interface Coverage Summary

### 1. **CLI Interface** ✅
```bash
paiml-mcp-agent-toolkit analyze dead-code --top-files 5 --format json
```

### 2. **MCP Interface** ✅
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "analyze_dead_code",
    "arguments": {
      "project_path": "./server",
      "top_files": 5,
      "include_unreachable": false
    }
  }
}
```

### 3. **HTTP/REST Interface** ✅
```bash
POST /api/analyze/dead-code
{
  "project_path": "./server",
  "top_files": 5,
  "include_unreachable": false
}
```

All three interfaces converge on the same core handler function, ensuring consistent behavior across all access methods. The unified protocol architecture handles the adaptation between different input/output formats automatically.


## Performance Considerations

1. **Caching**: Results are cached with file modification time checks
2. **Incremental Analysis**: Only re-analyze changed files
3. **Parallel Processing**: Use rayon for file-level parallelism
4. **Memory Efficiency**: Stream large codebases instead of loading all at once

## Future Enhancements

1. **IDE Integration**: VS Code extension showing dead code inline
2. **CI Integration**: Fail builds if dead code exceeds threshold
3. **Auto-fix**: Generate patches to remove simple dead code
4. **Historical Tracking**: Track dead code trends over time
5. **Language-specific Rules**: Custom dead code detection per language

## Documentation Updates

1. Update CLI reference with new command
2. Add dead code section to README metrics
3. Create user guide for interpreting results
4. Add to MCP tool documentation

## Migration Path

Since the `DeadCodeAnalyzer` already exists, implementation involves:

1. Week 1: CLI integration and basic formatting
2. Week 2: Ranking system and caching
3. Week 3: MCP tool and dogfooding integration
4. Week 4: Testing and documentation

## Success Criteria

- [ ] CLI command works with all specified options
- [ ] Output formats match specification
- [ ] Performance: <5s for analyzing 10K LOC
- [ ] Dogfooding shows our own dead code metrics
- [ ] 90%+ test coverage for new code
- [ ] Documentation is complete and accurate