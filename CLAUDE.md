# CLAUDE.md

## System Architecture Overview

This document serves as the operational guide for the paiml-mcp-agent-toolkit (pmat), a unified protocol implementation supporting CLI, MCP, and HTTP interfaces through a single binary architecture.

**Core Design Principle**: Protocol-agnostic service layer with deterministic behavior across all interfaces.
- Jidoka (自働化): Build quality in through proper error handling and verification (Never use TODO or leave unfinished code)
- Genchi Genbutsu (現地現物): Go and see the actual root causes instead of statistical approximations
- Hansei (反省): Focus on fixing existing broken functionality rather than adding new features
- Kaizen (改善): Continuous incremental improvement, especially through single file mode refactoring

## Operational Guidelines

### ABSOLUTE RULES
- NEVER work out of the server directory

### CRITICAL TOOLS FOR QUALITY ENFORCEMENT

#### Single File Mode (Toyota Way - Kaizen)
Single file mode is CRITICAL for incremental quality improvements. These three commands work together:

1. **pmat refactor auto --single-file-mode --file <path>**
   - Targets individual files for AI-powered refactoring
   - Achieves extreme quality standards incrementally
   - Use when: A file has high complexity, SATD, or lint violations

2. **pmat lint-hotspot --file <path>**
   - Analyzes single file for all quality violations
   - Returns JSON format for parsing by refactor auto
   - Use when: Need to understand specific file's issues

3. **pmat enforce extreme --file <path>**
   - Verifies single file meets all quality standards
   - Exit code 0 only if file passes all checks
   - Use when: Validating refactoring success

**IMPORTANT**: Always use single file mode for refactoring to follow Toyota Way principles of incremental improvement. Never attempt to refactor entire codebase at once.

### Quality Standards (Zero Tolerance)
- Cyclomatic Complexity: Max 20 (target 5)
- Test Coverage: Min 80% per file
- SATD: Zero (no TODO, FIXME, HACK comments)
- Lint: All clippy pedantic + nursery must pass

### Workflow for Fixing Quality Violations
1. Run `pmat lint-hotspot` to find worst file
2. Use `pmat refactor auto --single-file-mode --file <worst-file>`
3. Verify with `pmat enforce extreme --file <worst-file>`
4. Commit changes
5. Repeat for next worst file

### Running Quality Checks
After making any code changes, ALWAYS run:
```bash
make lint        # Extreme clippy standards
make typecheck   # Type checking
```

If these commands are not found, ask user for the correct commands and update this document.