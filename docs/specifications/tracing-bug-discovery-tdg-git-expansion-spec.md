# Specification: Tracing, Bug Discovery, and TDG Git Enforcement Expansion

**Version**: 1.0.0
**Status**: DRAFT
**Date**: October 29, 2025
**Related**: Sprint 71+ roadmap

---

## Executive Summary

This specification defines three interconnected systems that extend PMAT's capabilities for tracing, bug discovery, and continuous quality tracking:

1. **Interactive Tracing & Debugging** - DAP-based debugging with time-travel capabilities
2. **Intelligent Bug Discovery** - ML-powered bug prediction using TDG and git signals
3. **TDG Git Kernel** - Compressed codebase "map" continuously tracked in git commits

**Inspiration**: Combines proven techniques from:
- **bashrs**: REPL modes, interactive analysis, comprehensive audit
- **ruchyruchy**: DAP debugging, time-travel replay, 10 quality analysis tools
- **PMAT**: TDG enforcement, deep context generation, MCP integration

**Value Proposition**: Transform PMAT into an intelligent development companion that:
- Traces code execution like a time machine
- Predicts bugs before they happen
- Maintains a living "map" of codebase quality in git

---

## Part 1: Interactive Tracing & Debugging

### 1.1 Overview

**Goal**: Enable developers to trace, debug, and understand code behavior interactively through:
- Debug Adapter Protocol (DAP) server integration
- Time-travel debugging with execution recording
- Multi-language debugging support (17+ languages)
- Interactive REPL for live analysis

**Architecture**:
```
┌─────────────────────────────────────────────────────┐
│         Debug Adapter Protocol (DAP) Server          │
├─────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │  Breakpoint  │  │  Execution   │  │ Time      │ │
│  │  Management  │  │  Control     │  │ Travel    │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
├─────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │  Parse Stack │  │  AST Visual. │  │ Type      │ │
│  │  Inspection  │  │              │  │ Errors    │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
├─────────────────────────────────────────────────────┤
│            Language Analyzer Layer (17+)             │
└─────────────────────────────────────────────────────┘
```

### 1.2 Core Features

#### Feature 1.1: Debug Adapter Protocol Server

**Command**: `pmat debug --serve`

**Capabilities**:
- Standard DAP server implementation (compatible with VS Code, vim, etc.)
- Multi-language support (Rust, Python, TypeScript, Go, Java, etc.)
- Breakpoint management (line, conditional, function entry/exit)
- Variable inspection with AST-aware formatting
- Call stack visualization with source context

**Example Usage**:
```bash
# Start DAP server
pmat debug --serve --port 5678

# In editor (VS Code launch.json):
{
  "type": "pmat-debug",
  "request": "launch",
  "program": "${workspaceFolder}/src/main.py",
  "stopOnEntry": false
}
```

**Implementation**:
- Integrate with existing tree-sitter parsers
- Leverage PMAT's AST analysis for intelligent breakpoints
- Use deep context for variable inspection
- Generate source maps for multi-language tracing

#### Feature 1.2: Time-Travel Debugging

**Command**: `pmat debug --time-travel <file>`

**Capabilities**:
- Execution recording with snapshots
- Rewind/forward through execution history
- Deterministic replay of program state
- Delta-based storage for efficiency

**Architecture**:
```rust
struct ExecutionSnapshot {
    timestamp: u64,
    variables: HashMap<String, Value>,
    call_stack: Vec<Frame>,
    source_location: Location,
    delta_from_previous: Vec<Change>
}

struct TimeTravel {
    snapshots: Vec<ExecutionSnapshot>,
    current_index: usize,
    replay_mode: ReplayMode
}
```

**Example Usage**:
```bash
# Record execution
pmat debug --record --output execution.pmat my_script.py

# Replay with time-travel
pmat debug --replay execution.pmat --interactive

# REPL commands:
debug> step forward 10     # Advance 10 steps
debug> step backward 5     # Rewind 5 steps
debug> goto 0:01:23.45    # Jump to timestamp
debug> diff 100 200       # Compare states
```

**Performance Targets**:
- Snapshot creation: <5ms per snapshot
- Replay speed: Real-time to 10x
- Storage: <1MB per 1000 snapshots (delta compression)

#### Feature 1.3: Interactive REPL for Code Analysis

**Command**: `pmat repl --mode trace`

**Modes** (inspired by bashrs):
1. **trace**: Trace execution of code snippets
2. **parse**: Parse code and visualize AST
3. **analyze**: Run TDG analysis on code
4. **debug**: Step-through debugging
5. **fix**: Suggest fixes for issues

**Example Session**:
```bash
$ pmat repl --mode trace
pmat [trace]> :mode                  # Show current mode
Current mode: trace - Trace code execution

pmat [trace]> :parse
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

✓ Parsed successfully!
AST:
  FunctionDef(fibonacci)
    - Parameter(n)
    - If Statement
      - Test: BinaryOp(n <= 1)
      - Body: Return(n)
      - Orelse: Return(BinaryOp(...))

pmat [trace]> :mode analyze
Switched to analyze mode

pmat [analyze]> fibonacci(10)
🔍 TDG Analysis:
  Complexity: High (recursive without memoization)
  Grade: C (O(2^n) complexity)
  Suggestion: Add @cache decorator for O(n) complexity

pmat [analyze]> :mode fix
Switched to fix mode

pmat [fix]> fibonacci(10)
✨ Suggested Fix:
from functools import cache

@cache
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

Improvement: O(2^n) → O(n), Grade: C → A
```

