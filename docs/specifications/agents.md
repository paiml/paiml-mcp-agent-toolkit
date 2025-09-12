# PMAT Agents.md Integration Specification

## Executive Summary

This specification defines the integration of the AGENTS.md standard into PMAT (Pragmatic AI MCP Agent Toolkit), creating a unified system that bridges the simple, markdown-based AGENTS.md format with PMAT's comprehensive quality enforcement and MCP capabilities. This integration enables AI agents to seamlessly understand and work with codebases while maintaining PMAT's extreme quality standards.

## 1. Overview

### 1.1 Purpose
- **Unified Agent Protocol**: Bridge AGENTS.md simplicity with MCP's structured communication
- **Quality-First Integration**: Apply PMAT's quality gates to agent-generated code
- **Bidirectional Support**: Read AGENTS.md files AND generate them from PMAT analysis
- **Multi-Protocol Gateway**: Enable agents using AGENTS.md to access MCP tools

### 1.2 Design Principles
- **Zero Configuration**: Auto-detect and use AGENTS.md files
- **Progressive Enhancement**: Basic AGENTS.md support with advanced MCP features
- **Quality Enforcement**: All agent actions pass through PMAT quality gates
- **Backward Compatible**: Support existing AGENTS.md files without modification

## 2. Architecture

### 2.1 Component Architecture
```rust
pub mod agents_md {
    pub mod parser;      // AGENTS.md parsing and validation
    pub mod discovery;   // File discovery and hierarchy resolution
    pub mod executor;    // Command execution from AGENTS.md
    pub mod generator;   // Generate AGENTS.md from PMAT analysis
    pub mod bridge;      // MCP-AGENTS.md protocol bridge
    pub mod manifest;    // Agent capabilities and registration
    pub mod router;      // Request routing between protocols
    pub mod quality;     // Quality gate integration
}
```

### 2.2 Data Flow
```
AGENTS.md → Parser → Validator → Executor → Quality Gates → Output
     ↕                                              ↕
MCP Protocol ← Bridge → Router → Tools → Unified Quality System
```

## 3. Core Components

### 3.1 AGENTS.md Parser
```rust
pub struct AgentsMdParser {
    /// Parse markdown into structured format
    pub fn parse(&self, content: &str) -> Result<AgentsMdDocument>;
    
    /// Validate document structure
    pub fn validate(&self, doc: &AgentsMdDocument) -> Result<ValidationReport>;
    
    /// Extract sections by type
    pub fn extract_sections(&self, doc: &AgentsMdDocument) -> HashMap<SectionType, Section>;
}

pub struct AgentsMdDocument {
    pub metadata: DocumentMetadata,
    pub sections: Vec<Section>,
    pub commands: Vec<Command>,
    pub guidelines: Vec<Guideline>,
    pub quality_rules: Option<QualityRules>,
}
```

### 3.2 Discovery System
```rust
pub struct AgentsMdDiscovery {
    /// Find nearest AGENTS.md file
    pub fn find_nearest(&self, path: &Path) -> Option<PathBuf>;
    
    /// Discover all AGENTS.md in project
    pub fn discover_all(&self, root: &Path) -> Vec<AgentsMdFile>;
    
    /// Build hierarchy for monorepos
    pub fn build_hierarchy(&self, files: Vec<AgentsMdFile>) -> AgentsMdHierarchy;
    
    /// Cache discovered files
    cache: DashMap<PathBuf, AgentsMdFile>,
}
```

### 3.3 Command Executor
```rust
pub struct AgentsMdExecutor {
    /// Execute commands with safety checks
    pub async fn execute_command(&self, cmd: &Command) -> Result<CommandOutput>;
    
    /// Validate command safety
    pub fn validate_command(&self, cmd: &Command) -> Result<SafetyReport>;
    
    /// Apply quality gates to output
    pub fn apply_quality_gates(&self, output: &CommandOutput) -> Result<QualityReport>;
    
    /// Sandboxed execution environment
    sandbox: SandboxEnvironment,
}
```

### 3.4 AGENTS.md Generator
```rust
pub struct AgentsMdGenerator {
    /// Generate from PMAT analysis
    pub fn generate_from_analysis(&self, analysis: &PmatAnalysis) -> Result<String>;
    
    /// Generate from project structure
    pub fn generate_from_project(&self, project: &ProjectInfo) -> Result<String>;
    
    /// Update existing AGENTS.md
    pub fn update_existing(&self, current: &str, updates: Updates) -> Result<String>;
    
    /// Templates for different project types
    templates: HashMap<ProjectType, Template>,
}
```

