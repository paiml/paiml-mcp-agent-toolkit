# Deep Context Analysis

## Overview

Deep Context Analysis provides comprehensive codebase understanding by combining multiple analysis dimensions into a unified report. It leverages AST parsing, complexity metrics, git history, and semantic analysis to create rich context for AI agents and developers.

## Features

- **Multi-Language AST Analysis**: Full parsing for Rust, TypeScript, Python, C/C++
- **Holistic Metrics**: Combines complexity, churn, dependencies, and quality
- **Incremental Updates**: Efficient re-analysis of changed files only
- **AI-Optimized Output**: Structured for LLM consumption
- **Caching**: Sub-second analysis for unchanged code

## Usage

### Command Line

```bash
# Full deep context analysis
pmat analyze deep-context

# Specific output format
pmat analyze deep-context --format json
pmat analyze deep-context --format markdown

# Include specific analyses
pmat analyze deep-context --include ast,complexity,churn,dependencies

# Incremental mode (only analyze changes)
pmat analyze deep-context --incremental

# Save to file
pmat analyze deep-context --output deep-context.md
```

### Configuration

```toml
# deep-context.toml
[analysis]
include = ["ast", "complexity", "churn", "dependencies", "quality"]
exclude_paths = ["tests/", "vendor/", "node_modules/"]
max_file_size = "1MB"

[output]
format = "markdown"
include_snippets = true
max_snippet_lines = 50

[cache]
enabled = true
path = ".pmat-cache"
ttl = "24h"

[incremental]
enabled = true
baseline = ".pmat-baseline.json"
```

## Output Structure

### Markdown Format

```markdown
# Deep Context Analysis

Generated: 2024-06-09 10:30:45 UTC
Project: paiml-mcp-agent-toolkit
Language: Rust (primary), TypeScript (secondary)

## Project Overview

### Structure
- Total Files: 298
- Lines of Code: 45,231
- Test Coverage: 92.3%
- Primary Language: Rust (78%)

### Key Components
1. **Core Engine** (`src/services/`)
   - AST parsing and analysis
   - Complexity calculation
   - Cache management
   
2. **CLI Interface** (`src/cli/`)
   - Command parsing
   - Output formatting
   - Interactive modes

### Complexity Analysis

#### Hotspots
| File | Function | Cyclomatic | Cognitive |
|------|----------|------------|-----------|
| cli/mod.rs | handle_analyze_graph | 75 | 125 |
| services/analyzer.rs | process_ast | 45 | 67 |

### Recent Changes

#### High Churn Files (Last 30 days)
1. `src/services/refactor_engine.rs` - 23 commits, 3 authors
2. `src/cli/handlers/mod.rs` - 18 commits, 2 authors

### Dependencies

#### Direct Dependencies (Critical)
- tokio (1.35): Async runtime
- clap (4.4): CLI parsing
- serde (1.0): Serialization

### Quality Indicators

- Technical Debt Gradient: 1.45 (Moderate)
- Code Smells: 23 (15 high priority)
- Security Issues: 0
- Outdated Dependencies: 3
```

### JSON Format

```json
{
  "metadata": {
    "generated": "2024-06-09T10:30:45Z",
    "version": "0.21.5",
    "project_path": "/path/to/project"
  },
  "overview": {
    "total_files": 298,
    "total_lines": 45231,
    "languages": {
      "rust": { "files": 234, "percentage": 78.5 },
      "typescript": { "files": 64, "percentage": 21.5 }
    }
  },
  "structure": {
    "components": [
      {
        "name": "Core Engine",
        "path": "src/services",
        "purpose": "Main analysis engine",
        "complexity": "high",
        "dependencies": ["tokio", "rayon", "dashmap"]
      }
    ],
    "architecture_style": "modular",
    "entry_points": ["src/main.rs", "src/lib.rs"]
  },
  "complexity": {
    "summary": {
      "median_cyclomatic": 5,
      "p90_cyclomatic": 20,
      "max_cyclomatic": 75
    },
    "hotspots": [
      {
        "file": "cli/mod.rs",
        "function": "handle_analyze_graph",
        "cyclomatic": 75,
        "cognitive": 125,
        "recommendation": "Extract sub-functions"
      }
    ]
  },
  "quality": {
    "tdg_score": 1.45,
    "test_coverage": 92.3,
    "code_smells": 23,
    "security_issues": 0
  }
}
```

## Analysis Components

### 1. AST Analysis

Extracts structural information from source code:

- **Function signatures**: Parameters, return types, generics
- **Type definitions**: Structs, enums, traits/interfaces
- **Import graph**: Dependency relationships
- **Complexity metrics**: Per-function measurements
- **Documentation**: Extracted comments and docstrings

### 2. Complexity Analysis

Measures code complexity at multiple levels:

- **Function-level**: Cyclomatic and cognitive complexity
- **File-level**: Average and maximum complexity
- **Module-level**: Aggregate complexity scores
- **Trends**: Complexity changes over time

### 3. Git Churn Analysis

Analyzes version control history:

- **Change frequency**: Files modified most often
- **Author distribution**: Who modifies what
- **Coupled files**: Files that change together
- **Stability indicators**: Mature vs volatile code

