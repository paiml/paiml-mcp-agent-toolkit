# Claude Code Agent Mode - User Guide

## Overview

The PMAT Claude Code Agent transforms your development environment into an intelligent, always-working quality monitoring system. It seamlessly integrates with Claude Code through the Model Context Protocol (MCP), providing real-time code quality analysis, Toyota Way compliance enforcement, and AI-driven refactoring suggestions.

## Features

### 🚀 Core Capabilities

- **Real-time Quality Monitoring**: Continuous file system watching with instant feedback
- **Toyota Way Compliance**: Enforces ≤20 complexity standard with zero SATD tolerance
- **MCP Integration**: Native Claude Code integration via stdio transport
- **Persistent State**: Maintains monitoring state across restarts
- **Production Ready**: Environment-specific configurations for dev, prod, and CI/CD

### 🛠️ Available Tools

The agent exposes the following tools through MCP:

1. **start_quality_monitoring**: Begin monitoring a project
2. **stop_quality_monitoring**: Stop monitoring a project
3. **get_quality_status**: Get current quality metrics
4. **run_quality_gates**: Execute comprehensive quality checks
5. **analyze_complexity**: Analyze code complexity
6. **health_check**: Check agent health status

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-org/paiml-mcp-agent-toolkit.git
cd paiml-mcp-agent-toolkit

# Build the agent
cargo build --release --package pmat

# The binary will be at ./target/release/pmat
```

### Using Cargo

```bash
cargo install pmat
```

## Quick Start

### 1. Start the MCP Server (for Claude Code)

```bash
# Start in MCP mode for Claude Code integration
pmat agent mcp-server

# With custom configuration
pmat agent mcp-server --config configs/agent-development.toml

# Debug mode for troubleshooting
pmat agent mcp-server --debug
```

### 2. Start as Background Daemon

```bash
# Start the daemon
pmat agent start --project-path /path/to/project

# Check status
pmat agent status

# Stop the daemon
pmat agent stop
```

### 3. One-shot Analysis (CI/CD)

```bash
# Run quality gates on a project
pmat quality-gate --path /path/to/project

# Analyze complexity
pmat analyze complexity --path /path/to/project --top-files 10
```

## Configuration

### Configuration Files

PMAT provides three pre-configured templates:

- `configs/agent-development.toml` - Development environment
- `configs/agent-production.toml` - Production deployment
- `configs/agent-ci.toml` - CI/CD integration

### Key Configuration Options

```toml
[agent]
complexity_threshold = 20  # Toyota Way standard
satd_enabled = true       # Zero tolerance for technical debt
check_interval_seconds = 60

[quality_monitor]
debounce_ms = 2000
watch_patterns = ["*.rs", "*.ts", "*.js", "*.py"]
ignore_patterns = ["target/", "node_modules/", ".git/"]

[persistence]
state_dir = "/var/lib/pmat-agent/state"
auto_save_interval_secs = 300
history_retention_days = 30
```

## Claude Code Integration

### Setting up Claude Code

1. **Configure MCP in Claude Code settings**:

```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["agent", "mcp-server"],
      "env": {}
    }
  }
}
```

2. **Available Commands in Claude Code**:

- "Start monitoring my project"
- "Check quality status"
- "Run quality gates on src/"
- "Analyze complexity of main.rs"
- "Show me quality trends"

### Example Interactions

```typescript
// Claude Code will have access to these tools:

// Start monitoring
await use_mcp_tool("pmat", "start_quality_monitoring", {
  project_path: "/path/to/project",
  project_id: "my-project"
});

// Run quality gates
const result = await use_mcp_tool("pmat", "run_quality_gates", {
  target_path: "./src"
});

// Get complexity analysis
const complexity = await use_mcp_tool("pmat", "analyze_complexity", {
  file_path: "./src/main.rs"
});
```

## Deployment

### Systemd Service (Linux)

Create `/etc/systemd/system/pmat-agent.service`:

```ini
[Unit]
Description=PMAT Claude Code Agent
After=network.target

