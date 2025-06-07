# Single-Shot Deep Context Specification

## Executive Summary

This specification details the implementation of zero-configuration deep context generation for the PAIML MCP Agent Toolkit, enabling `paiml-mcp-agent-toolkit context` to "just work" without mandatory toolchain specification while maintaining full backward compatibility.

## Design Principles

1. **Zero Required Input**: The tool should work with just `paiml-mcp-agent-toolkit context`
2. **Intelligent Detection**: Automatically detect languages, build systems, and project structure
3. **Progressive Enhancement**: Start with basic analysis, enhance based on what's discovered
4. **Graceful Degradation**: Continue analysis even when some components fail
5. **Backward Compatibility**: Existing CLI arguments remain functional

## Implementation Architecture

### 1. Adaptive Language Detection Pipeline

```rust
pub struct PolyglotDetector {
    significance_weights: HashMap<Language, LanguageWeight>,
    detection_strategies: Vec<Box<dyn DetectionStrategy>>,
}

pub struct LanguageWeight {
    build_file_weight: f64,    // Cargo.toml = 1.5, package.json = 1.3
    entry_point_weight: f64,   // main.rs = 1.2, index.ts = 1.1
    file_count_weight: f64,    // Per-file contribution
    loc_weight: f64,           // Lines of code contribution
}

impl PolyglotDetector {
    pub async fn detect_project_languages(root: &Path) -> Result<Vec<(Language, f64)>> {
        let mut scores: HashMap<Language, f64> = HashMap::new();
        
        // Primary indicators (build files)
        self.detect_build_files(root, &mut scores).await?;
        
        // Secondary indicators (file extensions + content)
        self.detect_by_extensions(root, &mut scores).await?;
        
        // Tertiary indicators (shebang, imports, syntax)
        self.detect_by_content(root, &mut scores).await?;
        
        // Sort by significance and return
        Ok(scores.into_iter()
            .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
            .collect())
    }
}
```

**Detection Strategies:**

1. **Build File Detection** (Primary)
    - `Cargo.toml` + `Cargo.lock` → Rust (weight: 1.5)
    - `package.json` + `node_modules/` → TypeScript/JavaScript (weight: 1.3)
    - `pyproject.toml` / `setup.py` / `requirements.txt` → Python (weight: 1.4)
    - `go.mod` → Go (weight: 1.4)
    - `pom.xml` / `build.gradle` → Java (weight: 1.3)

2. **Extension-Based Detection** (Secondary)
    - Count files by extension with decreasing weights
    - Consider nested depth (deeper = less weight)
    - Account for test files separately

3. **Content-Based Detection** (Tertiary)
    - Shebang lines (`#!/usr/bin/env python`)
    - Import patterns (`use std::`, `import React`, `from typing`)
    - Syntax patterns (async/await, decorators, macros)

### 2. Progressive Enhancement Architecture

```rust
pub struct ProgressiveAnalyzer {
    stages: Vec<AnalysisStage>,
    failure_mode: FailureMode,
}

pub struct AnalysisStage {
    id: &'static str,
    required: bool,
    timeout: Duration,
    analyzer: Box<dyn StageAnalyzer>,
}

pub enum FailureMode {
    FailFast,      // Stop on first error
    BestEffort,    // Continue with degraded functionality
    Diagnostic,    // Log errors but continue
}

impl ProgressiveAnalyzer {
    pub async fn analyze_with_fallbacks(&self, path: &Path) -> DeepContext {
        let mut context = DeepContext::default();
        let mut metadata = AnalysisMetadata::default();
        
        for stage in &self.stages {
            let stage_result = timeout(stage.timeout, stage.analyzer.analyze(path)).await;
            
            match stage_result {
                Ok(Ok(data)) => {
                    context.merge(data);
                    metadata.successful_stages.push(stage.id);
                }
                Ok(Err(e)) if !stage.required => {
                    metadata.skipped_stages.push((stage.id, e.to_string()));
                    if let Some(fallback) = stage.analyzer.fallback_strategy() {
                        if let Ok(fallback_data) = fallback.analyze(path).await {
                            context.merge_partial(fallback_data);
                        }
                    }
                }
                Ok(Err(e)) if stage.required => {
                    return DeepContext::error(e);
                }
                Err(_) => {
                    metadata.timeout_stages.push(stage.id);
                }
                _ => {}
            }
        }
        
        context.metadata = Some(metadata);
        context
    }
}
```

