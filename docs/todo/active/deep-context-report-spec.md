# Deep Context Analysis Enhancement Specification

## Overview

This specification defines enhancements to the `paiml-mcp-agent-toolkit analyze deep-context` command to provide comprehensive codebase analysis with both terse (default) and full reporting modes.

## Core Requirements

### 1. Metrics Philosophy
- **NO AVERAGES**: Replace all average calculations with median and max values
- **Rationale**: Averages can be misleading with skewed distributions common in software metrics
- **Application**: All numeric metrics including complexity, file sizes, line counts, etc.

### 2. Bug Fixes Required

#### SATD Integration Bug (DEEP-CONTEXT-SATD-001)
- **Issue**: `satd_results: null` despite SATD analyzer finding items
- **Root Cause**: Integration failure in deep_context.rs between SATD service and result aggregation
- **Fix Required**:
    - Ensure SATD analyzer is properly invoked in the analysis pipeline
    - Correctly serialize SATD results into the `AnalysisResults` structure
    - Map SATD items to file-level annotations

## Default Mode (Terse Report)

### Structure
```
# Deep Context Analysis: <project_name>
**Generated:** <timestamp> UTC
**Tool Version:** <version>
**Analysis Time:** <duration>

## Executive Summary
**Overall Health Score:** <score>/100 <emoji>
**Predicted High-Risk Files:** <count>
**Technical Debt Items:** <count> (High: <n>, Medium: <n>, Low: <n>)

## Key Metrics
### Complexity
- **Median Cyclomatic:** <n>
- **Max Cyclomatic:** <n> (<file>:<function>)
- **Violations:** <count>

### Code Churn (30 days)
- **Median Changes:** <n>
- **Max Changes:** <n> (<file>)
- **Hotspot Files:** <count>

### Technical Debt (SATD)
- **Total Items:** <count>
- **High Severity:** <count>
- **Debt Hotspots:** <count> files

### Duplicates
- **Clone Coverage:** <n>%
- **Type-1/2 Clones:** <n>
- **Type-3/4 Clones:** <n>

### Dead Code
- **Unreachable Functions:** <n>
- **Dead Code %:** <n>%

## AST Network Analysis
**Module Centrality (PageRank):**
1. <module> (score: <n>)
2. <module> (score: <n>)
3. <module> (score: <n>)

**Function Importance:**
1. <function> (connections: <n>)
2. <function> (connections: <n>)
3. <function> (connections: <n>)

## Top 5 Predicted Defect Files
1. <file> (risk score: <n>)
   - Complexity: <percentile>, Churn: <percentile>, SATD: <count>
2. <file> (risk score: <n>)
   - Complexity: <percentile>, Churn: <percentile>, SATD: <count>
[...]
```

### Defect Prediction Algorithm

Based on empirical software engineering research, use weighted scoring:

```rust
risk_score = 
    complexity_percentile * 0.35 +  // Strongest predictor (McCabe, 1976; Gill & Kemerer, 1991)
    churn_percentile * 0.30 +       // Second strongest (Nagappan & Ball, 2005)
    satd_density * 0.15 +           // Technical debt indicator (Potdar & Shihab, 2014)
    duplicate_ratio * 0.10 +        // Maintenance burden (Roy & Cordy, 2007)
    dead_code_ratio * 0.10          // Code quality indicator
```

## Full Mode (`--full` flag)

### Structure (Matching deep-context.ts)

