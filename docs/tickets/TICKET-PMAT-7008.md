# PMAT-7008: Declarative Workflow API

**Status**: 🚀 TODO
**Priority**: P1 - High
**Complexity**: Medium
**Estimated Duration**: 3-5 days
**Sprint**: 24
**Created**: 2025-10-07

---

## Objective

Create a fluent builder API for declaring workflows with readable, chainable method calls, inspired by VoltAgent's declarative workflow system.

**Inspiration**: VoltAgent's `andThen()`, `andAgent()`, `andAll()`, `andRace()`, `andWhen()` API

---

## Background

### Current State
- DAG-based workflow executor exists (`server/src/workflow/executor.rs` - 996 lines)
- Parallel execution supported via `execute_parallel()`
- Workflows defined imperatively by constructing `WorkflowStep` objects
- Functional but verbose for complex workflows

### Problem
```rust
// Current imperative approach (verbose)
let mut workflow = WorkflowDefinition {
    id: Uuid::new_v4(),
    name: "quality-check".to_string(),
    steps: vec![
        WorkflowStep {
            id: "complexity".to_string(),
            action: WorkflowAction::AgentExecution {
                agent_name: "complexity-analyzer".to_string(),
                operation: "analyze".to_string(),
                params: json!({"threshold": 8}),
            },
            dependencies: vec![],
            retry_policy: None,
        },
        // ... manually construct each step
    ],
};
```

### Desired Solution
```rust
// Declarative fluent API (readable)
WorkflowBuilder::new("quality-check")
    .and_then(agent("complexity-analyzer").with_params(json!({"threshold": 8})))
    .and_when(|result| result.max_complexity > 10)
        .and_all([
            agent("mutation-tester"),
            agent("satd-detector"),
        ])
    .and_then(agent("quality-gate"))
    .build()
```

---

## Scope

### Core Features

**1. Fluent Builder Methods**
- `WorkflowBuilder::new(name)` - Initialize builder
- `.and_then(step)` - Sequential execution
- `.and_all(steps)` - Parallel execution (all must succeed)
- `.and_race(steps)` - Parallel execution (first to succeed wins)
- `.and_when(condition)` - Conditional branching
- `.and_unless(condition)` - Negative conditional
- `.with_recovery(policy)` - Retry/recovery configuration
- `.with_timeout(duration)` - Step-level timeout
- `.build()` - Compile to `WorkflowDefinition`

**2. Agent Step Builder**
- `agent(name)` - Create agent execution step
- `.with_params(params)` - Add parameters
- `.timeout(duration)` - Set timeout
- `.with_retry(policy)` - Configure retries

**3. Condition Types**
- Closure-based: `|result| result.value > threshold`
- Named conditions: `condition("high_complexity")`
- Combinators: `any_of()`, `all_of()`, `not()`

**4. Zero-Overhead Compilation**
- Builder pattern compiles to existing `WorkflowDefinition` DAG
- No runtime overhead
- Compile-time validation where possible

---

## Implementation Plan

### Phase 1: Core Builder (Day 1-2)

**1.1 WorkflowBuilder Module**
- New file: `server/src/workflow/builder.rs`
- Struct: `WorkflowBuilder` with fluent methods
- Compile to existing `WorkflowDefinition`

```rust
pub struct WorkflowBuilder {
    name: String,
    steps: Vec<WorkflowStepBuilder>,
    current_dependencies: Vec<String>,
}

impl WorkflowBuilder {
    pub fn new(name: impl Into<String>) -> Self { }

    pub fn and_then(mut self, step: WorkflowStepBuilder) -> Self { }

    pub fn and_all(mut self, steps: Vec<WorkflowStepBuilder>) -> Self { }

    pub fn and_race(mut self, steps: Vec<WorkflowStepBuilder>) -> Self { }

    pub fn and_when<F>(mut self, condition: F) -> ConditionalBuilder
    where F: Fn(&Value) -> bool + 'static { }

    pub fn build(self) -> Result<WorkflowDefinition, BuilderError> { }
}
```