### 4. Dependency Analysis

Maps internal and external dependencies:

- **Import graph**: Who depends on whom
- **Circular dependencies**: Architectural issues
- **External dependencies**: Version and security status
- **Unused dependencies**: Potential for cleanup

### 5. Quality Metrics

Comprehensive quality indicators:

- **Test coverage**: Line, branch, function coverage
- **Documentation coverage**: Public API documentation
- **Code smells**: Duplication, long methods, etc.
- **Technical debt**: Accumulated issues and TODOs

## AI Integration

### Optimized for LLMs

The output is structured for optimal LLM consumption:

1. **Hierarchical Structure**: Information organized from general to specific
2. **Context Windows**: Chunked for standard LLM context sizes
3. **Semantic Markers**: Clear section boundaries and relationships
4. **Actionable Insights**: Specific recommendations included

### Example AI Workflow

```python
import subprocess
import json

def get_project_context(project_path):
    """Generate deep context for AI analysis"""
    
    # Run deep context analysis
    result = subprocess.run([
        'pmat', 'analyze', 'deep-context',
        '--format', 'json',
        '--path', project_path
    ], capture_output=True, text=True)
    
    context = json.loads(result.stdout)
    
    # Create AI prompt
    prompt = f"""
    Project Analysis:
    - Language: {context['overview']['languages']}
    - Complexity Hotspots: {len(context['complexity']['hotspots'])}
    - Technical Debt: {context['quality']['tdg_score']}
    
    Key Areas of Concern:
    {format_hotspots(context['complexity']['hotspots'])}
    
    Please suggest refactoring strategies for the top 3 complexity hotspots.
    """
    
    return prompt
```

## Performance

### Benchmarks

| Codebase Size | Initial Analysis | Incremental Update | Cache Hit |
|---------------|-----------------|-------------------|-----------|
| 10K LOC | 2.3s | 0.4s | 0.05s |
| 100K LOC | 18.5s | 2.1s | 0.08s |
| 1M LOC | 3m 45s | 15.2s | 0.12s |

### Optimization Strategies

1. **Parallel Processing**: Uses all CPU cores for AST parsing
2. **Incremental Analysis**: Only re-analyzes changed files
3. **Smart Caching**: Content-based cache invalidation
4. **Streaming Output**: Results available as computed

## Integration Examples

### CI/CD Pipeline

```yaml
name: Deep Context Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install PMAT
        run: |
          curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
      
      - name: Run Deep Context Analysis
        run: |
          pmat analyze deep-context \
            --format markdown \
            --output DEEP_CONTEXT.md
      
      - name: Upload Context
        uses: actions/upload-artifact@v3
        with:
          name: deep-context
          path: DEEP_CONTEXT.md
      
      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const context = fs.readFileSync('DEEP_CONTEXT.md', 'utf8');
            
            // Extract summary section
            const summary = context.match(/## Summary[\s\S]*?(?=##|$)/)[0];
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `### Deep Context Analysis\n${summary}`
            });
```

### IDE Integration

```typescript
// vscode extension
import * as vscode from 'vscode';
import { execSync } from 'child_process';

export function activate(context: vscode.ExtensionContext) {
    let disposable = vscode.commands.registerCommand('pmat.deepContext', () => {
        const workspacePath = vscode.workspace.rootPath;
        
        vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: "Generating Deep Context",
            cancellable: false
        }, async (progress) => {
            progress.report({ increment: 0 });
            
            const result = execSync(
                `pmat analyze deep-context --format json`,
                { cwd: workspacePath, encoding: 'utf8' }
            );
            
            const context = JSON.parse(result);
            
            // Create webview with results
            const panel = vscode.window.createWebviewPanel(
                'deepContext',
                'Deep Context Analysis',
                vscode.ViewColumn.One,
                {}
            );
            
            panel.webview.html = generateHtml(context);
        });
    });
    
    context.subscriptions.push(disposable);
}
```

## Best Practices

1. **Regular Updates**: Run analysis daily to track trends
2. **Baseline Comparison**: Compare against known good state
3. **Focus on Changes**: Use incremental mode for PRs
4. **Cache Wisely**: Configure cache based on team size
5. **Customize Output**: Tailor format to consumer needs

## Troubleshooting

### Common Issues

**Q: Analysis is taking too long**
A: Enable caching and use incremental mode. Exclude large generated files.

**Q: Memory usage is high**
A: Reduce `max_file_size` and enable streaming output.

**Q: Cache seems stale**
A: Clear cache with `rm -rf .pmat-cache` or reduce TTL.

### Debug Mode

```bash
# Enable detailed logging
RUST_LOG=debug pmat analyze deep-context

# Profile performance
pmat analyze deep-context --profile

# Validate cache
pmat analyze deep-context --validate-cache
```

## Future Roadmap

- **Semantic Code Search**: Natural language queries
- **Change Impact Analysis**: Predict effects of changes
- **Architecture Visualization**: Auto-generated diagrams
- **Team Analytics**: Developer productivity metrics
- **Real-time Monitoring**: Continuous analysis daemon