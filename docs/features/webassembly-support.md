# WebAssembly and AssemblyScript Support

## Overview

PAIML MCP Agent Toolkit v0.26.2 introduces comprehensive WebAssembly analysis capabilities, supporting both WebAssembly binary/text formats and AssemblyScript source code.

## Features

### WebAssembly Analysis (`pmat analyze webassembly`)

Analyzes WebAssembly modules in both binary (`.wasm`) and text (`.wat`) formats.

```bash
# Analyze all WebAssembly files in current directory
pmat analyze webassembly

# Include only binary files
pmat analyze webassembly --include-binary --no-include-text

# Enable comprehensive analysis
pmat analyze webassembly --memory-analysis --security --complexity

# Output to JSON format
pmat analyze webassembly --format json -o wasm-analysis.json
```

#### Supported Metrics
- **Function count**: Total functions in the module
- **Import/Export analysis**: External dependencies and exposed APIs
- **Memory metrics**: Linear memory pages and usage patterns
- **Complexity analysis**: Cyclomatic and cognitive complexity
- **Security validation**: Basic security checks

### AssemblyScript Analysis (`pmat analyze assemblyscript`)

Specialized analysis for AssemblyScript - a TypeScript-like language that compiles to WebAssembly.

```bash
# Basic AssemblyScript analysis
pmat analyze assemblyscript

# Enable WASM complexity metrics
pmat analyze assemblyscript --wasm-complexity

# Full analysis with all features
pmat analyze assemblyscript --wasm-complexity --memory-analysis --security

# Custom timeout for large projects
pmat analyze assemblyscript --timeout 60
```

#### Features
- **TypeScript-like syntax support**: Familiar syntax for web developers
- **WASM-specific metrics**: Gas estimation, memory pressure
- **Memory safety**: Iterative parsing prevents stack overflow
- **Performance tracking**: Analysis time metrics

## Implementation Details

### Memory Safety
All WebAssembly parsers include comprehensive safety protections:
- **File size limits**: 10MB default maximum
- **Timeout protection**: 30-second parsing timeout
- **Iterative parsing**: Prevents stack overflow on deeply nested structures
- **Node limits**: Maximum 100,000 AST nodes per file

### Architecture
```
services/wasm/
├── assemblyscript.rs    # AssemblyScript parser
├── binary.rs           # WASM binary analyzer
├── wat.rs              # WebAssembly Text parser
├── complexity.rs       # Complexity metrics
├── security.rs         # Security validation
├── parallel.rs         # Parallel processing
├── types.rs            # Core type definitions
└── error.rs            # Error handling
```

### Output Formats
- **Summary**: Human-readable summary (default)
- **JSON**: Machine-readable format for tooling
- **SARIF**: IDE integration format
- **Markdown**: Documentation-friendly format

## Use Cases

### 1. WebAssembly Module Analysis
Analyze compiled WASM modules to understand their structure and complexity:
```bash
pmat analyze webassembly -p ./wasm-modules --format json
```

### 2. AssemblyScript Development
Monitor code quality during AssemblyScript development:
```bash
pmat analyze assemblyscript --wasm-complexity --watch
```

### 3. Security Auditing
Basic security validation for WebAssembly modules:
```bash
pmat analyze webassembly --security -o security-report.md
```

### 4. CI/CD Integration
Integrate into build pipelines:
```bash
pmat analyze assemblyscript --format sarif -o results.sarif
pmat quality-gate --fail-on-violation
```

## Limitations

- Basic implementation focused on core metrics
- Security validation is rudimentary
- No support for WASI (WebAssembly System Interface) yet
- AssemblyScript parsing uses heuristics rather than full AST

## Future Enhancements

- Full tree-sitter integration for AssemblyScript
- Advanced security analysis
- WASI support
- WebAssembly component model support
- Integration with wasmtime/wasmer for runtime analysis