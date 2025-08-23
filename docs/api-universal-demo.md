# Universal Demo API Reference

## Overview

The Universal Demo "Just Works" API provides AI-powered repository intelligence and analysis capabilities. These endpoints enable automated analysis of any GitHub repository with intelligent recommendations and multi-language project insights.

## Base URL

```
http://localhost:8080/api
```

## Authentication

No authentication required for local demo server.

## Endpoints

### AI-Powered Repository Recommendations

#### `GET /api/recommendations`

Returns AI-powered repository recommendations based on detected languages and complexity preferences.

**Response Format:**
```json
[
  {
    "name": "Example Repository",
    "url": "https://github.com/owner/repo",
    "description": "Repository description",
    "primary_language": "rust",
    "complexity_tier": "Intermediate",
    "frameworks": ["Tokio", "Serde"],
    "learning_value": 0.85,
    "confidence_score": 0.9
  }
]
```

**Fields:**
- `name` (string): Repository name
- `url` (string): GitHub repository URL
- `description` (string): Repository description
- `primary_language` (string): Primary programming language
- `complexity_tier` (string): Beginner | Intermediate | Advanced | Expert
- `frameworks` (array): Detected frameworks and libraries
- `learning_value` (number): Educational value score (0.0-1.0)
- `confidence_score` (number): Recommendation confidence (0.0-1.0)

**Example:**
```bash
curl http://localhost:8080/api/recommendations
```

### Multi-Language Project Intelligence

#### `GET /api/polyglot`

Returns comprehensive multi-language project analysis with cross-language dependencies and architecture patterns.

**Response Format:**
```json
{
  "languages": [
    {
      "language": "rust",
      "file_count": 25,
      "line_count": 2500,
      "complexity_score": 6.5,
      "test_coverage": 0.85,
      "primary_frameworks": ["Tokio", "Serde", "Clap"]
    }
  ],
  "cross_language_dependencies": [
    {
      "from_language": "rust",
      "to_language": "python",
      "dependency_type": "FFI",
      "coupling_strength": 0.7,
      "files_involved": ["Cargo.toml (PyO3)", "setup.py"]
    }
  ],
  "architecture_pattern": "Microservices",
  "integration_points": [
    {
      "name": "rust ↔ python",
      "languages": ["rust", "python"],
      "integration_type": "Memory",
      "risk_level": "Medium",
      "description": "Integration between rust and python via FFI"
    }
  ],
  "recommendation_score": 0.8
}
```

**Fields:**
- `languages` (array): Per-language statistics and frameworks
- `cross_language_dependencies` (array): Cross-language coupling analysis
- `architecture_pattern` (string): Detected architecture pattern
- `integration_points` (array): Risk assessment of integration points
- `recommendation_score` (number): Overall project recommendation score

**Dependency Types:**
- `FFI` - Foreign Function Interface
- `ProcessCommunication` - Inter-process communication
- `SharedDataStructure` - Shared data formats
- `ConfigurationFile` - Shared configuration
- `BuildSystem` - Build system integration
- `Testing` - Test integration

**Architecture Patterns:**
- `Microservices` - Distributed services architecture
- `Monolithic` - Single unified codebase
- `LayeredArchitecture` - Layered application structure
- `EventDriven` - Event-based architecture
- `PluginArchitecture` - Plugin-based extensibility
- `ClientServer` - Client-server pattern
- `Mixed` - Multiple patterns

**Risk Levels:**
- `Low` - Minimal coupling risk
- `Medium` - Moderate coupling with manageable complexity
- `High` - High coupling requiring attention
- `Critical` - Critical coupling needing immediate attention

**Example:**
```bash
curl http://localhost:8080/api/polyglot
```

### Repository Showcase Gallery

#### `GET /api/showcase`

Returns the curated repository showcase gallery with filtering and categorization capabilities.

