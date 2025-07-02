# CLI Reference

## Formal Grammar

```ebnf
command     ::= binary-name [global-opts] subcommand [subcommand-opts]
binary-name ::= "paiml-mcp-agent-toolkit"
global-opts ::= "--mode" mode-value
mode-value  ::= "cli" | "mcp"
subcommand  ::= "generate" | "scaffold" | "list" | "search" | "validate" | "context" | "analyze"
analyze     ::= "analyze" ("churn" | "complexity" | "dag" | "dead-code" | "deep-context" | 
                           "big-o" | "makefile-lint" | "proof-annotations" | "graph-metrics" |
                           "name-similarity" | "defect-prediction" | "incremental-coverage" |
                           "symbol-table" | "satd" | "tdg" | "assemblyscript" | "webassembly")
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

### `demo`

**NEW**: Demonstrate the toolkit's capabilities with protocol-agnostic architecture supporting multiple interfaces.

#### Arguments

- **-p, --path**: Path to analyze (default: current directory)
- **-u, --url**: GitHub URL to analyze (alternative to path)
- **--format**: Output format (`table`, `json`, `yaml`)
- **--web**: Launch web-based demo interface
- **--no-browser**: Don't open browser automatically (for web mode)
- **--port**: Port for web server (default: 3030)
- **--cli**: Force CLI-only mode
- **--protocol**: Protocol to use for demo (`cli`, `http`, `mcp`, `all`)
- **--show-api**: Show API introspection information
- **--export**: Export format (`markdown`, `json`, `sarif`) - save analysis results
- **--target-nodes**: Target number of nodes for graph reduction (default: 50)
- **--max-edges**: Maximum edges before triggering PageRank pruning (default: 400)
- **--pagerank-iterations**: Number of PageRank iterations for importance scoring (default: 10)

#### Protocol Modes

The demo command supports multiple protocol interfaces:

1. **CLI Protocol** (`--protocol cli`): Direct command-line execution with formatted output
2. **HTTP Protocol** (`--protocol http`): REST API demonstration (placeholder)
3. **MCP Protocol** (`--protocol mcp`): JSON-RPC 2.0 Model Context Protocol interface
4. **All Protocols** (`--protocol all`): Compare behavior across all protocols

#### API Introspection

When using `--show-api`, the demo displays detailed information about how the tool translates high-level requests into internal commands:

- **CLI Mode**: Shows exact command-line invocation
- **MCP Mode**: Displays JSON-RPC request/response format
- **HTTP Mode**: Shows REST API endpoints and parameters

#### Configuration

The demo system supports configuration through `.paiml-display.yaml` file with hot-reload capabilities:

```yaml
version: "1.0"
panels:
  dependency:
    max_nodes: 20        # Maximum nodes in dependency graph
    max_edges: 60        # Maximum edges before reduction
    grouping: module     # Grouping strategy: module, directory, none
  complexity:
    threshold: 15        # Complexity threshold for highlighting
    max_items: 50        # Maximum items to display
  churn:
    days: 30            # Analysis period in days
    max_items: 20       # Maximum churn items to show
  context:
    include_ast: true   # Include AST analysis
    include_metrics: true # Include quality metrics
    max_file_size: 500000 # Maximum file size to analyze
export:
  formats: ["markdown", "json", "sarif"] # Available export formats
  include_metadata: true  # Include analysis metadata
  include_raw_data: false # Include raw analysis data
performance:
  cache_enabled: true   # Enable analysis caching
  cache_ttl: 3600      # Cache time-to-live in seconds
  parallel_workers: 4   # Number of parallel workers
```

#### Export Formats

The demo supports multiple export formats for analysis results:

1. **Markdown** (`--export markdown`): Human-readable report with Mermaid diagrams
2. **JSON** (`--export json`): Structured data for programmatic consumption
3. **SARIF** (`--export sarif`): Static Analysis Results Interchange Format for CI/CD

#### Examples

```bash
# Basic demo with table output
paiml-mcp-agent-toolkit demo

# Demo with specific protocol and API introspection
paiml-mcp-agent-toolkit demo --protocol cli --show-api

# Analyze GitHub repository with MCP protocol
paiml-mcp-agent-toolkit demo --url https://github.com/user/repo --protocol mcp

# Web-based interactive demo
paiml-mcp-agent-toolkit demo --web --port 8080

