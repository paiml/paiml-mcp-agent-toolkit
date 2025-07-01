# Extreme Quality Enforcement Specification

## Overview

The `pmat enforce extreme` command provides a state-machine based approach to iteratively improve code quality until it meets "extreme quality" standards. This specification is designed for both human developers and AI agents (Claude, Gemini, etc.) to systematically refactor codebases to achieve excellence.

## Core Concept: Quality State Machine

The enforcement system operates as a state machine with clear transitions:

```
ANALYZING → VIOLATING → REFACTORING → VALIDATING → COMPLETE
     ↑           ↓            ↓            ↓
     └───────────┴────────────┴────────────┘
```

## Command Interface

### Basic Usage

```bash
# Run until quality score reaches 1.0 (complete)
pmat enforce extreme

# Single file mode - work on worst offender only
pmat enforce extreme --single-file-mode

# Dry run to see what would be changed
pmat enforce extreme --dry-run

# With specific profile
pmat enforce extreme --profile production

# With progress tracking
pmat enforce extreme --show-progress
```

### Agent-Optimized Output

The command returns structured output optimized for AI agents:

```json
{
  "state": "REFACTORING",
  "score": 0.73,
  "target": 1.0,
  "current_file": "src/core/analyzer.rs",
  "violations": [
    {
      "type": "complexity",
      "severity": "high",
      "location": "src/core/analyzer.rs:142-187",
      "current": 25,
      "target": 20,
      "suggestion": "Extract method for nested conditions"
    }
  ],
  "next_action": "refactor_extract_method",
  "progress": {
    "files_completed": 12,
    "files_remaining": 3,
    "estimated_iterations": 4
  }
}
```

## Quality Thresholds

### Default "Extreme" Profile

```toml
[extreme]
coverage.min = 80              # Minimum 80% test coverage
complexity.max = 20            # Maximum cyclomatic complexity
complexity.target = 10         # Target for optimal complexity
tdg.max = 1.0                 # Technical Debt Gradient maximum
satd.allowed = 0              # Zero self-admitted technical debt
duplication.max_lines = 0     # No code duplication
big_o.max = "O(n log n)"      # Maximum algorithmic complexity
provability.min = 0.8         # Minimum provability score
```

## State Machine Specification

### States

1. **ANALYZING**: Initial analysis of codebase
2. **VIOLATING**: Quality violations detected
3. **REFACTORING**: Applying improvements
4. **VALIDATING**: Checking if improvements meet standards
5. **COMPLETE**: All quality standards met (score = 1.0)

### Transitions

- `ANALYZING → VIOLATING`: Violations found
- `ANALYZING → COMPLETE`: No violations (rare on first run)
- `VIOLATING → REFACTORING`: Agent begins refactoring
- `REFACTORING → VALIDATING`: Refactoring complete, checking results
- `VALIDATING → VIOLATING`: Standards not met, continue refactoring
- `VALIDATING → COMPLETE`: All standards met

## Single File Mode

For large codebases with many violations:

```bash
pmat enforce extreme --single-file-mode
```

This mode:
1. Identifies the file with the worst quality score
2. Focuses exclusively on that file
3. Returns success when that single file meets standards
4. Provides clear next target file

### Example Agent Workflow

```bash
# Agent iterates until success
while true; do
  result=$(pmat enforce extreme --single-file-mode --json)
  state=$(echo $result | jq -r '.state')
  
  if [ "$state" = "COMPLETE" ]; then
    echo "File complete, moving to next"
    continue
  fi
  
  # Agent performs suggested refactoring
  # Then validates
done
```

## Exit Codes

- `0`: Quality standards met (state = COMPLETE)
- `1`: Violations remain (state = VIOLATING)
- `2`: Refactoring in progress (state = REFACTORING)
- `3`: Validation failed (state = VALIDATING)
- `127`: Error in analysis

## Agent Integration Examples

### Claude Integration

```python
async def enforce_extreme_quality(project_path):
    """Iteratively improve code quality until extreme standards are met."""
    
    while True:
        # Run enforcement check
        result = await run_command(f"pmat enforce extreme --json")
        
        if result['state'] == 'COMPLETE':
            return {"success": True, "final_score": result['score']}
        
        # Get specific violation details
        violation = result['violations'][0]
        
        # Generate refactoring based on violation type
        if violation['type'] == 'complexity':
            await refactor_complex_method(violation)
        elif violation['type'] == 'coverage':
            await add_tests(violation)
        # ... handle other violation types
        
        # Validate changes
        await run_command("pmat enforce extreme --validate-only")
```