### 3.5 MCP-AGENTS.md Bridge
```rust
pub struct McpAgentsMdBridge {
    /// Convert AGENTS.md to MCP tools
    pub fn agents_to_mcp(&self, doc: &AgentsMdDocument) -> Vec<McpTool>;
    
    /// Convert MCP capabilities to AGENTS.md
    pub fn mcp_to_agents(&self, tools: &[McpTool]) -> String;
    
    /// Bidirectional protocol translation
    pub fn translate_request(&self, req: Request) -> TranslatedRequest;
    
    /// Unified response handling
    pub fn unify_response(&self, resp: Response) -> UnifiedResponse;
}
```

### 3.6 Agent Manifest
```rust
pub struct AgentManifest {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub supported_protocols: Vec<Protocol>,
    pub quality_requirements: QualityRequirements,
    pub tools: Vec<Tool>,
}

pub enum Protocol {
    AgentsMd,
    Mcp,
    Http,
    WebSocket,
}

pub struct Capability {
    pub name: String,
    pub description: String,
    pub input_schema: JsonSchema,
    pub output_schema: JsonSchema,
    pub quality_gates: Vec<QualityGate>,
}
```

### 3.7 Router System
```rust
pub struct AgentRouter {
    /// Route requests to appropriate handler
    pub async fn route(&self, request: AgentRequest) -> Result<AgentResponse>;
    
    /// Register protocol handlers
    pub fn register_handler(&mut self, protocol: Protocol, handler: Box<dyn Handler>);
    
    /// Load balancing for multiple agents
    pub fn balance_load(&self, requests: Vec<AgentRequest>) -> Vec<RouteDecision>;
    
    /// Circuit breaker for failing agents
    circuit_breaker: CircuitBreaker,
}
```

### 3.8 Quality Integration
```rust
pub struct AgentQualityGate {
    /// Validate agent-generated code
    pub fn validate_code(&self, code: &str) -> Result<QualityReport>;
    
    /// Enforce complexity limits
    pub fn check_complexity(&self, ast: &Ast) -> Result<ComplexityReport>;
    
    /// SATD detection in agent output
    pub fn detect_satd(&self, content: &str) -> Result<SatdReport>;
    
    /// Auto-fix quality issues
    pub fn auto_fix(&self, code: &str) -> Result<String>;
}
```

## 4. Implementation Plan

### Phase 1: Core Parser and Discovery (Week 1)
- [ ] Implement markdown parser with pest grammar
- [ ] Build discovery system with file watching
- [ ] Create validation and error reporting
- [ ] Add comprehensive unit tests (TDD)

### Phase 2: Executor and Safety (Week 2)
- [ ] Implement sandboxed command execution
- [ ] Add safety validation and limits
- [ ] Integrate with PMAT quality gates
- [ ] Create property tests for security

### Phase 3: Generator and Templates (Week 3)
- [ ] Build AGENTS.md generator from analysis
- [ ] Create project-type templates
- [ ] Implement incremental updates
- [ ] Add documentation generation

### Phase 4: MCP Bridge (Week 4)
- [ ] Implement protocol translation
- [ ] Create tool mapping system
- [ ] Build request/response routing
- [ ] Add performance optimization

### Phase 5: Integration and Testing (Week 5)
- [ ] Complete integration tests
- [ ] Performance benchmarking
- [ ] Security audit
- [ ] Documentation and examples

## 5. Quality Requirements

### 5.1 Code Quality
- **Complexity**: Max cyclomatic complexity ≤10
- **Coverage**: Minimum 90% test coverage
- **Property Tests**: 100% of parsers and validators
- **SATD**: Zero technical debt comments
- **Documentation**: 100% public API documentation

### 5.2 Performance
- **Parsing**: <10ms for typical AGENTS.md
- **Discovery**: <100ms for large monorepos
- **Execution**: <1s for command validation
- **Memory**: <50MB for parser and cache

### 5.3 Security
- **Command Validation**: Whitelist-based execution
- **Sandboxing**: All commands in isolated environment
- **Resource Limits**: CPU, memory, and time bounds
- **Audit Logging**: All agent actions logged

## 6. Integration Examples