**Implementation**:
- Extend existing PMAT CLI framework
- Integrate tree-sitter for parsing
- Connect to TDG analyzer
- Provide mode-specific processing

### 1.3 Commands

#### Command: `pmat debug`

```bash
pmat debug [OPTIONS] <FILE>

OPTIONS:
  --serve             Start DAP server mode
  --port <PORT>       DAP server port (default: 5678)
  --time-travel       Enable time-travel debugging
  --record            Record execution to file
  --replay <FILE>     Replay recorded execution
  --interactive       Interactive replay with REPL
  --breakpoint <LOC>  Set breakpoint (line:col or function)
  --mode <MODE>       Debug mode (trace|parse|analyze|fix)

EXAMPLES:
  # Start DAP server
  pmat debug --serve --port 5678

  # Time-travel debug a Python script
  pmat debug --time-travel --record script.py

  # Replay with interactive controls
  pmat debug --replay execution.pmat --interactive

  # Set breakpoints and trace
  pmat debug --breakpoint main:10 --mode trace src/main.rs
```

#### Command: `pmat repl`

```bash
pmat repl [OPTIONS]

OPTIONS:
  --mode <MODE>       Initial mode (normal|trace|parse|analyze|debug|fix)
  --load <FILE>       Load file on startup
  --history <FILE>    History file location
  --no-history        Disable history
  --max-depth <N>     Max recursion depth (default: 1000)
  --timeout <SEC>     Command timeout (default: 120)

REPL COMMANDS:
  :mode [NAME]        Show/switch modes
  :parse <code>       Parse code and show AST
  :analyze <code>     Run TDG analysis
  :trace <code>       Trace execution
  :fix <code>         Suggest fixes
  :breakpoint <loc>   Set breakpoint
  :step [N]           Step N instructions
  :continue           Continue execution
  :vars               Show variables
  :history            Show command history
  :clear              Clear screen
  help                Show help
  quit                Exit REPL

EXAMPLES:
  # Start REPL in trace mode
  pmat repl --mode trace

  # Load file and analyze
  pmat repl --mode analyze --load src/complex.py
```

---

## Part 2: Intelligent Bug Discovery

### 2.1 Overview

**Goal**: Predict and find bugs before they cause problems using:
- ML-powered defect prediction (92% AUC-ROC)
- Code churn analysis (100% bug detection)
- Multi-signal correlation (TDG + git + AST)
- Fast search through codebase quality signals

**Inspiration from ruchyruchy**:
- QUALITY-003: ML Defect Prediction (100% bug detection)
- QUALITY-005: Code Churn Analysis (100% bug detection)
- QUALITY-006: Mutation Testing (83% bug detection)
- Combined: 85-95% bug prevention rate

### 2.2 Bug Discovery Signals

#### Signal 1: TDG Quality Scores

**Data**: Existing TDG baseline system

**Usage**: Files with grade < B are bug-prone
- Grade F: 5.2x more likely to have bugs
- Grade D: 3.1x more likely
- Grade C: 1.8x more likely

#### Signal 2: Code Churn Analysis

**Command**: `pmat bugs --churn`

**Analysis**:
```bash
# Analyze git history for hotspots
pmat bugs --churn --min-commits 10 --days 90

📊 Code Churn Analysis (Last 90 days)
════════════════════════════════════════════

High-Risk Files (>15 commits):
  1. src/parser.rs          18 commits   8 bugs   (0.44 bugs/commit) ⚠️
  2. src/type_checker.rs    16 commits   6 bugs   (0.38 bugs/commit) ⚠️
  3. src/codegen.rs         15 commits   4 bugs   (0.27 bugs/commit) ⚠️

Correlation: 0.85 (strong correlation between churn and bugs)

Recommendation:
  - Refactor src/parser.rs (highest bug density)
  - Add mutation testing to type_checker.rs
  - Review recent changes to codegen.rs
```

**Implementation**:
```rust
struct ChurnAnalysis {
    file_path: PathBuf,
    commit_count: usize,
    bug_count: usize,      // from git log --grep "fix\|bug"
    bug_density: f64,      // bugs/commit
    last_modified: SystemTime,
    contributors: HashSet<String>
}

fn analyze_churn(repo: &Repository, days: u64) -> Vec<ChurnAnalysis> {
    // 1. Get commits from last N days
    // 2. Count modifications per file
    // 3. Count bug-fix commits (grep for "fix"|"bug"|"hotfix")
    // 4. Calculate correlation
    // 5. Rank by bug density
}
```

#### Signal 3: ML Defect Prediction

**Command**: `pmat bugs --predict`

**Model**: Train on historical bugs from git

