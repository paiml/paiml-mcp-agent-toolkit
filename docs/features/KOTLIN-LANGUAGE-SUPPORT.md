# Kotlin Language Support in PMAT

## Overview

Kotlin language support in PMAT provides comprehensive analysis capabilities for Kotlin code, following the same unified AST architecture used for other supported languages. This feature enables static analysis, complexity detection, and quality metrics for Kotlin codebases.

The implementation leverages tree-sitter-kotlin-ng for accurate parsing and builds on the unified AST framework established in Sprint 49.

## Features

- **Kotlin Code Analysis**: Parse and analyze `.kt` and `.kts` files
- **AST Item Detection**:
  - Classes and data classes
  - Interfaces
  - Functions and methods
  - Coroutines (suspend functions)
  - Enums
- **Coroutine Analysis**: Special handling for Kotlin's suspend functions
- **Package-Aware**: Maintains proper package qualification in symbol names
- **Extension Methods**: Detects Kotlin extension methods
- **DSL Support**: Basic support for Kotlin DSL constructs

## Usage

Enable Kotlin language support by adding the `kotlin-ast` feature:

```shell
# CLI Usage
pmat analyze --include "*.kt" --features kotlin-ast /path/to/kotlin/project

# Deep context generation
pmat context --output kotlin_context.md --include "*.kt" --features kotlin-ast /path/to/kotlin/project
```

## Architecture

The Kotlin language support follows the unified AST framework established in Sprint 49 and is organized as follows:

1. **Core Components**:
   - `KotlinAstVisitor`: Extracts AST items from Kotlin source code
   - `KotlinAstParser`: Converts Kotlin code to unified AST representation
   - `KotlinStrategy`: Adapter implementing the `AstStrategy` trait for the unified AST framework

2. **Integration Points**:
   - AST Registry integration via feature flag
   - Unified AST representation for consistent analysis
   - Tree-sitter parser for Kotlin syntax analysis

## Example

```kotlin
// Example Kotlin code that can be analyzed
package com.example.demo

data class User(val name: String, val age: Int)

interface Repository<T> {
    fun save(item: T): Boolean
    fun findById(id: Int): T?
}

class UserRepository : Repository<User> {
    private val users = mutableListOf<User>()
    
    override fun save(item: User): Boolean {
        users.add(item)
        return true
    }
    
    override fun findById(id: Int): User? {
        return users.find { it.id == id }
    }
}

suspend fun fetchUserData(id: Int): User {
    delay(100) // Simulate network delay
    return User("John", 30)
}
```

## Technical Implementation

The Kotlin language support is implemented across multiple components:

1. **AST Visitor** (`KotlinAstVisitor`):
   - Analyzes Kotlin source code
   - Extracts function declarations, class declarations, interface declarations
   - Provides special handling for coroutines (suspend functions)
   - Located in `server/src/services/languages/kotlin.rs`

2. **AST Parser** (`KotlinAstParser`):
   - Converts Kotlin code to unified AST representation
   - Handles various Kotlin-specific constructs
   - Located in `server/src/services/ast_kotlin.rs`

3. **AST Strategy** (`KotlinStrategy`):
   - Implements the `AstStrategy` trait for the unified AST framework
   - Provides adapter pattern integration with the AST registry
   - Located in `server/src/services/ast/languages/kotlin.rs`

4. **Tree-Sitter Integration**:
   - Uses tree-sitter-kotlin-ng for accurate parsing
   - Processes Abstract Syntax Tree for complexity metrics

## Testing

Comprehensive tests are available to verify Kotlin language support:

1. **Unit Tests**:
   - Basic parsing tests for Kotlin constructs
   - Located in `server/src/services/languages/kotlin.rs`

2. **Integration Tests**:
   - End-to-end tests for Kotlin file analysis
   - Verifies the complete AST pipeline
   - Located in `server/tests/integration/kotlin_integration.rs`

3. **Property Tests**:
   - Randomized testing with Proptest
   - Tests robustness with varied inputs

## Configuration

Kotlin language support can be controlled via the following configuration options:

1. **Feature Flag**:
   - Enable via the `kotlin-ast` feature flag
   - Add to Cargo.toml features list

2. **Include Patterns**:
   - Use `--include "*.kt"` or `--include "*.kts"` to target Kotlin files

## Examples

### CLI Analysis

```bash
# Analyze a Kotlin project
pmat analyze --include "*.kt" --features kotlin-ast /path/to/kotlin/project

# Generate complexity metrics
pmat complexity --include "*.kt" --features kotlin-ast /path/to/kotlin/project
```

### Context Generation

```bash
# Generate deep context for a Kotlin project
pmat context --output kotlin_context.md --include "*.kt" --features kotlin-ast /path/to/kotlin/project
```

## Status and Future Work

Kotlin language support is now complete as part of Sprint 50. Future enhancements may include:

- Advanced coroutine flow analysis
- Kotlin multiplatform project support
- Enhanced Kotlin DSL analysis
- Integration with Kotlin compiler features
- Kotlin scripting extensions

## Related Documentation

- [AST Consolidation Plan](../architecture/ast-consolidation-plan.md)
- [AST Migration Plan](../architecture/ast-migration-plan.md)
- [Sprint 49 Summary](../architecture/sprint-49-summary.md)