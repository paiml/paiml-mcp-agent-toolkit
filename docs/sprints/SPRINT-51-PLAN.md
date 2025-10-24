# Sprint 51 Planning: JVM Language Expansion

**Date:** October 24, 2025  
**Sprint Duration:** 2 weeks  
**Version Target:** v2.172.0  

## Executive Summary

Sprint 51 will focus on extending PMAT's multi-language support capabilities by implementing full Java language analysis support and laying the groundwork for Scala language support. Building on the successful C/C++ implementation from Sprint 49 and Kotlin implementation from Sprint 50, this sprint will further enhance PMAT's JVM ecosystem support. Additionally, we will create comprehensive integration tests for the recently implemented C/C++/Kotlin language analyzers.

## Sprint Goals

1. **Implement Java Language Analyzer**: Create a comprehensive Java language analyzer that integrates with the unified AST framework, supporting Java classes, interfaces, methods, lambdas, and annotations.

2. **Start Scala Language Support**: Begin the implementation of Scala language support by creating the foundational components and adapters within the unified AST framework.

3. **Improve Test Coverage for C/C++/Kotlin**: Create comprehensive integration tests for the C, C++, and Kotlin language analyzers to ensure reliability and feature completeness.

4. **Documentation and Examples**: Update documentation with Java and Scala analysis examples and improve the existing language support documentation.

## Technical Approach

### 1. Java Language Analyzer Implementation

The Java language analyzer will follow the established pattern from the C/C++/Kotlin implementations:

1. **Strategy Pattern**:
   - Create `JavaStrategy` in `server/src/services/ast/languages/java.rs`
   - Implement the `AstStrategy` trait for Java

2. **AST Visitor**:
   - Create `JavaAstVisitor` to traverse Java source code
   - Implement AST extraction for Java classes, interfaces, methods, and annotations

3. **Tree-Sitter Integration**:
   - Use tree-sitter-java for parsing
   - Add tree-sitter-java as a dependency in Cargo.toml

4. **Feature Flag**:
   - Add `java-ast` feature flag to Cargo.toml
   - Configure conditional compilation in the relevant modules

5. **AST Registry Integration**:
   - Register the Java strategy with the AST registry
   - Enable automatic language detection for Java files

### 2. Scala Language Support Foundation

The Scala language support will begin with these components:

1. **Strategy Skeleton**:
   - Create `ScalaStrategy` in `server/src/services/ast/languages/scala.rs`
   - Implement a basic `AstStrategy` trait for Scala

2. **AST Visitor Framework**:
   - Create `ScalaAstVisitor` with foundational methods
   - Define AST structure for Scala-specific language features (traits, case classes, pattern matching)

3. **Tree-Sitter Integration**:
   - Evaluate tree-sitter-scala options
   - Add the best tree-sitter-scala implementation as a dependency

4. **Feature Flag**:
   - Add `scala-ast` feature flag to Cargo.toml
   - Configure conditional compilation for Scala support

### 3. Integration Tests for C/C++/Kotlin

For each language, we will create comprehensive integration tests:

1. **C/C++ Integration Tests**:
   - Test function, class, and struct detection
   - Test template handling
   - Test C++ specific features (namespaces, classes with inheritance)
   - Test C specific features (structs with typedefs)

2. **Kotlin Integration Tests**:
   - Test class, interface, and method detection
   - Test coroutine support
   - Test Kotlin-specific features (extension functions, data classes)
   - Test interop with Java (JVM annotations)

3. **Test Utilities**:
   - Create shared test utilities for JVM language testing
   - Implement test fixtures for Java/Kotlin/Scala comparison

### 4. Documentation and Examples

Update documentation with:

1. **Java Language Support Guide**:
   - Usage examples for Java analysis
   - Feature coverage details
   - Integration with IDE tools

2. **Scala Language Support (Initial)**:
   - Preliminary documentation for Scala support
   - Roadmap for future Scala feature coverage

3. **JVM Ecosystem Integration**:
   - Cross-language analysis capabilities
   - Shared JVM features detection

## Implementation Plan

### Week 1: Core Java Support and Test Infrastructure

| Day | Tasks |
|-----|-------|
| 1-2 | - Set up Java AST infrastructure<br>- Implement JavaStrategy class<br>- Add tree-sitter-java integration |
| 3-4 | - Implement JavaAstVisitor for core Java features<br>- Set up feature flag and registry integration |
| 5 | - Create comprehensive C/C++ integration tests<br>- Fix any issues discovered |

### Week 2: Scala Foundation and Completion

| Day | Tasks |
|-----|-------|
| 6-7 | - Complete Java language analyzer<br>- Create comprehensive Kotlin integration tests<br>- Start Scala language support foundation |
| 8-9 | - Implement basic ScalaStrategy<br>- Set up Scala AST visitor framework<br>- Create documentation for Java support |
| 10 | - Create initial Scala documentation<br>- Finalize tests and merge all changes<br>- Prepare for v2.172.0 release |

## Technical Requirements

### New Dependencies

1. **Java Support**:
   ```toml
   [dependencies]
   tree-sitter-java = "0.23.5"
   ```

2. **Scala Support** (evaluate options):
   ```toml
   [dependencies]
   tree-sitter-scala = "0.20.0"  # Or best available version
   ```

### Feature Flags

Add the following feature flags to Cargo.toml:

```toml
[features]
default = ["java-ast", "kotlin-ast", "c-ast", "cpp-ast"]  # Add java-ast to default features
java-ast = ["tree-sitter", "tree-sitter-java"]
scala-ast = ["tree-sitter", "tree-sitter-scala"]  # Optional, not in default features yet
```

## Testing Strategy

1. **Unit Tests**:
   - Test each component of the Java language analyzer in isolation
   - Verify AST extraction for different Java language features

2. **Integration Tests**:
   - Test end-to-end language analysis for Java files
   - Test cross-language analysis with Kotlin

3. **Property-Based Tests**:
   - Generate valid Java code snippets
   - Verify consistent AST extraction

4. **Regression Tests**:
   - Ensure existing language analyzers continue to work correctly

## Expected Results

By the end of Sprint 51, we should have:

1. **Fully functional Java language analyzer** integrated with the unified AST framework
2. **Foundation for Scala language support**
3. **Comprehensive integration tests** for C, C++, Kotlin, and Java
4. **Updated documentation** for JVM language support
5. **Readiness for v2.172.0 release**

## Success Metrics

1. **Feature Completeness**:
   - Java analyzer supports all core Java language features
   - Scala foundation handles basic Scala syntax

2. **Test Coverage**:
   - >90% test coverage for Java analyzer
   - Comprehensive integration tests for all language analyzers

3. **Documentation Quality**:
   - Complete, accurate documentation for Java support
   - Clear roadmap for Scala support completion

## Future Work (Sprint 52+)

1. **Complete Scala Language Support**: Finish full Scala language analyzer implementation
2. **Add Support for Groovy**: Extend JVM language support to include Groovy
3. **Enhanced JVM Features**: Implement advanced analysis for JVM bytecode
4. **Cross-Language Refactoring**: Support refactoring across Java/Kotlin/Scala codebases

## Conclusion

Sprint 51 will significantly expand PMAT's JVM language support capabilities, making it a more comprehensive tool for projects using Java, Kotlin, and (in the future) Scala. By following the established patterns from previous language implementations, we can efficiently add these capabilities while maintaining high quality standards.

---

*Document prepared by: Claude Code Agent*  
*Project: PMAT - Pragmatic AI MCP Agent Toolkit*  
*Sprint: 51 - JVM Language Expansion Planning*