# TICKET-PMAT-7005: PForge Agent Scaffolding Integration

**Status**: 🔨 TODO
**Priority**: Medium
**Complexity**: Medium (3-4 days)
**Sprint**: Sprint 23
**Created**: 2025-10-07
**Dependencies**: pforge crate from crates.io

## Objective

Integrate the pforge crate from crates.io to enable intelligent agent scaffolding within pmat, streamlining agent development with templates and boilerplate generation.

## Requirements

### 1. PForge Dependency Integration (1-2 days)
- [ ] Add pforge as dependency in Cargo.toml
- [ ] Verify pforge API compatibility
- [ ] Integrate pforge scaffolding API into pmat
- [ ] Pass agent specifications to pforge library
- [ ] Generate agent code into pmat workspace
- [ ] Handle pforge errors gracefully

### 2. Agent Template Generation (1 day)
- [ ] Use pforge templates for common agent patterns
- [ ] Generate boilerplate agent code (main.rs, lib.rs)
- [ ] Create agent configuration files (config.toml, agent.json)
- [ ] Set up agent dependencies in Cargo.toml
- [ ] Generate README with usage instructions
- [ ] Create example test files

### 3. Publishing Integration (1 day)
- [ ] Coordinate with MCP Registry publishing
- [ ] Use pforge for both local scaffolding and registry publishing
- [ ] Update publishing workflow to use pforge library
- [ ] CLI integration for seamless scaffolding
- [ ] Validation before publishing
- [ ] Error handling for registry failures

## Implementation Plan

### Files to Create
- `server/src/pforge/` (new module)
- `server/src/pforge/mod.rs`
- `server/src/pforge/integration.rs` - Main integration logic
- `server/src/pforge/templates.rs` - Template management
- `server/src/pforge/validation.rs` - Agent spec validation
- `server/src/cli/commands/scaffold_agent.rs` (extend)

### Cargo.toml
```toml
[dependencies]
pforge = "0.1"  # Check latest version from crates.io
```

### CLI Command Integration
```rust
// Extend existing scaffold command
#[derive(Parser, Debug)]
pub struct ScaffoldAgentArgs {
    /// Agent name
    #[arg(short, long)]
    name: String,

    /// Agent description
    #[arg(short, long)]
    description: Option<String>,

    /// Template to use (basic, advanced, custom)
    #[arg(short, long, default_value = "basic")]
    template: String,

    /// Output directory
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Use pforge for scaffolding
    #[arg(long, default_value = "true")]
    use_pforge: bool,
}
```

### PForge Integration Module
```rust
pub struct PForgeIntegration {
    client: PForgeClient,
    config: PForgeConfig,
}

impl PForgeIntegration {
    pub fn new() -> Result<Self>;
    pub fn scaffold_agent(&self, spec: &AgentSpec) -> Result<GeneratedAgent>;
    pub fn publish_to_registry(&self, agent: &Agent) -> Result<PublishResult>;
    pub fn list_templates(&self) -> Result<Vec<Template>>;
}
```

### Agent Specification
```rust
pub struct AgentSpec {
    pub name: String,
    pub description: String,
    pub template: String,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub author: Option<String>,
    pub license: Option<String>,
}
```

## CLI Usage

```bash
# Scaffold a new agent using pforge
pmat scaffold agent --name my-agent --description "My custom agent" --template basic

# Scaffold with advanced template
pmat scaffold agent --name advanced-agent --template advanced

# Scaffold and publish to registry
pmat scaffold agent --name published-agent --publish

# List available templates
pmat scaffold agent --list-templates

# Disable pforge (use legacy scaffolding)
pmat scaffold agent --name legacy-agent --use-pforge=false
```

## Success Criteria

- [ ] pforge dependency added and compiles
- [ ] Agent scaffolding works with pforge templates
- [ ] Generated agents have correct structure
- [ ] Publishing to MCP Registry works
- [ ] CLI command is user-friendly
- [ ] Error messages are clear and actionable
- [ ] Documentation updated
- [ ] Tests cover pforge integration (10+ tests)
- [ ] Legacy scaffolding still available as fallback

## Testing Strategy

1. **Unit Tests**: PForge API calls, template loading, validation
2. **Integration Tests**: End-to-end agent scaffolding
3. **CLI Tests**: Command parsing, error handling
4. **Mock Tests**: pforge crate mocked for offline testing
5. **Real Tests**: Actual pforge crate integration

## Generated Agent Structure

```
my-agent/
├── Cargo.toml           # Agent dependencies
├── README.md            # Usage instructions
├── src/
│   ├── main.rs         # Agent entry point
│   ├── lib.rs          # Agent logic
│   └── config.rs       # Configuration handling
├── tests/
│   └── integration.rs  # Integration tests
├── examples/
│   └── basic_usage.rs  # Usage examples
└── agent.json          # MCP agent metadata
```

## Template Types

### Basic Template
- Simple agent with minimal features
- Single capability
- No external dependencies
- Good for learning/prototyping

### Advanced Template
- Multiple capabilities
- External API integration
- Error handling
- Logging and monitoring
- Configuration management

### Custom Template
- User-defined template
- Maximum flexibility
- Requires template specification

## Error Handling

- **Network Errors**: Retry with exponential backoff
- **Template Not Found**: List available templates, suggest alternatives
- **Invalid Spec**: Detailed validation errors with fix suggestions
- **Registry Errors**: Clear error messages with troubleshooting steps
- **pforge Unavailable**: Fallback to legacy scaffolding

## Value Delivered

**Before**: Manual agent scaffolding with custom scripts
**After**: Automated scaffolding with industry-standard pforge templates
**Impact**: Faster agent development, consistent structure, reduced boilerplate
**ROI**: High - Accelerates agent creation workflow, improves consistency

## Estimated Effort

3-4 days

## Notes

- Check pforge crate version and API stability before integration
- Consider caching templates locally for offline use
- Monitor pforge crate updates for breaking changes
- Provide migration guide from legacy scaffolding
- Document pforge-specific features and limitations
