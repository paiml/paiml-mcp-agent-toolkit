# Scala Language Support in PMAT

**Status:** Beta ✅  
**Version:** 2.172.0  
**Implementation:** Sprint 51  

## Overview

PMAT provides support for Scala language analysis through its unified AST framework. The Scala language support allows PMAT to analyze Scala source code, extract AST information, and provide detailed insights into Scala codebases, complementing the Java and Kotlin language support for complete JVM ecosystem coverage.

## Features

The Scala language analyzer supports the following Scala language features:

- 🔍 **Class Detection**: Analyzes Scala classes with visibility and inheritance
- 🧩 **Trait Detection**: Identifies Scala traits and their methods
- 📦 **Case Class Support**: Special handling for Scala case classes
- 🏗️ **Object Detection**: Identifies Scala objects (singletons)
- 🔄 **Method Analysis**: Extracts methods with visibility and signatures
- 📐 **Pattern Matching**: Detects pattern matching expressions in code
- 🧠 **Complexity Analysis**: Calculates cyclomatic and cognitive complexity

## Implementation Details

The Scala language support in PMAT is implemented using the following components:

1. **ScalaAstVisitor**: Analyzes Scala source code and extracts AST items
   - Located in `server/src/services/languages/scala.rs`
   - Uses regex-based parsing for Scala constructs

2. **ScalaStrategy**: Integrates with the unified AST framework
   - Located in `server/src/services/ast/languages/scala.rs`
   - Implements the `AstStrategy` trait

3. **Tree-Sitter Integration**: Uses tree-sitter-scala for parsing
   - Configured via the `scala-ast` feature flag
   - Handles Scala syntax accurately

4. **AST Registry**: Automatically registers Scala language support
   - Detects .scala and .sc files based on extension

## Usage

### Enabling Scala Language Support

Scala language support is included in the `all-languages` and `most-languages` feature flags. To enable it explicitly:

```toml
# In Cargo.toml
[features]
default = ["scala-ast"]
scala-ast = ["tree-sitter", "tree-sitter-scala"]
```

### Analyzing Scala Code

Scala files can be analyzed using the standard PMAT CLI commands:

```bash
# Analyze a Scala project
pmat analyze --path /path/to/scala/project

# Generate context for Scala files
pmat context --path /path/to/scala/project --lang scala

# Analyze complexity of Scala files
pmat complexity --path /path/to/scala/project --lang scala
```

### Example Output

For a Scala file like:

```scala
package com.example.models

case class Person(name: String, age: Int) {
  def isAdult: Boolean = age >= 18
}

object Person {
  def apply(name: String): Person = new Person(name, 0)
}
```

PMAT will generate the following context:

```markdown
### Case Class: com.example.models::Person (public)
- Method: isAdult (public)

### Object: com.example.models::Person (public)
- Method: apply (public)
```

## Integration with Other JVM Languages

The Scala language analyzer is part of PMAT's unified AST framework, allowing seamless integration with other JVM languages:

- Java
- Kotlin
- Groovy (future)

This integration is particularly useful for polyglot JVM projects that combine multiple languages.

## Scala-Specific Features

### Case Class Detection

Scala case classes are detected and represented as structs with a "case" derive attribute, enabling special handling in tools and reports.

### Companion Objects

Scala companion objects are detected and represented as modules, with their relationship to case classes and regular classes preserved.

### Pattern Matching

Scala's pattern matching expressions are detected and contribute to complexity calculations, providing accurate complexity metrics for functional Scala code.

### Traits as Interfaces

Scala traits are detected and represented as traits in the AST, similar to Java interfaces but with Scala-specific semantics.

## Testing

Comprehensive integration tests for Scala language support are available in:
- `server/tests/integration/scala_integration.rs`

These tests cover various Scala language features:
- Basic class and method detection
- Case class detection
- Trait detection
- Object (singleton) detection
- Complex Scala constructs

## Current Limitations

As this is a beta implementation, there are some limitations:

1. Limited support for advanced Scala features:
   - Partial functions
   - Type classes
   - Implicit parameters and conversions
   - Higher-kinded types

2. Basic complexity calculation for functional constructs:
   - Improved complexity metrics for functional code coming in future releases

3. No direct integration with Scala build tools (sbt, Mill, etc.):
   - Project structure detection limited to file-based analysis

## Future Enhancements

Planned enhancements for Scala language support include:

1. **Enhanced AST Extraction**: More detailed AST with higher-kinded type parameters
2. **SBT Integration**: Understanding build.sbt files and project structure
3. **Akka/Play Framework Detection**: Special handling for Akka actors and Play controllers
4. **ScalaTest Integration**: Identifying test methods and test classes
5. **Functional Programming Metrics**: Special complexity metrics for functional code

## Examples

### Analyzing Scala Class Hierarchy

```bash
# Analyze a Scala project with traits and inheritance
pmat analyze --path /path/to/scala/project --feature class-hierarchy
```

### Finding Complex Scala Methods

```bash
# Find complex methods in Scala code
pmat complexity --path /path/to/scala/project --lang scala --threshold 15
```

### Generating Context for Scala Applications

```bash
# Generate context optimized for Scala applications
pmat context --path /path/to/scala/project --lang scala
```

## Conclusion

Scala language support in PMAT provides powerful capabilities for analyzing Scala codebases. Together with Java and Kotlin support, PMAT offers comprehensive analysis for the entire JVM ecosystem, helping you understand your code's structure, complexity, and quality across multiple languages.

---

*Documentation created as part of Sprint 51*