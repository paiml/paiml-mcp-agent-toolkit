```markdown
# Unified Emit-Refactor Engine Specification

**Version**: 1.0.0  
**Status**: Draft  
**Last Updated**: 2025-01-14

## Executive Summary

A dual-mode engine that combines real-time defect emission with systematic refactoring capabilities. Operates in either server mode (< 5ms latency) or interactive mode (agent-friendly) using the same analyzer infrastructure.

## 1. Architecture Overview

### 1.1 Unified Engine

```rust
pub struct UnifiedEngine {
    // Core analysis infrastructure
    ast_engine: Arc<UnifiedAstEngine>,
    cache: Arc<UnifiedCacheManager>,
    analyzers: AnalyzerPool,
    
    // Mode-specific components
    mode: EngineMode,
    state_machine: RefactorStateMachine,
    
    // Shared metrics
    metrics: Arc<EngineMetrics>,
}

pub enum EngineMode {
    Server {
        emit_buffer: RingBuffer<DefectPayload, 1024>,
        latency_target: Duration, // 5ms
    },
    Interactive {
        checkpoint_file: PathBuf,
        explain_level: ExplainLevel,
    },
}
```

### 1.2 State Machine Definition

```rust
#[derive(Serialize, Deserialize)]
pub enum State {
    // Analysis states
    Scan { targets: Vec<PathBuf> },
    Analyze { current: FileId },
    
    // Refactor states  
    Plan { violations: Vec<Violation> },
    Refactor { operation: RefactorOp },
    
    // Validation states
    Test { command: String },
    Lint { strict: bool },
    
    // Control states
    Emit { payload: DefectPayload },
    Checkpoint { reason: String },
    Complete { summary: Summary },
}
```

## 2. Analyzer Infrastructure

### 2.1 Unified Analyzer Trait

```rust
pub trait UnifiedAnalyzer: Send + Sync {
    // Metric computation (used by both modes)
    fn compute_metrics(&self, node: &AstNode) -> MetricSet;
    
    // Refactoring suggestions
    fn suggest_refactors(&self, metrics: &MetricSet) -> Vec<RefactorPlan>;
    
    // AST transformation
    fn apply_transform(&self, ast: &AstNode, plan: &RefactorPlan) -> Result<AstDelta>;
    
    // Incremental update (for emit server)
    fn update_incremental(&self, delta: &AstDelta) -> MetricDelta;
}

pub struct MetricSet {
    complexity: (u16, u16),      // (cyclomatic, cognitive)
    tdg_score: f32,              // Technical Debt Gradient
    dead_code: BitVec,           // Dead symbol indicators
    satd_count: u32,             // Self-admitted tech debt
    provability: f32,            // Proof annotation coverage
}
```

### 2.2 Language Analyzers

```rust
pub struct AnalyzerPool {
    rust: Arc<RustAnalyzer>,
    typescript: Arc<TypeScriptAnalyzer>,
    python: Arc<PythonAnalyzer>,
    c_cpp: Arc<CCppAnalyzer>,
    // ... 9 more languages
}

impl AnalyzerPool {
    pub fn get(&self, lang: Language) -> Arc<dyn UnifiedAnalyzer> {
        match lang {
            Language::Rust => self.rust.clone(),
            Language::TypeScript => self.typescript.clone(),
            // ...
        }
    }
}
```

## 3. Dual-Mode Operations

### 3.1 Server Mode

```rust
impl UnifiedEngine {
    pub async fn run_server(&mut self) -> Result<()> {
        loop {
            match self.state_machine.current() {
                State::Analyze { current } => {
                    let start = Instant::now();
                    
                    // Fast incremental analysis
                    let metrics = self.analyze_incremental(current).await?;
                    
                    // Emit if threshold crossed
                    if self.should_emit(&metrics) {
                        let payload = self.create_payload(current, metrics);
                        self.emit_buffer.push(payload);
                    }
                    
                    // Auto-advance if under latency budget
                    let elapsed = start.elapsed();
                    if elapsed < Duration::from_millis(5) {
                        self.state_machine.advance();
                    }
                }
                _ => self.state_machine.advance(),
            }
        }
    }
}
```

### 3.2 Interactive Mode

```rust
impl UnifiedEngine {
    pub async fn run_interactive(&mut self) -> Result<()> {
        loop {
            // Output current state as JSON
            let state_json = self.export_state();
            println!("{}", serde_json::to_string_pretty(&state_json)?);
            
            // Wait for command
            let command = self.read_command().await?;
            
            match command {
                Command::Continue => {
                    let result = self.step_with_explanation().await?;
                    println!("{}", result.explanation);
                }
                Command::Skip => self.state_machine.skip_current(),
                Command::Rollback => self.rollback_last_change()?,
                Command::Checkpoint => self.save_checkpoint()?,
                Command::Exit => break,
            }
        }
        Ok(())
    }
}
```

## 4. State Machine Implementation

### 4.1 Refactor State Machine

```rust
pub struct RefactorStateMachine {
    current: State,
    history: Vec<StateTransition>,
    config: RefactorConfig,
}