```
# Deep Context: <project_name>

Generated: <timestamp>
Version: <version>

## Project Structure

```
<tree structure>
```

## Rust Files (<count>)

### <file_path>

**Imports:**
- `<path>` as <alias> (line <n>)
  [...]

**Modules:**
- `<visibility> mod <name>` (file|inline) (line <n>)
  [...]

**Structs:**
- `<visibility> struct <name>` [derives: <list>] (line <n>)
  [...]

**Enums:**
- `<visibility> enum <name>` (line <n>)
  [...]

**Functions:**
- `<visibility> <async> fn <name>(...) -> <return_type>` (line <n>)
  [...]

**Implementations:**
- `impl <trait> for <struct>` (line <n>)
  [...]

**Traits:**
- `<visibility> trait <name>` (line <n>)
  [...]

## TypeScript Files (<count>)
[Similar structure for TS files]

## Makefiles (<count>)
[Table format with targets, dependencies, commands]

## README Files (<count>)
[Full content in markdown blocks]

## Defect Analysis Report

### Complexity Violations
<file>:<line> - <function> (cyclomatic: <n>, cognitive: <n>)
[Top 25 entries with remediation hints]

### Code Churn Hotspots
<file> - <commits> changes, <authors> authors
  Last change: <date> by <author>
  Risk: <description of why this is problematic>
[Top 25 entries]

### Technical Debt (SATD)
<file>:<line> - <category> - Severity: <HIGH|MEDIUM|LOW>
Comment: "<text>"
Context: <surrounding code context>
Suggested fix: <remediation>
[Top 25 entries grouped by severity]

### Duplicate Code
Clone Group <n> (Type-<1-4>):
- <file1>:<line1-line2>
- <file2>:<line1-line2>
  Similarity: <n>%
  Extract suggestion: <proposed function/module>
  [Top 25 clone groups]

### Dead Code
<file>:<line> - <type> "<name>"
Reason: <why it's dead>
Safe to remove: <YES|MAYBE|CHECK>
[Top 25 entries]

## Dependency Analysis

### Module Dependency Graph
```mermaid
<generated mermaid diagram>
```

### Critical Paths
1. <module_a> → <module_b> → <module_c> (coupling: <n>)
   [Top 10 paths]

### Circular Dependencies
[List of circular dependency chains]

---
Generated by deep-context v<version>
```

## Implementation Details

### 1. SATD Integration Fix

```rust
// In deep_context.rs analyze_project method
async fn analyze_project(&self, config: DeepContextConfig) -> Result<DeepContext> {
    // ... existing code ...
    
    // Add SATD analysis
    let satd_results = if config.analyses.contains(&AnalysisType::Satd) {
        let satd_detector = SATDDetector::new();
        let result = satd_detector.analyze_directory(&project_path).await?;
        Some(result)
    } else {
        None
    };
    
    // Ensure SATD results are properly included in AnalysisResults
    let analyses = AnalysisResults {
        complexity_report,
        churn_results,
        satd_results, // This is currently being set to None
        dead_code_results,
        duplicate_results, // New field
        // ...
    };
}
```

### 2. Duplicate Detection Integration

```rust
// Add to AnalysisResults struct
pub struct AnalysisResults {
    // ... existing fields ...
    pub duplicate_results: Option<DuplicateAnalysisResult>,
}

pub struct DuplicateAnalysisResult {
    pub total_files: usize,
    pub clone_coverage_percentage: f32,
    pub type1_clones: usize,  // Exact
    pub type2_clones: usize,  // Renamed
    pub type3_clones: usize,  // Gapped
    pub type4_clones: usize,  // Semantic
    pub clone_groups: Vec<CloneGroup>,
}
```

### 3. PageRank AST Analysis

```rust
pub struct AstNetworkAnalysis {
    pub module_centrality: Vec<(String, f32)>,  // Module path, PageRank score
    pub function_importance: Vec<(String, usize)>, // Function name, connection count
    pub critical_paths: Vec<Vec<String>>,
    pub circular_dependencies: Vec<Vec<String>>,
}

impl DeepContextAnalyzer {
    async fn analyze_ast_network(&self, dag: &DependencyGraph) -> AstNetworkAnalysis {
        // Convert DAG to petgraph for PageRank calculation
        let graph = self.dag_to_petgraph(dag);
        
        // Calculate PageRank scores
        let pagerank_scores = self.calculate_pagerank(&graph, 0.85, 100);
        
        // Extract top modules by centrality
        let module_centrality = self.extract_top_modules(pagerank_scores, 10);
        
        // Calculate function importance by connection count
        let function_importance = self.calculate_function_connections(&graph);
        
        // Find critical paths using longest path algorithm
        let critical_paths = self.find_critical_paths(&graph, 10);
        
        // Detect circular dependencies
        let circular_deps = self.detect_circular_dependencies(&graph);
        
        AstNetworkAnalysis {
            module_centrality,
            function_importance,
            critical_paths,
            circular_dependencies,
        }
    }
}
```

### 4. Metrics Replacement