### Batch Processing

```bash
#!/bin/bash
# Process all files one by one
pmat enforce extreme --list-violations | while read -r file; do
  echo "Processing $file"
  pmat enforce extreme --single-file-mode --file "$file"
done
```

## Refactoring Suggestions

The system provides actionable refactoring suggestions:

```json
{
  "refactoring_suggestions": [
    {
      "id": "extract_method",
      "description": "Extract lines 142-156 into separate method 'validate_input'",
      "confidence": 0.95,
      "impact": {
        "complexity": -5,
        "readability": "+15%",
        "testability": "+20%"
      }
    },
    {
      "id": "add_tests",
      "description": "Add unit tests for uncovered branch at line 201",
      "template": "test_edge_case_null_input",
      "confidence": 0.98
    }
  ]
}
```

## Progress Tracking

### Composite Score Calculation

```
XQ Score = (
  coverage_score * 0.30 +
  complexity_score * 0.20 +
  tdg_score * 0.20 +
  provability_score * 0.20 +
  cleanliness_score * 0.10
) * binary_gates

Where binary_gates = 0 if any critical violation exists (SATD, duplication)
```

### Progress Visualization

```
$ pmat enforce extreme --show-progress

🎯 Extreme Quality Enforcement Progress
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Overall Score: 0.73/1.00 ████████████████░░░░ 73%

Coverage:     85/80 ✅  ████████████████████ 100%
Complexity:   15/20 ✅  ████████████████░░░░ 75%
TDG:          1.4/1.0 ❌ ████████████░░░░░░░░ 60%
Provability:  0.7/0.8 ❌ ████████████████░░░░ 87%
SATD:         2/0 ❌    ░░░░░░░░░░░░░░░░░░░░ 0%
Duplication:  0/0 ✅    ████████████████████ 100%

Next Action: Remove SATD in src/utils/parser.rs:45
Estimated Iterations: 3-4
```

## Configuration Options

### .pmat-extreme.toml

```toml
[enforce]
# State machine behavior
max_iterations = 100          # Prevent infinite loops
iteration_delay = 0          # Delay between iterations (ms)
checkpoint_every = 10        # Save progress every N iterations

# Targeting
single_file_mode = false     # Focus on one file at a time
priority = "worst_first"     # Order: worst_first, coverage_first, complexity_first

# Agent optimizations  
json_output = true           # Always output JSON for agents
suggestions = true           # Include refactoring suggestions
explanations = true          # Include detailed explanations

# Thresholds (override defaults)
[enforce.thresholds]
coverage.min = 85
complexity.max = 15
```

## Implementation Notes

### For AI Agents

1. **Idempotency**: Running the command multiple times on compliant code always returns SUCCESS
2. **Deterministic**: Same input always produces same analysis
3. **Incremental**: Each iteration should make measurable progress
4. **Explanatory**: Every suggestion includes rationale

### State Persistence

The system maintains state between runs:

```json
// .pmat-enforce-state.json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "started": "2024-01-15T10:00:00Z",
  "current_state": "REFACTORING",
  "iterations": 12,
  "history": [
    {"iteration": 1, "score": 0.45, "action": "reduce_complexity"},
    {"iteration": 2, "score": 0.52, "action": "add_tests"}
  ]
}
```

## Example Usage Patterns

### 1. Full Codebase Enforcement

```bash
# AI agent runs this in a loop
while [ $(pmat enforce extreme --json | jq -r '.state') != "COMPLETE" ]; do
  pmat enforce extreme --apply-suggestions
done
```

### 2. Incremental Improvement

```bash
# Improve by 10% each day
pmat enforce extreme --target-improvement 0.1 --max-time 3600
```

### 3. CI/CD Integration

```yaml
- name: Enforce Extreme Quality
  run: |
    pmat enforce extreme --ci-mode || {
      echo "::error::Code quality below extreme standards"
      exit 1
    }
```

## Success Criteria

The command returns success (exit code 0) when:

1. Overall XQ Score = 1.0
2. All binary gates pass (no SATD, no duplication)
3. All thresholds are met or exceeded
4. No high-severity violations remain

## Future Extensions

- `--parallel` mode for multi-file processing
- `--auto-commit` for git integration
- `--explain-to-human` for detailed reports
- `--generate-pr` for automated pull requests
- Machine learning for better refactoring suggestions
