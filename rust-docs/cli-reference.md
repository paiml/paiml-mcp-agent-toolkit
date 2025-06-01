# CLI Reference

## Formal Grammar

```ebnf
command     ::= binary-name [global-opts] subcommand [subcommand-opts]
binary-name ::= "paiml-mcp-agent-toolkit"
global-opts ::= "--mode" mode-value
mode-value  ::= "cli" | "mcp"
subcommand  ::= "generate" | "scaffold" | "list" | "search" | "validate" | "context" | "analyze"
analyze     ::= "analyze" ("churn" | "complexity" | "dag" | "dead-code" | "deep-context")
```

## Memory Model

The CLI employs a **zero-copy architecture** for template rendering:

```rust
// Templates stored as Arc<str> for shared immutable access
static TEMPLATE_STORE: Lazy<HashMap<&'static str, Arc<str>>> = Lazy::new(|| {
    let mut map = HashMap::with_capacity(9); // Known template count
    map.insert("makefile/rust/cli", Arc::from(include_str!("../templates/makefile/rust/cli.hbs")));
    map
});
```

## Global Options

### `--mode`
Force specific execution mode. Auto-detected by default based on environment.

- **Values**: `cli`, `mcp`
- **Default**: Auto-detect (MCP when STDIO is redirected)
- **Example**: `paiml-mcp-agent-toolkit --mode mcp`

## Commands

### `generate` (aliases: `gen`, `g`)

Generate a single template file with zero-copy rendering.

#### Performance Characteristics

| Phase | Latency (p99) | Memory | Allocations |
|-------|---------------|--------|-------------|
| Parse | 0.1ms | 4KB | 12 |
| Validate | 0.2ms | 2KB | 8 |
| Render | 2.5ms | 64KB | 127 |
| **Total** | **2.8ms** | **70KB** | **147** |

#### Arguments

- **category** (required): Template category (e.g., `makefile`, `readme`, `gitignore`)
- **template** (required): Template path within category (e.g., `rust/cli`)
- **-p, --param** (repeated): Parameters as key=value pairs
- **-o, --output**: Output file path (defaults to stdout)
- **--create-dirs**: Create parent directories if they don't exist

#### Type System Integration

Parameter parsing leverages Rust's type system for zero-cost abstractions:

```rust
#[derive(Debug, Clone)]
pub enum TypedValue {
    Bool(bool),
    Integer(i64),
    String(SmartString), // Small string optimization
}

impl FromStr for TypedValue {
    type Err = Infallible;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "true" => TypedValue::Bool(true),
            "false" => TypedValue::Bool(false),
            s if let Ok(n) = s.parse::<i64>() => TypedValue::Integer(n),
            s => TypedValue::String(SmartString::from(s)),
        })
    }
}
```

#### Examples

```bash
# Generate Makefile for Rust CLI project
paiml-mcp-agent-toolkit generate makefile rust/cli \
  -p project_name=my-project \
  -p has_tests=true \
  -p has_benchmarks=false \
  -o Makefile

# Generate .gitignore to stdout
paiml-mcp-agent-toolkit gen gitignore rust/cli
```

### `scaffold`

Scaffold complete project structure with parallel template generation.

#### Concurrency Model

Template generation is **embarrassingly parallel** for batch operations:

```rust
pub fn scaffold_parallel(templates: Vec<TemplateRequest>) -> Result<Vec<GeneratedFile>> {
    templates
        .into_par_iter()
        .map(|req| generate_single(req))
        .collect::<Result<Vec<_>, _>>()
}
```

#### Arguments

- **toolchain** (required): Target toolchain (`rust`, `deno`, `python-uv`)
- **-t, --templates** (required): Comma-separated list of templates to generate
- **-p, --param** (repeated): Parameters as key=value pairs
- **--parallel**: Parallelism level (default: CPU count)

#### Examples

```bash
# Scaffold complete Rust project
paiml-mcp-agent-toolkit scaffold rust \
  -t makefile,readme,gitignore \
  -p project_name=my-cli \
  -p author="Jane Doe" \
  --parallel 4
```

### `list`

List available templates with filtering options.

#### Arguments

- **--toolchain**: Filter by specific toolchain
- **--category**: Filter by template category
- **--format**: Output format (`table`, `json`, `yaml`)

#### Examples

```bash
# List all templates in table format
paiml-mcp-agent-toolkit list

# List Rust templates as JSON
paiml-mcp-agent-toolkit list --toolchain rust --format json
```

### `search`

Search templates using fuzzy matching with relevance scoring.

#### Arguments

- **query** (required): Search query string
- **--toolchain**: Filter by specific toolchain
- **--limit**: Maximum results to display (default: 20)

#### Examples