**Features** (24 metrics):
1. **Code metrics**: Complexity, LOC, function count
2. **TDG scores**: Structural, semantic, duplication, coupling
3. **Git metrics**: Churn, authors, recency
4. **AST metrics**: Nesting depth, cyclomatic complexity
5. **Context metrics**: Import count, dependency count

**Training**:
```bash
# Train model on historical data
pmat bugs train --repo . --output bug_model.pmat

🧠 Training ML Bug Predictor
═══════════════════════════════

Collecting training data:
  ✓ Analyzed 1,247 commits
  ✓ Found 156 bug-fix commits
  ✓ Extracted 24 features per file
  ✓ Built training set: 15,620 samples

Training model:
  ✓ Algorithm: Gradient Boosted Trees
  ✓ Features: 24 metrics
  ✓ Accuracy: 89.2%
  ✓ Precision: 0.87
  ✓ Recall: 0.91
  ✓ AUC-ROC: 0.92

Model saved: bug_model.pmat
```

**Prediction**:
```bash
# Predict bugs in current codebase
pmat bugs predict --model bug_model.pmat --threshold 0.7

🎯 Bug Prediction Results
═════════════════════════

High-Risk Files (p > 0.7):
  1. src/parser.rs           p=0.94  ⚠️ CRITICAL
     Reasons: High churn (18 commits), complexity (C.C. 47), low coverage (45%)

  2. src/type_checker.rs     p=0.89  ⚠️ HIGH
     Reasons: Recent bugs (3 in last month), high coupling (14 deps)

  3. src/optimizer.rs        p=0.76  ⚠️ ELEVATED
     Reasons: New file (< 30 days old), no tests, complex logic

Medium-Risk Files (0.5 < p < 0.7):
  - src/lexer.rs             p=0.65
  - src/semantic.rs          p=0.58

Recommendation:
  → Add tests to parser.rs (current: 45% coverage, target: 80%)
  → Refactor type_checker.rs (reduce coupling from 14 to <10)
  → Review optimizer.rs with senior developer
```

**Model Architecture**:
```rust
struct BugPredictor {
    model: GradientBoostedTrees,
    features: Vec<FeatureExtractor>,
    threshold: f64
}

struct FeatureSe {
    // Code metrics
    lines_of_code: u32,
    cyclomatic_complexity: u32,
    nesting_depth: u32,
    function_count: u32,

    // TDG scores
    tdg_score: f64,
    structural_complexity: f64,
    semantic_complexity: f64,
    duplication_ratio: f64,
    coupling_score: f64,

    // Git metrics
    commit_count: u32,
    bug_fix_count: u32,
    author_count: u32,
    days_since_last_change: u32,

    // AST metrics
    max_nesting: u32,
    import_count: u32,
    dependency_count: u32,

    // Context
    file_age_days: u32,
    test_coverage: f64,
    mutation_score: f64
}
```

#### Signal 4: Multi-Signal Correlation

**Command**: `pmat bugs --analyze-all`

```bash
pmat bugs --analyze-all --output bug_report.md

🔍 Comprehensive Bug Analysis
══════════════════════════════

Correlation Matrix:
                    TDG    Churn   ML-Pred  Mutation  Coverage
TDG Score          1.00    0.62     0.71     0.54      0.68
Code Churn         0.62    1.00     0.85     0.43      0.51
ML Prediction      0.71    0.85     1.00     0.61      0.74
Mutation Score     0.54    0.43     0.61     1.00      0.82
Test Coverage      0.68    0.51     0.74     0.82      1.00

Top Predictors:
  1. ML Prediction + Churn:     95% accuracy
  2. TDG + Coverage:            89% accuracy
  3. Mutation Score + Churn:    86% accuracy

Critical Files (3+ signals agree):
  ✗ src/parser.rs
    - TDG: F (34.2)
    - Churn: 18 commits (8 bugs)
    - ML: p=0.94 (critical)
    - Mutation: 42% (low)
    - Coverage: 45% (insufficient)

  ✗ src/type_checker.rs
    - TDG: D (52.1)
    - Churn: 16 commits (6 bugs)
    - ML: p=0.89 (high)
    - Mutation: 67% (moderate)
    - Coverage: 68% (moderate)

Recommendation Priority:
  1. CRITICAL: Refactor src/parser.rs (all 5 signals red)
  2. HIGH: Add tests to type_checker.rs (4/5 signals red)
  3. MEDIUM: Review optimizer.rs (3/5 signals yellow)
```

### 2.3 Fast Bug Search

#### Feature 2.1: Indexed Quality Search

**Command**: `pmat bugs search <QUERY>`

**Index Structure**:
```rust
struct QualityIndex {
    // Inverted index for fast lookup
    tdg_scores: BTreeMap<Grade, Vec<FileId>>,
    churn_hotspots: BTreeMap<u32, Vec<FileId>>,
    ml_predictions: BTreeMap<OrderedFloat<f64>, Vec<FileId>>,

    // Full-text search on file content
    content_index: TantivyIndex,

    // AST-based search
    ast_patterns: PatternIndex
}
```

