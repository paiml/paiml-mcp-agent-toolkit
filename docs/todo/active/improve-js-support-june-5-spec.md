# Specification: Improved JavaScript, TypeScript, and Deno AST Analysis for `pmat`

**Status:** Proposed
**Author:** AI Assistant
**Date:** 2025-06-06
**Version:** 1.0

## 1. Overview

This document outlines the necessary improvements to elevate the JavaScript, TypeScript, and Deno analysis capabilities of the `pmat` toolkit to achieve feature parity with the existing Rust toolchain. The current implementation appears to perform a superficial scan for these languages, failing to extract meaningful symbolic information.

The primary goal is to integrate a high-performance Abstract Syntax Tree (AST) parser for the JS/TS ecosystem to enable deep, accurate, and deterministic analysis of code structure, complexity, and dependencies.

## 2. Current State & Limitations

The provided `deep_context.md` report reveals critical limitations in the current JS/TS analysis:

1.  **No Symbol Extraction:** For all `.ts` and `.js` files listed under "Enhanced AST Analysis", the metrics are `Total Symbols: 0`, `Functions: 0`, `Structs: 0`, etc. This indicates that **no actual AST parsing is occurring**.
2.  **Inaccurate Complexity Metrics:** Without an AST, any complexity scores for these languages are likely derived from inaccurate line-of-code heuristics or simple regex, not formal definitions like McCabe or Cognitive Complexity.
3.  **Superficial Dependency Analysis:** The current dependency graph (`pmat analyze dag`) for TS/JS projects likely relies on file system structure rather than parsing `import`/`export`/`require` statements, making it imprecise.
4.  **No Dead Code Analysis:** True dead code analysis is impossible without a complete call graph derived from a full AST.

## 3. Proposed Architecture & Technical Approach

To address these limitations, we will integrate a proper AST parser into the Rust-based `pmat` core.

### 3.1. Core Technology: SWC (Speedy Web Compiler)

We will use the **`swc`** library (`swc_ecma_parser`). It is the ideal choice because:
- It is written in Rust, ensuring seamless and high-performance integration.
- It is the same battle-tested parser used by Deno, Next.js, and other major tools.
- It has robust support for the entire modern JavaScript/TypeScript feature set, including JSX/TSX, decorators, and various module systems (ESM, CommonJS).

### 3.2. Integration Plan

1.  **Create a New Toolchain Module:**
    - A new module, `toolchain/src/toolchains/typescript.rs`, will be enhanced to implement the `SymbolicAnalyzer` trait fully.

2.  **Implement AST Parsing:**
    - Use `swc_ecma_parser` to parse `.js`, `.jsx`, `.ts`, and `.tsx` files into a `swc_ecma_ast::Module`.
    - Implement a visitor pattern to traverse the AST and extract symbolic information.

3.  **Define Symbolic Structures:**
    - Create Rust structs to represent extracted symbols, such as:
      ```rust
      struct TypeScriptSymbol {
          name: String,
          kind: SymbolKind, // Enum: Function, Class, Interface, TypeAlias, Variable
          span: (u32, u32), // Line numbers
          is_exported: bool,
          // ... other metadata
      }
      ```

4.  **Enhance Analysis Features:**
    - **Complexity:** Traverse the AST to accurately calculate McCabe and Cognitive complexity for functions and methods.
    - **Dependency Graph (DAG):** Analyze `ImportDeclaration`, `ExportDeclaration`, and `require()` calls to build a precise module-level dependency graph.
    - **Dead Code:** Use the new dependency graph and symbol information to perform reachability analysis and identify un-exported, unused symbols with high confidence.

### 3.3. Deno Support

Since Deno is built on the same V8/Rust/SWC stack, the TypeScript analyzer will cover the majority of Deno's language features. Specific Deno APIs and runtime-specific features (`Deno.*`) can be recognized as well-known symbols during the AST traversal. The auto-detection logic should be updated to recognize `deno.json` or `deno.jsonc` files as strong indicators of a Deno project.

## 4. Acceptance Criteria

1.  **AST Analysis Populated:** Running `pmat analyze deep-context` on a TypeScript/JavaScript project must produce a report where `.ts`/`.js` files show non-zero counts for **Functions**, **Imports**, and other relevant symbols.
2.  **Accurate Complexity:** `pmat analyze complexity` must compute and display McCabe and Cognitive complexity scores for functions within TS/JS files.
3.  **Precise Dependency Graph:** `pmat analyze dag` must generate a Mermaid diagram that accurately reflects the `import`/`export` relationships between modules.
4.  **Functional Dead Code Analysis:** `pmat analyze dead-code` must be able to identify and report unused exported functions, classes, and variables in a TS/JS project.
5.  **Framework Compatibility:** The parser must successfully handle syntax from modern frameworks, including React (JSX/TSX) and decorators.

## 5. Out of Scope (For This Iteration)

*   **Full Type Checking:** This would require running the entire TypeScript compiler (TSC), which is slow and complex. Analysis will be based on syntax (AST), not the full type system.
*   **Dynamic `require()` Analysis:** Analysis of dynamically generated paths in `require()` calls (e.g., `require('./lib/' + moduleName)`) will not be supported.
*   **Support for Svelte/Vue:** Single-file components for frameworks other than React are out of scope for this initial implementation.