### 6.1 Basic AGENTS.md Support
```rust
// Discover and use AGENTS.md
let discovery = AgentsMdDiscovery::new();
let agents_file = discovery.find_nearest(&current_path)?;
let parser = AgentsMdParser::new();
let document = parser.parse(&fs::read_to_string(agents_file)?)?;

// Execute testing command
let executor = AgentsMdExecutor::new();
for command in document.commands {
    let output = executor.execute_command(&command).await?;
    println!("Executed: {} -> {:?}", command.name, output);
}
```

### 6.2 MCP Bridge Usage
```rust
// Convert AGENTS.md to MCP tools
let bridge = McpAgentsMdBridge::new();
let mcp_tools = bridge.agents_to_mcp(&document);

// Register tools with MCP server
let mcp_server = MpcServer::new();
for tool in mcp_tools {
    mcp_server.register_tool(tool);
}
```

### 6.3 Quality-Enforced Generation
```rust
// Generate AGENTS.md with quality gates
let analyzer = PmatAnalyzer::new();
let analysis = analyzer.analyze_project(&project_path)?;

let generator = AgentsMdGenerator::new();
let agents_md = generator.generate_from_analysis(&analysis)?;

// Validate generated content
let quality_gate = AgentQualityGate::new();
let report = quality_gate.validate_code(&agents_md)?;
assert!(report.passed());
```

## 7. Testing Strategy

### 7.1 Unit Tests (TDD)
- Parser: 100% coverage of markdown parsing
- Discovery: File system traversal and caching
- Executor: Command validation and sandboxing
- Generator: Template rendering and updates

### 7.2 Property Tests
```rust
#[proptest]
fn test_parser_handles_any_markdown(content: String) {
    let parser = AgentsMdParser::new();
    let result = parser.parse(&content);
    prop_assert!(result.is_ok() || result.unwrap_err().is_recoverable());
}

#[proptest]
fn test_bridge_preserves_semantics(doc: AgentsMdDocument) {
    let bridge = McpAgentsMdBridge::new();
    let mcp_tools = bridge.agents_to_mcp(&doc);
    let regenerated = bridge.mcp_to_agents(&mcp_tools);
    prop_assert_eq!(normalize(doc), normalize(parse(regenerated)));
}
```

### 7.3 Integration Tests
- End-to-end agent workflows
- Multi-protocol communication
- Quality gate enforcement
- Performance benchmarks

## 8. Migration Path

### 8.1 For Existing AGENTS.md Users
1. PMAT auto-detects existing AGENTS.md files
2. Provides enhanced capabilities via MCP
3. No changes required to existing files
4. Progressive enhancement available

### 8.2 For PMAT Users
1. Auto-generate AGENTS.md from analysis
2. Expose MCP tools to AGENTS.md agents
3. Unified quality enforcement
4. Seamless protocol bridging

## 9. Success Metrics

### 9.1 Technical Metrics
- Parser accuracy: >99% on real-world files
- Discovery speed: <100ms for 1000+ files
- Bridge latency: <10ms translation overhead
- Quality compliance: 100% of agent code passes gates

### 9.2 Adoption Metrics
- Support for 15+ agent platforms
- Zero-config detection rate: >95%
- User satisfaction: >9/10
- Performance improvement: 2x faster agent operations

## 10. Future Enhancements

### 10.1 Advanced Features
- Machine learning for command prediction
- Distributed agent coordination
- Real-time collaborative editing
- Cloud-based agent marketplace

### 10.2 Protocol Extensions
- GraphQL API for agent queries
- WebSocket for real-time updates
- gRPC for high-performance RPC
- WASM plugins for custom logic

## Appendix A: AGENTS.md Example

```markdown
# AGENTS.md

## Project Overview
PMAT - Pragmatic AI MCP Agent Toolkit
A zero-configuration AI context generation system with extreme quality enforcement.

## Development Setup
```bash
# Install dependencies
cargo build --all

# Run tests
make test

# Check quality
pmat quality-gate --all
```

## Testing Instructions
- Run `make test` before committing
- Ensure 80%+ coverage maintained
- All functions must have complexity ≤10

## Code Style
- Use `rustfmt` for formatting
- Follow PMAT quality standards
- Zero SATD tolerance

## PR Guidelines
- Squash commits with conventional format
- Must pass all quality gates
- Include property tests for new features

## Security Considerations
- No secrets in code
- Validate all external input
- Use sandboxed execution for commands
```

## Appendix B: References

- [AGENTS.md Official Site](https://agents.md/)
- [GitHub Repository](https://github.com/openai/agents.md)
- [MCP Specification](https://modelcontextprotocol.io)
- [PMAT Documentation](https://docs.rs/pmat)