**Query Examples**:
```bash
# Find files with low TDG scores
pmat bugs search "tdg:<C"
→ 23 files with grade C or lower

# Find high-churn files
pmat bugs search "churn:>10 AND days:30"
→ 7 files with >10 commits in last 30 days

# Combine signals
pmat bugs search "tdg:<C AND churn:>5 AND ml:>0.7"
→ 3 files matching all criteria

# AST pattern search
pmat bugs search "pattern:recursive AND coverage:<50%"
→ 5 recursive functions with low coverage

# Full-text + quality
pmat bugs search "TODO|FIXME AND tdg:<B"
→ 42 files with tech debt markers and low quality
```

**Performance Targets**:
- Index creation: <500ms for 10K files
- Query execution: <50ms for complex queries
- Index size: <1MB per 1K files

#### Feature 2.2: Semantic Bug Pattern Search

**Command**: `pmat bugs patterns <PATTERN>`

**Built-in Patterns**:
```yaml
patterns:
  - name: unhandled_error
    description: Error ignored without handling
    query: |
      (try_statement
        body: (_)*
        handlers: (except_clause
          type: (_)
          body: (pass_statement)))
    severity: high

  - name: recursive_no_base_case
    description: Recursive function without obvious base case
    query: |
      (function_definition
        name: (identifier) @func_name
        body: (block
          (return_statement
            (call
              function: (identifier) @recursive_call
              (#eq? @func_name @recursive_call)))))
    severity: critical

  - name: magic_numbers
    description: Hardcoded numbers without explanation
    query: |
      (integer) @num
      (#not-match? @num "^[0-1]$")
    severity: low
```

**Usage**:
```bash
# Search for specific pattern
pmat bugs patterns unhandled_error

Found 7 matches:
  src/api.py:45     except Exception: pass
  src/db.py:103     except: pass
  src/utils.py:234  except Error: pass

# Search all patterns
pmat bugs patterns --all --severity high

Critical Issues (2):
  - recursive_no_base_case: 2 matches

High Issues (5):
  - unhandled_error: 7 matches
  - unchecked_return: 3 matches

# Custom pattern
pmat bugs patterns --query '(function_definition (return_statement (none)))' --name empty_return
```

### 2.4 Commands

#### Command: `pmat bugs`

```bash
pmat bugs [COMMAND] [OPTIONS]

COMMANDS:
  train        Train ML bug prediction model
  predict      Predict bugs in codebase
  search       Fast search through quality signals
  patterns     Search for bug patterns
  churn        Analyze code churn hotspots
  analyze-all  Comprehensive multi-signal analysis

TRAIN OPTIONS:
  --repo <PATH>         Repository to analyze
  --output <FILE>       Model output file
  --features <LIST>     Feature selection
  --algorithm <ALG>     ML algorithm (gbt|rf|svm)

PREDICT OPTIONS:
  --model <FILE>        Trained model file
  --threshold <FLOAT>   Prediction threshold (0.0-1.0)
  --output <FORMAT>     Output format (text|json|md)

SEARCH OPTIONS:
  --query <QUERY>       Search query (DSL)
  --limit <N>           Max results (default: 100)
  --sort <FIELD>        Sort by field

CHURN OPTIONS:
  --days <N>            Analysis window (default: 90)
  --min-commits <N>     Minimum commits (default: 5)
  --include-merges      Include merge commits

EXAMPLES:
  # Train model
  pmat bugs train --repo . --output model.pmat

  # Predict bugs
  pmat bugs predict --model model.pmat --threshold 0.7

  # Search for risky files
  pmat bugs search "tdg:<C AND churn:>10"

  # Find bug patterns
  pmat bugs patterns unhandled_error

  # Analyze churn
  pmat bugs churn --days 30 --min-commits 10
```

---

## Part 3: TDG Git Kernel - Compressed Codebase Map

### 3.1 Overview

**Goal**: Maintain a continuously-updated, compressed "kernel" of codebase knowledge in git, serving as:
- **Map/TOC**: High-level structure of codebase
- **Quality Tracker**: TDG scores and trends over time
- **LLM Context**: Optimized for AI consumption via MCP
- **Offline Reference**: Like "offline Google Maps" for code

**Size Target**: <100KB for 10K files (1000:1 compression vs deep_context.md)

**Update Frequency**: Every git commit (via pre-commit/post-commit hooks)

### 3.2 Kernel Structure

#### File Format: `.pmat/kernel.json`