**1.2 Agent Step Builder**
```rust
pub struct WorkflowStepBuilder {
    id: String,
    action: WorkflowAction,
    retry_policy: Option<RetryPolicy>,
    timeout: Option<Duration>,
}

pub fn agent(name: impl Into<String>) -> WorkflowStepBuilder {
    WorkflowStepBuilder {
        id: Uuid::new_v4().to_string(),
        action: WorkflowAction::AgentExecution {
            agent_name: name.into(),
            operation: "execute".to_string(),
            params: Value::Null,
        },
        retry_policy: None,
        timeout: None,
    }
}

impl WorkflowStepBuilder {
    pub fn with_params(mut self, params: Value) -> Self { }
    pub fn timeout(mut self, duration: Duration) -> Self { }
    pub fn with_retry(mut self, policy: RetryPolicy) -> Self { }
}
```

### Phase 2: Conditional Branching (Day 3)

**2.1 ConditionalBuilder**
```rust
pub struct ConditionalBuilder {
    parent: WorkflowBuilder,
    condition: Box<dyn Fn(&Value) -> bool>,
    then_steps: Vec<WorkflowStepBuilder>,
    else_steps: Vec<WorkflowStepBuilder>,
}

impl ConditionalBuilder {
    pub fn and_then(mut self, step: WorkflowStepBuilder) -> Self { }

    pub fn and_all(mut self, steps: Vec<WorkflowStepBuilder>) -> Self { }

    pub fn or_else(mut self) -> ElseBranchBuilder { }

    pub fn end_when(self) -> WorkflowBuilder { }
}
```

**2.2 Condition Types**
```rust
pub enum Condition {
    Closure(Box<dyn Fn(&Value) -> bool>),
    Named(String),
    Combined(ConditionCombinator),
}

pub enum ConditionCombinator {
    AnyOf(Vec<Condition>),
    AllOf(Vec<Condition>),
    Not(Box<Condition>),
}

pub fn any_of(conditions: Vec<Condition>) -> Condition { }
pub fn all_of(conditions: Vec<Condition>) -> Condition { }
pub fn not(condition: Condition) -> Condition { }
```

### Phase 3: Advanced Features (Day 4)

**3.1 Parallel Execution Strategies**
```rust
pub enum ParallelStrategy {
    All,         // and_all() - all must succeed
    Race,        // and_race() - first success wins
    AllSettled,  // Wait for all, report all results
    Any(usize),  // At least N must succeed
}
```

**3.2 Recovery Policies**
```rust
impl WorkflowBuilder {
    pub fn with_recovery(mut self, policy: RetryPolicy) -> Self { }
}

pub enum RetryPolicy {
    None,
    FixedDelay { attempts: u32, delay: Duration },
    ExponentialBackoff { attempts: u32, initial_delay: Duration },
    Custom(Box<dyn Fn(u32) -> Option<Duration>>),
}
```

**3.3 Workflow Composition**
```rust
impl WorkflowBuilder {
    pub fn include_workflow(mut self, workflow: WorkflowDefinition) -> Self { }

    pub fn fork_workflow(mut self, workflows: Vec<WorkflowDefinition>) -> Self { }
}
```

### Phase 4: Testing & Documentation (Day 5)

