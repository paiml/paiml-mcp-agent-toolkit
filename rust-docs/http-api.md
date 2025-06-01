# HTTP REST API Reference

## Overview

The PAIML MCP Agent Toolkit provides a comprehensive HTTP REST API that exposes all functionality available through the CLI and MCP interfaces. The API follows RESTful principles and supports both JSON and query parameter inputs.

## Getting Started

### Starting the HTTP Server

```bash
# Start server with default settings (port 3000)
paiml-mcp-agent-toolkit serve

# Start with custom port and CORS enabled
paiml-mcp-agent-toolkit serve --port 8080 --cors

# Custom host and port
paiml-mcp-agent-toolkit serve --host 0.0.0.0 --port 8080
```

### Base URL Structure

```
http://{host}:{port}/api/v1/{endpoint}
```

**Default Configuration:**
- **Host**: `127.0.0.1` (localhost)
- **Port**: `3000`
- **Base Path**: `/api/v1`

## Authentication

Currently, the API does not require authentication. All endpoints are publicly accessible when the server is running.

## Content Types

**Request Headers:**
- `Content-Type: application/json` (for POST requests with JSON body)
- `Accept: application/json` (recommended for all requests)

**Response Format:**
- All responses return JSON with consistent structure
- Error responses follow standard HTTP status codes

## Error Handling

### Standard HTTP Status Codes

| Status | Meaning | Description |
|--------|---------|-------------|
| 200 | OK | Request successful |
| 400 | Bad Request | Invalid request parameters |
| 404 | Not Found | Endpoint or resource not found |
| 500 | Internal Server Error | Server processing error |

### Error Response Format

```json
{
    "error": {
        "code": 400,
        "message": "Invalid request parameters",
        "details": "Missing required parameter: project_path"
    }
}
```

## Core Endpoints

### Health Check

#### `GET /health`

Health check endpoint for monitoring server status.

**Request:**
```bash
curl http://localhost:8080/health
```

**Response:**
```json
{
    "status": "healthy",
    "server": "paiml-mcp-agent-toolkit",
    "version": "0.18.5",
    "uptime_seconds": 3600
}
```

### Metrics

#### `GET /metrics`

Server performance metrics and statistics.

**Request:**
```bash
curl http://localhost:8080/metrics
```

**Response:**
```json
{
    "requests_total": 1234,
    "requests_per_second": 5.2,
    "average_response_time_ms": 45.7,
    "memory_usage_mb": 125.3,
    "cache_hit_rate": 0.85
}
```

## Template Management

### List Templates

#### `GET /api/v1/templates`

List available templates with optional filtering.

**Query Parameters:**
- `toolchain` (optional): Filter by toolchain (`rust`, `deno`, `python-uv`)
- `category` (optional): Filter by category (`makefile`, `readme`, `gitignore`)

**Request:**
```bash
curl "http://localhost:8080/api/v1/templates?toolchain=rust&category=makefile"
```

**Response:**
```json
{
    "templates": [
        {
            "uri": "template://makefile/rust/cli",
            "name": "Rust CLI Makefile",
            "description": "Professional Makefile template for Rust CLI projects",
            "toolchain": "rust",
            "category": "makefile",
            "parameters": {
                "project_name": {
                    "type": "string",
                    "required": true,
                    "description": "Name of the project"
                },
                "has_tests": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include test targets"
                }
            }
        }
    ]
}
```

### Get Template Details

#### `GET /api/v1/templates/{id}`

Get detailed information about a specific template.

**Request:**
```bash
curl http://localhost:8080/api/v1/templates/makefile-rust-cli
```

**Response:**
```json
{
    "uri": "template://makefile/rust/cli",
    "name": "Rust CLI Makefile",
    "description": "Professional Makefile template for Rust CLI projects",
    "toolchain": "rust",
    "category": "makefile",
    "content_preview": "# Rust CLI Project Makefile\n...",
    "parameters": {
        "project_name": {
            "type": "string",
            "required": true,
            "description": "Name of the project"
        }
    }
}
```

### Generate Template

#### `POST /api/v1/generate`

Generate content from a template.

**Request Body:**
```json
{
    "template_uri": "template://makefile/rust/cli",
    "parameters": {
        "project_name": "my-awesome-cli",
        "has_tests": true,
        "has_benchmarks": false
    }
}
```

