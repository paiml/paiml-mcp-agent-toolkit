# Java Language Support in PMAT

**Status:** Complete ✅  
**Version:** 2.172.0  
**Implementation:** Sprint 51  

## Overview

PMAT provides comprehensive support for Java language analysis through its unified AST framework. The Java language support allows PMAT to analyze Java source code, extract AST information, and provide detailed insights into Java codebases.

## Features

The Java language analyzer supports the following Java language features:

- 🔍 **Class Detection**: Analyzes Java classes with visibility and inheritance
- 🧩 **Interface Detection**: Identifies Java interfaces and their methods
- 🔄 **Method Analysis**: Extracts methods with visibility and signatures
- 🏷️ **Annotation Support**: Recognizes Java annotations
- 📦 **Package Structure**: Understands Java package organization
- 🧠 **Complexity Analysis**: Calculates cyclomatic and cognitive complexity

## Implementation Details

The Java language support in PMAT is implemented using the following components:

1. **JavaAstVisitor**: Analyzes Java source code and extracts AST items
   - Located in `server/src/services/languages/java.rs`
   - Uses regex-based parsing for basic Java constructs

2. **JavaStrategy**: Integrates with the unified AST framework
   - Located in `server/src/services/ast/languages/java.rs`
   - Implements the `AstStrategy` trait

3. **Tree-Sitter Integration**: Uses tree-sitter-java for parsing
   - Configured via the `java-ast` feature flag
   - Handles Java syntax accurately

4. **AST Registry**: Automatically registers Java language support
   - Detects .java files based on extension

## Usage

### Enabling Java Language Support

Java language support is included in the `all-languages` and `most-languages` feature flags by default. To enable it explicitly:

```toml
# In Cargo.toml
[features]
default = ["java-ast"]
java-ast = ["tree-sitter", "tree-sitter-java"]
```

### Analyzing Java Code

Java files can be analyzed using the standard PMAT CLI commands:

```bash
# Analyze a Java project
pmat analyze --path /path/to/java/project

# Generate context for Java files
pmat context --path /path/to/java/project --lang java

# Analyze complexity of Java files
pmat complexity --path /path/to/java/project --lang java
```

### Example Output

For a Java file like:

```java
package com.example;

public class Calculator {
    private double result;

    public double add(double x, double y) {
        this.result = x + y;
        return this.result;
    }

    public double multiply(double x, double y) {
        this.result = x * y;
        return this.result;
    }

    public double getResult() {
        return this.result;
    }
}
```

PMAT will generate the following context:

```markdown
### Class: com.example.Calculator (public)
- Method: add (public)
- Method: multiply (public)
- Method: getResult (public)
```

## Integration with Other Languages

The Java language analyzer is part of PMAT's unified AST framework, allowing seamless integration with other supported languages. This is particularly useful for multi-language projects that combine Java with:

- Kotlin (JVM)
- Scala (JVM)
- JavaScript/TypeScript (Web applications)
- Python (Data processing)

## Testing

Comprehensive integration tests for Java language support are available in:
- `server/tests/integration/java_integration.rs`

These tests cover various Java language features:
- Basic class and method detection
- Interface detection
- Inheritance detection
- Annotations detection
- Complex Java constructs

## Future Enhancements

Planned enhancements for Java language support include:

1. **Enhanced AST Extraction**: More detailed AST with generic type parameters
2. **Java Bytecode Analysis**: Support for analyzing compiled .class files
3. **Spring Framework Detection**: Special handling for Spring annotations and patterns
4. **JUnit Test Detection**: Identifying test methods and test classes
5. **Maven/Gradle Integration**: Understanding build file dependencies

## Known Limitations

Current limitations of Java language support:

1. Limited support for advanced Java features like lambdas and method references
2. No direct support for Java generics in the AST
3. Basic complexity calculation (improved version coming in future releases)

## Examples

### Analyzing Java Class Hierarchy

```bash
# Analyze a Java project with inheritance
pmat analyze --path /path/to/java/project --feature class-hierarchy
```

### Finding Complex Java Methods

```bash
# Find complex methods in Java code
pmat complexity --path /path/to/java/project --lang java --threshold 15
```

### Generating Context for Java Spring Applications

```bash
# Generate context optimized for Spring applications
pmat context --path /path/to/spring/project --lang java --feature spring
```

## Conclusion

Java language support in PMAT provides powerful capabilities for analyzing Java codebases. Whether you're working on a pure Java project or a multi-language application, PMAT's Java language analyzer helps you understand your code's structure, complexity, and quality.

---

*Documentation created as part of Sprint 51*