# Compare all protocols
paiml-mcp-agent-toolkit demo --protocol all --format json

# Export analysis results
paiml-mcp-agent-toolkit demo --export markdown -o analysis.md
paiml-mcp-agent-toolkit demo --export sarif -o results.sarif
```

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

##### `analyze dead-code`

Analyze unused code detection with confidence scoring.

**Arguments:**
- **-p, --project-path**: Project path to analyze (default: current directory)
- **--format**: Output format (`summary`, `detailed`, `json`, `sarif`)
- **-o, --output**: Output file path
- **--confidence-threshold**: Minimum confidence for dead code detection (0.0-1.0, default: 0.7)
- **--include-patterns**: File patterns to include in analysis
- **--exclude-patterns**: File patterns to exclude from analysis
- **--entry-points**: Specify entry points for analysis (e.g., `main.rs`, `lib.rs`)
- **--cross-reference**: Enable cross-language reference analysis

**Examples:**
```bash
# Basic dead code analysis
pmat analyze dead-code

# High-confidence detection only
pmat analyze dead-code \
  --confidence-threshold 0.9 \
  --format sarif \
  -o dead-code.sarif
```

##### `analyze duplicates`

**NEW**: Detect code duplicates using SIMD-accelerated MinHash algorithms.

**Arguments:**
- **--detection-type**: Type of detection (`exact`, `renamed`, `gapped`, `semantic`, `all`)
- **--threshold**: Similarity threshold for semantic clones (0.0-1.0, default: 0.85)
- **--gpu**: Use GPU acceleration if available
- **--perf**: Output performance metrics
- **--format**: Output format (`summary`, `detailed`, `json`, `sarif`)
- **-o, --output**: Output file path
- **--min-lines**: Minimum lines of code for duplicate detection (default: 5)

**Examples:**
```bash
# Fast structural duplicate detection
pmat analyze duplicates --detection-type exact

# Comprehensive semantic duplicate detection
pmat analyze duplicates --detection-type all --threshold 0.8

# GPU-accelerated analysis with performance metrics
pmat analyze duplicates --gpu --perf --format json
```

##### `analyze defect-probability`

**NEW**: ML-based defect prediction using feature vectors and confidence scoring.

**Arguments:**
- **--min-confidence**: Minimum confidence threshold (0.0-1.0, default: 0.7)
- **--explain**: Include feature importance breakdown
- **--sarif**: Output SARIF format for IDE integration
- **--format**: Output format (`summary`, `detailed`, `json`, `sarif`)
- **-o, --output**: Output file path

**Examples:**
```bash
# High-confidence defect predictions
pmat analyze defect-probability --min-confidence 0.8

# Detailed analysis with feature explanations
pmat analyze defect-probability --explain --format detailed