**Request:**
```bash
curl -X POST "http://localhost:8080/api/v1/generate" \
  -H "Content-Type: application/json" \
  -d '{
    "template_uri": "template://makefile/rust/cli",
    "parameters": {
        "project_name": "my-cli",
        "has_tests": true
    }
  }'
```

**Response:**
```json
{
    "generated_content": "# Rust CLI Project Makefile\n\nPROJECT_NAME := my-cli\n...",
    "template_uri": "template://makefile/rust/cli",
    "parameters_used": {
        "project_name": "my-cli",
        "has_tests": true
    }
}
```

## Code Analysis Endpoints

### Complexity Analysis

#### `GET /api/v1/analyze/complexity`

Analyze code complexity with query parameters.

**Query Parameters:**
- `project_path` (optional): Path to analyze (default: current directory)
- `toolchain` (optional): Override detected toolchain
- `format` (optional): Output format (`summary`, `full`, `json`, `sarif`)
- `max_cyclomatic` (optional): Custom cyclomatic complexity threshold
- `max_cognitive` (optional): Custom cognitive complexity threshold
- `top_files` (optional): Number of top complex files to show (0 = all violations)

**Request:**
```bash
curl "http://localhost:8080/api/v1/analyze/complexity?project_path=./&top_files=5&format=json"
```

#### `POST /api/v1/analyze/complexity`

Analyze code complexity with JSON body for complex parameters.

**Request Body:**
```json
{
    "project_path": "./",
    "toolchain": "rust",
    "format": "json",
    "top_files": 5,
    "max_cyclomatic": 15,
    "max_cognitive": 20,
    "include_patterns": ["src/**/*.rs", "tests/**/*.rs"]
}
```

**Response:**
```json
{
    "summary": {
        "total_files": 146,
        "total_functions": 15239,
        "median_cyclomatic": 2,
        "max_cyclomatic": 32,
        "violations": 12
    },
    "violations": [
        {
            "file": "./server/src/services/context.rs",
            "function": "analyze_project_context",
            "line": 145,
            "cyclomatic_complexity": 32,
            "cognitive_complexity": 28,
            "threshold_exceeded": "cyclomatic"
        }
    ],
    "top_files": {
        "requested": 5,
        "returned": 5,
        "rankings": [
            {
                "rank": 1,
                "file": "./server/src/services/context.rs",
                "function_count": 30,
                "max_cyclomatic": 32,
                "avg_cognitive": 5.8,
                "halstead_effort": 1950.0,
                "total_score": 30.92
            }
        ]
    }
}
```

### Code Churn Analysis

#### `POST /api/v1/analyze/churn`

Analyze git history for code churn metrics.

**Request Body:**
```json
{
    "project_path": "./",
    "period_days": 30,
    "format": "json"
}
```

**Request:**
```bash
curl -X POST "http://localhost:8080/api/v1/analyze/churn" \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "./",
    "period_days": 7,
    "format": "json"
  }'
```

**Response:**
```json
{
    "summary": {
        "total_commits": 1090,
        "files_changed": 360,
        "period_days": 7
    },
    "files": [
        {
            "relative_path": "server/Cargo.toml",
            "commit_count": 45,
            "unique_authors": ["alice@example.com", "bob@example.com"],
            "lines_added": 127,
            "lines_deleted": 83,
            "churn_score": 8.5
        }
    ],
    "hotspots": [
        {
            "file": "server/Cargo.toml",
            "risk_level": "high",
            "churn_frequency": 6.4,
            "author_count": 2
        }
    ]
}
```

### DAG Analysis

#### `POST /api/v1/analyze/dag`

Generate dependency graphs using Mermaid syntax.

**Request Body:**
```json
{
    "project_path": "./",
    "dag_type": "call-graph",
    "enhanced": true,
    "max_depth": 3,
    "show_complexity": true,
    "filter_external": true
}
```

**Request:**
```bash
curl -X POST "http://localhost:8080/api/v1/analyze/dag" \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "./",
    "dag_type": "import-graph",
    "enhanced": true
  }'
```