#[derive(Serialize, Deserialize)]
pub struct StateTransition {
    from: State,
    to: State,
    timestamp: u64,
    metrics_before: MetricSet,
    metrics_after: Option<MetricSet>,
    applied_refactor: Option<RefactorOp>,
}

impl RefactorStateMachine {
    pub fn advance(&mut self) -> Result<State> {
        let next = match &self.current {
            State::Scan { targets } => {
                State::Analyze { current: targets[0].into() }
            }
            State::Analyze { current } => {
                State::Plan { violations: self.find_violations(current) }
            }
            State::Plan { violations } if !violations.is_empty() => {
                State::Refactor { operation: violations[0].to_op() }
            }
            State::Refactor { .. } => State::Test { 
                command: "make test-fast".into() 
            },
            State::Test { .. } => State::Lint { strict: true },
            State::Lint { .. } => State::Emit { 
                payload: self.compute_payload() 
            },
            State::Emit { .. } => State::Checkpoint { 
                reason: "cycle_complete".into() 
            },
            State::Checkpoint { .. } => self.next_target()
                .map(|t| State::Analyze { current: t })
                .unwrap_or(State::Complete { 
                    summary: self.build_summary() 
                }),
            _ => State::Complete { summary: Default::default() },
        };
        
        self.transition_to(next)
    }
}
```

### 4.2 Refactor Operations

```rust
#[derive(Serialize, Deserialize)]
pub enum RefactorOp {
    ExtractFunction {
        name: String,
        start: BytePos,
        end: BytePos,
        params: Vec<String>,
    },
    FlattenNesting {
        function: String,
        strategy: NestingStrategy,
    },
    ReplaceHashMap {
        imports: Vec<String>,
        replacements: Vec<(String, String)>,
    },
    RemoveSatd {
        location: Location,
        fix: SatdFix,
    },
    SimplifyExpression {
        expr: String,
        simplified: String,
    },
}

pub enum NestingStrategy {
    EarlyReturn,
    ExtractCondition,
    GuardClause,
    StreamChain,
}
```

## 5. Command Line Interface

### 5.1 Unified Command

```bash
# Server mode - real-time emission
pmat refactor serve \
  --emit-buffer /tmp/defects.ring \
  --latency-target 5ms \
  --config refactor.toml

# Interactive mode - agent friendly
pmat refactor \
  --interactive \
  --explain detailed \
  --checkpoint state.json \
  --target complexity:20

# Analyze current state
pmat refactor status --checkpoint state.json

# Resume from checkpoint
pmat refactor resume --checkpoint state.json --steps 10
```

### 5.2 Configuration

```toml
[refactor]
# Common settings
target_complexity = 20
remove_satd = true
max_function_lines = 50

[refactor.thresholds]
cyclomatic_warn = 10
cyclomatic_error = 20
cognitive_warn = 15
cognitive_error = 30
tdg_warn = 1.5
tdg_error = 2.0

[refactor.strategies]
prefer_functional = true
use_early_returns = true
extract_helpers = true

[server]
emit_buffer_size = 1024
max_latency_ms = 5
batch_size = 10