**Analysis Stages (in order):**

1. **Language Detection** (required, 100ms timeout)
2. **Project Structure** (required, 200ms timeout)
3. **AST Analysis** (optional, 5s timeout per language)
4. **Git Analysis** (optional, 2s timeout, fallback: filesystem timestamps)
5. **Complexity Analysis** (optional, 3s timeout)
6. **Dependency Graph** (optional, 2s timeout)
7. **Dead Code Detection** (optional, 3s timeout)
8. **SATD Detection** (optional, 1s timeout)
9. **Test Coverage** (optional, 2s timeout, fallback: test file counting)

### 3. Intelligent Context Pruning

```rust
pub struct RelevanceScorer {
    project_idf: HashMap<String, f64>,      // Inverse document frequency
    centrality_scores: HashMap<NodeKey, f64>, // From PageRank
    complexity_scores: HashMap<PathBuf, f64>, // From complexity analysis
}

impl RelevanceScorer {
    pub fn score_item(&self, item: &ContextItem) -> f64 {
        let mut score = 0.0;
        
        // Base score from item type
        score += match item {
            ContextItem::PublicAPI { .. } => 10.0,
            ContextItem::EntryPoint { .. } => 9.0,
            ContextItem::CoreType { .. } => 8.0,
            ContextItem::ComplexFunction { complexity, .. } => {
                5.0 + (complexity as f64).ln()
            }
            _ => 1.0,
        };
        
        // Boost by cross-references
        if let Some(centrality) = self.centrality_scores.get(&item.id()) {
            score *= 1.0 + centrality;
        }
        
        // Boost by uniqueness (IDF)
        for term in item.significant_terms() {
            if let Some(idf) = self.project_idf.get(term) {
                score += idf * 0.1;
            }
        }
        
        // Penalty for technical debt
        if item.has_satd() {
            score *= 0.8;
        }
        
        score
    }
    
    pub fn prune_to_target_size(&self, items: Vec<ContextItem>, target_kb: usize) -> Vec<ContextItem> {
        let mut scored_items: Vec<(f64, ContextItem)> = items.into_iter()
            .map(|item| (self.score_item(&item), item))
            .collect();
            
        scored_items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        let mut result = Vec::new();
        let mut current_size = 0;
        
        for (_, item) in scored_items {
            let item_size = item.estimated_size();
            if current_size + item_size > target_kb * 1024 {
                break;
            }
            current_size += item_size;
            result.push(item);
        }
        
        // Ensure structural completeness
        self.add_containing_modules(&mut result);
        
        result
    }
}
```

**Relevance Factors:**

1. **Structural Importance**
    - Public APIs (×10)
    - Entry points (×9)
    - Core types/traits (×8)
    - Complex functions (×5 + ln(complexity))

2. **Graph Centrality**
    - PageRank scores from dependency analysis
    - In/out degree ratios
    - Betweenness centrality for key paths

3. **Information Uniqueness**
    - TF-IDF scoring for identifiers
    - Rare patterns get higher scores
    - Common boilerplate gets lower scores

4. **Quality Adjustments**
    - Technical debt penalty (×0.8)
    - Test coverage boost (×1.2)
    - Recent churn penalty (×0.9)

### 4. Smart Default Configuration