```bash
# Search for makefile templates
paiml-mcp-agent-toolkit search makefile

# Search Rust-specific templates
paiml-mcp-agent-toolkit search cli --toolchain rust --limit 10
```

### `validate`

Validate template parameters before generation.

#### Arguments

- **uri** (required): Template URI to validate
- **-p, --param** (repeated): Parameters to validate

#### Examples

```bash
# Validate makefile parameters
paiml-mcp-agent-toolkit validate template://makefile/rust/cli \
  -p project_name=my-project \
  -p has_tests=true
```

### `context`

Generate project context through AST analysis with persistent caching.

#### Cache Architecture

```rust
pub struct CacheHierarchy {
    l1: Arc<DashMap<CacheKey, CacheEntry>>,      // Thread-local, 100 entries
    l2: Arc<RwLock<LruCache<CacheKey, Arc<[u8]>>>>, // Shared, 1000 entries  
    l3: MmapCache,                                // Memory-mapped, unbounded
}
```

#### Arguments

- **toolchain** (required): Target toolchain for analysis
- **-p, --project-path**: Project path to analyze (default: current directory)
- **-o, --output**: Output file path
- **--format**: Output format (`markdown`, `json`)

#### Examples

```bash
# Analyze current Rust project
paiml-mcp-agent-toolkit context rust

# Analyze specific project as JSON
paiml-mcp-agent-toolkit context deno \
  -p /path/to/project \
  -o context.json \
  --format json
```

### `analyze`

Analyze code metrics and patterns with subcommands.

#### Subcommands

##### `analyze churn`

Analyze code change frequency using git history.

**Arguments:**
- **-p, --project-path**: Project path (default: current directory)
- **-d, --days**: Number of days to analyze (default: 30)
- **--format**: Output format (`summary`, `json`, `markdown`, `csv`)
- **-o, --output**: Output file path

**Examples:**
```bash
# Analyze last 30 days of code churn
paiml-mcp-agent-toolkit analyze churn

# Generate markdown report for last week
paiml-mcp-agent-toolkit analyze churn \
  -d 7 \
  --format markdown \
  -o HOTSPOTS.md
```

##### `analyze complexity`

Analyze code complexity metrics with configurable thresholds and file ranking.

**Arguments:**
- **-p, --project-path**: Project path (default: current directory)
- **--toolchain**: Override detected toolchain
- **--format**: Output format (`summary`, `full`, `json`, `sarif`)
- **-o, --output**: Output file path
- **--max-cyclomatic**: Custom cyclomatic complexity threshold
- **--max-cognitive**: Custom cognitive complexity threshold
- **--include**: Include file patterns (e.g., `**/*.rs`)
- **--watch**: Watch mode for continuous analysis (not yet implemented)
- **--top-files**: Number of top complex files to show (0 = show all violations)

**File Ranking System:**

The `--top-files` flag enables a sophisticated ranking system that identifies the most complex files using a composite scoring algorithm:

- **Cyclomatic Complexity** (40% weight): Number of linearly independent paths
- **Cognitive Complexity** (40% weight): How difficult code is to understand  
- **Function Count** (20% weight): Number of functions/methods in the file

The ranking table includes:
- Function count per file
- Maximum cyclomatic complexity
- Average cognitive complexity
- Halstead effort estimate
- Total composite score (0-100 scale)

**Examples:**
```bash
# Analyze with default thresholds
paiml-mcp-agent-toolkit analyze complexity

# Show top 5 most complex files
paiml-mcp-agent-toolkit analyze complexity --top-files 5

# Generate SARIF report for IDE integration
paiml-mcp-agent-toolkit analyze complexity \
  --format sarif \
  --max-cyclomatic 15 \
  --max-cognitive 20 \
  -o complexity.sarif

# Analyze specific files with top 3 ranking
paiml-mcp-agent-toolkit analyze complexity \
  --include "src/**/*.rs" \
  --include "tests/**/*.rs" \
  --top-files 3

# JSON output with ranking data
paiml-mcp-agent-toolkit analyze complexity \
  --top-files 10 \
  --format json \
  -o complexity-report.json
```

##### `analyze dag`

Generate dependency graphs using Mermaid syntax.

**Arguments:**
- **--dag-type**: Type of graph (`call-graph`, `import-graph`, `inheritance`, `full-dependency`)
- **-p, --project-path**: Project path (default: current directory)
- **-o, --output**: Output file path
- **--max-depth**: Maximum graph traversal depth
- **--filter-external**: Filter out external dependencies
- **--show-complexity**: Include complexity metrics in graph

**Examples:**
```bash
# Generate call graph
paiml-mcp-agent-toolkit analyze dag \
  --dag-type call-graph \
  -o call-graph.mmd

# Generate import graph with complexity
paiml-mcp-agent-toolkit analyze dag \
  --dag-type import-graph \
  --show-complexity \
  --max-depth 3 \
  --filter-external
```