**Response:**
```json
{
    "mermaid_diagram": "graph TD\n    A[main.rs] --> B[lib.rs]\n    B --> C[utils.rs]\n    ...",
    "dag_type": "import-graph",
    "nodes": 25,
    "edges": 42,
    "complexity_annotations": true,
    "generation_time_ms": 156
}
```

### Context Generation

#### `POST /api/v1/analyze/context`

Generate project context with AST analysis.

**Request Body:**
```json
{
    "toolchain": "rust",
    "project_path": "./",
    "format": "json"
}
```

**Response:**
```json
{
    "metadata": {
        "generated_at": "2025-06-01T12:00:00Z",
        "toolchain": "rust",
        "total_files": 146
    },
    "files": [
        {
            "path": "src/main.rs",
            "language": "rust",
            "items": [
                {
                    "name": "main",
                    "type": "function",
                    "line": 5,
                    "visibility": "public"
                }
            ]
        }
    ]
}
```

### Dead Code Analysis

#### `POST /api/v1/analyze/dead-code`

Analyze dead and unreachable code with confidence scoring.

**Request Body:**
```json
{
    "project_path": "./",
    "format": "json",
    "top_files": 10,
    "include_tests": false,
    "min_dead_lines": 5
}
```

**Request:**
```bash
curl -X POST "http://localhost:8080/api/v1/analyze/dead-code" \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "./",
    "top_files": 5,
    "format": "json"
  }'
```

**Response:**
```json
{
    "summary": {
        "total_files_analyzed": 146,
        "dead_functions": 23,
        "total_dead_lines": 1250,
        "confidence_distribution": {
            "high": 15,
            "medium": 6,
            "low": 2
        }
    },
    "ranked_files": [
        {
            "rank": 1,
            "path": "./server/src/unused_module.rs",
            "dead_functions": 8,
            "dead_lines": 245,
            "total_lines": 500,
            "dead_code_percentage": 49.0,
            "confidence": "high"
        }
    ]
}
```

### Deep Context Analysis ✨ **NEW**

#### `POST /api/v1/analyze/deep-context`

**NEW**: Comprehensive deep context analysis combining multiple analysis types into unified quality assessment.

**Request Body:**
```json
{
    "project_path": "./",
    "include": ["ast", "complexity", "churn", "satd"],
    "exclude": [],
    "period_days": 30,
    "dag_type": "call-graph",
    "max_depth": 10,
    "include_patterns": ["src/**/*.rs", "tests/**/*.rs"],
    "exclude_patterns": ["**/target/**", "**/node_modules/**"],
    "cache_strategy": "normal",
    "parallel": 8,
    "format": "json"
}
```

**Request:**
```bash
curl -X POST "http://localhost:8080/api/v1/analyze/deep-context" \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "./",
    "include": ["complexity", "churn", "satd"],
    "period_days": 30,
    "format": "json"
  }'
```