**Response Format:**
```json
{
  "repositories": [
    {
      "name": "Tokio",
      "url": "https://github.com/tokio-rs/tokio",
      "description": "A runtime for writing reliable asynchronous applications with Rust",
      "primary_language": "rust",
      "languages": ["rust"],
      "frameworks": ["Tokio"],
      "category": "SystemsProgramming",
      "complexity_tier": "Advanced",
      "estimated_analysis_time_seconds": 45,
      "highlights": [
        "Complex async runtime implementation",
        "Extensive use of unsafe Rust",
        "High test coverage",
        "Advanced concurrency patterns"
      ],
      "analysis_preview": {
        "estimated_files": 200,
        "estimated_functions": 1500,
        "estimated_complexity": 8.5,
        "key_insights": [
          "Complex state machines for async operations",
          "Sophisticated memory management",
          "Extensive macro usage for code generation"
        ],
        "recommended_focus_areas": [
          "Async runtime components",
          "Thread pool implementation",
          "I/O driver abstractions"
        ]
      }
    }
  ],
  "summary": {
    "total_repositories": 8,
    "total_languages": 4,
    "total_categories": 6,
    "complexity_distribution": {
      "Beginner": 1,
      "Intermediate": 2,
      "Advanced": 4,
      "Expert": 1
    },
    "featured_count": 4,
    "quick_start_count": 3
  },
  "featured": [
    // Featured repository objects (subset of repositories)
  ],
  "quickStart": [
    // Quick start repository objects (beginner/intermediate)
  ],
  "categories": [
    "WebFramework",
    "SystemsProgramming", 
    "DeveloperTools",
    "DataScience",
    "CloudNative"
  ]
}
```

**Repository Categories:**
- `WebFramework` - Web development frameworks
- `SystemsProgramming` - Low-level systems programming
- `DeveloperTools` - Development tools and utilities
- `DataScience` - Data analysis and scientific computing
- `CloudNative` - Cloud and containerization technologies
- `MachineLearning` - AI and machine learning
- `GameDevelopment` - Game engines and development
- `Blockchain` - Blockchain and cryptocurrency
- `Mobile` - Mobile application development
- `Embedded` - Embedded systems programming

**Complexity Tiers:**
- `Beginner` - Entry-level projects for learning
- `Intermediate` - Moderate complexity with clear patterns
- `Advanced` - Complex projects requiring deep understanding
- `Expert` - Highly complex projects for experienced developers

**Example:**
```bash
curl http://localhost:8080/api/showcase
```

## Usage Examples

### Analyze Repository and Get Recommendations

```bash
# 1. Start the demo server (analyze a repository first)
cargo run --example analyze_github_repo -- --url https://github.com/rust-lang/rust-clippy

# 2. In another terminal, start web demo
pmat demo --serve

# 3. Get AI recommendations
curl http://localhost:8080/api/recommendations | jq '.[0] | {name, complexity_tier, learning_value}'

# 4. Get polyglot analysis
curl http://localhost:8080/api/polyglot | jq '.architecture_pattern'

# 5. Browse showcase gallery
curl http://localhost:8080/api/showcase | jq '.summary'
```

### Filter Showcase by Complexity

```javascript
// Get beginner-friendly repositories
fetch('/api/showcase')
  .then(response => response.json())
  .then(data => {
    const beginnerRepos = data.repositories.filter(
      repo => repo.complexity_tier === 'Beginner'
    );
    console.log('Beginner repositories:', beginnerRepos);
  });
```

### Analyze Multi-Language Projects

```bash
# Analyze a polyglot repository
cargo run --example analyze_github_repo -- --url https://github.com/microsoft/vscode

# Check for cross-language dependencies
curl http://localhost:8080/api/polyglot | jq '.cross_language_dependencies'

# Get integration risk assessment  
curl http://localhost:8080/api/polyglot | jq '.integration_points[] | select(.risk_level == "High")'
```

## Error Responses

All endpoints return standard HTTP status codes:

- `200 OK` - Successful request
- `404 Not Found` - Endpoint not found
- `500 Internal Server Error` - Server error

Error response format:
```json
{
  "error": "Error description",
  "code": "ERROR_CODE"
}
```

## Integration with Web Demo

The API endpoints are designed to work seamlessly with the interactive web demo interface. The web demo provides:

- **Interactive Visualizations**: Dynamic charts and graphs powered by these APIs
- **Real-time Updates**: Live data updates as analysis progresses
- **Progressive Enhancement**: Graceful degradation when APIs are unavailable
- **Responsive Design**: Mobile-friendly interface for all screen sizes

Visit `http://localhost:8080` after starting the demo server to experience the full interactive interface.

## Development and Extension

The API endpoints are implemented using PMAT's service architecture:

- **Modular Design**: Each endpoint is backed by dedicated service modules
- **Toyota Way Quality**: Zero-defect implementation with comprehensive testing
- **Extensible Architecture**: Easy to add new analysis capabilities
- **Performance Optimized**: Efficient data structures and caching

For adding new endpoints or modifying existing ones, see the implementation in:
- `server/src/demo/server.rs` - HTTP handlers
- `server/src/demo/router.rs` - Route definitions
- `server/src/services/` - Business logic services