```rust
// Replace all average calculations
impl ComplexityReport {
    pub fn calculate_summary(&self) -> ComplexitySummary {
        let complexities: Vec<u32> = self.files.iter()
            .flat_map(|f| f.functions.iter().map(|fn| fn.cyclomatic))
            .collect();
        
        ComplexitySummary {
            median_cyclomatic: calculate_median(&complexities),
            max_cyclomatic: complexities.iter().max().cloned().unwrap_or(0),
            // Remove: average_cyclomatic
        }
    }
}

fn calculate_median<T: Ord + Clone>(values: &[T]) -> T {
    let mut sorted = values.to_vec();
    sorted.sort();
    let mid = sorted.len() / 2;
    sorted[mid].clone()
}
```

### 5. Weighted Defect Prediction

```rust
pub struct DefectPredictor {
    pub fn predict_defects(&self, context: &DeepContext) -> Vec<PredictedDefect> {
        let mut file_scores = HashMap::new();
        
        // Calculate percentiles for each metric
        let complexity_values = self.extract_complexity_values(context);
        let churn_values = self.extract_churn_values(context);
        
        for file in &context.files {
            let complexity_percentile = self.calculate_percentile(
                file.complexity_score, &complexity_values
            );
            let churn_percentile = self.calculate_percentile(
                file.churn_score, &churn_values
            );
            let satd_density = file.satd_items as f32 / file.total_lines as f32;
            let duplicate_ratio = file.duplicate_lines as f32 / file.total_lines as f32;
            let dead_code_ratio = file.dead_code_items as f32 / file.total_items as f32;
            
            // Research-based weights
            let risk_score = 
                complexity_percentile * 0.35 +
                churn_percentile * 0.30 +
                satd_density * 0.15 +
                duplicate_ratio * 0.10 +
                dead_code_ratio * 0.10;
            
            file_scores.insert(file.path.clone(), PredictedDefect {
                file: file.path.clone(),
                risk_score,
                complexity_percentile,
                churn_percentile,
                satd_count: file.satd_items,
                duplicate_percentage: duplicate_ratio * 100.0,
                dead_code_percentage: dead_code_ratio * 100.0,
            });
        }
        
        // Sort by risk score and return top N
        let mut defects: Vec<_> = file_scores.into_values().collect();
        defects.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap());
        defects
    }
}
```

## Command Line Interface

```bash
# Default terse mode
paiml-mcp-agent-toolkit analyze deep-context

# Full detailed mode
paiml-mcp-agent-toolkit analyze deep-context --full

# Custom configuration
paiml-mcp-agent-toolkit analyze deep-context \
  --include "ast,complexity,churn,satd,duplicate" \
  --exclude "dead-code" \
  --period-days 90 \
  --top-files 10 \
  --format json

# Output to file
paiml-mcp-agent-toolkit analyze deep-context --full --output report.md
```

## Testing Requirements

1. **SATD Integration Tests**
    - Verify SATD results are non-null when SATD items exist
    - Validate severity classification
    - Ensure file-level annotation counts match

2. **Duplicate Detection Tests**
    - Test all clone types (1-4) are detected
    - Verify clone coverage calculation
    - Test performance on large codebases

3. **PageRank Analysis Tests**
    - Verify deterministic results
    - Test on cyclic and acyclic graphs
    - Validate centrality scores sum to 1.0

4. **Metrics Tests**
    - Ensure no averages remain in output
    - Verify median calculation correctness
    - Test edge cases (empty sets, single values)

5. **Defect Prediction Tests**
    - Validate percentile calculations
    - Test weight sensitivity
    - Verify top-N selection

## Performance Considerations

1. **Caching Strategy**
    - Cache PageRank calculations (expensive)
    - Cache duplicate detection results
    - Invalidate on file changes

2. **Parallelization**
    - Run analyses in parallel where possible
    - Use rayon for parallel file processing
    - Limit concurrent file handles

3. **Memory Management**
    - Stream large file processing
    - Use memory-mapped files for large codebases
    - Implement progressive result building

## Success Criteria

1. Default mode completes in <5 seconds for typical projects
2. Full mode provides identical structure to deep-context.ts
3. SATD integration returns actual results
4. All metrics use median/max instead of averages
5. Defect predictions correlate with actual bug locations (validation study)