**Response Structure:**
```json
{
    "metadata": {
        "generated_at": "2025-06-01T12:00:00Z",
        "tool_version": "0.18.5",
        "project_root": "./",
        "analysis_duration": {
            "secs": 8,
            "nanos": 245000000
        },
        "cache_stats": {
            "hit_rate": 0.85,
            "memory_efficiency": 0.92,
            "time_saved_ms": 2450
        }
    },
    "file_tree": {
        "root": {
            "name": "project",
            "path": "./",
            "node_type": "Directory",
            "children": [
                {
                    "name": "src",
                    "path": "./src",
                    "node_type": "Directory",
                    "annotations": {
                        "defect_score": 0.75,
                        "complexity_score": 0.68,
                        "churn_score": 0.82,
                        "dead_code_items": 5,
                        "satd_items": 12
                    },
                    "children": [...]
                }
            ]
        },
        "total_files": 146,
        "total_size_bytes": 1250000
    },
    "analyses": {
        "ast_contexts": [...],
        "complexity_report": {
            "summary": {
                "total_files": 146,
                "total_functions": 15239,
                "median_cyclomatic": 2,
                "max_cyclomatic": 32
            },
            "violations": [...]
        },
        "churn_analysis": {
            "summary": {
                "total_commits": 1090,
                "files_changed": 360
            },
            "files": [...]
        },
        "satd_results": {
            "items": [
                {
                    "file": "./src/complex.rs",
                    "line": 42,
                    "category": "Performance",
                    "severity": "High",
                    "text": "TODO: Optimize this algorithm - currently O(n²)",
                    "age_days": 15
                }
            ]
        },
        "cross_language_refs": [...]
    },
    "quality_scorecard": {
        "overall_health": 78.5,
        "complexity_score": 65.2,
        "maintainability_index": 82.1,
        "modularity_score": 75.0,
        "test_coverage": 85.5,
        "technical_debt_hours": 45.2
    },
    "defect_summary": {
        "total_defects": 87,
        "by_severity": {
            "critical": 5,
            "high": 12,
            "medium": 35,
            "low": 35
        },
        "by_type": {
            "complexity": 25,
            "dead_code": 23,
            "technical_debt": 39
        },
        "defect_density": 0.58
    },
    "hotspots": [
        {
            "location": {
                "file": "./src/complex.rs",
                "line": 42,
                "column": 1
            },
            "composite_score": 0.85,
            "contributing_factors": [
                {
                    "Complexity": {
                        "cyclomatic": 32,
                        "cognitive": 28,
                        "violations": ["High cyclomatic complexity", "Deep nesting"]
                    }
                },
                {
                    "TechnicalDebt": {
                        "category": "Performance",
                        "severity": "High",
                        "age_days": 15
                    }
                }
            ],
            "refactoring_effort": {
                "estimated_hours": 4.5,
                "priority": "High",
                "impact": "High",
                "suggested_actions": [
                    "Extract method for complex logic",
                    "Implement memoization for performance",
                    "Add unit tests for edge cases"
                ]
            }
        }
    ],
    "recommendations": [
        {
            "title": "Reduce Code Complexity",
            "description": "Several functions exceed complexity thresholds. Consider refactoring complex functions into smaller, more focused units.",
            "priority": "High",
            "estimated_effort": {
                "secs": 28800
            },
            "impact": "High",
            "prerequisites": [
                "Identify most complex functions",
                "Create comprehensive test coverage"
            ]
        }
    ]
}
```

**Quality Scorecard Components:**
- **Overall Health Score** (0-100): Composite quality assessment
- **Complexity Score**: Inverse relationship with code complexity
- **Maintainability Index**: Based on complexity, churn, and structural metrics
- **Modularity Score**: Code organization and dependency quality
- **Test Coverage**: Estimated test coverage percentage
- **Technical Debt Hours**: Estimated effort to address all identified debt

**Analysis Types Available:**
- `"ast"`: Abstract syntax tree parsing and symbol extraction
- `"complexity"`: McCabe Cyclomatic and Cognitive complexity metrics
- `"churn"`: Git history and change frequency tracking
- `"dag"`: Dependency graph generation and visualization
- `"dead-code"`: Unused code detection with confidence scoring
- `"satd"`: Self-Admitted Technical Debt detection from comments
- `"defect-probability"`: ML-based defect prediction and hotspot identification

**Output Format Support:**
- `"json"`: Structured data for API consumption and tool integration
- `"markdown"`: Human-readable comprehensive reports with annotated file trees
- `"sarif"`: Static Analysis Results Interchange Format for IDE integration

## MCP Protocol Proxy

### MCP Method Proxy

#### `POST /mcp/{method}`

Proxy endpoint for MCP JSON-RPC 2.0 method calls.

**Request Body:** Standard JSON-RPC 2.0 format
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "analyze_complexity",
        "arguments": {
            "project_path": "./",
            "top_files": 5
        }
    },
    "id": 1
}
```

**Request:**
```bash
curl -X POST "http://localhost:8080/mcp/tools/call" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "analyze_complexity",
        "arguments": {"project_path": "./", "top_files": 5}
    },
    "id": 1
  }'
```

**Response:** Standard JSON-RPC 2.0 format
```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "content": [
            {
                "type": "text",
                "text": "{\"summary\": {...}, \"top_files\": {...}}"
            }
        ]
    }
}
```

## Rate Limiting

The API implements token bucket rate limiting:
- **Burst Rate**: 1000 requests/second
- **Sustained Rate**: 100 requests/second
- **Window**: 1 second

**Rate Limit Headers:**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1640995200
```

