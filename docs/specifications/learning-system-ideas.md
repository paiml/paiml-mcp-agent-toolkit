# Learning System Ideas - Future Enhancements

**Status**: Brainstorming / Specification Phase
**Created**: 2025-10-07
**Last Updated**: 2025-10-07

## Overview

This document captures potential learning and intelligence features that could enhance PMAT's capabilities. These ideas are inspired by analysis of VoltAgent and other AI agent frameworks, evaluated for fit with PMAT's existing architecture.

**Context**: PMAT currently has excellent static analysis, mutation testing, and agent orchestration. These ideas focus on making the system "learn" from past analyses and developer interactions.

---

## Priority 0: Claude Code Sub-Agent Scaffolding (IMMEDIATE)

### 0.1 Specialized Sub-Agent Templates for PMAT

**Inspiration**: [VoltAgent Awesome Claude Code Sub-Agents](https://github.com/VoltAgent/awesome-claude-code-subagents)

**Current State:**
- PMAT has agent scaffolding (`server/src/scaffold/agent/`) with 4 built-in templates:
  - `DeterministicCalculator`
  - `StateMachineWorkflow`
  - `HybridAnalyzer`
  - `MCPToolServer`
- Agent templates generate full projects with Cargo.toml, tests, quality gates
- Feature-based composition (state machines, quality gates, monitoring, etc.)
- **Gap**: No domain-specific PMAT sub-agents for code quality workflows

**Proposed Enhancement:**

Create a library of **specialized PMAT sub-agents** that integrate seamlessly with Claude Code, inspired by the 80+ sub-agents in VoltAgent's collection:

#### PMAT-Specific Sub-Agent Categories

**1. Code Quality Specialists**
```rust
pub enum PmatSubAgent {
    // Core Quality Agents
    ComplexityAnalyst,        // Focuses only on complexity analysis
    MutationTester,           // Mutation testing specialist
    SATDDetector,             // Technical debt identifier
    DeadCodeEliminator,       // Unused code removal

    // Language Specialists
    RustQualityExpert,        // Rust-specific quality patterns
    PythonQualityExpert,      // Python linting, type hints
    TypeScriptQualityExpert,  // TS/JS code quality
    WasmDeepInspector,        // WASM bytecode analysis (Issue #65)

    // Architecture & Design
    RefactoringAdvisor,       // Suggests refactorings based on patterns
    BorrowCheckerTutor,       // Rust ownership guidance
    PerformanceProfiler,      // Big-O analysis and optimization
    APIDesignReviewer,        // API consistency checks

    // Testing & Verification
    TestCoverageAnalyst,      // Coverage gap identification
    PropertyTestGenerator,    // Generates property-based tests
    MLMutationPredictor,      // Uses ML model for mutation prediction
    EquivalentMutantDetector, // Identifies redundant mutations

    // Documentation & Standards
    DocumentationEnforcer,    // Generic description detection
    CodeStyleGuardian,        // Style consistency checks
    CommitMessageReviewer,    // Git commit quality

    // Orchestration & Meta
    QualityGateOrchestrator,  // Coordinates multiple quality checks
    WorkflowOptimizer,        // Analyzes and improves workflows
    MetricsAggregator,        // Collects and reports on all metrics
}
```

**2. Sub-Agent Structure (Claude Code Compatible)**

Each sub-agent follows this standardized format:

```markdown
# [Agent Name]

## Description
One-line description of agent's purpose

## Capabilities
- Capability 1
- Capability 2
- Capability 3

## Tools Used
- MCP Tool 1 (from PMAT server)
- MCP Tool 2
- MCP Tool 3

## Role Definition
Detailed prompt defining the agent's expertise, constraints, and behavior

## Communication Protocol
How this agent interacts with:
- Main Claude Code instance
- Other sub-agents
- PMAT MCP server

## Implementation Workflow
Step-by-step process the agent follows

## Example Invocations
```bash
# Automatic (Claude detects need)
[User mentions "high complexity" → ComplexityAnalyst activates]

# Manual
"@ComplexityAnalyst analyze src/services for functions over CC=10"
```

## Quality Gates
- Minimum test coverage: 85%
- Max cyclomatic complexity: <8
- No generic descriptions in docstrings
```

**3. Example: Mutation Testing Specialist**

```markdown
# Mutation Testing Specialist

## Description
Expert in mutation testing, equivalent mutant detection, and test suite quality analysis using PMAT's ML-powered mutation engine.

## Capabilities
- Generate mutants using PMAT's operators (arithmetic, logical, relational, etc.)
- Predict mutant survival using Decision Tree ML model (75-95% accuracy)
- Detect equivalent mutants via AST-based semantic analysis
- Suggest test cases to kill surviving mutants
- Analyze mutation score trends over time

## Tools Used
- `pmat__mutation_test` (MCP)
- `pmat__mutation_predict` (MCP - ML inference)
- `pmat__equivalent_detector` (MCP)
- `pmat__analyze_complexity` (MCP - context for mutation placement)

## Role Definition
You are a mutation testing expert specializing in improving test suite effectiveness. You:

1. **Understand Mutation Operators**: Know when to apply arithmetic, logical, relational,
   boundary, unary, assignment mutations
2. **ML-Guided Analysis**: Use PMAT's Decision Tree predictor to prioritize high-value mutants
3. **Equivalent Mutant Detection**: Identify mutations that don't change semantics
4. **Test Gap Identification**: Pinpoint weaknesses in test coverage
5. **Performance Aware**: Balance mutation count vs. execution time

**Constraints**:
- Only suggest mutations with >70% survival probability (ML confidence)
- Skip equivalent mutants automatically
- Respect timeout limits (default 5min per mutant)
- Report mutation score trends, not just raw numbers

## Communication Protocol

**With Main Claude Code**:
- Receive: File paths, mutation targets, test command
- Return: Mutation report with surviving mutants, test suggestions

**With Other Sub-Agents**:
- **ComplexityAnalyst**: Get complexity metrics to prioritize mutation targets
- **TestCoverageAnalyst**: Coordinate on untested code paths
- **PropertyTestGenerator**: Request property tests for surviving mutants

**With PMAT MCP Server**:
- Call `mutation_test` tool with parameters
- Query ML predictor for survival probability
- Check equivalent mutant detector

## Implementation Workflow

1. **Receive Request**: Parse target files/functions
2. **Prioritize**: Use complexity analysis to focus on high-risk code
3. **Generate Mutants**: Call PMAT mutation engine
4. **Predict Outcomes**: Use ML model to filter low-value mutants
5. **Execute Tests**: Run test suite against each mutant
6. **Analyze Results**:
   - Report mutation score
   - Highlight surviving mutants
   - Detect patterns (e.g., "all boundary mutations survive")
7. **Suggest Improvements**:
   - Specific test cases to add
   - Code areas needing better coverage
8. **Report**: Markdown summary with actionable recommendations

## Example Invocations

**Automatic Invocation**:
```
User: "Our tests passed but I'm worried about quality"
→ MutationTestingSpecialist: "I can run mutation testing to verify your
   test suite catches bugs. Analyzing src/ now..."
```

**Manual Invocation**:
```
User: "@MutationTestingSpecialist analyze src/services/complexity.rs"
→ Agent: "Running mutation analysis on complexity.rs...
         Found 42 mutants, 37 killed (88% score).
         5 surviving mutants in calculate_cyclomatic():
         - Line 145: + → - (boundary condition not tested)
         - Line 156: > → >= (off-by-one case missing)
         Suggest adding test case: complex_if_chain_boundary_test()"
```

**Coordination**:
```
MutationTestingSpecialist → ComplexityAnalyst:
  "Which functions have CC>10? I'll prioritize those for mutation testing."

ComplexityAnalyst → MutationTestingSpecialist:
  "3 functions: calculate_metrics (CC=15), analyze_ast (CC=12),
   parse_expression (CC=11). Start with calculate_metrics."
```

## Quality Gates
- ML model accuracy: ≥75% (validated via cross-validation)
- Equivalent mutant detection rate: ≥90%
- Test execution timeout: <5min per mutant
- Report generation: <10s after analysis
- No false positives in equivalent detection
```

**4. Integration with PMAT Scaffold**

Extend existing agent generator to create sub-agent definitions:

```rust
// New in server/src/scaffold/agent/subagents.rs
pub struct SubAgentGenerator {
    template_dir: PathBuf,
}

impl SubAgentGenerator {
    pub fn generate_subagent(&self, agent_type: PmatSubAgent) -> Result<String> {
        let template = match agent_type {
            PmatSubAgent::MutationTester => self.generate_mutation_tester(),
            PmatSubAgent::ComplexityAnalyst => self.generate_complexity_analyst(),
            // ... 20+ specialized agents
        };
        Ok(template)
    }

    pub fn export_for_claude_code(&self, agent: PmatSubAgent) -> Result<PathBuf> {
        // Generate markdown file compatible with Claude Code sub-agent format
        let content = self.generate_subagent(agent)?;
        let path = PathBuf::from(format!(".claude/subagents/{}.md", agent.name()));
        std::fs::write(&path, content)?;
        Ok(path)
    }
}
```

**CLI Integration**:
```bash
# List available sub-agents
pmat scaffold subagent list

# Generate specific sub-agent for Claude Code
pmat scaffold subagent create mutation-tester --output .claude/subagents/

# Generate full suite
pmat scaffold subagent create-all --category quality

# Validate sub-agent definition
pmat scaffold subagent validate .claude/subagents/mutation-tester.md
```

**5. MCP Tool Mapping**

Each sub-agent maps to specific PMAT MCP tools:

| Sub-Agent | Primary MCP Tools | Secondary Tools |
|-----------|------------------|-----------------|
| ComplexityAnalyst | `analyze_complexity` | `analyze_cognitive_complexity` |
| MutationTester | `mutation_test`, `mutation_predict` | `equivalent_detector` |
| SATDDetector | `analyze_satd` | `analyze_context` |
| DeadCodeEliminator | `analyze_dead_code` | `analyze_imports` |
| WasmDeepInspector | `deep_wasm_analyze` | `wasm_disassemble` |
| RefactoringAdvisor | `suggest_refactorings` | `analyze_complexity`, `detect_patterns` |
| DocumentationEnforcer | `check_generic_docs` | `analyze_context` |

**6. Value Proposition**

**For Claude Code Users**:
- **Specialized Expertise**: Instead of general-purpose analysis, get domain-specific guidance
- **Automatic Delegation**: Claude Code routes quality questions to appropriate sub-agents
- **Parallel Execution**: Multiple sub-agents work simultaneously
- **Consistent Quality**: Each sub-agent follows PMAT best practices

**For PMAT Project**:
- **Ecosystem Growth**: Community can contribute new sub-agents
- **Use Case Discovery**: See how users combine sub-agents in workflows
- **Marketing**: "PMAT + Claude Code = 20+ AI quality experts"
- **Integration Showcase**: Demonstrates MCP tools in real workflows

**Example Workflow (Multi-Agent)**:
```
User: "Review this PR for quality issues"

→ QualityGateOrchestrator activates:
  ├─ ComplexityAnalyst (parallel)
  ├─ MutationTester (parallel)
  ├─ SATDDetector (parallel)
  └─ DocumentationEnforcer (parallel)

→ RefactoringAdvisor synthesizes results:
  "Found 3 high-complexity functions, 2 with low mutation scores,
   5 TODO comments, and 1 generic docstring.

   Priority 1: Refactor calculate_metrics() (CC=15, mutation score 60%)
   Priority 2: Add tests for parse_expression() (5 surviving mutants)
   Priority 3: Resolve TODO in line 42 (marked 6 months ago)"
```

**7. Implementation Roadmap**

**Phase 1 (Week 1-2): Core Sub-Agents**
- ComplexityAnalyst
- MutationTester
- SATDDetector
- DeadCodeEliminator
- DocumentationEnforcer

**Phase 2 (Week 3-4): Language Specialists**
- RustQualityExpert
- PythonQualityExpert
- TypeScriptQualityExpert
- WasmDeepInspector

**Phase 3 (Week 5-6): Advanced Agents**
- RefactoringAdvisor (uses pattern learning)
- MLMutationPredictor (exposes ML model)
- EquivalentMutantDetector (AST-based)
- PerformanceProfiler (Big-O analysis)

**Phase 4 (Week 7-8): Meta & Orchestration**
- QualityGateOrchestrator
- WorkflowOptimizer
- MetricsAggregator
- TestCoverageAnalyst

**8. Estimated Effort**

- **Sub-Agent Template Generation**: 1-2 days per agent (markdown + prompt engineering)
- **MCP Tool Mapping**: 1 day (already have tools, just need clean mapping)
- **CLI Integration**: 2-3 days (extend scaffold command)
- **Documentation**: 3-4 days (examples, tutorials, best practices)
- **Testing**: 2-3 days (validate sub-agents with Claude Code)

**Total**: 4-6 weeks for full library (20+ sub-agents)

**MVP (5 core agents)**: 1-2 weeks

**9. Risks & Mitigations**

**Risk 1**: Sub-agents hallucinate capabilities not in MCP tools
- **Mitigation**: Strict tool-only mode, validate against actual MCP schema

**Risk 2**: Sub-agents conflict (e.g., both suggest different refactorings)
- **Mitigation**: Orchestrator agent coordinates and resolves conflicts

**Risk 3**: Performance (20 agents running simultaneously)
- **Mitigation**: Lazy activation, cache results, parallel MCP calls

**Risk 4**: Maintenance burden (20+ agent definitions to keep updated)
- **Mitigation**: Generate from templates, automate updates when MCP tools change

**10. Success Metrics**

- **Adoption**: 50+ developers using PMAT sub-agents with Claude Code
- **Contributions**: 5+ community-contributed sub-agents
- **Usage**: 80% of PMAT MCP tools accessed via sub-agents (vs direct calls)
- **Quality**: 90%+ positive feedback on sub-agent suggestions
- **Performance**: <2s average sub-agent response time

---

## Priority 1: High ROI, Leverages Existing Infrastructure

### 1.1 Declarative Workflow API with Fluent Builder Pattern

**Current State:**
- DAG-based workflow executor exists (`server/src/workflow/executor.rs` - 996 lines)
- Parallel execution supported via `execute_parallel()`
- Requires manual `WorkflowStep` construction (imperative)

**Proposed Enhancement:**
```rust
// Fluent builder API for readable workflow definitions
WorkflowBuilder::new("comprehensive-quality-check")
    .and_then(agent("complexity-analyzer").with_params(json!({
        "threshold": 8,
        "include_cognitive": true
    })))
    .and_when(|result| result.max_complexity > 10)
        .and_all([
            agent("mutation-tester").timeout(Duration::from_secs(300)),
            agent("satd-detector"),
            agent("dead-code-analyzer")
        ])
        .and_race([
            agent("refactor-suggester-ml"),
            agent("refactor-suggester-rule-based").timeout(Duration::from_secs(5))
        ])
    .and_then(agent("quality-gate-enforcer"))
    .with_recovery(RetryPolicy::exponential_backoff(3))
    .build()
```

**Implementation Plan:**
- New module: `server/src/workflow/builder.rs`
- Fluent methods: `and_then()`, `and_all()`, `and_race()`, `and_when()`, `and_unless()`
- Compile to existing `WorkflowDefinition` DAG
- Zero runtime overhead (builder pattern)

**Value:**
- **Developer Experience**: 10x more readable than manual DAG construction
- **Reusability**: Named workflows become shareable templates
- **Type Safety**: Compile-time validation of workflow structure
- **Backward Compatible**: Existing `WorkflowExecutor` unchanged

**Estimated Effort**: 3-5 days
- Day 1: Design API surface, write RED tests
- Day 2-3: Implement builder methods and validation
- Day 4: Integration with existing executor
- Day 5: Documentation and examples

**Risks**: Low - purely additive, no breaking changes

---

### 1.2 Pattern Learning from Historical Analysis

**Current State:**
- Analysis results generated per-run (stateless)
- No persistence of insights across runs
- Memory manager exists (`server/src/services/memory_manager.rs`) but unused for learning

**Proposed Enhancement:**
Store and learn from historical analysis patterns:

```rust
pub struct PatternLearningService {
    storage: Arc<dyn PatternStorage>,
    similarity_threshold: f64,
}

#[derive(Serialize, Deserialize)]
pub struct CodePattern {
    pub id: Uuid,
    pub pattern_type: PatternType, // Complexity, SATD, Mutation
    pub features: Vec<f64>,        // Extracted features (AST metrics, etc.)
    pub context: PatternContext,
    pub successful_fixes: Vec<RefactoringAction>,
    pub confidence: f64,
    pub seen_count: u32,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug)]
pub enum PatternType {
    HighComplexity { avg_cc: f64, avg_cognitive: f64 },
    RecurringMutation { operator: String, survival_rate: f64 },
    SATDPattern { category: String, keywords: Vec<String> },
    DeadCodePattern { reason: String },
}
```

**Use Cases:**
1. **Mutation Prediction Improvement**: "Functions with similar AST structure had 85% survival rate for arithmetic mutations"
2. **Refactoring Suggestions**: "Teams using pattern X successfully reduced complexity by extracting methods"
3. **SATD Evolution**: "TODO comments in async code persist 3x longer than sync code"
4. **Cross-Project Learning**: "This complexity pattern appears in 12 other analyzed projects"

**Implementation Plan:**
- New module: `server/src/services/learning/patterns.rs`
- Storage backend: SQLite (via existing `rusqlite` dependency) or `sled` (already in Cargo.toml)
- Feature extraction: Reuse existing AST analysis
- Similarity matching: Cosine similarity on feature vectors

**Value:**
- **Accuracy**: ML mutation predictor could improve from 75-95% to 85-98%
- **Context-Aware**: Suggestions improve over time based on project-specific patterns
- **Knowledge Sharing**: Cross-project insights without sharing code

**Estimated Effort**: 5-7 days
- Day 1-2: Storage schema and persistence layer
- Day 3-4: Feature extraction and similarity matching
- Day 5-6: Integration with mutation/complexity analyzers
- Day 7: Testing and validation

**Risks**: Medium
- Storage growth over time (needs pruning strategy)
- Feature vector stability across PMAT versions

---

## Priority 2: Strategic Differentiation

### 2.1 RAG-Based Codebase Memory System

**Current State:**
- No vector embeddings or semantic search
- Context generation is stateless (per-run)
- No knowledge graph of analyzed codebases

**Proposed Enhancement:**
Full RAG (Retrieval-Augmented Generation) system for semantic code analysis:

```rust
pub struct SemanticMemoryService {
    vector_store: Arc<dyn VectorStore>,
    embeddings_provider: Arc<dyn EmbeddingsProvider>,
    knowledge_graph: Arc<KnowledgeGraph>,
}

pub trait VectorStore: Send + Sync {
    async fn store(&self, embedding: Embedding, metadata: Metadata) -> Result<Uuid>;
    async fn search(&self, query: Embedding, top_k: usize) -> Result<Vec<SearchResult>>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

pub trait EmbeddingsProvider: Send + Sync {
    async fn embed_code(&self, code: &str) -> Result<Vec<f32>>;
    async fn embed_query(&self, query: &str) -> Result<Vec<f32>>;
}

#[derive(Debug)]
pub struct CodeMemory {
    pub embedding: Vec<f32>,
    pub code_snippet: String,
    pub metadata: MemoryMetadata,
    pub analysis_results: AnalysisSnapshot,
}

#[derive(Debug)]
pub struct MemoryMetadata {
    pub file_path: PathBuf,
    pub function_name: Option<String>,
    pub language: String,
    pub complexity_score: f64,
    pub mutation_score: Option<f64>,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
}
```

**Supported Backends:**
1. **Qdrant** (vector-native, production-ready)
2. **PostgreSQL + pgvector** (leverage existing DB)
3. **Chroma** (embedded, no server required)
4. **In-Memory** (development/testing)

**Query Examples:**
```rust
// Semantic search
semantic_memory.query("high complexity async functions with error handling").await?;

// Similar code patterns
semantic_memory.find_similar_to(current_function, limit: 10).await?;

// Historical context
semantic_memory.get_evolution(file_path, since: 30.days().ago()).await?;

// Cross-project insights
semantic_memory.search_across_projects("authentication logic with JWT").await?;
```

**Integration Points:**
- **Mutation Testing**: Query similar functions to predict mutation survival
- **Complexity Analysis**: Find refactoring patterns that worked for similar code
- **SATD Detection**: Track TODO evolution and resolution patterns
- **Agent Memory**: Agents remember past decisions and outcomes

**Implementation Plan:**
- Phase 1 (Week 1): Abstract vector store trait + in-memory implementation
- Phase 2 (Week 2): Embeddings provider (OpenAI API + local sentence-transformers via Python binding)
- Phase 3 (Week 3): Integration with deep_context service
- Phase 4 (Week 4): Qdrant/pgvector backends + MCP tools
- Phase 5 (Week 5): Knowledge graph for relationships (function calls, imports, etc.)

**Value:**
- **Unprecedented**: No other static analysis tool has semantic memory
- **Compound Learning**: Gets smarter with every analysis
- **Developer Queries**: "Show me all functions that handle user authentication"
- **Pattern Discovery**: Automatically find common anti-patterns across projects

**Estimated Effort**: 4-5 weeks
- Infrastructure: 2 weeks
- Integration: 2 weeks
- Polish + testing: 1 week

**Risks**: Medium-High
- External dependencies (vector DB, embeddings API)
- Storage costs (embeddings are ~1.5KB per code block)
- Privacy concerns (embeddings are somewhat reversible)

**Mitigation:**
- Local embeddings option (sentence-transformers, no API)
- Configurable retention policies
- Anonymization layer for sensitive codebases

---

### 2.2 Workflow Templates Marketplace

**Current State:**
- Workflows defined programmatically
- No sharing mechanism between projects/teams
- Examples exist in docs but not executable

**Proposed Enhancement:**
Shareable, versioned workflow templates:

```yaml
# .pmat/workflows/comprehensive-rust-quality.yml
name: "Comprehensive Rust Quality Check"
version: "1.2.0"
author: "pragmatic-ai-labs"
description: "Full quality suite for production Rust projects"
tags: ["rust", "quality", "production"]

parameters:
  complexity_threshold:
    type: integer
    default: 8
    description: "Maximum cyclomatic complexity"

  mutation_timeout:
    type: duration
    default: "5m"
    description: "Timeout for mutation testing"

workflow:
  - step: "analyze-complexity"
    agent: "complexity-analyzer"
    params:
      threshold: ${{ parameters.complexity_threshold }}
      include_cognitive: true

  - step: "parallel-analysis"
    type: parallel
    steps:
      - agent: "mutation-tester"
        timeout: ${{ parameters.mutation_timeout }}
      - agent: "satd-detector"
      - agent: "dead-code-analyzer"
      - agent: "borrow-checker-analyzer"

  - step: "quality-gate"
    agent: "quality-gate-enforcer"
    when: ${{ steps.analyze-complexity.max_complexity > parameters.complexity_threshold }}

  - step: "generate-report"
    agent: "report-generator"
    params:
      format: ["markdown", "json", "html"]
      output: "./target/pmat-report"
```

**Features:**
- **Version Management**: Semantic versioning for workflows
- **Parameter Validation**: Type-safe parameters with defaults
- **Dependency Resolution**: Workflows can depend on other workflows
- **Import/Export**: Share via Git, npm registry, or PMAT registry
- **CLI Integration**: `pmat workflow run comprehensive-rust-quality`

**Marketplace Concept:**
```bash
# Discover workflows
pmat workflow search "rust quality"

# Install workflow
pmat workflow install pragmatic-ai-labs/comprehensive-rust-quality

# List installed
pmat workflow list

# Run workflow
pmat workflow run comprehensive-rust-quality --param complexity_threshold=10
```

**Implementation:**
- New module: `server/src/workflow/templates/`
- YAML parser (use `serde_yaml` - already in deps)
- Template engine (Handlebars - already in deps at 6.3.2)
- Registry client (optional - can start with Git-based distribution)

**Value:**
- **Community Building**: Teams share best practices
- **Onboarding**: New projects start with proven workflows
- **Standardization**: Company-wide quality standards as code
- **Discoverability**: Find workflows for specific languages/frameworks

**Estimated Effort**: 2-3 weeks
- Week 1: YAML schema, parser, validation
- Week 2: CLI commands, template engine
- Week 3: Documentation, example workflows

**Risks**: Low-Medium
- Schema evolution (versioning handles this)
- Security (malicious workflows - needs sandboxing)

---

## Priority 3: Production Operations

### 3.1 Visual Observability Dashboard (PMAT Observatory)

**Current State:**
- Basic metrics collector exists (`server/src/claude_integration/observability.rs`)
- RED method tracking (Rate, Errors, Duration)
- TDG web dashboard (`server/src/tdg/web_dashboard.rs`) but not real-time

**Proposed Enhancement:**
Production-grade observability UI inspired by VoltOps:

**Dashboard Sections:**

1. **Real-Time Workflow Execution**
   - Live DAG visualization with step completion status
   - Agent execution traces with timing breakdowns
   - Parallel step visualization (Gantt chart)

2. **MCP Tool Call Analytics**
   - Heatmap of tool latencies
   - Success/failure rates per tool
   - Most expensive operations
   - Tool usage trends over time

3. **ML Model Performance**
   - Mutation predictor accuracy over time
   - Feature importance visualization
   - Confidence score distribution
   - Prediction vs actual outcomes

4. **Cost Tracking**
   - API call costs (if using external embeddings/LLMs)
   - Compute time per analysis
   - Storage growth trends
   - Cost per project/team breakdown

5. **Error Pattern Detection**
   - Recurring error signatures
   - Error rate by component
   - Anomaly detection (sudden spikes)
   - Root cause suggestions

**Tech Stack:**
- **Backend**: Extend existing web dashboard (Warp framework already in use)
- **Frontend**: Static HTML + Alpine.js + Chart.js (keep it lightweight)
- **Real-Time**: WebSocket streaming for live updates
- **Storage**: Time-series data in SQLite or sled (already in deps)
- **API**: REST + WebSocket endpoints

**Implementation:**
```rust
// New module: server/src/observatory/
mod dashboard;      // HTTP endpoints
mod websocket;      // Live updates
mod collectors;     // Metrics aggregation
mod visualizations; // Chart data generation
mod alerts;         // Threshold-based alerting
```

**Features:**
- **Live Monitoring**: Watch analyses execute in real-time
- **Historical Analysis**: Query past performance
- **Alerting**: Slack/email when quality gates fail
- **Drill-Down**: Click on failed step → see full error context
- **Exportable**: Download metrics as JSON/CSV

**Value:**
- **Production Confidence**: Monitor PMAT in CI/CD pipelines
- **Performance Optimization**: Identify bottlenecks visually
- **Debugging**: Understand why workflows failed
- **Stakeholder Reporting**: Show quality trends to management

**Estimated Effort**: 3-4 weeks
- Week 1: Metrics collection and storage
- Week 2: REST API and WebSocket streaming
- Week 3: Frontend dashboard UI
- Week 4: Alerting and polish

**Risks**: Medium
- Real-time updates at scale (need rate limiting)
- Browser performance with large DAGs (need virtualization)

---

### 3.2 Agent Lifecycle Telemetry

**Current State:**
- Agents exist (`server/src/agents/`, `server/src/agent/`)
- Basic monitoring hooks in workflow executor
- No detailed agent-level telemetry

**Proposed Enhancement:**
Deep telemetry for multi-agent systems:

```rust
#[derive(Debug, Serialize)]
pub struct AgentTelemetry {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub lifecycle_events: Vec<LifecycleEvent>,
    pub performance_metrics: PerformanceMetrics,
    pub resource_usage: ResourceUsage,
    pub communication_log: Vec<MessageTrace>,
}

#[derive(Debug, Serialize)]
pub enum LifecycleEvent {
    Created { timestamp: DateTime<Utc> },
    Started { workflow_id: Uuid },
    StateTransition { from: String, to: String },
    Completed { duration: Duration, result: TaskResult },
    Failed { error: String, retry_count: u32 },
    Suspended { reason: String },
    Resumed { checkpoint: String },
    Terminated { reason: String },
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub total_execution_time: Duration,
    pub idle_time: Duration,
    pub cpu_time: Duration,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub average_task_duration: Duration,
}

#[derive(Debug, Serialize)]
pub struct MessageTrace {
    pub timestamp: DateTime<Utc>,
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: String,
    pub payload_size: usize,
    pub latency: Duration,
}
```

**Use Cases:**
- **Agent Performance Profiling**: Which agents are slowest?
- **Communication Patterns**: Visualize agent interactions (graph)
- **Bottleneck Detection**: Identify agents that block workflows
- **Resource Optimization**: Right-size agent pools

**Implementation:**
- Extend existing `WorkflowMonitor` trait
- Add telemetry collection to agent execution
- Store in Observatory backend
- Visualize in dashboard

**Value:**
- **Multi-Agent Debugging**: Understand complex agent interactions
- **Performance Tuning**: Optimize agent configurations
- **Capacity Planning**: Know when to scale agent pools

**Estimated Effort**: 1-2 weeks

---

## Priority 4: Novel Interaction Paradigms

### 4.1 Voice Interface for Code Analysis

**Current State:**
- CLI, MCP, HTTP interfaces only
- No audio/speech capabilities

**Proposed Enhancement:**
Voice-controlled PMAT for hands-free development:

**Capabilities:**
1. **Voice Commands**
   - "PMAT, analyze the complexity of src/services"
   - "What mutations survived in the last test run?"
   - "Show me the most complex functions"
   - "Run comprehensive quality check on main branch"

2. **Voice Responses**
   - Text-to-speech for analysis results
   - Summarized reports (not full JSON dump)
   - Interactive Q&A sessions

3. **Accessibility**
   - Support developers with visual impairments
   - Hands-free code reviews during pair programming
   - Voice-driven refactoring sessions

**Tech Stack:**
- **Speech-to-Text**: OpenAI Whisper API or local Whisper.cpp
- **Text-to-Speech**: OpenAI TTS API or local Coqui TTS
- **Voice Activity Detection**: WebRTC VAD
- **Transport**: WebSocket for real-time audio streaming

**Implementation:**
```rust
// New module: server/src/voice/
mod speech_to_text;  // STT provider abstraction
mod text_to_speech;  // TTS provider abstraction
mod vad;             // Voice Activity Detection
mod commands;        // Parse voice commands to CLI actions
mod responses;       // Format analysis results for voice
```

**Example Flow:**
```
User: "PMAT, analyze complexity"
→ VAD detects speech end
→ Send audio to Whisper API
→ Parse command: analyze_complexity { path: "." }
→ Execute analysis
→ Format result: "Found 3 files with complexity above threshold.
                  The highest is calculate_metrics in utils.rs with score 15."
→ TTS speaks response
```

**Value:**
- **Accessibility**: Critical for inclusive development
- **Novel UX**: Differentiate from other static analysis tools
- **Productivity**: Hands-free during mob programming
- **Demo Factor**: "Wow" factor for marketing

**Estimated Effort**: 2-3 weeks
- Week 1: STT/TTS integration, command parsing
- Week 2: WebSocket audio streaming, VAD
- Week 3: Response formatting, testing

**Risks**: Medium
- Latency (STT can take 1-2 seconds)
- Accuracy (need good microphone, quiet environment)
- Cost (API calls for STT/TTS)

**Mitigation:**
- Local Whisper.cpp for zero-cost STT
- Wake word ("Hey PMAT") to reduce false triggers
- Text fallback for noisy environments

---

### 4.2 Natural Language Query Interface

**Current State:**
- CLI requires exact command syntax
- No natural language understanding

**Proposed Enhancement:**
Ask questions in plain English:

```
"Show me all functions that might have memory leaks"
→ Translated to: pmat analyze dead-code --type unsafe-patterns

"Which files changed most in the last month with high complexity?"
→ Translated to: pmat analyze churn --since 30d --filter complexity>10

"Compare mutation scores between v1.0 and v2.0"
→ Translated to: pmat compare-branches v1.0 v2.0 --metric mutation-score
```

**Implementation:**
- Use LLM (Claude/GPT) to parse intent → structured command
- Fallback to rule-based parsing for common queries
- Learning system improves parsing over time

**Value:**
- **Lower Barrier**: Non-experts can use PMAT
- **Exploration**: Discover features through conversation
- **Documentation**: Query replaces reading docs

**Estimated Effort**: 1-2 weeks (if using LLM API)

---

## Priority 5: Speculative / Research

### 5.1 Mutation Testing Auto-Repair

**Idea:** When mutation survives, automatically generate test case to kill it.

**Approach:**
1. Detect surviving mutant
2. Analyze mutation type (e.g., `+` → `-`)
3. Generate test input that exercises mutated path
4. Use symbolic execution or fuzzing to find inputs where mutant fails
5. Output test case as PR suggestion

**Challenges:**
- Symbolic execution in Rust is hard (no mature tools)
- May generate brittle tests
- Works best for pure functions

**Estimated Effort**: 4-6 weeks (research spike)

---

### 5.2 Cross-Language Code Clone Detection

**Idea:** Detect when Rust code is semantically similar to Python/JS/etc.

**Use Case:** "This Rust function implements the same algorithm as your Python util"

**Approach:**
- Generate language-agnostic IR (e.g., simplified AST)
- Use tree edit distance or embeddings
- Report semantic clones across language boundaries

**Value:** Identify refactoring opportunities, shared logic

**Estimated Effort**: 3-4 weeks

---

### 5.3 Predictive Quality Forecasting

**Idea:** "Based on current trends, your test coverage will drop below 80% in 2 sprints"

**Approach:**
- Track quality metrics over time (complexity, coverage, debt)
- Time-series analysis (ARIMA, Prophet)
- Forecast future quality trends
- Alert when projections cross thresholds

**Value:** Proactive quality management

**Estimated Effort**: 2-3 weeks

---

## Implementation Roadmap

### Short-Term (Next 2 Months)
1. **Declarative Workflow API** (Priority 1.1) - High ROI, low risk
2. **Pattern Learning** (Priority 1.2) - Foundation for intelligence

### Mid-Term (3-6 Months)
3. **Visual Observatory** (Priority 3.1) - Production readiness
4. **Workflow Templates** (Priority 2.2) - Community building
5. **Agent Telemetry** (Priority 3.2) - Multi-agent debugging

### Long-Term (6-12 Months)
6. **RAG-Based Memory** (Priority 2.1) - Strategic differentiation
7. **Voice Interface** (Priority 4.1) - Novel UX (if use case emerges)
8. **Natural Language Queries** (Priority 4.2) - Lower barrier to entry

### Research Track (Ongoing)
9. **Mutation Auto-Repair** (Priority 5.1) - If feasible
10. **Cross-Language Clones** (Priority 5.2) - Interesting but niche

---

## Evaluation Criteria

For each idea, assess:
1. **Feasibility**: Can we build it with existing stack?
2. **Value**: Does it solve real pain points?
3. **Differentiation**: Does it make PMAT unique?
4. **Maintenance**: Can we sustain it long-term?
5. **Community Fit**: Will users actually use it?

---

## Notes

- **Avoid Feature Creep**: Only implement what users request or what provides 10x value
- **Leverage Existing Infrastructure**: Prefer ideas that reuse workflow/agent/MCP systems
- **Measure Impact**: Add telemetry to new features to validate usage
- **Iterative Development**: Start with MVP, gather feedback, iterate

---

## Contributing

Have an idea? Add it to this doc with:
- **Problem Statement**: What pain point does it solve?
- **Proposed Solution**: High-level approach
- **Value Proposition**: Why build this?
- **Estimated Effort**: Time investment
- **Risks**: What could go wrong?

---

**Last Updated**: 2025-10-07
**Next Review**: Sprint 24 Planning