**4.1 Unit Tests (RED → GREEN)**
```rust
#[test]
fn test_simple_sequential_workflow() {
    let workflow = WorkflowBuilder::new("test")
        .and_then(agent("agent1"))
        .and_then(agent("agent2"))
        .build()
        .unwrap();

    assert_eq!(workflow.steps.len(), 2);
    assert_eq!(workflow.steps[1].dependencies, vec![workflow.steps[0].id]);
}

#[test]
fn test_parallel_execution() {
    let workflow = WorkflowBuilder::new("test")
        .and_all(vec![
            agent("agent1"),
            agent("agent2"),
        ])
        .build()
        .unwrap();

    assert_eq!(workflow.steps.len(), 2);
    assert!(workflow.steps[0].dependencies.is_empty());
    assert!(workflow.steps[1].dependencies.is_empty());
}

#[test]
fn test_conditional_branching() {
    let workflow = WorkflowBuilder::new("test")
        .and_then(agent("complexity"))
        .and_when(|result| result["score"].as_i64().unwrap() > 10)
            .and_then(agent("refactor"))
        .end_when()
        .build()
        .unwrap();

    // Verify conditional step structure
}

#[test]
fn test_race_condition() {
    let workflow = WorkflowBuilder::new("test")
        .and_race(vec![
            agent("ml-predictor").timeout(Duration::from_secs(5)),
            agent("rule-based-fallback"),
        ])
        .build()
        .unwrap();

    // Verify race strategy
}
```

**4.2 Property Tests**
```rust
#[proptest]
fn test_builder_always_produces_valid_workflow(
    #[strategy(arb_workflow_builder())] builder: WorkflowBuilder
) {
    let workflow = builder.build();
    prop_assert!(workflow.is_ok());

    let workflow = workflow.unwrap();
    // Verify DAG properties
    prop_assert!(is_acyclic(&workflow));
    prop_assert!(all_dependencies_exist(&workflow));
}
```

**4.3 Integration Tests**
```rust
#[tokio::test]
async fn test_execute_declarative_workflow() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let workflow = WorkflowBuilder::new("integration-test")
        .and_then(agent("test-agent"))
        .build()
        .unwrap();

    let result = executor.execute(&workflow).await;
    assert!(result.is_ok());
}
```

**4.4 Documentation**
- `docs/features/DECLARATIVE_WORKFLOWS.md` - User guide
- Examples: 10+ workflow patterns
- API docs for all builder methods

---

## Files to Create

### New Files
```
server/src/workflow/builder.rs                   (600 lines)
server/src/workflow/builder_tests.rs             (400 lines)
server/src/workflow/conditions.rs                (200 lines)
server/src/workflow/retry_policies.rs            (150 lines)
docs/features/DECLARATIVE_WORKFLOWS.md           (600 lines)
examples/workflows/
  ├── simple_sequential.rs
  ├── parallel_execution.rs
  ├── conditional_branching.rs
  ├── race_strategy.rs
  ├── with_retries.rs
  └── complex_multi_agent.rs
```

### Files to Modify
```
server/src/workflow/mod.rs                       (export builder module)
server/src/workflow/executor.rs                  (ensure compatibility)
README.md                                        (add workflow examples)
```

**Estimated Total**: ~2,100 new lines + 30 modified lines

---

## Example Usage

### Simple Sequential Workflow
```rust
let workflow = WorkflowBuilder::new("quality-check")
    .and_then(agent("complexity-analyzer"))
    .and_then(agent("mutation-tester"))
    .and_then(agent("report-generator"))
    .build()?;
```

### Parallel Analysis
```rust
let workflow = WorkflowBuilder::new("comprehensive-analysis")
    .and_all(vec![
        agent("complexity-analyzer"),
        agent("satd-detector"),
        agent("dead-code-analyzer"),
    ])
    .and_then(agent("aggregator"))
    .build()?;
```

### Conditional Execution
```rust
let workflow = WorkflowBuilder::new("adaptive-quality")
    .and_then(agent("complexity-analyzer"))
    .and_when(|result| result["max_complexity"].as_i64().unwrap() > 10)
        .and_all(vec![
            agent("mutation-tester"),
            agent("refactoring-suggester"),
        ])
    .end_when()
    .and_then(agent("quality-gate"))
    .build()?;
```

### Race Strategy with Fallback
```rust
let workflow = WorkflowBuilder::new("ml-with-fallback")
    .and_race(vec![
        agent("ml-predictor").timeout(Duration::from_secs(5)),
        agent("rule-based-predictor"),
    ])
    .and_then(agent("validator"))
    .build()?;
```

