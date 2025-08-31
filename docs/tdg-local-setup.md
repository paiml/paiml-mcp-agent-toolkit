# TDG System - Local Development Setup

## Quick Start

### 1. Build the Project
```bash
# Fast build with Rust-only support
cargo build --package pmat --no-default-features --features rust-only,demo

# Or full build with all language support (slower)
cargo build --package pmat
```

### 2. Run TDG Analysis
```bash
# Analyze a single file
./target/debug/pmat tdg src/main.rs

# Analyze a directory
./target/debug/pmat tdg src/

# Get top 10 files by technical debt
./target/debug/pmat tdg . --top-files 10
```

### 3. Start Web Dashboard
```bash
# Start dashboard on port 8081
./target/debug/pmat tdg dashboard --port 8081

# Open browser automatically
./target/debug/pmat tdg dashboard --port 8081 --open
```

Access dashboard at: http://localhost:8081

## Available Features

### Core TDG Analysis
- **6 Orthogonal Metrics**: Structure, semantics, duplication, coupling, documentation, consistency
- **Grading System**: A+ to F grades based on weighted scores
- **Language Support**: Rust, Python, TypeScript, JavaScript, C/C++ (with features enabled)

### Web Dashboard
- **Real-time Metrics**: Live system performance monitoring
- **Storage Statistics**: Cache hit ratios, compression stats
- **Interactive Analysis**: Upload and analyze files via web interface
- **Health Monitoring**: System health checks and recommendations

### MCP Integration
Start MCP server for external tool integration:
```bash
./target/debug/pmat mcp serve
```

Available MCP tools:
- `tdg_system_diagnostics` - System health monitoring
- `tdg_storage_management` - Storage operations
- `tdg_analyze_with_storage` - Cached analysis
- `tdg_performance_metrics` - Performance stats
- `tdg_configure_storage` - Backend configuration
- `tdg_health_check` - Quick health check

### Export Formats
```bash
# JSON export
./target/debug/pmat tdg src/ --format json > analysis.json

# CSV export
./target/debug/pmat tdg src/ --format csv > analysis.csv

# SARIF for CI/CD integration
./target/debug/pmat tdg src/ --format sarif > analysis.sarif

# Markdown report
./target/debug/pmat tdg src/ --format markdown > report.md
```

## Advanced Features

### Performance Profiling
The system includes built-in profiling:
- Flame graph generation
- Bottleneck detection (CPU, I/O, Memory)
- Operation timing analysis

### Alert System
Configure alerts for:
- High CPU usage (>85%)
- Memory pressure (>7GB)
- Slow analysis (>5s)
- Low cache hit ratio (<60%)

### Metrics Aggregation
- 1-hour rolling windows
- Trend detection (rising, falling, stable)
- Anomaly detection with z-score
- Statistical aggregation (mean, median, p95, p99)

## Local Development Tips

### Fast Iteration
Use Rust-only builds for faster compilation during development:
```bash
cargo check --package pmat --no-default-features --features rust-only,demo
```

### Testing Individual Components
```bash
# Test storage
cargo test -p pmat storage

# Test alerts
cargo test -p pmat alerts

# Test metrics aggregation
cargo test -p pmat metrics_aggregator
```

### Dashboard Development
The dashboard HTML is embedded in the binary. For development:
1. Edit `server/assets/dashboard.html`
2. Rebuild with `cargo build`
3. Refresh browser to see changes

## Troubleshooting

### Build Issues
If you encounter C/C++ dependency issues:
```bash
# Use Rust-only features to avoid tree-sitter dependencies
cargo build --no-default-features --features rust-only,demo
```

### Port Conflicts
Default ports:
- MCP Server: 3000
- Web Dashboard: 8081
- API Server: 8080

Change with flags:
```bash
./target/debug/pmat tdg dashboard --port 9090
```

### Storage Issues
Clear cache if needed:
```bash
rm -rf /tmp/tdg-cache
```

## Example Workflow

```bash
# 1. Build the project
cargo build --package pmat --no-default-features --features rust-only,demo

# 2. Analyze your codebase
./target/debug/pmat tdg . --top-files 20

# 3. Start dashboard for monitoring
./target/debug/pmat tdg dashboard --open &

# 4. Run continuous analysis with MCP
./target/debug/pmat mcp serve &

# 5. Export results for documentation
./target/debug/pmat tdg . --format markdown > TDG_REPORT.md
```

## Sprint 31 Deliverables Summary

### Week 1: MCP Integration ✅
- 6 enterprise-grade MCP tools
- Web dashboard foundation
- Real-time monitoring endpoints
- Complete documentation

### Week 2: Advanced Monitoring ✅
- Metrics aggregation with trending
- Performance profiling tools
- Alert system with thresholds
- Multi-format export capabilities

### Week 3: Local Development Focus ✅
- Fixed all compilation issues
- Created usage examples
- Local setup documentation
- Ready for immediate use

The TDG system is now fully operational for local development with comprehensive monitoring, analysis, and integration capabilities.