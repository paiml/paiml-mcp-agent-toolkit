# Logging & Telemetry Guide

*Reference: SPECIFICATION.md Section 35*

## Overview

PMAT implements structured logging and telemetry using the `tracing` ecosystem for observability and debugging.

## Logging Levels

### Configuration

Set via environment variable or config:

```bash
# Environment variable
export RUST_LOG=pmat=debug,mcp=trace

# Or in pmat.toml
[logging]
level = "info"
filter = "pmat=debug,mcp=trace"
```

### Log Levels

| Level | Usage | Example |
|-------|-------|---------|
| ERROR | Failures requiring attention | Quality violations, parse errors |
| WARN | Recoverable issues | Fallback to basic analysis |
| INFO | Normal operations | Analysis complete, files processed |
| DEBUG | Detailed flow | Function entry/exit, intermediate values |
| TRACE | Maximum detail | AST nodes, token streams |

## Structured Logging

### Basic Usage

```rust
use tracing::{info, warn, error, debug, trace};

#[tracing::instrument]
fn analyze_file(path: &Path) -> Result<Analysis> {
    info!(?path, "Starting analysis");
    
    let content = fs::read_to_string(path)?;
    debug!(bytes = content.len(), "File loaded");
    
    match parse(&content) {
        Ok(ast) => {
            trace!(?ast, "AST generated");
            Ok(analyze_ast(ast))
        }
        Err(e) => {
            error!(?e, ?path, "Parse failed");
            Err(e)
        }
    }
}
```

### Structured Fields

```rust
use tracing::info_span;

let span = info_span!(
    "analysis",
    file = %path.display(),
    project = ?project_name,
    version = env!("CARGO_PKG_VERSION")
);

let _guard = span.enter();
// All logs within scope include span fields
```

## Telemetry Integration

### OpenTelemetry Export

```rust
use tracing_subscriber::prelude::*;
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_telemetry() {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("pmat")
        .install_simple()
        .unwrap();
    
    tracing_subscriber::registry()
        .with(OpenTelemetryLayer::new(tracer))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

### Metrics Collection

```rust
use metrics::{counter, histogram, gauge};

fn record_analysis_metrics(result: &AnalysisResult) {
    counter!("analyses_total", 1);
    histogram!("analysis_duration_seconds", result.duration.as_secs_f64());
    gauge!("complexity_max", result.max_complexity as f64);
    
    if result.has_violations() {
        counter!("quality_violations_total", 1, 
            "type" => result.violation_type());
    }
}
```

## Performance Monitoring

### Timing Operations

```rust
use tracing::{info_span, Instrument};
use std::time::Instant;

async fn analyze_project(path: &Path) -> Result<ProjectAnalysis> {
    async move {
        let start = Instant::now();
        let result = do_analysis(path).await?;
        
        info!(
            duration_ms = start.elapsed().as_millis(),
            files = result.file_count,
            "Analysis complete"
        );
        
        Ok(result)
    }
    .instrument(info_span!("analyze_project", ?path))
    .await
}
```

### Resource Tracking

```rust
use sysinfo::{System, SystemExt, ProcessExt};

fn log_resource_usage() {
    let mut sys = System::new();
    sys.refresh_processes();
    
    if let Some(process) = sys.process(std::process::id() as i32) {
        debug!(
            memory_mb = process.memory() / 1024 / 1024,
            cpu_percent = process.cpu_usage(),
            "Resource usage"
        );
    }
}
```

## Log Output Formats

### Human-Readable (Default)

```
2025-08-20T10:30:45.123Z  INFO pmat::analyze: Starting analysis path="src/main.rs"
2025-08-20T10:30:45.234Z DEBUG pmat::analyze: File loaded bytes=1234
```

### JSON (Machine-Readable)

```json
{
  "timestamp": "2025-08-20T10:30:45.123Z",
  "level": "INFO",
  "target": "pmat::analyze",
  "message": "Starting analysis",
  "path": "src/main.rs"
}
```

Configure with:
```rust
tracing_subscriber::fmt()
    .json()
    .init();
```

## MCP Request Tracing

Track MCP requests through the system:

```rust
#[tracing::instrument(skip(params))]
async fn handle_mcp_request(
    method: &str,
    params: Value,
    request_id: u64,
) -> Result<Value> {
    let span = info_span!(
        "mcp_request",
        method,
        request_id,
        trace_id = %Uuid::new_v4()
    );
    
    async move {
        info!("Processing MCP request");
        let result = dispatch_method(method, params).await?;
        info!("MCP request complete");
        Ok(result)
    }
    .instrument(span)
    .await
}
```

## Debug Features

### Verbose Mode

Enable with CLI flag:
```bash
pmat analyze complexity -v        # INFO level
pmat analyze complexity -vv       # DEBUG level
pmat analyze complexity -vvv      # TRACE level
```

### Debug Assertions

```rust
#[cfg(debug_assertions)]
{
    debug_assert!(complexity >= 0, "Complexity cannot be negative");
    trace!("Debug assertion passed");
}
```

## Log Rotation

For long-running MCP server:

```rust
use tracing_appender::rolling;

let file_appender = rolling::daily("logs", "pmat.log");
let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .init();
```

## Privacy Considerations

Sanitize sensitive data:

```rust
fn log_file_analysis(path: &Path) {
    // Don't log full paths in production
    let display_path = if cfg!(debug_assertions) {
        path.to_string_lossy().to_string()
    } else {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    };
    
    info!(file = %display_path, "Analyzing file");
}
```

## Best Practices

1. **Use structured logging**: Include context as fields
2. **Instrument key functions**: Add `#[tracing::instrument]`
3. **Log at boundaries**: Entry/exit of major operations
4. **Include request IDs**: Track requests through system
5. **Sanitize sensitive data**: Don't log passwords, tokens
6. **Use appropriate levels**: ERROR for failures, INFO for operations
7. **Add metrics**: Count operations, time durations

## Related Documentation

- [Error Handling](./error-handling.md)
- [Configuration](./configuration.md)
- [SPECIFICATION.md Section 35](../SPECIFICATION.md#35-logging-telemetry)