# PMAT Agent Architecture

PMAT includes a comprehensive sub-agent system built on enterprise-grade distributed computing principles.

## Overview

The PMAT Agent System provides autonomous code analysis, transformation, and quality enforcement through a distributed actor-based architecture.

## Core Agent Types

### 1. AnalyzerActor
- **Purpose**: Code analysis and metrics collection
- **Capabilities**:
  - Complexity analysis (cyclomatic, cognitive)
  - Entropy calculation (Shannon entropy)
  - SATD detection (Self-Admitted Technical Debt)
  - Multi-language support

### 2. TransformerActor
- **Purpose**: Code transformation and refactoring
- **Capabilities**:
  - AST-based transformations
  - Code restructuring
  - Pattern-based refactoring
  - Safe transformation verification

### 3. ValidatorActor
- **Purpose**: Code validation and quality checks
- **Capabilities**:
  - Quality gate enforcement
  - Rule-based validation
  - Compliance checking
  - Error detection and reporting

### 4. OrchestratorActor
- **Purpose**: Workflow coordination and agent orchestration
- **Capabilities**:
  - Multi-agent coordination
  - Workflow execution
  - Resource allocation
  - Error handling and recovery

## System Architecture

### Actor Framework
- **Engine**: Actix Actor System
- **Messaging**: Zero-copy message passing
- **State**: Event sourcing with snapshots
- **Consensus**: Raft protocol for distributed coordination
- **Resources**: CPU/Memory/GPU/Network/IO control

### Agent Registry
```rust
pub struct AgentRegistry {
    agents: Arc<DashMap<AgentId, AgentEntry>>,
    agents_by_name: Arc<DashMap<String, Arc<dyn Any + Send + Sync>>>,
}
```

Features:
- Dynamic agent registration and discovery
- Name-based and ID-based lookup
- Thread-safe concurrent access
- Hot-swappable agent implementations

### MCP Integration

**Model Context Protocol (MCP) Server**:
- Full MCP specification compliance
- Multiple transport modes:
  - TCP networking
  - Unix domain sockets
  - stdio transport
- Request/response and streaming support
- Configurable connection limits and timeouts

### Workflow Engine

**DSL-Based Workflows**:
```yaml
name: "Code Quality Pipeline"
version: "1.0"
steps:
  - name: "analyze"
    agent: "analyzer"
    config:
      languages: ["rust", "typescript"]

  - name: "transform"
    agent: "transformer"
    depends_on: ["analyze"]

  - name: "validate"
    agent: "validator"
    depends_on: ["transform"]
```

Features:
- YAML/JSON workflow definitions
- Step dependencies and parallelization
- Context sharing between steps
- Timeout and error handling
- Parameter passing and variable substitution

## CLI Interface

### Primary Command
```bash
pmat agent <subcommand>
```

### Subcommands

#### Daemon Management
```bash
pmat agent start         # Start background daemon
pmat agent stop          # Stop background daemon
pmat agent status        # Check daemon status
pmat agent reload        # Reload daemon configuration
```

#### Project Monitoring
```bash
pmat agent monitor <path>      # Start monitoring project
pmat agent unmonitor <path>    # Stop monitoring project
```

#### Quality Operations
```bash
pmat agent quality-gate <path>  # Run quality gates
pmat agent health               # System health check
```

#### MCP Server
```bash
pmat agent mcp-server          # Start MCP server for testing
```

## Standalone Agent Binary

**`pmat-agent`** - Extended enterprise agent with additional capabilities:

### Server Mode
```bash
pmat-agent serve --bind 127.0.0.1:3000    # TCP server
pmat-agent serve --socket /tmp/pmat.sock  # Unix socket
pmat-agent serve --stdio                  # stdio transport
```

### Workflow Execution
```bash
pmat-agent execute --file workflow.yaml --params '{"lang": "rust"}'
pmat-agent validate --file workflow.yaml
```

### Direct Analysis
```bash
pmat-agent analyze --path src/ --language rust --output json
pmat-agent quality-gate --path src/ --max-complexity 10 --fail-on-violation
```

### System Information
```bash
pmat-agent info    # Show system capabilities and version
```

## Quality Analysis Capabilities

### Complexity Analysis
- **Cyclomatic Complexity**: Control flow analysis
- **Cognitive Complexity**: Human comprehension difficulty
- **Nesting Depth**: Code structure analysis
- **Function Length**: Size-based metrics

### Technical Debt Detection
- **SATD Items**: Self-admitted technical debt comments
- **TODO/FIXME**: Development markers
- **Pattern Recognition**: Debt pattern detection
- **Severity Classification**: Critical/High/Medium/Low