# IDE integration with SARIF output
pmat analyze defect-probability --sarif -o defects.sarif
```

##### `analyze comprehensive`

**NEW**: Multi-dimensional analysis combining all analysis types with parallel execution.

**Arguments:**
- **--format**: Output format (`summary`, `detailed`, `json`, `markdown`, `sarif`)
- **--include-duplicates**: Enable duplicate detection analysis
- **--include-dead-code**: Enable dead code analysis
- **--include-defects**: Enable defect prediction analysis
- **--include-complexity**: Enable complexity analysis
- **--include-tdg**: Enable TDG (Technical Debt Gradient) analysis
- **--confidence-threshold**: Minimum confidence threshold for predictions (default: 0.5)
- **--min-lines**: Minimum lines of code for analysis (default: 10)
- **--include**: Include file patterns (e.g., `**/*.rs`)
- **--exclude**: Exclude file patterns (e.g., `**/target/**`)
- **-o, --output**: Output file path
- **--perf**: Show performance metrics for each analysis component
- **--executive-summary**: Generate executive summary only (faster analysis)

**Performance Characteristics:**
- **Duplicate detection**: ~84ms for 200-file codebase
- **Dead code analysis**: ~7ms analysis time
- **Defect prediction**: ~45ms ML inference
- **Complexity analysis**: ~36ms processing
- **Total comprehensive**: ~143ms for full analysis

**Examples:**
```bash
# Basic comprehensive analysis
pmat analyze comprehensive

# Full analysis with all components and performance metrics
pmat analyze comprehensive \
  --include-duplicates \
  --include-dead-code \
  --include-defects \
  --include-complexity \
  --include-tdg \
  --perf \
  --format detailed

# Executive summary for quick assessment
pmat analyze comprehensive --executive-summary --format markdown
```

##### `analyze graph-metrics`

**NEW**: Vectorized graph analytics with PageRank and centrality computation.

**Arguments:**
- **--metrics**: Metrics to compute (`centrality`, `pagerank`, `clustering`, `components`, `all`)
- **--pagerank-seeds**: Personalized PageRank seed nodes
- **--graphml**: Export as GraphML format
- **--format**: Output format (`summary`, `detailed`, `json`)
- **-o, --output**: Output file path

**Examples:**
```bash
# Compute all graph metrics
pmat analyze graph-metrics --metrics all

# Personalized PageRank analysis
pmat analyze graph-metrics \
  --metrics pagerank \
  --pagerank-seeds main.rs,lib.rs \
  --format json
```

##### `analyze name-similarity`

**NEW**: Semantic name similarity using embeddings and phonetic matching.

**Arguments:**
- **query**: Name to search for
- **--top-k**: Number of results (default: 10)
- **--phonetic**: Include phonetic matches
- **--scope**: Search scope (`functions`, `types`, `variables`, `all`)
- **--format**: Output format (`summary`, `detailed`, `json`)

**Examples:**
```bash
# Find similar function names
pmat analyze name-similarity "calculateTotal" --scope functions

# Phonetic similarity search
pmat analyze name-similarity "proces" --phonetic --top-k 5
```

##### `analyze big-o`

**NEW**: Detect algorithmic complexity with confidence scores.

**Arguments:**
- **-p, --project-path**: Project path (default: current directory)
- **--min-complexity**: Minimum complexity to report (e.g., "O(n^2)")
- **--min-confidence**: Minimum confidence threshold (0.0-1.0)
- **--include-evidence**: Show detailed evidence
- **--format**: Output format (`summary`, `detailed`, `json`)
- **--top-k**: Limit results to top K functions

**Examples:**
```bash
# Basic Big-O analysis
pmat analyze big-o

# Find quadratic or worse complexity
pmat analyze big-o --min-complexity "O(n^2)" --format json
```

##### `analyze makefile-lint`

**NEW**: Lint Makefiles with 50+ quality rules.

**Arguments:**
- **--makefile**: Path to Makefile (default: ./Makefile)
- **--min-severity**: Minimum severity level (`info`, `warning`, `error`)
- **--format**: Output format (`human`, `json`, `sarif`, `checkmake`)
- **--pedantic**: Enable all rules including pedantic
- **--fix**: Apply auto-fixes
- **--config**: Configuration file path

**Examples:**
```bash
# Basic linting
pmat analyze makefile-lint

# Fix issues automatically
pmat analyze makefile-lint --fix

# CI/CD integration
pmat analyze makefile-lint --min-severity error --format sarif
```

##### `analyze proof-annotations`

**NEW**: Lightweight formal verification for code properties.

**Arguments:**
- **-p, --project-path**: Project path (default: current directory)
- **--property-type**: Filter by property type (`nullability`, `bounds`, `aliasing`)
- **--high-confidence-only**: Only show high confidence properties
- **--include-evidence**: Include detailed evidence
- **--format**: Output format (`summary`, `detailed`, `json`)

**Examples:**
```bash
# Basic provability analysis
pmat analyze proof-annotations

# Focus on null safety
pmat analyze proof-annotations --property-type nullability --high-confidence-only
```

##### `analyze incremental-coverage`

**NEW**: Analyze coverage changes since base branch.

**Arguments:**
- **-p, --project-path**: Project path (default: current directory)
- **-b, --base-branch**: Base branch for comparison (default: main)
- **--coverage-format**: Input format (`lcov`, `cobertura`, `jacoco`)
- **--min-coverage**: Minimum coverage for new code
- **--format**: Output format (`summary`, `detailed`, `json`)

**Examples:**
```bash
# Check coverage for PR
pmat analyze incremental-coverage --base-branch main

# Enforce minimum coverage
pmat analyze incremental-coverage --min-coverage 80.0 --fail-on-decrease
```

##### `analyze symbol-table`

**NEW**: Generate comprehensive symbol tables.

**Arguments:**
- **-p, --project-path**: Project path (default: current directory)
- **--include-private**: Include private symbols
- **--cross-reference**: Generate cross-references
- **--format**: Output format (`summary`, `json`, `ctags`)
- **-o, --output**: Output file path

**Examples:**
```bash
# Generate symbol table
pmat analyze symbol-table --format json -o symbols.json

# Generate ctags format
pmat analyze symbol-table --format ctags --include-private
```

### `refactor`

**NEW**: Automated refactoring engine with interactive and batch modes.

#### Subcommands

##### `refactor auto`

**AI-powered automated refactoring** that enforces extreme quality standards.

**Arguments:**
- **-p, --project-path**: Project path to refactor (default: current directory)
- **--max-iterations**: Maximum iterations to run (default: 10)
- **--quality-profile**: Quality profile (`standard`, `strict`, `extreme`) (default: extreme)
- **--format**: Output format (`summary`, `detailed`, `json`) (default: detailed)
- **--dry-run**: Show what would be done without making changes
- **--skip-compilation**: Skip compilation checks (faster but less safe)
- **--skip-tests**: Skip test execution (not recommended)
- **--checkpoint**: Checkpoint file for resumable refactoring
- **-v, --verbose**: Enable verbose output

**Quality Standards (Extreme Profile):**
- Test coverage ≥ 80% per file
- Cyclomatic complexity ≤ 10 (target: 5)
- Zero SATD (no TODO, FIXME, HACK)
- Zero lint violations (pedantic + nursery)

**Prioritization:**
1. Compilation errors (highest priority)
2. Lint violations (sorted by count)
3. High complexity functions (>10)
4. SATD items
5. Coverage gaps (<80%)

**Examples:**
```bash
# Run automated refactoring
pmat refactor auto

# Dry run with JSON output
pmat refactor auto --dry-run --format json

# Resume from checkpoint
pmat refactor auto --checkpoint refactor-state.json

# Limited iterations
pmat refactor auto --max-iterations 5 --verbose
```

##### `refactor interactive`

Interactive refactoring with step-by-step guidance.

**Arguments:**
- **-p, --project-path**: Project path (default: current directory)
- **--explain**: Explanation level (`minimal`, `normal`, `detailed`)
- **--checkpoint**: Checkpoint file for state persistence
- **--target-complexity**: Target complexity threshold
- **--steps**: Maximum steps to execute
- **--config**: Configuration file path

**Examples:**
```bash
# Start interactive session
pmat refactor interactive

# Limited steps with target
pmat refactor interactive --steps 5 --target-complexity 15
```

##### `refactor serve`

Batch refactoring server for large-scale operations.

**Arguments:**
- **--refactor-mode**: Mode (`batch`, `interactive`)
- **-c, --config**: JSON configuration file
- **-p, --project**: Project directory
- **--parallel**: Number of parallel workers
- **--memory-limit**: Memory limit in MB
- **--batch-size**: Files per batch
- **--priority**: Priority expression
- **--checkpoint-dir**: Checkpoint directory
- **--resume**: Resume from checkpoint
- **--auto-commit**: Auto-commit template
- **--max-runtime**: Maximum runtime in seconds

**Examples:**
```bash
# Batch refactoring
pmat refactor serve --config refactor.json

# Resume with auto-commit
pmat refactor serve --resume --auto-commit "refactor: {file}"
```

##### `refactor status`

Show current refactoring status.

**Arguments:**
- **--checkpoint**: Checkpoint file
- **--format**: Output format (`json`, `yaml`, `summary`)

##### `refactor resume`

Resume refactoring from checkpoint.

**Arguments:**
- **--checkpoint**: Checkpoint file
- **--steps**: Maximum steps
- **--explain**: Override explanation level

##### `refactor docs`

**NEW**: AI-assisted documentation cleanup and refactoring that enforces Zero Tolerance Quality Standards.

**Arguments:**
- **-p, --project-path**: Project path to analyze (default: current directory)
- **--dry-run**: Show what would be done without making changes
- **--auto-remove**: Automatically remove files without confirmation
- **--backup**: Create backup before removing files (default: true when auto-removing)
- **--backup-dir**: Directory for backups (default: .pmat-backup)
- **--format**: Output format (`summary`, `detailed`, `json`, `interactive`)
- **--include-docs**: Include docs directory in cleanup (default: true)
- **--include-root**: Include root directory files in cleanup (default: true)
- **--include-scripts**: Include scripts directory in cleanup (default: true)
- **--min-age-days**: Minimum file age in days before considering for removal (default: 0)
- **--temp-patterns**: Additional temporary file patterns to match
- **--status-patterns**: Additional status file patterns to match
- **--artifact-patterns**: Additional artifact patterns to match
- **--preservation-patterns**: Patterns for files to always preserve
- **--show-reasons**: Show detailed reasons for each file classification
- **--verbose**: Enable verbose output

**Pattern Categories:**

1. **Temporary Scripts**: Files matching patterns like `fix-*`, `test-*`, `temp-*`, `tmp-*`, `quick-*`
2. **Status Reports**: Files matching `*_STATUS.md`, `*_PROGRESS.md`, `*_COMPLETE.md`, `*_SUCCESS.md`
3. **Build Artifacts**: Generated files like `*.mmd`, `optimization_state.json`, `complexity_report.json`
4. **Outdated Documentation**: Files with names like `OLD_*`, `DEPRECATED_*`, `OBSOLETE_*`

**Interactive Mode Features:**
- Review each file before removal
- See file content preview
- Choose actions: keep, remove, skip
- Batch operations: keep all, remove all

**Examples:**
```bash
# Dry run to see what would be removed
pmat refactor docs --dry-run

# Interactive review mode
pmat refactor docs --format interactive

# Auto-remove with backup
pmat refactor docs --auto-remove --backup

# Only remove files older than 7 days
pmat refactor docs --min-age-days 7 --auto-remove

# Custom patterns with preservation
pmat refactor docs \
  --temp-patterns "cleanup-*,old-*" \
  --preservation-patterns "*.spec.md,*.design.md" \
  --show-reasons

# Verbose JSON output for automation
pmat refactor docs --format json --verbose --dry-run
```

### `quality-gate`

**NEW**: Comprehensive quality checks with configurable thresholds.

**Arguments:**
- **--complexity-threshold**: Maximum allowed complexity (default: 10)
- **--duplication-threshold**: Maximum duplication percentage (default: 5.0)
- **--coverage-threshold**: Minimum test coverage (default: 80.0)
- **--defect-threshold**: Maximum defect probability (default: 0.3)
- **--format**: Output format (`summary`, `detailed`, `json`, `sarif`, `junit`)
- **-o, --output**: Output file path
- **--fail-on-violation**: Exit with error code on quality gate failures

**Examples:**
```bash
# Basic quality gate with default thresholds
pmat quality-gate

# Strict quality gate for CI/CD
pmat quality-gate \
  --complexity-threshold 8 \
  --duplication-threshold 3.0 \
  --coverage-threshold 90.0 \
  --fail-on-violation \
  --format junit \
  -o quality-results.xml
```

##### `analyze assemblyscript`

**NEW in v0.26.2**: Analyze AssemblyScript source code with WebAssembly-specific metrics.

**Arguments:**
- **-p, --project-path**: Project path to analyze (default: current directory)
- **-f, --format**: Output format (`summary`, `full`, `json`, `sarif`)
- **--wasm-complexity**: Include WASM complexity analysis
- **--memory-analysis**: Memory analysis with pool optimization
- **--security**: Security validation checks
- **-o, --output**: Output file path
- **--timeout**: Maximum parsing time in seconds (default: 30)
- **--perf**: Show performance metrics

**Examples:**
```bash
# Basic AssemblyScript analysis
pmat analyze assemblyscript

# Full analysis with all features
pmat analyze assemblyscript --wasm-complexity --memory-analysis --security

# JSON output for tooling
pmat analyze assemblyscript --format json -o analysis.json
```

##### `analyze webassembly`

**NEW in v0.26.2**: Analyze WebAssembly binary and text formats.

**Arguments:**
- **-p, --project-path**: Project path to analyze (default: current directory)
- **-f, --format**: Output format (`summary`, `full`, `json`, `sarif`)
- **--include-binary**: Include binary WASM (.wasm) files (default: true)
- **--include-text**: Include text WASM (.wat) files (default: true)
- **--memory-analysis**: Memory usage analysis
- **--security**: Security validation
- **--complexity**: Complexity analysis
- **-o, --output**: Output file path
- **--perf**: Show performance metrics

**Examples:**
```bash
# Analyze all WebAssembly files
pmat analyze webassembly

# Only analyze binary WASM files
pmat analyze webassembly --include-binary --no-include-text

# Comprehensive analysis
pmat analyze webassembly --memory-analysis --security --complexity
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