```json
{
  "version": "1.0.0",
  "generated_at": "2025-10-29T18:30:00Z",
  "git_commit": "ec4434cf",
  "stats": {
    "total_files": 1247,
    "total_lines": 156234,
    "languages": {
      "Rust": { "files": 892, "lines": 134567 },
      "Python": { "files": 234, "lines": 18432 },
      "TypeScript": { "files": 121, "lines": 3235 }
    }
  },

  "quality_summary": {
    "average_tdg": 78.3,
    "grade_distribution": {
      "A+": 234, "A": 456, "B": 342,
      "C": 156, "D": 45, "F": 14
    },
    "trend": "+2.3% from previous commit"
  },

  "structure": {
    "modules": [
      {
        "path": "server/src",
        "type": "module",
        "files": 234,
        "avg_tdg": 82.1,
        "exports": ["context", "tdg", "mutation", "cli"]
      }
    ],
    "entry_points": [
      "server/src/main.rs",
      "server/src/bin/pmat-agent.rs"
    ]
  },

  "critical_files": [
    {
      "path": "server/src/services/mutation/mod.rs",
      "role": "Core mutation testing orchestrator",
      "tdg": 88.5,
      "complexity": 23,
      "dependencies": 12,
      "importance": 0.94
    }
  ],

  "hotspots": [
    {
      "path": "server/src/cli/handlers/mutate.rs",
      "commits_last_30d": 18,
      "bugs_last_30d": 3,
      "tdg": 67.2,
      "risk_score": 0.89
    }
  ],

  "dependencies": {
    "internal": [
      { "from": "cli", "to": "services", "strength": 0.87 },
      { "from": "services", "to": "tdg", "strength": 0.65 }
    ],
    "external": [
      { "name": "serde", "version": "1.0", "usage_count": 456 }
    ]
  },

  "llm_context": {
    "summary": "PMAT is a zero-config AI context generation tool...",
    "capabilities": [
      "TDG quality scoring (17+ languages)",
      "Mutation testing (6 languages)",
      "Deep context generation for LLMs",
      "MCP server integration (19 tools)"
    ],
    "architecture": "CLI + MCP Server + Quality Analysis Pipeline"
  }
}
```

#### Compressed Format (Optional): `.pmat/kernel.bin`

**Binary encoding** for maximum compression:
- Use protocol buffers or MessagePack
- Delta encoding for commit-to-commit changes
- Achieves 5-10x additional compression

### 3.3 Kernel Generation

#### Command: `pmat kernel generate`

```bash
pmat kernel generate [OPTIONS]

OPTIONS:
  --output <PATH>       Output path (default: .pmat/kernel.json)
  --format <FMT>        Format: json|binary|both (default: json)
  --include <ITEMS>     Include: stats|structure|hotspots|deps|llm
  --compress            Enable gzip compression
  --verify              Verify against full analysis

EXAMPLES:
  # Generate default kernel
  pmat kernel generate

  # Generate with all features
  pmat kernel generate --include all --compress

  # Generate binary format
  pmat kernel generate --format binary --output .pmat/kernel.bin
```

#### Implementation:

```rust
struct CodebaseKernel {
    version: String,
    generated_at: SystemTime,
    git_commit: String,

    // High-level stats (tiny)
    stats: Stats,

    // Quality summary (compressed)
    quality: QualitySummary,

    // Structure map (essential paths only)
    structure: StructureMap,

    // Critical files (top 10% by importance)
    critical_files: Vec<CriticalFile>,

    // Hotspots (files needing attention)
    hotspots: Vec<Hotspot>,

    // Dependency graph (compressed)
    dependencies: DependencyGraph,

    // LLM-optimized context
    llm_context: LLMContext
}

impl CodebaseKernel {
    fn generate(config: &KernelConfig) -> Result<Self> {
        // 1. Analyze codebase with PMAT
        let analysis = analyze_codebase()?;

        // 2. Extract high-level stats
        let stats = extract_stats(&analysis);

        // 3. Compute quality summary
        let quality = compute_quality_summary(&analysis);

        // 4. Build structure map (module hierarchy)
        let structure = build_structure_map(&analysis);

        // 5. Identify critical files (PageRank algorithm)
        let critical_files = identify_critical_files(&analysis);

        // 6. Find hotspots (churn + bugs + TDG)
        let hotspots = find_hotspots(&analysis);

        // 7. Build dependency graph
        let dependencies = build_dependency_graph(&analysis);

        // 8. Generate LLM context
        let llm_context = generate_llm_context(&analysis);

        Ok(Self {
            version: "1.0.0".to_string(),
            generated_at: SystemTime::now(),
            git_commit: get_git_commit()?,
            stats,
            quality,
            structure,
            critical_files,
            hotspots,
            dependencies,
            llm_context
        })
    }

    fn compress(&self) -> Vec<u8> {
        // 1. Serialize to JSON
        let json = serde_json::to_string(self).unwrap();

        // 2. Gzip compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(json.as_bytes()).unwrap();
        encoder.finish().unwrap()
    }
}
```

### 3.4 Git Integration

#### Hook: Pre-Commit (Validation)

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Check if kernel needs update
if pmat kernel --check-stale; then
    echo "❌ Kernel is stale. Run: pmat kernel generate"
    exit 1
fi

# Validate kernel consistency
if ! pmat kernel --validate; then
    echo "❌ Kernel validation failed"
    exit 1
fi

exit 0
```

#### Hook: Post-Commit (Update)

```bash
#!/bin/sh
# .git/hooks/post-commit

# Auto-update kernel after successful commit
pmat kernel generate --quiet --compress

# Add to git if changed
if git diff --quiet .pmat/kernel.json; then
    # No changes, skip
    :
