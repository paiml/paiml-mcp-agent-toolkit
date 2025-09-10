# WebAssembly (WASM) Features in PMAT

## Table of Contents
- [Overview](#overview)
- [Quick Start](#quick-start)
- [Core Features](#core-features)
- [Usage Guide](#usage-guide)
- [Integration Examples](#integration-examples)
- [Technical Architecture](#technical-architecture)
- [API Reference](#api-reference)

## Overview

PMAT v2.77.0 introduces comprehensive WebAssembly module analysis capabilities, providing enterprise-grade quality assurance for WASM binaries. This feature suite is designed to analyze, verify, and optimize WebAssembly modules, with special support for Ruchy language notebooks and other WASM-based applications.

### Key Capabilities
- 🔍 **Deep Analysis**: Function-level complexity metrics and instruction profiling
- 🔒 **Formal Verification**: Mathematical proof of memory safety and type correctness
- 🛡️ **Security Scanning**: Pattern-based vulnerability detection
- 📊 **Performance Profiling**: Non-intrusive shadow stack profiling
- 📈 **Quality Baselines**: Multi-anchor regression detection system

## Quick Start

### Basic Analysis
```bash
# Analyze a WASM module
pmat analyze wasm module.wasm

# Output:
# WASM Analysis Summary
# ====================
# Functions: 42
# Instructions: 1337
# Binary Size: 65536 bytes
# Max Complexity: 15
```

### Complete Analysis Suite
```bash
# Run all analysis features
pmat analyze wasm module.wasm \
  --verify \      # Formal verification
  --security \    # Security scanning
  --profile \     # Performance profiling
  --baseline reference.wasm  # Quality comparison
```

## Core Features

### 1. Streaming Analysis Pipeline

PMAT uses a streaming parser to handle WASM files of any size efficiently:

```rust
// Internal architecture
WasmAnalyzer {
    parser: StreamingParser,      // Handles GB-sized files
    validator: IncrementalValidator,
    profiler: ShadowStackProfiler,
    detector: PatternDetector,
}
```

**Benefits:**
- Memory-efficient processing
- Real-time analysis feedback
- Support for large WASM binaries (>100MB)
- Progressive results reporting

### 2. Formal Verification

Mathematical verification of WASM module safety properties:

```bash
pmat analyze wasm module.wasm --verify
```

**Verification Checks:**
- **Memory Bounds**: All memory accesses proven within allocated bounds
- **Type Safety**: Stack operations maintain type consistency
- **Stack Balance**: Functions preserve stack discipline
- **Integer Safety**: Arithmetic operations checked for overflow
- **Control Flow**: Indirect calls validated against function table

**Example Output:**
```
Verification Result: ✅ SAFE
- Memory accesses: All bounded
- Type consistency: Maintained
- Stack balance: Verified
- Integer operations: No overflows detected
```

### 3. Security Vulnerability Scanning

Pattern-based detection of common WASM security issues:

```bash
pmat analyze wasm module.wasm --security
```

**Detected Patterns:**

| Pattern | Severity | Description |
|---------|----------|-------------|
| Buffer Overflow | Critical | Unchecked memory access that could exceed bounds |
| Integer Overflow | High | Arithmetic operations without overflow checks |
| Memory Growth | High | Unbounded memory.grow operations |
| Indirect Call | Medium | Unvalidated function pointer usage |
| Stack Overflow | Medium | Recursive calls without depth limits |
| Uninitialized Memory | Low | Reading memory before initialization |

**SARIF Output for CI/CD:**
```bash
pmat analyze wasm module.wasm --security --format sarif --output security.sarif
```

### 4. Performance Profiling

Non-intrusive shadow stack profiling for performance analysis:

```bash
pmat analyze wasm module.wasm --profile
```

**Profiling Metrics:**

```
Performance Profile
===================
Instruction Mix:
  Control Flow: 15% (branches, loops)
  Memory Ops: 25% (loads, stores)
  Arithmetic: 40% (math operations)
  Function Calls: 20% (direct, indirect)

Hot Functions (>5% time):
  1. process_data - 18.5% (4,231 samples)
  2. validate_input - 12.3% (2,812 samples)
  3. compute_hash - 8.7% (1,987 samples)

Memory Profile:
  Peak Usage: 4.2 MB
  Growth Events: 3
  Average Allocation: 256 KB
```

### 5. Quality Baselines

Compare WASM modules against reference baselines:

```bash
pmat analyze wasm current.wasm --baseline stable.wasm
```

**Multi-Anchor System:**

```yaml
Baselines:
  release:       # Stable production version
    complexity: 15
    size: 64KB
    performance: 100ms
    
  preview:       # Beta version
    complexity: 18
    size: 68KB
    performance: 95ms
    
  experimental:  # Development version
    complexity: 22
    size: 72KB
    performance: 90ms
```

**Quality Assessment:**
```
Quality Comparison
==================
Current vs Baseline (stable.wasm):
  ✅ Complexity: 14 (-1, improved)
  ⚠️ Size: 66KB (+2KB, increased)
  ✅ Performance: 98ms (-2ms, faster)
  
Overall: PASSING (no regressions)
Recommendation: Safe to deploy
```

## Usage Guide

### Analyzing Ruchy Notebooks

Ruchy language notebooks compile to WASM for execution. PMAT provides specialized analysis:

```bash
# Analyze Ruchy notebook WASM
pmat analyze wasm ../ruchy/notebooks/data_analysis.wasm \
  --verify \
  --profile \
  --output analysis.md

# Compare notebook versions
pmat analyze wasm ../ruchy/notebooks/v2/analysis.wasm \
  --baseline ../ruchy/notebooks/v1/analysis.wasm
```

### CI/CD Integration

#### GitHub Actions
```yaml
name: WASM Quality Gate
on: [push, pull_request]

jobs:
  wasm-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install PMAT
        run: cargo install pmat
      
      - name: Build WASM
        run: cargo build --target wasm32-unknown-unknown
      
      - name: Analyze WASM Security
        run: |
          pmat analyze wasm target/wasm32-unknown-unknown/release/app.wasm \
            --security \
            --format sarif \
            --output wasm-security.sarif
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: wasm-security.sarif
      
      - name: Quality Gate
        run: |
          pmat analyze wasm target/wasm32-unknown-unknown/release/app.wasm \
            --verify \
            --baseline .github/baseline.wasm
```

#### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check all WASM files
for wasm in $(git diff --cached --name-only | grep "\.wasm$"); do
  echo "Analyzing $wasm..."
  
  # Security check
  pmat analyze wasm "$wasm" --security --verify
  if [ $? -ne 0 ]; then
    echo "❌ WASM analysis failed for $wasm"
    exit 1
  fi
done

echo "✅ All WASM files passed quality checks"
```

### Makefile Integration

```makefile
# WASM analysis targets
.PHONY: wasm-check wasm-security wasm-profile wasm-baseline

WASM_FILE := build/module.wasm
BASELINE_WASM := releases/stable.wasm

wasm-check: $(WASM_FILE)
	@echo "Running comprehensive WASM analysis..."
	pmat analyze wasm $(WASM_FILE) \
	  --verify \
	  --security \
	  --profile \
	  --baseline $(BASELINE_WASM)

wasm-security: $(WASM_FILE)
	@echo "Security scanning WASM module..."
	pmat analyze wasm $(WASM_FILE) \
	  --security \
	  --format sarif \
	  --output reports/wasm-security.sarif

wasm-profile: $(WASM_FILE)
	@echo "Profiling WASM performance..."
	pmat analyze wasm $(WASM_FILE) \
	  --profile \
	  --format json \
	  --output reports/wasm-profile.json

wasm-baseline: $(WASM_FILE)
	@echo "Comparing against baseline..."
	pmat analyze wasm $(WASM_FILE) \
	  --baseline $(BASELINE_WASM) \
	  --format detailed \
	  --verbose
```

## Integration Examples

### 1. Automated Security Scanning

```python
#!/usr/bin/env python3
"""Automated WASM security scanner"""

import subprocess
import json
import sys

def scan_wasm(wasm_file):
    """Run security scan on WASM file"""
    result = subprocess.run([
        'pmat', 'analyze', 'wasm', wasm_file,
        '--security', '--format', 'json'
    ], capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"Error scanning {wasm_file}: {result.stderr}")
        return None
    
    return json.loads(result.stdout)

def main():
    wasm_files = sys.argv[1:]
    
    for wasm_file in wasm_files:
        print(f"Scanning {wasm_file}...")
        results = scan_wasm(wasm_file)
        
        if results and results.get('security'):
            critical = [v for v in results['security'] 
                       if v['severity'] == 'Critical']
            if critical:
                print(f"❌ {len(critical)} critical vulnerabilities found!")
                for vuln in critical:
                    print(f"  - {vuln['pattern']} at {vuln['location']}")
                sys.exit(1)
    
    print("✅ All WASM files passed security scan")

if __name__ == '__main__':
    main()
```

### 2. Performance Regression Detection

```bash
#!/bin/bash
# detect_performance_regression.sh

CURRENT_WASM=$1
BASELINE_WASM=$2
THRESHOLD=10  # 10% regression threshold

# Run performance comparison
OUTPUT=$(pmat analyze wasm "$CURRENT_WASM" \
  --profile \
  --baseline "$BASELINE_WASM" \
  --format json)

# Extract performance metrics
CURRENT_TIME=$(echo "$OUTPUT" | jq '.profiling.estimated_runtime_ms')
BASELINE_TIME=$(echo "$OUTPUT" | jq '.baseline.estimated_runtime_ms')

# Calculate regression percentage
REGRESSION=$(echo "scale=2; (($CURRENT_TIME - $BASELINE_TIME) / $BASELINE_TIME) * 100" | bc)

if (( $(echo "$REGRESSION > $THRESHOLD" | bc -l) )); then
  echo "❌ Performance regression detected: ${REGRESSION}%"
  echo "Current: ${CURRENT_TIME}ms, Baseline: ${BASELINE_TIME}ms"
  exit 1
fi

echo "✅ Performance acceptable: ${REGRESSION}% change"
```

### 3. Quality Dashboard Integration

```javascript
// wasm-quality-dashboard.js
const { exec } = require('child_process');
const express = require('express');
const app = express();

app.get('/api/wasm/analyze/:module', async (req, res) => {
  const module = req.params.module;
  
  exec(`pmat analyze wasm ${module} --format json`, (error, stdout) => {
    if (error) {
      return res.status(500).json({ error: error.message });
    }
    
    const analysis = JSON.parse(stdout);
    
    // Calculate quality score
    const score = calculateQualityScore(analysis);
    
    res.json({
      module,
      score,
      grade: getGrade(score),
      analysis
    });
  });
});

function calculateQualityScore(analysis) {
  let score = 100;
  
  // Deduct for complexity
  score -= Math.max(0, analysis.max_complexity - 10) * 2;
  
  // Deduct for security issues
  if (analysis.security) {
    analysis.security.forEach(vuln => {
      if (vuln.severity === 'Critical') score -= 10;
      if (vuln.severity === 'High') score -= 5;
      if (vuln.severity === 'Medium') score -= 2;
    });
  }
  
  // Deduct for verification failures
  if (analysis.verification && !analysis.verification.is_safe) {
    score -= 20;
  }
  
  return Math.max(0, score);
}

function getGrade(score) {
  if (score >= 90) return 'A+';
  if (score >= 80) return 'A';
  if (score >= 70) return 'B';
  if (score >= 60) return 'C';
  return 'F';
}

app.listen(3000, () => {
  console.log('WASM Quality Dashboard running on port 3000');
});
```

## Technical Architecture

### Component Overview

```
┌─────────────────────────────────────────────┐
│            WASM Analysis Pipeline           │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────────┐      ┌─────────────┐      │
│  │   Binary    │─────▶│  Streaming  │      │
│  │   Input     │      │   Parser    │      │
│  └─────────────┘      └─────────────┘      │
│                              │              │
│                              ▼              │
│                    ┌─────────────────┐     │
│                    │    Validator    │     │
│                    │  (Incremental)  │     │
│                    └─────────────────┘     │
│                              │              │
│          ┌───────────────────┼───────────┐  │
│          ▼                   ▼           ▼  │
│   ┌──────────┐     ┌──────────┐  ┌──────────┐
│   │Security  │     │ Profiler │  │Baseline  │
│   │ Scanner  │     │  Shadow  │  │Comparator│
│   └──────────┘     └──────────┘  └──────────┘
│          │                   │           │  │
│          └───────────────────┼───────────┘  │
│                              ▼              │
│                    ┌─────────────────┐     │
│                    │  Output Format  │     │
│                    │  (JSON/SARIF)   │     │
│                    └─────────────────┘     │
│                                             │
└─────────────────────────────────────────────┘
```

### Key Components

1. **Streaming Parser** (`wasm/analyzer.rs`)
   - Uses `wasmparser` crate for efficient parsing
   - Processes modules incrementally
   - Maintains minimal memory footprint

2. **Incremental Verifier** (`wasm/verifier.rs`)
   - Function-by-function verification
   - Shadow stack for type checking
   - SMT solver integration ready

3. **Pattern Detector** (`wasm/security.rs`)
   - AST pattern matching
   - Vulnerability signature database
   - Confidence scoring system

4. **Shadow Stack Profiler** (`wasm/profiler.rs`)
   - Non-intrusive profiling
   - Statistical sampling
   - Call graph construction

5. **Baseline Comparator** (`wasm/baseline.rs`)
   - Multi-anchor system
   - Fuzzy matching for hardware differences
   - Regression detection algorithms

### Performance Characteristics

| Operation | Time Complexity | Space Complexity |
|-----------|----------------|------------------|
| Parsing | O(n) | O(1) streaming |
| Verification | O(n*m) | O(m) per function |
| Security Scan | O(n*p) | O(p) patterns |
| Profiling | O(n*s) | O(s) samples |
| Baseline | O(n) | O(1) comparison |

Where:
- n = module size
- m = function complexity
- p = pattern count
- s = sample rate

## API Reference

### CLI Commands

#### `pmat analyze wasm`

Analyze WebAssembly modules for quality, security, and performance.

**Syntax:**
```bash
pmat analyze wasm <wasm_file> [OPTIONS]
```

**Options:**
| Option | Description | Default |
|--------|-------------|---------|
| `--verify` | Enable formal verification | false |
| `--security` | Enable security scanning | false |
| `--profile` | Enable performance profiling | false |
| `--baseline <path>` | Compare against baseline WASM | none |
| `--format <format>` | Output format (summary/json/detailed/sarif) | summary |
| `--output <path>` | Write output to file | stdout |
| `--verbose` | Enable verbose output | false |

**Examples:**
```bash
# Basic analysis
pmat analyze wasm app.wasm

# Security focus
pmat analyze wasm app.wasm --security --format sarif

# Performance analysis
pmat analyze wasm app.wasm --profile --verbose

# Full suite
pmat analyze wasm app.wasm --verify --security --profile
```

### Output Formats

#### Summary Format
Human-readable summary of key metrics:
```
WASM Analysis Summary
====================
Functions: 42
Instructions: 1337
Binary Size: 65536 bytes
Memory Pages: 4
Max Complexity: 15

Verification: ✅ SAFE
Security: ⚠️ 3 medium issues
Performance: 95/100
```

#### JSON Format
Machine-readable complete analysis:
```json
{
  "analysis": {
    "function_count": 42,
    "instruction_count": 1337,
    "binary_size": 65536,
    "memory_pages": 4,
    "max_complexity": 15
  },
  "verification": {
    "is_safe": true,
    "checks": {
      "memory_bounds": "passed",
      "type_safety": "passed",
      "stack_balance": "passed"
    }
  },
  "security": [
    {
      "severity": "Medium",
      "pattern": "unchecked_memory_access",
      "location": "function[5]",
      "confidence": 0.85
    }
  ],
  "profiling": {
    "instruction_mix": {
      "control_flow": 0.15,
      "memory_ops": 0.25,
      "arithmetic": 0.40,
      "calls": 0.20
    },
    "hot_functions": [
      {
        "name": "process_data",
        "percentage": 18.5,
        "samples": 4231
      }
    ]
  }
}
```

#### SARIF Format
Static Analysis Results Interchange Format for CI/CD:
```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [{
    "tool": {
      "driver": {
        "name": "pmat-wasm-analyzer",
        "version": "2.77.0",
        "rules": [...]
      }
    },
    "results": [...]
  }]
}
```

### Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| `WASM001` | Invalid WASM magic number | Ensure file is valid WASM binary |
| `WASM002` | Unsupported WASM version | Update to WASM version 1 |
| `WASM003` | Parsing error | Check WASM binary integrity |
| `WASM004` | Verification timeout | Simplify module or increase timeout |
| `WASM005` | Pattern database missing | Update PMAT installation |

## Best Practices

### 1. Regular Analysis
- Run analysis on every build
- Track metrics over time
- Set quality thresholds

### 2. Security First
- Always run security scanning
- Fix critical issues immediately
- Review medium/low issues regularly

### 3. Performance Monitoring
- Profile before optimization
- Compare against baselines
- Track regression trends

### 4. CI/CD Integration
- Automate quality gates
- Fail builds on regressions
- Generate reports for review

### 5. Baseline Management
- Maintain stable baselines
- Update baselines carefully
- Document baseline changes

## Troubleshooting

### Common Issues

**Issue: "Invalid WASM magic number"**
- Ensure file is a valid WebAssembly binary
- Check file hasn't been corrupted
- Verify build process generates WASM

**Issue: "Verification timeout"**
- Module may be too complex
- Try analyzing smaller functions
- Increase timeout with `--timeout`

**Issue: "No baseline found"**
- Ensure baseline file exists
- Check file path is correct
- Verify baseline is valid WASM

**Issue: "Security pattern not found"**
- Update PMAT to latest version
- Pattern database may be outdated
- Report issue if persists

## Related Documentation

- [WASM Analysis Guide](./wasm-analysis-guide.md) - Detailed usage guide
- [WASM Quality Assurance Specification](./specifications/wasm-quality-assurance.md) - Technical specification
- [Quality Gates Guide](./quality-gates.md) - Integration with quality gates
- [MCP Integration](./mcp-integration.md) - Using WASM analysis via MCP

## Support

For issues, questions, or feature requests:
- GitHub Issues: [paiml/paiml-mcp-agent-toolkit](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)
- Documentation: [docs.paiml.com](https://docs.paiml.com)
- Community: [Discord](https://discord.gg/paiml)