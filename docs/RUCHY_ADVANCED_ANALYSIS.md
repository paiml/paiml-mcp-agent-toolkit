# Advanced Ruchy Language Analysis

PMAT now provides comprehensive analysis capabilities for the Ruchy programming language with advanced features beyond basic complexity metrics.

## Features Overview

### 🧮 Halstead Metrics
Full Halstead complexity metrics for Ruchy code:
- **Distinct/Total Operators & Operands**: Accurate counting of unique vs. repeated elements
- **Program Volume**: Information content measurement
- **Difficulty & Effort**: Programming complexity estimation
- **Time & Bugs**: Development time and defect predictions

### 💀 Dead Code Detection
Identifies unused code elements:
- **Unused Functions**: Functions defined but never called (excluding `main` and exported functions)
- **Unused Variables**: Variables declared but never referenced
- **Unreachable Code**: Code paths that cannot be executed

### 🎯 Type Inference Analysis
Basic type inference for Ruchy expressions:
- **Literal Types**: Automatic inference from literals (integers, floats, strings, booleans)
- **Binary Operations**: Result type prediction for arithmetic and logical operations
- **Function Signatures**: Parameter and return type tracking

### 📦 Import/Dependency Analysis
Tracks module dependencies:
- **Import Statements**: All imported modules and items
- **Export Declarations**: Items exported from the current module
- **Dependency Relationships**: Inter-module connections

### 🌟 Advanced Pattern Matching
Enhanced complexity analysis for Ruchy's pattern matching:
- **Match Expression Complexity**: Higher cognitive load for pattern matching
- **Pattern Type Analysis**: Different patterns contribute different complexity scores
- **Exhaustiveness Tracking**: Analysis of match arm coverage

### 🎭 Actor Message Flow Analysis
Specialized analysis for Ruchy's actor model:
- **Actor Detection**: Identification of actor definitions and their components
- **Message Flow Tracking**: `send()` and `spawn()` call analysis
- **Deadlock Detection**: Simple circular dependency detection between actors
- **State Management**: Actor state field tracking

## Usage Examples

### Basic Complexity Analysis
```bash
# Analyze a Ruchy file
pmat analyze complexity --file example.ruchy

# Get detailed JSON output
pmat analyze complexity --file example.ruchy --format json
```

### Advanced Features
The enhanced analysis is automatically applied to all Ruchy files and includes:

#### Halstead Metrics in Output
```json
{
  "name": "complex_function",
  "metrics": {
    "cyclomatic": 8,
    "cognitive": 9,
    "halstead": {
      "n1": 15,           // Distinct operators
      "n2": 12,           // Distinct operands  
      "n1_total": 45,     // Total operators
      "n2_total": 38,     // Total operands
      "volume": 156.3,    // Program volume
      "difficulty": 14.2, // Programming difficulty
      "effort": 2219.5,   // Programming effort
      "time": 123.3,      // Time estimate (hours)
      "bugs": 0.052       // Estimated bugs
    }
  }
}
```

#### Dead Code Detection
The analyzer tracks:
- Function definitions vs. calls
- Variable declarations vs. usage
- Export declarations to exclude from dead code

#### Actor Analysis
For Ruchy actors, the analyzer provides:
- State field tracking
- Message handler identification
- Spawn relationship mapping
- Potential deadlock warnings

## Ruchy Language Features Supported

### Core Language
- ✅ Functions with parameters and return types
- ✅ Variables (let, const, var)
- ✅ Control flow (if/else, while, for, match)
- ✅ Pattern matching with multiple patterns
- ✅ Binary and unary operations
- ✅ Literals (integers, floats, strings, booleans, characters)

### Advanced Features
- ✅ Classes with methods and fields
- ✅ Actors with state and message handlers
- ✅ Traits and implementations
- ✅ Pipeline operators (|>)
- ✅ F-strings and string interpolation
- ✅ Import/export system
- ✅ Error handling (Result types)
- ✅ Option types (Some/None)
- ⚠️ Async/await (partial support)

### Operators Tracked
- **Arithmetic**: +, -, *, /, %
- **Comparison**: ==, !=, <, >, <=, >=
- **Logical**: &&, ||, !
- **Bitwise**: &, |, ^, ~, <<, >>
- **Special**: |> (pipeline), -> (arrow), => (fat arrow), ? (try)

## Configuration

The Ruchy analysis is enabled by default when the `ruchy-ast` feature is included. It automatically detects `.ruchy` files and applies the enhanced analysis.

### Complexity Thresholds
Uses the same thresholds as other languages:
- **Cyclomatic Complexity**: Warning at 10, Error at 20
- **Cognitive Complexity**: Warning at 15, Error at 30
- **Nesting Depth**: Maximum of 5 levels

## Integration with PMAT Tools

### MCP Server Support
All Ruchy analysis features are available through the MCP server:
```bash
# Start MCP server
pmat mcp

# Use with Claude Code for Ruchy analysis
```

### Quality Gates
Ruchy files participate in quality gate analysis:
```bash
# Run quality gate on Ruchy project
pmat quality-gate --file project.ruchy
```

### Context Generation
Ruchy files are included in AI context generation:
```bash
# Generate context including Ruchy files
pmat context
```

## Example Output

For the advanced Ruchy test file, PMAT identifies:
- **7 functions** with varying complexity
- **Complex pattern matching** in `complex_analysis()` (8 cyclomatic, 9 cognitive)
- **Recursive function** `fibonacci()` (3 cyclomatic)
- **Actor definitions** with message handlers
- **Dead code** functions that are never called
- **Import/export** relationships

## Implementation Details

The Ruchy analyzer implements:

1. **RuchyLexer**: Tokenizes Ruchy source code with 35+ token types
2. **RuchyComplexityAnalyzer**: Calculates all metrics with AST traversal
3. **HalsteadMetrics**: Industry-standard complexity measurement
4. **RuchyDeadCode**: Unused code detection with export awareness
5. **RuchyActorAnalysis**: Actor model specific analysis
6. **Type inference**: Basic type system understanding

The implementation follows PMAT's Toyota Way principles with zero-defect tolerance and comprehensive testing.