### Code Quality Gates
```rust
pub struct QualityThresholds {
    max_complexity: u32,
    max_satd_items: usize,
    min_test_coverage: f64,
    max_duplication: f64,
}
```

Configurable thresholds with:
- Complexity limits
- SATD item counts
- Test coverage requirements
- Code duplication detection

## Enterprise Features

### Distributed Computing
- **Raft Consensus**: Multi-node coordination
- **Event Sourcing**: Full audit trail
- **Snapshots**: Efficient state recovery
- **Fault Tolerance**: Automatic failover

### Resource Management
- **CPU Throttling**: Prevent system overload
- **Memory Limits**: Bounded resource usage
- **GPU Allocation**: ML/AI workload support
- **Network QoS**: Bandwidth management
- **IO Prioritization**: Disk access control

### Monitoring & Observability
- **Health Checks**: System status monitoring
- **Metrics Collection**: Performance tracking
- **Distributed Tracing**: Request flow analysis
- **Log Aggregation**: Centralized logging

## Integration Examples

### MCP Client Connection
```typescript
import { Client } from '@modelcontextprotocol/sdk/client/index.js';

const client = new Client({
  name: "pmat-integration",
  version: "1.0.0"
}, {
  capabilities: {}
});

await client.connect(transport);
```

### Workflow Integration
```yaml
name: "CI/CD Quality Pipeline"
version: "1.0"
trigger: "on_push"
steps:
  - name: "complexity_check"
    agent: "analyzer"
    config:
      max_cyclomatic: 10
      max_cognitive: 15

  - name: "debt_analysis"
    agent: "analyzer"
    config:
      satd_threshold: 5

  - name: "auto_refactor"
    agent: "transformer"
    condition: "complexity_check.violations > 0"

  - name: "validate_changes"
    agent: "validator"
    depends_on: ["auto_refactor"]
```

### Quality Gate Integration
```rust
use pmat::quality::gate_runner::{QualityGateRunner, QualityThresholds};

let gate = QualityGateRunner::new(
    vec![
        Box::new(ComplexityAnalyzer::default()),
        Box::new(SatdDetectorWithItems::new()),
    ],
    QualityThresholds {
        max_complexity: 10,
        max_satd_items: 0,
        min_test_coverage: 80.0,
        max_duplication: 0.1,
    },
);

let result = gate.check(&code, &language).await;
```

## Configuration

### Agent Configuration
```toml
[agent]
max_concurrent_tasks = 100
health_check_interval = "30s"
snapshot_interval = "5m"

[agent.resources]
max_cpu_percent = 80.0
max_memory_mb = 2048
max_gpu_memory_mb = 1024

[agent.mcp]
bind_address = "127.0.0.1:3000"
max_connections = 50
request_timeout = "30s"

[agent.quality]
default_complexity_threshold = 10
default_satd_threshold = 0
auto_refactor_enabled = true
```

### Workflow Configuration
```yaml
defaults:
  timeout: "5m"
  retry_count: 3

agents:
  analyzer:
    max_parallel: 10
    timeout: "2m"

  transformer:
    max_parallel: 5
    timeout: "10m"

  validator:
    max_parallel: 20
    timeout: "1m"
```

## Performance Characteristics

### Throughput
- **Agent Messaging**: 1M+ messages/second
- **Workflow Execution**: 100+ concurrent workflows
- **Quality Analysis**: 1000+ files/minute
- **MCP Requests**: 10K+ requests/second

### Scalability
- **Horizontal**: Multi-node cluster support
- **Vertical**: Multi-core CPU utilization
- **Resource**: Dynamic resource allocation
- **Load**: Automatic load balancing

### Reliability
- **Uptime**: 99.9%+ availability target
- **Recovery**: Sub-second failover
- **Consistency**: Strong consistency guarantees
- **Durability**: Persistent state storage

## Future Roadmap

### Planned Features
- **WebAssembly Agents**: Portable cross-platform execution
- **Machine Learning**: AI-powered code analysis
- **Real-time Collaboration**: Multi-developer workflows
- **Cloud Integration**: Managed cloud deployment
- **IDE Extensions**: Direct editor integration

### Protocol Extensions
- **MCP 2.0**: Next-generation protocol features
- **GraphQL API**: Flexible query interface
- **gRPC Support**: High-performance RPC
- **WebSocket Streams**: Real-time updates

This architecture provides the foundation for scalable, enterprise-grade code quality automation with autonomous agent capabilities.