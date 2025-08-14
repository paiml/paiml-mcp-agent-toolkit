# Agent Scaffolding Specification

## Overview

This specification defines the implementation requirements for extending PMAT's scaffolding system to generate deterministic MCP agent templates. The feature addresses the gap between PMAT's current generic project scaffolding and the specific architectural patterns required for production-grade deterministic agents.

## Technical Requirements

### 1. Core Architecture

#### 1.1 Template Engine Integration

```rust
use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Context for agent template generation
pub struct AgentContext {
    name: String,
    template_type: AgentTemplate,
    features: HashSet<AgentFeature>,
    quality_level: QualityLevel,
    deterministic_core: Option<CoreSpec>,
    probabilistic_wrapper: Option<WrapperSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentTemplate {
    DeterministicCalculator,
    StateMachineWorkflow,
    HybridAnalyzer,
    MCPToolServer,
    CustomAgent(PathBuf),
}
```

#### 1.2 File System Layout

```
generated-agent/
├── Cargo.toml                  # Agent-specific dependencies
├── .pmat/
│   ├── agent.toml              # Agent metadata and configuration
│   └── quality-gates.toml      # Enforcement rules
├── src/
│   ├── main.rs                 # Entry point with MCP/CLI support
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── core.rs             # Deterministic core logic
│   │   ├── state.rs            # State machine implementation
│   │   └── handlers.rs         # Tool/request handlers
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── server.rs           # MCP server implementation
│   │   ├── tools.rs            # Tool definitions
│   │   └── transport.rs        # Transport layer
│   └── quality/
│       ├── mod.rs
│       ├── invariants.rs       # Runtime invariant checks
│       └── validators.rs       # Input/output validation
├── tests/
│   ├── deterministic.rs        # Determinism verification
│   ├── property.rs             # Property-based tests
│   └── integration.rs          # MCP integration tests
└── benches/
    └── performance.rs          # Latency/throughput benchmarks
```

### 2. Implementation Details

#### 2.1 Template Generation Pipeline

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

/// Trait for template generation
pub trait TemplateGenerator: Send + Sync {
    fn generate(&self, ctx: &AgentContext) -> Result<GeneratedFiles>;
    fn validate_context(&self, ctx: &AgentContext) -> Result<()>;
    fn post_generation_hooks(&self, path: &Path) -> Result<()>;
}

pub struct GeneratedFiles {
    files: HashMap<PathBuf, FileContent>,
    permissions: HashMap<PathBuf, u32>,
    symlinks: Vec<(PathBuf, PathBuf)>,
}

pub enum FileContent {
    Text(String),
    Binary(Vec<u8>),
    Template(String),
}
```

#### 2.2 Agent Feature Composition

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgentFeature {
    // Core features
    StateMachine { states: Vec<String> },
    QualityGates { level: QualityLevel },
    
    // MCP features
    ToolComposition,
    AsyncHandlers,
    ResourceSubscriptions,
    
    // Analysis features
    ComplexityAnalysis,
    SATDDetection,
    DeadCodeElimination,
    
    // Runtime features
    Monitoring { backend: MonitoringBackend },
    Tracing { exporter: TraceExporter },
    HealthChecks,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum QualityLevel {
    Standard,      // Basic quality checks
    Strict,        // Zero warnings, high coverage
    Extreme,       // Toyota Way: zero SATD, max complexity 10
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MonitoringBackend {
    Prometheus,
    OpenTelemetry,
    Custom(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TraceExporter {
    Jaeger,
    Zipkin,
    OTLP,
}
```