```rust
pub struct SmartDefaults {
    heuristics: Vec<Box<dyn ConfigHeuristic>>,
}

pub struct DeepContextConfig {
    pub analysis_depth: AnalysisDepth,
    pub include_private: bool,
    pub max_file_size: usize,
    pub timeout_budget: Duration,
    pub output_format: OutputFormat,
    pub target_size: OutputSize,
}

pub enum AnalysisDepth {
    Overview,    // <10K LOC: everything
    Standard,    // 10K-100K LOC: public + complex
    Large,       // 100K-1M LOC: public only
    Monorepo,    // >1M LOC: per-package analysis
}

pub enum OutputSize {
    Compact,     // ~20KB for LLM context
    Standard,    // ~100KB for documentation
    Full,        // ~500KB for analysis
    Unlimited,   // Everything
}

impl SmartDefaults {
    pub fn infer_config(path: &Path) -> Result<DeepContextConfig> {
        let mut config = DeepContextConfig::default();
        
        // Detect project size
        let metrics = quick_project_metrics(path)?;
        
        config.analysis_depth = match metrics.total_loc {
            0..=10_000 => AnalysisDepth::Overview,
            10_001..=100_000 => AnalysisDepth::Standard,
            100_001..=1_000_000 => AnalysisDepth::Large,
            _ => AnalysisDepth::Monorepo,
        };
        
        // Detect output intent from environment
        if std::env::var("CI").is_ok() {
            config.output_format = OutputFormat::MachineReadable;
        } else if std::io::stdout().is_terminal() {
            config.output_format = OutputFormat::Markdown;
        }
        
        // Detect monorepo patterns
        if has_workspace_file(path) || has_multiple_package_files(path) {
            config.analysis_depth = AnalysisDepth::Monorepo;
        }
        
        // Performance tuning based on system
        let cpu_count = num_cpus::get();
        config.timeout_budget = match (metrics.total_loc, cpu_count) {
            (loc, cpus) if loc < 50_000 && cpus >= 8 => Duration::from_secs(30),
            (loc, cpus) if loc < 100_000 && cpus >= 4 => Duration::from_secs(45),
            _ => Duration::from_secs(60),
        };
        
        Ok(config)
    }
}
```

### 5. Universal Output Adapter

```rust
pub struct UniversalOutputAdapter {
    format_detectors: Vec<Box<dyn FormatDetector>>,
    quality_enhancers: Vec<Box<dyn QualityEnhancer>>,
}

impl UniversalOutputAdapter {
    pub fn generate_optimal_output(&self, context: &DeepContext, hints: &EnvironmentHints) -> Result<String> {
        let format = self.detect_format(hints)?;
        let enhanced_context = self.enhance_for_format(context, &format)?;
        
        match format {
            OutputFormat::Markdown => self.generate_markdown(&enhanced_context),
            OutputFormat::Json => self.generate_json(&enhanced_context),
            OutputFormat::Sarif => self.generate_sarif(&enhanced_context),
            OutputFormat::LLMContext => self.generate_llm_context(&enhanced_context),
        }
    }
    
    fn detect_format(&self, hints: &EnvironmentHints) -> OutputFormat {
        // Check explicit format request
        if let Some(fmt) = hints.explicit_format {
            return fmt;
        }
        
        // Check output destination
        if let Some(path) = &hints.output_path {
            return match path.extension().and_then(|e| e.to_str()) {
                Some("json") => OutputFormat::Json,
                Some("sarif") => OutputFormat::Sarif,
                Some("md") => OutputFormat::Markdown,
                _ => OutputFormat::Markdown,
            };
        }
        
        // Check environment
        if hints.is_ci {
            OutputFormat::Sarif
        } else if hints.is_pipe {
            OutputFormat::LLMContext
        } else {
            OutputFormat::Markdown
        }
    }
    
    fn generate_llm_context(&self, context: &DeepContext) -> Result<String> {
        let mut output = String::new();
        
        // Optimized for AI consumption
        writeln!(&mut output, "# Project Context\n")?;
        writeln!(&mut output, "## Key APIs")?;
        
        for api in context.public_apis.iter().take(20) {
            writeln!(&mut output, "- `{}`: {}", api.signature, api.purpose)?;
        }
        
        writeln!(&mut output, "\n## Core Types")?;
        for typ in context.core_types.iter().take(15) {
            writeln!(&mut output, "- `{}`: {} fields", typ.name, typ.field_count)?;
        }
        
        writeln!(&mut output, "\n## Entry Points")?;
        for entry in &context.entry_points {
            writeln!(&mut output, "- `{}`", entry.path)?;
        }
        
        Ok(output)
    }
}
```