**Rate Limit Response (429):**
```json
{
    "error": {
        "code": 429,
        "message": "Rate limit exceeded",
        "details": "Too many requests. Limit: 1000/second"
    }
}
```

## Performance Considerations

### Response Caching

The API implements intelligent caching for expensive operations:
- **Complexity Analysis**: Results cached for 10 minutes
- **Churn Analysis**: Results cached based on git commit history
- **Template Generation**: Templates cached indefinitely (until restart)

### Request Optimization

**Best Practices:**
1. **Batch Operations**: Use deep context analysis instead of multiple individual analyses
2. **Appropriate Timeframes**: Use shorter periods for churn analysis when possible
3. **File Filtering**: Use include/exclude patterns to limit analysis scope
4. **Format Selection**: Use JSON for machine consumption, markdown for human review

### Concurrent Requests

The server supports up to 128 concurrent requests with automatic backpressure management.

## API Examples

### Complete Project Analysis Workflow

```bash
# 1. Check server health
curl http://localhost:8080/health

# 2. Comprehensive analysis
curl -X POST "http://localhost:8080/api/v1/analyze/deep-context" \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "./",
    "include": ["complexity", "churn", "dead-code", "satd"],
    "period_days": 30,
    "top_files": 10,
    "format": "json"
  }' | jq '.'

# 3. Generate project templates
curl -X POST "http://localhost:8080/api/v1/generate" \
  -H "Content-Type: application/json" \
  -d '{
    "template_uri": "template://readme/rust/cli",
    "parameters": {
        "project_name": "my-project",
        "description": "A high-performance CLI tool",
        "has_tests": true
    }
  }' | jq -r '.generated_content'
```

### Monitoring and Observability

```bash
# Server metrics
curl http://localhost:8080/metrics | jq '.'

# Health check with detailed response
curl http://localhost:8080/health | jq '.'

# Performance monitoring
curl "http://localhost:8080/api/v1/analyze/complexity?project_path=./&format=json" \
  -w "Time: %{time_total}s\nSize: %{size_download} bytes\n"
```

## Error Recovery

### Common Error Scenarios

1. **Project Path Not Found (400)**
   ```json
   {
       "error": {
           "code": 400,
           "message": "Project path does not exist",
           "details": "Path './nonexistent' not found"
       }
   }
   ```

2. **Invalid Analysis Type (400)**
   ```json
   {
       "error": {
           "code": 400,
           "message": "Invalid analysis type",
           "details": "Unknown analysis type: 'invalid_analysis'"
       }
   }
   ```

3. **Server Overload (503)**
   ```json
   {
       "error": {
           "code": 503,
           "message": "Service temporarily unavailable",
           "details": "Server is processing too many requests"
       }
   }
   ```

### Retry Strategy

**Recommended retry logic:**
```bash
for i in {1..3}; do
    response=$(curl -s -w "%{http_code}" "http://localhost:8080/api/v1/analyze/complexity")
    status_code=${response: -3}
    if [ "$status_code" -eq 200 ]; then
        echo "Success"
        break
    elif [ "$status_code" -eq 429 ] || [ "$status_code" -eq 503 ]; then
        sleep $((i * 2))  # Exponential backoff
    else
        echo "Error: $status_code"
        break
    fi
done
```

## Security Considerations

1. **Input Validation**: All path parameters are sanitized to prevent directory traversal
2. **Resource Limits**: 
   - Max request size: 1MB
   - Max response size: 10MB
   - Request timeout: 30 seconds
3. **CORS Support**: Configurable via `--cors` flag
4. **No Authentication**: Designed for local development environments

## Migration from CLI/MCP

### CLI to HTTP Mapping

| CLI Command | HTTP Endpoint | Method |
|-------------|---------------|---------|
| `analyze complexity --top-files 5` | `/api/v1/analyze/complexity?top_files=5` | GET |
| `analyze churn --days 30` | `/api/v1/analyze/churn` | POST |
| `analyze deep-context --include complexity,churn` | `/api/v1/analyze/deep-context` | POST |
| `generate makefile rust/cli` | `/api/v1/generate` | POST |

### MCP to HTTP Mapping

MCP tools can be called directly via the `/mcp/{method}` proxy endpoint using standard JSON-RPC 2.0 format.