[Service]
Type=simple
User=pmat
Group=pmat
WorkingDirectory=/opt/pmat-agent
ExecStart=/usr/local/bin/pmat agent start --config /etc/pmat/agent-production.toml
ExecStop=/usr/local/bin/pmat agent stop
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable pmat-agent
sudo systemctl start pmat-agent
```

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --package pmat

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pmat /usr/local/bin/
COPY configs/agent-production.toml /etc/pmat/config.toml

EXPOSE 9090
CMD ["pmat", "agent", "mcp-server", "--config", "/etc/pmat/config.toml"]
```

## Monitoring & Observability

### Metrics

The agent exposes Prometheus metrics on port 9090:

- `pmat_analyses_total` - Total analyses performed
- `pmat_violations_detected` - Violations by type
- `pmat_quality_score` - Current quality score per project
- `pmat_processing_duration_seconds` - Analysis performance

### Logging

Configure logging in your config file:

```toml
[logging]
level = "info"  # debug, info, warn, error
format = "json" # json or pretty
show_timestamp = true

[logging.rotation]
enabled = true
max_size_mb = 100
max_files = 10
```

### Health Checks

```bash
# CLI health check
pmat agent status

# HTTP health endpoint (when metrics enabled)
curl http://localhost:9090/health
```

## Best Practices

### 1. Project Structure

- Keep configuration in version control
- Use `.pmatignore` for custom ignore patterns
- Set appropriate watch patterns for your tech stack

### 2. Quality Gates

- Start with warning mode before enforcing
- Gradually tighten thresholds
- Document exceptions with clear rationale

### 3. CI/CD Integration

```yaml
# GitHub Actions example
- name: Run PMAT Quality Gates
  run: |
    pmat quality-gate --path . --config configs/agent-ci.toml
  continue-on-error: false
```

### 4. Performance Tuning

For large codebases:

```toml
[performance]
max_concurrent_analyses = 4
cache_size_mb = 500
analysis_timeout_secs = 60

[performance.limits]
max_cpu_percent = 50
max_io_operations_per_sec = 1000
```

## Troubleshooting

### Common Issues

1. **MCP Connection Failed**
   - Check Claude Code MCP settings
   - Verify `pmat` is in PATH
   - Run with `--debug` flag

2. **High Memory Usage**
   - Adjust `max_memory_mb` in config
   - Reduce `cache_size_mb`
   - Increase `debounce_ms`

3. **Slow Analysis**
   - Check ignore patterns
   - Reduce watch depth
   - Increase concurrent analyses

### Debug Mode

```bash
# Enable debug logging
pmat agent mcp-server --debug

# Check logs
tail -f /var/log/pmat-agent/agent.log

# Test MCP protocol
python test-mcp-client.py
```

## API Reference

### MCP Protocol

The agent implements MCP 2024-11-05 specification:

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "run_quality_gates",
    "arguments": {
      "target_path": "./src"
    }
  },
  "id": 1
}
```

### Quality Metrics

```typescript
interface QualityMetrics {
  avg_complexity: number;      // Average function complexity
  max_complexity: number;      // Maximum complexity found
  satd_count: number;         // SATD violations count
  dead_code_percentage: number; // Percentage of dead code
  quality_score: number;      // Overall score (0-100)
  files_analyzed: number;     // Total files analyzed
  total_violations: number;   // All violations found
}
```

## Support

- **Documentation**: https://docs.pmat.dev
- **Issues**: https://github.com/your-org/paiml-mcp-agent-toolkit/issues
- **Discord**: https://discord.gg/pmat
- **Email**: support@pmat.dev

## License

MIT License - See LICENSE file for details

## Contributing

We welcome contributions! Please see CONTRIBUTING.md for guidelines.

---

*Built with ❤️ following the Toyota Way principles*