[interactive]
explain_level = "detailed"
auto_checkpoint = true
checkpoint_interval = 10
```

## 6. Protocol Specifications

### 6.1 Extended DefectPayload

```rust
#[repr(C, align(64))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct DefectPayload {
    // Original fields
    file_hash: u64,
    tdg_score: f32,
    complexity: (u16, u16),
    dead_symbols: u32,
    timestamp: u64,
    severity_flags: u8,
    
    // Extended for refactoring
    refactor_available: bool,
    refactor_type: RefactorType,
    estimated_improvement: f32,
    
    // Padding to maintain 64-byte alignment
    _padding: [u8; 2],
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum RefactorType {
    None = 0,
    ExtractFunction = 1,
    FlattenNesting = 2,
    SimplifyLogic = 3,
    RemoveDeadCode = 4,
}
```

### 6.2 Interactive Protocol

```json
{
  "state": {
    "type": "Refactor",
    "current_file": "src/services/deep_context.rs",
    "current_function": "analyze_deep_context",
    "line_range": [234, 456]
  },
  "metrics": {
    "before": {
      "complexity": [45, 67],
      "tdg": 2.3,
      "satd": 6
    },
    "projected": {
      "complexity": [12, 18],
      "tdg": 0.8,
      "satd": 0
    }
  },
  "suggestion": {
    "type": "ExtractFunction",
    "description": "Extract complex analysis logic into 3 helper functions",
    "operations": [
      {
        "name": "analyze_single_file",
        "lines": [250, 290],
        "complexity_reduction": 15
      }
    ]
  },
  "commands": ["continue", "skip", "modify", "rollback", "explain"],
  "explanation": "This refactor reduces cyclomatic complexity from 45 to 12 by extracting nested loops into focused helper functions. Each helper has a single responsibility and complexity < 10."
}
```

## 7. Implementation Roadmap

### Phase 1: Core State Machine (Week 1)
- [ ] RefactorStateMachine implementation
- [ ] State persistence/checkpoint system
- [ ] Basic state transitions
- [ ] JSON protocol for interactive mode

### Phase 2: Analyzer Integration (Week 2)
- [ ] UnifiedAnalyzer trait implementation
- [ ] Rust analyzer with refactoring
- [ ] MetricSet computation
- [ ] RefactorOp application

### Phase 3: Dual Mode Support (Week 3)
- [ ] Server mode with ring buffer
- [ ] Interactive mode with explanations
- [ ] Mode switching logic
- [ ] Command parsing

### Phase 4: Refactoring Engine (Week 4)
- [ ] ExtractFunction implementation
- [ ] FlattenNesting strategies
- [ ] SATD removal logic
- [ ] Test validation integration

### Phase 5: Production Features (Week 5)
- [ ] Progress tracking
- [ ] Rollback mechanisms
- [ ] Performance optimization
- [ ] Documentation

## 8. Success Criteria

### Performance
- Server mode: < 5ms latency (P99)
- Interactive mode: < 100ms response time
- Memory usage: < 128MB
- Refactor success rate: > 95%

### Quality
- All refactored functions: complexity < 20
- Zero SATD in processed files
- All tests passing after refactor
- No new lint warnings

### Usability
- Claude Code integration working
- Clear explanations in interactive mode
- Checkpoint/resume reliability: 100%
- Progress tracking accuracy: 100%

## 9. Example Usage

### Server Mode Flow
```
1. Start server: emit metrics in real-time
2. Editor makes change
3. Incremental parse (< 1ms)
4. Compute metrics (< 2ms)
5. If violation: emit refactor suggestion
6. IDE shows inline hint
7. User accepts: apply refactor
8. Re-emit improved metrics
```

### Interactive Mode Flow
```
1. Claude Code: "pmat refactor --interactive"
2. System: {"state": "Analyze", "file": "main.rs"}
3. Claude: "continue"
4. System: {"violation": "complexity:45", "suggestion": "extract"}
5. Claude: "explain"
6. System: {"explanation": "Function has 3 nested loops..."}
7. Claude: "continue"
8. System: {"applied": "ExtractFunction", "new_complexity": 12}
9. Claude: "continue"
10. System: {"state": "Test", "command": "make test-fast"}
```

## Appendix A: Agent Integration Examples

### Claude Code Integration
```javascript
// Read current state
const state = JSON.parse(await runCommand('pmat refactor status --json'));

// Make decision based on metrics
if (state.metrics.complexity[0] > 30) {
  await runCommand('pmat refactor continue');
} else {
  await runCommand('pmat refactor skip');
}

// Get explanation for learning
const explanation = await runCommand('pmat refactor explain --verbose');
console.log(`Learned: ${explanation.key_insight}`);
```

### CI/CD Integration
```yaml
quality-gate:
  script:
    - pmat refactor serve --daemon &
    - make test
    - pmat refactor status --assert "complexity<20"
```
```