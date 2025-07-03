# Gemini Agent Project Guide: paiml-mcp-agent-toolkit (pmat)

This document provides a summary of the `pmat` project for the Gemini agent.

## Project Overview

`pmat` is a **zero-configuration AI context generation system** designed to instantly analyze any codebase. It provides deep context analysis, refactoring tools, and quality gates through CLI, MCP, and HTTP interfaces. The project is built with extreme quality standards and has a zero-tolerance policy for technical debt.

## Key Features

- **Code Analysis:** AST-based analysis, complexity metrics (cyclomatic, cognitive), dead code detection, technical debt quantification (TDG), and SATD (Self-Admitted Technical Debt) detection.
- **Refactoring Tools:** AI-powered auto-refactoring, documentation cleanup, and interactive refactoring.
- **Quality Gates:** Lint hotspot analysis, formal verification, defect prediction, and CI/CD integration.
- **Language Support:** Extensive support for Rust, TypeScript/JavaScript, Python, Kotlin, C/C++, and WebAssembly.

## Core CLI Commands

### Analysis
- `pmat context`: Generate project context (auto-detects language).
- `pmat analyze complexity --top-files 10`: Analyze code complexity.
- `pmat analyze dead-code`: Find unused code.
- `pmat analyze satd`: Find self-admitted technical debt in comments.
- `pmat analyze churn --days 30`: Analyze Git history.
- `pmat analyze dag`: Generate a dependency graph.
- `pmat quality-gate --strict`: Run comprehensive quality checks.

### Refactoring
- `pmat refactor auto`: AI-powered automatic refactoring.
- `pmat refactor docs`: Clean up documentation and temporary files.
- `pmat refactor interactive`: Step-by-step guided refactoring.

## Development Workflow

- **Build:** `cargo build --release`
- **Testing:**
    - `make test-fast`: Run quick unit and service tests.
    - `make test-all`: Run the complete test suite.
    - `make coverage`: Generate a coverage report.
- **Linting:** `make lint` (uses extreme clippy standards: pedantic, nursery).

## Quality Standards

The project adheres to **Zero Tolerance Quality Standards**:
- **ZERO SATD**: No `TODO`, `FIXME`, `HACK` comments.
- **ZERO High Complexity**: No function's cyclomatic complexity should exceed 20.
- **ZERO Known Defects**: All code must be fully functional.
- **ZERO Incomplete Features**: Only complete, tested features are merged.