else
    git add .pmat/kernel.json .pmat/kernel.bin
    git commit --amend --no-edit --no-verify
fi
```

#### Command: `pmat kernel track`

```bash
pmat kernel track [OPTIONS]

Enable continuous kernel tracking in git repository.

OPTIONS:
  --install-hooks       Install pre-commit and post-commit hooks
  --auto-commit         Auto-commit kernel updates
  --format <FMT>        Track format: json|binary|both
  --baseline <FILE>     Initialize from baseline

EXAMPLES:
  # Install hooks for auto-tracking
  pmat kernel track --install-hooks --auto-commit

  # Initialize from existing baseline
  pmat kernel track --baseline .pmat/tdg-baseline.json
```

### 3.5 MCP Integration

#### Tool: `kernel_query`

**MCP Server**: Expose kernel via MCP for LLM consumption

```typescript
// MCP Tool Definition
{
  name: "kernel_query",
  description: "Query compressed codebase kernel for quick insights",
  inputSchema: {
    type: "object",
    properties: {
      query_type: {
        type: "string",
        enum: ["summary", "structure", "hotspots", "critical_files", "quality"]
      },
      filters: {
        type: "object",
        properties: {
          path_pattern: { type: "string" },
          min_tdg: { type: "number" },
          max_complexity: { type: "number" }
        }
      }
    },
    required: ["query_type"]
  }
}
```

**Usage**:
```typescript
// From Claude or other LLM via MCP
const kernel = await mcp.call_tool("kernel_query", {
  query_type: "hotspots",
  filters: {
    min_tdg: 70,
    max_complexity: 20
  }
});

// Returns:
{
  hotspots: [
    {
      path: "server/src/cli/handlers/mutate.rs",
      tdg: 67.2,
      complexity: 23,
      commits_30d: 18,
      risk_score: 0.89
    }
  ]
}
```

### 3.6 Kernel Diff

#### Command: `pmat kernel diff`

```bash
pmat kernel diff <COMMIT1> <COMMIT2>

Compare kernels across commits to see quality trends.

OPTIONS:
  --stat              Show summary statistics
  --hotspots          Show new/removed hotspots
  --critical          Show changes to critical files
  --format <FMT>      Output format: text|json|html

EXAMPLE OUTPUT:
$ pmat kernel diff HEAD~10 HEAD

Kernel Diff: v2.180.1 (10 commits ago) → v2.181.0 (now)
═══════════════════════════════════════════════════════

Quality Trend:
  Average TDG:  76.8 → 78.3  (+1.5) ✓
  Grade A+:     198 → 234    (+36)  ✓
  Grade F:      18 → 14      (-4)   ✓

New Critical Files:
  + server/src/services/mutation/cargo_mutants_wrapper.rs (tdg: 92.3)
  + server/src/services/mutation/json_parser.rs (tdg: 88.7)

New Hotspots:
  + server/src/cli/handlers/cargo_mutants_backend.rs (18 commits, 3 bugs)

Removed Hotspots:
  - server/tests/storage_backend_tests.rs (bug fixed)

Architecture Changes:
  + New module: mutation/cargo_mutants
  + 10 new exports
  + 17 new dependencies (12 internal, 5 external)
```

---

## Part 4: Integration & Workflows

### 4.1 Unified Workflow Example

**Scenario**: Developer working on new feature

```bash
# 1. Check quality before starting
$ pmat kernel query --type hotspots

🔥 Current Hotspots:
  - src/parser.rs (tdg: 45.2, risk: 0.94)
  - src/type_checker.rs (tdg: 67.1, risk: 0.78)

# 2. Predict where bugs might occur
$ pmat bugs predict --file src/parser.rs

⚠️ High Bug Risk (p=0.94)
Reasons:
  - 18 commits in last 30 days
  - 8 bugs fixed recently
  - Cyclomatic complexity: 47
  - Test coverage: 45%

# 3. Start debugging interactively
$ pmat repl --mode debug --load src/parser.rs

pmat [debug]> :breakpoint parse_expression:10
✓ Breakpoint set at parse_expression:10

pmat [debug]> :run test_input.txt
⏸ Paused at src/parser.rs:123

pmat [debug]> :vars
  tokens = [Token(INT, "42"), Token(PLUS, "+"), Token(INT, "10")]
  current = 0
  ast = None

pmat [debug]> :step 5
⏸ Paused at src/parser.rs:128

pmat [debug]> :analyze
🔍 TDG Analysis at current location:
  Function: parse_expression
  Complexity: High (C.C. 23)
  Grade: C
  Suggestion: Extract helper functions

# 4. Make changes with TDD
$ cargo test parser::test_expression_parsing
# ... fix bugs ...

# 5. Verify quality improved
$ pmat tdg check-quality --file src/parser.rs --min-grade B
✅ Quality check passed: Grade B (score: 82.1)

# 6. Commit with auto-kernel update
$ git add src/parser.rs
$ git commit -m "fix: Improve parser expression handling"

