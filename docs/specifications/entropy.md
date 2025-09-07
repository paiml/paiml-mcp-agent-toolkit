# Code Similarity and Entropy Detection Specification

## Overview

This specification defines a comprehensive code similarity detection system that identifies both exact duplicates and semantically similar code patterns using entropy analysis, structural comparison, and established computer science algorithms.

## Goals

1. **Exact Duplicate Detection**: Find identical code blocks across the codebase
2. **Structural Similarity**: Identify code with similar AST structure but different names/literals
3. **Semantic Similarity**: Find code that performs similar operations despite syntactic differences
4. **Pattern Detection**: Identify sloppy patterns that should be refactored
5. **Cross-Language Support**: Work across Rust, TypeScript, Python, and other languages

## Algorithms and Techniques

### 1. Winnowing Algorithm (Fingerprinting)
- **Purpose**: Efficient substring matching for near-duplicates
- **Method**: Rolling hash with guaranteed detection of matching substrings
- **Parameters**:
  - Window size: 40 tokens (configurable)
  - K-gram size: 15 tokens (configurable)
- **Complexity**: O(n) where n is the number of tokens

### 2. Shannon Entropy Analysis
- **Purpose**: Measure code complexity and randomness
- **Formula**: H(X) = -Σ p(xi) * log2(p(xi))
- **Use Cases**:
  - Identify overly complex code (high entropy)
  - Find repetitive patterns (low entropy)
  - Detect copy-paste with minor modifications

### 3. AST-Based Structural Similarity
- **Purpose**: Find structurally similar code regardless of naming
- **Method**: 
  - Parse code into AST
  - Normalize identifiers and literals
  - Compare tree structure using tree edit distance
- **Similarity Metric**: 1 - (edit_distance / max_tree_size)

### 4. Token-Based Semantic Similarity
- **Purpose**: Find semantically similar code blocks
- **Methods**:
  - **TF-IDF**: Weight tokens by frequency and inverse document frequency
  - **Cosine Similarity**: Measure angle between token vectors
  - **Jaccard Index**: Set-based similarity measurement
- **Normalization**: Remove comments, normalize whitespace, abstract identifiers

### 5. Fuzzy Matching
- **Purpose**: Find similar but not identical code
- **Methods**:
  - **Levenshtein Distance**: Character-level edit distance
  - **Token Edit Distance**: Token-level modifications
  - **N-gram Similarity**: Overlapping subsequence matching

## Detection Levels

### Level 1: Exact Duplicates (Type-1 Clones)
- **Definition**: Identical code except for whitespace and comments
- **Threshold**: 100% similarity
- **Algorithm**: Hash-based exact matching

### Level 2: Renamed Duplicates (Type-2 Clones)
- **Definition**: Identical structure with renamed variables/functions
- **Threshold**: >90% structural similarity
- **Algorithm**: AST normalization + hashing

### Level 3: Modified Duplicates (Type-3 Clones)
- **Definition**: Similar code with added/deleted/modified statements
- **Threshold**: >70% similarity
- **Algorithm**: Winnowing + token similarity

### Level 4: Semantic Duplicates (Type-4 Clones)
- **Definition**: Different syntax but same semantics
- **Threshold**: >60% semantic similarity
- **Algorithm**: AST pattern matching + data flow analysis

## Entropy-Based Pattern Detection

### High Entropy Patterns (Complex/Messy)
- Nested conditionals with mixed logic
- Long parameter lists with unclear purpose
- Mixed abstraction levels in single function
- **Action**: Suggest decomposition and simplification

### Low Entropy Patterns (Repetitive)
- Copy-pasted code with minor variations
- Repeated if-else chains
- Similar function implementations
- **Action**: Suggest extraction and parameterization

### Medium Entropy Anti-Patterns
- Similar but inconsistent error handling
- Repeated boilerplate with variations
- Almost-identical data structures
- **Action**: Suggest standardization and abstraction

## Implementation Requirements

### Performance Targets
- Process 100K LOC in <5 seconds
- Memory usage <500MB for 1M LOC
- Incremental analysis support
- Parallel processing for large codebases