#### 2.3 Hybrid Architecture Support

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridAgentSpec {
    deterministic_core: CoreSpec,
    probabilistic_wrapper: WrapperSpec,
    boundary: BoundarySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSpec {
    verification_method: VerificationMethod,
    max_complexity: u32,
    invariants: Vec<Invariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperSpec {
    model_type: ModelType,
    fallback_strategy: FallbackStrategy,
    confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySpec {
    serialization: SerializationFormat,
    validation: ValidationStrategy,
    error_propagation: ErrorPropagation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    PropertyTests,
    FormalProof,
    ModelChecking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    GPT4,
    Claude,
    Local(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackStrategy {
    Deterministic,
    DefaultValue,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    JSON,
    MessagePack,
    Protobuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStrategy {
    Schema,
    Runtime,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorPropagation {
    Immediate,
    Deferred,
    Logged,
}
```

### 3. CLI Interface Specification

#### 3.1 Command Structure

```bash
pmat scaffold agent [OPTIONS] --name <NAME> --template <TEMPLATE>

OPTIONS:
    --template <TEMPLATE>
        Template type: calculator, state-machine, hybrid, mcp-server, custom:<path>
    
    --features <FEATURES>
        Comma-separated list: state-machine,quality-gates:extreme,monitoring:prometheus
    
    --deterministic-core <SPEC>
        Core specification: verified:property,complexity:10,invariants:path/to/invariants.toml
    
    --probabilistic-wrapper <SPEC>
        Wrapper specification: model:gpt4,fallback:deterministic,confidence:0.95
    
    --output <PATH>
        Output directory (default: ./<name>)
    
    --force
        Overwrite existing directory
    
    --dry-run
        Show what would be generated without creating files
```

#### 3.2 Interactive Mode

```rust
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use console::Term;
use anyhow::Result;

pub struct InteractiveScaffolder {
    term: Term,
    theme: ColorfulTheme,
}

impl InteractiveScaffolder {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
            theme: ColorfulTheme::default(),
        }
    }

    pub fn run(&mut self) -> Result<AgentContext> {
        let name = self.prompt_name()?;
        let template = self.prompt_template()?;
        let features = self.prompt_features(&template)?;
        let quality = self.prompt_quality_level()?;
        
        let mut ctx = AgentContext {
            name,
            template_type: template.clone(),
            features,
            quality_level: quality,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };
        
        if matches!(template, AgentTemplate::HybridAnalyzer) {
            ctx.deterministic_core = Some(self.prompt_deterministic_core()?);
            ctx.probabilistic_wrapper = Some(self.prompt_probabilistic_wrapper()?);
        }
        
        self.confirm_and_generate(&ctx)?;
        Ok(ctx)
    }

    fn prompt_name(&self) -> Result<String> {
        Input::with_theme(&self.theme)
            .with_prompt("Agent name")
            .interact_text()
            .map_err(Into::into)
    }

    fn prompt_template(&self) -> Result<AgentTemplate> {
        let items = vec![
            "Deterministic Calculator",
            "State Machine Workflow",
            "Hybrid Analyzer",
            "MCP Tool Server",
            "Custom Template",
        ];
        
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select template type")
            .items(&items)
            .interact()?;
        
        Ok(match selection {
            0 => AgentTemplate::DeterministicCalculator,
            1 => AgentTemplate::StateMachineWorkflow,
            2 => AgentTemplate::HybridAnalyzer,
            3 => AgentTemplate::MCPToolServer,
            4 => {
                let path = Input::with_theme(&self.theme)
                    .with_prompt("Custom template path")
                    .interact_text()?;
                AgentTemplate::CustomAgent(PathBuf::from(path))
            }
            _ => unreachable!(),
        })
    }

    fn prompt_features(&self, template: &AgentTemplate) -> Result<HashSet<AgentFeature>> {
        // Implementation details omitted for brevity
        Ok(HashSet::new())
    }

    fn prompt_quality_level(&self) -> Result<QualityLevel> {
        let items = vec!["Standard", "Strict", "Extreme (Toyota Way)"];
        
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Quality level")
            .items(&items)
            .interact()?;
        
        Ok(match selection {
            0 => QualityLevel::Standard,
            1 => QualityLevel::Strict,
            2 => QualityLevel::Extreme,
            _ => unreachable!(),
        })
    }

    fn prompt_deterministic_core(&self) -> Result<CoreSpec> {
        // Implementation details omitted for brevity
        Ok(CoreSpec {
            verification_method: VerificationMethod::PropertyTests,
            max_complexity: 10,
            invariants: Vec::new(),
        })
    }

    fn prompt_probabilistic_wrapper(&self) -> Result<WrapperSpec> {
        // Implementation details omitted for brevity
        Ok(WrapperSpec {
            model_type: ModelType::GPT4,
            fallback_strategy: FallbackStrategy::Deterministic,
            confidence_threshold: 0.95,
        })
    }

    fn confirm_and_generate(&self, ctx: &AgentContext) -> Result<()> {
        let confirm = Confirm::with_theme(&self.theme)
            .with_prompt("Generate agent with these settings?")
            .interact()?;
        
        if !confirm {
            anyhow::bail!("Generation cancelled by user");
        }
        
        Ok(())
    }
}
```

### 4. Template Specifications

#### 4.1 MCP Server Template

```toml
# .pmat/templates/mcp-server/template.toml
[template]
name = "mcp-server"
version = "1.0.0"
min_pmat_version = "0.30.0"

[dependencies]
pmcp = "0.3.1"
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"

[files]
"src/main.rs" = { template = "main.rs.tera" }
"src/mcp/server.rs" = { template = "server.rs.tera" }
"src/mcp/tools.rs" = { template = "tools.rs.tera" }

[variables]
agent_name = { type = "string", required = true }
tools = { type = "array", items = "tool_definition" }
transport = { type = "enum", values = ["stdio", "websocket", "http"], default = "stdio" }
```

#### 4.2 State Machine Template

```rust
use async_trait::async_trait;
use anyhow::Result;

/// Generated state machine trait
#[async_trait]
pub trait AgentStateMachine: Send + Sync {
    type State: AgentState;
    type Event: AgentEvent;
    type Context: AgentContext;
    
    fn initial_state(&self) -> Self::State;
    
    async fn transition(
        &self,
        state: &Self::State,
        event: &Self::Event,
        ctx: &mut Self::Context,
    ) -> Result<Self::State>;
    
    fn validate_transition(
        &self,
        from: &Self::State,
        to: &Self::State,
        event: &Self::Event,
    ) -> Result<()>;
    
    fn invariants(&self) -> &[Box<dyn Invariant<Self::State, Self::Context>>];
}

pub trait AgentState: Clone + Send + Sync + std::fmt::Debug {}
pub trait AgentEvent: Clone + Send + Sync + std::fmt::Debug {}
pub trait AgentContext: Send + Sync {}

pub trait Invariant<S, C>: Send + Sync {
    fn check(&self, state: &S, ctx: &C) -> Result<()>;
    fn name(&self) -> &str;
}
```

### 5. Quality Enforcement

#### 5.1 Generated Quality Gates

```toml
# .pmat/quality-gates.toml
[complexity]
max_cyclomatic = 10
max_cognitive = 7
max_nesting = 3

[coverage]
min_line = 90
min_branch = 85
min_function = 95

[linting]
deny = ["unsafe_code", "missing_docs", "clippy::all"]
warn = ["clippy::pedantic", "clippy::nursery"]

[satd]
allowed_markers = []  # No TODO, FIXME, HACK allowed
scan_comments = true
scan_strings = false

[verification]
property_tests = true
fuzz_tests = true
formal_specs = "specs/*.tla"
```

#### 5.2 Runtime Invariant Checking

```rust
use std::fmt;
use anyhow::Result;
use tracing::error;

/// Runtime invariant checker for agent state validation
pub struct InvariantChecker<S, C> {
    invariants: Vec<Box<dyn Invariant<S, C>>>,
    violation_handler: ViolationHandler,
}

#[derive(Debug, Clone)]
pub enum ViolationAction {
    Panic,
    Log,
    Fallback(fn(&S, &C) -> S),
}

pub struct ViolationHandler {
    default_action: ViolationAction,
}

impl ViolationHandler {
    pub fn handle(&self, violation: &InvariantViolation) -> ViolationAction {
        self.default_action.clone()
    }
}

#[derive(Debug)]
pub struct InvariantViolation {
    invariant_name: String,
    message: String,
}

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invariant '{}' violated: {}", self.invariant_name, self.message)
    }
}

impl<S: AgentState, C: AgentContext> InvariantChecker<S, C> {
    pub fn check(&self, state: &S, ctx: &C) -> Result<()> {
        for invariant in &self.invariants {
            if let Err(e) = invariant.check(state, ctx) {
                let violation = InvariantViolation {
                    invariant_name: invariant.name().to_string(),
                    message: e.to_string(),
                };
                
                match self.violation_handler.handle(&violation) {
                    ViolationAction::Panic => panic!("{}", violation),
                    ViolationAction::Log => error!("{}", violation),
                    ViolationAction::Fallback(recovery_fn) => {
                        let recovered = recovery_fn(state, ctx);
                        // Validate recovered state
                        self.check(&recovered, ctx)?;
                    }
                }
            }
        }
        Ok(())
    }
}
```

### 6. Testing Infrastructure

#### 6.1 Determinism Verification

```rust
#[cfg(test)]
mod determinism_tests {
    use super::*;
    use proptest::prelude::*;
    
    fn create_agent(seed: u64) -> impl AgentStateMachine {
        // Create deterministic agent with seed
        TestAgent::new(seed)
    }
    
    proptest! {
        #[test]
        fn agent_is_deterministic(
            input in any::<TestInput>(),
            seed: u64,
        ) {
            let agent1 = create_agent(seed);
            let agent2 = create_agent(seed);
            
            let mut ctx1 = TestContext::default();
            let mut ctx2 = TestContext::default();
            
            let result1 = tokio_test::block_on(agent1.process(&input, &mut ctx1));
            let result2 = tokio_test::block_on(agent2.process(&input, &mut ctx2));
            
            prop_assert_eq!(result1, result2);
        }
        
        #[test]
        fn state_transitions_are_valid(
            initial_state in any::<TestState>(),
            event in any::<TestEvent>(),
        ) {
            let agent = TestAgent::default();
            let mut ctx = TestContext::default();
            
            if let Ok(next_state) = tokio_test::block_on(
                agent.transition(&initial_state, &event, &mut ctx)
            ) {
                prop_assert!(
                    agent.validate_transition(&initial_state, &next_state, &event).is_ok()
                );
            }
        }
    }
}
```

#### 6.2 Performance Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;

fn bench_agent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let agent = create_test_agent();
    
    let mut group = c.benchmark_group("agent_operations");
    
    group.bench_function("state_transition", |b| {
        b.to_async(&rt).iter(|| async {
            let mut ctx = TestContext::default();
            agent.transition(
                black_box(&TestState::Initial),
                black_box(&TestEvent::Process),
                black_box(&mut ctx),
            ).await
        })
    });
    
    group.bench_function("tool_invocation", |b| {
        b.to_async(&rt).iter(|| async {
            agent.invoke_tool(
                black_box("analyze_complexity"),
                black_box(&serde_json::json!({"file": "src/main.rs"})),
            ).await
        })
    });
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_processing", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let batch = generate_test_batch(size);
                    agent.process_batch(black_box(batch)).await
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_agent_operations);
criterion_main!(benches);
```

### 7. Integration Points

#### 7.1 Existing PMAT Commands

```rust
use clap::{Args, Command, Subcommand};
use anyhow::Result;

/// Extension to existing scaffold command
#[derive(Args)]
pub struct ScaffoldCommand {
    #[command(subcommand)]
    command: ScaffoldSubcommand,
}

#[derive(Subcommand)]
enum ScaffoldSubcommand {
    /// Scaffold a deterministic MCP agent
    Agent(AgentScaffoldArgs),
    /// List available templates
    ListTemplates,
    /// Validate a template
    ValidateTemplate { path: PathBuf },
}

#[derive(Args)]
struct AgentScaffoldArgs {
    /// Agent name
    #[arg(short, long)]
    name: String,
    
    /// Template type
    #[arg(short, long)]
    template: String,
    
    /// Features to include
    #[arg(short, long, value_delimiter = ',')]
    features: Vec<String>,
    
    /// Output directory
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Overwrite existing directory
    #[arg(long)]
    force: bool,
    
    /// Show what would be generated
    #[arg(long)]
    dry_run: bool,
}

impl ScaffoldCommand {
    pub async fn execute(self) -> Result<()> {
        match self.command {
            ScaffoldSubcommand::Agent(args) => {
                scaffold_agent(args).await
            }
            ScaffoldSubcommand::ListTemplates => {
                list_templates().await
            }
            ScaffoldSubcommand::ValidateTemplate { path } => {
                validate_template(path).await
            }
        }
    }
}
```

#### 7.2 Template Registry

```rust
use std::collections::HashMap;
use url::Url;
use anyhow::Result;

pub struct TemplateRegistry {
    builtin: HashMap<String, Box<dyn TemplateGenerator>>,
    custom: HashMap<String, PathBuf>,
    remote: HashMap<String, Url>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        let mut builtin = HashMap::new();
        
        // Register built-in templates
        builtin.insert(
            "mcp-server".to_string(),
            Box::new(MCPServerTemplate::default()) as Box<dyn TemplateGenerator>,
        );
        builtin.insert(
            "state-machine".to_string(),
            Box::new(StateMachineTemplate::default()) as Box<dyn TemplateGenerator>,
        );
        
        Self {
            builtin,
            custom: HashMap::new(),
            remote: HashMap::new(),
        }
    }
    
    pub async fn fetch_remote(&self, name: &str) -> Result<Box<dyn TemplateGenerator>> {
        let url = self.remote.get(name)
            .ok_or_else(|| anyhow::anyhow!("Remote template '{}' not found", name))?;
        
        // Fetch template from remote repository
        fetch_template_from_url(url).await
    }
    
    pub fn validate_template(&self, template: &dyn TemplateGenerator) -> Result<()> {
        // Validate template structure
        template.validate_context(&AgentContext::default())?;
        
        // Check template dependencies
        validate_dependencies(template)?;
        
        // Verify template compatibility
        check_version_compatibility(template)?;
        
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Result<Box<dyn TemplateGenerator>> {
        if let Some(template) = self.builtin.get(name) {
            Ok(template.clone())
        } else if let Some(path) = self.custom.get(name) {
            load_custom_template(path)
        } else {
            Err(anyhow::anyhow!("Template '{}' not found", name))
        }
    }
}

async fn fetch_template_from_url(url: &Url) -> Result<Box<dyn TemplateGenerator>> {
    // Implementation details for fetching remote templates
    unimplemented!()
}

fn validate_dependencies(template: &dyn TemplateGenerator) -> Result<()> {
    // Check that all required dependencies are available
    Ok(())
}

fn check_version_compatibility(template: &dyn TemplateGenerator) -> Result<()> {
    // Verify template is compatible with current PMAT version
    Ok(())
}

fn load_custom_template(path: &Path) -> Result<Box<dyn TemplateGenerator>> {
    // Load and parse custom template from filesystem
    unimplemented!()
}
```

### 8. Error Handling

```rust
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    
    #[error("Invalid agent configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Directory already exists: {0}")]
    DirectoryExists(PathBuf),
    
    #[error("Template generation failed")]
    GenerationFailed(#[source] anyhow::Error),
    
    #[error("Post-generation hook failed: {0}")]
    HookFailed(String),
    
    #[error("Template validation failed")]
    ValidationFailed(#[source] anyhow::Error),
    
    #[error("Incompatible template version: requires PMAT {required}, but running {current}")]
    IncompatibleVersion {
        required: String,
        current: String,
    },
    
    #[error("Missing required feature: {0}")]
    MissingFeature(String),
    
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}

impl From<ScaffoldError> for anyhow::Error {
    fn from(err: ScaffoldError) -> Self {
        anyhow::Error::new(err)
    }
}
```

### 9. Performance Considerations

- **Template Rendering**: Use lazy evaluation and cache compiled templates to reduce overhead
- **File I/O**: Batch file operations and use `tokio::fs` for asynchronous generation
- **Memory Usage**: Stream large templates instead of loading entire contents into memory
- **Parallelization**: Generate independent files concurrently using `tokio::task::spawn`
- **Caching**: Implement LRU cache for frequently used templates and compiled Tera instances

### 10. Future Extensions

1. **Template Marketplace**: Community-driven template sharing and discovery platform
2. **Version Migration**: Automated upgrade path for existing agents to new template versions
3. **Template Composition**: Combine multiple templates to create complex, multi-faceted agents
4. **Validation Suite**: Comprehensive pre-flight checks for generated agents
5. **Metrics Collection**: Anonymous usage statistics for template improvement (opt-in)
6. **CI/CD Integration**: GitHub Actions and GitLab CI templates for automated agent testing
7. **Visual Designer**: Web-based UI for designing agent architectures
8. **Template Testing**: Framework for testing template generators themselves

## Implementation Timeline

### Phase 1: Core Foundation (Week 1)
- Template engine integration
- Basic MCP server template
- CLI command structure
- Error handling framework

### Phase 2: State Machine Support (Week 2)
- State machine template implementation
- Quality gate integration
- Invariant checking system
- Property-based testing templates

### Phase 3: Hybrid Architecture (Week 3)
- Deterministic core specifications
- Probabilistic wrapper support
- Boundary validation
- Advanced feature composition

### Phase 4: Testing & Documentation (Days 1-3)
- Comprehensive test suite
- User documentation
- Template examples
- Integration tests

### Phase 5: Integration & Polish (Days 4-5)
- PMAT command integration
- Performance optimization
- Template registry
- Release preparation

## Success Criteria

- Generate fully functional MCP agents in under 3 seconds
- 100% of generated code passes `cargo clippy -- -D warnings`
- All templates include comprehensive test suites with >90% coverage
- Zero manual modifications required for basic functionality
- Support for custom templates with validation
- Interactive mode for guided agent creation
- Full integration with existing PMAT quality gates
- Documentation and examples for all template types

## Testing Strategy

### Unit Tests

#### Core Unit Test Requirements

```rust
#[cfg(test)]
mod template_generator_tests {
    use super::*;
    
    #[test]
    fn test_agent_context_creation() {
        let ctx = AgentContext {
            name: "test_agent".to_string(),
            template_type: AgentTemplate::MCPToolServer,
            features: HashSet::new(),
            quality_level: QualityLevel::Extreme,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };
        
        assert_eq!(ctx.name, "test_agent");
        assert_eq!(ctx.quality_level, QualityLevel::Extreme);
    }
    
    #[test]
    fn test_template_validation() {
        let generator = MCPServerTemplate::default();
        let invalid_ctx = AgentContext::default();
        
        assert!(generator.validate_context(&invalid_ctx).is_err());
    }
    
    #[test]
    fn test_file_generation() {
        let generator = MCPServerTemplate::default();
        let ctx = create_valid_context();
        
        let files = generator.generate(&ctx).unwrap();
        assert!(files.contains_key(&PathBuf::from("src/main.rs")));
        assert!(files.contains_key(&PathBuf::from("Cargo.toml")));
    }
    
    #[test]
    fn test_error_handling() {
        let result = ScaffoldError::TemplateNotFound("missing".to_string());
        assert_eq!(result.to_string(), "Template not found: missing");
    }
}
```

### Code Coverage Requirements

#### Minimum Coverage Thresholds

```toml
# .pmat/coverage.toml
[coverage]
min_line_coverage = 95.0
min_branch_coverage = 90.0
min_function_coverage = 100.0

[coverage.enforcement]
fail_on_decrease = true
generate_reports = ["html", "lcov", "json"]
exclude_patterns = ["tests/*", "benches/*", "examples/*"]

[coverage.per_module]
"src/scaffold/agent.rs" = { min_line = 98.0 }
"src/scaffold/templates.rs" = { min_line = 95.0 }
"src/scaffold/registry.rs" = { min_line = 92.0 }
```

#### Coverage Implementation

```rust
/// Coverage tracking for scaffold commands
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use cargo_tarpaulin::config::Config;
    
    #[test]
    fn verify_coverage_thresholds() {
        let config = Config::from_file(".pmat/coverage.toml").unwrap();
        let coverage = run_coverage_analysis(&config).unwrap();
        
        assert!(coverage.line_rate >= 0.95, 
                "Line coverage {} below threshold 95%", coverage.line_rate * 100.0);
        assert!(coverage.branch_rate >= 0.90,
                "Branch coverage {} below threshold 90%", coverage.branch_rate * 100.0);
        assert!(coverage.function_rate >= 1.00,
                "Function coverage {} below threshold 100%", coverage.function_rate * 100.0);
    }
}
```

### Doctests Requirements

#### Comprehensive Doctest Examples

```rust
/// Template generator for MCP server agents.
/// 
/// # Examples
/// 
/// ```
/// use pmat::scaffold::{MCPServerTemplate, AgentContext, AgentTemplate, QualityLevel};
/// use std::collections::HashSet;
/// 
/// let generator = MCPServerTemplate::default();
/// let ctx = AgentContext {
///     name: "example_agent".to_string(),
///     template_type: AgentTemplate::MCPToolServer,
///     features: HashSet::new(),
///     quality_level: QualityLevel::Standard,
///     deterministic_core: None,
///     probabilistic_wrapper: None,
/// };
/// 
/// let files = generator.generate(&ctx).unwrap();
/// assert!(files.contains_key(&PathBuf::from("src/main.rs")));
/// ```
/// 
/// # Error Handling
/// 
/// ```
/// # use pmat::scaffold::{MCPServerTemplate, AgentContext};
/// let generator = MCPServerTemplate::default();
/// let invalid_ctx = AgentContext::default(); // Missing required fields
/// 
/// assert!(generator.generate(&invalid_ctx).is_err());
/// ```
pub struct MCPServerTemplate {
    // Implementation
}

/// Creates a new agent context with the specified parameters.
/// 
/// # Examples
/// 
/// ```
/// use pmat::scaffold::create_agent_context;
/// 
/// let ctx = create_agent_context("my_agent", "mcp-server")
///     .with_feature("quality-gates:extreme")
///     .with_feature("monitoring:prometheus")
///     .build()
///     .unwrap();
/// 
/// assert_eq!(ctx.name, "my_agent");
/// assert_eq!(ctx.quality_level, QualityLevel::Extreme);
/// ```
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The agent name is invalid
/// - The template type is not recognized
/// - Required features are missing
/// 
/// ```
/// # use pmat::scaffold::create_agent_context;
/// let result = create_agent_context("", "invalid-template").build();
/// assert!(result.is_err());
/// ```
pub fn create_agent_context(name: &str, template: &str) -> AgentContextBuilder {
    AgentContextBuilder::new(name, template)
}
```

### Property Tests Requirements

#### Comprehensive Property-Based Testing

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::collection::{hash_set, vec};
    
    proptest! {
        #[test]
        fn prop_agent_context_roundtrip(
            name in "[a-z][a-z0-9_]{0,63}",
            features in hash_set(any::<AgentFeature>(), 0..10),
            quality_level in prop::sample::select(vec![
                QualityLevel::Standard,
                QualityLevel::Strict,
                QualityLevel::Extreme
            ])
        ) {
            let ctx = AgentContext {
                name: name.clone(),
                template_type: AgentTemplate::MCPToolServer,
                features: features.clone(),
                quality_level,
                deterministic_core: None,
                probabilistic_wrapper: None,
            };
            
            // Serialize and deserialize
            let json = serde_json::to_string(&ctx).unwrap();
            let deserialized: AgentContext = serde_json::from_str(&json).unwrap();
            
            prop_assert_eq!(ctx.name, deserialized.name);
            prop_assert_eq!(ctx.features, deserialized.features);
            prop_assert_eq!(ctx.quality_level, deserialized.quality_level);
        }
        
        #[test]
        fn prop_template_generation_deterministic(
            seed: u64,
            name in "[a-z][a-z0-9_]{0,63}",
        ) {
            let ctx1 = create_seeded_context(&name, seed);
            let ctx2 = create_seeded_context(&name, seed);
            
            let generator = MCPServerTemplate::default();
            let files1 = generator.generate(&ctx1).unwrap();
            let files2 = generator.generate(&ctx2).unwrap();
            
            // Same input should produce same output
            prop_assert_eq!(files1.len(), files2.len());
            for (path, content1) in files1.iter() {
                let content2 = files2.get(path).unwrap();
                prop_assert_eq!(content1, content2);
            }
        }
        
        #[test]
        fn prop_error_messages_informative(
            invalid_name in prop::string::string_regex("[^a-zA-Z0-9_]+").unwrap(),
        ) {
            let result = create_agent_context(&invalid_name, "mcp-server").build();
            
            if let Err(e) = result {
                let msg = e.to_string();
                prop_assert!(msg.contains("invalid") || msg.contains("Invalid"));
                prop_assert!(msg.contains(&invalid_name) || msg.contains("name"));
            }
        }
        
        #[test]
        fn prop_quality_gates_enforced(
            complexity in 1u32..100,
            quality_level in prop::sample::select(vec![
                QualityLevel::Standard,
                QualityLevel::Strict,
                QualityLevel::Extreme
            ])
        ) {
            let max_allowed = match quality_level {
                QualityLevel::Standard => 20,
                QualityLevel::Strict => 15,
                QualityLevel::Extreme => 10,
            };
            
            let spec = CoreSpec {
                verification_method: VerificationMethod::PropertyTests,
                max_complexity: complexity,
                invariants: Vec::new(),
            };
            
            let is_valid = validate_complexity_for_quality(&spec, quality_level);
            prop_assert_eq!(is_valid, complexity <= max_allowed);
        }
    }
}
```

### Cargo Examples Requirements

#### Example Programs

```rust
// examples/scaffold_agent.rs
//! Example of scaffolding a new MCP agent using PMAT.
//! 
//! Run with: `cargo run --example scaffold_agent`

use pmat::scaffold::{scaffold_agent, AgentConfig};
use anyhow::Result;

fn main() -> Result<()> {
    // Example 1: Basic MCP server
    let config = AgentConfig::builder()
        .name("example_mcp_server")
        .template("mcp-server")
        .build()?;
    
    scaffold_agent(config)?;
    println!("✅ Created MCP server agent");
    
    // Example 2: State machine with quality gates
    let config = AgentConfig::builder()
        .name("state_machine_agent")
        .template("state-machine")
        .add_feature("quality-gates:extreme")
        .add_feature("monitoring:prometheus")
        .build()?;
    
    scaffold_agent(config)?;
    println!("✅ Created state machine agent with quality gates");
    
    // Example 3: Hybrid agent with deterministic core
    let config = AgentConfig::builder()
        .name("hybrid_analyzer")
        .template("hybrid")
        .with_deterministic_core(
            CoreSpec::new()
                .verification_method(VerificationMethod::PropertyTests)
                .max_complexity(10)
        )
        .with_probabilistic_wrapper(
            WrapperSpec::new()
                .model_type(ModelType::GPT4)
                .fallback_strategy(FallbackStrategy::Deterministic)
                .confidence_threshold(0.95)
        )
        .build()?;
    
    scaffold_agent(config)?;
    println!("✅ Created hybrid analyzer agent");
    
    Ok(())
}
```

```rust
// examples/interactive_scaffold.rs
//! Interactive agent scaffolding example.
//! 
//! Run with: `cargo run --example interactive_scaffold`

use pmat::scaffold::InteractiveScaffolder;
use anyhow::Result;

fn main() -> Result<()> {
    let mut scaffolder = InteractiveScaffolder::new();
    
    println!("🚀 PMAT Agent Scaffolder");
    println!("========================");
    println!();
    
    let context = scaffolder.run()?;
    
    println!();
    println!("📦 Generated agent: {}", context.name);
    println!("   Template: {:?}", context.template_type);
    println!("   Quality: {:?}", context.quality_level);
    println!("   Features: {} enabled", context.features.len());
    
    Ok(())
}
```

```rust
// examples/validate_template.rs
//! Template validation example.
//! 
//! Run with: `cargo run --example validate_template -- path/to/template`

use pmat::scaffold::{TemplateRegistry, validate_template_file};
use std::env;
use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <template_path>", args[0]);
        std::process::exit(1);
    }
    
    let template_path = &args[1];
    
    println!("🔍 Validating template: {}", template_path);
    
    match validate_template_file(template_path) {
        Ok(report) => {
            println!("✅ Template is valid!");
            println!();
            println!("Template Details:");
            println!("  Name: {}", report.name);
            println!("  Version: {}", report.version);
            println!("  Min PMAT Version: {}", report.min_pmat_version);
            println!("  Files: {}", report.file_count);
            println!("  Dependencies: {}", report.dependencies.len());
        }
        Err(e) => {
            eprintln!("❌ Template validation failed:");
            eprintln!("   {}", e);
            
            // Print detailed errors
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("   Caused by: {}", err);
                source = err.source();
            }
            
            std::process::exit(1);
        }
    }
    
    Ok(())
}
```

### PMAT Quality Checks Compliance

#### Quality Gate Integration

```rust
/// All generated code must pass PMAT quality checks.
/// 
/// # Quality Standards
/// 
/// ```
/// use pmat::quality::{QualityGate, QualityConfig};
/// use pmat::scaffold::generate_agent;
/// 
/// let agent_path = generate_agent("test_agent", "mcp-server").unwrap();
/// 
/// let config = QualityConfig {
///     max_complexity: 10,
///     max_cognitive_complexity: 7,
///     max_nesting: 3,
///     allow_satd: false,
///     min_coverage: 90.0,
/// };
/// 
/// let gate = QualityGate::new(config);
/// let result = gate.check_path(&agent_path).unwrap();
/// 
/// assert!(result.passed, "Generated code must pass quality gates");
/// assert_eq!(result.satd_count, 0, "No SATD comments allowed");
/// assert!(result.max_complexity <= 10, "Complexity must be ≤10");
/// ```
pub fn ensure_quality_compliance(path: &Path) -> Result<QualityReport> {
    let gate = QualityGate::extreme(); // Toyota Way standards
    gate.check_path(path)
}

#[cfg(test)]
mod quality_compliance_tests {
    use super::*;
    
    #[test]
    fn test_generated_template_passes_quality_gates() {
        let templates = vec![
            "mcp-server",
            "state-machine",
            "hybrid",
            "calculator",
        ];
        
        for template in templates {
            let temp_dir = tempfile::tempdir().unwrap();
            let config = AgentConfig::builder()
                .name("quality_test")
                .template(template)
                .output(temp_dir.path())
                .build()
                .unwrap();
            
            scaffold_agent(config).unwrap();
            
            // Run PMAT quality checks
            let result = ensure_quality_compliance(temp_dir.path()).unwrap();
            
            assert!(result.passed, 
                    "Template {} failed quality gates: {:?}", template, result);
            assert_eq!(result.satd_count, 0, 
                    "Template {} contains SATD comments", template);
            assert!(result.max_complexity <= 10,
                    "Template {} exceeds complexity limit: {}", template, result.max_complexity);
            assert!(result.clippy_warnings == 0,
                    "Template {} has clippy warnings: {}", template, result.clippy_warnings);
        }
    }
    
    #[test]
    fn test_generated_code_compiles_without_warnings() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = AgentConfig::builder()
            .name("compile_test")
            .template("mcp-server")
            .output(temp_dir.path())
            .build()
            .unwrap();
        
        scaffold_agent(config).unwrap();
        
        // Compile with strict settings
        let output = std::process::Command::new("cargo")
            .args(&["build", "--release"])
            .env("RUSTFLAGS", "-D warnings")
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
        
        assert!(output.status.success(), 
                "Generated code failed to compile: {}", 
                String::from_utf8_lossy(&output.stderr));
    }
    
    #[test]
    fn test_generated_tests_pass() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = AgentConfig::builder()
            .name("test_runner")
            .template("state-machine")
            .output(temp_dir.path())
            .build()
            .unwrap();
        
        scaffold_agent(config).unwrap();
        
        // Run all tests
        let output = std::process::Command::new("cargo")
            .args(&["test", "--all-features"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
        
        assert!(output.status.success(),
                "Generated tests failed: {}",
                String::from_utf8_lossy(&output.stderr));
    }
}
```

#### PMAT Integration Tests

```rust
#[cfg(test)]
mod pmat_integration_tests {
    use super::*;
    use pmat::{analyze, refactor, quality_gate};
    
    #[test]
    fn test_scaffold_integrates_with_pmat_analyze() {
        let agent_path = scaffold_test_agent("analyzer_integration");
        
        // Run PMAT analysis commands
        let complexity = analyze::complexity(&agent_path).unwrap();
        assert!(complexity.max_complexity <= 10);
        
        let satd = analyze::satd(&agent_path).unwrap();
        assert_eq!(satd.total_count, 0);
        
        let dead_code = analyze::dead_code(&agent_path).unwrap();
        assert_eq!(dead_code.unused_functions.len(), 0);
    }
    
    #[test]
    fn test_scaffold_compatible_with_refactor_auto() {
        let agent_path = scaffold_test_agent("refactor_integration");
        let file = agent_path.join("src/main.rs");
        
        // Ensure refactor auto can process generated code
        let plan = refactor::auto(&file).unwrap();
        
        // Generated code should need minimal refactoring
        assert!(plan.suggestions.len() <= 2,
                "Generated code should be high quality, found {} suggestions",
                plan.suggestions.len());
    }
    
    #[test]
    fn test_quality_gate_enforcement() {
        let agent_path = scaffold_test_agent("quality_gate_test");
        
        let result = quality_gate::check(&agent_path).unwrap();
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }
}
```

### Integration Tests
- End-to-end scaffolding
- Generated agent compilation
- Quality gate compliance
- MCP server functionality

### Property Tests
- Determinism verification
- State machine invariants
- Input validation
- Performance characteristics

### Acceptance Tests
- User workflow scenarios
- Template customization
- Error recovery
- Performance benchmarks