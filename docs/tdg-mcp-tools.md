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

### 4. `tdg_performance_metrics`
**Purpose**: Real-time performance and adaptive threshold monitoring

**Parameters**:
- `include_history` (boolean): Include historical metrics
- `metrics` (array): Specific metrics to retrieve

**Example Usage**:
```json
{
  "name": "tdg_performance_metrics",
  "arguments": {
    "include_history": false,
    "metrics": ["analysis_time", "queue_depth"]
  }
}
```

**Returns**: Current performance statistics and adaptive threshold status

---

### 5. `tdg_configure_storage`
**Purpose**: Dynamic storage backend configuration

**Parameters**:
- `backend_type` (string): Target backend ["sled", "rocksdb", "inmemory"]
- `cache_size_mb` (number): Cache size in megabytes
- `compression` (boolean): Enable/disable compression
- `path` (string, optional): Storage path

**Example Usage**:
```json
{
  "name": "tdg_configure_storage",
  "arguments": {
    "backend_type": "rocksdb",
    "cache_size_mb": 256,
    "compression": true,
    "path": "/data/tdg"
  }
}
```

**Returns**: Configuration status and new storage settings

---

### 6. `tdg_health_check`
**Purpose**: Quick system health assessment

**Parameters**:
- `include_recommendations` (boolean): Include improvement recommendations
- `check_storage` (boolean): Include storage health check
- `check_performance` (boolean): Include performance health check

**Example Usage**:
```json
{
  "name": "tdg_health_check",
  "arguments": {
    "include_recommendations": true,
    "check_storage": true,
    "check_performance": true
  }
}
```

**Returns**: Overall system health status with actionable recommendations

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

## Implementation Status - Sprint 31 Week 1 ✅

- ✅ **MCP Tools Registered**: All 6 TDG tools integrated into server
- ✅ **Handler Implementation**: Complete pmcp ToolHandler trait implementation
- ✅ **Web Dashboard Foundation**: Axum-based real-time dashboard with HTML/CSS/JS
- ✅ **Real-time Monitoring**: Server-Sent Events simulation for metrics streaming
- ✅ **CLI Integration**: Dashboard command with browser auto-open
- ✅ **Compilation Success**: Full build compatibility with rust-only features
- ✅ **Quality Verification**: Zero critical errors, warnings only

**Next Sprint Focus**: Enhanced monitoring, advanced analytics, and production deployment optimizations.