### Quality Requirements
- All functions must have cyclomatic complexity ≤10
- 100% test coverage for core algorithms
- Property-based testing for edge cases
- Integration tests for CLI and MCP

### Output Formats

#### JSON Output
```json
{
  "exact_duplicates": [...],
  "structural_similarities": [...],
  "semantic_similarities": [...],
  "entropy_analysis": {
    "high_entropy_blocks": [...],
    "low_entropy_patterns": [...],
    "refactoring_opportunities": [...]
  },
  "metrics": {
    "duplication_percentage": 12.5,
    "average_entropy": 3.2,
    "total_clones": 45
  }
}
```

#### Human-Readable Report
```
Code Similarity Analysis Report
================================

Exact Duplicates: 12 blocks (450 lines)
Similar Patterns: 28 instances
Refactoring Opportunities: 15

Top Issues:
1. Duplicate error handling in auth module (8 instances)
2. Similar data validation logic across controllers
3. Repeated database query patterns

Recommendations:
- Extract common error handling to shared utility
- Create validation middleware
- Implement query builder pattern
```

## API Design

### Core Interface
```rust
pub trait SimilarityDetector {
    fn detect_exact_duplicates(&self, files: &[File]) -> Vec<DuplicateBlock>;
    fn detect_structural_similarity(&self, files: &[File], threshold: f64) -> Vec<SimilarBlock>;
    fn detect_semantic_similarity(&self, files: &[File], threshold: f64) -> Vec<SimilarBlock>;
    fn analyze_entropy(&self, files: &[File]) -> EntropyReport;
    fn find_refactoring_opportunities(&self, files: &[File]) -> Vec<RefactoringHint>;
}
```

### Configuration
```rust
pub struct SimilarityConfig {
    pub min_lines: usize,           // Minimum block size (default: 6)
    pub min_tokens: usize,          // Minimum token count (default: 50)
    pub similarity_threshold: f64,   // Similarity threshold (default: 0.7)
    pub enable_entropy: bool,        // Enable entropy analysis
    pub enable_ast: bool,           // Enable AST analysis
    pub enable_semantic: bool,       // Enable semantic analysis
    pub window_size: usize,         // Winnowing window size
    pub k_gram_size: usize,         // K-gram size for fingerprinting
}
```

## Testing Strategy

### Unit Tests
- Test each algorithm independently
- Test entropy calculations
- Test similarity metrics
- Test normalization functions

### Integration Tests
- CLI command testing
- MCP tool testing
- Cross-file detection
- Performance benchmarks

### Property-Based Tests
- Random code generation
- Mutation testing
- Invariant checking
- Threshold validation

### Example Tests
```rust
#[test]
fn test_exact_duplicate_detection() {
    let code1 = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let code2 = "fn add(a: i32, b: i32) -> i32 { a + b }";
    assert_eq!(detector.similarity(code1, code2), 1.0);
}

#[test]
fn test_structural_similarity() {
    let code1 = "fn add(x: i32, y: i32) -> i32 { x + y }";
    let code2 = "fn sum(a: i32, b: i32) -> i32 { a + b }";
    assert!(detector.structural_similarity(code1, code2) > 0.9);
}

#[test]
fn test_entropy_calculation() {
    let repetitive = "if x { y } if x { y } if x { y }";
    let complex = "match x { A => b, C(d) => e.f()?, _ => g }";
    assert!(entropy(repetitive) < entropy(complex));
}
```

## Success Criteria

1. **Accuracy**: >95% precision in duplicate detection
2. **Performance**: <5s for 100K LOC analysis
3. **Usability**: Clear, actionable recommendations
4. **Coverage**: Support for Rust, TypeScript, Python
5. **Quality**: Zero functions with complexity >10
6. **Testing**: 100% coverage of core algorithms
7. **Integration**: Working CLI and MCP interfaces

## References

- Schleimer, S., Wilkerson, D. S., & Aiken, A. (2003). Winnowing: local algorithms for document fingerprinting.
- Shannon, C. E. (1948). A mathematical theory of communication.
- Kamiya, T., & Kusumoto, S. (2002). CCFinder: a multilinguistic token-based code clone detection system.
- Roy, C. K., & Cordy, J. R. (2007). A survey on software clone detection research.