##### `analyze deep-context`

**NEW**: Generate comprehensive deep context analysis combining multiple analysis types into unified quality assessment.

**Arguments:**
- **-p, --project-path**: Project path to analyze (default: current directory)
- **--format**: Output format (`markdown`, `json`, `sarif`)
- **-o, --output**: Output file path
- **--include**: Comma-separated list of analyses to include (`ast`, `complexity`, `churn`, `dag`, `dead-code`, `satd`, `defect-probability`)
- **--exclude**: Comma-separated list of analyses to exclude
- **--period-days**: Period for churn analysis (default: 30 days)
- **--dag-type**: DAG type for dependency analysis (`call-graph`, `import-graph`, `inheritance`, `full-dependency`)
- **--max-depth**: Maximum directory traversal depth
- **--include-pattern**: Include file patterns (can be specified multiple times)
- **--exclude-pattern**: Exclude file patterns (can be specified multiple times)
- **--cache-strategy**: Cache usage strategy (`normal`, `force-refresh`, `offline`)
- **--parallel**: Parallelism level for analysis
- **--full**: Enable full detailed report (default is terse)

**Multi-Analysis Pipeline:**

Deep context analysis combines multiple analysis types:

1. **AST Analysis**: Abstract syntax tree parsing and symbol extraction
2. **Complexity Analysis**: McCabe Cyclomatic and Cognitive complexity metrics
3. **Churn Analysis**: Git history and change frequency tracking
4. **DAG Analysis**: Dependency graph generation and visualization
5. **Dead Code Analysis**: Unused code detection with confidence scoring
6. **SATD Analysis**: Self-Admitted Technical Debt detection from comments
7. **Defect Probability**: ML-based defect prediction and hotspot identification

**Quality Scorecard Features:**

- **Overall Health Score** (0-100): Composite quality assessment
- **Maintainability Index**: Code maintainability metrics
- **Technical Debt Hours**: Estimated effort to address debt
- **Defect Correlation**: Cross-analysis insights and risk prediction
- **Prioritized Recommendations**: AI-generated actionable improvement suggestions

**Performance Characteristics:**

- **Parallel Execution**: Tokio-based concurrent analysis using JoinSet
- **Cache Integration**: Smart caching strategies for incremental analysis
- **Memory Efficiency**: Optimized data structures with streaming output
- **Analysis Time**: ~2.5ms for focused analysis, ~8 seconds for full project

**Examples:**
```bash
# Basic deep context analysis with markdown output
paiml-mcp-agent-toolkit analyze deep-context

# Targeted analysis with specific components
paiml-mcp-agent-toolkit analyze deep-context \
  --include "complexity,churn,satd" \
  --format json \
  -o deep-context.json

# Full analysis with all components and SARIF output
paiml-mcp-agent-toolkit analyze deep-context \
  --include "ast,complexity,churn,dag,dead-code,satd,defect-probability" \
  --format sarif \
  --period-days 90 \
  --cache-strategy force-refresh \
  -o analysis.sarif

# Comprehensive analysis with custom patterns
paiml-mcp-agent-toolkit analyze deep-context \
  --include "complexity,dead-code,satd" \
  --include-pattern "src/**/*.rs" \
  --include-pattern "tests/**/*.rs" \
  --exclude-pattern "**/target/**" \
  --parallel 8 \
  --full \
  --format markdown \
  -o DEEP_CONTEXT.md

# Performance-optimized analysis with offline cache
paiml-mcp-agent-toolkit analyze deep-context \
  --include "complexity,churn" \
  --cache-strategy offline \
  --max-depth 5 \
  --format json
```

## Environment Variable Expansion

The CLI supports environment variable expansion in default values:

```rust
pub fn expand_env_vars(template: &str) -> String {
    // Simple ${VAR} expansion
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_else(|_| caps[0].to_string())
    })
    .to_string()
}
```

## Exit Codes

- **0**: Success
- **1**: General error
- **2**: Invalid arguments
- **3**: Template not found
- **4**: Validation error
- **5**: I/O error

## Performance Considerations

### Zero-Copy Operations

1. **Template Storage**: All templates are compiled into the binary as `&'static str`
2. **Parameter Parsing**: Uses `Cow<str>` to avoid unnecessary allocations
3. **JSON Rendering**: Leverages `simd-json` for SIMD-accelerated parsing

### Memory Efficiency

- **Pre-allocated Buffers**: 64KB read/write buffers
- **Object Pooling**: Reusable buffer pool for concurrent operations
- **Small String Optimization**: Uses `SmartString` for parameter values

### Concurrency

- **Parallel File I/O**: Bounded concurrent file operations
- **Lock-Free Caching**: `DashMap` for thread-safe template access
- **Work Stealing**: Rayon-based parallel iteration