🔄 Auto-updating kernel...
✓ Kernel updated: .pmat/kernel.json
  - Average TDG: 78.3 → 79.1 (+0.8)
  - Hotspots: 3 → 2 (-1, parser.rs resolved)

[master abc1234] fix: Improve parser expression handling
 2 files changed, 45 insertions(+), 23 deletions(-)
```

### 4.2 CI/CD Integration

**GitHub Actions Workflow**:
```yaml
name: Quality Gates

on: [push, pull_request]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Full history for churn analysis

      - name: Install PMAT
        run: cargo install pmat

      - name: Check kernel staleness
        run: pmat kernel --check-stale

      - name: Run bug prediction
        run: |
          pmat bugs train --repo . --output model.pmat
          pmat bugs predict --model model.pmat --threshold 0.7 --fail-on-critical

      - name: Analyze code churn
        run: pmat bugs churn --days 30 --fail-if-hotspots

      - name: TDG regression check
        run: pmat tdg check-regression --fail-on-regression

      - name: Verify kernel consistency
        run: pmat kernel --validate

      - name: Generate quality report
        run: |
          pmat bugs analyze-all --output quality_report.md
          cat quality_report.md >> $GITHUB_STEP_SUMMARY
```

### 4.3 MCP Server Enhancement

**New MCP Tools**:
```typescript
{
  tools: [
    // Existing PMAT MCP tools (19 tools)
    ...existingTools,

    // New tracing & debugging tools
    {
      name: "debug_trace",
      description: "Trace execution of code segment",
      inputSchema: { /* ... */ }
    },
    {
      name: "time_travel_debug",
      description: "Debug with time-travel capabilities",
      inputSchema: { /* ... */ }
    },

    // New bug discovery tools
    {
      name: "predict_bugs",
      description: "Predict bugs using ML model",
      inputSchema: { /* ... */ }
    },
    {
      name: "search_quality",
      description: "Fast search through quality signals",
      inputSchema: { /* ... */ }
    },
    {
      name: "find_hotspots",
      description: "Find code churn hotspots",
      inputSchema: { /* ... */ }
    },

    // New kernel tools
    {
      name: "kernel_query",
      description: "Query compressed codebase kernel",
      inputSchema: { /* ... */ }
    },
    {
      name: "kernel_diff",
      description: "Compare kernels across commits",
      inputSchema: { /* ... */ }
    }
  ]
}
```

---

## Part 5: Implementation Plan

### 5.1 Phase 1: Interactive Tracing (Sprints 71-73)

**Sprint 71**: DAP Server Foundation
- TRACE-001: Implement DAP protocol server
- TRACE-002: Breakpoint management system
- TRACE-003: Variable inspection with AST awareness

**Sprint 72**: Time-Travel Debugging
- TRACE-004: Execution recording infrastructure
- TRACE-005: Snapshot management and delta storage
- TRACE-006: Replay engine with forward/backward navigation

**Sprint 73**: Interactive REPL
- TRACE-007: REPL framework with mode system
- TRACE-008: Integration with TDG analyzer
- TRACE-009: Fix suggestion engine

### 5.2 Phase 2: Bug Discovery (Sprints 74-76)

**Sprint 74**: ML Bug Prediction
- BUG-001: Feature extraction from codebase
- BUG-002: ML model training pipeline
- BUG-003: Prediction engine with confidence scores

**Sprint 75**: Code Churn Analysis
- BUG-004: Git history analysis
- BUG-005: Hotspot detection algorithm
- BUG-006: Correlation analysis with bugs

**Sprint 76**: Fast Search
- BUG-007: Quality signal indexing
- BUG-008: Query DSL implementation
- BUG-009: Semantic pattern search

### 5.3 Phase 3: TDG Git Kernel (Sprints 77-79)

**Sprint 77**: Kernel Generation
- KERNEL-001: Kernel structure definition
- KERNEL-002: Compression algorithm
- KERNEL-003: Generation pipeline

**Sprint 78**: Git Integration
- KERNEL-004: Pre-commit/post-commit hooks
- KERNEL-005: Kernel validation
- KERNEL-006: Auto-tracking system

**Sprint 79**: MCP Integration
- KERNEL-007: MCP tool implementation
- KERNEL-008: Kernel diff visualization
- KERNEL-009: LLM context optimization

### 5.4 Success Metrics

**Part 1 (Tracing)**:
- DAP server compatible with VS Code, vim, etc.
- Time-travel replay at real-time speed
- Snapshot creation <5ms
- REPL response time <100ms

**Part 2 (Bug Discovery)**:
- ML model AUC-ROC >0.90
- Churn analysis accuracy >95%
- Search query time <50ms
- Bug prevention rate >85%

**Part 3 (TDG Kernel)**:
- Kernel size <100KB for 10K files
- Generation time <500ms
- Git hook overhead <100ms
- MCP query response <50ms

---

## Part 6: Technical Architecture

### 6.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      PMAT CLI & MCP Server                   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌────────────────┐  ┌────────────────┐  ┌───────────────┐ │
│  │   Interactive   │  │  Bug Discovery  │  │  TDG Kernel   │ │
│  │    Tracing      │  │                │  │               │ │
│  └────────────────┘  └────────────────┘  └───────────────┘ │
│          │                   │                    │          │
│          │                   │                    │          │
├──────────┼───────────────────┼────────────────────┼─────────┤
│          │                   │                    │          │
│  ┌───────▼────┐    ┌────────▼─────┐    ┌────────▼──────┐  │
│  │ DAP Server │    │ ML Predictor  │    │   Kernel Gen  │  │
│  │ Time-Travel│    │ Churn Analyzer│    │   Git Hooks   │  │
│  │    REPL    │    │ Pattern Search│    │   MCP Tools   │  │
│  └────────────┘    └───────────────┘    └───────────────┘  │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│                    Shared Infrastructure                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  TDG Analyzer │ Tree-Sitter │ Git Integration │ AST  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Data Flow

```
Input Source
    │
    ├──► Interactive Tracing ──┬──► DAP Protocol ──► IDE
    │                          │
    │                          ├──► Time-Travel ──► Replay
    │                          │
    │                          └──► REPL ──► Terminal
    │
    ├──► Bug Discovery ────────┬──► ML Model ──► Predictions
    │                          │
    │                          ├──► Churn Analysis ──► Hotspots
    │                          │
    │                          └──► Pattern Search ──► Matches
    │
    └──► TDG Kernel ──────────┬──► JSON ──► Git
                              │
                              ├──► Binary ──► Storage
                              │
                              └──► MCP ──► LLM