## CLI Interface Changes

### Current Interface (Maintained)
```bash
# Explicit toolchain selection still works
paiml-mcp-agent-toolkit context rust --project-path ./src
paiml-mcp-agent-toolkit context deno --format json
paiml-mcp-agent-toolkit context python-uv --output analysis.md
```

### New Zero-Config Interface
```bash
# Just works - auto-detects everything
paiml-mcp-agent-toolkit context

# Works for any project type
cd ~/projects/mixed-rust-typescript && paiml-mcp-agent-toolkit context
cd ~/projects/python-ml && paiml-mcp-agent-toolkit context
cd ~/projects/monorepo && paiml-mcp-agent-toolkit context

# Smart format detection
paiml-mcp-agent-toolkit context -o report.json  # JSON output
paiml-mcp-agent-toolkit context | llm           # Compact LLM format
CI=1 paiml-mcp-agent-toolkit context            # SARIF format
```

### Implementation Strategy

1. **Make toolchain argument optional**:
   ```rust
   #[derive(Parser)]
   pub struct ContextArgs {
       /// Target toolchain (auto-detected if not specified)
       #[arg(value_name = "TOOLCHAIN")]
       pub toolchain: Option<String>,
       
       // ... rest of args unchanged
   }
   ```

2. **Implement auto-detection fallback**:
   ```rust
   pub async fn run_context_command(args: ContextArgs) -> Result<()> {
       let toolchain = match args.toolchain {
           Some(t) => t,
           None => {
               let detector = PolyglotDetector::new();
               let languages = detector.detect_project_languages(&args.project_path).await?;
               
               if languages.is_empty() {
                   return Err(anyhow!("No supported languages detected"));
               }
               
               // Use primary language or "mixed" for multi-language
               if languages.len() == 1 {
                   language_to_toolchain(languages[0].0)
               } else {
                   "mixed".to_string()
               }
           }
       };
       
       // Continue with existing logic
   }
   ```

## Performance Guarantees

1. **Startup Time**: <50ms for detection phase
2. **Memory Usage**: <100MB for projects up to 1M LOC
3. **Timeout Budget**: 60s total for all analysis
4. **Output Size**: 20KB (compact) to 500KB (full)
5. **Parallelism**: Uses all available cores with work-stealing

## Error Handling

1. **No Languages Detected**: Clear error with suggestions
2. **Partial Failures**: Continue with degraded analysis
3. **Timeout**: Return partial results with metadata
4. **Large Projects**: Automatic sampling and pruning
5. **Access Errors**: Skip inaccessible files, note in metadata

## Migration Path

1. **Phase 1**: Implement detection without changing CLI
2. **Phase 2**: Make toolchain optional with fallback
3. **Phase 3**: Add smart defaults and pruning
4. **Phase 4**: Implement progressive enhancement
5. **Phase 5**: Add format auto-detection

## Testing Strategy

1. **Unit Tests**: Each detector and scorer
2. **Integration Tests**: Full pipeline with various project types
3. **Performance Tests**: Ensure guarantees are met
4. **Regression Tests**: Existing CLI behavior unchanged
5. **Dogfooding**: Use on paiml-mcp-agent-toolkit itself

## Success Metrics

1. **Zero-config success rate**: >95% of projects
2. **Performance**: <10s for 90th percentile
3. **Output quality**: Relevant content in top 20KB
4. **Backward compatibility**: 100% existing tests pass
5. **User satisfaction**: Reduced friction, same quality