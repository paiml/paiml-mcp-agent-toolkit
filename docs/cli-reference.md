# PMAT CLI Reference Guide

Complete reference for all PMAT command-line interface commands and options.

> **v2.8.0**: Toyota Way Complexity Excellence with unified MCP server architecture and integrated quality proxy ✅

## Table of Contents

- [Installation](#installation)
- [Global Options](#global-options)
- [Main Commands](#main-commands)
- [Analyze Commands](#analyze-commands)
- [Refactor Commands](#refactor-commands)
- [Enforce Commands](#enforce-commands)
- [Output Formats](#output-formats)
- [Environment Variables](#environment-variables)
- [Configuration](#configuration)
- [Examples](#examples)

## Installation

```bash
# Install from crates.io
cargo install pmat

# Quick install script
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh

# Verify installation
pmat --version
```

## Global Options

Available with all commands:

| Option | Description | Default |
|--------|-------------|---------|
| `--mode <MODE>` | Force CLI/MCP mode | Auto-detect |
| `--verbose`, `-v` | Enable verbose output | `false` |
| `--debug` | Enable debug output | `false` |
| `--trace` | Enable trace output | `false` |
| `--trace-filter <FILTER>` | Custom trace filter | None |
| `--help`, `-h` | Show help information | |
| `--version`, `-V` | Show version information | |

## Main Commands

### Template Management

#### `generate`
Generate single project template.

```bash
pmat generate [OPTIONS] <TEMPLATE_TYPE> <PATH>

Options:
  --project-name <NAME>    Project name for template variables
  --language <LANG>        Programming language hint
  --overwrite             Overwrite existing files
  --dry-run               Preview changes without writing

Examples:
  pmat generate makefile .
  pmat generate readme ./my-project --project-name "My App"
  pmat generate gitignore . --language rust
```

#### `scaffold`
Create complete project structure with templates.

```bash
pmat scaffold [OPTIONS] <PROJECT_TYPE> <PATH>

Options:
  --features <FEATURES>    Comma-separated feature list
  --license <LICENSE>      License type (MIT, Apache-2.0, etc.)
  --git-init              Initialize git repository
  --no-examples           Skip example files

Examples:
  pmat scaffold rust ./new-project
  pmat scaffold typescript ./web-app --features "testing,linting"
```

#### `list`
List available templates.

```bash
pmat list [OPTIONS]

Options:
  --filter <TYPE>         Filter by template type
  --format <FORMAT>       Output format (table, json, yaml)

Examples:
  pmat list
  pmat list --filter makefile
  pmat list --format json
```

#### `search`
Search templates by keyword.

```bash
pmat search [OPTIONS] <QUERY>

Options:
  --limit <NUM>           Maximum results to return (default: 10)
  --include-deprecated    Include deprecated templates

Examples:
  pmat search "rust testing"
  pmat search dockerfile --limit 5
```

#### `validate`
Validate template parameters.

```bash
pmat validate [OPTIONS] <TEMPLATE> [PARAMS...]

Options:
  --strict               Strict validation mode
  --format <FORMAT>      Output format for validation report

Examples:
  pmat validate makefile project_name=myapp
  pmat validate readme --strict
```

### Analysis and Context

#### `context`
Generate comprehensive project context for AI understanding.

```bash
pmat context [OPTIONS]

Options:
  -t, --toolchain <TOOLCHAIN>    Target toolchain (auto-detected if not specified)
  -p, --project-path <PATH>      Project path to analyze (default: .)
  -o, --output <FILE>            Output file path
  --format <FORMAT>              Output format (markdown, json, yaml) (default: markdown)
  --include-large-files          Include large files (>500KB) that are normally skipped
  --skip-expensive-metrics       Skip expensive metrics (TDG, complexity analysis) for faster execution

Examples:
  pmat context                                   # Analyze current directory
  pmat context --format json                     # JSON output
  pmat context -p ./src --output context.md      # Analyze specific path
  pmat context --skip-expensive-metrics          # Fast mode for large projects
  pmat context --include-large-files             # Include large files in analysis
```

#### `demo`
Interactive demonstration of analysis capabilities.

```bash
pmat demo [OPTIONS] [PATH]

Options:
  --mode <MODE>              Demo mode (web, tui, cli, all)
  --port <PORT>              Web server port (default: random)
  --host <HOST>              Web server host (default: 127.0.0.1)
  --no-browser               Don't open browser for web mode
  --protocol <PROTOCOL>      Protocol to demonstrate (cli, http, mcp, tui)
  --repo <URL>               Clone and analyze repository
  --target-nodes <NUM>       Target graph nodes for complexity reduction
  --centrality-threshold <N> Centrality threshold for graph filtering
  --merge-threshold <N>      Node merge threshold
  --skip-vendor              Skip vendor/node_modules directories
  --max-line-length <N>      Maximum line length for analysis

Examples:
  pmat demo                              # Interactive web demo
  pmat demo --mode tui                   # Terminal UI mode
  pmat demo --mode cli ./src             # CLI mode analysis
  pmat demo --repo https://github.com/user/repo  # Clone and demo
  pmat demo --no-browser --port 8080     # Web mode without browser
```

#### `quality-gate`
Run quality gate checks against project standards.

```bash
pmat quality-gate [OPTIONS] <PATH>

Options:
  --profile <PROFILE>        Quality profile (standard, strict, extreme)
  --file <FILE>              Check specific file only
  --threshold <NUM>          Complexity threshold override
  --format <FORMAT>          Output format (table, json, junit, sarif)
  --fail-on-violations       Exit with error on violations
  --config <FILE>            Custom quality gate configuration

Examples:
  pmat quality-gate . --profile strict
  pmat quality-gate src/main.rs --file
  pmat quality-gate . --format junit --fail-on-violations
```

#### `report`
Generate enhanced analysis reports.

```bash
pmat report [OPTIONS] <PATH>

Options:
  --output <FILE>            Output file path
  --format <FORMAT>          Report format (markdown, html, pdf, json)
  --include <TYPES>          Analysis types to include (comma-separated)
  --template <TEMPLATE>      Report template
  --include-visualizations   Include graphs and charts
  --summary-only             Generate summary report only

Examples:
  pmat report . --format html --output report.html
  pmat report ./src --include "complexity,tdg,churn"
  pmat report . --format pdf --include-visualizations
```

#### `serve`
Start HTTP API server.

```bash
pmat serve [OPTIONS]

Options:
  --port <PORT>              Server port (default: 8080)
  --host <HOST>              Server host (default: 127.0.0.1)
  --cors                     Enable CORS headers
  --auth-token <TOKEN>       Required authentication token
  --config <FILE>            Server configuration file

Examples:
  pmat serve --port 3000
  pmat serve --host 0.0.0.0 --cors
  pmat serve --auth-token secret123
```

#### `diagnose`
Run comprehensive self-diagnostics.

```bash
pmat diagnose [OPTIONS]

Options:
  --output <FILE>            Save diagnostic report to file
  --include-env              Include environment variables
  --check-dependencies       Check external dependencies
  --verbose                  Detailed diagnostic output

Examples:
  pmat diagnose
  pmat diagnose --output diagnostics.json --verbose
  pmat diagnose --check-dependencies
```

## Analyze Commands

All analyze commands support these common options:

| Option | Description | Default |
|--------|-------------|---------|
| `--format <FORMAT>` | Output format | `table` |
| `--output <FILE>` | Output file path | stdout |
| `--include <PATTERNS>` | Include patterns | All files |
| `--exclude <PATTERNS>` | Exclude patterns | Default excludes |
| `--parallel <NUM>` | Number of parallel workers | CPU cores |
| `--quiet` | Suppress progress output | `false` |

### Core Analysis

#### `analyze complexity`
Analyze code complexity metrics with MCP tool composition support.

```bash
pmat analyze complexity [OPTIONS] <PATH>

Options:
  --file <FILE>              Analyze single file (conflicts with --files)
  --files <FILES>            Analyze specific files (comma-separated, for MCP composition)
  --max-cyclomatic <NUM>     Maximum cyclomatic complexity (default: 20)
  --max-cognitive <NUM>      Maximum cognitive complexity (default: 15)
  --functions-only           Only analyze functions (not files)
  --top-files <NUM>          Show top N most complex files
  --threshold <NUM>          Complexity threshold for reporting
  --profile <PROFILE>        Quality profile (standard, strict, extreme)
  --fail-on-violation        Exit with code 1 if violations found (CI/CD mode)

Examples:
  pmat analyze complexity .
  pmat analyze complexity ./src --max-cyclomatic 15 --top-files 10
  pmat analyze complexity . --format json --max-cognitive 10
  
  # CI/CD Integration
  pmat analyze complexity . --max-cyclomatic 15 --fail-on-violation
  pmat analyze complexity . --max-cognitive 10 --fail-on-violation --format json
  
  # MCP Tool Composition
  pmat analyze complexity . --files src/complex.rs,src/legacy.rs --format json
  pmat analyze complexity . --file src/hotspot.rs
```

#### `analyze churn`
Analyze code change frequency from git history.

```bash
pmat analyze churn [OPTIONS] <PATH>

Options:
  --days <NUM>               Days to analyze (default: 30)
  --authors                  Include author information
  --threshold <NUM>          Minimum changes threshold
  --exclude-merges           Exclude merge commits
  --branch <BRANCH>          Analyze specific branch

Examples:
  pmat analyze churn . --days 90
  pmat analyze churn ./src --authors --threshold 5
  pmat analyze churn . --exclude-merges --format csv
```

#### `analyze dag`
Generate dependency analysis graphs.

```bash
pmat analyze dag [OPTIONS] <PATH>

Options:
  --output-format <FORMAT>   Graph format (mermaid, dot, json, svg)
  --max-depth <NUM>          Maximum dependency depth
  --filter-external          Filter external dependencies
  --include-types            Include type dependencies
  --layout <LAYOUT>          Graph layout (td, lr, bt, rl)

Examples:
  pmat analyze dag . --output-format mermaid
  pmat analyze dag ./src --max-depth 3 --filter-external
  pmat analyze dag . --output-format svg --output deps.svg
```

#### `analyze dead-code`
Detect unused and unreachable code.

```bash
pmat analyze dead-code [OPTIONS] <PATH>

Options:
  --top-files <NUM>          Show top N files by dead code (default: show all)
  --include-unreachable      Include unreachable code detection
  --min-dead-lines <NUM>     Minimum dead lines to report (default: 5)
  --include-tests            Include test files in analysis
  --fail-on-violation        Exit with code 1 if violations found (CI/CD mode)
  --max-percentage <NUM>     Maximum dead code percentage allowed (default: 15.0)

Examples:
  pmat analyze dead-code .
  pmat analyze dead-code ./src --top-files 10 --include-unreachable
  pmat analyze dead-code . --min-dead-lines 3 --format json
  
  # CI/CD Integration
  pmat analyze dead-code . --max-percentage 10.0 --fail-on-violation
  pmat analyze dead-code . --max-percentage 5.0 --fail-on-violation --format json
```

### Advanced Analysis

#### `analyze satd`
Detect Self-Admitted Technical Debt in comments.

```bash
pmat analyze satd [OPTIONS] <PATH>

Options:
  --severity <LEVEL>         Filter by severity (low, medium, high, critical)
  --critical-only            Show only critical SATD
  --include-tests            Include test files in analysis
  --strict                   Use strict detection mode (catches more patterns)
  --evolution                Track SATD evolution over time
  --days <NUM>               Days to look back for evolution (default: 30)
  --metrics                  Include detailed metrics
  --top-files <NUM>          Show top N files by SATD count (0 = all)
  --fail-on-violation        Exit with code 1 if ANY SATD found (CI/CD mode)

Examples:
  pmat analyze satd .
  pmat analyze satd . --strict --format json
  pmat analyze satd ./src --critical-only
  pmat analyze satd . --evolution --days 60
  
  # CI/CD Integration (zero tolerance)
  pmat analyze satd . --strict --fail-on-violation
  pmat analyze satd . --critical-only --fail-on-violation --format json
```

#### `analyze deep-context`
Generate comprehensive deep context analysis with defect detection.

```bash
pmat analyze deep-context [OPTIONS]

Options:
  -p, --project-path <PATH>      Project path to analyze (default: .)
  -o, --output <FILE>            Output file path
  --format <FORMAT>              Output format (markdown, json, sarif) (default: markdown)
  --full                         Enable full detailed report (default is terse)
  --include <LIST>               Comma-separated list of analyses to include
  --exclude <LIST>               Comma-separated list of analyses to exclude
  --period-days <NUM>            Days of history for git analysis (default: 30)
  --dag-type <TYPE>              DAG type for dependency analysis (call-graph, import-graph, inheritance, full-dependency) (default: call-graph)
  --max-depth <NUM>              Maximum directory traversal depth
  --include-pattern <PATTERNS>   Include file patterns (can be specified multiple times)
  --exclude-pattern <PATTERNS>   Exclude file patterns (can be specified multiple times)
  --cache-strategy <STRATEGY>    Cache usage strategy (normal, force-refresh, offline) (default: normal)
  --parallel                     Enable parallel processing
  --verbose                      Enable verbose logging
  --top-files <NUM>              Number of top files to show in summary (default: 10)

Examples:
  pmat analyze deep-context                          # Basic analysis of current directory
  pmat analyze deep-context --full                   # Full detailed analysis
  pmat analyze deep-context --format json            # JSON output for tool integration
  pmat analyze deep-context --output report.md       # Save to file
  pmat analyze deep-context --period-days 90         # Analyze 90 days of git history
  pmat analyze deep-context --include-pattern "*.rs" # Only analyze Rust files
  pmat analyze deep-context --exclude-pattern "**/tests/**" # Exclude test directories
  pmat analyze deep-context --top-files 20           # Show top 20 files by complexity
```

#### `analyze tdg`
Technical Debt Gradient analysis.

```bash
pmat analyze tdg [OPTIONS] <PATH>

Options:
  --include-predictions      Include ML predictions
  --gradient-threshold <NUM> TDG score threshold
  --detailed                 Include detailed breakdown
  --baseline <REF>           Git reference for baseline comparison

Examples:
  pmat analyze tdg .
  pmat analyze tdg . --include-predictions --detailed
  pmat analyze tdg . --baseline main --format json
```

#### `analyze lint-hotspot`
Find the file with highest defect density (lint violations per line).

```bash
pmat analyze lint-hotspot [OPTIONS] [PATH]

Options:
  --file <FILE>              Analyze a specific file instead of finding the hotspot
  --format <FORMAT>          Output format (summary, detailed, json, sarif, enforcement-json)
  --max-density <NUM>        Maximum allowed defect density (default: 5.0 violations/100 lines)
  --clippy-flags <FLAGS>     Custom clippy flags (default: extreme quality mode)
  --top-files <NUM>          Number of top files to show by density (0 = all, default: 10)
  --enforce                  Enforce quality standards (exit non-zero if violations found)
  --dry-run                  Show what would be fixed without making changes
  --min-confidence <NUM>     Minimum confidence for automated fixes (0.0-1.0, default: 0.8)
  --enforcement-metadata     Include enforcement metadata in output

Examples:
  pmat analyze lint-hotspot                                    # Find hotspot in current directory
  pmat analyze lint-hotspot ./src --top-files 5               # Show top 5 files by defect density
  pmat analyze lint-hotspot --file src/main.rs --format detailed  # Analyze specific file
  pmat analyze lint-hotspot . --enforce --max-density 3.0     # Enforce stricter quality gate
  pmat analyze lint-hotspot . --format json                   # Machine-readable output
```

#### `analyze makefile`
Analyze Makefile quality and best practices.

```bash
pmat analyze makefile [OPTIONS] <PATH>

Options:
  --strict                   Use strict analysis mode
  --check-portability        Check POSIX portability
  --suggest-improvements     Include improvement suggestions

Examples:
  pmat analyze makefile .
  pmat analyze makefile ./build --strict --suggest-improvements
```

#### `analyze provability`
Abstract interpretation and formal verification analysis.

```bash
pmat analyze provability [OPTIONS] <PATH>

Options:
  --verification-level <LVL> Verification level (basic, advanced)
  --timeout <SECONDS>        Analysis timeout
  --include-proofs           Include proof annotations

Examples:
  pmat analyze provability . --verification-level advanced
  pmat analyze provability ./src --include-proofs --timeout 300
```

### Specialized Analysis

#### `analyze duplicates`
Advanced duplicate code detection.

```bash
pmat analyze duplicates [OPTIONS] <PATH>

Options:
  --algorithm <ALG>          Algorithm (exact, fuzzy, semantic, all)
  --min-lines <NUM>          Minimum duplicate block size (default: 6)
  --similarity <NUM>         Similarity threshold (0.0-1.0)
  --cross-language           Detect cross-language duplicates

Examples:
  pmat analyze duplicates . --algorithm semantic
  pmat analyze duplicates ./src --min-lines 10 --similarity 0.8
```

#### `analyze defect-prediction`
ML-based defect probability analysis.

```bash
pmat analyze defect-prediction [OPTIONS] <PATH>

Options:
  --model <MODEL>            ML model (default, advanced, custom)
  --confidence <NUM>         Minimum confidence threshold
  --include-features         Include feature importance
  --training-data <FILE>     Custom training data

Examples:
  pmat analyze defect-prediction . --model advanced
  pmat analyze defect-prediction ./src --include-features
```

#### `analyze comprehensive`
Multi-dimensional analysis combining all analysis types with MCP tool composition support.

```bash
pmat analyze comprehensive [OPTIONS] <PATH>

Options:
  --file <FILE>              Single file to analyze (conflicts with --files)
  --files <FILES>            Multiple files to analyze (comma-separated, for MCP composition)
  --include-all              Include all analysis types
  --quick                    Quick analysis mode
  --depth <NUM>              Analysis depth level (1-3)
  --report-format <FORMAT>   Comprehensive report format
  --confidence-threshold <NUM> Minimum confidence threshold (0.0-1.0)
  --include-duplicates       Include duplicate detection
  --include-dead-code        Include dead code analysis
  --include-defects          Include defect prediction
  --include-complexity       Include complexity metrics
  --include-tdg              Include Technical Debt Gradient

Examples:
  pmat analyze comprehensive . --include-all
  pmat analyze comprehensive ./src --quick --format html
  pmat analyze comprehensive . --file src/main.rs --format json
  pmat analyze comprehensive . --file lib/parser.rs --confidence-threshold 0.8
  
  # MCP Tool Composition (NEW)
  pmat analyze comprehensive . --files src/complex.rs,src/legacy.rs --include-all
  pmat analyze comprehensive . --files hotspot1.rs,hotspot2.rs --format json
```

### Graph and Network Analysis

#### `analyze graph-metrics`
Advanced graph centrality and network analysis.

```bash
pmat analyze graph-metrics [OPTIONS] <PATH>

Options:
  --metrics <METRICS>        Metrics to compute (pagerank,betweenness,closeness,degree)
  --iterations <NUM>         PageRank iterations (default: 100)
  --dampening <NUM>          PageRank dampening factor (default: 0.85)
  --normalize                Normalize metric values

Examples:
  pmat analyze graph-metrics . --metrics pagerank,betweenness
  pmat analyze graph-metrics ./src --normalize --format json
```

#### `analyze name-similarity`
Name similarity analysis with semantic embeddings.

```bash
pmat analyze name-similarity [OPTIONS] <PATH>

Options:
  --threshold <NUM>          Similarity threshold (0.0-1.0)
  --algorithm <ALG>          Similarity algorithm (levenshtein, semantic, phonetic)
  --include-suggestions      Include naming suggestions
  --cross-module             Check similarity across modules

Examples:
  pmat analyze name-similarity . --threshold 0.7
  pmat analyze name-similarity ./src --algorithm semantic --include-suggestions
```

### Code Analysis

#### `analyze symbol-table`
Symbol analysis with cross-references.

```bash
pmat analyze symbol-table [OPTIONS] <PATH>

Options:
  --include-cross-refs       Include cross-reference information
  --scope <SCOPE>            Analysis scope (local, module, global)
  --export-format <FORMAT>   Export format (json, csv, graphml)
  --unused-only              Only show unused symbols

Examples:
  pmat analyze symbol-table . --include-cross-refs
  pmat analyze symbol-table ./src --scope global --unused-only
```

#### `analyze big-o`
Algorithmic complexity analysis.

```bash
pmat analyze big-o [OPTIONS] <PATH>

Options:
  --include-worst-case       Include worst-case analysis
  --analysis-depth <DEPTH>   Analysis depth (shallow, deep)
  --timeout <SECONDS>        Analysis timeout per function
  --confidence <NUM>         Minimum confidence for complexity assessment

Examples:
  pmat analyze big-o . --include-worst-case
  pmat analyze big-o ./src --analysis-depth deep --timeout 60
```

### Language-Specific Analysis

#### `analyze assemblyscript`
AssemblyScript-specific code analysis.

```bash
pmat analyze assemblyscript [OPTIONS] <PATH>

Options:
  --optimization-level <LVL> Optimization level (O0, O1, O2, O3)
  --target <TARGET>          Compilation target
  --include-wasm-analysis    Include WebAssembly analysis
  --performance-hints        Include performance optimization hints

Examples:
  pmat analyze assemblyscript . --optimization-level O2
  pmat analyze assemblyscript ./src --include-wasm-analysis
```

#### `analyze webassembly`
WebAssembly binary and text format analysis.

```bash
pmat analyze webassembly [OPTIONS] <PATH>

Options:
  --format <FORMAT>          WASM format (binary, text, auto)
  --include-imports          Include import analysis
  --include-exports          Include export analysis
  --memory-analysis          Analyze memory usage patterns
  --function-metrics         Include per-function metrics

Examples:
  pmat analyze webassembly . --format binary --include-imports
  pmat analyze webassembly ./dist --memory-analysis --function-metrics
```

### Performance Analysis

#### `analyze incremental-coverage`
Incremental coverage analysis with intelligent caching.

```bash
pmat analyze incremental-coverage [OPTIONS] <PATH>

Options:
  --baseline <REF>           Git reference for baseline
  --cache-enabled            Enable intelligent caching
  --include-branch-coverage  Include branch coverage
  --threshold <NUM>          Coverage threshold percentage

Examples:
  pmat analyze incremental-coverage . --baseline main
  pmat analyze incremental-coverage ./src --cache-enabled --threshold 80
```

#### `analyze proof-annotations`
Collect and analyze formal proof annotations.

```bash
pmat analyze proof-annotations [OPTIONS] <PATH>

Options:
  --annotation-types <TYPES> Annotation types to collect
  --verify-proofs            Attempt proof verification
  --include-statistics       Include proof statistics
  --export-format <FORMAT>   Export format for proofs

Examples:
  pmat analyze proof-annotations . --verify-proofs
  pmat analyze proof-annotations ./src --annotation-types "requires,ensures,invariant"
```

## Vectorized/SIMD Analysis Commands

High-performance analysis using SIMD instructions and parallel processing:

#### `analyze duplicates-vectorized`
SIMD-accelerated duplicate detection.

```bash
pmat analyze duplicates-vectorized [OPTIONS] <PATH>

Options:
  --simd-level <LEVEL>       SIMD instruction set (SSE2, AVX, AVX2, AVX512)
  --chunk-size <NUM>         Processing chunk size
  --parallel-factor <NUM>    Parallelization factor

Examples:
  pmat analyze duplicates-vectorized . --simd-level AVX2
```

#### `analyze graph-metrics-vectorized`
Vectorized graph analysis with parallel processing.

```bash
pmat analyze graph-metrics-vectorized [OPTIONS] <PATH>

Examples:
  pmat analyze graph-metrics-vectorized . --parallel-factor 8
```

#### `analyze name-similarity-vectorized`
SIMD-based name similarity computation.

```bash
pmat analyze name-similarity-vectorized [OPTIONS] <PATH>

Examples:
  pmat analyze name-similarity-vectorized . --chunk-size 1000
```

#### `analyze symbol-table-vectorized`
Parallel symbol table analysis.

#### `analyze incremental-coverage-vectorized`
Vectorized coverage analysis.

#### `analyze big-o-vectorized`
Parallel Big-O complexity analysis.

#### `generate enhanced-report`
Generate comprehensive enhanced analysis reports.

```bash
pmat generate enhanced-report [OPTIONS] <PATH>

Options:
  --analysis-types <TYPES>   Analysis types to include
  --output-format <FORMAT>   Report format (markdown, html, pdf, json)
  --include-visualizations   Include charts and graphs
  --template <TEMPLATE>      Custom report template

Examples:
  pmat generate enhanced-report . --format html --include-visualizations
  pmat generate enhanced-report ./src --analysis-types "complexity,tdg,churn,duplicates"
```

## Refactor Commands

### `refactor auto`
AI-powered automated refactoring.

```bash
pmat refactor auto [OPTIONS] <PATH>

Options:
  --dry-run                  Preview changes without applying
  --max-iterations <NUM>     Maximum refactoring iterations
  --scope <SCOPE>            Refactoring scope (function, file, module)
  --format <FORMAT>          Output format for refactoring plan
  --github-url <URL>         GitHub URL for context
  --bug-report-path <PATH>   Bug report markdown file path

Examples:
  pmat refactor auto ./src --dry-run
  pmat refactor auto . --max-iterations 5 --scope module
  pmat refactor auto --github-url https://github.com/user/repo/issues/123
  pmat refactor auto --bug-report-path docs/bugs/issue-456.md
```

### `refactor serve`
Start batch processing server for refactoring.

```bash
pmat refactor serve [OPTIONS]

Options:
  --port <PORT>              Server port
  --workers <NUM>            Number of worker processes
  --queue-size <NUM>         Refactoring queue size

Examples:
  pmat refactor serve --port 9000 --workers 4
```

### `refactor interactive`
Interactive refactoring mode with TUI.

```bash
pmat refactor interactive [OPTIONS] <PATH>

Options:
  --preview-mode             Preview mode only
  --backup                   Create backups before refactoring

Examples:
  pmat refactor interactive ./src --backup
```

### `refactor status`
Show current refactoring status and progress.

```bash
pmat refactor status [OPTIONS]

Options:
  --job-id <ID>              Specific job ID to check
  --verbose                  Detailed status information

Examples:
  pmat refactor status
  pmat refactor status --job-id abc123 --verbose
```

### `refactor resume`
Resume refactoring from checkpoint.

```bash
pmat refactor resume [OPTIONS] <CHECKPOINT>

Options:
  --force                    Force resume even if conflicts exist
  --verify                   Verify checkpoint integrity

Examples:
  pmat refactor resume .pmat/checkpoint-abc123
  pmat refactor resume ./backup/checkpoint.json --force
```

### `refactor docs`
Documentation cleanup and refactoring.

```bash
pmat refactor docs [OPTIONS] <PATH>

Options:
  --fix-links                Fix broken documentation links
  --update-examples          Update code examples
  --standardize-format       Standardize documentation format

Examples:
  pmat refactor docs ./docs --fix-links --update-examples
```

## Quality Proxy

**NEW in v2.1** - Intercept and validate AI-generated code before it's written to ensure quality standards.

The Quality Proxy acts as a gatekeeper between AI agents (like Claude Code, GitHub Copilot, etc.) and your codebase, enforcing quality standards on all generated code.

### Features

- **Real-time validation** - Check code quality before it's written
- **Three enforcement modes** - Strict (reject), Advisory (warn), Auto-Fix (refactor)
- **Comprehensive checks** - Complexity, SATD, documentation, lint violations
- **AI agent integration** - Works with MCP protocol and HTTP API
- **Automatic refactoring** - Fix issues automatically in Auto-Fix mode

### Usage via MCP

When using with AI agents through MCP:

```json
{
  "tool": "quality_proxy",
  "arguments": {
    "operation": "write",
    "file_path": "src/example.rs",
    "content": "// Code to validate",
    "mode": "strict",
    "quality_config": {
      "max_complexity": 20,
      "allow_satd": false,
      "require_docs": true,
      "auto_format": true
    }
  }
}
```

### Usage via Library

```rust
use pmat::services::quality_proxy::QualityProxyService;
use pmat::models::proxy::{ProxyRequest, ProxyOperation, ProxyMode, QualityConfig};

let service = QualityProxyService::new();
let request = ProxyRequest {
    operation: ProxyOperation::Write,
    file_path: "example.rs".to_string(),
    content: Some(code),
    mode: ProxyMode::Strict,
    quality_config: QualityConfig::default(),
};

let response = service.proxy_operation(request).await?;
```

### Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **Strict** | Reject code that doesn't meet standards | Production, CI/CD |
| **Advisory** | Warn but allow code through | Development, debugging |
| **Auto-Fix** | Automatically fix issues | Rapid prototyping |

### Quality Checks

1. **Complexity Analysis**
   - McCabe cyclomatic complexity
   - Cognitive complexity
   - Configurable thresholds (default: 20)

2. **SATD Detection**
   - TODO, FIXME, HACK comments
   - Incomplete implementations
   - Zero tolerance in strict mode

3. **Documentation**
   - Public API documentation
   - Function/struct/enum docs
   - Configurable requirement

4. **Lint Violations**
   - Clippy checks
   - Format validation
   - Style consistency

### Example Output

```json
{
  "status": "rejected",
  "quality_report": {
    "passed": false,
    "metrics": {
      "max_complexity": 35,
      "satd_count": 2,
      "lint_violations": 3
    },
    "violations": [
      {
        "type": "complexity",
        "severity": "error",
        "location": "src/main.rs:45",
        "message": "Function complexity 35 exceeds threshold 20",
        "suggestion": "Split into smaller functions"
      }
    ]
  }
}
```

### Demo

Run the interactive demo to see all features:

```bash
cargo run --example quality_proxy_demo
```

## Enforce Commands

### `enforce extreme`
Enforce extreme quality standards with zero tolerance.

```bash
pmat enforce extreme [OPTIONS] <PATH>

Options:
  --fix-automatically        Automatically fix violations when possible
  --fail-fast                Stop on first violation
  --report <FILE>            Generate enforcement report
  --exclude <PATTERNS>       Exclude patterns from enforcement

Examples:
  pmat enforce extreme . --fix-automatically
  pmat enforce extreme ./src --fail-fast --report violations.json
```

## Output Formats

PMAT supports extensive output formats across commands:

### General Formats
- `table` - Human-readable table format (default for most commands)
- `json` - Machine-readable JSON
- `yaml` - YAML format
- `csv` - Comma-separated values
- `markdown` - Markdown tables and text
- `html` - HTML format with styling
- `xml` - XML format

### Specialized Formats
- `sarif` - Static Analysis Results Interchange Format
- `junit` - JUnit XML test format
- `lcov` - LCOV coverage format
- `gcc` - GCC-style compiler output
- `enforcement-json` - Enforcement-specific JSON format
- `llm-optimized` - Optimized for LLM consumption
- `graphml` - GraphML format for graph data
- `dashboard` - Interactive dashboard format

### Graph Formats
- `mermaid` - Mermaid diagram syntax
- `dot` - Graphviz DOT format
- `svg` - Scalable Vector Graphics
- `png` - Portable Network Graphics
- `pdf` - Portable Document Format

## Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `RUST_LOG` | Logging level and filters | `info` |
| `MCP_VERSION` | Force MCP mode detection | Not set |
| `DOCS_RS` | Docs.rs build environment | Not set |
| `PMAT_CONFIG` | Configuration file path | `.pmat.toml` |
| `PMAT_CACHE_DIR` | Cache directory location | `~/.cache/pmat` |
| `PMAT_GPU_ENABLED` | Enable GPU acceleration | Not set |
| `PMAT_PARALLEL_WORKERS` | Default parallel workers | CPU cores |
| `PMAT_MAX_FILE_SIZE` | Maximum file size to analyze | `10MB` |

## Configuration

### Configuration File

PMAT looks for configuration in these locations (in order):
1. `--config <file>` command line option
2. `PMAT_CONFIG` environment variable
3. `.pmat.toml` in current directory
4. `~/.config/pmat/config.toml`
5. Built-in defaults

### Sample Configuration

```toml
# .pmat.toml
[analysis]
max_file_size = "10MB"
exclude_patterns = ["target/", "node_modules/", "*.min.js"]
include_hidden = false
follow_symlinks = false

[complexity]
max_cyclomatic = 10
max_cognitive = 15
profile = "strict"

[performance]
parallel_workers = 8
chunk_size = 1000
enable_simd = true
gpu_acceleration = false

[cache]
enabled = true
max_size = "1GB"
ttl_hours = 24
strategy = "lru"

[output]
default_format = "table"
color_output = true
progress_bars = true

[quality_gate]
fail_on_violations = true
complexity_threshold = 15
coverage_threshold = 80

[logging]
level = "info"
file = "pmat.log"
rotate = true
max_size = "100MB"
```

## CI/CD Integration

All analyze commands support `--fail-on-violation` for seamless CI/CD integration. When this flag is set, commands will exit with code 1 if violations exceed configured thresholds.

### Exit Codes
- **0**: Success - no violations found or within thresholds
- **1**: Failure - violations exceed configured thresholds

### GitHub Actions
```yaml
name: Code Quality
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install pmat
        run: cargo install pmat
        
      - name: Check Complexity
        run: |
          pmat analyze complexity \
            --max-cyclomatic 15 \
            --max-cognitive 10 \
            --fail-on-violation
            
      - name: Check Technical Debt  
        run: pmat analyze satd --strict --fail-on-violation
        
      - name: Check Dead Code
        run: |
          pmat analyze dead-code \
            --max-percentage 10.0 \
            --fail-on-violation
```

### GitLab CI
```yaml
quality-check:
  stage: test
  script:
    - cargo install pmat
    - pmat quality-gate --fail-on-violation --format json --output report.json
  artifacts:
    reports:
      codequality: report.json
```

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit
pmat analyze satd --strict --fail-on-violation || exit 1
pmat analyze complexity --max-cyclomatic 15 --fail-on-violation || exit 1
```

### Makefile Integration
```makefile
.PHONY: quality-check
quality-check:
	pmat analyze complexity --max-cyclomatic 15 --fail-on-violation
	pmat analyze satd --strict --fail-on-violation
	pmat analyze dead-code --max-percentage 10 --fail-on-violation
```

## Examples

### Basic Usage

```bash
# Analyze current directory complexity
pmat analyze complexity .

# Generate project context for AI
pmat context . --format markdown

# Run interactive demo
pmat demo

# Check quality gates
pmat quality-gate . --profile strict
```

### Advanced Analysis

```bash
# Comprehensive analysis with multiple metrics
pmat analyze comprehensive . --include-all --format html --output report.html

# Find duplicate code using semantic analysis
pmat analyze duplicates . --algorithm semantic --min-lines 8

# Analyze technical debt with ML predictions
pmat analyze tdg . --include-predictions --format json

# Generate dependency graph
pmat analyze dag . --output-format mermaid --max-depth 5
```

### Performance Analysis

```bash
# Use vectorized analysis for large codebases
pmat analyze duplicates-vectorized . --simd-level AVX2

# Parallel complexity analysis
pmat analyze complexity . --parallel 16

# Incremental coverage with caching
pmat analyze incremental-coverage . --baseline main --cache-enabled
```

### Integration Examples

```bash
# CI/CD Quality Gate
pmat quality-gate . --format junit --fail-on-violations > quality-report.xml

# Generate SARIF for GitHub Actions
pmat analyze complexity . --format sarif > complexity.sarif

# MCP mode for AI integration
pmat --mode mcp

# Export analysis for external tools
pmat analyze graph-metrics . --format graphml --output network.graphml
```

### Refactoring Workflows

```bash
# Automated refactoring with preview
pmat refactor auto ./src --dry-run --max-iterations 3

# Interactive refactoring with TUI
pmat refactor interactive ./src --backup

# Fix issues from GitHub issue
pmat refactor auto --github-url https://github.com/user/repo/issues/123

# Fix issues from bug report
pmat refactor auto --bug-report-path docs/bugs/memory-leak.md
```

## Tips and Best Practices

1. **Start with `demo`** - Use `pmat demo` to explore capabilities interactively
2. **Use `--dry-run`** - Always preview refactoring changes before applying
3. **Enable caching** - Use caching for repeated analysis of large codebases
4. **Parallel processing** - Increase `--parallel` for faster analysis on multi-core systems
5. **Quality profiles** - Use appropriate quality profiles (`standard`, `strict`, `extreme`)
6. **Output formats** - Choose appropriate output formats for your workflow
7. **Exclude patterns** - Use exclude patterns to skip generated or vendor code
8. **Environment variables** - Set `RUST_LOG=debug` for troubleshooting

## Getting Help

```bash
# General help
pmat --help

# Command-specific help
pmat analyze --help
pmat analyze complexity --help

# Version information
pmat --version

# Self-diagnostics
pmat diagnose --verbose
```