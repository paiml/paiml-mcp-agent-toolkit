# Error Handling Guide

*Reference: SPECIFICATION.md Section 34*

## Overview

PMAT follows Rust's Result-based error handling with structured error types and comprehensive error context. This guide documents our error handling patterns and recovery strategies.

## Error Types

### Core Error Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum PmatError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Analysis failed: {0}")]
    Analysis(String),
    
    #[error("Quality violation: {0}")]
    QualityViolation(String),
    
    #[error("MCP protocol error: {0}")]
    Protocol(#[from] serde_json::Error),
}
```

### Error Context

Using `anyhow` for context chaining:

```rust
use anyhow::{Context, Result};

fn analyze_file(path: &Path) -> Result<Analysis> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    parse_code(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    
    Ok(analysis)
}
```

## Error Recovery Strategies

### 1. Graceful Degradation

When non-critical operations fail:

```rust
// Try advanced analysis, fall back to basic
match perform_deep_analysis(&code) {
    Ok(deep) => AnalysisResult::Deep(deep),
    Err(e) => {
        log::warn!("Deep analysis failed: {}, using basic", e);
        AnalysisResult::Basic(perform_basic_analysis(&code)?)
    }
}
```

### 2. Retry Logic

For transient failures:

```rust
use backoff::{ExponentialBackoff, retry};

retry(ExponentialBackoff::default(), || {
    fetch_remote_data()
        .map_err(|e| match e {
            Error::Network(_) => backoff::Error::Transient(e),
            _ => backoff::Error::Permanent(e),
        })
}).context("Failed after retries")?;
```

### 3. Fail-Fast

For quality violations:

```rust
if complexity > threshold {
    return Err(PmatError::QualityViolation(
        format!("Complexity {} exceeds threshold {}", complexity, threshold)
    ));
}
```

## Exit Codes

Consistent exit codes for CLI:

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | Analysis complete |
| 1 | Quality violation | Complexity exceeded |
| 2 | Input error | File not found |
| 3 | Parse error | Invalid syntax |
| 4 | System error | Out of memory |

Implementation:

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(match e {
            PmatError::QualityViolation(_) => 1,
            PmatError::Io(_) => 2,
            PmatError::Parse(_) => 3,
            _ => 4,
        });
    }
}
```

## MCP Error Handling

JSON-RPC error responses:

```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "error": {
        "code": -32602,
        "message": "Invalid params",
        "data": {
            "details": "Missing required field: file_path"
        }
    }
}
```

Error codes:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

## Logging Errors

Structured error logging:

```rust
use tracing::{error, warn, info};

match operation() {
    Ok(result) => {
        info!("Operation succeeded");
        result
    }
    Err(e) if e.is_recoverable() => {
        warn!("Recoverable error: {}", e);
        fallback()
    }
    Err(e) => {
        error!("Fatal error: {:#}", e);
        return Err(e);
    }
}
```

## User-Friendly Messages

Convert technical errors to actionable messages:

```rust
impl fmt::Display for PmatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(e) if e.kind() == io::ErrorKind::NotFound => {
                write!(f, "File not found. Check the path and try again.")
            }
            Self::QualityViolation(msg) => {
                write!(f, "Quality check failed: {}. Run 'pmat refactor auto' to fix.", msg)
            }
            _ => write!(f, "{}", self),
        }
    }
}
```

## Testing Error Paths

```rust
#[test]
fn test_handles_missing_file() {
    let result = analyze_file(Path::new("/nonexistent"));
    assert!(matches!(result, Err(PmatError::Io(_))));
}

#[test]
fn test_quality_violation_exit_code() {
    let error = PmatError::QualityViolation("Too complex".into());
    assert_eq!(error.exit_code(), 1);
}
```

## Best Practices

1. **Use Result everywhere**: No unwrap() in production
2. **Add context**: Use .context() for better error messages
3. **Log at boundaries**: Log errors at API boundaries
4. **Fail fast for quality**: Don't continue with violations
5. **Recover when possible**: Degrade gracefully
6. **Test error paths**: Cover error cases in tests

## Related Documentation

- [Logging & Telemetry](./telemetry.md)
- [Configuration](./configuration.md)
- [SPECIFICATION.md Section 34](../SPECIFICATION.md#34-error-handling)