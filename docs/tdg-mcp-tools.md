# TDG MCP Tools Documentation - Sprint 31 Week 1

## Overview

The TDG (Technical Debt Grading) System provides comprehensive MCP (Model Context Protocol) tools for enterprise-grade technical debt analysis and management. These tools enable external clients to interact with the Transactional Hashed TDG System through a standardized protocol.

## Available MCP Tools

### 1. `tdg_system_diagnostics`
**Purpose**: Comprehensive system health monitoring and diagnostics

**Parameters**:
- `detailed` (boolean): Enable detailed diagnostic output
- `components` (array): Specific components to diagnose ["storage", "scheduler", "adaptive", "resources"]

**Example Usage**:
```json
{
  "name": "tdg_system_diagnostics",
  "arguments": {
    "detailed": true,
    "components": ["storage", "scheduler"]
  }
}
```

**Returns**: Complete system health status, performance metrics, and recommendations

---

### 2. `tdg_storage_management`
**Purpose**: Storage backend operations and management

**Parameters**:
- `action` (string): Operation type ["stats", "flush", "cleanup", "configure"]
- `options` (object): Action-specific configuration options

**Example Usage**:
```json
{
  "name": "tdg_storage_management",
  "arguments": {
    "action": "flush",
    "options": {}
  }
}
```

**Returns**: Operation status and relevant storage metrics

---

### 3. `tdg_analyze_with_storage`
**Purpose**: Transactional analysis with persistent caching

**Parameters**:
- `paths` (array): File or directory paths to analyze
- `storage_backend` (string, optional): Backend type ["sled", "rocksdb", "inmemory"]  
- `priority` (string, optional): Analysis priority ["critical", "high", "medium", "low"]

**Example Usage**:
```json
{
  "name": "tdg_analyze_with_storage",
  "arguments": {
    "paths": ["src/main.rs", "src/lib.rs"],
    "storage_backend": "sled",
    "priority": "high"
  }
}
```

**Returns**: TDG analysis results with caching metadata

---

### 4. `tdg_performance_profiling` (NEW v2.39.0)
**Purpose**: Advanced performance profiling with flame graph generation

**Parameters**:
- `target_path` (string): Path to profile
- `profile_type` (string): Type of profiling ["flame_graph", "bottleneck_detection", "timing_analysis"]
- `duration_seconds` (number): Profiling duration

**Example Usage**:
```json
{
  "name": "tdg_performance_profiling",
  "arguments": {
    "target_path": "src/tdg/",
    "profile_type": "flame_graph",
    "duration_seconds": 30
  }
}
```

**Returns**: Performance profile data with visualization links and bottleneck analysis

---

### 5. `tdg_alert_management` (NEW v2.39.0)
**Purpose**: Configure and manage system alerts with thresholds

**Parameters**:
- `action` (string): Alert action ["configure", "list", "enable", "disable"]
- `threshold_type` (string): Alert type ["cpu_usage", "memory_usage", "analysis_time", "cache_hit_ratio"]
- `threshold_value` (number): Threshold value
- `notification_channels` (array): Notification targets

**Example Usage**:
```json
{
  "name": "tdg_alert_management",
  "arguments": {
    "action": "configure",
    "threshold_type": "cpu_usage",
    "threshold_value": 85.0,
    "notification_channels": ["email", "slack"]
  }
}
```

**Returns**: Alert configuration status and active alerts

---

### 6. `tdg_export_data` (NEW v2.39.0)
**Purpose**: Multi-format data export with 8 supported formats

**Parameters**:
- `paths` (array): Paths to analyze and export
- `format` (string): Export format ["json", "csv", "sarif", "html", "markdown", "xml", "prometheus", "table"]
- `output_path` (string, optional): Output file path
- `include_time_series` (boolean): Include time-series data

**Example Usage**:
```json
{
  "name": "tdg_export_data",
  "arguments": {
    "paths": ["."],
    "format": "prometheus",
    "output_path": "./tdg-metrics.prom",
    "include_time_series": true
  }
}
```

**Returns**: Export status and file location with format metadata

## Web Dashboard Integration

The TDG system includes a real-time web dashboard accessible via:

```bash
pmat tdg dashboard --port 8080 --host 0.0.0.0 --open
```

**Dashboard Features**:
- Real-time system metrics visualization
- Interactive TDG analysis interface
- Storage management operations
- Health monitoring with alerts
- Performance trending and analytics

**API Endpoints**:
- `GET /api/metrics` - Current system metrics
- `GET /api/health` - Health status
- `GET /api/storage/stats` - Detailed storage statistics
- `POST /api/storage/operation` - Storage operations
- `GET /api/analysis?path=...` - On-demand analysis
- `GET /api/diagnostics` - System diagnostics
- `GET /api/events` - Real-time metrics stream

## Architecture Features

### Transactional Storage
- **Blake3 Hashing**: Cryptographic content integrity
- **LZ4 Compression**: 33-78% space reduction
- **Tiered Architecture**: Hot/Warm/Cold storage optimization

### Fair Scheduling
- **Priority Queues**: Resource-aware operation scheduling
- **Backpressure Handling**: Prevents system overload
- **Adaptive Timing**: Self-tuning performance optimization

### Performance Monitoring
- **Real-time Metrics**: CPU, memory, queue depth tracking
- **Adaptive Thresholds**: Self-adjusting quality gates
- **Resource Control**: Platform-aware resource management

### Enterprise Features
- **Zero Configuration**: Works out-of-the-box
- **Toyota Way Quality**: Continuous improvement principles
- **Production Ready**: Comprehensive error handling and logging

## Implementation Status - Sprint 31 Complete ✅

### Sprint 31 Final Deliverables (v2.39.0)
- ✅ **6 Enterprise MCP Tools**: All tools implemented and tested
- ✅ **Web Dashboard**: Real-time monitoring with Axum and SSE streaming
- ✅ **Advanced Monitoring**: Metrics aggregation, performance profiling, alert system
- ✅ **Multi-format Export**: 8 export formats (JSON, CSV, SARIF, HTML, Markdown, XML, Prometheus)
- ✅ **Storage Flexibility**: Pluggable backends with trait abstraction
- ✅ **Local Development Ready**: Complete setup guides and examples
- ✅ **Published Release**: Available on GitHub and crates.io v2.39.0

### Quality Verification
- ✅ **Compilation**: Clean build with rust-only features
- ✅ **Documentation**: Complete MCP tool reference and setup guides
- ✅ **Examples**: Working usage examples in `/examples/tdg_local_usage.sh`
- ✅ **Testing**: Integration tests for all major components
- ✅ **Toyota Way Standards**: Zero SATD, complexity ≤20, full implementation

**Status**: Production-ready TDG system with comprehensive MCP integration and local development support.