```

### 6.3 Storage Architecture

```
.pmat/
├── kernel.json              # Human-readable kernel
├── kernel.bin               # Compressed binary kernel
├── kernel.db               # SQLite index for fast queries
├── bug-model.pmat          # Trained ML model
├── execution-traces/       # Time-travel recordings
│   ├── trace-001.pmat
│   └── trace-002.pmat
├── quality-index/          # Inverted index for search
│   ├── tdg-scores.idx
│   ├── churn-hotspots.idx
│   └── ml-predictions.idx
└── history/                # Kernel history
    ├── ec4434cf.json       # Kernel at commit ec4434cf
    └── abc1234d.json       # Kernel at commit abc1234d
```

---

## Part 7: Quality Gates

### 7.1 Testing Requirements

**Unit Tests**:
- All components >85% coverage
- Property-based tests for kernel compression
- Mutation testing for ML model

**Integration Tests**:
- End-to-end DAP server workflow
- Full bug prediction pipeline
- Kernel generation and validation

**Performance Tests**:
- DAP server latency <100ms
- Kernel generation <500ms
- Search query <50ms
- Time-travel replay at real-time speed

### 7.2 Documentation Requirements

- User guide for each major feature
- API documentation for MCP tools
- Architecture decision records (ADRs)
- Performance benchmarks and optimization guide

### 7.3 Release Criteria

**Phase 1 (Tracing)**:
- [ ] DAP server passes VS Code compatibility tests
- [ ] Time-travel debugging validated on real codebases
- [ ] REPL supports all modes
- [ ] Documentation complete

**Phase 2 (Bug Discovery)**:
- [ ] ML model achieves >90% AUC-ROC
- [ ] Churn analysis validated against known bugs
- [ ] Search performance meets targets
- [ ] Integration with CI/CD demonstrated

**Phase 3 (TDG Kernel)**:
- [ ] Kernel generation <500ms for 10K files
- [ ] Git integration transparent to users
- [ ] MCP tools functional and documented
- [ ] Compression ratio >1000:1 achieved

---

## Appendices

### Appendix A: Terminology

- **DAP**: Debug Adapter Protocol - standard for debugger communication
- **TDG**: Technical Debt Grading - PMAT's quality scoring system
- **Kernel**: Compressed codebase map tracked in git
- **Hotspot**: High-churn file with quality issues
- **Churn**: Number of commits modifying a file
- **Time-Travel**: Ability to rewind/forward through execution history

### Appendix B: Related Work

**Inspiration Sources**:
- bashrs: REPL modes, audit system, quality scoring
- ruchyruchy: DAP debugging, time-travel, 10 quality tools
- rr-debugger: Time-travel debugging for C/C++
- Omniscient Debugger: Execution recording
- CodeQL: Semantic code search
- SonarQube: Multi-signal quality analysis

### Appendix C: Performance Benchmarks

**Target Performance** (10K file codebase):
- Kernel generation: <500ms
- Kernel query: <50ms
- Bug prediction: <2s
- Churn analysis: <1s
- Pattern search: <100ms
- DAP server latency: <100ms
- Time-travel snapshot: <5ms

### Appendix D: Future Enhancements

**Beyond v1.0**:
- Real-time kernel updates (file watcher)
- Distributed kernel (monorepo support)
- Visual debugger UI
- Browser-based REPL
- Custom ML models per project
- Integration with GitHub Copilot
- Kernel federation across projects

---

**End of Specification**

**Status**: DRAFT v1.0.0
**Next Steps**: Review, refine, begin Sprint 71 implementation
**Estimated Effort**: 9 sprints (~18-27 weeks at 2 sprints/month)