### With Retry Logic
```rust
let workflow = WorkflowBuilder::new("resilient-analysis")
    .and_then(
        agent("flaky-analyzer")
            .with_retry(RetryPolicy::ExponentialBackoff {
                attempts: 3,
                initial_delay: Duration::from_secs(1),
            })
    )
    .build()?;
```

### Complex Multi-Agent Coordination
```rust
let workflow = WorkflowBuilder::new("pr-review")
    .and_then(agent("git-diff-analyzer"))
    .and_all(vec![
        agent("complexity-analyst"),
        agent("security-scanner"),
        agent("style-checker"),
    ])
    .and_when(|result| result["issues_found"].as_i64().unwrap() > 0)
        .and_race(vec![
            agent("auto-fixer").timeout(Duration::from_secs(30)),
            agent("manual-review-requester"),
        ])
    .end_when()
    .and_then(agent("pr-comment-generator"))
    .build()?;
```

---

## Success Criteria

### Functional Requirements ✅
- ✅ All fluent builder methods implemented
- ✅ Sequential, parallel, race, conditional execution
- ✅ Compiles to existing `WorkflowDefinition` DAG
- ✅ Zero runtime overhead

### Quality Requirements ✅
- ✅ Test coverage ≥85%
- ✅ Property tests validate DAG correctness
- ✅ Integration tests with executor
- ✅ Documentation with 10+ examples

### Performance Requirements ✅
- ✅ Build time <1ms for typical workflows
- ✅ Execution time identical to imperative approach

---

## Risks & Mitigation

### Risk 1: API Ergonomics
**Impact**: High - Poorly designed API won't be adopted
**Mitigation**:
- Study VoltAgent's patterns (proven in production)
- Iterate on API with real-world workflow examples
- Gather early feedback from team

### Risk 2: Complexity of Conditional Logic
**Impact**: Medium - Nested conditions could be confusing
**Mitigation**:
- Keep nesting shallow (max 2-3 levels)
- Provide named conditions for reusability
- Clear error messages for invalid structures

### Risk 3: Breaking Changes to Executor
**Impact**: Low - Builder must work with existing executor
**Mitigation**:
- Compile to existing `WorkflowDefinition`
- Comprehensive integration tests
- No changes to executor required

---

## Dependencies

### Internal
- Existing workflow executor (`server/src/workflow/executor.rs`)
- Agent registry (`server/src/agents/registry.rs`)
- Workflow types (`server/src/workflow/types.rs`)

### External
- serde_json (for params)
- tokio (for async tests)

---

## Deliverables

1. **Code**
   - WorkflowBuilder module
   - Conditional branching
   - Retry policies
   - Tests (unit + property + integration)

2. **Documentation**
   - User guide with examples
   - API documentation
   - Migration guide (imperative → declarative)

3. **Examples**
   - 6 example workflows
   - Pattern library

---

## Post-MVP Enhancements

### Phase 2: Advanced Features (Deferred)
- Workflow templates (reusable patterns)
- Workflow visualization (Mermaid diagram generation)
- Workflow debugging (step-by-step execution)
- Workflow optimization (automatic parallelization)

### Phase 3: Integration (Deferred)
- CLI: `pmat workflow run <name>`
- MCP tool: `workflow_execute`
- Sub-agents can define workflows declaratively

---

## Related Tickets

- PMAT-7007: Sub-agents can use fluent workflows
- PMAT-7003: Workflow executor (already complete)
- PMAT-7009: Pattern learning can optimize workflows

---

## References

- [VoltAgent Workflows](https://github.com/VoltAgent/voltagent)
- [Learning System Ideas](../specifications/learning-system-ideas.md#11-declarative-workflow-api)
- [Existing Workflow Executor](../../server/src/workflow/executor.rs)

---

**Created**: 2025-10-07
**Last Updated**: 2025-10-07
**Status**: Ready for implementation
